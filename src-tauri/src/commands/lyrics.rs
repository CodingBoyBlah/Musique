use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    lyrics::{self, Lyrics, TrackRef},
    state::AppState,
};

/// Grab the lyrics for a track (or read them from cache).
///
/// Returns as soon as there is something *correctly synced* to render - usually
/// after a single request against the exact Spotify track id. If a word-by-word
/// source later proves it agrees with that timing, it arrives separately on the
/// `lyrics:upgraded` event and the panel swaps it in silently.
///
/// The frontend hands us the metadata it already has so we don't need a Spotify
/// round trip just to learn the track's own name. `isrc` is the strongest
/// identifier of the lot - it pins the exact *recording*, which is what stops
/// providers handing back the remaster/radio-edit with plausible but wrong
/// timings.
#[tauri::command]
pub async fn get_lyrics(
    app:         AppHandle,
    track_id:    String,
    name:        String,
    artist:      String,
    album:       Option<String>,
    isrc:        Option<String>,
    duration_ms: i64,
    force:       Option<bool>,
) -> Result<Lyrics, AppError> {
    let pool = app.state::<AppState>().db.clone();

    // the frontend only has an ISRC for tracks it fetched as full track objects.
    // for everything else (album pages, playlists, anything restored from the
    // local catalog) fall back to the one we persisted during library sync -
    // one indexed primary-key lookup, and it means library tracks get exact
    // recording matching without the frontend having to carry the field around.
    let isrc = match isrc.filter(|s| !s.trim().is_empty()) {
        Some(i) => Some(i),
        None => sqlx::query_scalar::<_, Option<String>>("SELECT isrc FROM tracks WHERE id = ?")
            .bind(&track_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .flatten(),
    };

    let track = TrackRef {
        id: track_id,
        name,
        artist,
        album: album.unwrap_or_default(),
        isrc,
        duration_ms,
    };

    lyrics::get_or_fetch(&app, &pool, track, force.unwrap_or(false)).await
}

/// Switch the panel to another provider we already hold cached for this track.
/// Never touches the network, so the source switcher stays instant and works
/// offline.
#[tauri::command]
pub async fn set_lyrics_source(
    app:      AppHandle,
    track_id: String,
    source:   String,
) -> Result<Lyrics, AppError> {
    let pool = app.state::<AppState>().db.clone();
    lyrics::switch_source(&pool, &track_id, &source).await
}
