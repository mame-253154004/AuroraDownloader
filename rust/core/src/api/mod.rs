pub mod service;

use chrono::Utc;

use crate::models::{DownloadId, DownloadRequest};

#[derive(Debug, Clone)]
pub struct BridgeDownloadRequest {
    pub url: String,
    pub destination_dir: String,
    pub file_name: String,
    pub max_bytes: Option<u64>,
}

pub fn new_download_request(input: BridgeDownloadRequest) -> DownloadRequest {
    DownloadRequest {
        id: DownloadId::new(),
        url: input.url,
        destination_dir: input.destination_dir,
        requested_file_name: input.file_name,
        max_bytes: input.max_bytes,
        created_at: Utc::now(),
    }
}
