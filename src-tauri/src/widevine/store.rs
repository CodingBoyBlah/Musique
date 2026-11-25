//! Where the module lives on disk: `<dir>/<version>/<library>` with Google's `manifest.json` and
//! `LICENSE` beside it. `<dir>` is the app's local data folder plus `widevine`, the same folder
//! Tauri's `app_local_data_dir()` names, so it can be found from any thread without an
//! AppHandle (and from the host process, which never starts Tauri).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The bundle identifier in tauri.conf.json; the data folder is named after it.
const IDENTIFIER: &str = "dev.boyblah.musique";

/// Points at a module to use instead of an installed one (development and testing).
pub const OVERRIDE_ENV: &str = "MUSIQUE_WIDEVINE_CDM";

/// Google's name for this platform in the component updater and inside the archive.
pub const OS: &str = if cfg!(target_os = "windows") {
    "win"
} else if cfg!(target_os = "macos") {
    "mac"
} else {
    "linux"
};

/// Google's name for this processor; empty where no module is built.
pub const ARCH: &str = if cfg!(target_arch = "x86_64") {
    "x64"
} else if cfg!(target_arch = "aarch64") {
    "arm64"
} else {
    ""
};

/// The module's file name on this platform.
pub const LIBRARY: &str = if cfg!(target_os = "windows") {
    "widevinecdm.dll"
} else if cfg!(target_os = "macos") {
    "libwidevinecdm.dylib"
} else {
    "libwidevinecdm.so"
};

/// Google's terms, kept beside the module.
pub const LICENSE: &str = "LICENSE";
pub const MANIFEST: &str = "manifest.json";

static DIR: OnceLock<PathBuf> = OnceLock::new();

/// Sets the install folder. Optional: without it the same folder is derived from the
/// platform's data directory. Only the first call counts.
pub fn init(dir: PathBuf) {
    let _ = DIR.set(dir);
}

/// The install folder.
pub fn dir() -> PathBuf {
    DIR.get().cloned().unwrap_or_else(default_dir)
}

fn default_dir() -> PathBuf {
    let env = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if cfg!(target_os = "windows") {
        env("LOCALAPPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library").join("Application Support"))
    } else {
        env("XDG_DATA_HOME").or_else(|| env("HOME").map(|h| h.join(".local").join("share")))
    };
    base.unwrap_or_else(std::env::temp_dir)
        .join(IDENTIFIER)
        .join("widevine")
}

/// Whether Google builds a module for this platform (and the host can run it).
pub fn supported() -> bool {
    !ARCH.is_empty()
        && cfg!(any(target_os = "windows", target_os = "macos", target_os = "linux"))
}

/// An installed module.
#[derive(Clone, Debug)]
pub struct Installed {
    pub version: String,
    pub library: PathBuf,
}

/// The newest complete install, if any.
pub fn installed() -> Option<Installed> {
    let entries = std::fs::read_dir(dir()).ok()?;
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let key = version_key(&name)?;
            let library = entry.path().join(LIBRARY);
            library.is_file().then(|| (key, Installed { version: name, library }))
        })
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, found)| found)
}

/// The module to load: the override, else the newest install.
pub fn module_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(OVERRIDE_ENV).map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    installed().map(|i| i.library)
}

/// `"4.10.2830.0"` -> `[4, 10, 2830, 0]`; anything that is not a dotted version is `None`.
pub fn version_key(name: &str) -> Option<Vec<u64>> {
    let parts: Option<Vec<u64>> = name.split('.').map(|p| p.parse().ok()).collect();
    parts.filter(|p| p.len() >= 2)
}

/// Google's terms as the installed module carries them.
pub fn installed_license() -> Option<String> {
    let installed = installed()?;
    let folder = installed.library.parent()?;
    std::fs::read_to_string(folder.join(LICENSE)).ok()
}

/// Removes every version folder except `keep`, ignoring folders still in use.
pub fn prune(keep: &Path) {
    let Ok(entries) = std::fs::read_dir(dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == keep {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let ours = version_key(&name).is_some() || name.starts_with(".staging-");
        if ours && path.is_dir() {
            if let Err(e) = std::fs::remove_dir_all(&path) {
                log::debug!("widevine: cannot remove old {name} yet: {e}");
            }
        }
    }
}
