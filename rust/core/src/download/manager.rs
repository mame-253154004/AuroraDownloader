use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use tokio_util::sync::CancellationToken;

use crate::download::engine::{download_http, DownloadEventSink, DownloaderConfig};
use crate::download::path::safe_target_path;
use crate::models::{DownloadError, DownloadId, DownloadRecord, DownloadRequest, DownloadStatus};
use crate::security::url_policy::UrlPolicy;
use crate::storage::DownloadRepository;

#[derive(Clone)]
pub struct DownloadManager {
    repository: Arc<dyn DownloadRepository>,
    policy: UrlPolicy,
    config: DownloaderConfig,
    sink: Option<Arc<dyn DownloadEventSink>>,
    active: Arc<Mutex<HashMap<DownloadId, CancellationToken>>>,
    cancellation_requested: Arc<Mutex<HashSet<DownloadId>>>,
}

impl DownloadManager {
    pub fn new(
        repository: Arc<dyn DownloadRepository>,
        policy: UrlPolicy,
        config: DownloaderConfig,
    ) -> Self {
        Self {
            repository,
            policy,
            config,
            sink: None,
            active: Arc::new(Mutex::new(HashMap::new())),
            cancellation_requested: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn with_sink(mut self, sink: Arc<dyn DownloadEventSink>) -> Self {
        self.sink = Some(sink);
        self
    }

    pub fn enqueue(&self, request: DownloadRequest) -> Result<(), DownloadError> {
        self.policy.validate(&request.url)?;
        if self.repository.get(&request.id).is_some() {
            return Err(DownloadError::Internal {
                reason: "download id already exists".to_string(),
            });
        }

        let destination_path = safe_target_path(
            Path::new(&request.destination_dir),
            &request.requested_file_name,
        )
        .map(|p| p.display().to_string())?;

        let record = DownloadRecord {
            id: request.id,
            url: request.url,
            destination_path,
            status: DownloadStatus::Queued,
            downloaded_bytes: 0,
            total_bytes: None,
            error: None,
            created_at: request.created_at,
            started_at: None,
            completed_at: None,
        };

        self.repository
            .insert(record)
            .map_err(|reason| DownloadError::Internal {
                reason: format!("failed to insert queued download: {reason}"),
            })
    }

    pub fn start(&self, request: DownloadRequest) -> Result<(), DownloadError> {
        let mut record =
            self.repository
                .get(&request.id)
                .ok_or_else(|| DownloadError::Internal {
                    reason: "download must be enqueued before start".to_string(),
                })?;

        let started_at = Utc::now();
        record.status = DownloadStatus::Running;
        record.started_at = Some(started_at);

        self.repository
            .update(record)
            .map_err(|reason| DownloadError::Internal {
                reason: format!("failed to mark download as running: {reason}"),
            })?;

        let token = CancellationToken::new();
        self.active
            .lock()
            .map_err(|_| DownloadError::Internal {
                reason: "active downloads lock poisoned".to_string(),
            })?
            .insert(request.id.clone(), token.clone());

        let repository = Arc::clone(&self.repository);
        let active = Arc::clone(&self.active);
        let policy = self.policy.clone();
        let config = self.config.clone();
        let sink = self.sink.clone();
        let request_for_task = request.clone();
        let started_at_for_task = started_at;
        let cancellation_requested = Arc::clone(&self.cancellation_requested);

        tokio::spawn(async move {
            let sink_ref = sink.as_deref();
            let result = download_http(
                &request_for_task,
                Path::new(&request_for_task.destination_dir),
                &policy,
                &config,
                sink_ref,
                token,
            )
            .await;

            let cancelled_by_request = cancellation_requested
                .lock()
                .map(|mut set| set.remove(&request_for_task.id))
                .unwrap_or(false);

            let final_record = match result {
                Ok(record) => record,
                Err(err) => DownloadRecord {
                    id: request_for_task.id.clone(),
                    url: request_for_task.url.clone(),
                    destination_path: safe_target_path(
                        Path::new(&request_for_task.destination_dir),
                        &request_for_task.requested_file_name,
                    )
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| request_for_task.requested_file_name.clone()),
                    status: if cancelled_by_request || matches!(err, DownloadError::Cancelled) {
                        DownloadStatus::Cancelled
                    } else {
                        DownloadStatus::Failed
                    },
                    downloaded_bytes: 0,
                    total_bytes: None,
                    error: Some(if cancelled_by_request {
                        DownloadError::Cancelled
                    } else {
                        err
                    }),
                    created_at: request_for_task.created_at,
                    started_at: Some(started_at_for_task),
                    completed_at: Some(Utc::now()),
                },
            };

            let _ = repository.update(final_record);
            if let Ok(mut guard) = active.lock() {
                guard.remove(&request_for_task.id);
            }
        });

        Ok(())
    }

    pub fn cancel(&self, id: &DownloadId) -> Result<bool, DownloadError> {
        let token = self
            .active
            .lock()
            .map_err(|_| DownloadError::Internal {
                reason: "active downloads lock poisoned".to_string(),
            })?
            .get(id)
            .cloned();

        if let Some(token) = token {
            if let Ok(mut set) = self.cancellation_requested.lock() {
                set.insert(id.clone());
            }
            token.cancel();
            return Ok(true);
        }

        if let Some(mut record) = self.repository.get(id) {
            if matches!(
                record.status,
                DownloadStatus::Queued | DownloadStatus::Running
            ) {
                record.status = DownloadStatus::Cancelled;
                record.error = Some(DownloadError::Cancelled);
                record.completed_at = Some(Utc::now());
                self.repository
                    .update(record)
                    .map_err(|reason| DownloadError::Internal {
                        reason: format!("failed to mark download as cancelled: {reason}"),
                    })?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn list(&self) -> Vec<DownloadRecord> {
        self.repository.list()
    }

    pub fn get(&self, id: &DownloadId) -> Option<DownloadRecord> {
        self.repository.get(id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;

    use crate::models::{DownloadId, DownloadRequest, DownloadStatus};
    use crate::security::url_policy::UrlPolicyConfig;
    use crate::storage::InMemoryDownloadRepository;

    use super::*;

    fn sample_request() -> DownloadRequest {
        DownloadRequest {
            id: DownloadId::new(),
            url: "https://example.com/archive.bin".to_string(),
            destination_dir: "/tmp".to_string(),
            requested_file_name: "archive.bin".to_string(),
            max_bytes: Some(1024),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn enqueue_creates_queued_record() {
        let repo = Arc::new(InMemoryDownloadRepository::default());
        let manager = DownloadManager::new(
            repo,
            UrlPolicy::new(UrlPolicyConfig::default()),
            DownloaderConfig::default(),
        );

        let request = sample_request();
        let id = request.id.clone();

        manager.enqueue(request).expect("enqueue");

        let entries = manager.list();
        let record = entries
            .iter()
            .find(|item| item.id == id)
            .expect("record must exist");
        assert_eq!(record.status, DownloadStatus::Queued);
    }

    #[test]
    fn cancel_queued_download_marks_cancelled() {
        let repo = Arc::new(InMemoryDownloadRepository::default());
        let manager = DownloadManager::new(
            repo,
            UrlPolicy::new(UrlPolicyConfig::default()),
            DownloaderConfig::default(),
        );

        let request = sample_request();
        let id = request.id.clone();

        manager.enqueue(request).expect("enqueue");
        let cancelled = manager.cancel(&id).expect("cancel");

        assert!(cancelled);
        let record = manager.get(&id).expect("record exists");
        assert_eq!(record.status, DownloadStatus::Cancelled);
    }

    #[test]
    fn enqueue_rejects_duplicate_id() {
        let repo = Arc::new(InMemoryDownloadRepository::default());
        let manager = DownloadManager::new(
            repo,
            UrlPolicy::new(UrlPolicyConfig::default()),
            DownloaderConfig::default(),
        );

        let request = sample_request();
        let duplicate = request.clone();
        manager.enqueue(request).expect("first enqueue");
        let err = manager.enqueue(duplicate).expect_err("must fail");
        assert!(matches!(err, DownloadError::Internal { .. }));
    }
}
