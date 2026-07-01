use crate::{
    auth::{self, http, pkce, token},
    errors::AppError,
    state::AppState,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use tauri::{AppHandle, Manager, State};
use url::Url;

const SCOPES: &str = "user-read-private user-read-email streaming \
    user-library-read user-library-modify \
    playlist-read-private playlist-read-collaborative \
    user-follow-read user-follow-modify \
    user-top-read user-read-recently-played";

fn redirect_uri() -> String {
    format!("http://127.0.0.1:{}/callback", http::CALLBACK_PORT)
}

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
        "LD_LIBRARY_PATH", "LD_PRELOAD",
        "GTK_PATH", "GDK_PIXBUF_MODULE_FILE", "GIO_MODULE_DIR",
        "GST_PLUGIN_SYSTEM_PATH", "GST_PLUGIN_PATH", "QT_PLUGIN_PATH",
        "GSETTINGS_SCHEMA_DIR", "PYTHONPATH", "PERLLIB", "PYTHONHOME",
    ] {
        let Ok(val) = std::env::var(var) else { continue };
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
        ("xdg-open",  &[]),
        ("gio",       &["open"]),
        ("gvfs-open", &[]),
        ("kde-open5", &[]),
        ("kde-open",  &[]),
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
        "x-www-browser", "sensible-browser",
        "firefox", "firefox-esr",
        "google-chrome", "google-chrome-stable",
        "chromium", "chromium-browser",
        "brave-browser", "microsoft-edge",
    ] {
        if spawn_clean(prog, &[url]).is_ok() {
            return Ok(());
        }
    }
    Err("Could not open a browser. Install xdg-utils or a web browser, then try again.".into())
}

fn build_auth_url(client_id: &str, challenge: &str, state: &str, redirect: &str) -> String {
    let mut url = Url::parse("https://accounts.spotify.com/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id",             client_id)
        .append_pair("response_type",         "code")
        .append_pair("redirect_uri",          redirect)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge",        challenge)
        .append_pair("state",                 state)
        .append_pair("scope",                 SCOPES);
    url.to_string()
}

#[tauri::command]
pub async fn start_login(app: AppHandle) -> Result<auth::AuthStatus, AppError> {
    // TODO DONE i think, cant remember what this was even for lol
    let db   = app.state::<AppState>().db.clone();
    let auth = app.state::<AppState>().auth.clone();

    let client_id = auth::get_setting_value(&db, "spotify_client_id")
        .await?
        .ok_or_else(|| {
            AppError::Auth(
                "Configure Spotify Client ID in Settings → Spotify API first.".into(),
            )
        })?;

    let verifier    = pkce::generate_verifier();
    let challenge   = pkce::derive_challenge(&verifier);
    let state_token = gen_state();
    let redirect    = redirect_uri();

    let auth_url = build_auth_url(&client_id, &challenge, &state_token, &redirect);

    
    #[cfg(target_os = "linux")]
    open_url_linux(&auth_url).map_err(AppError::Auth)?;

    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_url(&auth_url, None::<&str>)
            .map_err(|e| AppError::Auth(format!("Cannot open browser: {e}")))?;
    }

    let code = http::wait_for_callback(&state_token).await?;

    auth::complete_login(&client_id, &code, &verifier, &redirect, &db, &auth).await
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    token::clear_tokens()?;

    for key in [
        "spotify_token_expires_at",
        "spotify_user_id",
        "spotify_display_name",
        "spotify_email",
        "spotify_product",
        "spotify_image_url",
    ] {
        sqlx::query("DELETE FROM settings WHERE key = ?")
            .bind(key)
            .execute(&state.db)
            .await?;
    }

    *state.auth.write().await = crate::state::AuthState::default();
    Ok(())
}

#[tauri::command]
pub async fn get_auth_status(app: AppHandle) -> Result<auth::AuthStatus, AppError> {
    let db        = app.state::<AppState>().db.clone();
    let auth      = app.state::<AppState>().auth.clone();
    let logged_in = auth.read().await.access_token.is_some();
    auth::build_auth_status(&db, logged_in).await
}



#[derive(serde::Serialize)]
pub struct Profile {
    pub id:                      Option<String>,
    pub display_name:            Option<String>,
    pub email:                   Option<String>,
    pub country:                 Option<String>,
    pub product:                 Option<String>,
    pub followers:               i64,
    pub image_url:               Option<String>,
    pub spotify_url:             Option<String>,
    pub explicit_filter_enabled: bool,
    pub explicit_filter_locked:  bool,
}

#[derive(serde::Deserialize)]
struct SpProfileFull {
    id:               Option<String>,
    display_name:     Option<String>,
    email:            Option<String>,
    country:          Option<String>,
    product:          Option<String>,
    followers:        Option<SpFollowers>,
    images:           Option<Vec<SpImg>>,
    external_urls:    Option<SpExtUrls>,
    explicit_content: Option<SpExplicit>,
}

#[derive(serde::Deserialize)]
struct SpFollowers { total: i64 }
#[derive(serde::Deserialize)]
struct SpImg { url: String }
#[derive(serde::Deserialize)]
struct SpExtUrls { spotify: Option<String> }
#[derive(serde::Deserialize)]
struct SpExplicit { filter_enabled: bool, filter_locked: bool }


async fn cached_profile(db: &sqlx::SqlitePool) -> Result<Profile, AppError> {
    let non_empty = |s: Option<String>| s.filter(|v| !v.is_empty());
    let user_id   = auth::get_setting_value(db, "spotify_user_id").await?;
    let followers = auth::get_setting_value(db, "spotify_followers").await?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Ok(Profile {
        id:                      user_id.clone(),
        display_name:            non_empty(auth::get_setting_value(db, "spotify_display_name").await?),
        email:                   non_empty(auth::get_setting_value(db, "spotify_email").await?),
        country:                 non_empty(auth::get_setting_value(db, "spotify_country").await?),
        product:                 non_empty(auth::get_setting_value(db, "spotify_product").await?),
        followers,
        image_url:               non_empty(auth::get_setting_value(db, "spotify_image_url").await?),
        spotify_url:             user_id.map(|id| format!("https://open.spotify.com/user/{id}")),
        explicit_filter_enabled: false,
        explicit_filter_locked:  false,
    })
}

#[tauri::command]
pub async fn get_profile(app: AppHandle) -> Result<Profile, AppError> {
    let db   = app.state::<AppState>().db.clone();
    let auth_state = app.state::<AppState>().auth.clone();

    let token = auth::get_valid_token(&db, &auth_state).await?;

    let resp = reqwest::Client::new()
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
        if p.explicit_content.as_ref().map(|e| e.filter_enabled).unwrap_or(false) { "1" } else { "0" },
    ).await;

    Ok(Profile {
        id:                      p.id,
        display_name:            p.display_name.filter(|s| !s.is_empty()),
        email:                   p.email.filter(|s| !s.is_empty()),
        country:                 p.country.filter(|s| !s.is_empty()),
        product:                 p.product.filter(|s| !s.is_empty()),
        followers,
        image_url:               p.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        spotify_url:             p.external_urls.and_then(|u| u.spotify),
        explicit_filter_enabled: p.explicit_content.as_ref().map(|e| e.filter_enabled).unwrap_or(false),
        explicit_filter_locked:  p.explicit_content.as_ref().map(|e| e.filter_locked).unwrap_or(false),
    })
}
