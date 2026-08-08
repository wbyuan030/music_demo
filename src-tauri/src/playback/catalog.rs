use std::collections::HashMap;

use tokio::sync::Mutex;

use super::model::PlayableEntry;

#[derive(Debug, Default)]
pub struct TrackCatalog {
    entries: Mutex<HashMap<String, PlayableEntry>>,
}

impl TrackCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&self, id: String, entry: PlayableEntry) {
        self.entries.lock().await.insert(id, entry);
    }

    pub async fn get(&self, id: &str) -> Option<PlayableEntry> {
        self.entries.lock().await.get(id).cloned()
    }

    pub async fn remove(&self, id: &str) {
        self.entries.lock().await.remove(id);
    }
}
