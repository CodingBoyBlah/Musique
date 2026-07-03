use crate::errors::AppError;
use base64::Engine;
use std::path::PathBuf;

// desktop wallpaper as a `data:` URL (base64) so the frontend can sample its
// colors for the "Wallpaper colors" theme. Returns None when the OS wallpaper
// can't be located (unsupported DE, missing file, etc) — the frontend then just
// keeps the current accent.
#[tauri::command]
pub async fn get_wallpaper_data_url() -> Result<Option<String>, AppError> {
    let Some(path) = wallpaper_path() else { return Ok(None); };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[theme] wallpaper read failed ({}): {e}", path.display());
            return Ok(None);
        }
    };
    let mime = match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("heic") | Some("heif") => "image/heic",
        // Windows' TranscodedWallpaper has no extension and is always JPEG
        _ => "image/jpeg",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(Some(format!("data:{mime};base64,{b64}")))
}

// macOS system accent color as a hex string. None on every other OS. This is the
// ONLY color the engine does not brightness-normalize — the user asked for their
// actual system color.
#[tauri::command]
pub fn get_system_accent() -> Result<Option<String>, AppError> {
    #[cfg(target_os = "macos")]
    {
        Ok(Some(macos_accent()))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn wallpaper_path() -> Option<PathBuf> {
    // The current wallpaper is always cached here as a JPEG regardless of the
    // source format — no registry read / winapi needed.
    let appdata = std::env::var_os("APPDATA")?;
    let p = PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Themes")
        .join("TranscodedWallpaper");
    p.exists().then_some(p)
}

#[cfg(target_os = "macos")]
fn wallpaper_path() -> Option<PathBuf> {
    // System Events returns the POSIX path of the current desktop picture.
    // NOTE: on modern macOS (dynamic/aggregate wallpapers) this can return empty
    // or the first display only, and may prompt once for Automation permission.
    // The macOS-native path for this app is "System colors" anyway.
    let out = std::process::Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get picture of current desktop",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        return None;
    }
    let p = PathBuf::from(s);
    p.exists().then_some(p)
}

#[cfg(target_os = "linux")]
fn wallpaper_path() -> Option<PathBuf> {
    // GNOME (and GNOME-based DEs). Best-effort: other desktops return None and
    // the wallpaper option simply does nothing there.
    let out = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    let s = raw.trim().trim_matches('\'').trim_matches('"');
    let s = s.strip_prefix("file://").unwrap_or(s);
    let p = PathBuf::from(percent_decode(s));
    p.exists().then_some(p)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn wallpaper_path() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "linux")]
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(target_os = "macos")]
fn macos_accent() -> String {
    // `AppleAccentColor` is an integer preference. When the key is ABSENT the
    // user is on "Multicolor" (the default) which reads as blue. -1 = graphite.
    let out = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleAccentColor"])
        .output();
    let code = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return "#007AFF".to_string(), // key absent -> multicolor/default blue
    };
    match code.as_str() {
        "0" => "#FF5257",
        "1" => "#F7821B",
        "2" => "#FFC600",
        "3" => "#62BA46",
        "4" => "#007AFF",
        "5" => "#A550A7",
        "6" => "#F74F9E",
        "-1" => "#8C8C8C",
        _ => "#007AFF",
    }
    .to_string()
}
