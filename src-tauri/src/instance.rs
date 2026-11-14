//! One copy of the app at a time.
//!
//! A second copy would open the same database and sign a second librespot
//! session in under the same Connect device id. Spotify keeps one session per
//! device, so each would knock the other off, and each one's recovery
//! (`recover_dropped_sessions`) would reconnect and knock the other off again.
//!
//! The first copy holds a lock on a file in the app's data folder, which the
//! system lets go when the process ends, even after a crash. It listens on a
//! loopback port written into that file. A second launch that finds the file
//! locked asks the first, over that port, to show its window, and exits.
//! Anything that can connect can only ask that.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

const IDENTIFIER: &str = "dev.boyblah.musique";
const SHOW: &str = "show";

/// Where Tauri keeps the app's data (`app_data_dir`), worked out before Tauri
/// has started.
fn data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    base.map(|base| base.join(IDENTIFIER))
}

/// The running copy's window to show when another launch asks.
static ON_SHOW: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// The lock, held for the life of the process.
pub struct Instance {
    _lock: File,
}

/// What `claim` found.
pub enum Claim {
    /// This is the only copy; keep the guard.
    First(Instance),
    /// Another copy is running and was asked to show itself.
    Second,
}

/// Takes the lock, or hands over to the copy that has it. A folder that cannot
/// be used lets this copy run unguarded rather than not at all.
pub fn claim() -> Option<Claim> {
    let dir = data_dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    claim_in(&dir.join("instance.lock"))
}

fn claim_in(path: &std::path::Path) -> Option<Claim> {
    let mut file = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path).ok()?;
    match file.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => {
            // the running copy wrote its port when it took the lock. on
            // windows a locked file cannot be read through another handle, so
            // the port also sits beside it
            let mut port = String::new();
            let _ = std::fs::File::open(path.with_extension("port")).and_then(|mut f| f.read_to_string(&mut port));
            if let Ok(port) = port.trim().parse::<u16>() {
                if let Ok(mut stream) = TcpStream::connect_timeout(&(Ipv4Addr::LOCALHOST, port).into(), Duration::from_secs(2)) {
                    let _ = writeln!(stream, "{SHOW}");
                }
            }
            return Some(Claim::Second);
        }
        Err(std::fs::TryLockError::Error(e)) => {
            eprintln!("[instance] could not lock {}: {e}; running unguarded", path.display());
            return None;
        }
    }
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).ok()?;
    let port = listener.local_addr().ok()?.port();
    let _ = file.set_len(0).and_then(|()| file.seek(SeekFrom::Start(0)).map(drop)).and_then(|()| write!(file, "{port}"));
    let _ = std::fs::write(path.with_extension("port"), port.to_string());
    std::thread::Builder::new()
        .name("instance".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut line = String::new();
                if BufReader::new(stream).take(64).read_line(&mut line).is_ok() && line.trim() == SHOW {
                    if let Some(show) = ON_SHOW.get() {
                        show();
                    }
                }
            }
        })
        .ok()?;
    Some(Claim::First(Instance { _lock: file }))
}

/// What to do when another launch asks this copy to show itself.
pub fn on_show(show: impl Fn() + Send + Sync + 'static) {
    let _ = ON_SHOW.set(Box::new(show));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_claim_hands_over_to_the_first() {
        let dir = std::env::temp_dir().join(format!("musique-instance-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("instance.lock");
        let first = claim_in(&path);
        assert!(matches!(first, Some(Claim::First(_))));
        let (shown, seen) = std::sync::mpsc::channel();
        let shown = std::sync::Mutex::new(shown);
        on_show(move || {
            let _ = shown.lock().unwrap().send(());
        });
        assert!(matches!(claim_in(&path), Some(Claim::Second)));
        seen.recv_timeout(Duration::from_secs(5)).expect("the first copy was asked to show itself");
        // once the first lets go, the next launch is the first again
        drop(first);
        assert!(matches!(claim_in(&path), Some(Claim::First(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
