//! Audio output for local playback.
//!
//! librespot ships a rodio sink, but it opens the output device with
//! `.unwrap()` on the player thread, and the release profile aborts on any
//! panic. A Windows PC with no default playback device (nothing in the jack,
//! a Bluetooth headset that is off, a remote desktop session) takes
//! the whole app down the moment playback is authorized. This sink opens the
//! device only when playback starts, reports failures gracefully, resamples
//! cleanly to the device's native rate, and handles thread priority.
//!
//! Decoded audio goes into a `playout::Ring` that runs `playout::AHEAD` in
//! front of the speaker, so the player thread can be held up (a busy machine,
//! a slow read) for seconds without a dropout. The ring is flushed at every
//! seek and every track the user picks, pause holds it rather than draining
//! it, and the natural end of a track lets its tail play into the next one.
//! Which of those is happening is read off the player's own events, which
//! arrive in step with the audio.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait};
use librespot_playback::audio_backend::{Sink, SinkError, SinkResult};
use librespot_playback::convert::Converter;
use librespot_playback::decoder::AudioPacket;
use librespot_playback::mixer::VolumeGetter;
use librespot_playback::player::{PlayerEvent, PlayerEventChannel};
use librespot_playback::{NUM_CHANNELS, SAMPLE_RATE};

use crate::playout::{self, Chunk, Ring, RingSource};
use crate::resample::Resampler;

/// Told about output failures, with a message fit for the interface.
pub type ErrorHook = Arc<dyn Fn(String) + Send + Sync>;

/// Where the player's events are handed to the sink. The sink is built on the
/// player's own thread before there is a player to subscribe to, so the
/// channel is put here afterwards and picked up on first use.
pub type EventSlot = Arc<Mutex<Option<PlayerEventChannel>>>;

/// The librespot output's queue, and what the rest of playback needs to know
/// about it.
pub struct Playout {
    pub ring:   Arc<Ring>,
    /// Loads the user asked for. One made after a track finished decoding
    /// means its tail is not wanted.
    user_loads: AtomicU64,
    /// The end of the last track has been reported to the interface, so the
    /// next load is most likely the queue moving on, not the user.
    told_end:   AtomicBool,
    /// The play request the player is on.
    current:    AtomicU64,
}

impl Playout {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            ring:       Ring::new(NUM_CHANNELS as usize),
            user_loads: AtomicU64::new(0),
            told_end:   AtomicBool::new(false),
            current:    AtomicU64::new(u64::MAX),
        })
    }

    /// A load is coming. Unless it is the queue moving on after the end of
    /// a track was reported, it's the user's, and what is queued goes.
    pub fn loading(&self) {
        if !self.told_end.swap(false, Ordering::SeqCst) {
            self.user_loads.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// The user skipped, jumped or loaded something; what is queued goes.
    pub fn user_load(&self) {
        self.told_end.store(false, Ordering::SeqCst);
        self.user_loads.fetch_add(1, Ordering::SeqCst);
    }

    pub fn user_loads(&self) -> u64 {
        self.user_loads.load(Ordering::SeqCst)
    }

    pub fn told_end(&self) {
        self.told_end.store(true, Ordering::SeqCst);
    }

    /// Where the listener is in the track the player is on, once any of it
    /// has been heard.
    pub fn heard_now(&self) -> Option<u32> {
        self.ring.heard(self.current.load(Ordering::Relaxed))
    }
}

/// How often playback looks at which output the system calls its default.
const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// How much sound the audio engine is asked to hold for the device, in milliseconds.
pub const DEFAULT_BUFFER_MS: u32 = 100;

pub const BUFFER_MS_RANGE: std::ops::RangeInclusive<u32> = 20..=500;

fn engine_buffer(
    sample_rate: u32,
    ms: u32,
    supported: cpal::SupportedBufferSize,
) -> cpal::BufferSize {
    let ms = ms.clamp(*BUFFER_MS_RANGE.start(), *BUFFER_MS_RANGE.end());
    let frames = (u64::from(sample_rate) * u64::from(ms) / 1000).max(1) as u32;
    match supported {
        cpal::SupportedBufferSize::Range { min, max } if min <= max && max > 0 => {
            cpal::BufferSize::Fixed(frames.clamp(min.max(1), max))
        }
        _ => cpal::BufferSize::Fixed(frames),
    }
}

/// Computes output latency in milliseconds from frames queued in the sink,
/// the output sample rate, and the granted device buffer duration.
///
/// Returns 0 if `sample_rate` is 0 and no device buffer duration is provided.
pub fn calculate_latency_ms(
    queued_frames: usize,
    sample_rate: u32,
    device_buffer_ms: Option<u32>,
) -> i64 {
    let sink_ms = if sample_rate > 0 {
        (queued_frames as u64 * 1000) / sample_rate as u64
    } else {
        0
    };
    let device_ms = device_buffer_ms.unwrap_or(0) as u64;
    (sink_ms + device_ms) as i64
}

/// Convenience alias for `calculate_latency_ms`.
pub fn latency_ms(
    queued_frames: usize,
    sample_rate: u32,
    device_buffer_ms: Option<u32>,
) -> i64 {
    calculate_latency_ms(queued_frames, sample_rate, device_buffer_ms)
}

pub struct RodioSink {
    /// The output device name; `None` means the default.
    device: Option<String>,
    output: Option<Output>,
    on_error: ErrorHook,
    /// The player's volume, applied here at the output so a change is heard
    /// at once instead of after the queue drains.
    volume: Box<dyn VolumeGetter + Send>,
    applied_volume: f32,
    /// Keeps asking which output the system calls its default.
    watch: Option<DefaultWatch>,
    watch_version: u64,
    buffer_ms: u32,
    /// Output latency in milliseconds past the point positions are reported
    /// from: the device buffer. The queue in front of it is already accounted
    /// for, since positions are what is heard (see `Playout`).
    latency: Arc<AtomicI64>,
    playout: Arc<Playout>,
    event_slot: EventSlot,
    events: Option<PlayerEventChannel>,
    track: Track,
    /// Underruns already logged.
    underruns_seen: u64,
}

/// Where the player is, as far as the audio reaching the sink goes.
struct Track {
    /// The play request the audio belongs to.
    tag: u64,
    /// Where in the track the next packet starts, in milliseconds.
    pos_ms: f64,
    /// Decoded to the end, and nothing since: the queue is that track's tail.
    at_end: bool,
    /// `Playout::user_loads` when it got there.
    loads_at_end: u64,
    /// A new play request with none of its audio written yet.
    fresh: bool,
}

struct Output {
    sink: rodio::Sink,
    _stream: rodio::OutputStream,
    /// The name of the device the stream was opened on.
    device_name: Option<String>,
    /// Set from the audio thread when the stream dies (device unplugged).
    failed: Arc<AtomicBool>,
    /// The rate the stream runs at, and the converter to it when that is
    /// not Spotify's.
    sample_rate: u32,
    resampler: Option<Resampler>,
    /// pitch-preserving speed change (podcasts at 1.5x etc). None at 1x
    stretcher: Option<crate::stretch::TimeStretch>,
}

impl Output {
    fn failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }
}

impl RodioSink {
    pub fn new(
        device: Option<String>,
        on_error: ErrorHook,
        volume: Box<dyn VolumeGetter + Send>,
        buffer_ms: u32,
        playout: Arc<Playout>,
        event_slot: EventSlot,
    ) -> Self {
        Self {
            device,
            output: None,
            on_error,
            volume,
            applied_volume: -1.0,
            watch: None,
            watch_version: 0,
            buffer_ms,
            latency: Arc::new(AtomicI64::new(0)),
            playout,
            event_slot,
            events: None,
            track: Track { tag: u64::MAX, pos_ms: 0.0, at_end: false, loads_at_end: 0, fresh: false },
            underruns_seen: 0,
        }
    }

    /// Attaches an external atomic tracker so callers can observe output
    /// latency updates without locking the sink.
    pub fn with_latency_tracker(mut self, latency: Arc<AtomicI64>) -> Self {
        self.latency = latency;
        self
    }

    /// Returns a handle to the latency tracker atomic.
    pub fn latency_tracker(&self) -> Arc<AtomicI64> {
        Arc::clone(&self.latency)
    }

    /// Current output latency in milliseconds.
    pub fn output_latency_ms(&self) -> i64 {
        self.latency.load(Ordering::Relaxed)
    }

    /// Catch up on what the player has done since the sink last ran.
    ///
    /// The player sends its events from the thread that writes the audio,
    /// before the audio that follows them, so what is read here lines up
    /// exactly with the packets on either side.
    fn catch_up(&mut self) {
        if self.events.is_none() {
            self.events = self.event_slot.lock().unwrap_or_else(PoisonError::into_inner).take();
        }
        let Some(events) = self.events.as_mut() else {
            return;
        };
        let mut flush = false;
        while let Ok(event) = events.try_recv() {
            match event {
                PlayerEvent::PlayRequestIdChanged { play_request_id } => {
                    // a track that played to its end rolls straight on into
                    // the next; anything the user picked starts right away
                    let rolling_on = self.track.at_end
                        && self.playout.user_loads() == self.track.loads_at_end;
                    flush |= !rolling_on;
                    self.track.tag = play_request_id;
                    self.track.pos_ms = 0.0;
                    self.track.at_end = false;
                    self.track.fresh = true;
                    self.playout.current.store(play_request_id, Ordering::Relaxed);
                }
                PlayerEvent::Seeked { position_ms, .. } => {
                    flush = true;
                    self.track.pos_ms = position_ms as f64;
                }
                // where a load starts. (on a resume these carry the start of
                // the last packet already written, so they're left alone)
                PlayerEvent::Loading { position_ms, .. }
                | PlayerEvent::Playing { position_ms, .. }
                | PlayerEvent::Paused { position_ms, .. }
                    if self.track.fresh =>
                {
                    self.track.pos_ms = position_ms as f64;
                }
                // sent just before the packet they describe
                PlayerEvent::PositionCorrection { position_ms, .. }
                | PlayerEvent::PositionChanged { position_ms, .. } => {
                    self.track.pos_ms = position_ms as f64;
                }
                PlayerEvent::EndOfTrack { .. } => {
                    self.track.at_end = true;
                    self.track.loads_at_end = self.playout.user_loads();
                    self.playout.ring.set_streaming(false);
                }
                _ => {}
            }
        }
        if flush {
            self.playout.ring.flush();
            self.playout.ring.set_heard(self.track.tag, self.track.pos_ms as u64);
            if let Some(output) = &mut self.output {
                output.stretcher = None;
            }
        }
    }

    fn report_underruns(&mut self) {
        let underruns = self.playout.ring.underruns();
        if underruns != self.underruns_seen {
            log::warn!(
                "the audio ran dry {} time(s) mid-track: the player fell more than {}s behind",
                underruns - self.underruns_seen,
                playout::AHEAD.as_secs()
            );
            self.underruns_seen = underruns;
        }
    }

    fn follow_default(&mut self, at_once: bool) {
        if cfg!(target_os = "linux") || self.device.is_some() {
            return;
        }
        let Some(output) = &self.output else {
            return;
        };
        let watch = self.watch.get_or_insert_with(DefaultWatch::start);
        if !at_once && watch.version() == self.watch_version {
            return;
        }
        let (version, current) = if at_once { watch.ask() } else { watch.name() };
        self.watch_version = version;
        if current.is_some() && current != output.device_name {
            log::info!(
                "the default audio output is now {}; moving playback to it",
                current.as_deref().unwrap_or("[unknown device]")
            );
            self.output = None;
        }
    }

    fn apply_volume(&mut self) {
        let factor = self.volume.attenuation_factor() as f32;
        if let Some(output) = &self.output {
            if (factor - self.applied_volume).abs() > 0.001 {
                output.sink.set_volume(factor);
                self.applied_volume = factor;
            }
        }
    }

    /// Opens the output if it is not open, or if it died since.
    fn ensure_open(&mut self) -> SinkResult<()> {
        if self.output.as_ref().is_some_and(Output::failed) {
            log::warn!("the audio output stopped working; reopening it");
            self.output = None;
        }
        if self.output.is_some() {
            return Ok(());
        }
        match open_output(self.device.as_deref(), self.buffer_ms, &self.playout.ring) {
            Ok((output, device_buffer_ms)) => {
                self.latency.store(i64::from(device_buffer_ms), Ordering::Relaxed);
                self.output = Some(output);
                // Recheck a new stream even if the default name has not changed.
                self.watch_version = 0;
                self.applied_volume = -1.0;
                Ok(())
            }
            Err(error) => {
                let message = error.to_string();
                log::error!("{message}");
                (self.on_error)(message.clone());
                Err(SinkError::ConnectionRefused(message))
            }
        }
    }
}

impl Sink for RodioSink {
    fn start(&mut self) -> SinkResult<()> {
        playout::boost_current_thread();
        self.catch_up();
        self.follow_default(true);
        self.ensure_open()?;
        self.apply_volume();
        self.playout.ring.set_held(false);
        Ok(())
    }

    fn stop(&mut self) -> SinkResult<()> {
        self.catch_up();
        // Spotify Connect stops the player the moment the last track finishes
        // decoding, seconds before it finishes playing. Holding then would cut
        // off the end of the song.
        if !self.track.at_end {
            self.playout.ring.set_held(true);
        }
        Ok(())
    }

    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let samples = packet
            .samples()
            .map_err(|error| SinkError::OnWrite(error.to_string()))?;
        let samples = converter.f64_to_f32(samples);
        self.catch_up();
        self.follow_default(false);
        self.ensure_open()?;
        self.apply_volume();
        self.report_underruns();
        let Some(output) = &mut self.output else {
            return Err(SinkError::NotConnected(
                "the audio output is not open".into(),
            ));
        };

        // where in the track this packet sits, from Spotify's 44.1 kHz frames
        // (before the speed change: the position is the track's, not the clock's)
        let start_ms = self.track.pos_ms;
        let frames = samples.len() / NUM_CHANNELS as usize;
        self.track.pos_ms += frames as f64 * 1000.0 / SAMPLE_RATE as f64;
        self.track.at_end = false;
        self.track.fresh = false;
        self.playout.ring.set_streaming(true);

        let samples = match &mut output.resampler {
            Some(resampler) => resampler.process(&samples),
            None => samples,
        };
        // playback speed: rebuild the stretcher when the speed changes, drop it
        // at 1x so normal playback is untouched
        let speed = crate::stretch::speed() as f64;
        if (speed - 1.0).abs() < 1e-3 {
            output.stretcher = None;
        } else if output.stretcher.as_ref().map(|s| (s.speed() - speed).abs() > 1e-3).unwrap_or(true) {
            output.stretcher = Some(crate::stretch::TimeStretch::new(speed));
        }
        let samples = match &mut output.stretcher {
            Some(stretcher) => stretcher.process(&samples),
            None => samples,
        };
        if samples.is_empty() {
            // the stretcher is still filling its first frame
            return Ok(());
        }

        let chunk = Chunk {
            samples,
            rate: output.sample_rate,
            tag: self.track.tag,
            start_ms,
            end_ms: self.track.pos_ms,
        };
        // Blocks while the queue is full, which is what paces the decoder.
        let limit = playout::frames_in(playout::AHEAD, output.sample_rate);
        let failed = Arc::clone(&output.failed);
        let queued = self
            .playout
            .ring
            .push(chunk, None, limit, &|| failed.load(Ordering::Relaxed));
        if !queued && failed.load(Ordering::Relaxed) {
            let message = "The audio output stopped working".to_string();
            (self.on_error)(message.clone());
            return Err(SinkError::OnWrite(message));
        }
        Ok(())
    }
}

/// Opens the default output at its own rate, with a device buffer of about
/// `buffer_ms`: the YouTube backend's output. Falls back to whatever rodio
/// can get when that is refused.
pub fn open_default_output(buffer_ms: u32) -> Result<rodio::OutputStream, rodio::StreamError> {
    let device = cpal::default_host().default_output_device();
    let config = device.as_ref().and_then(|device| device.default_output_config().ok());
    if let (Some(device), Some(config)) = (device, config) {
        let buffer = engine_buffer(config.sample_rate().0, buffer_ms, *config.buffer_size());
        let opened = rodio::OutputStreamBuilder::from_device(device)
            .map(|builder| builder.with_buffer_size(buffer))
            .and_then(|builder| builder.open_stream());
        match opened {
            Ok(stream) => return Ok(stream),
            Err(error) => log::warn!("cannot open the output with a {buffer_ms}ms buffer ({error}); using its default"),
        }
    }
    rodio::OutputStreamBuilder::open_default_stream()
}

fn open_stream(
    device: &cpal::Device,
    on_error: impl FnMut(cpal::StreamError) + Send + Clone + 'static,
    buffer_ms: u32,
) -> Result<rodio::OutputStream, rodio::StreamError> {
    let supported = device
        .default_output_config()
        .map(|config| *config.buffer_size())
        .unwrap_or(cpal::SupportedBufferSize::Unknown);
    let builder = |sample_rate: u32, buffer: bool| -> Result<_, rodio::StreamError> {
        let builder = rodio::OutputStreamBuilder::from_device(device.clone())?
            .with_channels(NUM_CHANNELS as rodio::ChannelCount)
            .with_sample_rate(sample_rate as rodio::SampleRate)
            .with_error_callback(on_error.clone());
        Ok(if buffer {
            builder.with_buffer_size(engine_buffer(sample_rate, buffer_ms, supported))
        } else {
            builder
        })
    };
    if let Ok(stream) = builder(SAMPLE_RATE, true)?.open_stream() {
        return Ok(stream);
    }
    if let Ok(config) = device.default_output_config() {
        if let Ok(stream) = builder(config.sample_rate().0, true)?.open_stream() {
            return Ok(stream);
        }
    }
    builder(SAMPLE_RATE, false)?.open_stream_or_fallback()
}

struct DefaultWatch(Arc<DefaultName>);

struct DefaultName {
    state: Mutex<(u64, Option<String>)>,
    version: AtomicU64,
}

impl DefaultName {
    fn new() -> Self {
        Self { state: Mutex::new((1, None)), version: AtomicU64::new(1) }
    }

    fn update(&self, name: Option<String>) -> (u64, Option<String>) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if state.1 != name {
            state.0 += 1;
            state.1 = name.clone();
            self.version.store(state.0, Ordering::Release);
        }
        (state.0, name)
    }

    fn snapshot(&self) -> (u64, Option<String>) {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        (state.0, state.1.clone())
    }
}

impl DefaultWatch {
    fn start() -> Self {
        let shared = Arc::new(DefaultName::new());
        let weak = Arc::downgrade(&shared);
        let watching = thread::Builder::new()
            .name("audio-default-watch".into())
            .spawn(move || {
                while let Some(shared) = weak.upgrade() {
                    shared.update(default_output_name());
                    // The watcher must not keep the sink alive while sleeping.
                    drop(shared);
                    thread::sleep(DEFAULT_CHECK_INTERVAL);
                }
            });
        if let Err(error) = watching {
            log::warn!("cannot watch the default audio output: {error}");
        }
        Self(shared)
    }

    fn version(&self) -> u64 {
        self.0.version.load(Ordering::Acquire)
    }

    fn name(&self) -> (u64, Option<String>) {
        self.0.snapshot()
    }

    fn ask(&self) -> (u64, Option<String>) {
        self.0.update(default_output_name())
    }
}

fn default_output_name() -> Option<String> {
    cpal::default_host()
        .default_output_device()
        .and_then(|device| device.name().ok())
}

#[derive(Debug, thiserror::Error)]
enum OpenError {
    #[error("No audio output device was found. Connect or enable one, then press play again.")]
    NoDevice,
    #[error("Cannot list the audio devices: {0}")]
    Devices(#[from] cpal::DevicesError),
    #[error("Cannot open the audio output: {0}")]
    Stream(#[from] rodio::StreamError),
}

/// Opens an output playing from `ring`, and says how much the device buffers
/// in milliseconds.
fn open_output(
    preferred: Option<&str>,
    buffer_ms: u32,
    ring: &Arc<Ring>,
) -> Result<(Output, u32), OpenError> {
    let host = cpal::default_host();
    let device = match preferred.map(str::trim).filter(|name| !name.is_empty()) {
        Some(name) => {
            let chosen = host
                .output_devices()?
                .find(|device| device.name().is_ok_and(|found| found == name));
            match chosen {
                Some(device) => device,
                None => {
                    log::warn!("audio device {name:?} is not available; using the default");
                    host.default_output_device().ok_or(OpenError::NoDevice)?
                }
            }
        }
        None => host.default_output_device().ok_or(OpenError::NoDevice)?,
    };
    let device_name = device.name().ok();
    log::info!(
        "audio output: {}",
        device_name.as_deref().unwrap_or("[unknown device]")
    );

    let failed = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&failed);
    let on_error = move |error: cpal::StreamError| {
        log::error!("audio stream error: {error}");
        flag.store(true, Ordering::Relaxed);
    };
    let mut stream = open_stream(&device, on_error, buffer_ms)?;
    stream.log_on_drop(false);
    let sample_rate = stream.config().sample_rate();

    // read the actual negotiated buffer size from the output stream.
    // cpal clamped the requested buffer_ms against the device's hardware
    // supported range inside engine_buffer(); on many audio interfaces
    // the granted buffer size is constrained to hardware periods and
    // differs from the requested 100ms. If cpal reports BufferSize::Default
    // (e.g. on fallback or backends that do not expose buffer size), we
    // cannot query the driver's true buffer size from cpal once running,
    // so fall back to the configured buffer_ms.
    let device_buffer_ms = match stream.config().buffer_size() {
        cpal::BufferSize::Fixed(frames) if sample_rate > 0 => {
            ((u64::from(*frames) * 1000 + u64::from(sample_rate) / 2) / u64::from(sample_rate)) as u32
        }
        _ => buffer_ms,
    };

    let resampler = Resampler::new(SAMPLE_RATE, sample_rate, NUM_CHANNELS as usize);
    if resampler.is_some() {
        log::info!(
            "the output runs at {sample_rate} Hz; the music is converted from {SAMPLE_RATE} Hz"
        );
    }
    let sink = rodio::Sink::connect_new(stream.mixer());
    sink.append(RingSource::endless(
        Arc::clone(ring),
        NUM_CHANNELS as rodio::ChannelCount,
        sample_rate,
    ));
    sink.play();
    let output = Output {
        sink,
        _stream: stream,
        device_name,
        failed,
        sample_rate,
        resampler,
        stretcher: None,
    };
    Ok((output, device_buffer_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_arithmetic_standard() {
        // 4410 frames at 44.1 kHz is exactly 100ms in the sink queue.
        // With a 100ms device buffer, total latency is 200ms.
        let latency = calculate_latency_ms(4410, 44100, Some(100));
        assert_eq!(latency, 200);
    }

    #[test]
    fn test_latency_arithmetic_48k() {
        // 4800 frames at 48 kHz is 100ms. With a 42ms hardware buffer,
        // total latency is 142ms.
        let latency = calculate_latency_ms(4800, 48000, Some(42));
        assert_eq!(latency, 142);
    }

    #[test]
    fn test_latency_arithmetic_zero_frames() {
        // Queue empty, but device buffer still holds audio.
        let latency = calculate_latency_ms(0, 48000, Some(50));
        assert_eq!(latency, 50);
    }

    #[test]
    fn test_latency_arithmetic_none_buffer() {
        // Device buffer unknown / None, only queued frames contribute.
        let latency = calculate_latency_ms(4410, 44100, None);
        assert_eq!(latency, 100);
    }

    #[test]
    fn test_latency_arithmetic_zero_and_none() {
        // Zero frames, valid rate, None buffer -> 0ms.
        let latency = calculate_latency_ms(0, 44100, None);
        assert_eq!(latency, 0);

        // Zero frames, zero rate, None buffer -> 0ms (no divide-by-zero).
        let latency = calculate_latency_ms(0, 0, None);
        assert_eq!(latency, 0);

        // Zero rate with device buffer -> returns device buffer safely.
        let latency = calculate_latency_ms(0, 0, Some(100));
        assert_eq!(latency, 100);
    }

    #[test]
    fn test_latency_ms_alias() {
        assert_eq!(latency_ms(2205, 44100, Some(50)), 100);
    }

    #[test]
    fn unchanged_output_does_not_require_another_name_snapshot() {
        let watch = DefaultWatch(Arc::new(DefaultName::new()));
        assert_eq!(watch.version(), 1);
        watch.0.update(Some("Speakers".into()));
        assert_eq!(watch.version(), 2);
        watch.0.update(Some("Speakers".into()));
        assert_eq!(watch.version(), 2);
        watch.0.update(Some("Headphones".into()));
        assert_eq!(watch.name(), (3, Some("Headphones".into())));
        watch.0.update(None);
        assert_eq!(watch.name(), (4, None));
    }

    #[test]
    fn output_name_and_version_are_one_snapshot_during_updates() {
        let state = Arc::new(DefaultName::new());
        let writer = state.clone();
        let updates = thread::spawn(move || {
            for version in 2..10_000 {
                writer.update(Some(version.to_string()));
            }
        });
        while !updates.is_finished() {
            let (version, name) = state.snapshot();
            if let Some(name) = name {
                assert_eq!(name.parse::<u64>().unwrap(), version);
            }
        }
        updates.join().unwrap();
        assert_eq!(state.snapshot(), (9_999, Some("9999".into())));
    }
}
