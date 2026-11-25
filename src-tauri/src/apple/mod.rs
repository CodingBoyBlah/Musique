//! Apple Music as a full provider: account, catalog and library API, and Widevine playback.
//!
//! - `auth`: the web player's bearer token and the account's `media-user-token`.
//! - `client`: the amp-api client the rest of the app reads Apple Music through.
//! - `wire`: Apple JSON into the app's own models.
//! - `api`: the calls the command layer makes, answering in the app's models.
//! - `library`: writes the Apple library into the local database a sync fills.
//! - `playback`: resolving, licensing and decrypting a track into clear audio.

#![allow(dead_code)]

pub mod api;
pub mod auth;
pub mod client;
pub mod library;
pub mod links;
pub mod wire;

pub mod playback;

use std::sync::atomic::{AtomicBool, Ordering};

/// The `settings` row naming which service the signed-in account belongs to: `"apple"` for
/// Apple Music, anything else (or no row) for Spotify. One provider is signed in at a time;
/// switching signs the other out and clears the local library.
pub const PROVIDER_KEY: &str = "provider";

/// The provider as last read or set, so hot paths need no database round trip.
static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether Apple Music is the signed-in provider. Reads the in-memory flag that
/// [`load`] and [`set_active`] keep current.
pub fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// Reads the provider row into the in-memory flag. Called once at startup.
pub async fn load(pool: &sqlx::SqlitePool) -> bool {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(PROVIDER_KEY)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let apple = row.is_some_and(|(value,)| value == "apple");
    ACTIVE.store(apple, Ordering::Relaxed);
    apple
}

/// Records which provider is signed in, in the database and the in-memory flag.
pub async fn set_active(pool: &sqlx::SqlitePool, apple: bool) -> Result<(), crate::errors::AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as i64);
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(PROVIDER_KEY)
    .bind(if apple { "apple" } else { "spotify" })
    .bind(now)
    .execute(pool)
    .await?;
    ACTIVE.store(apple, Ordering::Relaxed);
    Ok(())
}
