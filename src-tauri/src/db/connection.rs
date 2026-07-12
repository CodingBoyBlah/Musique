use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

// opens sqlite db, runs migrations, hands back the pool
// recreate db (everything is resyncable cache, so its safe {notes db/struct/ all notes})
// RENAME corrupt files instead of deleting

pub async fn create_pool(app: &AppHandle) -> SqlitePool {
    match open_file_db(app).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("[db] file database unavailable ({e}); using in-memory db so the app still launches");
            in_memory().await
        }
    }
}

// Try the real on-disk db. On the first failure we assume corruption, move the
// bad files aside, and try once more on a clean file. Returns Err only if BOTH
// attempts fail (or the data dir itself is unusable) - the caller then drops to
// in-memory.
async fn open_file_db(app: &AppHandle) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    use tauri::Manager;

    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("spotify-client.db");

    match try_open(&db_path).await {
        Ok(pool) => return Ok(pool),
        Err(e) => {
            eprintln!("[db] open/migrate failed ({e}); quarantining corrupt db and recreating");
        }
    }

    quarantine(&db_path);

    // second, final attempt on a clean file
    Ok(try_open(&db_path).await?)
}

// wal journaling + a 5s busy timeout so concurrent writes wait instead of
// blowing up instantly with SQLITE_BUSY.
async fn try_open(db_path: &Path) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        // wal lets a bunch of readers go at once but only one writer. without a
        // busy timeout the second write just dies instantly with SQLITE_BUSY
        // (database is locked). just wait for the lock instead of erroring out.
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// Rename the db and its wal/shm sidecars out of the way so a fresh one can be
// created. Renaming (not deleting) keeps the corrupt data recoverable. If a
// rename fails because a stale process still holds the file open (common on
// Windows), fall back to deleting; if THAT fails too, the follow-up open will
// error and we drop to in-memory - either way the app launches.
fn quarantine(db_path: &Path) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    for suffix in ["", "-wal", "-shm"] {
        let from = PathBuf::from(format!("{}{}", db_path.to_string_lossy(), suffix));
        if !from.exists() {
            continue;
        }
        let to = PathBuf::from(format!("{}.corrupt-{ts}", from.to_string_lossy()));
        if std::fs::rename(&from, &to).is_err() {
            let _ = std::fs::remove_file(&from);
        }
    }
}

// IN MEMORY DB - last resort
async fn in_memory() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory sqlite pool must open");
    let _ = sqlx::migrate!("./migrations").run(&pool).await;
    pool
}


