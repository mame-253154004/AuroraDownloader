use std::path::Path;
use std::time::Duration;

use chrono::Utc;
use futures_util::StreamExt;
use reqwest::redirect::Policy;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use crate::download::path::safe_target_path;
use crate::models::{
    DownloadError, DownloadId, DownloadProgress, DownloadRecord, DownloadRequest, DownloadStatus,
};
use crate::security::url_policy::UrlPolicy;

#[derive(Debug, Clone)]
pub struct DownloaderConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub max_download_size_bytes: u64,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(120),
            max_retries: 2,
            retry_delay: Duration::from_secs(2),
            max_download_size_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started {
        id: DownloadId,
        total_bytes: Option<u64>,
    },
    Progress(DownloadProgress),
    Retrying {
        id: DownloadId,
        attempt: u32,
        reason: String,
    },
    Completed {
        id: DownloadId,
        destination_path: String,
    },
    Cancelled {
        id: DownloadId,
    },
    Failed {
        id: DownloadId,
        error: DownloadError,
    },
}

pub trait DownloadEventSink: Send + Sync {
    fn on_event(&self, event: DownloadEvent);
}

/// Placeholder interface for pause/resume support.
/// TODO: Persist resume metadata (etag/range map), then implement semantics safely.
pub trait PauseResumeApi {
    fn pause(&self, _id: &DownloadId) -> Result<(), DownloadError> {
        Err(DownloadError::Unsupported {
            reason: "pause is planned but not implemented in this prototype".to_string(),
        })
    }

    fn resume(&self, _id: &DownloadId) -> Result<(), DownloadError> {
        Err(DownloadError::Unsupported {
            reason: "resume is planned but not implemented in this prototype".to_string(),
        })
    }
}

pub async fn download_http(
    request: &DownloadRequest,
    destination_dir: &Path,
    policy: &UrlPolicy,
    config: &DownloaderConfig,
    sink: Option<&dyn DownloadEventSink>,
    cancellation_token: CancellationToken,
) -> Result<DownloadRecord, DownloadError> {
    let mut attempts = 0;
    let mut last_err = None;

    while attempts <= config.max_retries {
        match download_once(
            request,
            destination_dir,
            policy,
            config,
            sink,
            cancellation_token.child_token(),
        )
        .await
        {
            Ok(record) => return Ok(record),
            Err(err) if should_retry(&err) && attempts < config.max_retries => {
                attempts += 1;
                if let Some(sink) = sink {
                    sink.on_event(DownloadEvent::Retrying {
                        id: request.id.clone(),
                        attempt: attempts,
                        reason: format!("{err:?}"),
                    });
                }
                tokio::time::sleep(config.retry_delay).await;
                last_err = Some(err);
            }
            Err(err) => {
                if let Some(sink) = sink {
                    sink.on_event(DownloadEvent::Failed {
                        id: request.id.clone(),
                        error: err.clone(),
                    });
                }
                return Err(err);
            }
        }
    }

    let last = last_err.unwrap_or(DownloadError::Internal {
        reason: "retry loop exhausted without explicit error".to_string(),
    });

    Err(DownloadError::RetryExhausted {
        attempts,
        last_error: format!("{last:?}"),
    })
}

fn should_retry(error: &DownloadError) -> bool {
    matches!(
        error,
        DownloadError::Network { .. } | DownloadError::Io { .. } | DownloadError::Timeout
    )
}

async fn download_once(
    request: &DownloadRequest,
    destination_dir: &Path,
    policy: &UrlPolicy,
    config: &DownloaderConfig,
    sink: Option<&dyn DownloadEventSink>,
    cancellation_token: CancellationToken,
) -> Result<DownloadRecord, DownloadError> {
    let url = policy.validate(&request.url)?;

    let redirect_policy = Policy::limited(policy.config().max_redirects);
    let client = reqwest::Client::builder()
        .redirect(redirect_policy)
        .timeout(config.timeout)
        .build()
        .map_err(|e| DownloadError::Internal {
            reason: format!("failed to build http client: {e}"),
        })?;

    tokio::fs::create_dir_all(destination_dir)
        .await
        .map_err(|e| DownloadError::Io {
            reason: e.to_string(),
        })?;

    let target_path = safe_target_path(destination_dir, &request.requested_file_name)?;
    let part_path = target_path.with_extension("part");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| DownloadError::Network {
            reason: e.to_string(),
        })?
        .error_for_status()
        .map_err(|e| DownloadError::Network {
            reason: e.to_string(),
        })?;

    let total_bytes = response.content_length();
    let max_allowed = request.max_bytes.unwrap_or(
        config
            .max_download_size_bytes
            .min(policy.config().max_download_size_bytes),
    );

    if let Some(total) = total_bytes {
        if total > max_allowed {
            return Err(DownloadError::SizeLimitExceeded {
                max_bytes: max_allowed,
            });
        }
    }

    if let Some(sink) = sink {
        sink.on_event(DownloadEvent::Started {
            id: request.id.clone(),
            total_bytes,
        });
    }

    let mut part_file =
        tokio::fs::File::create(&part_path)
            .await
            .map_err(|e| DownloadError::Io {
                reason: e.to_string(),
            })?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(next_chunk) = stream.next().await {
        if cancellation_token.is_cancelled() {
            let _ = tokio::fs::remove_file(&part_path).await;
            if let Some(sink) = sink {
                sink.on_event(DownloadEvent::Cancelled {
                    id: request.id.clone(),
                });
            }
            return Err(DownloadError::Cancelled);
        }

        let chunk = next_chunk.map_err(|e| DownloadError::Network {
            reason: e.to_string(),
        })?;

        downloaded = downloaded.saturating_add(chunk.len() as u64);
        if downloaded > max_allowed {
            let _ = tokio::fs::remove_file(&part_path).await;
            return Err(DownloadError::SizeLimitExceeded {
                max_bytes: max_allowed,
            });
        }

        part_file
            .write_all(&chunk)
            .await
            .map_err(|e| DownloadError::Io {
                reason: e.to_string(),
            })?;

        if let Some(sink) = sink {
            sink.on_event(DownloadEvent::Progress(DownloadProgress {
                id: request.id.clone(),
                downloaded_bytes: downloaded,
                total_bytes,
                status: DownloadStatus::Running,
            }));
        }
    }

    part_file.flush().await.map_err(|e| DownloadError::Io {
        reason: e.to_string(),
    })?;

    tokio::fs::rename(&part_path, &target_path)
        .await
        .map_err(|e| DownloadError::Io {
            reason: e.to_string(),
        })?;

    if let Some(sink) = sink {
        sink.on_event(DownloadEvent::Completed {
            id: request.id.clone(),
            destination_path: target_path.display().to_string(),
        });
    }

    Ok(DownloadRecord {
        id: request.id.clone(),
        url: request.url.clone(),
        destination_path: target_path.display().to_string(),
        status: DownloadStatus::Completed,
        downloaded_bytes: downloaded,
        total_bytes,
        error: None,
        created_at: request.created_at,
        started_at: Some(Utc::now()),
        completed_at: Some(Utc::now()),
    })
}
