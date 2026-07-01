use sqlx::SqlitePool;
use std::sync::{mpsc, Arc};
use tokio::sync::{Mutex, RwLock};

use crate::media_controls::MediaMsg;

pub struct AppState {
    pub db:       SqlitePool,
    pub auth:     Arc<RwLock<AuthState>>,
    pub playback: Arc<Mutex<Option<crate::playback::PlaybackInner>>>,
    pub media_tx: mpsc::SyncSender<MediaMsg>,
    
    
    pub backdrop_active: bool,
}

#[derive(Default)]
pub struct AuthState {
    pub access_token:  Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at:    Option<i64>,
}
