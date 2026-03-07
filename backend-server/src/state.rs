use dashmap::DashMap;
use mongodb::Database;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::models::{message::SystemEvent, room::Room};
use crate::services::storage_service::ActiveStorage;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub db: Database,
    pub rooms: DashMap<String, Room>,
    pub room_idle_timeout_seconds: u64,
    pub system_tx: broadcast::Sender<SystemEvent>,
    pub jwt_secret: String,
    pub storage: RwLock<ActiveStorage>,
}

impl AppState {
    pub async fn storage_snapshot(&self) -> ActiveStorage {
        self.storage.read().await.clone()
    }

    pub async fn replace_storage(&self, next: ActiveStorage) {
        *self.storage.write().await = next;
    }
}
