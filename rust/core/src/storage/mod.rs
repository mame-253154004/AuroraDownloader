use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{DownloadId, DownloadRecord};

pub trait DownloadRepository: Send + Sync {
    fn insert(&self, record: DownloadRecord) -> Result<(), String>;
    fn update(&self, record: DownloadRecord) -> Result<(), String>;
    fn get(&self, id: &DownloadId) -> Option<DownloadRecord>;
    fn list(&self) -> Vec<DownloadRecord>;
}

#[derive(Default, Clone)]
pub struct InMemoryDownloadRepository {
    state: Arc<Mutex<HashMap<DownloadId, DownloadRecord>>>,
}

impl DownloadRepository for InMemoryDownloadRepository {
    fn insert(&self, record: DownloadRecord) -> Result<(), String> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| "mutex poisoned".to_string())?;
        guard.insert(record.id.clone(), record);
        Ok(())
    }

    fn update(&self, record: DownloadRecord) -> Result<(), String> {
        self.insert(record)
    }

    fn get(&self, id: &DownloadId) -> Option<DownloadRecord> {
        self.state.lock().ok()?.get(id).cloned()
    }

    fn list(&self) -> Vec<DownloadRecord> {
        self.state
            .lock()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }
}

// NOTE: SQLite integration is intentionally deferred to keep this first bootstrap small.
// TODO(storage): Introduce SQLite-backed repository with migrations and encrypted-at-rest options.
// Sensitive cookies/tokens must never be stored here.
