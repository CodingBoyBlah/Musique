//! pitch-preserving playback speed (WSOLA time-stretch) for podcasts.
//!
//! plain resampling to play faster raises the pitch (1.5x speech sounds like
//! chipmunks). WSOLA instead cuts the input into overlapping windowed frames
//! and lays them down at a different hop than they were read at - reading
//! `speed` times further per output frame. each next frame is nudged within a
//! small search range to where it best lines up (cross-correlation) with what
//! would naturally have followed the previous one, which is what keeps the
//! waveform continuous and speech clean at 0.75x-2x.
//!
//! works on interleaved stereo f32 at whatever rate the sink runs at.

use std::sync::atomic::{AtomicU32, Ordering};

const CHANNELS: usize = 2;

/// the speed the sink plays at right now (f32 bits). set from the ui; the
/// sink reads it per packet so a change lands within one packet
static SPEED: AtomicU32 = AtomicU32::new(0x3f80_0000); // 1.0

pub fn set_speed(speed: f32) {
    SPEED.store(speed.clamp(0.5, 3.0).to_bits(), Ordering::Relaxed);
}

pub fn speed() -> f32 {
    f32::from_bits(SPEED.load(Ordering::Relaxed))
}
/// analysis/synthesis frame, in frames per channel (~23ms at 44.1k)
const FRAME: usize = 1024;
/// synthesis hop = half a frame (50% overlap, hann window sums to 1)
const HOP: usize = FRAME / 2;
/// how far a frame may slide to find the best alignment (~6ms at 44.1k)
const SEEK: usize = 256;

pub struct TimeStretch {
    speed: f64,
    window: Vec<f32>,
    /// pending input, interleaved
    input: Vec<f32>,
    /// where the next frame is read from in `input` (frames, fractional)
    read_pos: f64,
    /// where the frame laid down last time would naturally have continued
    /// (its start + one hop), in `input` frames
    natural: Option<usize>,
    /// second half of the last windowed frame, waiting to be overlap-added
    tail: Vec<f32>,
}

impl TimeStretch {
    pub fn new(speed: f64) -> Self {
        let window = (0..FRAME)
            .map(|i| {
                let x = std::f64::consts::PI * 2.0 * i as f64 / FRAME as f64;
                (0.5 - 0.5 * x.cos()) as f32
            })
            .collect();
        Self {
            speed: speed.clamp(0.5, 3.0),
            window,
            input: Vec::new(),
            read_pos: 0.0,
            natural: None,
            tail: vec![0.0; HOP * CHANNELS],
        }
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    fn frames(&self) -> usize {
        self.input.len() / CHANNELS
    }

    /// mono sample at frame `f` (for the alignment search)
    fn mono(&self, f: usize) -> f32 {
        let i = f * CHANNELS;
        (self.input[i] + self.input[i + 1]) * 0.5
    }

    /// best start near `nominal`: the offset whose opening overlaps most like
    /// the natural continuation of the previous frame
    fn align(&self, nominal: usize) -> usize {
        let Some(natural) = self.natural else { return nominal };
        let lo = nominal.saturating_sub(SEEK);
        let hi = nominal + SEEK;
        let mut best = nominal;
        let mut best_score = f32::MIN;

        // The natural continuation is identical for every search offset.
        let mut natural_mono = [0.0f32; HOP / 4];
        let mut j = 0;
        for item in &mut natural_mono {
            *item = self.mono(natural + j);
            j += 4;
        }

        // every 2nd offset and every 4th sample: plenty for alignment, a
        // fraction of the cost
        let mut k = lo;
        while k <= hi {
            let mut score = 0.0f32;
            let mut j = 0;
            for &nat in &natural_mono {
                score += self.mono(k + j) * nat;
                j += 4;
            }
            if score > best_score {
                best_score = score;
                best = k;
            }
            k += 2;
        }
        best
    }

    /// feed input, get whatever output is ready. at speed 1 this is a straight
    /// pass-through (no windowing, no latency)
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        if (self.speed - 1.0).abs() < 1e-3 {
            return samples.to_vec();
        }
        self.input.extend_from_slice(samples);
        let analysis_hop = HOP as f64 * self.speed;
        let mut out = Vec::with_capacity((samples.len() as f64 / self.speed) as usize + HOP * CHANNELS);

        loop {
            let nominal = self.read_pos.round() as usize;
            // need room to search ahead of the nominal start and to read the
            // natural continuation of the previous frame
            let need = (nominal + SEEK + FRAME).max(self.natural.map(|n| n + HOP).unwrap_or(0));
            if need > self.frames() {
                break;
            }
            let start = self.align(nominal);
            for f in 0..HOP {
                let w = self.window[f];
                let in_idx = (start + f) * CHANNELS;
                let tail_idx = f * CHANNELS;
                out.push(self.tail[tail_idx] + self.input[in_idx] * w);
                out.push(self.tail[tail_idx + 1] + self.input[in_idx + 1] * w);
            }
            for f in HOP..FRAME {
                let w = self.window[f];
                let in_idx = (start + f) * CHANNELS;
                let tail_idx = (f - HOP) * CHANNELS;
                self.tail[tail_idx] = self.input[in_idx] * w;
                self.tail[tail_idx + 1] = self.input[in_idx + 1] * w;
            }
            self.natural = Some(start + HOP);
            self.read_pos += analysis_hop;
        }

        // drop input nothing will read again
        let keep_from = (self.read_pos as usize)
            .saturating_sub(SEEK)
            .min(self.natural.unwrap_or(usize::MAX));
        if keep_from > 0 && keep_from <= self.frames() {
            self.input.drain(..keep_from * CHANNELS);
            self.read_pos -= keep_from as f64;
            self.natural = self.natural.map(|n| n - keep_from);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frames: usize, hz: f32, rate: f32) -> Vec<f32> {
        (0..frames)
            .flat_map(|i| {
                let v = (i as f32 * hz * 2.0 * std::f32::consts::PI / rate).sin() * 0.5;
                [v, v]
            })
            .collect()
    }

    #[test]
    fn unity_is_passthrough() {
        let mut s = TimeStretch::new(1.0);
        let input = tone(4096, 440.0, 44_100.0);
        assert_eq!(s.process(&input), input);
    }

    #[test]
    fn output_length_tracks_speed() {
        for speed in [0.75, 1.5, 2.0] {
            let mut s = TimeStretch::new(speed);
            let input = tone(44_100 * 2, 220.0, 44_100.0);
            let mut out = Vec::new();
            for chunk in input.chunks(4096 * CHANNELS) {
                out.extend(s.process(chunk));
            }
            let ratio = (input.len() as f64 / out.len() as f64) / speed;
            assert!((ratio - 1.0).abs() < 0.05, "speed {speed}: ratio {ratio}");
        }
    }

    #[test]
    fn keeps_level_and_stays_bounded() {
        let mut s = TimeStretch::new(1.5);
        let input = tone(44_100, 300.0, 44_100.0);
        let out = s.process(&input);
        let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak > 0.3 && peak < 0.8, "peak {peak}");
        // the input buffer doesn't grow without bound
        assert!(s.input.len() < (FRAME + 2 * SEEK + HOP) * CHANNELS * 3);
    }

    #[test]
    fn keeps_pitch() {
        // a 440hz tone played at 1.5x must still be 440hz, just shorter
        let mut s = TimeStretch::new(1.5);
        let input = tone(44_100 * 2, 440.0, 44_100.0);
        let out = s.process(&input);
        let left: Vec<f32> = out.iter().step_by(CHANNELS).copied().collect();
        let body = &left[HOP * 2..left.len() - HOP * 2];
        let crossings = body.windows(2).filter(|w| (w[0] <= 0.0) != (w[1] <= 0.0)).count();
        let hz = crossings as f32 / 2.0 / (body.len() as f32 / 44_100.0);
        assert!((hz - 440.0).abs() < 15.0, "came out at {hz}hz");
    }

    #[test]
    fn silence_stays_silent() {
        let mut s = TimeStretch::new(2.0);
        let out = s.process(&vec![0.0; 8192 * CHANNELS]);
        assert!(out.iter().all(|v| *v == 0.0));
    }
}
