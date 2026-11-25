//! Progressive in-memory stream and lazy CENC decryptor for Apple Music tracks.
//!
//! Encrypted tracks stay strictly in memory (never written to disk). The download pump appends
//! bytes while readers block only on bytes past the parsed fragment frontier (`walked`), and
//! decrypt only the samples overlapping each `Read` call on demand. Once the download completes,
//! a paced background pass decrypts any untouched samples and drops the CDM handle so the
//! Widevine host process can exit.

use std::io::{self, Read, Seek, SeekFrom};
use std::ops::Range;
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::{Duration, Instant};

use tokio::sync::watch;

use crate::errors::AppError;
use crate::widevine::{Cdm, Sample};

use super::fmp4::{
    box_extent, read_fragment, read_init, relabel_enca_to_mp4a, EncryptedTrack, FragmentStep,
    SampleEntry,
};

const PREROLL_BYTES: usize = 256 * 1024;
const BOX_HEADER_MAX: usize = 16;
const MAX_TRACK_BYTES: usize = 256 * 1024 * 1024;
const READ_PATIENCE: Duration = Duration::from_secs(20);
const DECRYPT_PACE: Duration = Duration::from_micros(500);
const TAIL_PROBE_BYTES: u64 = 8 * 1024;
const MAX_BATCH_SAMPLES: usize = 512;
const MAX_BATCH_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SampleSeal {
    #[default]
    Encrypted,
    Clear,
    Lost,
}

#[derive(Clone)]
struct CdmKeys {
    cdm:    Cdm,
    key_id: Vec<u8>,
}

#[derive(Default)]
enum LicenseState {
    #[default]
    Pending,
    Held(CdmKeys),
    Released,
}

#[derive(Clone, Copy, Debug)]
struct FragmentLoc {
    at:    usize,
    start: Duration,
}

/// Target location for splicing `[0..head]` onto `[graft..]` when seeking in a CENC fMP4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpliceEntry {
    pub head:              usize,
    pub graft:             usize,
    pub into:              Duration,
    pub in_first_fragment: bool,
}

#[derive(Default)]
struct CencState {
    plain:          bool,
    init_end:       Option<usize>,
    tracks:         Vec<EncryptedTrack>,
    timescale:      u32,
    walked:         usize,
    awaited:        usize,
    fragments:      Vec<FragmentLoc>,
    sample_batches: Vec<Range<usize>>,
    samples:        Vec<SampleEntry>,
    seals:          Vec<SampleSeal>,
    license:        LicenseState,
    scratch:        Vec<u8>,
    decrypted:      usize,
    decrypt_time:   Duration,
}

impl CencState {
    fn index_available(&mut self, spool: &mut Vec<u8>) {
        let arrived = spool.len();
        if arrived < self.awaited {
            return;
        }
        if self.init_end.is_none() && !self.parse_init(spool) {
            return;
        }
        while self.walked < arrived {
            let Some((kind, end)) = self.whole_box(spool, self.walked) else {
                break;
            };
            let end = if &kind == b"moof" {
                match self.whole_box(spool, end) {
                    Some((_, mdat_end)) => mdat_end,
                    None => break,
                }
            } else {
                end
            };

            match read_fragment(&spool[self.walked..end], self.walked, &self.tracks) {
                FragmentStep::Fragment {
                    samples,
                    next,
                    decode_ticks,
                } => {
                    if let Some(start) = self.ticks_to_duration(decode_ticks) {
                        self.fragments.push(FragmentLoc {
                            at: self.walked,
                            start,
                        });
                    }
                    let base = self.samples.len();
                    let end = base + samples.len();
                    if base < end {
                        self.sample_batches.push(base..end);
                    }
                    self.seals.resize(end, SampleSeal::Encrypted);
                    self.samples.extend(samples);
                    self.walked = next;
                }
                FragmentStep::Other { next } => {
                    self.walked = next;
                }
                FragmentStep::Partial => break,
            }
        }
    }

    fn parse_init(&mut self, spool: &mut Vec<u8>) -> bool {
        let mut at = 0usize;
        let end = loop {
            let Some((kind, next)) = self.whole_box(spool, at) else {
                return false;
            };
            if &kind == b"moov" {
                break next;
            }
            at = next;
        };
        let Some(init) = read_init(&spool[..end]) else {
            return false;
        };
        relabel_enca_to_mp4a(&mut spool[..end], &init);
        self.walked = init.end;
        self.init_end = Some(init.end);
        self.timescale = init.timescale;
        self.tracks = init.tracks;
        true
    }

    fn whole_box(&mut self, spool: &[u8], at: usize) -> Option<([u8; 4], usize)> {
        let arrived = spool.len();
        let avail = arrived.saturating_sub(at);
        if avail < 8 {
            self.awaited = at + 8;
            return None;
        }
        let head_len = avail.min(BOX_HEADER_MAX);
        let Some((kind, total)) = box_extent(&spool[at..at + head_len]) else {
            if head_len < BOX_HEADER_MAX {
                self.awaited = at + BOX_HEADER_MAX;
            }
            return None;
        };
        let end = at.checked_add(total)?;
        if end > arrived {
            self.awaited = end;
            return None;
        }
        Some((kind, end))
    }

    fn ticks_to_duration(&self, ticks: Option<u64>) -> Option<Duration> {
        let ticks = ticks?;
        (self.timescale > 0)
            .then(|| Duration::from_secs_f64(ticks as f64 / f64::from(self.timescale)))
    }

    fn batch_range_for(&self, first: usize, last_excl: usize) -> Range<usize> {
        let b0 = self.sample_batches.partition_point(|b| b.end <= first);
        let b1 = self.sample_batches.partition_point(|b| b.start < last_excl);
        if b0 < b1 {
            self.sample_batches[b0].start.min(first)..self.sample_batches[b1 - 1].end.max(last_excl)
        } else {
            first..last_excl
        }
    }

    fn decrypt_batch(&mut self, spool: &mut [u8], range: Range<usize>) -> io::Result<()> {
        let pending: Vec<usize> = range
            .filter(|&i| self.seals[i] == SampleSeal::Encrypted)
            .collect();
        if pending.is_empty() {
            return Ok(());
        }
        let keys = match &self.license {
            LicenseState::Held(keys) => keys,
            LicenseState::Pending => {
                return Err(io::Error::other("Widevine license not yet installed"));
            }
            LicenseState::Released => {
                return Err(io::Error::other(
                    "sample still encrypted after Widevine license release",
                ));
            }
        };
        for &idx in &pending {
            if self.samples[idx].end() > spool.len() {
                return Err(io::Error::other("sample offset runs past downloaded bytes"));
            }
        }

        let mut at = 0usize;
        while at < pending.len() {
            let next = next_batch_end(&self.samples, &pending, at);
            let chunk = &pending[at..next];

            self.scratch.clear();
            for &idx in chunk {
                let s = &self.samples[idx];
                self.scratch.extend_from_slice(&spool[s.start..s.end()]);
            }

            let mut batch = Vec::with_capacity(chunk.len());
            let mut rest = self.scratch.as_mut_slice();
            for &idx in chunk {
                let s = &self.samples[idx];
                let (data, tail) = rest.split_at_mut(s.len);
                rest = tail;
                batch.push(Sample {
                    data,
                    iv: &s.iv,
                    subsamples: &s.subs,
                });
            }

            let started = Instant::now();
            let res = keys.cdm.decrypt_many(&keys.key_id, &mut batch);
            drop(batch);
            if let Err(err) = res {
                for &idx in chunk {
                    self.seals[idx] = SampleSeal::Lost;
                }
                return Err(io::Error::other(format!("Widevine decrypt failed: {err}")));
            }
            self.decrypt_time += started.elapsed();
            self.decrypted += chunk.len();

            let mut offset = 0usize;
            for &idx in chunk {
                let s = &self.samples[idx];
                let span_end = offset + s.len;
                spool[s.start..s.end()].copy_from_slice(&self.scratch[offset..span_end]);
                self.seals[idx] = SampleSeal::Clear;
                offset = span_end;
            }

            at = next;
        }
        Ok(())
    }

    fn decrypt_range(&mut self, spool: &mut [u8], start: usize, end: usize) -> io::Result<()> {
        if self.plain {
            return Ok(());
        }
        let first = self.samples.partition_point(|s| s.end() <= start);
        let rel_end = self.samples[first..].partition_point(|s| s.start < end);
        let last_excl = first + rel_end;
        if first >= last_excl {
            return Ok(());
        }
        if (first..last_excl).any(|i| self.seals[i] == SampleSeal::Lost) {
            return Err(io::Error::other("encrypted sample state was lost"));
        }
        if (first..last_excl).all(|i| self.seals[i] == SampleSeal::Clear) {
            return Ok(());
        }
        let batch = self.batch_range_for(first, last_excl);
        self.decrypt_batch(spool, batch)
    }

    fn background_step(
        &mut self,
        spool: &mut Vec<u8>,
        next_idx: &mut usize,
    ) -> io::Result<Option<Option<CdmKeys>>> {
        self.index_available(spool);
        let found = (*next_idx..self.seals.len()).find(|&i| self.seals[i] == SampleSeal::Encrypted);
        let Some(first) = found else {
            let released = match std::mem::replace(&mut self.license, LicenseState::Released) {
                LicenseState::Held(keys) => Some(keys),
                LicenseState::Pending | LicenseState::Released => None,
            };
            return Ok(Some(released));
        };
        let batch_end = self
            .batch_range_for(first, first + 1)
            .end
            .min(first + MAX_BATCH_SAMPLES)
            .max(first + 1)
            .min(self.samples.len());
        self.decrypt_batch(spool, first..batch_end)?;
        *next_idx = batch_end;
        Ok(None)
    }

    fn splice_entry(&self, at: Duration, complete: bool) -> Option<SpliceEntry> {
        let head = self.init_end?;
        let last = self.fragments.last()?;
        if at > last.start && !complete {
            return None;
        }
        let found = self.fragments.iter().rev().find(|f| f.start <= at)?;
        Some(SpliceEntry {
            head,
            graft: found.at,
            into: at.saturating_sub(found.start),
            in_first_fragment: found.at == head,
        })
    }

    fn all_clear(&self) -> bool {
        self.plain || matches!(self.license, LicenseState::Released)
    }
}

fn next_batch_end(samples: &[SampleEntry], pending: &[usize], start: usize) -> usize {
    let mut end = start;
    let mut bytes = 0usize;
    while end < pending.len() && end - start < MAX_BATCH_SAMPLES {
        let s = &samples[pending[end]];
        let wire = s.len.saturating_add(24).saturating_add(s.subs.len() * 8);
        if end > start && bytes.saturating_add(wire) > MAX_BATCH_BYTES {
            break;
        }
        bytes = bytes.saturating_add(wire);
        end += 1;
    }
    end
}

struct StreamBuffer {
    spool:    Vec<u8>,
    total:    Option<u64>,
    complete: bool,
    failed:   Option<String>,
    cenc:     CencState,
}

struct SharedStream {
    state:  Mutex<StreamBuffer>,
    notify: Condvar,
}

impl SharedStream {
    fn finish(&self, error: Option<String>) {
        if let Ok(mut st) = self.state.lock() {
            if st.failed.is_none() {
                st.failed = error;
            }
            st.complete = true;
        }
        self.notify.notify_all();
    }
}

/// Shared handle over a downloading (and lazily decrypting) Apple Music track.
#[derive(Clone)]
pub struct MediaStream {
    shared:  Arc<SharedStream>,
    arrived: watch::Receiver<usize>,
}

impl MediaStream {
    /// Starts streaming `resp` into memory. When `plain` is false, bytes are indexed as
    /// CENC fMP4 fragments and decrypted lazily once `install_license` provides the CDM.
    pub fn start(resp: reqwest::Response, plain: bool) -> Self {
        let total = resp.content_length();
        let initial_cap = total
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(PREROLL_BYTES)
            .min(MAX_TRACK_BYTES);

        let shared = Arc::new(SharedStream {
            state: Mutex::new(StreamBuffer {
                spool:    Vec::with_capacity(initial_cap),
                total,
                complete: false,
                failed:   None,
                cenc:     CencState {
                    plain,
                    ..CencState::default()
                },
            }),
            notify: Condvar::new(),
        });
        let (tx, arrived) = watch::channel(0usize);
        tokio::spawn(pump_response(resp, Arc::downgrade(&shared), tx));
        Self { shared, arrived }
    }

    /// Creates a `MediaStream` directly from in-memory bytes (for tests).
    #[cfg(test)]
    pub(crate) fn from_bytes(bytes: Vec<u8>, plain: bool) -> Self {
        let len = bytes.len();
        let shared = Arc::new(SharedStream {
            state: Mutex::new(StreamBuffer {
                spool:    bytes,
                total:    Some(len as u64),
                complete: true,
                failed:   None,
                cenc:     CencState {
                    plain,
                    ..CencState::default()
                },
            }),
            notify: Condvar::new(),
        });
        let (_, arrived) = watch::channel(len);
        Self { shared, arrived }
    }

    pub fn install_license(&self, cdm: Cdm, key_id: Vec<u8>) -> Result<(), AppError> {
        let mut st = self
            .shared
            .state
            .lock()
            .map_err(|_| AppError::Playback("stream buffer poisoned".into()))?;
        st.cenc.license = LicenseState::Held(CdmKeys { cdm, key_id });
        Ok(())
    }

    /// Returns whether all samples are already clear (or the asset is unencrypted),
    /// meaning a paused track no longer relies on an active CDM license.
    pub fn is_all_clear(&self) -> bool {
        self.shared
            .state
            .lock()
            .map(|st| st.cenc.all_clear())
            .unwrap_or(false)
    }

    /// Spawns a weak background pass that decrypts remaining samples after the download
    /// completes and drops the CDM handle so the host process can retire.
    pub fn release_when_downloaded(&self) {
        let weak = Arc::downgrade(&self.shared);
        let mut arrived = self.arrived.clone();
        tokio::spawn(async move {
            loop {
                match weak.upgrade() {
                    Some(shared) => {
                        let done = shared.state.lock().map(|s| s.complete).unwrap_or(true);
                        if done {
                            break;
                        }
                    }
                    None => return,
                }
                if arrived.changed().await.is_err() {
                    break;
                }
            }
            let _ = tokio::task::spawn_blocking(move || clear_remaining_samples(weak)).await;
        });
    }

    /// Waits for `moov` + `PREROLL_BYTES` (or preroll bytes / EOF for plain assets).
    pub async fn prime(&mut self) -> Result<(), AppError> {
        loop {
            let (wanted, arrived, plain, done) = {
                let mut st = self
                    .shared
                    .state
                    .lock()
                    .map_err(|_| AppError::Playback("stream buffer poisoned".into()))?;
                let StreamBuffer {
                    spool,
                    cenc,
                    complete,
                    total,
                    ..
                } = &mut *st;
                if cenc.plain {
                    let target = total
                        .and_then(|t| usize::try_from(t).ok())
                        .map_or(PREROLL_BYTES, |t| t.min(PREROLL_BYTES));
                    (Some(target), spool.len(), true, *complete)
                } else {
                    cenc.index_available(spool);
                    (
                        cenc.init_end.map(|end| end + PREROLL_BYTES),
                        spool.len(),
                        false,
                        *complete,
                    )
                }
            };

            match wanted {
                Some(target) => {
                    self.wait_for(target).await?;
                    break;
                }
                None if done => {
                    if !plain {
                        return Err(AppError::Playback("apple stream contains no moov box".into()));
                    }
                    break;
                }
                None => self.wait_for(arrived + 1).await?,
            }
        }
        Ok(())
    }

    async fn wait_for(&mut self, wanted: usize) -> Result<(), AppError> {
        loop {
            let arrived = *self.arrived.borrow_and_update();
            let done = self.done();
            if arrived >= wanted || done {
                break;
            }
            if self.arrived.changed().await.is_err() {
                break;
            }
        }
        if let Some(err) = self.failed() {
            return Err(AppError::Playback(format!("apple stream failed: {err}")));
        }
        Ok(())
    }

    pub fn done(&self) -> bool {
        self.shared
            .state
            .lock()
            .map(|s| s.complete)
            .unwrap_or(true)
    }

    pub fn failed(&self) -> Option<String> {
        self.shared
            .state
            .lock()
            .ok()
            .and_then(|s| s.failed.clone())
    }

    pub fn is_plain(&self) -> bool {
        self.shared
            .state
            .lock()
            .map(|s| s.cenc.plain)
            .unwrap_or(false)
    }

    pub fn total_bytes(&self) -> Option<u64> {
        let st = self.shared.state.lock().ok()?;
        st.total
            .or_else(|| st.complete.then_some(st.spool.len() as u64))
    }

    pub fn arrived_bytes(&self) -> usize {
        self.shared
            .state
            .lock()
            .map(|s| s.spool.len())
            .unwrap_or(0)
    }

    pub fn init_header(&self) -> Option<Vec<u8>> {
        let st = self.shared.state.lock().ok()?;
        let end = st.cenc.init_end?;
        st.spool.get(..end).map(|s| s.to_vec())
    }

    pub fn metrics(&self) -> (usize, usize, Duration) {
        self.shared
            .state
            .lock()
            .map(|s| {
                (
                    s.cenc.decrypted,
                    s.cenc.samples.len(),
                    s.cenc.decrypt_time,
                )
            })
            .unwrap_or_default()
    }

    pub fn reader(&self) -> MediaReader {
        MediaReader {
            shared: Arc::clone(&self.shared),
            head:   0,
            graft:  0,
            at:     0,
        }
    }

    pub fn reader_at(&self, entry: SpliceEntry) -> MediaReader {
        MediaReader {
            shared: Arc::clone(&self.shared),
            head:   entry.head,
            graft:  entry.graft,
            at:     0,
        }
    }

    /// Blocks until the fragment containing `pos` has been indexed or the stream ends/times out.
    pub fn splice_entry(&self, pos: Duration) -> Option<SpliceEntry> {
        let mut st = self.shared.state.lock().ok()?;
        let deadline = Instant::now() + READ_PATIENCE;
        loop {
            let StreamBuffer {
                spool,
                cenc,
                complete,
                failed,
                ..
            } = &mut *st;
            cenc.index_available(spool);
            if let Some(entry) = cenc.splice_entry(pos, *complete) {
                return Some(entry);
            }
            if *complete || failed.is_some() {
                return None;
            }
            let left = deadline.checked_duration_since(Instant::now())?;
            let (guard, timed_out) = self.shared.notify.wait_timeout(st, left).ok()?;
            if timed_out.timed_out() {
                return None;
            }
            st = guard;
        }
    }
}

async fn pump_response(
    mut resp: reqwest::Response,
    weak: Weak<SharedStream>,
    progress: watch::Sender<usize>,
) {
    loop {
        let chunk = match resp.chunk().await {
            Ok(Some(c)) => c,
            Ok(None) => {
                if let Some(shared) = weak.upgrade() {
                    shared.finish(None);
                }
                let _ = progress.send_modify(|_| {});
                return;
            }
            Err(err) => {
                let msg = err.without_url().to_string();
                log::warn!("apple: media download broke: {msg}");
                if let Some(shared) = weak.upgrade() {
                    shared.finish(Some(msg));
                }
                let _ = progress.send_modify(|_| {});
                return;
            }
        };

        let Some(shared) = weak.upgrade() else {
            return;
        };
        let arrived = {
            let Ok(mut st) = shared.state.lock() else {
                return;
            };
            if st
                .spool
                .len()
                .checked_add(chunk.len())
                .is_none_or(|n| n > MAX_TRACK_BYTES)
            {
                drop(st);
                shared.finish(Some("track exceeds maximum buffer size".into()));
                let _ = progress.send_modify(|_| {});
                return;
            }
            st.spool.extend_from_slice(&chunk);
            st.spool.len()
        };
        shared.notify.notify_all();
        let _ = progress.send(arrived);
    }
}

fn clear_remaining_samples(weak: Weak<SharedStream>) {
    let mut next_idx = 0usize;
    loop {
        let Some(shared) = weak.upgrade() else {
            return;
        };
        let step = {
            let Ok(mut st) = shared.state.lock() else {
                return;
            };
            let StreamBuffer { spool, cenc, .. } = &mut *st;
            cenc.background_step(spool, &mut next_idx)
        };
        drop(shared);
        match step {
            Ok(Some(released)) => {
                drop(released);
                return;
            }
            Ok(None) => {}
            Err(err) => {
                log::warn!("apple: background decrypt stopped: {err}");
                return;
            }
        }
        std::thread::sleep(DECRYPT_PACE);
    }
}

/// Blocking `Read + Seek` cursor over a `MediaStream` (whole file or `[0..head] + [graft..]`).
pub struct MediaReader {
    shared: Arc<SharedStream>,
    head:   usize,
    graft:  usize,
    at:     u64,
}

impl MediaReader {
    fn physical(&self, virt: u64) -> u64 {
        if virt < self.head as u64 {
            virt
        } else {
            self.graft as u64 + (virt - self.head as u64)
        }
    }

    fn presented(&self, phys_len: usize) -> u64 {
        self.head as u64 + phys_len.saturating_sub(self.graft) as u64
    }

    fn end(&self) -> u64 {
        let Ok(mut st) = self.shared.state.lock() else {
            return 0;
        };
        loop {
            if let Some(total) = st.total {
                return self.presented(total as usize);
            }
            if st.complete {
                return self.presented(st.spool.len());
            }
            let Ok((guard, timed_out)) = self.shared.notify.wait_timeout(st, READ_PATIENCE) else {
                return 0;
            };
            if timed_out.timed_out() {
                return self.presented(guard.spool.len());
            }
            st = guard;
        }
    }
}

impl Read for MediaReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        let mut st = self
            .shared
            .state
            .lock()
            .map_err(|_| io::Error::other("stream buffer poisoned"))?;

        loop {
            let StreamBuffer {
                spool,
                cenc,
                complete,
                total,
                ..
            } = &mut *st;

            let phys_limit = if cenc.plain {
                spool.len()
            } else {
                cenc.index_available(spool);
                if *complete {
                    spool.len()
                } else {
                    cenc.walked
                }
            };
            let virt_limit = self.presented(phys_limit);

            if self.at < virt_limit {
                let max_bytes = if self.at < self.head as u64 {
                    out.len().min(self.head - self.at as usize)
                } else {
                    out.len()
                };
                let phys_start = self.physical(self.at) as usize;
                let phys_end = (phys_start + max_bytes).min(self.physical(virt_limit) as usize);
                cenc.decrypt_range(spool, phys_start, phys_end)?;
                let n = phys_end - phys_start;
                out[..n].copy_from_slice(&spool[phys_start..phys_end]);
                self.at += n as u64;
                return Ok(n);
            }

            let tail_probe = !cenc.plain
                && total
                    .map(|t| self.presented(t as usize))
                    .is_some_and(|t| self.at + TAIL_PROBE_BYTES >= t && self.at < t);

            if let Some(err) = &st.failed {
                return Err(io::Error::other(err.clone()));
            }
            if st.complete || tail_probe {
                return Ok(0);
            }

            let (guard, timed_out) = self
                .shared
                .notify
                .wait_timeout(st, READ_PATIENCE)
                .map_err(|_| io::Error::other("stream buffer poisoned"))?;
            if timed_out.timed_out() {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "apple stream timed out waiting for bytes",
                ));
            }
            st = guard;
        }
    }
}

impl Seek for MediaReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(offset) => i128::from(offset),
            SeekFrom::Current(delta) => i128::from(self.at) + i128::from(delta),
            SeekFrom::End(delta) => i128::from(self.end()) + i128::from(delta),
        };
        if target < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot seek before stream start",
            ));
        }
        self.at = target as u64;
        Ok(self.at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_over_encrypted_sample_without_license_fails() {
        let mut cenc = CencState {
            walked: 64,
            samples: vec![SampleEntry {
                start: 16,
                len:   16,
                iv:    [0; 16],
                subs:  Vec::new(),
            }],
            seals: vec![SampleSeal::Encrypted],
            ..CencState::default()
        };
        let mut spool = vec![7u8; 64];
        assert!(cenc.decrypt_range(&mut spool, 0, 8).is_ok());
        assert!(cenc.decrypt_range(&mut spool, 0, 32).is_err());
    }

    #[test]
    fn splice_entry_selects_latest_fragment_not_after_target() {
        let cenc = CencState {
            init_end: Some(1000),
            fragments: vec![
                FragmentLoc {
                    at:    1000,
                    start: Duration::ZERO,
                },
                FragmentLoc {
                    at:    5000,
                    start: Duration::from_secs(10),
                },
                FragmentLoc {
                    at:    9000,
                    start: Duration::from_secs(20),
                },
            ],
            ..CencState::default()
        };

        let entry = cenc.splice_entry(Duration::from_secs(12), false).unwrap();
        assert_eq!(entry.head, 1000);
        assert_eq!(entry.graft, 5000);
        assert_eq!(entry.into, Duration::from_secs(2));
        assert!(!entry.in_first_fragment);

        assert!(cenc.splice_entry(Duration::from_secs(30), false).is_none());
        let tail = cenc.splice_entry(Duration::from_secs(30), true).unwrap();
        assert_eq!(tail.graft, 9000);

        let first = cenc.splice_entry(Duration::from_secs(3), false).unwrap();
        assert!(first.in_first_fragment);
    }

    #[test]
    fn spliced_reader_presents_head_then_graft() {
        let mut data = vec![1u8; 10];
        data.extend_from_slice(&[2u8; 20]);
        data.extend_from_slice(&[3u8; 15]);
        let media = MediaStream::from_bytes(data, true);
        let mut reader = media.reader_at(SpliceEntry {
            head:              10,
            graft:             30,
            into:              Duration::ZERO,
            in_first_fragment: false,
        });
        let mut out = Vec::new();
        reader.read_to_end(&mut out).unwrap();
        assert_eq!(out.len(), 25);
        assert!(out[..10].iter().all(|&b| b == 1));
        assert!(out[10..].iter().all(|&b| b == 3));
    }

    #[test]
    fn sample_batches_expand_per_fragment_and_respect_caps() {
        let samples = (0..5)
            .map(|i| SampleEntry {
                start: i * 16,
                len:   16,
                iv:    [0; 16],
                subs:  Vec::new(),
            })
            .collect::<Vec<_>>();
        let mut cenc = CencState {
            walked: 80,
            sample_batches: vec![0..3, 3..5],
            samples,
            seals: vec![
                SampleSeal::Clear,
                SampleSeal::Encrypted,
                SampleSeal::Lost,
                SampleSeal::Encrypted,
                SampleSeal::Encrypted,
            ],
            ..CencState::default()
        };

        assert_eq!(cenc.batch_range_for(1, 2), 0..3);
        assert_eq!(cenc.batch_range_for(2, 4), 0..5);
        assert_eq!(cenc.batch_range_for(3, 5), 3..5);

        let mut spool = vec![0u8; 80];
        assert!(cenc.decrypt_range(&mut spool, 0, 16).is_ok());
        assert!(cenc.decrypt_range(&mut spool, 32, 48).is_err());
        assert!(cenc.decrypt_range(&mut spool, 16, 32).is_err());
        // Missing license must not mark Encrypted samples as Lost.
        assert_eq!(cenc.seals[1], SampleSeal::Encrypted);
        assert_eq!(cenc.seals[3], SampleSeal::Encrypted);
        assert_eq!(cenc.seals[4], SampleSeal::Encrypted);

        // Batch chunking bounds stay within MAX_BATCH_SAMPLES and MAX_BATCH_BYTES.
        let large_samples: Vec<SampleEntry> = (0..3)
            .map(|i| SampleEntry {
                start: i * (3 * 1024 * 1024),
                len:   3 * 1024 * 1024,
                iv:    [0; 16],
                subs:  Vec::new(),
            })
            .collect();
        let pending = vec![0, 1, 2];
        assert_eq!(next_batch_end(&large_samples, &pending, 0), 1);
    }
}
