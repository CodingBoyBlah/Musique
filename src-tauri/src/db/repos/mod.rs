pub mod albums;
pub mod artists;
pub mod playback_history;
pub mod playlists;
pub mod saved_tracks;
pub mod search_history;
pub mod tracks;
pub mod users;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
