//! Getting Spotify's Soloist player onto this computer.
//!
//! Spotify forbids redistributing the binary, so it is never bundled: it is
//! downloaded for the listener, straight from Spotify's own server, the first
//! time they turn Soloist on. Spotify publishes no checksums or signatures, so
//! the SHA-256 kept here only says the installed copy has not changed since it
//! was downloaded.
//!
//! A build stops working 90 days after it was built. Each one lives in a
//! folder named after its digest and is never written to again; `current.json`
//! and `pending.json` point at the one to run and the one to move to, so a
//! running executable is never replaced underneath itself.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CDN: &str = "soloist-builds.spotifycdn.com";
/// The archive is about 20 MB. Anything far past that is not it.
const ARCHIVE_LIMIT: usize = 80 * 1024 * 1024;
const EXPANDED_LIMIT: u64 = 160 * 1024 * 1024;
pub const DAY: u64 = 86_400;
/// How long a build works for, counted from when Spotify built it.
pub const LIFETIME: u64 = 90 * DAY;
/// How old a build is before a newer one is looked for.
const RENEW_AFTER: u64 = 60 * DAY;

/// The files taken out of Spotify's archive. Nothing else in it is written.
const NOTICES: [&str; 2] = ["THIRD_PARTY_LICENSES.txt", "CHANGELOG.md"];

/// The archive Spotify publishes for this kind of computer, if any.
pub fn artifact(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("linux", "x86_64") => Some("soloist_release_x86_64.tar.gz"),
        ("linux", "aarch64") => Some("soloist_release_arm64.tar.gz"),
        ("linux", "arm") => Some("soloist_release_arm32.tar.gz"),
        ("macos", "aarch64") => Some("soloist_release_osx_arm64.tar.gz"),
        ("windows", "x86_64") => Some("soloist_release_windows_x86_64.zip"),
        _ => None,
    }
}

pub fn supported() -> bool {
    artifact(std::env::consts::OS, std::env::consts::ARCH).is_some()
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

pub fn executable() -> &'static str {
    if cfg!(windows) { "soloist.exe" } else { "soloist" }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Build {
    pub version: String,
    /// When Spotify built it, in seconds since the epoch.
    pub built:   u64,
    /// When a newer one was last looked for.
    pub checked: u64,
    /// SHA-256 of the executable, and the name of the folder it is in.
    pub digest:  String,
}

impl Build {
    pub fn path(&self, root: &Path) -> PathBuf {
        root.join(&self.digest).join(executable())
    }

    pub fn expires(&self) -> u64 {
        self.built.saturating_add(LIFETIME)
    }

    /// Built in the past (give or take a day of clock error), not yet
    /// expired, and named by a real digest rather than a path.
    pub fn usable(&self, at: u64) -> bool {
        self.built <= at.saturating_add(DAY)
            && at < self.expires()
            && self.digest.len() == 64
            && self.digest.bytes().all(|b| b.is_ascii_hexdigit())
    }

    /// Old enough to look for its replacement, at most once a day.
    pub fn due(&self, at: u64) -> bool {
        at.saturating_sub(self.built) >= RENEW_AFTER && at.saturating_sub(self.checked) >= DAY
    }
}

fn digest_of(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// The build `name` (`current.json` or `pending.json`) points at, when it is
/// still usable and the executable is the one that was downloaded.
pub async fn saved(root: &Path, name: &str) -> Option<Build> {
    let build: Build = serde_json::from_slice(&tokio::fs::read(root.join(name)).await.ok()?).ok()?;
    if !build.usable(now()) {
        return None;
    }
    let bytes = tokio::fs::read(build.path(root)).await.ok()?;
    (digest_of(&bytes) == build.digest).then_some(build)
}

pub async fn remember(root: &Path, name: &str, build: &Build) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(build).map_err(std::io::Error::other)?;
    // written beside it and moved over, so a crash never leaves half a pointer
    let staged = root.join(format!("{name}.tmp"));
    tokio::fs::write(&staged, bytes).await?;
    tokio::fs::rename(&staged, root.join(name)).await
}

/// Remove builds nothing points at any more, and staging folders a download
/// that never finished left behind.
pub async fn collect(root: &Path, keep: &[String]) {
    let Ok(mut entries) = tokio::fs::read_dir(root).await else {
        return;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();
        let old_build = name.len() == 64
            && name.bytes().all(|b| b.is_ascii_hexdigit())
            && !keep.contains(&name);
        let old_stage = name.starts_with("staging-")
            && entry
                .metadata()
                .await
                .ok()
                .and_then(|meta| meta.modified().ok())
                .and_then(|at| at.elapsed().ok())
                .is_some_and(|age| age > Duration::from_secs(DAY));
        if (old_build || old_stage) && entry.file_type().await.is_ok_and(|kind| kind.is_dir()) {
            let _ = tokio::fs::remove_dir_all(entry.path()).await;
        }
    }
}

/// Reads `soloist 1.2.3 ... build 1790000000 ...`, what `soloist --version`
/// prints: the version and when it was built.
pub fn parse_version(text: &str, at: u64) -> Result<(String, u64), String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.first() != Some(&"soloist") {
        return Err("Unexpected Soloist version response".into());
    }
    let version = words.get(1).ok_or("Missing Soloist version")?.to_string();
    let built: u64 = words
        .iter()
        .position(|word| *word == "build")
        .and_then(|at| words.get(at + 1))
        .and_then(|word| word.parse().ok())
        .ok_or("Missing Soloist build date")?;
    if built > at.saturating_add(DAY) || at >= built.saturating_add(LIFETIME) {
        return Err(
            "Spotify's downloaded Soloist build has expired. Check the computer's date and try again later."
                .into(),
        );
    }
    Ok((version, built))
}

/// The name to save an archive entry under: only the executable and the
/// notices, only from a plain relative path. `Ok(None)` skips the entry.
fn accepted(path: &Path, exe: &str) -> Result<Option<String>, String> {
    use std::path::Component;
    if !path.components().all(|part| matches!(part, Component::Normal(_) | Component::CurDir)) {
        return Err("Unsafe path in Soloist archive".into());
    }
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err("Invalid Soloist archive entry".into());
    };
    if name.contains('\\') {
        return Err("Unsafe path in Soloist archive".into());
    }
    Ok((name == exe || NOTICES.contains(&name)).then(|| name.to_string()))
}

/// Writes the accepted entries of the archive in `bytes` into `destination`.
/// Links and special files are refused, sizes are bounded, and nothing is
/// written outside `destination`.
fn extract(bytes: &[u8], zipped: bool, destination: &Path, exe: &str) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    let mut seen = std::collections::HashSet::new();
    let mut total = 0u64;
    let mut save = |name: &str, size: u64, reader: &mut dyn Read| -> Result<(), String> {
        total = total.checked_add(size).ok_or("Soloist archive too large")?;
        if total > EXPANDED_LIMIT || !seen.insert(name.to_owned()) {
            return Err("Invalid Soloist archive size or duplicate entry".into());
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination.join(name))
            .map_err(|e| e.to_string())?;
        let written = std::io::copy(&mut reader.take(size + 1), &mut file).map_err(|e| e.to_string())?;
        if written != size {
            return Err("Truncated Soloist archive".into());
        }
        file.sync_all().map_err(|e| e.to_string())
    };

    if zipped {
        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        if archive.len() > 64 {
            return Err("Too many Soloist archive entries".into());
        }
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
            if entry.is_symlink() {
                return Err("Link in Soloist archive".into());
            }
            let path = PathBuf::from(entry.name());
            let name = accepted(&path, exe)?;
            if let (true, Some(name)) = (entry.is_file(), name) {
                let size = entry.size();
                save(&name, size, &mut entry)?;
            }
        }
    } else {
        let decoded = flate2::read::GzDecoder::new(bytes).take(EXPANDED_LIMIT + 1);
        let mut archive = tar::Archive::new(decoded);
        for (index, entry) in archive.entries().map_err(|e| e.to_string())?.enumerate() {
            if index >= 64 {
                return Err("Too many Soloist archive entries".into());
            }
            let mut entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.header().entry_type();
            if !kind.is_file() && !kind.is_dir() {
                return Err("Link or special file in Soloist archive".into());
            }
            let path = entry.path().map_err(|e| e.to_string())?.into_owned();
            let name = accepted(&path, exe)?;
            if let (true, Some(name)) = (kind.is_file(), name) {
                let size = entry.size();
                save(&name, size, &mut entry)?;
            }
        }
    }

    if !seen.contains(exe) {
        return Err("Soloist executable missing from archive".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(destination.join(exe), std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Download Spotify's current build for this computer into `root` and say
/// what it is. `progress` is told bytes received and the total, when known.
pub async fn download(root: &Path, progress: &(dyn Fn(u64, u64) + Send + Sync)) -> Result<Build, String> {
    let artifact = artifact(std::env::consts::OS, std::env::consts::ARCH)
        .ok_or("Spotify does not publish Soloist for this kind of computer")?;
    let mut response = crate::http::client()
        .get(format!("https://{CDN}/{artifact}"))
        .timeout(Duration::from_secs(180))
        .send()
        .await
        .and_then(|response| response.error_for_status())
        .map_err(|e| format!("Soloist download failed: {e}"))?;
    if response.url().scheme() != "https" || response.url().host_str() != Some(CDN) {
        return Err("Soloist download left Spotify's download server".into());
    }
    let total = response.content_length().unwrap_or(0);
    if total > ARCHIVE_LIMIT as u64 {
        return Err("Soloist download is too large".into());
    }
    let mut bytes = Vec::with_capacity(total as usize);
    let mut reported = std::time::Instant::now();
    progress(0, total);
    while let Some(chunk) = response.chunk().await.map_err(|e| format!("Soloist download failed: {e}"))? {
        if bytes.len().saturating_add(chunk.len()) > ARCHIVE_LIMIT {
            return Err("Soloist download is too large".into());
        }
        bytes.extend_from_slice(&chunk);
        if reported.elapsed() >= Duration::from_millis(150) {
            progress(bytes.len() as u64, total);
            reported = std::time::Instant::now();
        }
    }
    progress(bytes.len() as u64, total);

    tokio::fs::create_dir_all(root).await.map_err(|e| e.to_string())?;
    let stage = root.join(format!("staging-{:016x}", rand::random::<u64>()));
    let extracting = stage.clone();
    let extracted = tokio::task::spawn_blocking(move || {
        extract(&bytes, artifact.ends_with(".zip"), &extracting, executable())
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|result| result);

    let installed = match extracted {
        Ok(()) => install_staged(root, &stage).await,
        Err(error) => Err(error),
    };
    let _ = tokio::fs::remove_dir_all(&stage).await;
    installed
}

/// Ask the staged executable what it is, then move it to the folder named
/// after its digest.
async fn install_staged(root: &Path, stage: &Path) -> Result<Build, String> {
    let mut command = tokio::process::Command::new(stage.join(executable()));
    command.arg("--version").kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // no console window
    let output = tokio::time::timeout(Duration::from_secs(10), command.output())
        .await
        .map_err(|_| "Downloaded Soloist did not answer".to_string())?
        .map_err(|e| format!("Downloaded Soloist could not run on this computer: {e}"))?;
    if !output.status.success() {
        return Err("Downloaded Soloist could not run on this computer".into());
    }
    let (version, built) = parse_version(&String::from_utf8_lossy(&output.stdout), now())?;
    let exe = tokio::fs::read(stage.join(executable())).await.map_err(|e| e.to_string())?;
    let build = Build { version, built, checked: now(), digest: digest_of(&exe) };
    let destination = root.join(&build.digest);
    if !destination.exists() {
        tokio::fs::rename(stage, &destination).await.map_err(|e| e.to_string())?;
    } else {
        // the same build is already there; it has to still be the same bytes
        let present = tokio::fs::read(build.path(root)).await.map_err(|e| e.to_string())?;
        if digest_of(&present) != build.digest {
            return Err("The installed Soloist build failed its integrity check. Remove it and try again.".into());
        }
    }
    Ok(build)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spotify_publishes_for_some_computers_only() {
        assert!(artifact("macos", "x86_64").is_none());
        assert!(artifact("windows", "aarch64").is_none());
        assert!(artifact("windows", "x86_64").unwrap().ends_with(".zip"));
        for arch in ["arm", "aarch64", "x86_64"] {
            assert!(artifact("linux", arch).unwrap().ends_with(".tar.gz"));
        }
    }

    #[test]
    fn a_build_expires_ninety_days_after_it_was_built() {
        let at = 1_800_000_000;
        assert!(parse_version(&format!("soloist 1.3 build {} (date)", at - 89 * DAY), at).is_ok());
        assert!(parse_version(&format!("soloist 1.3 build {} (date)", at - 90 * DAY), at).is_err());
        assert!(parse_version("soloist 1.3 build nonsense", at).is_err());
        assert!(parse_version("something else 1.3 build 5", at).is_err());
        assert_eq!(
            parse_version(&format!("soloist 1.3.0 (abc) build {at}"), at),
            Ok(("1.3.0".to_string(), at))
        );

        let build = Build { version: "1".into(), built: at - 61 * DAY, checked: at - 2 * DAY, digest: "a".repeat(64) };
        assert!(build.usable(at) && build.due(at));
        assert!(!Build { checked: at, ..build.clone() }.due(at));
        assert!(!Build { built: at - 10 * DAY, ..build.clone() }.due(at));
        assert!(!Build { built: at - 91 * DAY, ..build.clone() }.usable(at));
        assert!(!Build { digest: "../../evil".into(), ..build }.usable(at));
    }

    #[test]
    fn only_the_player_and_its_notices_come_out_of_the_archive() {
        assert_eq!(accepted(Path::new("soloist/soloist.exe"), "soloist.exe"), Ok(Some("soloist.exe".into())));
        assert_eq!(accepted(Path::new("./CHANGELOG.md"), "soloist"), Ok(Some("CHANGELOG.md".into())));
        assert_eq!(accepted(Path::new("soloist/README.md"), "soloist"), Ok(None));
        assert!(accepted(Path::new("../soloist"), "soloist").is_err());
        assert!(accepted(Path::new("/usr/bin/soloist"), "soloist").is_err());
    }

    /// Downloads Spotify's real build and asks it what it is. Needs the
    /// network, so it only runs when asked for: `cargo test -- --ignored soloist`.
    #[tokio::test]
    #[ignore]
    async fn live_download_installs_a_build_that_identifies_itself() {
        if !supported() {
            return;
        }
        let root = std::env::temp_dir().join(format!("musique-soloist-live-{:016x}", rand::random::<u64>()));
        let build = download(&root, &|_, _| {}).await.expect("download");
        println!("soloist {} built {} digest {}", build.version, build.built, build.digest);
        assert!(build.usable(now()));
        assert!(build.path(&root).is_file());
        remember(&root, "current.json", &build).await.unwrap();
        assert_eq!(saved(&root, "current.json").await, Some(build.clone()));
        // a second download of the same build lands on the same folder
        let again = download(&root, &|_, _| {}).await.expect("second download");
        assert_eq!(again.digest, build.digest);
        if let Some(keep) = std::env::var_os("MUSIQUE_SOLOIST_KEEP") {
            let _ = std::fs::remove_dir_all(&keep);
            std::fs::rename(&root, &keep).unwrap();
            return;
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    fn tarball(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast()));
        for (name, body) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, name, *body).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn extraction_keeps_to_its_folder_and_needs_the_executable() {
        let dir = std::env::temp_dir().join(format!("musique-soloist-test-{:016x}", rand::random::<u64>()));
        let good = tarball(&[("soloist/soloist", b"player"), ("soloist/extra.bin", b"no"), ("soloist/CHANGELOG.md", b"notes")]);
        extract(&good, false, &dir.join("good"), "soloist").unwrap();
        assert_eq!(std::fs::read(dir.join("good/soloist")).unwrap(), b"player");
        assert!(dir.join("good/CHANGELOG.md").exists());
        assert!(!dir.join("good/extra.bin").exists());

        let missing = tarball(&[("soloist/CHANGELOG.md", b"notes")]);
        assert!(extract(&missing, false, &dir.join("missing"), "soloist").is_err());

        let twice = tarball(&[("a/soloist", b"one"), ("b/soloist", b"two")]);
        assert!(extract(&twice, false, &dir.join("twice"), "soloist").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
