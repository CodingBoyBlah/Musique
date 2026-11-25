//! Widevine CDM: install, host process, license exchange and CENC sample decryption.
//!
//! Contract used by `crate::apple::playback`. Owned by the widevine worker; the bodies below are
//! placeholders until that work lands.

#![allow(dead_code)]

use std::path::PathBuf;

use crate::errors::AppError;

/// The argument that starts this executable as the CDM host instead of the app.
pub const HOST_ARG: &str = "--widevine-host";

/// The Widevine system id key format, as HLS playlists name it.
pub const KEY_FORMAT: &str = "urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed";

/// Where an installed CDM module is, if any.
pub fn module_path() -> Option<PathBuf> {
    None
}

/// Whether a usable CDM is installed.
pub fn available() -> bool {
    module_path().is_some()
}

/// A `pssh` box for the Widevine system carrying `key_id`, the init data a challenge is made from.
pub fn pssh(_key_id: &[u8]) -> Vec<u8> {
    Vec::new()
}

/// One CDM session. Cheap to clone; every method blocks and is safe to call from any thread.
#[derive(Clone)]
pub struct Cdm;

impl Cdm {
    pub fn open() -> Result<Cdm, AppError> {
        Err(AppError::Playback("widevine is not available yet".into()))
    }

    pub fn challenge(&self, _init_data: &[u8]) -> Result<Vec<u8>, AppError> {
        Err(AppError::Playback("widevine is not available yet".into()))
    }

    pub fn accept(&self, _license: &[u8]) -> Result<(), AppError> {
        Err(AppError::Playback("widevine is not available yet".into()))
    }

    /// Decrypts one CENC (`cenc`, AES-CTR) sample in place. `subsamples` are
    /// `(clear_bytes, encrypted_bytes)` pairs; empty means the whole sample is encrypted.
    pub fn decrypt(
        &self,
        _data: &mut [u8],
        _key_id: &[u8],
        _iv: &[u8],
        _subsamples: &[(u32, u32)],
    ) -> Result<(), AppError> {
        Err(AppError::Playback("widevine is not available yet".into()))
    }
}
