//! Spotify and Apple Music signed in side by side, one of them shown at a time.
//!
//! Each service keeps its own sign-in (Spotify's tokens and Apple's `media-user-token` both
//! live in the OS keyring), so switching signs nothing out. What the interface reads (library,
//! cached catalogue, history, taste, lyrics) sits in the same sqlite tables for either service,
//! so a switch moves the outgoing service's rows into a side database of its own
//! (`library-<service>.db` in the app data folder) and moves the incoming service's rows back
//! in. The whole library is a few MB, so this takes a moment rather than a resync; the sync
//! that follows only tops it up.

use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

use crate::{errors::AppError, state::AppState};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    Spotify,
    Apple,
}

impl Provider {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "spotify" => Some(Self::Spotify),
            "apple" => Some(Self::Apple),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Spotify => "spotify",
            Self::Apple => "apple",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Spotify => "Spotify",
            Self::Apple => "Apple Music",
        }
    }

    pub fn other(self) -> Self {
        match self {
            Self::Spotify => Self::Apple,
            Self::Apple => Self::Spotify,
        }
    }

    /// The service the interface is showing.
    pub fn active() -> Self {
        if crate::apple::is_active() { Self::Apple } else { Self::Spotify }
    }
}

/// Settings rows that describe the library in the shared tables rather than an account, so
/// they travel with those tables.
pub const LIBRARY_SETTING_KEYS: &[&str] = &["library_last_synced"];

/// Serialises switches, and sign-ins and sign-outs that switch.
static SWITCH: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(crate) async fn switch_lock() -> tokio::sync::MutexGuard<'static, ()> {
    SWITCH.lock().await
}

pub async fn spotify_signed_in(app: &AppHandle) -> bool {
    let auth = app.state::<AppState>().auth.clone();
    let signed_in = auth.read().await.refresh_token.is_some();
    signed_in
}

pub fn apple_signed_in() -> bool {
    crate::apple::auth::load_user_token().is_some()
}

pub async fn signed_in(app: &AppHandle, provider: Provider) -> bool {
    match provider {
        Provider::Spotify => spotify_signed_in(app).await,
        Provider::Apple => apple_signed_in(),
    }
}

fn stash_path(app: &AppHandle, provider: Provider) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|dir| dir.join(format!("library-{}.db", provider.as_str())))
}

/// Forgets the library a signed-out service had set aside.
pub fn delete_stash(app: &AppHandle, provider: Provider) {
    if let Some(path) = stash_path(app, provider) {
        remove_db_file(&path);
    }
}

fn remove_db_file(path: &Path) {
    for file in [path.to_path_buf(), path.with_extension("db-journal")] {
        if file.exists() {
            if let Err(e) = std::fs::remove_file(&file) {
                eprintln!("[accounts] could not remove {}: {e}", file.display());
            }
        }
    }
}

/// Shows `target` instead of the service shown now: stops what plays, sets the outgoing
/// library aside and brings the incoming one back. Both stay signed in.
pub async fn switch(app: &AppHandle, target: Provider) -> Result<(), AppError> {
    let _turn = switch_lock().await;
    switch_locked(app, target).await
}

/// [`switch`] for a caller already holding [`switch_lock`].
pub(crate) async fn switch_locked(app: &AppHandle, target: Provider) -> Result<(), AppError> {
    let current = Provider::active();
    if current == target {
        return Ok(());
    }
    if !signed_in(app, target).await {
        return Err(AppError::Auth(format!("Sign in to {} first", target.label())));
    }
    eprintln!("[accounts] switching {} -> {}", current.as_str(), target.as_str());

    // Work still running for the outgoing service (a library sync above all) stops writing.
    {
        let _commit = crate::auth::session_commit().await;
        crate::auth::bump_auth_epoch();
    }
    crate::apple::library::bump_session_epoch();

    stop_audio(app, target).await;

    let (db, gate) = {
        let s = app.state::<AppState>();
        (s.db.clone(), s.sync_gate.clone())
    };
    // A sync mid-step finishes the step it is on (the epoch has moved, so it writes nothing
    // more); the timeout keeps a stuck one from holding the switch.
    let _sync_barrier = tokio::time::timeout(Duration::from_secs(15), gate.write()).await.ok();

    crate::commands::spotify::clear_spotify_memory_caches();
    crate::library::clear_known_artists_cache();

    let out = stash_path(app, current)
        .ok_or_else(|| AppError::InvalidInput("no app data folder to keep the library in".into()))?;
    let into = stash_path(app, target)
        .ok_or_else(|| AppError::InvalidInput("no app data folder to keep the library in".into()))?;
    swap_library(&db, &out, &into, target).await?;
    crate::apple::set_flag(target == Provider::Apple);

    // The incoming service's player gets ready behind the switch, so the first play is quick.
    let app = app.clone();
    match target {
        Provider::Spotify => {
            tauri::async_runtime::spawn(crate::commands::soloist::start_if_enabled(app.clone()));
            tauri::async_runtime::spawn(crate::commands::playback::warm_session_if_possible(app));
        }
        Provider::Apple => {
            tauri::async_runtime::spawn(async {
                let _ = crate::apple::client::current().await;
            });
        }
    }
    Ok(())
}

/// Silences every local player before the other service takes over.
async fn stop_audio(app: &AppHandle, target: Provider) {
    crate::apple::playback::claim();
    crate::apple::playback::clear_active().await;
    crate::apple::playback::clear_prepared().await;
    if target == Provider::Apple {
        // Soloist marks the shared player's events as not heard while it runs, which would
        // hide Apple Music's playback from the interface. It comes back with Spotify.
        let soloist = app.state::<AppState>().soloist.clone();
        soloist.stop().await;
    }
    crate::commands::auth::tear_down_playback(app).await;
}

/// The tables the shared library lives in: everything but settings and sqlite's own.
async fn library_tables(conn: &mut sqlx::SqliteConnection, schema: &str) -> Result<Vec<String>, AppError> {
    let sql = format!(
        "SELECT name FROM \"{schema}\".sqlite_master
          WHERE type = 'table'
            AND name NOT LIKE 'sqlite_%'
            AND name NOT IN ('settings', '_sqlx_migrations', 'library_settings')"
    );
    let rows: Vec<(String,)> = sqlx::query_as(&sql).persistent(false).fetch_all(&mut *conn).await?;
    Ok(rows.into_iter().map(|(name,)| name).collect())
}

async fn columns(conn: &mut sqlx::SqliteConnection, schema: &str, table: &str) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM pragma_table_info(?, ?)")
        .bind(table)
        .bind(schema)
        .persistent(false)
        .fetch_all(&mut *conn)
        .await?;
    Ok(rows.into_iter().map(|(name,)| name).collect())
}

async fn exec(conn: &mut sqlx::SqliteConnection, sql: &str) -> Result<(), AppError> {
    sqlx::query(sql).persistent(false).execute(&mut *conn).await?;
    Ok(())
}

async fn attach(conn: &mut sqlx::SqliteConnection, path: &Path, alias: &str) -> Result<(), AppError> {
    sqlx::query(&format!("ATTACH DATABASE ? AS {alias}"))
        .bind(path.to_string_lossy().into_owned())
        .persistent(false)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Copies the shared tables out to `out`, then replaces them with what `into` set aside
/// (empty when it has nothing) and records `target` as the provider, in one transaction.
///
/// Copying out happens first and on its own: if bringing the other library in fails, the
/// tables still hold the outgoing library, the provider is unchanged, and nothing is lost.
async fn swap_library(db: &SqlitePool, out: &Path, into: &Path, target: Provider) -> Result<(), AppError> {
    let started = std::time::Instant::now();
    let mut conn = db.acquire().await?;

    // 1. the outgoing library, into a fresh file that replaces its old copy only once complete
    let tmp = out.with_extension("db.tmp");
    remove_db_file(&tmp);
    attach(&mut conn, &tmp, "stash_out").await?;
    let copied = copy_out(&mut conn).await;
    let _ = exec(&mut conn, "DETACH DATABASE stash_out").await;
    if let Err(e) = copied {
        remove_db_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, out)
        .map_err(|e| AppError::Database(format!("cannot keep the library aside: {e}")))?;

    // 2. the incoming library (if it was kept), and the provider, together
    let has_stash = into.exists();
    if has_stash {
        attach(&mut conn, into, "stash_in").await?;
    }
    let restored = copy_in(&mut conn, has_stash, target).await;
    if has_stash {
        let _ = exec(&mut conn, "DETACH DATABASE stash_in").await;
    }
    restored?;
    // what it held is in the live tables now; a later switch writes a fresh copy
    if has_stash {
        remove_db_file(into);
    }
    eprintln!(
        "[accounts] library swapped in {}ms (restored: {has_stash})",
        started.elapsed().as_millis()
    );
    Ok(())
}

async fn copy_out(conn: &mut sqlx::SqliteConnection) -> Result<(), AppError> {
    let tables = library_tables(conn, "main").await?;
    exec(conn, "BEGIN").await?;
    let result = async {
        for table in &tables {
            // names come from sqlite_master, never from input
            exec(conn, &format!("CREATE TABLE stash_out.\"{table}\" AS SELECT * FROM main.\"{table}\"")).await?;
        }
        let keys = LIBRARY_SETTING_KEYS.iter().map(|k| format!("'{k}'")).collect::<Vec<_>>().join(",");
        exec(
            conn,
            &format!("CREATE TABLE stash_out.library_settings AS SELECT key, value, updated_at FROM main.settings WHERE key IN ({keys})"),
        )
        .await?;
        Ok::<(), AppError>(())
    }
    .await;
    finish(conn, result).await
}

async fn copy_in(conn: &mut sqlx::SqliteConnection, has_stash: bool, target: Provider) -> Result<(), AppError> {
    let tables = library_tables(conn, "main").await?;
    let kept = if has_stash { library_tables(conn, "stash_in").await? } else { Vec::new() };
    let keys = LIBRARY_SETTING_KEYS.iter().map(|k| format!("'{k}'")).collect::<Vec<_>>().join(",");
    exec(conn, "BEGIN").await?;
    let result = async {
        exec(conn, "PRAGMA defer_foreign_keys = ON").await?;
        for table in &tables {
            exec(conn, &format!("DELETE FROM main.\"{table}\"")).await?;
        }
        for table in tables.iter().filter(|t| kept.contains(t)) {
            // only the columns both sides have: a migration since the copy was made may have
            // added or dropped some, and a row that no longer fits is left for the sync
            let live = columns(conn, "main", table).await?;
            let saved = columns(conn, "stash_in", table).await?;
            let shared: Vec<String> = live
                .into_iter()
                .filter(|c| saved.contains(c))
                .map(|c| format!("\"{c}\""))
                .collect();
            if shared.is_empty() {
                continue;
            }
            let cols = shared.join(", ");
            exec(
                conn,
                &format!("INSERT OR IGNORE INTO main.\"{table}\" ({cols}) SELECT {cols} FROM stash_in.\"{table}\""),
            )
            .await?;
        }
        exec(conn, &format!("DELETE FROM main.settings WHERE key IN ({keys})")).await?;
        if has_stash {
            let has_settings: Option<(String,)> = sqlx::query_as(
                "SELECT name FROM stash_in.sqlite_master WHERE type = 'table' AND name = 'library_settings'",
            )
            .persistent(false)
            .fetch_optional(&mut *conn)
            .await?;
            if has_settings.is_some() {
                exec(
                    conn,
                    "INSERT OR REPLACE INTO main.settings (key, value, updated_at)
                     SELECT key, value, updated_at FROM stash_in.library_settings",
                )
                .await?;
            }
        }
        // A row whose parent was left out above would fail the deferred foreign-key check
        // and with it the whole switch; drop it instead, and the sync fetches it again.
        let orphans: Vec<(String, Option<i64>)> = sqlx::query_as("SELECT \"table\", rowid FROM pragma_foreign_key_check")
            .persistent(false)
            .fetch_all(&mut *conn)
            .await?;
        for (table, rowid) in orphans {
            if let Some(rowid) = rowid {
                sqlx::query(&format!("DELETE FROM main.\"{table}\" WHERE rowid = ?"))
                    .bind(rowid)
                    .persistent(false)
                    .execute(&mut *conn)
                    .await?;
            }
        }
        sqlx::query(
            "INSERT INTO main.settings (key, value, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(crate::apple::PROVIDER_KEY)
        .bind(target.as_str())
        .bind(crate::auth::now_ms())
        .persistent(false)
        .execute(&mut *conn)
        .await?;
        Ok::<(), AppError>(())
    }
    .await;
    finish(conn, result).await
}

/// Commits a transaction `result` says succeeded, and rolls back one that failed.
async fn finish(conn: &mut sqlx::SqliteConnection, result: Result<(), AppError>) -> Result<(), AppError> {
    match result {
        Ok(()) => match exec(conn, "COMMIT").await {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = exec(conn, "ROLLBACK").await;
                Err(e)
            }
        },
        Err(e) => {
            let _ = exec(conn, "ROLLBACK").await;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    async fn pool(dir: &Path) -> SqlitePool {
        let opts = SqliteConnectOptions::new()
            .filename(dir.join("main.db"))
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new().max_connections(2).connect_with(opts).await.unwrap();
        for sql in [
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL)",
            "CREATE TABLE playlists (id TEXT PRIMARY KEY, name TEXT NOT NULL)",
            "CREATE TABLE playlist_tracks (playlist_id TEXT NOT NULL REFERENCES playlists(id), track TEXT NOT NULL)",
        ] {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }
        pool
    }

    async fn names(pool: &SqlitePool) -> Vec<String> {
        sqlx::query_as::<_, (String,)>("SELECT name FROM playlists ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|(n,)| n)
            .collect()
    }

    async fn setting(pool: &SqlitePool, key: &str) -> Option<String> {
        sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await
            .unwrap()
            .map(|(v,)| v)
    }

    #[tokio::test]
    async fn switching_sets_each_library_aside_and_brings_it_back() {
        let dir = std::env::temp_dir().join(format!("musique-accounts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let pool = pool(&dir).await;
        let spotify = dir.join("library-spotify.db");
        let apple = dir.join("library-apple.db");

        sqlx::query("INSERT INTO playlists VALUES ('s1', 'Spotify mix')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO playlist_tracks VALUES ('s1', 't1')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO settings VALUES ('library_last_synced', '111', 0), ('volume', '50', 0)")
            .execute(&pool)
            .await
            .unwrap();

        // to Apple: nothing kept for it yet, so it starts empty
        swap_library(&pool, &spotify, &apple, Provider::Apple).await.unwrap();
        assert!(names(&pool).await.is_empty());
        assert_eq!(setting(&pool, "library_last_synced").await, None);
        assert_eq!(setting(&pool, "volume").await.as_deref(), Some("50"));
        assert_eq!(setting(&pool, crate::apple::PROVIDER_KEY).await.as_deref(), Some("apple"));
        assert!(spotify.exists());

        sqlx::query("INSERT INTO playlists VALUES ('a1', 'Apple mix')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO settings VALUES ('library_last_synced', '222', 0)").execute(&pool).await.unwrap();

        // back to Spotify: its library returns whole, Apple's is kept
        swap_library(&pool, &apple, &spotify, Provider::Spotify).await.unwrap();
        assert_eq!(names(&pool).await, vec!["Spotify mix"]);
        let tracks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM playlist_tracks").fetch_one(&pool).await.unwrap();
        assert_eq!(tracks.0, 1);
        assert_eq!(setting(&pool, "library_last_synced").await.as_deref(), Some("111"));
        assert_eq!(setting(&pool, crate::apple::PROVIDER_KEY).await.as_deref(), Some("spotify"));
        assert!(apple.exists());
        assert!(!spotify.exists());

        swap_library(&pool, &spotify, &apple, Provider::Apple).await.unwrap();
        assert_eq!(names(&pool).await, vec!["Apple mix"]);
        assert_eq!(setting(&pool, "library_last_synced").await.as_deref(), Some("222"));

        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
