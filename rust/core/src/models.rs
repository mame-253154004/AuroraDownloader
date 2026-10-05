use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DownloadId(pub String);

impl DownloadId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for DownloadId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub id: DownloadId,
    pub url: String,
    pub destination_dir: String,
    pub requested_file_name: String,
    pub max_bytes: Option<u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub id: DownloadId,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DownloadError {
    InvalidUrl { reason: String },
    PolicyViolation { reason: String },
    Network { reason: String },
    Io { reason: String },
    Timeout,
    Cancelled,
    SizeLimitExceeded { max_bytes: u64 },
    Unsupported { reason: String },
    RetryExhausted { attempts: u32, last_error: String },
    Internal { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: DownloadId,
    pub url: String,
    pub destination_path: String,
    pub status: DownloadStatus,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub error: Option<DownloadError>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_download_record_round_trip() {
        let record = DownloadRecord {
            id: DownloadId("id-1".to_string()),
            url: "https://example.com/file.bin".to_string(),
            destination_path: "/tmp/file.bin".to_string(),
            status: DownloadStatus::Failed,
            downloaded_bytes: 12,
            total_bytes: Some(100),
            error: Some(DownloadError::Network {
                reason: "connection reset".to_string(),
            }),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };

        let json = serde_json::to_string(&record).expect("serialize");
        let parsed: DownloadRecord = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.id, record.id);
        assert_eq!(parsed.status, DownloadStatus::Failed);
        assert!(matches!(parsed.error, Some(DownloadError::Network { .. })));
    }
}
