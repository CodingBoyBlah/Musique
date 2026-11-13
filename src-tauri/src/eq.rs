//! A ten-band graphic equalizer, applied where the sound leaves for the device.
//!
//! It runs on the device thread (`playout::RingSource`), after the queue, so a
//! band that is moved is heard at once instead of four seconds later, and it
//! shapes every backend's audio alike. That thread must never wait, so the
//! settings are plain atomics: the bands' gains, whether it is on, and a
//! version that says when to work the filters out again.
//!
//! Each band is a peaking filter (the RBJ cookbook biquad) an octave apart.
//! Boosting a band can push a loud master past full scale, so the whole signal
//! is brought down by the largest boost: the shape is heard, never clipping.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};

/// The centre of each band, in Hz.
pub const BANDS: [f32; 10] = [31.25, 62.5, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];

/// How far a band can be moved either way, in dB.
pub const MAX_GAIN_DB: f32 = 12.0;

/// How wide a band is. Bands an octave apart at this width add up to a level
/// response when they are all set alike.
const Q: f32 = 1.41;

static ENABLED: AtomicBool = AtomicBool::new(false);
/// Each band's gain, in thousandths of a dB.
static GAINS: [AtomicI32; 10] = [const { AtomicI32::new(0) }; 10];
/// Bumped by every change.
static VERSION: AtomicU64 = AtomicU64::new(1);

/// What the equalizer is set to.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub enabled: bool,
    /// One gain per band of `BANDS`, in dB.
    pub gains:   Vec<f32>,
}

pub fn settings() -> Settings {
    Settings {
        enabled: ENABLED.load(Ordering::Relaxed),
        gains:   GAINS.iter().map(|gain| gain.load(Ordering::Relaxed) as f32 / 1000.0).collect(),
    }
}

/// Set the equalizer. Missing bands are left flat and gains are kept within
/// `MAX_GAIN_DB`. Heard within a few milliseconds.
pub fn set(enabled: bool, gains: &[f32]) {
    for (band, slot) in GAINS.iter().enumerate() {
        let gain = gains.get(band).copied().filter(|gain| gain.is_finite()).unwrap_or(0.0);
        slot.store((gain.clamp(-MAX_GAIN_DB, MAX_GAIN_DB) * 1000.0).round() as i32, Ordering::Relaxed);
    }
    ENABLED.store(enabled, Ordering::Relaxed);
    VERSION.fetch_add(1, Ordering::Release);
}

/// What is saved in settings: `1;0,0,3.5,...`.
pub fn encode(settings: &Settings) -> String {
    let gains: Vec<String> = settings.gains.iter().map(|gain| format!("{gain:.2}")).collect();
    format!("{};{}", if settings.enabled { 1 } else { 0 }, gains.join(","))
}

pub fn decode(saved: &str) -> Option<Settings> {
    let (enabled, gains) = saved.split_once(';')?;
    let gains: Vec<f32> = gains.split(',').filter_map(|gain| gain.trim().parse().ok()).collect();
    (gains.len() == BANDS.len()).then_some(Settings { enabled: enabled.trim() == "1", gains })
}

#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl Biquad {
    /// A peaking filter of `gain_db` at `hz`, for audio at `rate`.
    fn peaking(hz: f32, gain_db: f32, rate: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * hz / rate;
        let alpha = w0.sin() / (2.0 * Q);
        let cos = w0.cos();
        let a0 = 1.0 + alpha / a;
        Self {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos) / a0,
            a2: (1.0 - alpha / a) / a0,
        }
    }
}

/// The most channels filtered. Anything past stereo passes through.
const CHANNELS: usize = 2;

/// The filters for one output, and their memory.
pub struct Equalizer {
    rate:    f32,
    version: u64,
    active:  bool,
    /// Only the bands that are off flat, so a gentle curve costs little.
    filters: [Biquad; 10],
    used:    usize,
    /// Brings the whole signal down by the largest boost.
    preamp:  f32,
    /// Each filter's two delayed values, per channel (transposed direct form II).
    state:   [[[f32; 2]; 10]; CHANNELS],
}

impl Equalizer {
    pub fn new(rate: u32) -> Self {
        Self {
            rate: rate.max(1) as f32,
            version: 0,
            active: false,
            filters: [Biquad::default(); 10],
            used: 0,
            preamp: 1.0,
            state: [[[0.0; 2]; 10]; CHANNELS],
        }
    }

    /// Take up any change to the settings. Cheap when there is none: one load.
    /// Call between frames.
    #[inline]
    pub fn refresh(&mut self) {
        let version = VERSION.load(Ordering::Acquire);
        if version != self.version {
            self.rebuild(version);
        }
    }

    #[cold]
    fn rebuild(&mut self, version: u64) {
        self.version = version;
        let enabled = ENABLED.load(Ordering::Relaxed);
        let mut used = 0;
        let mut boost: f32 = 0.0;
        for (band, hz) in BANDS.iter().enumerate() {
            let gain = GAINS[band].load(Ordering::Relaxed) as f32 / 1000.0;
            // a band above what this rate can carry has nothing to act on
            if gain.abs() < 0.05 || *hz >= self.rate * 0.45 {
                continue;
            }
            self.filters[used] = Biquad::peaking(*hz, gain, self.rate);
            used += 1;
            boost = boost.max(gain);
        }
        self.used = used;
        self.preamp = 10f32.powf(-boost / 20.0);
        let active = enabled && used > 0;
        if active && !self.active {
            // coming on: start from silence in the filters, not from whatever
            // was left in them the last time
            self.state = [[[0.0; 2]; 10]; CHANNELS];
        }
        self.active = active;
    }

    #[inline]
    pub fn process(&mut self, channel: usize, sample: f32) -> f32 {
        if !self.active || channel >= CHANNELS {
            return sample;
        }
        let mut x = sample * self.preamp;
        let state = &mut self.state[channel];
        for (filter, z) in self.filters[..self.used].iter().zip(state.iter_mut()) {
            let y = filter.b0 * x + z[0];
            z[0] = filter.b1 * x - filter.a1 * y + z[1];
            z[1] = filter.b2 * x - filter.a2 * y;
            x = y;
        }
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// The settings are process-wide; tests that move them take turns.
    static TURN: Mutex<()> = Mutex::new(());

    /// How loud a sine at `hz` comes out, relative to how it went in, in dB.
    fn response_db(hz: f32, rate: u32) -> f32 {
        let mut eq = Equalizer::new(rate);
        eq.refresh();
        let frames = rate as usize;
        let mut peak: f32 = 0.0;
        for n in 0..frames {
            let x = (2.0 * std::f32::consts::PI * hz * n as f32 / rate as f32).sin();
            let y = eq.process(0, x);
            // once the filters have settled
            if n > frames / 2 {
                peak = peak.max(y.abs());
            }
        }
        20.0 * peak.log10()
    }

    #[test]
    fn flat_or_off_changes_nothing() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        set(true, &[0.0; 10]);
        let mut eq = Equalizer::new(44_100);
        eq.refresh();
        assert_eq!(eq.process(0, 0.25), 0.25);
        set(false, &[6.0; 10]);
        eq.refresh();
        assert_eq!(eq.process(1, -0.5), -0.5);
        set(false, &[0.0; 10]);
    }

    #[test]
    fn a_band_moves_its_own_frequency_and_leaves_the_far_ones() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        let mut gains = [0.0; 10];
        gains[5] = -9.0; // 1 kHz down
        set(true, &gains);
        assert!((response_db(1000.0, 44_100) + 9.0).abs() < 0.5, "{}", response_db(1000.0, 44_100));
        assert!(response_db(62.5, 44_100).abs() < 0.5);
        assert!(response_db(12_000.0, 44_100).abs() < 0.7);
        // the same curve at another output rate
        assert!((response_db(1000.0, 48_000) + 9.0).abs() < 0.5);
        set(false, &[0.0; 10]);
    }

    #[test]
    fn a_boost_never_pushes_past_full_scale() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        let mut gains = [0.0; 10];
        gains[5] = 12.0;
        set(true, &gains);
        // the boosted band comes out no louder than it went in; the rest lower
        assert!(response_db(1000.0, 44_100) <= 0.1);
        assert!((response_db(62.5, 44_100) + 12.0).abs() < 0.6);
        set(false, &[0.0; 10]);
    }

    #[test]
    fn settings_survive_being_saved_and_are_kept_in_range() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        set(true, &[40.0, -40.0, f32::NAN, 1.5]);
        let now = settings();
        assert_eq!(now.gains[..4], [12.0, -12.0, 0.0, 1.5]);
        assert_eq!(now.gains.len(), 10);
        assert_eq!(decode(&encode(&now)), Some(now));
        assert_eq!(decode("nonsense"), None);
        assert_eq!(decode("1;1,2,3"), None);
        set(false, &[0.0; 10]);
    }
}
