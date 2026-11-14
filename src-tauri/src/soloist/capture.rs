//! What Soloist plays, observed for the visualizer.
//!
//! Soloist opens the audio device itself, so its sound never passes through
//! the app's own output, where the visualizer taps the other backends. On
//! Windows the sound of that one process (and nothing else on the system) is
//! read back with WASAPI process loopback (`native/windows.cpp`). It is
//! observation only: the sound that is heard is not touched, and no session
//! or device volume is changed. It runs only while the visualizer is watching
//! and Soloist is playing. Elsewhere there is nothing to read it with, and
//! the visualizer rests while Soloist plays.
//!
//! Adapted from Spotifast (https://github.com/crmne/spotifast),
//! Copyright (c) 2026 Carmine Paolino, under the MIT License (LICENSE-spotifast).

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;

use serde_json::Value;

struct State {
    stop:    AtomicBool,
    playing: AtomicBool,
    /// Soloist's own volume, 0 to 100, which the captured sound is after.
    volume:  AtomicU8,
}

/// The capture of one Soloist process. Dropping it stops the capture.
pub struct Capture {
    state: Arc<State>,
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
    }
}

/// The gain Soloist itself applies at a volume of `volume` percent: 0.5 dB a
/// percent down to 10%, then a straight fade to silence.
fn soloist_gain(volume: u8) -> f32 {
    let volume = f32::from(volume.min(100));
    if volume >= 10.0 {
        10f32.powf((volume - 100.0) / 40.0)
    } else {
        volume / 10.0 * 10f32.powf(-90.0 / 40.0)
    }
}

/// What the visualizer is given for sound captured at `volume`: the sound as
/// it was before Soloist turned it down, as the other backends' tap sees it.
fn undo_volume(volume: u8) -> f32 {
    let gain = soloist_gain(volume);
    if gain > 0.0 { 1.0 / gain } else { 0.0 }
}

impl Capture {
    /// Starts watching the Soloist process `pid`, playing at `volume`.
    pub fn start(pid: Option<u32>, volume: u8) -> Self {
        let state = Arc::new(State {
            stop:    AtomicBool::new(false),
            playing: AtomicBool::new(false),
            volume:  AtomicU8::new(volume.min(100)),
        });
        #[cfg(windows)]
        if let Some(pid) = pid {
            native::spawn(pid, state.clone());
        }
        #[cfg(not(windows))]
        let _ = pid;
        Self { state }
    }

    /// Follows what Soloist reports: whether it plays, and at what volume.
    pub fn update(&self, message: &Value) {
        if let Some(volume) = message["volume"].as_u64() {
            self.state.volume.store(volume.min(100) as u8, Ordering::Relaxed);
        }
        match message["status"].as_str() {
            Some("playing" | "buffering") => self.state.playing.store(true, Ordering::Release),
            Some("idle" | "paused" | "stopped") => self.state.playing.store(false, Ordering::Release),
            _ => {}
        }
        if message["is_active"] == false {
            self.state.playing.store(false, Ordering::Release);
        }
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::ffi::c_void;

    extern "C" {
        fn musique_soloist_audio(
            pid: u32,
            context: *mut c_void,
            control: extern "C" fn(*mut c_void) -> u32,
            process: extern "C" fn(*mut c_void, *mut f32, usize, u32),
            event: extern "C" fn(*mut c_void, u32),
        ) -> i32;
    }

    /// How long to wait before watching again after the capture ended on
    /// its own (Soloist not making sound yet, a device change).
    const AGAIN_AFTER: std::time::Duration = std::time::Duration::from_secs(2);

    /// Bit 0: stop. Bit 1: capture now.
    extern "C" fn control(context: *mut c_void) -> u32 {
        // SAFETY: `context` is the `State` borrowed for the whole native call.
        let state = unsafe { &*(context as *const State) };
        let stop = state.stop.load(Ordering::Acquire);
        let wanted = state.playing.load(Ordering::Acquire) && crate::vis::on();
        u32::from(stop) | (u32::from(wanted) << 1)
    }

    extern "C" fn process(context: *mut c_void, samples: *mut f32, len: usize, _rate: u32) {
        // SAFETY: as in `control`; the native side lends `len` samples for
        // the length of this call only.
        let state = unsafe { &*(context as *const State) };
        let samples = unsafe { std::slice::from_raw_parts(samples, len) };
        let gain = undo_volume(state.volume.load(Ordering::Relaxed));
        crate::vis::push(samples, 2, gain);
    }

    extern "C" fn event(_context: *mut c_void, _event: u32) {}

    pub(super) fn spawn(pid: u32, state: Arc<State>) {
        let spawned = std::thread::Builder::new().name("soloist-capture".into()).spawn(move || {
            while !state.stop.load(Ordering::Acquire) {
                // nothing to do until the visualizer watches a playing soloist
                if control(Arc::as_ptr(&state) as *mut c_void) & 2 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                // SAFETY: `state` outlives the call, which returns before the
                // loop goes round; the callbacks only read its atomics.
                let result = unsafe {
                    musique_soloist_audio(pid, Arc::as_ptr(&state) as *mut c_void, control, process, event)
                };
                if result != 0 {
                    log::debug!("soloist capture ended ({result:#x})");
                }
                std::thread::sleep(AGAIN_AFTER);
            }
        });
        if let Err(error) = spawned {
            log::warn!("could not start the soloist capture: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_capture_undoes_soloists_own_volume() {
        assert!((soloist_gain(100) - 1.0).abs() < 1e-6);
        // 0.5 dB a percent: 60% is 20 dB down
        assert!((soloist_gain(60) - 0.1).abs() < 1e-4);
        assert!((undo_volume(60) - 10.0).abs() < 1e-2);
        assert_eq!(undo_volume(0), 0.0);
        assert!(soloist_gain(5) < soloist_gain(10));
    }

    #[test]
    fn it_follows_what_soloist_reports() {
        let capture = Capture::start(None, 40);
        assert!(!capture.state.playing.load(Ordering::Relaxed));
        capture.update(&json!({"type": "playback_changed", "status": "playing"}));
        assert!(capture.state.playing.load(Ordering::Relaxed));
        capture.update(&json!({"type": "volume_changed", "volume": 70}));
        assert_eq!(capture.state.volume.load(Ordering::Relaxed), 70);
        capture.update(&json!({"type": "playback_state", "is_active": false}));
        assert!(!capture.state.playing.load(Ordering::Relaxed));
        let state = capture.state.clone();
        drop(capture);
        assert!(state.stop.load(Ordering::Relaxed));
    }

    /// Reads back a throwaway process playing a tone at -60 dB, inaudible but
    /// not silent, and nothing else on the system.
    /// `cargo test live_capture -- --ignored --nocapture`
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn live_capture_of_one_process() {
        let dir = std::env::temp_dir().join(format!("musique-capture-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let wav = dir.join("tone.wav");
        let (rate, seconds) = (48_000u32, 6u32);
        let frames = rate * seconds;
        let mut bytes = Vec::new();
        bytes.extend(b"RIFF");
        bytes.extend((36 + frames * 4).to_le_bytes());
        bytes.extend(b"WAVEfmt ");
        bytes.extend(16u32.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(2u16.to_le_bytes());
        bytes.extend(rate.to_le_bytes());
        bytes.extend((rate * 4).to_le_bytes());
        bytes.extend(4u16.to_le_bytes());
        bytes.extend(16u16.to_le_bytes());
        bytes.extend(b"data");
        bytes.extend((frames * 4).to_le_bytes());
        for n in 0..frames {
            let value = (0.001 * (std::f32::consts::TAU * 440.0 * n as f32 / rate as f32).sin() * 32767.0) as i16;
            bytes.extend(value.to_le_bytes());
            bytes.extend(value.to_le_bytes());
        }
        std::fs::write(&wav, bytes).unwrap();
        let mut player = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command"])
            .arg(format!("(New-Object Media.SoundPlayer '{}').PlaySync()", wav.display()))
            .spawn()
            .unwrap();
        crate::vis::wanted();
        let capture = Capture::start(Some(player.id()), 100);
        capture.update(&json!({"status": "playing"}));
        std::thread::sleep(std::time::Duration::from_secs(4));
        let frame = crate::vis::frame(64);
        drop(capture);
        let _ = player.kill();
        let _ = std::fs::remove_dir_all(&dir);
        let loudest = frame.scope.iter().fold(0f32, |m, s| m.max(s.abs()));
        println!("scope peak {loudest:.5}, bands moving: {}", frame.moving);
        assert!(loudest > 0.0005 && loudest < 0.01, "captured {loudest}");
    }
}
