//! The CDM host process: this executable started with `--widevine-host <module>`.
//!
//! It loads the module through the native shim (shim/host.cc), answers the app over
//! stdin/stdout with the frames in [`super::wire`], and exits when stdin closes. Every CDM call
//! happens on the main thread; a reader thread only moves frames off stdin so the main thread
//! can fire the module's timers while it waits for the next request.

use std::collections::VecDeque;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, BufWriter};
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use super::wire::{self, Reader};

/// Sessions kept open at once. Each track opens one; the oldest are closed past this, which
/// leaves the playing track and a preloaded next one with their keys.
const MAX_SESSIONS: usize = 6;

const ERR_CAP: usize = 512;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
extern "C" {
    fn wvh_take_stdout(ipc: *mut isize) -> c_int;
    fn wvh_open(path: *const c_char, err: *mut c_char, err_cap: u32) -> c_int;
    fn wvh_interface_version() -> c_int;
    fn wvh_module_version() -> *const c_char;
    fn wvh_next_timer_ms() -> i64;
    fn wvh_fire_timers();
    fn wvh_generate(init: *const u8, size: u32, err: *mut c_char, err_cap: u32) -> c_int;
    fn wvh_message(size: *mut u32) -> *const u8;
    fn wvh_session(size: *mut u32) -> *const c_char;
    fn wvh_update(
        session: *const c_char,
        session_size: u32,
        license: *const u8,
        size: u32,
        usable: *mut u32,
        err: *mut c_char,
        err_cap: u32,
    ) -> c_int;
    fn wvh_close(session: *const c_char, session_size: u32) -> c_int;
    fn wvh_decrypt(
        data: *const u8,
        size: u32,
        key_id: *const u8,
        key_id_size: u32,
        iv: *const u8,
        iv_size: u32,
        subsamples: *const u32,
        count: u32,
        out: *mut u8,
    ) -> c_int;
}

/// Runs the host until the app closes its end of the pipe. Answers the process exit code.
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub fn serve_host(module: &Path) -> i32 {
    let mut ipc = match take_stdout() {
        Ok(file) => BufWriter::with_capacity(64 * 1024, file),
        Err(error) => {
            eprintln!("widevine host: cannot take stdout: {error}");
            return 2;
        }
    };

    let mut err = [0 as c_char; ERR_CAP];
    let opened = match CString::new(module.to_string_lossy().into_owned()) {
        Ok(path) => unsafe { wvh_open(path.as_ptr(), err.as_mut_ptr(), ERR_CAP as u32) },
        Err(_) => {
            let _ = wire::write_frame(&mut ipc, wire::FAILED, &[b"the module path has a nul"]);
            return 1;
        }
    };
    if opened != 0 {
        let why = c_text(&err);
        let _ = wire::write_frame(&mut ipc, wire::FAILED, &[why.as_bytes()]);
        return 1;
    }
    let hello = format!(
        "{}\n{}\n{}",
        wire::PROTOCOL,
        unsafe { c_str(wvh_module_version()) },
        unsafe { wvh_interface_version() }
    );
    if wire::write_frame(&mut ipc, wire::OK, &[hello.as_bytes()]).is_err() {
        return 0;
    }

    // Frames come off stdin on their own thread; buffers flow back for reuse.
    let (frames_tx, frames) = mpsc::sync_channel::<(u8, Vec<u8>)>(4);
    let (spare_tx, spare) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut input = io::BufReader::with_capacity(64 * 1024, stdin.lock());
        loop {
            let mut buf = spare.try_recv().unwrap_or_default();
            match wire::read_frame(&mut input, &mut buf) {
                Ok(op) => {
                    if frames_tx.send((op, buf)).is_err() {
                        return;
                    }
                }
                Err(_) => return, // the app is gone, or closed the pipe on purpose
            }
        }
    });

    let mut server = Server::default();
    loop {
        let next = unsafe { wvh_next_timer_ms() };
        let frame = if next < 0 {
            frames.recv().map_err(|_| RecvTimeoutError::Disconnected)
        } else {
            frames.recv_timeout(Duration::from_millis(next as u64))
        };
        match frame {
            Ok((op, payload)) => {
                unsafe { wvh_fire_timers() };
                let (status, ok) = server.handle(op, &payload);
                let _ = spare_tx.send(payload);
                let body: &[u8] = if ok { &server.out } else { server.error.as_bytes() };
                if wire::write_frame(&mut ipc, status, &[body]).is_err() {
                    return 0;
                }
            }
            Err(RecvTimeoutError::Timeout) => unsafe { wvh_fire_timers() },
            Err(RecvTimeoutError::Disconnected) => return 0,
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn serve_host(_module: &Path) -> i32 {
    eprintln!("widevine host: not supported on this platform");
    1
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
#[derive(Default)]
struct Server {
    sessions: VecDeque<Vec<u8>>,
    out: Vec<u8>,
    error: String,
    subsamples: Vec<u32>,
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
impl Server {
    /// Handles one request. On success the answer is in `out`; otherwise `error` says why.
    fn handle(&mut self, op: u8, payload: &[u8]) -> (u8, bool) {
        self.out.clear();
        self.error.clear();
        let result = match op {
            wire::GENERATE => self.generate(payload),
            wire::UPDATE => self.update(payload),
            wire::DECRYPT => self.decrypt(payload),
            wire::DECRYPT_MANY => self.decrypt_many(payload),
            wire::CLOSE => {
                self.close(payload);
                Ok(())
            }
            other => Err((wire::FAILED, format!("unknown request {other}"))),
        };
        match result {
            Ok(()) => (wire::OK, true),
            Err((status, why)) => {
                self.error = why;
                (status, false)
            }
        }
    }

    fn generate(&mut self, init: &[u8]) -> Result<(), (u8, String)> {
        let mut err = [0 as c_char; ERR_CAP];
        let code =
            unsafe { wvh_generate(init.as_ptr(), init.len() as u32, err.as_mut_ptr(), ERR_CAP as u32) };
        if code != 0 {
            return Err((wire::FAILED, c_text(&err)));
        }
        let (mut message_len, mut session_len) = (0u32, 0u32);
        let message = unsafe { wvh_message(&mut message_len) };
        let session = unsafe { wvh_session(&mut session_len) };
        let message = unsafe { slice(message, message_len) };
        let session = unsafe { slice(session as *const u8, session_len) }.to_vec();
        self.out.extend_from_slice(&(session.len() as u16).to_le_bytes());
        self.out.extend_from_slice(&session);
        self.out.extend_from_slice(message);
        self.sessions.push_back(session);
        while self.sessions.len() > MAX_SESSIONS {
            if let Some(old) = self.sessions.pop_front() {
                unsafe { wvh_close(old.as_ptr() as *const c_char, old.len() as u32) };
            }
        }
        Ok(())
    }

    fn update(&mut self, payload: &[u8]) -> Result<(), (u8, String)> {
        let mut r = Reader::new(payload);
        let bad = |_| (wire::FAILED, "malformed license frame".to_string());
        let len = r.u16().map_err(bad)? as usize;
        let named = r.take(len).map_err(bad)?;
        let license = r.rest();
        // No session named: the newest one, which is the one a lone challenge just opened.
        let session = if named.is_empty() {
            self.sessions.back().cloned().unwrap_or_default()
        } else {
            named.to_vec()
        };
        if session.is_empty() {
            return Err((wire::FAILED, "no session is waiting for a license".into()));
        }
        let mut usable = 0u32;
        let mut err = [0 as c_char; ERR_CAP];
        let code = unsafe {
            wvh_update(
                session.as_ptr() as *const c_char,
                session.len() as u32,
                license.as_ptr(),
                license.len() as u32,
                &mut usable,
                err.as_mut_ptr(),
                ERR_CAP as u32,
            )
        };
        if code != 0 {
            return Err((wire::FAILED, c_text(&err)));
        }
        self.out.extend_from_slice(&usable.to_le_bytes());
        Ok(())
    }

    fn close(&mut self, session: &[u8]) {
        if let Some(at) = self.sessions.iter().position(|s| s == session) {
            self.sessions.remove(at);
            unsafe { wvh_close(session.as_ptr() as *const c_char, session.len() as u32) };
        }
    }

    fn decrypt(&mut self, payload: &[u8]) -> Result<(), (u8, String)> {
        let mut r = Reader::new(payload);
        let bad = |_| (wire::FAILED, "malformed decrypt frame".to_string());
        let kid_len = r.u8().map_err(bad)? as usize;
        let kid = r.take(kid_len).map_err(bad)?;
        let iv = r.take(16).map_err(bad)?;
        let count = r.u32().map_err(bad)? as usize;
        let pairs = r.take(count * 8).map_err(bad)?;
        let sample = r.rest();
        let start = self.out.len();
        self.out.resize(start + sample.len(), 0);
        decrypt_one(&mut self.subsamples, kid, iv, pairs, count, sample, &mut self.out[start..])
    }

    fn decrypt_many(&mut self, payload: &[u8]) -> Result<(), (u8, String)> {
        let mut r = Reader::new(payload);
        let bad = |_| (wire::FAILED, "malformed decrypt frame".to_string());
        let kid_len = r.u8().map_err(bad)? as usize;
        let kid = r.take(kid_len).map_err(bad)?;
        let n = r.u32().map_err(bad)?;
        for index in 0..n {
            let iv = r.take(16).map_err(bad)?;
            let count = r.u32().map_err(bad)? as usize;
            let pairs = r.take(count * 8).map_err(bad)?;
            let len = r.u32().map_err(bad)? as usize;
            let sample = r.take(len).map_err(bad)?;
            let start = self.out.len();
            self.out.resize(start + len, 0);
            decrypt_one(&mut self.subsamples, kid, iv, pairs, count, sample, &mut self.out[start..])
                .map_err(|(status, why)| (status, format!("sample {index}: {why}")))?;
        }
        Ok(())
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn decrypt_one(
    scratch: &mut Vec<u32>,
    kid: &[u8],
    iv: &[u8],
    pairs: &[u8],
    count: usize,
    sample: &[u8],
    out: &mut [u8],
) -> Result<(), (u8, String)> {
    scratch.clear();
    scratch.extend(
        pairs
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
    );
    let code = unsafe {
        wvh_decrypt(
            sample.as_ptr(),
            sample.len() as u32,
            kid.as_ptr(),
            kid.len() as u32,
            iv.as_ptr(),
            iv.len() as u32,
            scratch.as_ptr(),
            count as u32,
            out.as_mut_ptr(),
        )
    };
    match code {
        0 => Ok(()),
        2 => Err((wire::NO_KEY, "no usable key for this sample".into())),
        4 => Err((wire::FAILED, "the module could not decrypt the sample".into())),
        101 => Err((wire::FAILED, "the module answered no buffer".into())),
        102 => Err((wire::FAILED, "the module answered a buffer of the wrong size".into())),
        other => Err((wire::FAILED, format!("decrypt failed (status {other})"))),
    }
}

/// The IPC end of stdout, with stdout itself redirected to stderr.
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn take_stdout() -> io::Result<File> {
    let mut raw: isize = 0;
    let code = unsafe { wvh_take_stdout(&mut raw) };
    if code != 0 {
        return Err(io::Error::other(format!("redirect failed ({code})")));
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::FromRawHandle;
        Ok(unsafe { File::from_raw_handle(raw as *mut std::ffi::c_void) })
    }
    #[cfg(unix)]
    {
        use std::os::unix::io::FromRawFd;
        Ok(unsafe { File::from_raw_fd(raw as i32) })
    }
}

unsafe fn slice<'a>(ptr: *const u8, len: u32) -> &'a [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(ptr, len as usize)
    }
}

unsafe fn c_str(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

fn c_text(buf: &[c_char]) -> String {
    let bytes: Vec<u8> = buf.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if text.is_empty() {
        "the module failed without saying why".into()
    } else {
        text
    }
}
