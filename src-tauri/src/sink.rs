//! Audio output for local playback.
//!
//! librespot ships a rodio sink, but it opens the output device with
//! `.unwrap()` on the player thread, and the release profile aborts on any
//! panic. A Windows PC with no default playback device (nothing in the jack,
//! a Bluetooth headset that is off, a remote desktop session) takes
//! the whole app down the moment playback is authorized. This sink opens the
//! device only when playback starts, reports failures gracefully, resamples
//! cleanly to the device's native rate, and handles thread priority.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait};
use librespot_playback::audio_backend::{Sink, SinkError, SinkResult};
use librespot_playback::convert::Converter;
use librespot_playback::decoder::AudioPacket;
use librespot_playback::mixer::VolumeGetter;
use librespot_playback::{NUM_CHANNELS, SAMPLE_RATE};

use crate::resample::Resampler;

/// Told about output failures, with a message fit for the interface.
pub type ErrorHook = Arc<dyn Fn(String) + Send + Sync>;

/// How many chunks may wait in rodio's queue before `write` blocks. Chunks
/// run from a few hundred to a few thousand samples; this is about a fifth
/// of a second, which is also how long a pause takes to be heard.
const QUEUE_LIMIT: usize = 12;

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
    buffer_ms: u32,
    /// Total output latency in milliseconds (queued frames in sink + device buffer).
    /// Updated on every audio append so reads on a timer are cheap atomic loads.
    latency: Arc<AtomicI64>,
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
    /// Duration of the audio device buffer in milliseconds granted by cpal / engine_buffer.
    device_buffer_ms: u32,
    /// Ring buffer of frame counts for recently appended chunks.
    recent_chunk_frames: [u32; 32],
    recent_chunk_idx: usize,
    recent_chunk_count: usize,
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
    ) -> Self {
        Self {
            device,
            output: None,
            on_error,
            volume,
            applied_volume: -1.0,
            watch: None,
            buffer_ms,
            latency: Arc::new(AtomicI64::new(0)),
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

    fn follow_default(&mut self, at_once: bool) {
        if cfg!(target_os = "linux") || self.device.is_some() {
            return;
        }
        let Some(output) = &self.output else {
            return;
        };
        let watch = self.watch.get_or_insert_with(DefaultWatch::start);
        let current = if at_once { watch.ask() } else { watch.name() };
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
        match open_output(self.device.as_deref(), self.buffer_ms) {
            Ok(output) => {
                self.output = Some(output);
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
        take_precedence();
        self.follow_default(true);
        self.ensure_open()?;
        self.apply_volume();
        if let Some(output) = &self.output {
            output.sink.play();
        }
        Ok(())
    }

    fn stop(&mut self) -> SinkResult<()> {
        if let Some(output) = &self.output {
            output.sink.pause();
        }
        Ok(())
    }

    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let samples = packet
            .samples()
            .map_err(|error| SinkError::OnWrite(error.to_string()))?;
        let samples = converter.f64_to_f32(samples);
        self.follow_default(false);
        self.ensure_open()?;
        self.apply_volume();
        let Some(output) = &mut self.output else {
            return Err(SinkError::NotConnected(
                "the audio output is not open".into(),
            ));
        };
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
        // track the frame count of this chunk before appending to rodio
        let chunk_frames = (samples.len() / NUM_CHANNELS as usize) as u32;
        let slot = output.recent_chunk_idx;
        output.recent_chunk_frames[slot] = chunk_frames;
        output.recent_chunk_idx = (slot + 1) % 32;
        if output.recent_chunk_count < 32 {
            output.recent_chunk_count += 1;
        }

        output.sink.append(rodio::buffer::SamplesBuffer::new(
            NUM_CHANNELS as rodio::ChannelCount,
            output.sample_rate as rodio::SampleRate,
            samples,
        ));
        // Let rodio drain a little; without this the whole track would be decoded into memory at once.
        while output.sink.len() > QUEUE_LIMIT {
            if output.failed() {
                let message = "The audio output stopped working".to_string();
                (self.on_error)(message.clone());
                return Err(SinkError::OnWrite(message));
            }
            thread::sleep(Duration::from_millis(10));
        }

        // compute current output latency: rodio's queued frames plus the
        // device buffer duration. rodio::Sink::len() tells us how many
        // sources remain in the FIFO queue; summing the last `len` entries
        // gives the honest frame count without guessing.
        let queued_sources = output.sink.len();
        let queued_frames = if queued_sources == 0 {
            0
        } else {
            let n = queued_sources.min(output.recent_chunk_count);
            let mut sum: u64 = 0;
            for i in 0..n {
                let idx = (output.recent_chunk_idx + 32 - 1 - i) % 32;
                sum += output.recent_chunk_frames[idx] as u64;
            }
            if queued_sources > n && n > 0 {
                let avg = sum / n as u64;
                sum += (queued_sources - n) as u64 * avg;
            }
            sum as usize
        };

        let latency_ms = calculate_latency_ms(
            queued_frames,
            output.sample_rate,
            Some(output.device_buffer_ms),
        );
        self.latency.store(latency_ms, Ordering::Relaxed);

        Ok(())
    }
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

#[cfg(windows)]
fn take_precedence() {
    use windows_sys::Win32::System::Threading::{
        GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_ABOVE_NORMAL,
    };
    unsafe {
        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL);
    }
}

#[cfg(not(windows))]
fn take_precedence() {}

struct DefaultWatch(Arc<Mutex<Option<String>>>);

impl DefaultWatch {
    fn start() -> Self {
        let shared = Arc::new(Mutex::new(None));
        let weak = Arc::downgrade(&shared);
        let watching = thread::Builder::new()
            .name("audio-default-watch".into())
            .spawn(move || {
                while let Some(shared) = weak.upgrade() {
                    let name = default_output_name();
                    *shared.lock().unwrap_or_else(PoisonError::into_inner) = name;
                    drop(shared);
                    thread::sleep(DEFAULT_CHECK_INTERVAL);
                }
            });
        if let Err(error) = watching {
            log::warn!("cannot watch the default audio output: {error}");
        }
        Self(shared)
    }

    fn name(&self) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn ask(&self) -> Option<String> {
        let name = default_output_name();
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = name.clone();
        name
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

fn open_output(preferred: Option<&str>, buffer_ms: u32) -> Result<Output, OpenError> {
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
    Ok(Output {
        sink,
        _stream: stream,
        device_name,
        failed,
        sample_rate,
        resampler,
        stretcher: None,
        device_buffer_ms,
        recent_chunk_frames: [0; 32],
        recent_chunk_idx: 0,
        recent_chunk_count: 0,
    })
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
}
