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
        // An empty value is a tombstone left by `clear_tokens` when the store
        // refused to delete the entry. It is not a usable token.
        Ok(v) if v.trim().is_empty() => Ok(None),
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

/// Remove the stored tokens.
///
/// A credential store that refuses to delete must not leave a usable token
/// behind: the next launch would read it and sign the user back in after they
/// had signed out. So a failed delete falls back to overwriting the entry with
/// an empty value, which `load_token` treats as absent.
pub fn clear_tokens() -> Result<(), AppError> {
    let mut failure: Option<String> = None;

    for account in ["access_token", "refresh_token"] {
        let entry = match Entry::new(SERVICE, account) {
            Ok(e) => e,
            Err(e) => {
                failure.get_or_insert(e.to_string());
                continue;
            }
        };

        match entry.delete_password() {
            Ok(()) | Err(keyring::Error::NoEntry) => continue,
            Err(e) => {
                eprintln!("[auth] could not delete {account} from the keyring: {e}");
                if let Err(e2) = entry.set_password("") {
                    failure.get_or_insert(format!("{e} (and blanking it failed: {e2})"));
                }
            }
        }
    }

    match failure {
        Some(e) => Err(AppError::Keyring(e)),
        None => Ok(()),
    }
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token:  String,
    pub expires_in:    u64,
    pub refresh_token: Option<String>,
}
