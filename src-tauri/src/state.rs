use sqlx::SqlitePool;
use std::sync::{mpsc, Arc};
use tokio::sync::{Mutex, RwLock};

use crate::media_controls::MediaMsg;

pub struct AppState {
    pub db:       SqlitePool,
    pub auth:     Arc<RwLock<AuthState>>,
    pub playback: Arc<Mutex<Option<crate::playback::PlaybackInner>>>,
    /// Audio backend for free accounts, which cannot stream through librespot.
    ///
    /// Held alongside `playback` rather than replacing it because the two are
    /// not interchangeable: the librespot session also backs Spotify Connect
    /// and the spclient calls the lyrics pipeline makes, and those stay
    /// meaningful on a free account even when its audio comes from elsewhere.
    /// Exactly one of the two drives audio at a time - see
    /// `commands::playback::uses_youtube`.
    pub yt:       Arc<Mutex<Option<crate::playback::youtube::YtPlayback>>>,
    pub media_tx: mpsc::SyncSender<MediaMsg>,

    /// Held for reading while a library sync runs, and for writing while a
    /// sign-out clears the database.
    ///
    /// A sync is long and writes as it goes, so one that was already in flight
    /// when you signed out used to re-insert the previous account's library
    /// into the tables logout had just emptied - the account "coming back" on
    /// its own. Logout now waits here for the sync to unwind before it purges.
    pub sync_gate: Arc<RwLock<()>>,

    pub backdrop_active: bool,
}

#[derive(Default)]
pub struct AuthState {
    pub access_token:  Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at:    Option<i64>,
}
