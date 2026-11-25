//! Downloading the module from Google's component updater, the service every Chrome asks for
//! its Widevine component.
//!
//! The service answers with the current version, the archive's sha256 and where to download
//! it. The archive is a CRX3: `"Cr24"`, format version 3, the length of a signed header, the
//! header, then an ordinary zip. Its sha256 is checked against the announced one before
//! anything is read out of it (the CRX signature itself is not checked; TLS and the checksum
//! from Google's own service vouch for it).

use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::store::{self, ARCH, LIBRARY, LICENSE, MANIFEST, OS};
use crate::errors::AppError;

const UPDATE_URL: &str = "https://update.googleapis.com/service/update2/json";
/// The Widevine component's id on that service.
const APP_ID: &str = "oimompecagnajdejgnnjijobebaeigek";
/// The updater version the request claims; the service answers older ones with nothing.
const UPDATER_VERSION: &str = "151.0.7922.173";
/// The component version the request claims to have, old enough that any answer is newer.
const HAVE: &str = "1.4.9.1088";

#[derive(Clone, Debug)]
pub struct Release {
    pub version: String,
    pub size: u64,
    sha256: String,
    urls: Vec<String>,
}

/// An archive downloaded (and checked) to show its terms, kept for the install that follows.
static OFFERED: Mutex<Option<(Release, Vec<u8>)>> = Mutex::new(None);

fn client() -> Result<&'static reqwest::Client, AppError> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    if let Some(c) = CLIENT.get() {
        return Ok(c);
    }
    let built = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Network(format!("cannot build an http client: {e}")))?;
    Ok(CLIENT.get_or_init(|| built))
}

fn unsupported() -> AppError {
    AppError::Playback("Google builds no Widevine module for this computer".into())
}

/// Asks the service what it offers for this platform.
pub async fn latest() -> Result<Release, AppError> {
    if !store::supported() {
        return Err(unsupported());
    }
    let request = json!({
        "request": {
            "@os": "",
            "@updater": "",
            "acceptformat": "crx3,download,puff,run,xz,zucc",
            "apps": [{
                "appid": APP_ID,
                "installsource": "ondemand",
                "updatecheck": {},
                "version": HAVE,
            }],
            "dedup": "cr",
            "ismachine": false,
            "arch": ARCH,
            "os": { "arch": ARCH, "platform": OS },
            "protocol": "4.0",
            "updaterversion": UPDATER_VERSION,
        }
    });
    let net = |e: reqwest::Error| AppError::Network(format!("google's update service: {e}"));
    let text = client()?
        .post(UPDATE_URL)
        .json(&request)
        .send()
        .await
        .map_err(net)?
        .error_for_status()
        .map_err(net)?
        .text()
        .await
        .map_err(net)?;
    parse_release(&text)
}

fn parse_release(text: &str) -> Result<Release, AppError> {
    let bad = |why: &str| AppError::Network(format!("google's update service: {why}"));
    // The JSON sits behind a `)]}'` guard line.
    let body = text.trim_start();
    let body = if body.starts_with(")]}'") {
        body.split_once('\n').map_or("", |(_, rest)| rest)
    } else {
        body
    };
    let answer: Value = serde_json::from_str(body).map_err(|_| bad("the answer is not json"))?;
    let check = &answer["response"]["apps"][0]["updatecheck"];
    let status = check["status"].as_str().unwrap_or("");
    if status != "ok" {
        return Err(bad(&format!("no module offered for this platform ({status})")));
    }
    let version = check["nextversion"]
        .as_str()
        .filter(|v| store::version_key(v).is_some())
        .ok_or_else(|| bad("the offer names no version"))?
        .to_string();
    let op = &check["pipelines"][0]["operations"][0];
    let size = op["size"].as_u64().unwrap_or(0);
    let sha256 = op["out"]["sha256"]
        .as_str()
        .ok_or_else(|| bad("the offer carries no checksum"))?
        .to_ascii_lowercase();
    let mut urls: Vec<String> = op["urls"]
        .as_array()
        .map(|list| list.iter().filter_map(|u| u["url"].as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    // Prefer https; the service lists a plain http mirror first.
    urls.sort_by_key(|u| !u.starts_with("https://"));
    if urls.is_empty() {
        return Err(bad("the offer names nowhere to download from"));
    }
    Ok(Release {
        version,
        size,
        sha256,
        urls,
    })
}

/// Downloads the archive from the first mirror that serves it, and checks its sha256.
async fn download(
    release: &Release,
    progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
) -> Result<Vec<u8>, AppError> {
    let mut last = String::from("no mirror answered");
    for url in &release.urls {
        let response = match client()?.get(url).send().await.and_then(|r| r.error_for_status()) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("widevine: mirror failed: {e}");
                last = e.to_string();
                continue;
            }
        };
        let total = response.content_length().or((release.size > 0).then_some(release.size));
        let mut response = response;
        let mut bytes = Vec::with_capacity(total.unwrap_or(0) as usize);
        let mut hasher = Sha256::new();
        progress(0, total);
        let mut failed = None;
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    hasher.update(&chunk);
                    bytes.extend_from_slice(&chunk);
                    progress(bytes.len() as u64, total);
                }
                Ok(None) => break,
                Err(e) => {
                    failed = Some(e.to_string());
                    break;
                }
            }
        }
        if let Some(e) = failed {
            log::warn!("widevine: download broke off: {e}");
            last = e;
            continue;
        }
        let digest: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if digest != release.sha256 {
            return Err(AppError::Network(
                "the downloaded Widevine module does not match Google's checksum".into(),
            ));
        }
        return Ok(bytes);
    }
    Err(AppError::Network(format!("cannot download the Widevine module: {last}")))
}

/// The zip inside a CRX3.
fn crx_zip(archive: &[u8]) -> Result<&[u8], AppError> {
    let bad = |why: &str| AppError::Network(format!("the Widevine archive {why}"));
    if archive.len() < 12 || &archive[..4] != b"Cr24" {
        return Err(bad("is not a crx"));
    }
    let word = |at: usize| u32::from_le_bytes([archive[at], archive[at + 1], archive[at + 2], archive[at + 3]]);
    if word(4) != 3 {
        return Err(bad("is not crx3"));
    }
    let header = word(8) as usize;
    archive.get(12 + header..).ok_or_else(|| bad("ends inside its header"))
}

/// One file out of the zip, or `None` when the archive has no such entry.
fn entry(zip: &[u8], name: &str) -> Result<Option<Vec<u8>>, AppError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip))
        .map_err(|e| AppError::Network(format!("the Widevine archive is damaged: {e}")))?;
    let mut file = match archive.by_name(name) {
        Ok(f) => f,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(AppError::Network(format!("the Widevine archive is damaged: {e}"))),
    };
    let mut out = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut out)
        .map_err(|e| AppError::Network(format!("the Widevine archive is damaged: {e}")))?;
    Ok(Some(out))
}

fn license_of(archive: &[u8]) -> Result<String, AppError> {
    let text = entry(crx_zip(archive)?, LICENSE)?
        .ok_or_else(|| AppError::Network("the Widevine archive carries no license".into()))?;
    Ok(String::from_utf8_lossy(&text).trim().to_string())
}

/// Google's terms for the module: the installed copy's, or else the current release's, which
/// is downloaded (and kept for [`install`]) to read them.
pub async fn license_text() -> Result<String, AppError> {
    if let Some(text) = store::installed_license() {
        return Ok(text);
    }
    let release = latest().await?;
    if let Some((offered, archive)) = OFFERED.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        if offered.version == release.version {
            return license_of(archive);
        }
    }
    let archive = download(&release, &|_, _| {}).await?;
    let text = license_of(&archive)?;
    *OFFERED.lock().unwrap_or_else(|e| e.into_inner()) = Some((release, archive));
    Ok(text)
}

/// Installs the module Google currently offers (unless that version is installed already) and
/// answers the library's path. `progress` gets `(bytes done, bytes total)` while downloading.
pub async fn install(
    progress: impl Fn(u64, Option<u64>) + Send + Sync + 'static,
) -> Result<PathBuf, AppError> {
    let release = latest().await?;
    let folder = store::dir().join(&release.version);
    let library = folder.join(LIBRARY);
    if library.is_file() {
        progress(release.size, Some(release.size));
        return Ok(library);
    }
    let cached = OFFERED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .filter(|(offered, _)| offered.version == release.version)
        .map(|(_, archive)| archive);
    let archive = match cached {
        Some(archive) => {
            progress(archive.len() as u64, Some(archive.len() as u64));
            archive
        }
        None => download(&release, &progress).await?,
    };
    let version = release.version.clone();
    tokio::task::spawn_blocking(move || unpack(&archive, &version))
        .await
        .map_err(|e| AppError::Playback(format!("the install stopped: {e}")))?
}

/// Writes the module into a staging folder and renames it into place, so a half-written
/// install is never picked up.
fn unpack(archive: &[u8], version: &str) -> Result<PathBuf, AppError> {
    let zip = crx_zip(archive)?;
    let inner = format!("_platform_specific/{OS}_{ARCH}/{LIBRARY}");
    let module = entry(zip, &inner)?
        .filter(|m| !m.is_empty())
        .ok_or_else(|| AppError::Playback("the Widevine archive has no module for this computer".into()))?;

    let root = store::dir();
    let disk = |e: std::io::Error| AppError::Playback(format!("cannot install the Widevine module: {e}"));
    std::fs::create_dir_all(&root).map_err(disk)?;
    let staging = root.join(format!(".staging-{version}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(disk)?;
    let written = (|| -> std::io::Result<()> {
        std::fs::write(staging.join(LIBRARY), &module)?;
        for name in [LICENSE, MANIFEST] {
            if let Ok(Some(bytes)) = entry(zip, name) {
                std::fs::write(staging.join(name), bytes)?;
            }
        }
        Ok(())
    })();
    if let Err(e) = written {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(disk(e));
    }
    let folder = root.join(version);
    if folder.exists() {
        // A leftover without its library (it would have been used otherwise).
        let _ = std::fs::remove_dir_all(&folder);
    }
    if let Err(e) = std::fs::rename(&staging, &folder) {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(disk(e));
    }
    store::prune(&folder);
    log::info!("widevine: installed module {version}");
    Ok(folder.join(LIBRARY))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_offer() {
        let text = r#")]}'
{"response":{"apps":[{"updatecheck":{"status":"ok","nextversion":"4.10.2891.0","pipelines":[{"operations":[{"size":10,"out":{"sha256":"AB"},"urls":[{"url":"http://a/x.crx3"},{"url":"https://b/x.crx3"}]}]}]}}]}}"#;
        let r = parse_release(text).unwrap();
        assert_eq!(r.version, "4.10.2891.0");
        assert_eq!(r.sha256, "ab");
        assert_eq!(r.urls[0], "https://b/x.crx3");
    }
}
