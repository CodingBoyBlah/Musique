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

/// Input samples each output sample is made from. A Blackman window
/// over 128 of them makes a transition band about 1.9 kHz wide, so the
/// passband stays flat to 20 kHz and the images are gone by 22 kHz.
const TAPS: usize = 128;

/// Where the filter's -6 dB point sits, as a fraction of the lower
/// Nyquist limit: 0.95 of 22.05 kHz is 20.9 kHz.
const CUTOFF: f64 = 0.95;

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
/// Nyquist limits, under a Blackman window centred on it, each phase
/// scaled to unit gain so a steady level comes out at the level it went in.
///
/// `cutoff` is in units of the input's Nyquist frequency: `sin(pi c u) /
/// (pi u)` passes up to `c * fs / 2`. It was once 0.475 here, read as a
/// fraction of the sample rate, which cut everything above 10.5 kHz.
fn kernel(up: usize, down: usize) -> Vec<f32> {
    let half = (TAPS / 2) as f64;
    let cutoff = CUTOFF * (up as f64 / down as f64).min(1.0);
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
            // the sinc peaks at k = half - 1 + offset; centre the window there
            let x = (k as f64 + 1.0 - offset) / TAPS as f64;
            let blackman = 0.42 - 0.5 * (2.0 * PI * x).cos() + 0.08 * (4.0 * PI * x).cos();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Level of a stereo sine at `hz` after 44.1 kHz -> 48 kHz, in dB
    /// relative to the input, measured in chunks as the sink feeds it.
    fn gain_db(hz: f64) -> f64 {
        let mut resampler = Resampler::new(44_100, 48_000, 2).unwrap();
        let input: Vec<f32> = (0..44_100)
            .flat_map(|n| {
                let s = (2.0 * PI * hz * n as f64 / 44_100.0).sin() as f32 * 0.5;
                [s, s]
            })
            .collect();
        let out: Vec<f32> = input.chunks(2 * 1024).flat_map(|c| resampler.process(c)).collect();
        // skip the warm-up and the tail, left channel only
        let body: Vec<f64> = out[4_000..out.len() - 4_000].iter().step_by(2).map(|&x| x as f64).collect();
        let rms = (body.iter().map(|x| x * x).sum::<f64>() / body.len() as f64).sqrt();
        20.0 * (rms / (0.5 / 2f64.sqrt())).log10()
    }

    /// What is left of the output once the tone at `hz` is fitted and
    /// taken out, in dB under the tone: aliases, images and window leakage.
    fn residue_db(hz: f64) -> f64 {
        let mut resampler = Resampler::new(44_100, 48_000, 2).unwrap();
        let input: Vec<f32> = (0..44_100)
            .flat_map(|n| {
                let s = (2.0 * PI * hz * n as f64 / 44_100.0).sin() as f32 * 0.5;
                [s, s]
            })
            .collect();
        let out: Vec<f32> = input.chunks(2 * 1024).flat_map(|c| resampler.process(c)).collect();
        let body: Vec<f64> = out[4_000..out.len() - 4_000].iter().step_by(2).map(|&x| x as f64).collect();
        let w = 2.0 * PI * hz / 48_000.0;
        let (mut ss, mut sc, mut cc, mut ys, mut yc) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for (n, y) in body.iter().enumerate() {
            let (s, c) = (w * n as f64).sin_cos();
            ss += s * s; sc += s * c; cc += c * c; ys += y * s; yc += y * c;
        }
        let det = ss * cc - sc * sc;
        let a = (ys * cc - yc * sc) / det;
        let b = (yc * ss - ys * sc) / det;
        let residue: f64 = body
            .iter()
            .enumerate()
            .map(|(n, y)| {
                let (s, c) = (w * n as f64).sin_cos();
                (y - a * s - b * c).powi(2)
            })
            .sum::<f64>()
            / body.len() as f64;
        10.0 * (residue / 0.125).log10()
    }

    #[test]
    fn nothing_but_the_tone_comes_out() {
        for hz in [440.0, 5_000.0, 12_000.0, 18_000.0] {
            let db = residue_db(hz);
            eprintln!("{hz:>6} Hz residue: {db:.1} dB");
            assert!(db < -70.0, "{hz} Hz leaves a residue at {db:.1} dB");
        }
    }

    #[test]
    fn passband_reaches_the_top_of_hearing() {
        let levels: Vec<(f64, f64)> = [1_000.0, 5_000.0, 10_000.0, 12_000.0, 15_000.0, 18_000.0, 20_000.0]
            .into_iter()
            .map(|hz| (hz, gain_db(hz)))
            .collect();
        for (hz, db) in &levels {
            eprintln!("{hz:>6} Hz: {db:+.2} dB");
        }
        for (hz, db) in levels {
            assert!(db.abs() < 0.5, "{hz} Hz comes out at {db:+.2} dB");
        }
    }
}
