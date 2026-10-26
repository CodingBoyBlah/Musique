/// Exercise production token refresh and library sync without creating a window.
/// The caller must supply a database snapshot, so diagnostics cannot replace the
/// user's library. Credentials remain in the existing OS credential store.
pub fn sync_probe() -> i32 {
    let Some(path) = std::env::var_os("MUSIQUE_SYNC_PROBE_DB") else {
        eprintln!("Set MUSIQUE_SYNC_PROBE_DB to a database snapshot first");
        return 2;
    };
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2).thread_stack_size(2 * 1024 * 1024).enable_all().build()
    {
        Ok(runtime) => runtime,
        Err(_) => return 3,
    };
    runtime.block_on(Box::pin(async move {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(std::path::PathBuf::from(path))
            .create_if_missing(false);
        let pool = match sqlx::sqlite::SqlitePoolOptions::new().max_connections(4)
            .connect_with(options).await
        {
            Ok(pool) => pool,
            Err(e) => { eprintln!("sync probe database: {e}"); return 4; }
        };
        let state = crate::auth::init_auth_state(&pool).await;
        let token = match crate::auth::get_valid_token(&pool, &tokio::sync::RwLock::new(state)).await {
            Ok(token) => token,
            Err(e) => { eprintln!("sync probe authentication: {e}"); return 5; }
        };
        let mut failed = false;
        for attempt in 1..=2 {
            match crate::library::sync_all(&pool, &token, &|step| println!("sync step completed: {step}")).await {
                Ok(result) => println!("sync attempt {attempt} completed: {result:?}"),
                Err(e) => { eprintln!("sync attempt {attempt} returned an error without exiting: {e}"); failed = true; }
            }
        }
        println!("sync probe survived both attempts");
        if failed { 6 } else { 0 }
    }))
}
