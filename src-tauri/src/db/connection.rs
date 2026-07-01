use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use tauri::AppHandle;

// open the sqlite db or make it if its not there, run migrations, give back the pool
// wal journaling + a 5s busy timeout so concurrent writes wait instead of
// blowing up instantly with SQLITE_BUSY
pub async fn create_pool(app: &AppHandle) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    use tauri::Manager;

    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("spotify-client.db");

    let opts = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        // wal lets a bunch of readers go at once but only one writer. without a busy
        // timeout the second write just dies instantly with
        // SQLITE_BUSY (database is locked) thats what was secretly breaking the
        // saved albums sync. just wait for the lock instead of erroring out
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
