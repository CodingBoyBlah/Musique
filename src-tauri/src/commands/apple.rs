//! Apple Music sign-in, sign-out and account status.
//!
//! Signing in opens music.apple.com's own login page in a private window of the app. Apple
//! finishes on music.apple.com with the account's `media-user-token` cookie set, and that
//! cookie is the whole account: the window is closed as soon as it appears and the cookie is
//! handed to `crate::apple::client::sign_in`, which checks the subscription and stores it.
//!
//! The window loads a remote page, so it is given no IPC: the app's capability only names the
//! main window.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::{auth, errors::AppError, state::AppState};

/// The page the sign-in window opens.
const SIGN_IN_URL: &str = "https://music.apple.com/login";
/// What the account cookie is read for.
const COOKIE_URL: &str = "https://music.apple.com/";
/// The cookie that proves the account is signed in.
const ACCOUNT_COOKIE: &str = "media-user-token";
/// The sign-in window's label. Not in any capability, so the page gets no IPC.
const WINDOW: &str = "apple-sign-in";
/// How often the window's cookies are read.
const POLL: Duration = Duration::from_millis(400);
/// How long a sign-in may sit open before it is given up on.
const PATIENCE: Duration = Duration::from_secs(15 * 60);

/// The `settings` row holding the signed-in Apple account, as JSON.
pub const ACCOUNT_SETTING: &str = "apple_account";

/// What is kept of the signed-in Apple account for the auth status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct StoredAccount {
    storefront:   String,
    display_name: Option<String>,
    handle:       Option<String>,
    image_url:    Option<String>,
}

/// Opens Apple's sign-in page, waits for the account cookie, and signs Apple Music in as the
/// app's provider. Fails if the window is closed first.
#[tauri::command]
pub async fn apple_sign_in(app: AppHandle) -> Result<auth::AuthStatus, AppError> {
    if let Some(stale) = app.get_webview_window(WINDOW) {
        let _ = stale.destroy();
    }
    let url: tauri::Url = SIGN_IN_URL.parse().map_err(|e| AppError::InvalidInput(format!("{e}")))?;
    let window = WebviewWindowBuilder::new(&app, WINDOW, WebviewUrl::External(url))
        .title("Sign in to Apple Music")
        .inner_size(500.0, 760.0)
        .min_inner_size(380.0, 560.0)
        .center()
        .incognito(true)
        .focused(true)
        .build()
        .map_err(|e| AppError::Auth(format!("cannot open the Apple Music sign-in window: {e}")))?;

    let cookie_url: tauri::Url = COOKIE_URL.parse().map_err(|e| AppError::InvalidInput(format!("{e}")))?;
    let started = Instant::now();
    let header = loop {
        tokio::time::sleep(POLL).await;
        // Closed by the listener: the sign-in is over.
        let Some(window) = app.get_webview_window(WINDOW) else {
            return Err(AppError::Auth("Apple Music sign-in was cancelled".into()));
        };
        if started.elapsed() > PATIENCE {
            let _ = window.destroy();
            return Err(AppError::Auth("Apple Music sign-in timed out".into()));
        }
        // Read off the main thread: this is an async command, which is what keeps the cookie
        // read from deadlocking WebView2.
        let cookies = match window.cookies_for_url(cookie_url.clone()) {
            Ok(cookies) => cookies,
            Err(e) => {
                log::debug!("apple sign-in: cannot read cookies yet: {e}");
                continue;
            }
        };
        let signed_in = cookies
            .iter()
            .any(|cookie| cookie.name() == ACCOUNT_COOKIE && !cookie.value().is_empty());
        if signed_in {
            break cookies
                .iter()
                .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
                .collect::<Vec<_>>()
                .join("; ");
        }
    };
    let _ = window.destroy();
    finish_sign_in(&app, &header).await
}

/// Signs in with a pasted `media-user-token` (or a whole Cookie header), for when the window
/// cannot be used.
#[tauri::command]
pub async fn apple_sign_in_with_token(app: AppHandle, token: String) -> Result<auth::AuthStatus, AppError> {
    finish_sign_in(&app, &token).await
}

/// Closes a sign-in window that is still open, which ends the waiting `apple_sign_in`.
#[tauri::command]
pub async fn apple_cancel_sign_in(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.destroy();
    }
    Ok(())
}

async fn finish_sign_in(app: &AppHandle, cookies: &str) -> Result<auth::AuthStatus, AppError> {
    let account = crate::apple::client::sign_in(cookies).await?;
    let db = app.state::<AppState>().db.clone();

    // A new identity: anything still running for the previous one stops writing.
    {
        let _commit = auth::session_commit().await;
        auth::bump_auth_epoch();
    }
    let stored = StoredAccount {
        storefront:   account.storefront.clone(),
        display_name: account.display_name.clone(),
        handle:       account.handle.clone(),
        image_url:    account.image_url.clone(),
    };
    let body = serde_json::to_string(&stored).map_err(|e| AppError::InvalidInput(e.to_string()))?;
    auth::upsert_setting(&db, ACCOUNT_SETTING, &body).await?;
    crate::apple::set_active(&db, true).await?;
    let _ = sqlx::query("DELETE FROM settings WHERE key = 'spotify_signed_out'")
        .execute(&db)
        .await;
    log::info!("apple: signed in, storefront {}", account.storefront);
    Ok(status_from(Some(stored)))
}

/// The auth status of the signed-in Apple account, or signed out when no token is kept.
pub async fn auth_status(db: &sqlx::SqlitePool) -> auth::AuthStatus {
    if crate::apple::auth::load_user_token().is_none() {
        return status_from(None);
    }
    let stored = auth::get_setting_value(db, ACCOUNT_SETTING)
        .await
        .ok()
        .flatten()
        .and_then(|body| serde_json::from_str::<StoredAccount>(&body).ok())
        .unwrap_or_default();
    status_from(Some(stored))
}

fn status_from(account: Option<StoredAccount>) -> auth::AuthStatus {
    match account {
        None => auth::AuthStatus {
            logged_in:    false,
            user_id:      None,
            display_name: None,
            email:        None,
            product:      None,
            image_url:    None,
            provider:     "apple".into(),
        },
        Some(account) => auth::AuthStatus {
            logged_in:    true,
            user_id:      Some(
                account
                    .handle
                    .clone()
                    .unwrap_or_else(|| format!("apple-{}", account.storefront)),
            ),
            display_name: account.display_name.or(account.handle).or(Some("Apple Music".into())),
            email:        None,
            product:      Some("apple".into()),
            image_url:    account.image_url,
            provider:     "apple".into(),
        },
    }
}

/// Signs the Apple account out: forgets its token and session, and hands the app back to
/// Spotify. The caller purges the local library afterwards, as for any sign-out.
pub async fn sign_out(db: &sqlx::SqlitePool) {
    crate::apple::auth::clear_user_token();
    crate::apple::client::reset();
    let _ = sqlx::query("DELETE FROM settings WHERE key = ?")
        .bind(ACCOUNT_SETTING)
        .execute(db)
        .await;
    if let Err(e) = crate::apple::set_active(db, false).await {
        log::warn!("apple: cannot record the sign-out: {e}");
    }
}
