//! Sample rate conversion for an output that will not take 44.1 kHz.
//!
//! Spotify's audio is stereo 44.1 kHz and the output is opened at that
//! rate when the device allows it. Windows shares a device at one rate,
//! the one in its sound settings, and that is 48 kHz on most PCs, so the
//! stream falls back to it. rodio would then resample each chunk on its
//! own, starting its interpolator afresh at every chunk boundary, thirty
//! times a second, and each restart is a small step in the waveform:
//! heard as crackle and static. This converter keeps its state across
//! chunks, so the output is one continuous signal.
//!
//! It is a polyphase windowed sinc: the input is notionally raised by
//! `up`, low-passed, and every `down`th sample kept, with only the taps
//! that reach an output sample computed.

use std::f64::consts::PI;

/// Input samples each output sample is made from. Sixty-four give a
/// passband flat to about 19 kHz and images under the window's floor.
const TAPS: usize = 64;

pub struct Resampler {
    up: usize,
    down: usize,
    channels: usize,
    /// `up` phases of `TAPS` coefficients each, normalised to unit gain.
    taps: Vec<f32>,
    /// Interleaved input frames still in reach: the tail of what came
    /// before, then whatever has not produced its outputs yet.
    input: Vec<f32>,
    /// The frame in `input` the next output sample sits on or just after.
    next: usize,
    /// How far past `next` the output sits, in steps of `1 / up`.
    phase: usize,
}

impl Resampler {
    /// `None` when the rates agree and nothing needs doing.
    pub fn new(from_hz: u32, to_hz: u32, channels: usize) -> Option<Self> {
        if from_hz == to_hz || from_hz == 0 || to_hz == 0 || channels == 0 {
            return None;
        }
        let divisor = gcd(from_hz, to_hz);
        let up = (to_hz / divisor) as usize;
        let down = (from_hz / divisor) as usize;
        let half = TAPS / 2;
        Some(Self {
            up,
            down,
            channels,
            taps: kernel(up, down),
            input: vec![0.0; (half - 1) * channels],
            next: half - 1,
            phase: 0,
        })
    }

    /// Converts a chunk of interleaved frames. The output is what the
    /// input so far allows; the last few frames wait for the next chunk.
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        self.input.extend_from_slice(samples);
        let half = TAPS / 2;
        let frames = self.input.len() / self.channels;
        let expected = samples.len() * self.up / self.down + self.channels;
        let mut out = Vec::with_capacity(expected);
        while self.next + half < frames {
            let taps = &self.taps[self.phase * TAPS..(self.phase + 1) * TAPS];
            let start = (self.next + 1 - half) * self.channels;
            for channel in 0..self.channels {
                let sum: f32 = taps
                    .iter()
                    .enumerate()
                    .map(|(k, tap)| self.input[start + k * self.channels + channel] * tap)
                    .sum();
                out.push(sum);
            }
            let position = self.phase + self.down;
            self.next += position / self.up;
            self.phase = position % self.up;
        }
        // Keep only the frames the next output still reaches back to.
        let keep_from = (self.next + 1 - half).min(frames);
        self.input.drain(..keep_from * self.channels);
        self.next -= keep_from;
        out
    }
}

/// The taps for every phase: a sinc cut just under the lower of the two
/// Nyquist limits, under a Blackman window, each phase scaled to unit
/// gain so a steady level comes out at the level it went in.
fn kernel(up: usize, down: usize) -> Vec<f32> {
    let half = (TAPS / 2) as f64;
    let cutoff = 0.475 * (up as f64 / down as f64).min(1.0);
    let mut taps = Vec::with_capacity(up * TAPS);
    for phase in 0..up {
        let offset = phase as f64 / up as f64;
        let start = taps.len();
        for k in 0..TAPS {
            let u = offset + half - 1.0 - k as f64;
            let sinc = if u.abs() < 1e-9 {
                1.0
            } else {
                (PI * cutoff * u).sin() / (PI * u)
            };
            let blackman = 0.42
                - 0.5 * (2.0 * PI * (k as f64 + offset) / TAPS as f64).cos()
                + 0.08 * (4.0 * PI * (k as f64 + offset) / TAPS as f64).cos();
            taps.push((sinc * blackman) as f32);
        }
        let sum: f64 = taps[start..].iter().map(|&x| x as f64).sum();
        if sum.abs() > 1e-9 {
            for tap in &mut taps[start..] {
                *tap = (*tap as f64 / sum) as f32;
            }
        }
    }
    taps
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}
