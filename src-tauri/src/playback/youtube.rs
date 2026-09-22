// Audio backend for YouTube-sourced playback.
//
// This is the free-account counterpart to the librespot `PlaybackInner` in
// `mod.rs`. It deliberately mirrors that type's control surface - `play_track`,
// `resume`, `pause`, `seek`, `is_playing`, `is_ended`, `current_track` - so the
// command layer can hold either one behind the same calls, and emits the same
// `player:event` / `PlayerMsg` payloads so the frontend cannot tell which
// backend is running except where we explicitly tell it.
//
// Where it differs from the Spotify path, and why:
//
//  * **No Spirc / Connect.** Spotify Connect is a Spotify-account feature. A
//    free account playing YouTube audio must not advertise itself as a Spotify
//    Connect device or report state to Spotify's backend - it isn't playing
//    Spotify content.
//  * **Whole track buffered before playback.** See `youtube::stream`. At ~3-4 MB
//    per track this is cheap, and it makes seeking exact and instant instead of
//    requiring range bookkeeping inside the decoder.
//  * **Position is polled, not pushed.** rodio has no event channel, so a timer
//    task samples `Sink::get_pos`. Emission is coalesced on the same ~1/second
//    rule the librespot path uses, for the same reason: every event is a JSON
//    IPC message plus a React state update, and the frontend interpolates
//    between them anyway.

use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::{errors::AppError, media_controls::MediaMsg, youtube::ExtractedStream};

use super::{PlayerMsg, SharedVolume};

/// How often the watcher samples playback position.
///
/// 250ms keeps end-of-track detection tight enough that gapless-ish queue
/// advance feels immediate, while the 1/second emit coalescing below keeps the
/// IPC traffic identical to the librespot path.
const POLL_INTERVAL: Duration = Duration::from_millis(250);

pub struct YtPlayback {
    // Dropping the stream closes the audio device, so it is owned here even
    // though nothing reads it.
    _stream:      rodio::OutputStream,
    sink:         Arc<rodio::Sink>,
    volume:       SharedVolume,
    /// Emitted state changes go through here, same channel the librespot path
    /// uses, so the frontend cannot tell the backends apart.
    app:          AppHandle,
    media_tx:     std::sync::mpsc::SyncSender<MediaMsg>,
    /// Audio sitting between the decoder and the speaker, in milliseconds.
    ///
    /// This is not cosmetic: `lib/outputLatency.ts` subtracts it from the
    /// reported position to place the lyric highlight, because the position is
    /// where the *decoder* is and the listener hears what left it a buffer ago.
    /// Reporting nothing here left the lyric clock using its 250ms placeholder,
    /// which is wrong by however much the real buffer differs.
    latency_ms:   i64,
    /// Spotify track id currently loaded - the frontend's identifier, not
    /// YouTube's, so events match what the rest of the app is showing.
    current:      Arc<Mutex<Option<String>>>,
    is_playing:   Arc<AtomicBool>,
    is_ended:     Arc<AtomicBool>,
    duration_ms:  Arc<AtomicU64>,
    /// Bumped on every load. The watcher compares against it so a track that
    /// finishes after a newer one has been loaded cannot fire a stale
    /// EndOfTrack and skip the user past the track they just started.
    generation:   Arc<AtomicU64>,
    /// Per-track linear gain from YouTube's `loudnessDb`, folded into the sink
    /// volume so switching backends does not change perceived loudness.
    gain:         Arc<Mutex<f32>>,
    _watch_task:  tauri::async_runtime::JoinHandle<()>,
}

impl YtPlayback {
    /// Open the default output device and start the position watcher.
    pub fn new(
        app: AppHandle,
        volume: SharedVolume,
        media_tx: std::sync::mpsc::SyncSender<MediaMsg>,
    ) -> Result<Self, AppError> {
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .map_err(|e| AppError::Playback(format!("audio output: {e}")))?;
        let sink = Arc::new(rodio::Sink::connect_new(stream.mixer()));

        // rodio updates the sink position as the mixer pulls samples, so what
        // is still unheard is essentially the device buffer. Read the granted
        // size rather than assuming one; `Default` means cpal never told us, in
        // which case the librespot path's figure is a far better guess than the
        // frontend's hardcoded placeholder.
        let sample_rate = stream.config().sample_rate();
        let device_buffer_ms = match stream.config().buffer_size() {
            cpal::BufferSize::Fixed(frames) if sample_rate > 0 => {
                ((*frames as u64 * 1000) / sample_rate as u64) as u32
            }
            _ => crate::sink::DEFAULT_BUFFER_MS,
        };
        let latency_ms = crate::sink::calculate_latency_ms(0, sample_rate, Some(device_buffer_ms));
        eprintln!("[youtube] audio out {sample_rate}Hz, output latency ~{latency_ms}ms");

        let current     = Arc::new(Mutex::new(None::<String>));
        let is_playing  = Arc::new(AtomicBool::new(false));
        let is_ended    = Arc::new(AtomicBool::new(false));
        let duration_ms = Arc::new(AtomicU64::new(0));
        let generation  = Arc::new(AtomicU64::new(0));

        let watch_task = spawn_watcher(
            app.clone(),
            media_tx.clone(),
            Arc::clone(&sink),
            Arc::clone(&current),
            Arc::clone(&is_playing),
            Arc::clone(&is_ended),
            Arc::clone(&generation),
        );

        Ok(Self {
            _stream: stream,
            sink,
            volume,
            app,
            media_tx,
            latency_ms,
            current,
            is_playing,
            is_ended,
            duration_ms,
            generation,
            gain: Arc::new(Mutex::new(1.0)),
            _watch_task: watch_task,
        })
    }

    /// Decode `audio` and begin playing `track_id` from `position_ms`.
    ///
    /// `audio` is the complete encoded file from `youtube::fetch_audio`.
    pub fn play_track(
        &self,
        track_id: &str,
        stream: &ExtractedStream,
        audio: Arc<[u8]>,
        position_ms: u32,
    ) -> Result<(), AppError> {
        // Bump the generation *before* clearing so an in-flight watcher tick
        // cannot attribute the old track's drained queue to the new track.
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.sink.clear();

        let cursor = Cursor::new(audio);
        // itag 140 is AAC in an MP4 container. Naming the container skips
        // sniffing and, more usefully, produces a clear error if we are ever
        // handed something else by a future format change.
        let decoder = if stream.format.mime_type.contains("mp4") {
            rodio::Decoder::new_mp4(cursor)
        } else {
            rodio::Decoder::new(cursor)
        }
        .map_err(|e| {
            AppError::Playback(format!(
                "decode {} (itag {}, {}): {e}",
                stream.video_id, stream.format.itag, stream.format.mime_type
            ))
        })?;

        self.sink.append(decoder);

        *self.gain.lock().unwrap() = loudness_gain(stream.format.loudness_db);
        self.apply_volume();

        if position_ms > 0 {
            // A failed seek is not fatal - the track still plays, just from the
            // start. Better than refusing to play at all.
            if let Err(e) = self.sink.try_seek(Duration::from_millis(position_ms as u64)) {
                eprintln!("[youtube] seek to {position_ms}ms on load failed: {e}");
            }
        }

        *self.current.lock().unwrap() = Some(track_id.to_string());
        self.duration_ms.store(stream.duration_ms.unwrap_or(0), Ordering::Relaxed);
        self.is_ended.store(false, Ordering::Relaxed);
        self.is_playing.store(true, Ordering::Relaxed);
        self.sink.play();

        eprintln!(
            "[youtube] playing {track_id} via {} (itag {}) from {position_ms}ms",
            stream.video_id, stream.format.itag
        );
        self.announce_playing(position_ms);
        Ok(())
    }

    pub fn resume(&self) -> Result<(), AppError> {
        self.sink.play();
        self.is_playing.store(true, Ordering::Relaxed);
        self.is_ended.store(false, Ordering::Relaxed);
        // Safe to read back here, unlike on load: the source is already loaded
        // and its position has been tracked all along, so this is accurate.
        self.announce_playing(self.position_ms());
        Ok(())
    }

    pub fn pause(&self) -> Result<(), AppError> {
        self.sink.pause();
        self.is_playing.store(false, Ordering::Relaxed);
        let position_ms = self.position_ms();
        let _ = self.media_tx.try_send(MediaMsg::Paused { position_ms: position_ms as u64 });
        emit(&self.app, PlayerMsg::Paused { track_id: self.current_track(), position_ms });
        Ok(())
    }

    /// Audio buffered between the decoder and the speaker, in milliseconds.
    pub fn output_latency_ms(&self) -> i64 {
        self.latency_ms
    }

    /// Tell the frontend playback started.
    ///
    /// Not decoration: the `playing` event is what clears the optimistic
    /// `targetState`, marks the session ready, and drives scrobbling, now-
    /// playing, desktop notifications and the taste signals. Emitting only
    /// position updates left all of that dead on this backend.
    /// `position_ms` is passed explicitly rather than read back from the sink:
    /// rodio only refreshes its position from the source on a 5ms periodic
    /// access, so immediately after a load-and-seek `get_pos()` can still be
    /// reporting where the *previous* track ended. Emitting that would hand the
    /// lyric clock a wildly wrong base for a moment.
    fn announce_playing(&self, position_ms: u32) {
        let _ = self.media_tx.try_send(MediaMsg::Playing { position_ms: position_ms as u64 });
        emit(&self.app, PlayerMsg::Playing { track_id: self.current_track(), position_ms });
    }

    pub fn seek(&self, position_ms: u32) -> Result<(), AppError> {
        self.sink
            .try_seek(Duration::from_millis(position_ms as u64))
            .map_err(|e| AppError::Playback(format!("seek: {e}")))
    }

    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        let track_id = self.current_track();
        self.sink.clear();
        *self.current.lock().unwrap() = None;
        self.is_playing.store(false, Ordering::Relaxed);
        self.is_ended.store(true, Ordering::Relaxed);
        let _ = self.media_tx.try_send(MediaMsg::Stopped);
        emit(&self.app, PlayerMsg::Stopped { track_id });
    }

    pub fn position_ms(&self) -> u32 {
        self.sink.get_pos().as_millis().min(u32::MAX as u128) as u32
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    pub fn is_ended(&self) -> bool {
        self.is_ended.load(Ordering::Relaxed)
    }

    pub fn current_track(&self) -> Option<String> {
        self.current.lock().unwrap().clone()
    }

    /// Set the output level (0.0..=1.0) and push it to the sink.
    pub fn set_level(&self, level: f64) {
        self.volume.set_level(level);
        self.apply_volume();
    }

    /// Mute or unmute, and push it to the sink.
    pub fn set_muted(&self, muted: bool) {
        self.volume.set_muted(muted);
        self.apply_volume();
    }

    /// Re-read the shared volume and push it to the sink.
    ///
    /// The librespot path reads `SharedVolume` from inside its sink on every
    /// audio chunk; rodio's sink holds a plain value, so volume changes have to
    /// be pushed in explicitly whenever they happen.
    pub fn apply_volume(&self) {
        let level = if self.volume.is_muted() { 0.0 } else { self.volume.level() };
        let gain = *self.gain.lock().unwrap();
        self.sink.set_volume((level as f32 * gain).clamp(0.0, 4.0));
    }
}

/// Convert YouTube's `loudnessDb` into a linear gain.
///
/// The field is how many dB *above* YouTube's normalisation target the content
/// sits, so the correction is its negation. Clamped to +/-6 dB: a wild value
/// here would otherwise blow out or bury a track, and beyond that range the
/// figure is more likely bad metadata than a real mastering difference.
fn loudness_gain(loudness_db: Option<f64>) -> f32 {
    match loudness_db {
        Some(db) if db.is_finite() => {
            let corrected = (-db).clamp(-6.0, 6.0);
            10f64.powf(corrected / 20.0) as f32
        }
        _ => 1.0,
    }
}

/// Poll playback position, emit coalesced events, and detect end of track.
#[allow(clippy::too_many_arguments)]
fn spawn_watcher(
    app: AppHandle,
    media_tx: std::sync::mpsc::SyncSender<MediaMsg>,
    sink: Arc<rodio::Sink>,
    current: Arc<Mutex<Option<String>>>,
    is_playing: Arc<AtomicBool>,
    is_ended: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut last_emit = std::time::Instant::now()
            .checked_sub(Duration::from_secs(1))
            .unwrap_or_else(std::time::Instant::now);
        let mut last_pos_ms: u32 = 0;
        // Generation this loop last saw audio queued for. Guards against both
        // the startup state (nothing ever loaded) and a stale EndOfTrack.
        let mut watching_gen: Option<u64> = None;

        loop {
            tokio::time::sleep(POLL_INTERVAL).await;

            let gen = generation.load(Ordering::SeqCst);
            let queued = !sink.empty();
            let track = current.lock().unwrap().clone();

            if queued {
                watching_gen = Some(gen);
            } else if watching_gen == Some(gen) && !is_ended.load(Ordering::Relaxed) {
                // The queue drained on the generation we were following: this
                // track really did finish.
                watching_gen = None;
                is_ended.store(true, Ordering::Relaxed);
                is_playing.store(false, Ordering::Relaxed);
                let _ = media_tx.try_send(MediaMsg::Stopped);
                emit(&app, PlayerMsg::EndOfTrack { track_id: track.clone() });
                continue;
            }

            if !queued || sink.is_paused() {
                continue;
            }

            let position_ms = sink.get_pos().as_millis().min(u32::MAX as u128) as u32;
            let due = last_emit.elapsed() >= Duration::from_millis(1000);
            // A jump means a seek happened; report it immediately rather than
            // letting the frontend interpolate towards a stale position.
            let moved = position_ms.abs_diff(last_pos_ms) >= 1000;
            if due || moved {
                last_emit = std::time::Instant::now();
                last_pos_ms = position_ms;
                let _ = media_tx.try_send(MediaMsg::Playing { position_ms: position_ms as u64 });
                emit(&app, PlayerMsg::PositionChanged { track_id: track, position_ms });
            }
        }
    })
}

fn emit(app: &AppHandle, msg: PlayerMsg) {
    if let Err(e) = app.emit("player:event", msg) {
        eprintln!("[youtube] emit failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loudness_gain_is_unity_without_metadata() {
        assert_eq!(loudness_gain(None), 1.0);
        assert_eq!(loudness_gain(Some(f64::NAN)), 1.0);
        assert_eq!(loudness_gain(Some(0.0)), 1.0);
    }

    #[test]
    fn loudness_gain_attenuates_loud_content() {
        // +6 dB over target should come back down by ~half amplitude.
        let g = loudness_gain(Some(6.0));
        assert!((g - 0.501).abs() < 0.01, "got {g}");
    }

    #[test]
    fn loudness_gain_boosts_quiet_content() {
        let g = loudness_gain(Some(-6.0));
        assert!((g - 1.995).abs() < 0.01, "got {g}");
    }

    /// Bad metadata must not be able to blow out the output.
    #[test]
    fn loudness_gain_is_clamped() {
        assert!(loudness_gain(Some(-60.0)) <= 2.0);
        assert!(loudness_gain(Some(60.0)) >= 0.5);
    }
}
