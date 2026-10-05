use crate::api::{new_download_request, BridgeDownloadRequest};
use crate::download::DownloadManager;
use crate::models::{DownloadError, DownloadId, DownloadRecord};

#[derive(Clone)]
pub struct CoreDownloadService {
    manager: DownloadManager,
}

impl CoreDownloadService {
    pub fn new(manager: DownloadManager) -> Self {
        Self { manager }
    }

    pub fn enqueue_only(&self, input: BridgeDownloadRequest) -> Result<DownloadId, DownloadError> {
        let request = new_download_request(input);
        let id = request.id.clone();
        self.manager.enqueue(request)?;
        Ok(id)
    }

    pub fn enqueue_and_start(
        &self,
        input: BridgeDownloadRequest,
    ) -> Result<DownloadId, DownloadError> {
        let request = new_download_request(input);
        let id = request.id.clone();
        self.manager.enqueue(request.clone())?;
        self.manager.start(request)?;
        Ok(id)
    }

    pub fn cancel(&self, id: &DownloadId) -> Result<bool, DownloadError> {
        self.manager.cancel(id)
    }

    pub fn get(&self, id: &DownloadId) -> Option<DownloadRecord> {
        self.manager.get(id)
    }

    pub fn list(&self) -> Vec<DownloadRecord> {
        self.manager.list()
    }
}
