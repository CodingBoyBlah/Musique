//! Widevine CDM: install, host process, license exchange and CENC sample decryption.
//!
//! Google's module is downloaded from Chrome's component updater on the user's word (see
//! [`fetch`]), installed per user (see [`store`]), and loaded only in a child process, this
//! same executable started with [`HOST_ARG`] (see [`host`]). The module maps about 80 MB it
//! never frees, and a crash in it should not take the app down; the app talks to it over
//! stdin/stdout (see [`client`], [`link`], [`wire`]). Content keys never leave the module.

#![allow(dead_code, unused_imports)]

mod client;
mod fetch;
mod host;
mod link;
mod store;
mod wire;

use std::path::PathBuf;

pub use client::{shutdown, Cdm, Sample};
pub use fetch::{install, latest, license_text, Release};
pub use host::serve_host;
pub use store::{dir, init, installed, supported, Installed, OVERRIDE_ENV};

use crate::errors::AppError;

/// The argument that starts this executable as the CDM host instead of the app.
pub const HOST_ARG: &str = "--widevine-host";

/// The Widevine system id key format, as HLS playlists name it.
pub const KEY_FORMAT: &str = "urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed";

/// The Widevine DRM system id, the `pssh` box's SystemID.
pub const SYSTEM_ID: [u8; 16] = [
    0xed, 0xef, 0x8b, 0xa9, 0x79, 0xd6, 0x4a, 0xce, 0xa3, 0xc8, 0x27, 0xdc, 0xd5, 0x1d, 0x21, 0xed,
];

/// Where an installed CDM module is, if any.
pub fn module_path() -> Option<PathBuf> {
    store::module_path()
}

/// Whether a usable CDM is installed.
pub fn available() -> bool {
    supported() && module_path().is_some()
}

/// A version-0 `pssh` box for the Widevine system carrying `key_id`: the init data a challenge
/// is made from. The payload is a `WidevinePsshData` message with `algorithm = AESCTR`, the key
/// id, and empty `provider` and `policy`, the shape web players send for Apple Music.
pub fn pssh(key_id: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(key_id.len() + 10);
    data.extend_from_slice(&[0x08, 0x01]); // field 1 (algorithm), varint 1 = AESCTR
    data.push(0x12); // field 2 (key_id), length-delimited
    push_varint(&mut data, key_id.len() as u64);
    data.extend_from_slice(key_id);
    data.extend_from_slice(&[0x1a, 0x00]); // field 3 (provider) = ""
    data.extend_from_slice(&[0x32, 0x00]); // field 6 (policy) = ""

    let size = 4 + 4 + 4 + 16 + 4 + data.len();
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&(size as u32).to_be_bytes());
    out.extend_from_slice(b"pssh");
    out.extend_from_slice(&[0, 0, 0, 0]); // version 0, no flags
    out.extend_from_slice(&SYSTEM_ID);
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(&data);
    out
}

fn push_varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// What the settings screen shows.
#[derive(Clone, Debug)]
pub struct Status {
    pub supported: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
}

pub fn status() -> Status {
    let found = installed();
    let path = module_path();
    Status {
        supported: supported(),
        installed: supported() && path.is_some(),
        version: found.map(|i| i.version),
        path,
    }
}

/// Stops the host and removes every installed module.
pub fn uninstall() -> Result<(), AppError> {
    shutdown();
    let root = dir();
    if !root.exists() {
        return Ok(());
    }
    // A host that just exited may hold the library a moment longer on Windows.
    let mut last = None;
    for _ in 0..10 {
        match std::fs::remove_dir_all(&root) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => last = Some(e),
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    Err(AppError::Playback(format!(
        "cannot remove the Widevine module: {}",
        last.map(|e| e.to_string()).unwrap_or_default()
    )))
}

/// `--widevine-probe`: installs the module if needed, opens it through the host, makes a
/// challenge and times decrypt round trips. Prints what it finds; answers the exit code.
pub fn probe() -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("probe: no runtime: {e}");
            return 2;
        }
    };
    println!("widevine probe: platform {}_{} supported={}", store::OS, store::ARCH, supported());
    println!("install dir: {}", dir().display());
    if !supported() {
        return 1;
    }
    if module_path().is_none() {
        println!("no module installed; installing from Google's component updater...");
        let started = std::time::Instant::now();
        let last = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let result = runtime.block_on(install(move |done, total| {
            let step = done / (2 << 20);
            if step != last.swap(step, std::sync::atomic::Ordering::Relaxed) {
                println!("  {done} / {}", total.map_or("?".into(), |t| t.to_string()));
            }
        }));
        match result {
            Ok(path) => println!("installed {} in {:?}", path.display(), started.elapsed()),
            Err(e) => {
                println!("install FAILED: {e}");
                return 1;
            }
        }
    }
    let st = status();
    println!("module: {:?} version {:?}", st.path, st.version);
    match runtime.block_on(license_text()) {
        Ok(text) => println!("license text: {} chars", text.len()),
        Err(e) => println!("license text FAILED: {e}"),
    }

    let started = std::time::Instant::now();
    let cdm = match Cdm::open() {
        Ok(cdm) => cdm,
        Err(e) => {
            println!("open FAILED: {e}");
            return 1;
        }
    };
    println!(
        "host opened in {:?}: module {} interface {}",
        started.elapsed(),
        cdm.module_version(),
        cdm.interface_version()
    );

    let init = pssh(&[0u8; 16]);
    let started = std::time::Instant::now();
    match cdm.challenge(&init) {
        Ok(challenge) => println!(
            "challenge: {} bytes in {:?} (pssh {} bytes)",
            challenge.len(),
            started.elapsed(),
            init.len()
        ),
        Err(e) => {
            println!("challenge FAILED: {e}");
            return 1;
        }
    }
    // A bogus license must be refused, not crash the host.
    match cdm.accept(b"not a license") {
        Ok(()) => println!("accept(garbage): unexpectedly accepted"),
        Err(e) => println!("accept(garbage) refused as expected: {e}"),
    }

    let mut sample = vec![0x5au8; 1500];
    let rounds = 500u32;
    let mut first_error = None;
    let started = std::time::Instant::now();
    for _ in 0..rounds {
        if let Err(e) = cdm.decrypt(&mut sample, &[0u8; 16], &[1u8; 8], &[(16, 1484)]) {
            first_error.get_or_insert(e.to_string());
        }
    }
    let each = started.elapsed() / rounds;
    println!(
        "decrypt: {rounds} x 1500 B round trips, {each:?} each; result: {}",
        first_error.unwrap_or_else(|| "ok".into())
    );

    let mut buffers: Vec<Vec<u8>> = (0..64).map(|_| vec![0x33u8; 1500]).collect();
    let iv = [2u8; 16];
    let mut samples: Vec<Sample> = buffers
        .iter_mut()
        .map(|b| Sample { data: b, iv: &iv, subsamples: &[] })
        .collect();
    let started = std::time::Instant::now();
    let many = cdm.decrypt_many(&[0u8; 16], &mut samples);
    println!(
        "decrypt_many: 64 samples in {:?}; result: {}",
        started.elapsed(),
        many.err().map_or("ok".into(), |e| e.to_string())
    );

    // The host must still be alive after all that.
    match cdm.challenge(&init) {
        Ok(c) => println!("second challenge: {} bytes (host still healthy)", c.len()),
        Err(e) => {
            println!("second challenge FAILED: {e}");
            return 1;
        }
    }
    drop(cdm);
    shutdown();
    println!("probe OK");
    0
}
