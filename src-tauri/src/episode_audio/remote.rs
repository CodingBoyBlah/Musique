//! a remote audio file the decoder can read (and seek in) while it downloads.
//!
//! music on the youtube backend is small enough to download whole before it
//! plays (see `youtube::stream`). episodes aren't: an hour is 60-300 MB, which
//! is seconds to minutes of waiting and far too much to hold in memory. so the
//! file is fetched in ranged requests that follow wherever the decoder is
//! reading, and only a window of it is kept: a few minutes behind the playhead
//! and a stretch ahead. playback starts on the first chunk, and a seek halfway
//! into a three hour episode fetches the bytes around the new spot next instead
//! of everything before it.
//!
//! (it's memory rather than a temp file on purpose: writing far into a fresh
//! file makes windows zero-fill everything before that point first, which
//! turned a seek to the end of a 300 MB episode into a 16 second stall.)
//!
//! the decoder side is plain blocking `Read + Seek`: a read of bytes that
//! haven't arrived yet waits for them (with a timeout, so a dead connection
//! surfaces as an error rather than a hang).

use std::{
    collections::HashMap,
    io::{self, Read, Seek, SeekFrom},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc, Condvar, Mutex,
    },
    time::Duration,
};

use crate::{errors::AppError, http};

/// first request: enough for the decoder to probe the format and start
const FIRST: u64 = 256 * 1024;
/// a request right after a jump (seek) - small, so the new spot plays soon
const MIN_CHUNK: u64 = 256 * 1024;
/// requests grow while reading straight through, up to this. kept well under
/// what the shared client's 20s timeout allows on a slow connection
const MAX_CHUNK: u64 = 2 * 1024 * 1024;
/// how long a read waits for bytes before giving up
const READ_TIMEOUT: Duration = Duration::from_secs(30);
/// storage granularity
const BLOCK: u64 = 256 * 1024;
/// how much is kept at most (48 MB); past this the blocks furthest from the
/// playhead go
const MAX_BLOCKS: usize = 192;
/// how far ahead of the playhead to download before idling (~25 minutes of a
/// 128 kbps episode)
const AHEAD: u64 = 24 * 1024 * 1024;

/// byte ranges downloaded so far: sorted, disjoint, half-open
#[derive(Debug, Default)]
struct Ranges(Vec<(u64, u64)>);

impl Ranges {
    fn insert(&mut self, start: u64, end: u64) {
        if start >= end {
            return;
        }
        let mut merged = (start, end);
        let mut out = Vec::with_capacity(self.0.len() + 1);
        for &(a, b) in &self.0 {
            if b < merged.0 || a > merged.1 {
                out.push((a, b));
            } else {
                merged = (merged.0.min(a), merged.1.max(b));
            }
        }
        out.push(merged);
        out.sort_unstable();
        self.0 = out;
    }

    /// bytes available contiguously from `pos` (0 when `pos` isn't downloaded)
    fn available_from(&self, pos: u64) -> u64 {
        self.0.iter().find(|&&(a, b)| a <= pos && pos < b).map(|&(_, b)| b - pos).unwrap_or(0)
    }

    /// the first missing stretch at or after `from`, else the first one before
    /// it; None once the whole file is here
    fn next_gap(&self, from: u64, len: u64) -> Option<(u64, u64)> {
        let gap_after = |from: u64| {
            let mut pos = from;
            for &(a, b) in &self.0 {
                if b <= pos {
                    continue;
                }
                if a > pos {
                    return Some((pos, a));
                }
                pos = b;
            }
            (pos < len).then_some((pos, len))
        };
        gap_after(from.min(len)).or_else(|| gap_after(0))
    }

    fn remove(&mut self, start: u64, end: u64) {
        let mut out = Vec::with_capacity(self.0.len() + 1);
        for &(a, b) in &self.0 {
            if b <= start || a >= end {
                out.push((a, b));
                continue;
            }
            if a < start {
                out.push((a, start));
            }
            if b > end {
                out.push((end, b));
            }
        }
        self.0 = out;
    }
}

struct State {
    /// BLOCK-sized pieces of the file, by index
    blocks: HashMap<u64, Box<[u8]>>,
    have:   Ranges,
    failed: Option<String>,
}

struct Shared {
    state:  Mutex<State>,
    ready:  Condvar,
    len:    u64,
    /// where the decoder is reading; the downloader fetches from here onwards
    head:   AtomicU64,
    /// readers (decoders) still alive; the last one going stops the download
    readers: AtomicUsize,
    /// every reader was dropped - stop downloading
    closed: AtomicBool,
}

impl Shared {
    fn write_at(&self, offset: u64, bytes: &[u8]) {
        let mut st = self.state.lock().unwrap();
        let mut at = offset;
        let mut rest = bytes;
        while !rest.is_empty() {
            let index = at / BLOCK;
            let within = (at % BLOCK) as usize;
            let n = rest.len().min(BLOCK as usize - within);
            let block = st.blocks.entry(index).or_insert_with(|| vec![0u8; BLOCK as usize].into_boxed_slice());
            block[within..within + n].copy_from_slice(&rest[..n]);
            at += n as u64;
            rest = &rest[n..];
        }
        st.have.insert(offset, at);
        self.evict(&mut st, offset / BLOCK);
        drop(st);
        self.ready.notify_all();
    }

    /// over budget: drop the blocks furthest from the playhead. the first
    /// block stays (an mp4's header lives there and every seek re-reads it),
    /// and so does the one just written
    fn evict(&self, st: &mut State, keep: u64) {
        if st.blocks.len() <= MAX_BLOCKS {
            return;
        }
        let head = self.head.load(Ordering::Relaxed) / BLOCK;
        let mut indices: Vec<u64> = st.blocks.keys().copied().filter(|&i| i != 0 && i != keep).collect();
        indices.sort_by_key(|&i| std::cmp::Reverse(i.abs_diff(head)));
        for i in indices.into_iter().take(st.blocks.len() - MAX_BLOCKS) {
            st.blocks.remove(&i);
            st.have.remove(i * BLOCK, (i + 1) * BLOCK);
        }
    }

    fn fail(&self, why: String) {
        self.state.lock().unwrap().failed = Some(why);
        self.ready.notify_all();
    }

    fn covered(&self, pos: u64) -> bool {
        pos >= self.len || self.state.lock().unwrap().have.available_from(pos) > 0
    }
}

/// the decoder's end: blocking `Read + Seek` over the file as it arrives.
///
/// a reader can also be a *spliced* view: the file's first `head` bytes
/// followed directly by everything from `resume` on. that's how a fragmented
/// mp4 seeks without its demuxer walking every fragment in between - see
/// `source::EpisodeDecoder`.
pub struct RemoteFile {
    shared: Arc<Shared>,
    pos:    u64,
    splice: Option<(u64, u64)>,
}

impl Drop for RemoteFile {
    fn drop(&mut self) {
        if self.shared.readers.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.shared.closed.store(true, Ordering::Relaxed);
        }
    }
}

impl RemoteFile {
    fn new(shared: Arc<Shared>, splice: Option<(u64, u64)>) -> Self {
        shared.readers.fetch_add(1, Ordering::SeqCst);
        Self { shared, pos: 0, splice }
    }

    /// another reader over the same download, from the start
    pub fn reopen(&self) -> RemoteFile {
        RemoteFile::new(Arc::clone(&self.shared), None)
    }

    /// the first `head` bytes, then the file from `resume` on
    pub fn spliced(&self, head: u64, resume: u64) -> RemoteFile {
        let head = head.min(self.shared.len);
        RemoteFile::new(Arc::clone(&self.shared), Some((head, resume.clamp(head, self.shared.len))))
    }

    /// this view's length
    pub fn len(&self) -> u64 {
        match self.splice {
            Some((head, resume)) => head + (self.shared.len - resume),
            None => self.shared.len,
        }
    }

    /// where `pos` is in the real file, and how many bytes can be read from
    /// there before this view jumps
    fn physical(&self, pos: u64) -> (u64, u64) {
        match self.splice {
            Some((head, _)) if pos < head => (pos, head - pos),
            Some((head, resume)) => (resume + (pos - head), u64::MAX),
            None => (pos, u64::MAX),
        }
    }
}

impl Read for RemoteFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let s = &self.shared;
        if self.pos >= self.len() || buf.is_empty() {
            return Ok(0);
        }
        let (at, limit) = self.physical(self.pos);
        s.head.store(at, Ordering::Relaxed);
        let mut st = s.state.lock().unwrap();
        loop {
            let within = at % BLOCK;
            let avail = st.have.available_from(at).min(limit).min(BLOCK - within);
            if avail > 0 {
                let n = (avail as usize).min(buf.len());
                let block = st.blocks.get(&(at / BLOCK)).ok_or_else(|| io::Error::other("block missing"))?;
                buf[..n].copy_from_slice(&block[within as usize..within as usize + n]);
                self.pos += n as u64;
                return Ok(n);
            }
            if let Some(why) = &st.failed {
                return Err(io::Error::other(why.clone()));
            }
            let (next, timeout) = s.ready.wait_timeout(st, READ_TIMEOUT).unwrap();
            st = next;
            if timeout.timed_out() && st.have.available_from(at) == 0 {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "episode audio stalled"));
            }
        }
    }
}

impl Seek for RemoteFile {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let len = self.len() as i128;
        let next = match to {
            SeekFrom::Start(p) => p as i128,
            SeekFrom::End(d) => len + d as i128,
            SeekFrom::Current(d) => self.pos as i128 + d as i128,
        };
        if next < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "seek before start"));
        }
        self.pos = next as u64;
        Ok(self.pos)
    }
}

/// the player's end: lets a seek fetch the bytes it's about to need before the
/// audio thread asks for them
#[derive(Clone)]
pub struct RemoteHandle(Arc<Shared>);

impl RemoteHandle {
    pub fn len(&self) -> u64 {
        self.0.len
    }

    /// move the download to `byte` and wait (up to `wait`) until it's there
    pub async fn prefetch(&self, byte: u64, wait: Duration) {
        let byte = byte.min(self.0.len.saturating_sub(1));
        if self.0.covered(byte) {
            return;
        }
        self.0.head.store(byte, Ordering::Relaxed);
        let deadline = tokio::time::Instant::now() + wait;
        while !self.0.covered(byte) && tokio::time::Instant::now() < deadline {
            if self.0.state.lock().unwrap().failed.is_some() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
}

pub struct Opened {
    pub reader:       RemoteFile,
    pub handle:       RemoteHandle,
    pub len:          u64,
    pub content_type: Option<String>,
}

fn request(url: &str, user_agent: Option<&'static str>, start: u64, end: u64) -> reqwest::RequestBuilder {
    let mut req = http::client().get(url).header("Range", format!("bytes={start}-{}", end - 1));
    if let Some(ua) = user_agent {
        req = req.header("User-Agent", ua);
    }
    req
}

/// "bytes 0-262143/293600446" -> 293600446
fn total_from_content_range(v: &str) -> Option<u64> {
    v.rsplit('/').next()?.trim().parse().ok()
}

/// start fetching `url`. returns once the first chunk is on disk, so the
/// decoder can probe the format straight away
pub async fn open(url: &str, user_agent: Option<&'static str>) -> Result<Opened, AppError> {
    let mut res = request(url, user_agent, 0, FIRST)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("episode audio: {e}")))?;
    let status = res.status();
    if !status.is_success() {
        return Err(AppError::Network(format!("episode audio returned {status}")));
    }
    let ranged = status.as_u16() == 206;
    let len = if ranged {
        res.headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(total_from_content_range)
    } else {
        res.content_length()
    }
    .filter(|&n| n > 0)
    .ok_or_else(|| AppError::Playback("episode audio has no length".into()))?;
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    // tracking redirects (podtrac, chartable...) resolve to the real cdn url;
    // later ranges go straight there instead of bouncing through them again
    let final_url = res.url().to_string();

    let shared = Arc::new(Shared {
        state: Mutex::new(State { blocks: HashMap::new(), have: Ranges::default(), failed: None }),
        ready: Condvar::new(),
        len,
        head: AtomicU64::new(0),
        readers: AtomicUsize::new(0),
        closed: AtomicBool::new(false),
    });

    let bg = Arc::clone(&shared);
    if ranged {
        let mut pos = 0u64;
        let first = FIRST.min(len);
        while let Some(chunk) = res.chunk().await.map_err(|e| AppError::Network(format!("episode audio: {e}")))? {
            // a server that answers 206 but sends more than asked still only
            // gets the bytes we asked for
            let take = chunk.len().min((first - pos) as usize);
            shared.write_at(pos, &chunk[..take]);
            pos += take as u64;
            if pos >= first {
                break;
            }
        }
        tauri::async_runtime::spawn(fill(bg, final_url, user_agent, Some(pos)));
    } else {
        // the server ignores ranges: the whole file is coming in this one
        // response, so just keep writing it down in order
        tauri::async_runtime::spawn(async move {
            let mut pos = 0u64;
            loop {
                if bg.closed.load(Ordering::Relaxed) {
                    return;
                }
                match res.chunk().await {
                    Ok(Some(chunk)) => {
                        bg.write_at(pos, &chunk);
                        pos += chunk.len() as u64;
                    }
                    Ok(None) => return,
                    Err(e) => {
                        bg.fail(format!("episode audio: {e}"));
                        return;
                    }
                }
            }
        });
    }

    Ok(Opened {
        reader: RemoteFile::new(Arc::clone(&shared), None),
        handle: RemoteHandle(shared),
        len,
        content_type,
    })
}

/// keep fetching whatever's missing, starting where the decoder is reading
async fn fill(shared: Arc<Shared>, url: String, user_agent: Option<&'static str>, mut last_end: Option<u64>) {
    let mut size = MIN_CHUNK;
    let mut failures = 0u32;
    loop {
        if shared.closed.load(Ordering::Relaxed) {
            return;
        }
        let head = shared.head.load(Ordering::Relaxed);
        let gap = shared.state.lock().unwrap().have.next_gap(head, shared.len);
        let Some((start, gap_end)) = gap else { return };
        // far enough ahead (or everything after the head is here): wait for
        // the playhead to move rather than fill memory with the rest
        if start < head || start >= head + AHEAD {
            tokio::time::sleep(Duration::from_millis(250)).await;
            last_end = None;
            continue;
        }
        size = if last_end == Some(start) { (size * 2).min(MAX_CHUNK) } else { MIN_CHUNK };
        let end = (start + size).min(gap_end);
        match fetch_into(&shared, &url, user_agent, start, end).await {
            Ok(reached) => {
                failures = 0;
                last_end = Some(reached);
            }
            Err(e) => {
                failures += 1;
                eprintln!("[episode] range {start}-{end}: {e} (attempt {failures})");
                if failures >= 4 {
                    shared.fail(e);
                    return;
                }
                tokio::time::sleep(Duration::from_millis(400 * failures as u64)).await;
                last_end = None;
            }
        }
    }
}

/// fetch [start, end) onto disk; returns how far it got. bails early (Ok) when
/// the decoder has jumped somewhere this range won't reach, so the jump is
/// served next instead of after the rest of this chunk
async fn fetch_into(
    shared: &Shared,
    url: &str,
    user_agent: Option<&'static str>,
    start: u64,
    end: u64,
) -> Result<u64, String> {
    let mut res = request(url, user_agent, start, end).send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    if status.as_u16() != 206 {
        return Err(format!("expected a partial response, got {status}"));
    }
    let mut pos = start;
    while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
        let take = chunk.len().min((end - pos) as usize);
        shared.write_at(pos, &chunk[..take]);
        pos += take as u64;
        if pos >= end || shared.closed.load(Ordering::Relaxed) {
            break;
        }
        let head = shared.head.load(Ordering::Relaxed);
        if (head < start || head >= end) && !shared.covered(head) {
            break;
        }
    }
    if pos == start {
        return Err("empty response".into());
    }
    Ok(pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_merge() {
        let mut r = Ranges::default();
        r.insert(10, 20);
        r.insert(30, 40);
        r.insert(20, 30);
        assert_eq!(r.0, vec![(10, 40)]);
        r.insert(0, 5);
        r.insert(3, 12);
        assert_eq!(r.0, vec![(0, 40)]);
    }

    #[test]
    fn availability() {
        let mut r = Ranges::default();
        r.insert(0, 100);
        r.insert(200, 300);
        assert_eq!(r.available_from(0), 100);
        assert_eq!(r.available_from(99), 1);
        assert_eq!(r.available_from(100), 0);
        assert_eq!(r.available_from(250), 50);
    }

    #[test]
    fn gaps_follow_the_head_then_wrap() {
        let mut r = Ranges::default();
        r.insert(0, 100);
        r.insert(200, 300);
        assert_eq!(r.next_gap(0, 400), Some((100, 200)));
        assert_eq!(r.next_gap(150, 400), Some((150, 200)));
        assert_eq!(r.next_gap(250, 400), Some((300, 400)));
        r.insert(300, 400);
        // nothing left after the head: go back for what was skipped
        assert_eq!(r.next_gap(250, 400), Some((100, 200)));
        r.insert(100, 200);
        assert_eq!(r.next_gap(0, 400), None);
    }

    #[test]
    fn ranges_remove() {
        let mut r = Ranges::default();
        r.insert(0, 100);
        r.remove(20, 30);
        assert_eq!(r.0, vec![(0, 20), (30, 100)]);
        r.remove(0, 25);
        assert_eq!(r.0, vec![(30, 100)]);
        r.remove(90, 200);
        assert_eq!(r.0, vec![(30, 90)]);
    }

    #[test]
    fn content_range_total() {
        assert_eq!(total_from_content_range("bytes 0-262143/293600446"), Some(293_600_446));
        assert_eq!(total_from_content_range("bytes 0-1/*"), None);
    }
}
