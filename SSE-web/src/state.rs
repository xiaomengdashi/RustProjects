use tokio::sync::broadcast;
use std::sync::Arc;

pub const CHANNEL_CAPACITY: usize = 100;

pub struct AppState {
    pub tx: broadcast::Sender<(usize, String)>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Arc::new(AppState { tx })
    }
} 