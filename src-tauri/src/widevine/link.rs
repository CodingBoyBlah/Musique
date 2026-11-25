//! One running CDM host process and the pipe to it.

use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use super::wire;
use crate::errors::AppError;

/// How long one request (or the host's start) may take before the host is presumed hung.
const PATIENCE: Duration = Duration::from_secs(15);
/// How long a host that was asked to exit gets before it is killed.
const EXIT_WAIT: Duration = Duration::from_secs(2);

struct Io {
    input: Option<BufWriter<ChildStdin>>,
    output: BufReader<ChildStdout>,
    answer: Vec<u8>,
}

pub(super) struct Link {
    io: Mutex<Io>,
    child: Mutex<Child>,
    dead: AtomicBool,
    /// Milliseconds since `born` when the current request started, or 0 when idle.
    busy_since: AtomicU64,
    born: Instant,
    pub module_version: String,
    pub interface: u32,
    /// Held across a challenge and its license by `Cdm::exchange`.
    pub licensing: Mutex<()>,
}

impl Link {
    pub fn start(module: &Path) -> Result<Arc<Link>, AppError> {
        let exe = host_executable()
            .map_err(|e| AppError::Playback(format!("cannot find this executable: {e}")))?;
        let mut command = Command::new(exe);
        command
            .arg(super::HOST_ARG)
            .arg(module)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if cfg!(debug_assertions) { Stdio::inherit() } else { Stdio::null() });
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = command
            .spawn()
            .map_err(|e| AppError::Playback(format!("cannot start the widevine host: {e}")))?;
        let input = child.stdin.take().map(|s| BufWriter::with_capacity(64 * 1024, s));
        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            return Err(AppError::Playback("the widevine host has no stdout".into()));
        };
        let output = BufReader::with_capacity(64 * 1024, stdout);

        // The hello frame, read on a helper thread so a module that hangs while loading
        // cannot hang the caller.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut output = output;
            let mut payload = Vec::new();
            let tag = wire::read_frame(&mut output, &mut payload);
            let _ = tx.send((tag, payload, output));
        });
        let (tag, payload, output) = match rx.recv_timeout(PATIENCE) {
            Ok(v) => v,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(AppError::Playback("the widevine host did not start in time".into()));
            }
        };
        let fail = |mut child: Child, why: String| {
            let _ = child.kill();
            let _ = child.wait();
            Err(AppError::Playback(why))
        };
        let tag = match tag {
            Ok(tag) => tag,
            Err(e) => return fail(child, format!("the widevine host stopped while starting: {e}")),
        };
        let text = String::from_utf8_lossy(&payload).into_owned();
        if tag != wire::OK {
            return fail(child, format!("the widevine module did not open: {text}"));
        }
        let mut lines = text.split('\n');
        if lines.next() != Some(wire::PROTOCOL) {
            return fail(
                child,
                "the widevine host speaks another protocol (was the app updated while running?)"
                    .into(),
            );
        }
        let module_version = lines.next().unwrap_or("").to_string();
        let interface = lines.next().and_then(|v| v.trim().parse().ok()).unwrap_or(0);

        let link = Arc::new(Link {
            io: Mutex::new(Io {
                input,
                output,
                answer: Vec::new(),
            }),
            child: Mutex::new(child),
            dead: AtomicBool::new(false),
            busy_since: AtomicU64::new(0),
            born: Instant::now(),
            module_version,
            interface,
            licensing: Mutex::new(()),
        });
        watch(Arc::downgrade(&link));
        log::info!(
            "widevine: host up, module {} (interface {})",
            link.module_version,
            link.interface
        );
        Ok(link)
    }

    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::Acquire)
    }

    /// One request; the answer is copied out.
    pub fn call(&self, op: u8, parts: &[&[u8]]) -> Result<Vec<u8>, AppError> {
        self.call_into(op, parts, |answer| Ok(answer.to_vec()))
    }

    /// One request; `take` reads the answer straight out of the link's buffer.
    pub fn call_into<T>(
        &self,
        op: u8,
        parts: &[&[u8]],
        take: impl FnOnce(&[u8]) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let gone = || AppError::Playback("the widevine host has stopped".into());
        if self.is_dead() {
            return Err(gone());
        }
        let mut guard = self.io.lock().unwrap_or_else(|e| e.into_inner());
        let io = &mut *guard;
        let Some(input) = io.input.as_mut() else {
            return Err(gone());
        };
        let started = self.born.elapsed().as_millis() as u64;
        self.busy_since.store(started.max(1), Ordering::Release);
        let status = wire::write_frame(input, op, parts)
            .and_then(|_| wire::read_frame(&mut io.output, &mut io.answer));
        self.busy_since.store(0, Ordering::Release);
        let status = match status {
            Ok(status) => status,
            Err(e) => {
                self.dead.store(true, Ordering::Release);
                return Err(AppError::Playback(format!("lost the widevine host: {e}")));
            }
        };
        match status {
            wire::OK => take(&io.answer),
            wire::NO_KEY => Err(AppError::Playback(
                "no widevine key for this sample (no license yet, or it expired)".into(),
            )),
            _ => Err(AppError::Playback(format!(
                "widevine: {}",
                String::from_utf8_lossy(&io.answer)
            ))),
        }
    }

    /// Closes the host's stdin so it exits, and kills it if it does not.
    pub fn stop(&self) {
        self.dead.store(true, Ordering::Release);
        // A request in flight holds the lock; killing below ends it.
        if let Ok(mut io) = self.io.try_lock() {
            io.input = None;
        }
        let mut child = self.child.lock().unwrap_or_else(|e| e.into_inner());
        let deadline = Instant::now() + EXIT_WAIT;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20))
                }
                _ => break,
            }
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Kills the host when a request outlives PATIENCE; ends once the link is gone or dead.
fn watch(link: Weak<Link>) {
    let _ = std::thread::Builder::new()
        .name("widevine-watchdog".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            let Some(link) = link.upgrade() else { return };
            if link.is_dead() {
                return;
            }
            let since = link.busy_since.load(Ordering::Acquire);
            if since == 0 {
                continue;
            }
            let now = link.born.elapsed().as_millis() as u64;
            if now.saturating_sub(since) > PATIENCE.as_millis() as u64 {
                log::warn!("widevine: host stuck on a request, killing it");
                link.dead.store(true, Ordering::Release);
                let _ = link.child.lock().unwrap_or_else(|e| e.into_inner()).kill();
                return;
            }
        });
}

/// The executable to start as the host: this one.
fn host_executable() -> std::io::Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let proc_exe = PathBuf::from("/proc/self/exe");
        if proc_exe.exists() {
            return Ok(proc_exe);
        }
    }
    std::env::current_exe()
}
