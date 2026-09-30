//! Keeps seconds of decoded audio between whatever makes it and the speaker, so
//! a stall anywhere upstream is never heard.
//!
//! The audio device pulls samples on a real-time thread. When that thread (or
//! the one feeding it) has to decode, resample, or read a file that is still
//! downloading, every hiccup - a busy CPU, the machine lagging, a slow network -
//! comes out of the speaker as a stutter. Here the device thread only copies
//! samples out of a queue. Everything else runs on a raised-priority thread that
//! works up to `AHEAD` in front of the speaker, so it can be held up for seconds
//! before anyone hears a thing.
//!
//! Every chunk remembers where in the track it came from, so the queue also
//! knows what the listener is hearing *now* (`heard_ms`). The decoder is seconds
//! ahead of that, so positions are reported from here.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use rodio::source::SeekError;
use rodio::{ChannelCount, Sample, SampleRate, Source};

/// How much decoded audio is kept ready in front of the speaker.
pub const AHEAD: Duration = Duration::from_secs(4);

/// Frames pulled from a source per chunk on the YouTube side (~23ms at 44.1k).
const BLOCK: usize = 1024;

/// Silence handed out at a time while there is nothing to play, in frames.
/// Small, so playback picks up again within a couple of milliseconds of audio
/// arriving, and a whole number of frames, so the channels never swap.
const GAP: usize = 64;

/// How often the heard position is brought up to date inside a chunk, in frames.
const TICK: usize = 256;

/// No chunk has been heard yet.
const NOTHING: u64 = u64::MAX;

/// How many frames `span` of audio is at `rate`.
pub fn frames_in(span: Duration, rate: SampleRate) -> usize {
    (span.as_millis() as u64 * rate as u64 / 1000) as usize
}

/// A run of decoded, interleaved audio and where in the track it came from.
pub struct Chunk {
    pub samples:  Vec<Sample>,
    pub rate:     SampleRate,
    /// Which play it belongs to (librespot's play request id). A position only
    /// means something against the play it was taken in.
    pub tag:      u64,
    /// Where in the track it starts and ends, in milliseconds.
    pub start_ms: f64,
    pub end_ms:   f64,
}

struct State {
    chunks:   VecDeque<Chunk>,
    /// Frames waiting, over all of `chunks`.
    frames:   usize,
    /// The producer reached the end of its source.
    finished: bool,
    /// The listening side is gone for good.
    closed:   bool,
    /// A seek for the producer to carry out.
    seek:     Option<Duration>,
    /// Bumped by every flush. A chunk made before one is stale.
    epoch:    u64,
    /// Used sample buffers, handed back so the producer need not allocate and
    /// the device thread need not free.
    spare:    Vec<Vec<Sample>>,
}

pub struct Ring {
    state:     Mutex<State>,
    changed:   Condvar,
    channels:  usize,
    /// Paused: the device gets silence and the queue is kept for later.
    held:      AtomicBool,
    /// The producer is mid-track, so running dry now is a real underrun.
    streaming: AtomicBool,
    heard_tag: AtomicU64,
    heard_ms:  AtomicU64,
    flushes:   AtomicU64,
    underruns: AtomicU64,
}

impl Ring {
    pub fn new(channels: usize) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                chunks:   VecDeque::new(),
                frames:   0,
                finished: false,
                closed:   false,
                seek:     None,
                epoch:    0,
                spare:    Vec::new(),
            }),
            changed:   Condvar::new(),
            channels:  channels.max(1),
            held:      AtomicBool::new(false),
            streaming: AtomicBool::new(false),
            heard_tag: AtomicU64::new(NOTHING),
            heard_ms:  AtomicU64::new(0),
            flushes:   AtomicU64::new(0),
            underruns: AtomicU64::new(0),
        })
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Queue a chunk, waiting while `limit` frames are already queued.
    ///
    /// Returns false when the chunk was not queued: a flush since `epoch` made
    /// it stale, the listener is gone, a seek is waiting, or `interrupt` said to
    /// stop waiting.
    ///
    /// While held (paused) this does not wait, up to a hard ceiling: the thread
    /// pushing may be the one that has to act on the pause, and it can't while
    /// it is stuck in here.
    pub fn push(
        &self,
        chunk: Chunk,
        epoch: Option<u64>,
        limit: usize,
        interrupt: &dyn Fn() -> bool,
    ) -> bool {
        let limit = limit.max(1);
        let frames = chunk.samples.len() / self.channels;
        let mut state = self.lock();
        let mut stuck_since: Option<Instant> = None;
        loop {
            if state.closed || state.seek.is_some() || epoch.is_some_and(|e| e != state.epoch) {
                return false;
            }
            let full = if self.held.load(Ordering::Relaxed) {
                state.frames >= limit * 3
            } else {
                state.frames >= limit
            };
            if !full {
                break;
            }
            if interrupt() {
                return false;
            }
            if self.held.load(Ordering::Relaxed) {
                // paused and far past full: nobody is coming to drain it, so
                // let this chunk go rather than wedge the thread
                let since = *stuck_since.get_or_insert_with(Instant::now);
                if since.elapsed() > Duration::from_secs(2) {
                    return false;
                }
            }
            state = self
                .changed
                .wait_timeout(state, Duration::from_millis(50))
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        state.frames += frames;
        state.finished = false;
        state.chunks.push_back(chunk);
        true
    }

    /// A cleared buffer to fill, reused when one is going.
    pub fn spare(&self, capacity: usize) -> Vec<Sample> {
        let mut buffer = self.lock().spare.pop().unwrap_or_default();
        buffer.clear();
        buffer.reserve(capacity);
        buffer
    }

    /// Throw away everything queued.
    pub fn flush(&self) {
        let mut state = self.lock();
        Self::clear(&mut state);
        drop(state);
        self.flushes.fetch_add(1, Ordering::SeqCst);
        self.changed.notify_all();
    }

    fn clear(state: &mut State) {
        while let Some(chunk) = state.chunks.pop_front() {
            if state.spare.len() < 8 {
                state.spare.push(chunk.samples);
            }
        }
        state.frames = 0;
        state.finished = false;
        state.epoch += 1;
    }

    /// Flush and have the producer seek to `pos`. What is heard is `pos` from
    /// here on, even before the producer gets there.
    pub fn request_seek(&self, pos: Duration) {
        let mut state = self.lock();
        Self::clear(&mut state);
        state.seek = Some(pos);
        self.heard_ms.store(pos.as_millis() as u64, Ordering::Relaxed);
        drop(state);
        self.flushes.fetch_add(1, Ordering::SeqCst);
        self.changed.notify_all();
    }

    /// A waiting seek, and the epoch to push against once it is done.
    fn take_seek(&self) -> Option<(Duration, u64)> {
        let mut state = self.lock();
        state.seek.take().map(|pos| (pos, state.epoch))
    }

    pub fn epoch(&self) -> u64 {
        self.lock().epoch
    }

    /// The producer is out of audio. Blocks until there is a reason to carry on
    /// (a seek); false when the listener is gone instead.
    fn finish_and_wait(&self, epoch: u64) -> bool {
        let mut state = self.lock();
        if state.epoch == epoch && state.seek.is_none() {
            state.finished = true;
            self.streaming.store(false, Ordering::Relaxed);
        }
        self.changed.notify_all();
        while !state.closed && state.seek.is_none() {
            state = self.changed.wait(state).unwrap_or_else(PoisonError::into_inner);
        }
        !state.closed
    }

    fn close(&self) {
        self.lock().closed = true;
        self.changed.notify_all();
    }

    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }

    /// Pause (true) or resume (false) what the device hears, keeping the queue.
    pub fn set_held(&self, held: bool) {
        self.held.store(held, Ordering::Relaxed);
        self.changed.notify_all();
    }

    pub fn held(&self) -> bool {
        self.held.load(Ordering::Relaxed)
    }

    /// Whether running dry now counts as an underrun (the producer is mid-track).
    pub fn set_streaming(&self, streaming: bool) {
        self.streaming.store(streaming, Ordering::Relaxed);
    }

    /// Where the listener is in play `tag`, if that is the play being heard.
    pub fn heard(&self, tag: u64) -> Option<u32> {
        (self.heard_tag.load(Ordering::Relaxed) == tag)
            .then(|| self.heard_ms.load(Ordering::Relaxed).min(u32::MAX as u64) as u32)
    }

    /// Where the listener is, in whatever is being heard.
    pub fn heard_ms(&self) -> u64 {
        self.heard_ms.load(Ordering::Relaxed)
    }

    /// Say that `ms` into play `tag` is what is heard next.
    pub fn set_heard(&self, tag: u64, ms: u64) {
        self.heard_tag.store(tag, Ordering::Relaxed);
        self.heard_ms.store(ms, Ordering::Relaxed);
    }

    /// Milliseconds still queued from play `tag`.
    pub fn queued_ms_of(&self, tag: u64) -> u64 {
        let state = self.lock();
        let ms: f64 = state
            .chunks
            .iter()
            .filter(|c| c.tag == tag && c.rate > 0)
            .map(|c| (c.samples.len() / self.channels) as f64 * 1000.0 / c.rate as f64)
            .sum();
        ms as u64
    }

    /// How many flushes there have been. A change means a discontinuity.
    pub fn flushes(&self) -> u64 {
        self.flushes.load(Ordering::SeqCst)
    }

    /// How many times playback has run dry mid-track.
    pub fn underruns(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }

    fn pop(&self, rate: SampleRate, used: Vec<Sample>) -> Pop {
        let mut state = self.lock();
        if used.capacity() > 0 && state.spare.len() < 8 {
            state.spare.push(used);
        }
        loop {
            let Some(chunk) = state.chunks.pop_front() else {
                return if state.finished { Pop::Finished } else { Pop::Empty };
            };
            state.frames = state.frames.saturating_sub(chunk.samples.len() / self.channels);
            self.changed.notify_all();
            // made for an output that has since been replaced by one at
            // another rate; playing it would come out at the wrong pitch
            if chunk.rate == rate {
                return Pop::Chunk(chunk);
            }
            if state.spare.len() < 8 {
                state.spare.push(chunk.samples);
            }
        }
    }
}

enum Pop {
    Chunk(Chunk),
    Empty,
    Finished,
}

/// What the device plays: whatever is at the front of a `Ring`.
///
/// Never blocks for long and never decodes. When the ring is empty it plays
/// silence and keeps going rather than ending, so a producer that is late is a
/// brief gap and never the end of the track.
pub struct RingSource {
    ring:     Arc<Ring>,
    channels: ChannelCount,
    rate:     SampleRate,
    cur:      Vec<Sample>,
    idx:      usize,
    tag:      u64,
    start_ms: f64,
    end_ms:   f64,
    /// Silent samples left to hand out before looking again.
    silence:  usize,
    starving: bool,
    /// Plays forever and leaves the ring open when dropped (the librespot
    /// output, which outlives any one device). Otherwise it ends with its
    /// source and closes the ring behind it.
    endless:  bool,
}

impl RingSource {
    /// A source that follows the ring for good, surviving the device being
    /// swapped underneath it.
    pub fn endless(ring: Arc<Ring>, channels: ChannelCount, rate: SampleRate) -> Self {
        Self::new(ring, channels, rate, true)
    }

    fn new(ring: Arc<Ring>, channels: ChannelCount, rate: SampleRate, endless: bool) -> Self {
        Self {
            ring,
            channels: channels.max(1),
            rate,
            cur: Vec::new(),
            idx: 0,
            tag: NOTHING,
            start_ms: 0.0,
            end_ms: 0.0,
            silence: 0,
            starving: false,
            endless,
        }
    }

    fn note_heard(&self) {
        let frac = if self.cur.is_empty() { 0.0 } else { self.idx as f64 / self.cur.len() as f64 };
        let ms = self.start_ms + (self.end_ms - self.start_ms) * frac;
        self.ring.set_heard(self.tag, ms.max(0.0) as u64);
    }
}

impl Iterator for RingSource {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        let channels = self.channels as usize;
        loop {
            if self.silence > 0 {
                self.silence -= 1;
                return Some(0.0);
            }
            if self.idx < self.cur.len() {
                // only between frames, so left and right never trade places
                if self.idx % channels == 0 {
                    if self.ring.held() {
                        self.silence = GAP * channels;
                        continue;
                    }
                    if self.idx % (TICK * channels) == 0 {
                        self.note_heard();
                    }
                }
                let sample = self.cur[self.idx];
                self.idx += 1;
                return Some(sample);
            }
            if self.ring.held() {
                self.silence = GAP * channels;
                continue;
            }
            let used = std::mem::take(&mut self.cur);
            match self.ring.pop(self.rate, used) {
                Pop::Chunk(chunk) => {
                    self.cur = chunk.samples;
                    self.idx = 0;
                    self.tag = chunk.tag;
                    self.start_ms = chunk.start_ms;
                    self.end_ms = chunk.end_ms;
                    self.starving = false;
                    self.note_heard();
                }
                Pop::Empty => {
                    if !self.starving && self.ring.streaming.load(Ordering::Relaxed) {
                        self.ring.underruns.fetch_add(1, Ordering::Relaxed);
                        self.starving = true;
                    }
                    self.silence = GAP * channels;
                }
                Pop::Finished if !self.endless => return None,
                Pop::Finished => self.silence = GAP * channels,
            }
        }
    }
}

impl Source for RingSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }

    /// Hands the seek to the producer and returns at once: the producer may
    /// have to wait on the network to get there, and this runs on the device
    /// thread.
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        if self.endless {
            return Err(SeekError::NotSupported { underlying_source: "the librespot output" });
        }
        self.ring.request_seek(pos);
        let used = std::mem::take(&mut self.cur);
        if used.capacity() > 0 {
            let mut state = self.ring.lock();
            if state.spare.len() < 8 {
                state.spare.push(used);
            }
        }
        self.idx = 0;
        self.starving = false;
        Ok(())
    }
}

impl Drop for RingSource {
    fn drop(&mut self) {
        if !self.endless {
            self.ring.close();
        }
    }
}

/// Play `inner` through a ring that a thread of its own keeps `AHEAD` full.
///
/// `at` is where in the track `inner` is (or, with `seek_first`, where it is
/// to be taken before anything plays). `media_clock` is the source's own idea
/// of where it is, for sources whose sample count isn't track time (a
/// time-stretched episode); without one, time is counted from the samples.
///
/// Returns the source to hand to rodio and the ring, whose `heard_ms` is the
/// position to report.
pub fn prebuffer<S>(
    inner: S,
    at: Duration,
    seek_first: bool,
    media_clock: Option<Arc<AtomicU64>>,
) -> (RingSource, Arc<Ring>)
where
    S: Source + Send + 'static,
{
    let channels = inner.channels().max(1);
    let rate = inner.sample_rate();
    let ring = Ring::new(channels as usize);
    ring.set_heard(0, at.as_millis() as u64);
    let feeding = Arc::clone(&ring);
    let spawned = thread::Builder::new()
        .name("audio-prebuffer".into())
        .spawn(move || fill(inner, feeding, at, seek_first, media_clock));
    if let Err(error) = spawned {
        // the source is gone with the closure; the ring ends at once and the
        // track reads as finished instead of hanging
        log::error!("cannot start the audio prebuffer thread: {error}");
        let mut state = ring.lock();
        state.finished = true;
    }
    let source = RingSource::new(Arc::clone(&ring), channels, rate, false);
    (source, ring)
}

fn fill<S: Source>(
    mut inner: S,
    ring: Arc<Ring>,
    at: Duration,
    seek_first: bool,
    media_clock: Option<Arc<AtomicU64>>,
) {
    boost_current_thread();
    let channels = inner.channels().max(1) as usize;
    let rate = inner.sample_rate();
    let limit = frames_in(AHEAD, rate);
    let stretched = media_clock.is_some();
    let mut speed = crate::stretch::speed();
    let mut epoch = ring.epoch();
    let mut pos_ms = at.as_millis() as f64;
    let mut pending = seek_first.then_some(at);

    loop {
        if let Some((pos, e)) = ring.take_seek() {
            pending = Some(pos);
            epoch = e;
        }
        if let Some(pos) = pending.take() {
            if let Err(error) = inner.try_seek(pos) {
                log::warn!("seek to {}ms failed: {error}", pos.as_millis());
            }
            pos_ms = pos.as_millis() as f64;
            if let Some(clock) = &media_clock {
                clock.store(pos.as_millis() as u64, Ordering::Relaxed);
            }
        }
        // the queue was stretched at the old speed; start again from what is
        // being heard so the new speed takes hold now, not seconds from now
        if stretched && speed_changed(speed) {
            speed = crate::stretch::speed();
            ring.request_seek(Duration::from_millis(ring.heard_ms()));
            continue;
        }

        let mut samples = ring.spare(BLOCK * channels);
        let mut ended = false;
        while samples.len() < BLOCK * channels {
            match inner.next() {
                Some(sample) => samples.push(sample),
                None => {
                    ended = true;
                    break;
                }
            }
        }
        samples.truncate(samples.len() - samples.len() % channels);

        let start_ms = pos_ms;
        pos_ms = match &media_clock {
            Some(clock) => (clock.load(Ordering::Relaxed) as f64).max(start_ms),
            None => start_ms + (samples.len() / channels) as f64 * 1000.0 / rate.max(1) as f64,
        };

        if !samples.is_empty() {
            ring.set_streaming(true);
            let chunk = Chunk { samples, rate, tag: 0, start_ms, end_ms: pos_ms };
            let interrupt = || stretched && speed_changed(speed);
            if !ring.push(chunk, Some(epoch), limit, &interrupt) {
                if ring.is_closed() {
                    return;
                }
                continue;
            }
        }
        if ended && !ring.finish_and_wait(epoch) {
            return;
        }
    }
}

fn speed_changed(speed: f32) -> bool {
    (crate::stretch::speed() - speed).abs() > 1e-3
}

/// Let the thread that feeds the audio output go first.
///
/// On a loaded machine an ordinary thread can be kept waiting for longer than
/// the audio queued ahead of it lasts. Windows gets the thread registered with
/// the multimedia scheduler (the same class the audio engine's own threads run
/// in), and the process opted out of power throttling, which otherwise parks it
/// on slow cores when its window is in the background. macOS gets the
/// user-interactive QoS class.
#[cfg(windows)]
pub fn boost_current_thread() {
    use std::cell::Cell;
    use windows_sys::Win32::System::Threading::{
        AvSetMmThreadCharacteristicsW, GetCurrentThread, SetThreadPriority,
        THREAD_PRIORITY_HIGHEST,
    };

    thread_local!(static BOOSTED: Cell<bool> = const { Cell::new(false) });
    if BOOSTED.with(|b| b.replace(true)) {
        return;
    }
    keep_full_speed();
    let task: Vec<u16> = "Pro Audio\0".encode_utf16().collect();
    let mut index = 0u32;
    let handle = unsafe { AvSetMmThreadCharacteristicsW(task.as_ptr(), &mut index) };
    if handle.is_null() {
        log::warn!("the multimedia scheduler refused the audio thread; raising its priority instead");
        unsafe {
            SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
        }
    }
}

#[cfg(windows)]
fn keep_full_speed() {
    use std::sync::Once;
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, ProcessPowerThrottling, SetProcessInformation,
        PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        PROCESS_POWER_THROTTLING_STATE,
    };

    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // control the execution-speed policy and turn it off
        let state = PROCESS_POWER_THROTTLING_STATE {
            Version:     PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask:   0,
        };
        let ok = unsafe {
            SetProcessInformation(
                GetCurrentProcess(),
                ProcessPowerThrottling,
                &state as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
            )
        };
        if ok == 0 {
            log::warn!("could not opt out of power throttling");
        }
    });
}

#[cfg(target_os = "macos")]
pub fn boost_current_thread() {
    extern "C" {
        fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
    }
    const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;
    unsafe {
        pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn boost_current_thread() {}

#[cfg(test)]
mod tests {
    use super::*;

    /// A source that counts up, so the order and completeness of what comes
    /// out can be checked.
    struct Counter {
        next: u32,
        len:  u32,
    }

    impl Iterator for Counter {
        type Item = Sample;
        fn next(&mut self) -> Option<Sample> {
            (self.next < self.len).then(|| {
                self.next += 1;
                self.next as Sample
            })
        }
    }

    impl Source for Counter {
        fn current_span_len(&self) -> Option<usize> {
            None
        }
        fn channels(&self) -> ChannelCount {
            2
        }
        fn sample_rate(&self) -> SampleRate {
            1000
        }
        fn total_duration(&self) -> Option<Duration> {
            None
        }
        fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
            self.next = pos.as_millis() as u32 * 2;
            Ok(())
        }
    }

    fn drain(source: &mut RingSource) -> Vec<Sample> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut out = Vec::new();
        for sample in source.by_ref() {
            if sample != 0.0 {
                out.push(sample);
            }
            assert!(Instant::now() < deadline, "the source never ended");
        }
        out
    }

    #[test]
    fn plays_everything_in_order_then_ends() {
        let (mut source, _ring) = prebuffer(Counter { next: 0, len: 10_000 }, Duration::ZERO, false, None);
        let out = drain(&mut source);
        assert_eq!(out.len(), 10_000);
        assert!(out.windows(2).all(|w| w[1] == w[0] + 1.0));
    }

    #[test]
    fn a_seek_lands_where_asked() {
        let (mut source, ring) = prebuffer(Counter { next: 0, len: 10_000 }, Duration::ZERO, false, None);
        source.next();
        source.try_seek(Duration::from_millis(3000)).unwrap();
        assert_eq!(ring.heard_ms(), 3000);
        let out = drain(&mut source);
        assert_eq!(out.first().copied(), Some(6001.0));
        assert_eq!(out.last().copied(), Some(10_000.0));
    }

    #[test]
    fn heard_position_follows_what_was_played() {
        let (mut source, ring) = prebuffer(Counter { next: 0, len: 20_000 }, Duration::ZERO, false, None);
        // 1000 frames at 1 kHz, stereo: one second in
        let mut taken = 0;
        while taken < 2000 {
            if source.next().unwrap() != 0.0 {
                taken += 1;
            }
        }
        // brought up to date every TICK frames: 256ms at this rate, ~6ms at 44.1k
        let heard = ring.heard_ms();
        assert!((1000 - TICK as u64..=1000).contains(&heard), "heard {heard}ms");
    }

    #[test]
    fn held_plays_silence_without_losing_anything() {
        let ring = Ring::new(2);
        let chunk = Chunk { samples: vec![1.0; 200], rate: 1000, tag: 7, start_ms: 0.0, end_ms: 100.0 };
        assert!(ring.push(chunk, None, 10_000, &|| false));
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000);
        ring.set_held(true);
        assert!((0..500).all(|_| source.next() == Some(0.0)));
        ring.set_held(false);
        let played = (0..1000).filter(|_| source.next() == Some(1.0)).count();
        assert_eq!(played, 200);
        assert!(ring.heard(7).is_some());
    }

    #[test]
    fn chunks_for_an_old_output_rate_are_skipped() {
        let ring = Ring::new(2);
        let old = Chunk { samples: vec![1.0; 20], rate: 44_100, tag: 1, start_ms: 0.0, end_ms: 1.0 };
        let new = Chunk { samples: vec![2.0; 20], rate: 48_000, tag: 1, start_ms: 1.0, end_ms: 2.0 };
        assert!(ring.push(old, None, 10_000, &|| false));
        assert!(ring.push(new, None, 10_000, &|| false));
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 48_000);
        assert!((0..20).all(|_| source.next() == Some(2.0)));
    }

    #[test]
    fn an_empty_ring_counts_one_underrun_mid_track() {
        let ring = Ring::new(2);
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000);
        ring.set_streaming(true);
        for _ in 0..1000 {
            source.next();
        }
        assert_eq!(ring.underruns(), 1);
    }
}
