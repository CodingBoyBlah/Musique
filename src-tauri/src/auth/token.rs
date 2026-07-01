use crate::errors::AppError;
use keyring::Entry;
use serde::Deserialize;

const SERVICE: &str = "spotify-client";

pub fn store_token(account: &str, value: &str) -> Result<(), AppError> {
    Entry::new(SERVICE, account)?.set_password(value)?;
    Ok(())
}

fn load_token(account: &str) -> Result<Option<String>, AppError> {
    match Entry::new(SERVICE, account)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

pub fn load_tokens() -> Result<Option<(String, String)>, AppError> {
    let at = match load_token("access_token")? {
        Some(t) => t,
        None => return Ok(None),
    };
    let rt = match load_token("refresh_token")? {
        Some(t) => t,
        None => return Ok(None),
    };
    Ok(Some((at, rt)))
}

pub fn clear_tokens() -> Result<(), AppError> {
    for account in ["access_token", "refresh_token"] {
        match Entry::new(SERVICE, account)?.delete_password() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(AppError::Keyring(e.to_string())),
        }
    }
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token:  String,
    pub expires_in:    u64,
    pub refresh_token: Option<String>,
}
