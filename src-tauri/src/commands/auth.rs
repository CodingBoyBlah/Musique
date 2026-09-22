use crate::{
    auth::{self, http, pkce, token},
    errors::AppError,
    state::AppState,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use url::Url;

const SCOPES: &str = "user-read-private user-read-email streaming \
    user-read-playback-state user-modify-playback-state app-remote-control \
    user-library-read user-library-modify \
    playlist-read-private playlist-read-collaborative \
    playlist-modify-public playlist-modify-private \
    user-follow-read user-follow-modify \
    user-top-read user-read-recently-played user-read-playback-position";

fn gen_state() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(target_os = "linux")]
fn scrub_appimage_env(cmd: &mut std::process::Command) {
    let appdir = match std::env::var_os("APPDIR") {
        Some(d) => std::path::PathBuf::from(d),
        None => return,
    };

    for var in [
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "GTK_PATH",
        "GDK_PIXBUF_MODULE_FILE",
        "GIO_MODULE_DIR",
        "GST_PLUGIN_SYSTEM_PATH",
        "GST_PLUGIN_PATH",
        "QT_PLUGIN_PATH",
        "GSETTINGS_SCHEMA_DIR",
        "PYTHONPATH",
        "PERLLIB",
        "PYTHONHOME",
    ] {
        let Ok(val) = std::env::var(var) else {
            continue;
        };
        let kept: Vec<&str> = val
            .split(':')
            .filter(|p| !p.is_empty() && !std::path::Path::new(p).starts_with(&appdir))
            .collect();
        if kept.is_empty() {
            cmd.env_remove(var);
        } else {
            cmd.env(var, kept.join(":"));
        }
    }
}

#[cfg(target_os = "linux")]
fn spawn_clean(program: &str, args: &[&str]) -> std::io::Result<std::process::Child> {
    use std::process::{Command, Stdio};
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    scrub_appimage_env(&mut cmd);
    cmd.spawn()
}

#[cfg(target_os = "linux")]
pub(crate) fn open_url_linux(url: &str) -> Result<(), String> {
    let launchers: &[(&str, &[&str])] = &[
        ("xdg-open", &[]),
        ("gio", &["open"]),
        ("gvfs-open", &[]),
        ("kde-open5", &[]),
        ("kde-open", &[]),
    ];
    for (prog, pre) in launchers {
        let mut args: Vec<&str> = pre.to_vec();
        args.push(url);
        if let Ok(mut child) = spawn_clean(prog, &args) {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }
    }

    for prog in [
        "x-www-browser",
        "sensible-browser",
        "firefox",
        "firefox-esr",
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "brave-browser",
        "microsoft-edge",
    ] {
        if spawn_clean(prog, &[url]).is_ok() {
            return Ok(());
        }
    }
    Err("Could not open a browser. Install xdg-utils or a web browser, then try again.".into())
}

fn build_auth_url(
    client_id: &str,
    challenge: &str,
    state: &str,
    redirect: &str,
    scopes: &str,
    show_dialog: bool,
) -> String {
    let mut url = Url::parse("https://accounts.spotify.com/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", redirect)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", challenge)
        .append_pair("state", state)
        .append_pair("scope", scopes);
    if show_dialog {
        // Without this Spotify silently reuses whichever account the browser is
        // already signed in as. That is exactly what made "log out, log back in
        // as someone else" impossible: the approval screen never appeared, so
        // there was never anywhere to pick a different account.
        url.query_pairs_mut().append_pair("show_dialog", "true");
    }
    url.to_string()
}

fn open_browser(app: &AppHandle, auth_url: &str) -> Result<(), AppError> {
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        open_url_linux(auth_url).map_err(AppError::Auth)
    }

    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_url(auth_url, None::<&str>)
            .map_err(|e| AppError::Auth(format!("Cannot open browser: {e}")))
    }
}

#[tauri::command]
pub async fn start_login(app: AppHandle) -> Result<auth::AuthStatus, AppError> {
    let (db, auth) = {
        let s = app.state::<AppState>();
        (s.db.clone(), s.auth.clone())
    };

    let client_id = auth::get_active_client_id(&db).await;
    let port = http::CALLBACK_PORT;

    // Whose data is currently cached, so we can tell a re-login from a switch.
    let previous_user = auth::get_setting_value(&db, "spotify_user_id")
        .await
        .ok()
        .flatten()
        .filter(|s| !s.is_empty());

    let verifier = pkce::generate_verifier();
    let challenge = pkce::derive_challenge(&verifier);
    let state_token = gen_state();
    let redirect = format!("http://127.0.0.1:{port}/login");

    // Bind the redirect port BEFORE opening the browser: if the port is taken
    // (another copy of the app, an abandoned sign-in) the user finds out now
    // instead of after filling in their password.
    let listener = http::open_redirect(port).await?;

    let auth_url = build_auth_url(&client_id, &challenge, &state_token, &redirect, SCOPES, true);
    eprintln!("[auth] starting login with client_id={client_id} port={port}");
    open_browser(&app, &auth_url)?;

    let code = listener.wait(&state_token).await?;
    let status = auth::complete_login(&client_id, &code, &verifier, &redirect, &db, &auth).await?;

    // A different account (or one we could not identify) signed in: everything
    // cached for the previous one has to go, even if the last logout only got
    // half way - or never happened at all.
    let switched = match (&previous_user, &status.user_id) {
        (Some(prev), Some(now)) => prev != now,
        (Some(_), None) => true,
        _ => false,
    };
    if switched {
        eprintln!("[auth] account changed ({previous_user:?} -> {:?}); clearing previous account data", status.user_id);
        purge_account_data(&app, Purge::PreviousAccount).await;
    }

    Ok(status)
}

pub async fn authorize_playback_token(app: &AppHandle) -> Result<String, AppError> {
    let db = {
        let s = app.state::<AppState>();
        s.db.clone()
    };
    let port = http::PLAYBACK_PORT;
    let client_id = auth::PLAYBACK_CLIENT_ID;

    let verifier = pkce::generate_verifier();
    let challenge = pkce::derive_challenge(&verifier);
    let state_token = gen_state();
    let redirect = format!("http://127.0.0.1:{port}/login");

    let listener = http::open_redirect(port).await?;

    // No approval dialog here: this grant has to land on the same account the
    // Web API login just picked, and by now that is the browser's session.
    let auth_url = build_auth_url(
        client_id,
        &challenge,
        &state_token,
        &redirect,
        auth::PLAYBACK_SCOPES,
        false,
    );

    eprintln!("[playback auth] starting playback authorization on port {port}");
    open_browser(app, &auth_url)?;

    let code = listener.wait(&state_token).await?;
    let resp = auth::call_token_endpoint(&[
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", &redirect),
        ("client_id", client_id),
        ("code_verifier", &verifier),
    ])
    .await?;

    auth::upsert_setting(&db, "spotify_playback_token", &resp.access_token).await?;
    eprintln!("[playback auth] playback token acquired successfully");
    Ok(resp.access_token)
}

#[tauri::command]
pub async fn authorize_playback(app: AppHandle) -> Result<(), AppError> {
    let _ = authorize_playback_token(&app).await?;
    // Drop the old session so the next play rebuilds it against the new grant.
    // A plain `*guard = None` left the spirc task running and the old Connect
    // device registered.
    tear_down_playback(&app).await;
    Ok(())
}

// ─── signing out / switching accounts ────────────────────────────────────────

/// How much account state to clear.
#[derive(Clone, Copy, PartialEq)]
enum Purge {
    /// Signing out: nothing about the account survives.
    Everything,
    /// A different account just signed in: clear the old one's data, but keep
    /// the session rows the new login has already written.
    PreviousAccount,
}

/// Stop every audio backend and tear the librespot session down for real.
///
/// Both halves matter. The YouTube backend (used for free accounts) was never
/// touched by logout at all, so the music simply kept playing; and the librespot
/// session was only dropped, which detaches its tasks instead of ending them.
async fn tear_down_playback(app: &AppHandle) {
    // Any browser authorization still waiting for a redirect belongs to the
    // account we are getting rid of.
    http::cancel_pending_flows();

    let (playback, yt, media_tx) = {
        let s = app.state::<AppState>();
        (s.playback.clone(), s.yt.clone(), s.media_tx.clone())
    };

    // Bounded, because `create_inner` can be holding this lock while it waits
    // on an interactive playback grant. `cancel_pending_flows` above unblocks
    // that, but a sign-out must not be able to hang on it either way.
    let inner = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        playback.lock(),
    )
    .await
    {
        Ok(mut guard) => guard.take(),
        Err(_) => {
            eprintln!("[logout] playback session busy; leaving it to rebuild itself");
            None
        }
    };
    if let Some(inner) = inner {
        let _ = inner.pause();
        inner.shutdown().await;
    }

    let yt_inner = match tokio::time::timeout(std::time::Duration::from_secs(5), yt.lock()).await {
        Ok(mut guard) => guard.take(),
        Err(_) => {
            eprintln!("[logout] youtube backend busy; could not stop it");
            None
        }
    };
    if let Some(yt) = yt_inner {
        yt.stop();
    }

    let _ = media_tx.try_send(crate::media_controls::MediaMsg::Stopped);
}

/// Delete a cache directory, retrying while something still holds a file open.
/// On Windows a handle released moments ago can still fail the first attempt.
async fn wipe_dir(path: PathBuf) {
    for attempt in 1..=5 {
        if !path.exists() {
            return;
        }
        match std::fs::remove_dir_all(&path) {
            Ok(()) => return,
            Err(e) => {
                eprintln!("[logout] could not remove {} (attempt {attempt}): {e}", path.display());
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    }
}

/// Empty every table except `settings` and the migration bookkeeping.
///
/// Reads the table list out of the schema rather than hard-coding it: the old
/// hard-coded list silently missed every table added since it was written, and
/// one `DELETE` against a table that did not exist yet aborted the whole logout
/// with the account still on screen.
async fn purge_tables(db: &sqlx::SqlitePool) {
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master
          WHERE type = 'table'
            AND name NOT LIKE 'sqlite_%'
            AND name NOT IN ('settings', '_sqlx_migrations')",
    )
    .fetch_all(db)
    .await
    .unwrap_or_else(|e| {
        eprintln!("[logout] could not list tables: {e}");
        Vec::new()
    });

    // Foreign keys are enforced on this pool, so delete order would otherwise
    // matter. Deferring them until commit lets the tables be emptied in any
    // order - and means a table added by a future migration needs no thought.
    match db.begin().await {
        Ok(mut tx) => {
            let _ = sqlx::query("PRAGMA defer_foreign_keys = ON").execute(&mut *tx).await;
            clear_tables(&mut tx, &tables).await;
            if let Err(e) = tx.commit().await {
                eprintln!("[logout] purge transaction failed ({e}); clearing table by table");
                if let Ok(mut conn) = db.acquire().await {
                    clear_tables(&mut conn, &tables).await;
                }
            }
        }
        Err(e) => {
            eprintln!("[logout] could not open purge transaction: {e}");
            if let Ok(mut conn) = db.acquire().await {
                clear_tables(&mut conn, &tables).await;
            }
        }
    }
}

/// `DELETE FROM` each table, logging and carrying on past any that fails.
async fn clear_tables(conn: &mut sqlx::SqliteConnection, tables: &[(String,)]) {
    for (table,) in tables {
        // the names come from sqlite_master, never from user input
        if let Err(e) = sqlx::query(&format!("DELETE FROM \"{table}\""))
            .execute(&mut *conn)
            .await
        {
            eprintln!("[logout] could not clear {table}: {e}");
        }
    }
}

/// Clear everything tied to the signed-in account. Best-effort throughout: a
/// step that fails is logged and the rest still runs, because a half-finished
/// logout that reports an error is what left stale playlists on screen.
async fn purge_account_data(app: &AppHandle, scope: Purge) {
    let (db, sync_gate) = {
        let s = app.state::<AppState>();
        (s.db.clone(), s.sync_gate.clone())
    };

    // 1. stop anything that is still playing - or still writing
    tear_down_playback(app).await;

    // 2. let an in-flight library sync unwind before emptying the tables. The
    //    auth epoch has already moved by the time we get here, so the sync's
    //    remaining steps are no-ops and this returns as soon as the step it is
    //    on finishes. The timeout is a backstop: a stuck sync must not be able
    //    to block signing out.
    let _sync_barrier = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        sync_gate.write(),
    )
    .await
    {
        Ok(guard) => Some(guard),
        Err(_) => {
            eprintln!("[logout] library sync did not finish in time; clearing anyway");
            None
        }
    };

    // 3. in-memory caches
    crate::commands::spotify::clear_spotify_memory_caches();
    crate::library::clear_known_artists_cache();

    // 4. on-disk caches, after the teardown above so librespot cannot write its
    //    credentials back out from under us
    if let Ok(app_data) = app.path().app_data_dir() {
        wipe_dir(app_data.join("credentials")).await;
        wipe_dir(app_data.join("audio_cache")).await;
    }

    // 5. cached catalogue, library and history rows
    purge_tables(&db).await;

    // 6. account-scoped settings
    for key in auth::ACCOUNT_SETTING_KEYS {
        if scope == Purge::PreviousAccount && auth::SESSION_SETTING_KEYS.contains(key) {
            continue;
        }
        if let Err(e) = sqlx::query("DELETE FROM settings WHERE key = ?")
            .bind(key)
            .execute(&db)
            .await
        {
            eprintln!("[logout] could not clear setting {key}: {e}");
        }
    }

    if scope == Purge::Everything {
        let _ = sqlx::query("VACUUM").execute(&db).await;
    }
}

#[tauri::command]
pub async fn logout(app: AppHandle) -> Result<(), AppError> {
    // Invalidate the identity FIRST. Everything else in the app checks either
    // the auth epoch or the (now empty) auth state, so background work - a
    // library sync in particular - stops before the purge instead of writing
    // the old account's playlists back in behind it.
    auth::bump_auth_epoch();
    {
        let auth_state = {
            let s = app.state::<AppState>();
            s.auth.clone()
        };
        *auth_state.write().await = crate::state::AuthState::default();
    }

    if let Err(e) = token::clear_tokens() {
        // A keyring that refuses to delete must not abort the sign-out; the
        // tokens are already unusable because the in-memory state is gone.
        eprintln!("[logout] could not clear stored tokens: {e}");
    }

    purge_account_data(&app, Purge::Everything).await;

    // The session had a moment to shut down while the tables were cleared, so
    // make sure it did not flush a credentials file on its way out - that file
    // is what used to sign the previous account straight back in.
    if let Ok(app_data) = app.path().app_data_dir() {
        wipe_dir(app_data.join("credentials")).await;
    }

    eprintln!("[logout] signed out and cleared local account data");
    Ok(())
}

#[tauri::command]
pub async fn get_auth_status(app: AppHandle) -> Result<auth::AuthStatus, AppError> {
    let db = app.state::<AppState>().db.clone();
    let auth = app.state::<AppState>().auth.clone();
    let logged_in = auth.read().await.access_token.is_some();
    auth::build_auth_status(&db, logged_in).await
}

#[derive(serde::Serialize)]
pub struct Profile {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub country: Option<String>,
    pub product: Option<String>,
    pub followers: i64,
    pub image_url: Option<String>,
    pub spotify_url: Option<String>,
    pub explicit_filter_enabled: bool,
    pub explicit_filter_locked: bool,
}

#[derive(serde::Deserialize)]
struct SpProfileFull {
    id: Option<String>,
    display_name: Option<String>,
    email: Option<String>,
    country: Option<String>,
    product: Option<String>,
    followers: Option<SpFollowers>,
    images: Option<Vec<SpImg>>,
    external_urls: Option<SpExtUrls>,
    explicit_content: Option<SpExplicit>,
}

#[derive(serde::Deserialize)]
struct SpFollowers {
    total: i64,
}
#[derive(serde::Deserialize)]
struct SpImg {
    url: String,
}
#[derive(serde::Deserialize)]
struct SpExtUrls {
    spotify: Option<String>,
}
#[derive(serde::Deserialize)]
struct SpExplicit {
    filter_enabled: bool,
    filter_locked: bool,
}

async fn cached_profile(db: &sqlx::SqlitePool) -> Result<Profile, AppError> {
    let non_empty = |s: Option<String>| s.filter(|v| !v.is_empty());
    let user_id = auth::get_setting_value(db, "spotify_user_id").await?;
    let followers = auth::get_setting_value(db, "spotify_followers")
        .await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Ok(Profile {
        id: user_id.clone(),
        display_name: non_empty(auth::get_setting_value(db, "spotify_display_name").await?),
        email: non_empty(auth::get_setting_value(db, "spotify_email").await?),
        country: non_empty(auth::get_setting_value(db, "spotify_country").await?),
        product: non_empty(auth::get_setting_value(db, "spotify_product").await?),
        followers,
        image_url: non_empty(auth::get_setting_value(db, "spotify_image_url").await?),
        spotify_url: user_id.map(|id| format!("https://open.spotify.com/user/{id}")),
        explicit_filter_enabled: false,
        explicit_filter_locked: false,
    })
}

#[tauri::command]
pub async fn get_profile(app: AppHandle) -> Result<Profile, AppError> {
    let db = app.state::<AppState>().db.clone();
    let auth_state = app.state::<AppState>().auth.clone();

    let token = auth::get_valid_token(&db, &auth_state).await?;

    let resp = crate::http::client()
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(&token)
        .send()
        .await?;

    if !resp.status().is_success() {
        return cached_profile(&db).await;
    }

    let p: SpProfileFull = resp
        .json()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    let followers = p.followers.as_ref().map(|f| f.total).unwrap_or(0);

    if let Some(c) = p.country.as_deref() {
        let _ = auth::upsert_setting(&db, "spotify_country", c).await;
    }
    let _ = auth::upsert_setting(&db, "spotify_followers", &followers.to_string()).await;
    let _ = auth::upsert_setting(
        &db,
        "spotify_explicit_filter",
        if p.explicit_content
            .as_ref()
            .map(|e| e.filter_enabled)
            .unwrap_or(false)
        {
            "1"
        } else {
            "0"
        },
    )
    .await;

    Ok(Profile {
        id: p.id,
        display_name: p.display_name.filter(|s| !s.is_empty()),
        email: p.email.filter(|s| !s.is_empty()),
        country: p.country.filter(|s| !s.is_empty()),
        product: p.product.filter(|s| !s.is_empty()),
        followers,
        image_url: p
            .images
            .as_ref()
            .and_then(|v| v.first())
            .map(|i| i.url.clone()),
        spotify_url: p.external_urls.and_then(|u| u.spotify),
        explicit_filter_enabled: p
            .explicit_content
            .as_ref()
            .map(|e| e.filter_enabled)
            .unwrap_or(false),
        explicit_filter_locked: p
            .explicit_content
            .as_ref()
            .map(|e| e.filter_locked)
            .unwrap_or(false),
    })
}
