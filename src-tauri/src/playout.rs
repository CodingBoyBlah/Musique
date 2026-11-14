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
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError, TryLockError};
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

/// How long the sound takes to go down before a pause, a seek or a skip cuts
/// it, and to come back up after one. Cutting a waveform mid-swing is a click.
const FADE: Duration = Duration::from_millis(20);

/// Used sample buffers kept for the producer to take back. Room for every
/// chunk a full ring can hold, so the device thread never frees one.
const SPARE: usize = 2048;

/// Chunks the device thread takes from the ring at a time and keeps to
/// itself. It never waits for the ring's lock: it tries it, and when someone
/// else has it, it plays on from these and tries again at the next chunk.
const RESERVE: usize = 4;

/// The device thread tops its reserve up when it is down to this many.
const LOW: usize = 2;

/// How much has to be queued before a stream that is starting (a new track,
/// the far side of a seek, the far side of a dropout) begins to sound. The
/// device used to play the first packet the moment it arrived, and ran dry
/// again before the second had been fetched and decoded: a hole a few
/// milliseconds into every track.
const START_AHEAD: Duration = Duration::from_millis(150);

/// How many `GAP`s of silence a starting stream is given to get that far
/// (about half a second) before what there is plays anyway.
const START_PATIENCE: u32 = 350;

/// When an output last played real sound, in milliseconds since `clock`.
static LAST_SOUND: AtomicU64 = AtomicU64::new(0);

fn clock() -> u64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64 + 1
}

/// Whether any output played sound in the last `within`. Housekeeping that
/// would disturb the audio threads (emptying the working set) asks first.
pub fn sounding(within: Duration) -> bool {
    let last = LAST_SOUND.load(Ordering::Relaxed);
    last != 0 && clock().saturating_sub(last) <= within.as_millis() as u64
}

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
    /// Milliseconds queued per play, so asking is not a walk over the queue
    /// with the device thread waiting on the lock.
    queued:   Vec<(u64, f64)>,
    /// Producers asleep in `push`, so the device thread only signals when
    /// someone is there to hear it.
    waiting:  usize,
}

impl State {
    fn add_queued(&mut self, tag: u64, ms: f64) {
        match self.queued.iter_mut().find(|(t, _)| *t == tag) {
            Some((_, total)) => *total += ms,
            None => self.queued.push((tag, ms)),
        }
    }

    fn sub_queued(&mut self, tag: u64, ms: f64) {
        if let Some(at) = self.queued.iter().position(|(t, _)| *t == tag) {
            self.queued[at].1 -= ms;
            if self.queued[at].1 <= 0.5 {
                self.queued.swap_remove(at);
            }
        }
    }
}

fn chunk_ms(chunk: &Chunk, channels: usize) -> f64 {
    if chunk.rate == 0 {
        return 0.0;
    }
    (chunk.samples.len() / channels) as f64 * 1000.0 / chunk.rate as f64
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
    /// The play a flush left alone (see `skip_to_tag`). A chunk of it that
    /// the device is partway through plays on.
    kept_tag:  AtomicU64,
    underruns: AtomicU64,
    /// Audio actually pulled by the output, normalized to 44.1 kHz frames.
    /// Queued, flushed, paused and underrun samples never enter this counter.
    pub rendered_frames: Arc<AtomicU64>,
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
                spare:    Vec::with_capacity(SPARE),
                queued:   Vec::with_capacity(4),
                waiting:  0,
            }),
            changed:   Condvar::new(),
            channels:  channels.max(1),
            held:      AtomicBool::new(false),
            streaming: AtomicBool::new(false),
            heard_tag: AtomicU64::new(NOTHING),
            heard_ms:  AtomicU64::new(0),
            flushes:   AtomicU64::new(0),
            kept_tag:  AtomicU64::new(NOTHING),
            underruns: AtomicU64::new(0),
            rendered_frames: Arc::new(AtomicU64::new(0)),
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
            state.waiting += 1;
            state = self
                .changed
                .wait_timeout(state, Duration::from_millis(50))
                .unwrap_or_else(PoisonError::into_inner)
                .0;
            state.waiting -= 1;
        }
        state.frames += frames;
        state.finished = false;
        let ms = chunk_ms(&chunk, self.channels);
        state.add_queued(chunk.tag, ms);
        state.chunks.push_back(chunk);
        // buffers the device thread handed back and nobody reused: freed
        // here, on this thread, once the lock is let go
        let used = if state.spare.len() > 8 { state.spare.split_off(8) } else { Vec::new() };
        drop(state);
        drop(used);
        true
    }

    /// A cleared buffer to fill, reused when one is going.
    pub fn spare(&self, capacity: usize) -> Vec<Sample> {
        let mut buffer = self.lock().spare.pop().unwrap_or_default();
        buffer.clear();
        buffer.reserve(capacity);
        buffer
    }

    /// Throw away everything queued, including what the device is partway
    /// through.
    pub fn flush(&self) {
        let mut state = self.lock();
        self.clear(&mut state);
        drop(state);
        self.changed.notify_all();
    }

    /// Flush for a jump to `ms` into play `tag` that the producer has yet to
    /// make. What is heard is `ms` from now until its audio arrives, so
    /// positions reported meanwhile don't drift back to where it was.
    pub fn flush_at(&self, tag: u64, ms: u64) {
        let mut state = self.lock();
        self.clear(&mut state);
        self.set_heard(tag, ms);
        drop(state);
        // silence until the producer catches up is expected, not an underrun
        self.set_streaming(false);
        self.changed.notify_all();
    }

    /// Bumps the flush count under the lock, so a chunk popped after it is
    /// never mistaken for one from before.
    fn clear(&self, state: &mut State) {
        while let Some(chunk) = state.chunks.pop_front() {
            if state.spare.len() < 8 {
                state.spare.push(chunk.samples);
            }
        }
        state.frames = 0;
        state.queued.clear();
        state.finished = false;
        state.epoch += 1;
        self.kept_tag.store(NOTHING, Ordering::SeqCst);
        self.flushes.fetch_add(1, Ordering::SeqCst);
    }

    /// Throw away what is queued ahead of play `tag` and carry straight on
    /// with it: the rest of a track whose successor is already decoded behind
    /// it. Nothing of `tag` is lost and its producer is not turned away.
    pub fn skip_to_tag(&self, tag: u64) {
        let mut dropped = Vec::new();
        let mut state = self.lock();
        while state.chunks.front().is_some_and(|chunk| chunk.tag != tag) {
            if let Some(chunk) = state.chunks.pop_front() {
                state.frames = state.frames.saturating_sub(chunk.samples.len() / self.channels);
                dropped.push(chunk.samples);
            }
        }
        state.queued.retain(|(t, _)| *t == tag);
        let start = state.chunks.front().map_or(0.0, |chunk| chunk.start_ms);
        self.kept_tag.store(tag, Ordering::SeqCst);
        self.flushes.fetch_add(1, Ordering::SeqCst);
        self.set_heard(tag, start.max(0.0) as u64);
        drop(state);
        drop(dropped);
        self.changed.notify_all();
    }

    /// Convert what is queued to `rate`, for an output that has just been
    /// replaced by one running at another rate. Only call with no output
    /// attached: the queue is taken out while it is converted.
    pub fn retime(&self, rate: SampleRate) {
        let chunks = {
            let mut state = self.lock();
            if state.chunks.iter().all(|chunk| chunk.rate == rate) {
                return;
            }
            std::mem::take(&mut state.chunks)
        };
        let mut converter: Option<(SampleRate, crate::resample::Resampler)> = None;
        let mut converted = VecDeque::with_capacity(chunks.len());
        for mut chunk in chunks {
            if chunk.rate != rate && chunk.rate != 0 {
                if converter.as_ref().map(|(from, _)| *from) != Some(chunk.rate) {
                    converter = crate::resample::Resampler::new(chunk.rate, rate, self.channels)
                        .map(|resampler| (chunk.rate, resampler));
                }
                if let Some((_, resampler)) = &mut converter {
                    chunk.samples = resampler.process(&chunk.samples);
                    chunk.rate = rate;
                }
            }
            converted.push_back(chunk);
        }
        let mut state = self.lock();
        // anything pushed meanwhile came after what was taken out
        converted.extend(state.chunks.drain(..));
        state.frames = converted.iter().map(|chunk| chunk.samples.len() / self.channels).sum();
        state.chunks = converted;
        drop(state);
        self.changed.notify_all();
    }

    /// Flush and have the producer seek to `pos`. What is heard is `pos` from
    /// here on, even before the producer gets there.
    pub fn request_seek(&self, pos: Duration) {
        let mut state = self.lock();
        self.clear(&mut state);
        state.seek = Some(pos);
        self.heard_ms.store(pos.as_millis() as u64, Ordering::Relaxed);
        drop(state);
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

    /// Puts audio an output had taken from the queue and not yet played back
    /// at its front, so the output opened next carries on from the same
    /// sample: one closed while paused (`sink::IDLE_CLOSE`) or swapped for
    /// another device. What a flush has since made stale is left out.
    fn put_back(&self, taken: Vec<(Chunk, u64)>) {
        let flushes = self.flushes();
        let kept = self.kept_tag.load(Ordering::SeqCst);
        let mut state = self.lock();
        for (chunk, taken_at) in taken.into_iter().rev() {
            if chunk.samples.is_empty() || (taken_at != flushes && chunk.tag != kept) {
                continue;
            }
            state.frames += chunk.samples.len() / self.channels;
            let ms = chunk_ms(&chunk, self.channels);
            state.add_queued(chunk.tag, ms);
            state.chunks.push_front(chunk);
        }
        drop(state);
        self.changed.notify_all();
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

    /// Milliseconds still queued from play `tag`. The few chunks the device
    /// thread has already taken for itself are not counted.
    pub fn queued_ms_of(&self, tag: u64) -> u64 {
        let state = self.lock();
        state
            .queued
            .iter()
            .find(|(t, _)| *t == tag)
            .map_or(0, |(_, ms)| ms.max(0.0) as u64)
    }

    /// How many flushes there have been. A change means a discontinuity.
    pub fn flushes(&self) -> u64 {
        self.flushes.load(Ordering::SeqCst)
    }

    /// How many times playback has run dry mid-track.
    pub fn underruns(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }

    /// Move chunks from the ring into the device thread's own `ahead`, and
    /// the buffers it has finished with back the other way.
    ///
    /// Runs on the device thread, so it never waits: when the lock is taken
    /// it returns at once and the caller plays on from what it has. Under the
    /// lock it only moves things - nothing is allocated or freed - and a
    /// producer is only signalled when one is waiting, after the lock is let go.
    fn refill(
        &self,
        rate: SampleRate,
        ahead: &mut VecDeque<(Chunk, u64)>,
        used: &mut Vec<Vec<Sample>>,
        starting: bool,
    ) -> Refill {
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return Refill::Busy,
        };
        while state.spare.len() < state.spare.capacity() {
            match used.pop() {
                Some(buffer) => state.spare.push(buffer),
                None => break,
            }
        }
        // a stream that is starting waits until enough of it is queued. not
        // when the producer has stopped mid-way or finished: then what is
        // here is all there will be (the tail of a track)
        if starting
            && ahead.is_empty()
            && !state.finished
            && self.streaming.load(Ordering::Relaxed)
            && state.frames < frames_in(START_AHEAD, rate)
        {
            return Refill::Starting;
        }
        let flushes = self.flushes.load(Ordering::SeqCst);
        while ahead.len() < RESERVE {
            let Some(chunk) = state.chunks.pop_front() else {
                break;
            };
            state.frames = state.frames.saturating_sub(chunk.samples.len() / self.channels);
            let ms = chunk_ms(&chunk, self.channels);
            state.sub_queued(chunk.tag, ms);
            // made for an output that has since been replaced by one at
            // another rate (and not converted: see `retime`); playing it
            // would come out at the wrong pitch
            if chunk.rate == rate {
                ahead.push_back((chunk, flushes));
            } else if state.spare.len() < state.spare.capacity() {
                state.spare.push(chunk.samples);
            }
        }
        let refill = if !ahead.is_empty() {
            Refill::Got
        } else if state.finished {
            Refill::Finished
        } else {
            Refill::Empty
        };
        let wake = state.waiting > 0;
        drop(state);
        if wake {
            self.changed.notify_all();
        }
        refill
    }
}

/// What became of an attempt to take chunks from the ring.
enum Refill {
    /// Someone else has the ring's lock. Not waited for.
    Busy,
    /// There is audio to play.
    Got,
    /// A stream is starting and not enough of it is queued yet.
    Starting,
    /// The ring has run dry.
    Empty,
    /// It has run dry and the producer has no more to give.
    Finished,
}

/// What the device plays: whatever is at the front of a `Ring`.
///
/// Never blocks and never decodes: the ring's lock is only ever tried, and a
/// few chunks kept here cover the moments someone else has it. When the ring is empty it plays
/// silence and keeps going rather than ending, so a producer that is late is a
/// brief gap and never the end of the track.
pub struct RingSource {
    ring:     Arc<Ring>,
    channels: ChannelCount,
    rate:     SampleRate,
    /// Chunks taken from the ring and not yet played, each with the ring's
    /// flush count when it was taken.
    ahead:    VecDeque<(Chunk, u64)>,
    /// Buffers played out, on their way back to the producer.
    used:     Vec<Vec<Sample>>,
    cur:      Vec<Sample>,
    idx:      usize,
    tag:      u64,
    start_ms: f64,
    end_ms:   f64,
    /// Silent samples left to hand out before looking again.
    silence:  usize,
    starving: bool,
    /// The ring's flush count when `cur` was taken. Once it moves on, the rest
    /// of `cur` is from before a seek or a skip and is dropped.
    flushes:  u64,
    /// Plays forever and leaves the ring open when dropped (the librespot
    /// output, which outlives any one device). Otherwise it ends with its
    /// source and closes the ring behind it.
    endless:  bool,
    counted: usize,
    rendered: u64,
    reported: u64,
    /// The level the sound is at, 0 to 1, and how far it moves per frame.
    gain:     f32,
    step:     f32,
    /// The equalizer, here so a band that is moved is heard at once and not
    /// after the queue has played out. (Left out under test, where exact
    /// samples are compared while other tests move its settings.)
    eq:       Option<crate::eq::Equalizer>,
    /// Sound is flowing. False from a dry ring until enough is queued again.
    flowing:  bool,
    /// `GAP`s of silence played waiting for a starting stream to queue up.
    waited:   u32,
}

impl RingSource {
    /// A source that follows the ring for good, surviving the device being
    /// swapped underneath it.
    pub fn endless(ring: Arc<Ring>, channels: ChannelCount, rate: SampleRate) -> Self {
        Self::new(ring, channels, rate, true)
    }

    /// Cuts at once, so a test can check exact samples either side of one.
    #[cfg(test)]
    pub fn without_fades(mut self) -> Self {
        self.step = 1.0;
        self
    }

    fn new(ring: Arc<Ring>, channels: ChannelCount, rate: SampleRate, endless: bool) -> Self {
        Self {
            ring,
            channels: channels.max(1),
            rate,
            ahead: VecDeque::with_capacity(RESERVE),
            used: Vec::with_capacity(RESERVE * 4),
            cur: Vec::new(),
            idx: 0,
            tag: NOTHING,
            start_ms: 0.0,
            end_ms: 0.0,
            silence: 0,
            starving: false,
            flushes: 0,
            endless,
            counted: 0,
            rendered: 0,
            reported: 0,
            gain: 1.0,
            step: 1.0 / frames_in(FADE, rate).max(1) as f32,
            eq: (!cfg!(test)).then(|| crate::eq::Equalizer::new(rate)),
            flowing: false,
            waited: 0,
        }
    }

    fn count_rendered(&mut self) {
        self.rendered += (self.idx.saturating_sub(self.counted) / self.channels as usize) as u64;
        self.counted = self.idx;
        let normalized = self.rendered * 44_100 / u64::from(self.rate.max(1));
        if normalized > self.reported {
            self.ring.rendered_frames.fetch_add(normalized - self.reported, Ordering::Relaxed);
            self.reported = normalized;
        }
    }

    fn note_heard(&mut self) {
        self.count_rendered();
        let frac = if self.cur.is_empty() { 0.0 } else { self.idx as f64 / self.cur.len() as f64 };
        let ms = self.start_ms + (self.end_ms - self.start_ms) * frac;
        self.ring.set_heard(self.tag, ms.max(0.0) as u64);
        LAST_SOUND.store(clock(), Ordering::Relaxed);
    }

    /// Put a played-out buffer aside for the producer. Only dropped here, on
    /// the device thread, if the ring's lock has been out of reach for long
    /// enough to fill the pile.
    fn hand_back(&mut self, buffer: Vec<Sample>) {
        if buffer.capacity() > 0 && self.used.len() < self.used.capacity() {
            self.used.push(buffer);
        }
    }

    /// Whether what is left of `cur` is from before a flush. A flush that
    /// kept this chunk's play (`Ring::skip_to_tag`) is taken note of and the
    /// chunk plays on.
    fn stale(&mut self) -> bool {
        let flushes = self.ring.flushes();
        if flushes == self.flushes {
            return false;
        }
        if self.ring.kept_tag.load(Ordering::SeqCst) == self.tag {
            self.flushes = flushes;
            return false;
        }
        true
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
                    if let Some(eq) = &mut self.eq {
                        eq.refresh();
                    }
                    let stale = self.stale();
                    if stale || self.ring.held() {
                        // let the sound down over a few milliseconds of
                        // what was playing, then cut
                        if self.gain <= 0.0 {
                            self.count_rendered();
                            if stale {
                                self.idx = self.cur.len();
                                self.counted = self.idx;
                            } else {
                                self.silence = GAP * channels;
                            }
                            continue;
                        }
                        self.gain = (self.gain - self.step).max(0.0);
                    } else {
                        if self.gain < 1.0 {
                            self.gain = (self.gain + self.step).min(1.0);
                        }
                        if self.idx % (TICK * channels) == 0 {
                            self.note_heard();
                        }
                    }
                }
                let sample = match &mut self.eq {
                    Some(eq) => eq.process(self.idx % channels, self.cur[self.idx]),
                    None => self.cur[self.idx],
                } * self.gain;
                self.idx += 1;
                return Some(sample);
            }
            self.count_rendered();
            if self.ring.held() {
                // whatever comes after the pause comes up from silence
                self.gain = 0.0;
                self.silence = GAP * channels;
                continue;
            }
            let used = std::mem::take(&mut self.cur);
            self.hand_back(used);
            let refill = if self.ahead.len() < LOW {
                let starting = !self.flowing && self.waited < START_PATIENCE;
                self.ring.refill(self.rate, &mut self.ahead, &mut self.used, starting)
            } else {
                Refill::Got
            };
            // what was taken before a seek or a skip is not played, unless
            // the skip kept its play (`Ring::skip_to_tag`)
            let flushes = self.ring.flushes();
            let kept = self.ring.kept_tag.load(Ordering::SeqCst);
            let next = loop {
                match self.ahead.pop_front() {
                    Some((chunk, taken)) if taken != flushes && chunk.tag != kept => {
                        self.hand_back(chunk.samples);
                    }
                    next => break next,
                }
            };
            match next {
                Some((chunk, _)) => {
                    if flushes != self.flushes {
                        // the first sound after a seek or a skip
                        self.gain = 0.0;
                    }
                    self.flushes = flushes;
                    self.flowing = true;
                    self.waited = 0;
                    self.cur = chunk.samples;
                    self.idx = 0;
                    self.counted = 0;
                    self.tag = chunk.tag;
                    self.start_ms = chunk.start_ms;
                    self.end_ms = chunk.end_ms;
                    self.starving = false;
                    self.note_heard();
                }
                None => {
                    match refill {
                        Refill::Finished if !self.endless => return None,
                        Refill::Empty => {
                            // only a stream that was flowing can run dry;
                            // one still starting has simply not begun
                            if self.flowing && !self.starving && self.ring.streaming.load(Ordering::Relaxed) {
                                self.ring.underruns.fetch_add(1, Ordering::Relaxed);
                                self.starving = true;
                            }
                            self.flowing = false;
                        }
                        Refill::Starting => self.waited += 1,
                        // the lock was taken with nothing left here, or all
                        // that was here was stale: look again in a moment
                        _ => {}
                    }
                    self.gain = 0.0;
                    self.silence = GAP * channels;
                }
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
        self.count_rendered();
        let used = std::mem::take(&mut self.cur);
        self.hand_back(used);
        while let Some((chunk, _)) = self.ahead.pop_front() {
            self.hand_back(chunk.samples);
        }
        self.idx = 0;
        self.counted = 0;
        self.starving = false;
        Ok(())
    }
}

impl Drop for RingSource {
    fn drop(&mut self) {
        self.count_rendered();
        if !self.endless {
            self.ring.close();
            return;
        }
        // the output is going and the queue stays: what this one took and
        // did not play goes back for the next
        let channels = (self.channels as usize).max(1);
        let mut taken = Vec::with_capacity(self.ahead.len() + 1);
        if self.idx < self.cur.len() && self.idx % channels == 0 && !self.stale() {
            let played = self.idx as f64 / self.cur.len() as f64;
            taken.push((
                Chunk {
                    samples:  self.cur[self.idx..].to_vec(),
                    rate:     self.rate,
                    tag:      self.tag,
                    start_ms: self.start_ms + (self.end_ms - self.start_ms) * played,
                    end_ms:   self.end_ms,
                },
                self.flushes,
            ));
        }
        taken.extend(self.ahead.drain(..));
        if !taken.is_empty() {
            self.ring.put_back(taken);
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
        let (source, _ring) = prebuffer(Counter { next: 0, len: 10_000 }, Duration::ZERO, false, None);
        let mut source = source.without_fades();
        let out = drain(&mut source);
        assert_eq!(out.len(), 10_000);
        assert!(out.windows(2).all(|w| w[1] == w[0] + 1.0));
    }

    #[test]
    fn a_seek_lands_where_asked() {
        let (source, ring) = prebuffer(Counter { next: 0, len: 10_000 }, Duration::ZERO, false, None);
        let mut source = source.without_fades();
        source.next();
        source.try_seek(Duration::from_millis(3000)).unwrap();
        assert_eq!(ring.heard_ms(), 3000);
        let out = drain(&mut source);
        assert_eq!(out.first().copied(), Some(6001.0));
        assert_eq!(out.last().copied(), Some(10_000.0));
    }

    #[test]
    fn a_direct_seek_while_paused_drops_the_old_position() {
        let (source, ring) = prebuffer(Counter { next: 0, len: 10_000 }, Duration::ZERO, false, None);
        let mut source = source.without_fades();
        while source.next() == Some(0.0) {}
        // Controls take effect between stereo frames, never between channels.
        assert_eq!(source.next(), Some(2.0));
        ring.set_held(true);
        ring.request_seek(Duration::from_millis(3000));
        assert_eq!(ring.heard_ms(), 3000);
        assert!((0..100).all(|_| source.next() == Some(0.0)));
        ring.set_held(false);
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
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        ring.set_held(true);
        assert!((0..500).all(|_| source.next() == Some(0.0)));
        ring.set_held(false);
        let played = (0..1000).filter(|_| source.next() == Some(1.0)).count();
        assert_eq!(played, 200);
        assert!(ring.heard(7).is_some());
    }

    #[test]
    fn an_output_closed_while_paused_hands_back_what_it_had_not_played() {
        // the same pause and resume, once on one output and once with the
        // output closed and another opened in between: the same sound
        let listen = |reopen: bool| {
            let ring = Ring::new(2);
            for n in 0..6u32 {
                let start = f64::from(n) * 100.0;
                let chunk = Chunk {
                    samples:  vec![f32::from(n as u8 + 1); 200],
                    rate:     1000,
                    tag:      4,
                    start_ms: start,
                    end_ms:   start + 100.0,
                };
                assert!(ring.push(chunk, None, 10_000, &|| false));
            }
            let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
            let mut heard: Vec<Sample> = (0..100).filter_map(|_| source.next()).collect();
            ring.set_held(true);
            assert!((0..50).all(|_| source.next() == Some(0.0)));
            if reopen {
                drop(source);
                source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
            }
            ring.set_held(false);
            heard.extend((0..2000).filter_map(|_| source.next()).filter(|s| *s != 0.0));
            heard
        };
        let kept = listen(false);
        let reopened = listen(true);
        assert_eq!(reopened.len(), kept.len());
        assert_eq!(reopened, kept);
        assert_eq!(reopened.last().copied(), Some(6.0));
        assert!(reopened.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn audio_from_before_a_flush_is_not_handed_back() {
        let ring = Ring::new(2);
        let chunk = Chunk { samples: vec![1.0; 2000], rate: 1000, tag: 3, start_ms: 0.0, end_ms: 1000.0 };
        assert!(ring.push(chunk, None, 10_000, &|| false));
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        assert!((0..10).all(|_| source.next() == Some(1.0)));
        ring.flush_at(3, 60_000);
        drop(source);
        let mut next = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        assert!((0..500).all(|_| next.next() == Some(0.0)));
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
    fn a_flush_cuts_off_the_chunk_being_played() {
        let ring = Ring::new(2);
        let chunk = Chunk { samples: vec![1.0; 2000], rate: 1000, tag: 3, start_ms: 0.0, end_ms: 1000.0 };
        assert!(ring.push(chunk, None, 10_000, &|| false));
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        ring.set_streaming(true);
        assert!((0..10).all(|_| source.next() == Some(1.0)));
        ring.flush_at(3, 60_000);
        // not one more sample of what was playing, and the position holds at
        // the target while the producer gets there
        assert!((0..1000).all(|_| source.next() == Some(0.0)));
        assert_eq!(ring.heard(3), Some(60_000));
        assert_eq!(ring.underruns(), 0);
        let after = Chunk { samples: vec![2.0; 20], rate: 1000, tag: 3, start_ms: 60_000.0, end_ms: 60_010.0 };
        assert!(ring.push(after, None, 10_000, &|| false));
        assert!((0..100).any(|_| source.next() == Some(2.0)));
    }

    #[test]
    fn a_pause_fades_out_and_a_resume_fades_back_in() {
        let ring = Ring::new(2);
        let chunk = Chunk { samples: vec![1.0; 4000], rate: 1000, tag: 1, start_ms: 0.0, end_ms: 2000.0 };
        assert!(ring.push(chunk, None, 10_000, &|| false));
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000);
        assert!((0..10).all(|_| source.next() == Some(1.0)));
        ring.set_held(true);
        // 20ms at 1 kHz: twenty frames on the way down, each quieter
        let down: Vec<Sample> = (0..40).map(|_| source.next().unwrap()).collect();
        assert!(down.chunks(2).all(|frame| frame[0] == frame[1]));
        assert!(down.windows(3).step_by(2).all(|w| w[2] < w[0]));
        assert!(down[0] < 1.0 && down[38] == 0.0);
        assert!((0..200).all(|_| source.next() == Some(0.0)));
        ring.set_held(false);
        let mut up = Vec::new();
        while up.len() < 40 {
            let sample = source.next().unwrap();
            if sample != 0.0 || !up.is_empty() {
                up.push(sample);
            }
        }
        assert!(up[0] > 0.0 && up[0] < 1.0);
        assert!(up.windows(3).step_by(2).all(|w| w[2] >= w[0]));
        assert!((0..10).all(|_| source.next() == Some(1.0)));
    }

    #[test]
    fn skipping_a_tail_starts_the_next_play_and_loses_none_of_it() {
        let ring = Ring::new(2);
        let epoch = ring.epoch();
        let tail = Chunk { samples: vec![1.0; 400], rate: 1000, tag: 1, start_ms: 0.0, end_ms: 200.0 };
        let next = Chunk { samples: vec![2.0; 100], rate: 1000, tag: 2, start_ms: 0.0, end_ms: 50.0 };
        assert!(ring.push(tail, Some(epoch), 10_000, &|| false));
        assert!(ring.push(next, Some(epoch), 10_000, &|| false));
        assert_eq!(ring.queued_ms_of(2), 50);
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        assert!((0..10).all(|_| source.next() == Some(1.0)));
        ring.skip_to_tag(2);
        assert_eq!(ring.heard(2), Some(0));
        // the producer of the play that was kept is not turned away
        let more = Chunk { samples: vec![2.0; 100], rate: 1000, tag: 2, start_ms: 50.0, end_ms: 100.0 };
        assert!(ring.push(more, Some(epoch), 10_000, &|| false));
        let out: Vec<Sample> = (0..400).map(|_| source.next().unwrap()).filter(|s| *s != 0.0).collect();
        assert_eq!(out.len(), 200);
        assert!(out.iter().all(|s| *s == 2.0));
        // a chunk of the kept play that is already playing carries on
        let later = Chunk { samples: vec![3.0; 100], rate: 1000, tag: 2, start_ms: 100.0, end_ms: 150.0 };
        assert!(ring.push(later, Some(epoch), 10_000, &|| false));
        assert_eq!(source.by_ref().find(|s| *s != 0.0), Some(3.0));
        ring.skip_to_tag(2);
        assert!((0..99).all(|_| source.next() == Some(3.0)));
    }

    #[test]
    fn a_new_output_rate_converts_the_queue_instead_of_dropping_it() {
        let ring = Ring::new(2);
        let queued = Chunk { samples: vec![0.5; 2 * 44_100], rate: 44_100, tag: 1, start_ms: 0.0, end_ms: 1000.0 };
        assert!(ring.push(queued, None, 1_000_000, &|| false));
        ring.retime(48_000);
        assert_eq!(ring.queued_ms_of(1), 1000);
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 48_000);
        let played = (0..2 * 48_000).filter(|_| source.next().unwrap() != 0.0).count();
        // all of it but the converter's last few frames, at the new rate
        assert!(played > 2 * 47_800 && played <= 2 * 48_000, "played {played}");
    }

    #[test]
    fn the_device_thread_plays_on_while_the_ring_is_locked() {
        let ring = Ring::new(2);
        for at in 0..8u32 {
            let chunk = Chunk {
                samples: vec![(at + 1) as Sample; 20], rate: 1000, tag: 1,
                start_ms: at as f64 * 10.0, end_ms: (at + 1) as f64 * 10.0,
            };
            assert!(ring.push(chunk, None, 10_000, &|| false));
        }
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        assert_eq!(source.next(), Some(1.0));
        // someone holds the ring for as long as they like
        let held = ring.lock();
        let started = Instant::now();
        let played: Vec<Sample> = (0..400).map(|_| source.next().unwrap()).collect();
        assert!(started.elapsed() < Duration::from_secs(1), "the device thread waited for the lock");
        // everything it had taken plays, in order, and then silence - not a wait
        let heard: Vec<Sample> = played.iter().copied().filter(|s| *s != 0.0).collect();
        assert_eq!(heard.len(), 19 + 3 * 20);
        assert!(heard.windows(2).all(|w| w[1] >= w[0]));
        assert_eq!(ring.underruns(), 0);
        drop(held);
        // and it picks the rest up as soon as the lock is free
        let rest: Vec<Sample> = (0..400).map(|_| source.next().unwrap()).filter(|s| *s != 0.0).collect();
        assert_eq!(rest.len(), 4 * 20);
        assert_eq!(rest.first().copied(), Some(5.0));
        assert_eq!(rest.last().copied(), Some(8.0));
    }

    #[test]
    fn an_empty_ring_counts_one_underrun_mid_track() {
        let ring = Ring::new(2);
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000);
        ring.set_streaming(true);
        // nothing has played yet: a stream that has not started has not run dry
        for _ in 0..1000 {
            source.next();
        }
        assert_eq!(ring.underruns(), 0);
        let chunk = Chunk { samples: vec![1.0; 400], rate: 1000, tag: 1, start_ms: 0.0, end_ms: 200.0 };
        assert!(ring.push(chunk, None, 10_000, &|| false));
        assert_eq!(source.by_ref().take(2000).filter(|s| *s != 0.0).count(), 400);
        for _ in 0..1000 {
            source.next();
        }
        assert_eq!(ring.underruns(), 1);
    }

    #[test]
    fn a_starting_stream_waits_until_enough_is_queued() {
        let ring = Ring::new(2);
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        ring.set_streaming(true);
        // 100ms of the 150ms it wants: held back, and nothing is counted
        let first = Chunk { samples: vec![1.0; 200], rate: 1000, tag: 1, start_ms: 0.0, end_ms: 100.0 };
        assert!(ring.push(first, None, 10_000, &|| false));
        assert!((0..2000).all(|_| source.next() == Some(0.0)));
        assert_eq!(ring.underruns(), 0);
        let second = Chunk { samples: vec![1.0; 200], rate: 1000, tag: 1, start_ms: 100.0, end_ms: 200.0 };
        assert!(ring.push(second, None, 10_000, &|| false));
        assert_eq!(source.by_ref().take(2000).filter(|s| *s == 1.0).count(), 400);

        // the short end of a track is all there will be, and plays at once
        let ring = Ring::new(2);
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        let tail = Chunk { samples: vec![1.0; 40], rate: 1000, tag: 1, start_ms: 0.0, end_ms: 20.0 };
        assert!(ring.push(tail, None, 10_000, &|| false));
        assert_eq!(source.by_ref().take(200).filter(|s| *s == 1.0).count(), 40);

        // and a producer that never gets there is not waited on for ever
        let ring = Ring::new(2);
        let mut source = RingSource::endless(Arc::clone(&ring), 2, 1000).without_fades();
        ring.set_streaming(true);
        let little = Chunk { samples: vec![1.0; 40], rate: 1000, tag: 1, start_ms: 0.0, end_ms: 20.0 };
        assert!(ring.push(little, None, 10_000, &|| false));
        let waited = source.by_ref().take(200_000).take_while(|s| *s == 0.0).count();
        assert!(waited > 20_000 && waited < 100_000, "waited {waited} samples");
    }
}
