//! A spectrum analyser and oscilloscope of what is being heard.
//!
//! The tap sits where the sound leaves for the device (`playout::RingSource`),
//! after the equalizer and before the volume, so the picture follows the
//! speaker rather than the queue four seconds ahead of it, and a quiet volume
//! does not flatten it. It costs nothing until something asks for frames:
//! the device thread checks one flag per sample, and when it is set gathers a
//! mono mix a block at a time and hands it over only if the lock is free.
//! The picture can miss a block; the sound never waits for the picture.
//!
//! The analysis is Winamp's classic analyser: a 512-sample FFT under a Hann
//! window, seventy-five semitone-spaced bands, bars that jump up and fall at a
//! fixed rate, and peak caps that hang and then drop.
//!
//! Adapted from Spotifast (https://github.com/crmne/spotifast),
//! Copyright (c) 2026 Carmine Paolino, under the MIT License (LICENSE-spotifast).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

/// Samples that go into one spectrum.
pub const FFT_SAMPLES: usize = 512;
const SPECTRUM_BINS: usize = FFT_SAMPLES / 2;
/// The analyser's bands, a semitone apart.
pub const BANDS: usize = 75;
/// The scope reads every seventh sample.
pub const SCOPE_STEP: usize = 7;
/// The most scope points asked for at once.
const MAX_POINTS: usize = 512;
/// What the tap keeps: comfortably more than one spectrum or scope.
const KEPT: usize = 8192;
/// Mono samples gathered on the device thread before one is handed over.
const BLOCK: usize = 256;
/// The tap counts as silent once nothing has arrived for this long (paused,
/// stopped, or another device playing).
const QUIET_AFTER: Duration = Duration::from_millis(120);
/// The tallest a band gets, in Winamp's rows.
const MAX_HEIGHT: f32 = 15.0;
/// How far a bar falls each sixtieth of a second, in rows.
const FALLOFF: f32 = 12.0 / 16.0;
/// Converts the tap's channel mean to Winamp's channel sum.
const CHANNEL_SUM: f32 = 2.0;
/// Winamp's own scale on every magnitude.
const SPEC_SCALE: f32 = 0.5;
/// How fast a band falls, in heights per second.
const FALL: f32 = FALLOFF / MAX_HEIGHT / 0.016_667;
/// How long a peak cap hangs before it drops, and how fast it then falls.
const PEAK_HOLD: f32 = 0.35;
const PEAK_GRAVITY: f32 = 2.4;
/// How often frames are sent while someone is watching.
const FRAME_EVERY: Duration = Duration::from_micros(16_667);

/// Set while frames are wanted. Read by the device thread on every sample.
static ON: AtomicBool = AtomicBool::new(false);

struct Tap {
    samples: VecDeque<f32>,
    last:    Option<Instant>,
}

static TAP: LazyLock<Mutex<Tap>> =
    LazyLock::new(|| Mutex::new(Tap { samples: VecDeque::with_capacity(KEPT), last: None }));

#[inline]
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Gathers the mono mix on the device thread. One lives in each output.
pub struct Collector {
    block: [f32; BLOCK],
    len:   usize,
    frame: f32,
}

impl Default for Collector {
    fn default() -> Self {
        Self { block: [0.0; BLOCK], len: 0, frame: 0.0 }
    }
}

impl Collector {
    /// One sample of an interleaved frame, as it goes to the device.
    #[inline]
    pub fn add(&mut self, channel: usize, channels: usize, sample: f32) {
        self.frame += sample;
        if channel + 1 < channels {
            return;
        }
        self.block[self.len] = self.frame / channels.max(1) as f32;
        self.frame = 0.0;
        self.len += 1;
        if self.len == BLOCK {
            self.hand_over();
        }
    }

    /// Hands the block over if nothing is reading the tap; otherwise drops it.
    fn hand_over(&mut self) {
        if let Ok(mut tap) = TAP.try_lock() {
            let kept = tap.samples.len();
            let excess = (kept + self.len).saturating_sub(KEPT).min(kept);
            tap.samples.drain(..excess);
            tap.samples.extend(&self.block[..self.len]);
            tap.last = Some(Instant::now());
        }
        self.len = 0;
    }
}

/// The newest `count` samples, with silence where there are fewer, and all
/// silence once the sound has stopped.
fn window(count: usize) -> Vec<f32> {
    let tap = TAP.lock().unwrap_or_else(|p| p.into_inner());
    let mut out = vec![0.0; count];
    if tap.last.is_none_or(|last| last.elapsed() > QUIET_AFTER) {
        return out;
    }
    let taken = tap.samples.len().min(count);
    let start = tap.samples.len() - taken;
    for (slot, sample) in out[count - taken..].iter_mut().zip(tap.samples.range(start..)) {
        *slot = *sample;
    }
    out
}

/// The FFT Winamp's analyser uses: 512 samples under a Hann window, 256
/// magnitudes out, each halved.
struct Fft {
    bit_reversed: Vec<usize>,
    envelope:     Vec<f32>,
    twiddles:     Vec<(f32, f32)>,
    real:         Vec<f32>,
    imaginary:    Vec<f32>,
}

impl Fft {
    fn new(n: usize) -> Self {
        let mut bit_reversed: Vec<usize> = (0..n).collect();
        let mut j = 0;
        for i in 0..n {
            if j > i {
                bit_reversed.swap(i, j);
            }
            let mut m = n >> 1;
            while m >= 1 && j >= m {
                j -= m;
                m >>= 1;
            }
            j += m;
        }
        let envelope = (0..n)
            .map(|i| {
                let phase = i as f32 / n as f32 * std::f32::consts::TAU;
                0.5 + 0.5 * (phase - std::f32::consts::FRAC_PI_2).sin()
            })
            .collect();
        let mut twiddles = Vec::new();
        let mut size = 2;
        while size <= n {
            let theta = -std::f32::consts::TAU / size as f32;
            twiddles.push((theta.cos(), theta.sin()));
            size <<= 1;
        }
        Self { bit_reversed, envelope, twiddles, real: vec![0.0; n], imaginary: vec![0.0; n] }
    }

    fn spectrum(&mut self, wave: &[f32], out: &mut [f32]) {
        let n = self.real.len();
        for i in 0..n {
            let from = self.bit_reversed[i];
            self.real[i] = wave.get(from).copied().unwrap_or(0.0) * self.envelope[from];
            self.imaginary[i] = 0.0;
        }
        let mut size = 2;
        let mut stage = 0;
        while size <= n {
            let (wpr, wpi) = self.twiddles[stage];
            let (mut wr, mut wi) = (1.0f32, 0.0f32);
            let half = size >> 1;
            for m in 0..half {
                let mut i = m;
                while i < n {
                    let j = i + half;
                    let tr = wr * self.real[j] - wi * self.imaginary[j];
                    let ti = wr * self.imaginary[j] + wi * self.real[j];
                    self.real[j] = self.real[i] - tr;
                    self.imaginary[j] = self.imaginary[i] - ti;
                    self.real[i] += tr;
                    self.imaginary[i] += ti;
                    i += size;
                }
                let previous = wr;
                wr = wr * wpr - wi * wpi;
                wi = wi * wpr + previous * wpi;
            }
            size <<= 1;
            stage += 1;
        }
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = (self.real[i] * self.real[i] + self.imaginary[i] * self.imaginary[i]).sqrt() * SPEC_SCALE;
        }
    }
}

/// Winamp's bands, from its published source: seventy-five spans a semitone
/// apart (`2^(x/12)`), each summing its share of the 256 bins, the fractional
/// edges read through a Hermite curve, the sum clipped at 255.
fn bands(spectrum: &[f32; SPECTRUM_BINS]) -> [f32; BANDS] {
    let bla = 255.0 / 2f32.powf(75.0 / 12.0);
    let warp = |x: f32| (2f32.powf(x / 12.0) - 1.0) * bla;
    let sample = |index: usize| spectrum.get(index).copied().unwrap_or(0.0);
    let hermite = |x: f32, y0: f32, y1: f32, y2: f32, y3: f32| {
        let c1 = 0.5 * (y2 - y0);
        let c3 = 1.5 * (y1 - y2) + 0.5 * (y3 - y0);
        let c2 = y0 - y1 + c1 - c3;
        ((c3 * x + c2) * x + c1) * x + y1
    };
    let mut columns = [0.0; BANDS];
    let mut next = warp(0.0) + 1.0;
    for (x, column) in columns.iter_mut().enumerate() {
        let low = next;
        next = warp(x as f32 + 1.0) + 1.0;
        let mut value = 0.0f32;
        let mut bin = low.floor() as usize;
        let end = (next.floor() as usize).min(SPECTRUM_BINS - 1);
        let mut fraction = low;
        let mut mult = (bin as f32 + 1.0) - low;
        let mut herm = true;
        loop {
            if bin == end {
                mult = next - fraction;
                herm = true;
            }
            if herm {
                value += hermite(
                    fraction - bin as f32,
                    sample(bin.saturating_sub(1)),
                    sample(bin),
                    sample(bin + 1),
                    sample(bin + 2),
                ) * mult;
            } else {
                value += sample(bin);
            }
            herm = false;
            bin += 1;
            if bin > end {
                break;
            }
            fraction = bin as f32;
        }
        *column = value.min(255.0);
    }
    columns
}

/// The analyser's memory: where each band and its peak cap are, 0 to 1, with
/// falls timed by the clock so they move alike at any frame rate.
pub struct Analyser {
    fft:      Fft,
    wave:     [f32; FFT_SAMPLES],
    spectrum: [f32; SPECTRUM_BINS],
    levels:   [f32; BANDS],
    peaks:    [f32; BANDS],
    held:     [f32; BANDS],
    speed:    [f32; BANDS],
    last:     Option<Instant>,
}

impl Default for Analyser {
    fn default() -> Self {
        Self {
            fft:      Fft::new(FFT_SAMPLES),
            wave:     [0.0; FFT_SAMPLES],
            spectrum: [0.0; SPECTRUM_BINS],
            levels:   [0.0; BANDS],
            peaks:    [0.0; BANDS],
            held:     [0.0; BANDS],
            speed:    [0.0; BANDS],
            last:     None,
        }
    }
}

impl Analyser {
    /// Moves the bands towards the spectrum of `samples` (mono, -1 to 1):
    /// up at once, down at Winamp's rate for the time since the last step.
    pub fn step(&mut self, samples: &[f32], now: Instant) -> [f32; BANDS] {
        let elapsed = self
            .last
            .map_or(FRAME_EVERY, |last| now.saturating_duration_since(last))
            .min(Duration::from_millis(250))
            .as_secs_f32();
        self.last = Some(now);
        self.wave.fill(0.0);
        for (slot, sample) in self.wave.iter_mut().zip(samples) {
            *slot = sample * CHANNEL_SUM;
        }
        self.fft.spectrum(&self.wave, &mut self.spectrum);
        let columns = bands(&self.spectrum);
        for (level, column) in self.levels.iter_mut().zip(columns) {
            let target = column.min(MAX_HEIGHT) / MAX_HEIGHT;
            *level = (*level - FALL * elapsed).max(target);
            if *level < 0.01 {
                *level = 0.0;
            }
        }
        for band in 0..BANDS {
            let level = self.levels[band];
            if level >= self.peaks[band] {
                self.peaks[band] = level;
                self.held[band] = 0.0;
                self.speed[band] = 0.0;
            } else if self.held[band] < PEAK_HOLD {
                self.held[band] += elapsed;
            } else {
                self.speed[band] += PEAK_GRAVITY * elapsed;
                self.peaks[band] = (self.peaks[band] - self.speed[band] * elapsed).max(level);
            }
            if self.peaks[band] < 0.01 {
                self.peaks[band] = 0.0;
            }
        }
        self.levels
    }

    pub fn peaks(&self) -> [f32; BANDS] {
        self.peaks
    }

    /// Whether every band and peak has fallen to rest.
    pub fn settled(&self) -> bool {
        self.levels.iter().chain(&self.peaks).all(|level| *level == 0.0)
    }
}

/// The scope's trace: every seventh sample, nothing smoothed, for `points`
/// columns. Each value runs from -1 to 1; like Winamp's, anything louder than
/// half full scale reaches the edge.
pub fn scope_line(samples: &[f32], points: usize) -> Vec<f32> {
    (0..points)
        .map(|point| (samples.get(point * SCOPE_STEP).copied().unwrap_or(0.0) * 2.0).clamp(-1.0, 1.0))
        .collect()
}

/// One picture: the bands and their peak caps from low to high, 0 to 1, and
/// the scope's trace, -1 to 1.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Frame {
    pub bands: Vec<f32>,
    pub peaks: Vec<f32>,
    pub scope: Vec<f32>,
    /// False once the sound has stopped and everything has fallen to rest.
    pub moving: bool,
}

static ANALYSER: LazyLock<Mutex<Analyser>> = LazyLock::new(|| Mutex::new(Analyser::default()));

/// The picture now.
pub fn frame(points: usize) -> Frame {
    let points = points.clamp(1, MAX_POINTS);
    let samples = window(FFT_SAMPLES.max(points * SCOPE_STEP));
    let newest = &samples[samples.len() - FFT_SAMPLES..];
    let mut analyser = ANALYSER.lock().unwrap_or_else(|p| p.into_inner());
    let bands = analyser.step(newest, Instant::now());
    let scope_from = &samples[samples.len() - points * SCOPE_STEP..];
    let scope = scope_line(scope_from, points);
    Frame {
        bands: bands.to_vec(),
        peaks: analyser.peaks().to_vec(),
        moving: !analyser.settled() || scope.iter().any(|s| *s != 0.0),
        scope,
    }
}

/// Bumped by every start, so a stream that has been replaced stops.
static STREAM: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Starts or stops the tap. While it runs, `visualizer:frame` is sent sixty
/// times a second, and not at all while everything is at rest.
pub fn stream(app: tauri::AppHandle, enabled: bool, points: usize) {
    use tauri::Emitter;
    let generation = STREAM.fetch_add(1, Ordering::AcqRel) + 1;
    ON.store(enabled, Ordering::Relaxed);
    if !enabled {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(FRAME_EVERY);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut resting = false;
        while STREAM.load(Ordering::Acquire) == generation {
            ticker.tick().await;
            let frame = frame(points);
            // one frame at rest so the picture lands on zero, then quiet
            if !frame.moving && resting {
                continue;
            }
            resting = !frame.moving;
            if app.emit("visualizer:frame", &frame).is_err() {
                break;
            }
        }
    });
}

/// Starts the tap without a stream, for frames asked for one at a time.
pub fn wanted() {
    ON.store(true, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tap is process-wide; tests that use it take turns.
    static TURN: Mutex<()> = Mutex::new(());

    fn sine(hz: f32, rate: f32, len: usize, level: f32) -> Vec<f32> {
        (0..len).map(|n| level * (std::f32::consts::TAU * hz * n as f32 / rate).sin()).collect()
    }

    #[test]
    fn a_tone_lifts_its_own_band_and_leaves_the_far_ones() {
        let mut analyser = Analyser::default();
        let low = analyser.step(&sine(100.0, 44_100.0, FFT_SAMPLES, 0.5), Instant::now());
        let loudest = (0..BANDS).max_by(|a, b| low[*a].total_cmp(&low[*b])).unwrap();
        assert!(low[loudest] > 0.5, "{low:?}");
        assert!(loudest < BANDS / 2, "a 100 Hz tone peaked in band {loudest}");
        assert!(low[BANDS - 1] < 0.1);

        let mut analyser = Analyser::default();
        let high = analyser.step(&sine(8_000.0, 44_100.0, FFT_SAMPLES, 0.5), Instant::now());
        let loudest_high = (0..BANDS).max_by(|a, b| high[*a].total_cmp(&high[*b])).unwrap();
        assert!(loudest_high > loudest + 20, "{loudest_high} vs {loudest}");
    }

    #[test]
    fn bands_fall_at_their_rate_and_come_to_rest() {
        let mut analyser = Analyser::default();
        let start = Instant::now();
        let up = analyser.step(&sine(440.0, 44_100.0, FFT_SAMPLES, 0.5), start);
        let top = up.iter().cloned().fold(0.0, f32::max);
        assert!(top > 0.5);
        let silence = [0.0; FFT_SAMPLES];
        let down = analyser.step(&silence, start + Duration::from_millis(100));
        let fallen = down.iter().cloned().fold(0.0, f32::max);
        assert!(fallen < top && fallen > top - 0.5, "{top} -> {fallen}");
        assert!(!analyser.settled());
        let mut at = start;
        for _ in 0..200 {
            at += Duration::from_millis(50);
            analyser.step(&silence, at);
        }
        assert!(analyser.settled());
    }

    #[test]
    fn the_scope_clamps_at_half_full_scale_and_pads_with_silence() {
        let line = scope_line(&[0.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.9], 4);
        assert_eq!(line, [0.5, 1.0, -1.0, 0.0]);
    }

    #[test]
    fn the_device_thread_hands_over_a_mono_mix_and_silence_reads_as_silence() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        {
            let mut tap = TAP.lock().unwrap();
            tap.samples.clear();
            tap.last = None;
        }
        assert!(window(4).iter().all(|s| *s == 0.0));
        let mut collector = Collector::default();
        for frame in 0..BLOCK {
            let value = frame as f32 / BLOCK as f32;
            collector.add(0, 2, value);
            collector.add(1, 2, -value + 0.5);
        }
        // each frame is the mean of its two channels
        assert!(window(8).iter().all(|s| (*s - 0.25).abs() < 1e-6));
        // a reader holding the lock costs the picture a block, never waits
        let held = TAP.lock().unwrap();
        for _ in 0..BLOCK {
            collector.add(0, 1, 1.0);
        }
        drop(held);
        assert!(window(1)[0] < 1.0);
        TAP.lock().unwrap().last = Some(Instant::now() - QUIET_AFTER * 2);
        assert!(window(8).iter().all(|s| *s == 0.0));
    }
}
