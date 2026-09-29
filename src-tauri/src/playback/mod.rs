pub mod youtube;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use librespot_connect::{ConnectConfig, LoadRequest, LoadRequestOptions, PlayingTrack, Spirc};
use librespot_core::{
    authentication::Credentials,
    cache::Cache,
    config::{DeviceType, SessionConfig},
    session::Session,
    Error as LibrespotError,
    SpotifyId,
    SpotifyUri,
};
use librespot_playback::{
    audio_backend::{Sink, SinkResult},
    config::{Bitrate, PlayerConfig},
    convert::Converter,
    decoder::AudioPacket,
    mixer::{Mixer, MixerConfig, VolumeGetter},
    player::{Player, PlayerEvent},
    NUM_CHANNELS, SAMPLE_RATE,
};
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::RwLock;

use crate::{
    auth,
    errors::AppError,
    state::AuthState,
};

// shared volume control

struct VolumeState {
    level: f64,  // 0.0 = silent, 1.0 = full blast
    muted: bool,
}

#[derive(Clone)]
pub struct SharedVolume(Arc<Mutex<VolumeState>>);

impl SharedVolume {
    pub fn new(level: f64, muted: bool) -> Self {
        SharedVolume(Arc::new(Mutex::new(VolumeState {
            level: level.clamp(0.0, 1.0),
            muted,
        })))
    }

    pub fn set_level(&self, level: f64) {
        self.0.lock().unwrap().level = level.clamp(0.0, 1.0);
    }

    pub fn set_muted(&self, muted: bool) {
        self.0.lock().unwrap().muted = muted;
    }

    pub fn level(&self) -> f64 {
        self.0.lock().unwrap().level
    }

    pub fn is_muted(&self) -> bool {
        self.0.lock().unwrap().muted
    }
}

impl VolumeGetter for SharedVolume {
    fn attenuation_factor(&self) -> f64 {
        let s = self.0.lock().unwrap();
        if s.muted { 0.0 } else { s.level }
    }
}

// Mixer adapter so Spotify Connect (Spirc) shares our ONE volume source of truth.
//
// The Player is fed the SharedVolume directly (it keeps mute + the persisted
// level). Spirc, though, controls volume through a `Mixer` (0..=u16::MAX) and
// REPORTS that value in the connect-state it PUTs to Spotify. Wiring the mixer
// to the same SharedVolume means: (a) the volume Spotify shows for this device
// matches what's actually playing, and (b) a remote volume change from another
// Spotify client (phone/web) moves OUR real volume too. `open()` is never called
// (we construct it directly with the live SharedVolume) - it only exists to
// satisfy the trait.
#[derive(Clone)]
struct SharedMixer(SharedVolume);

impl Mixer for SharedMixer {
    fn open(_config: MixerConfig) -> Result<Self, LibrespotError> {
        Ok(SharedMixer(SharedVolume::new(0.5, false)))
    }
    fn volume(&self) -> u16 {
        (self.0.level() * u16::MAX as f64).round() as u16
    }
    fn set_volume(&self, volume: u16) {
        self.0.set_level(volume as f64 / u16::MAX as f64);
    }
    // Player uses the SharedVolume directly, so the default NoOpVolume soft
    // getter here is never used for actual attenuation.
}

// silent fallback sink

// just throws away all audio. used when opening the real output device PANICS (e.g a
// headless/busted linux box with no alsa/pulse device, or a host with a
// broken audio stack). without this that panic kills the librespot player thread
// mid stream, with it playback runs silently and the app stays usable instead
// of looking like it froze
//
// note: only catches unwinding failures. a hard segfault inside a system audio
// framework (apples coreaudio hal on some virtualized macs) is a SIGSEGV and
// cant be caught from rust, thats an environment fault not something this
// guard can do anything about
struct NullSink;

impl Sink for NullSink {
    fn write(&mut self, packet: AudioPacket, _converter: &mut Converter) -> SinkResult<()> {
        // throw the audio away BUT pace at real time. librespot's player thread
        // calls write() as fast as we return, so returning instantly makes the
        // decoder race through the whole track in a few ms -> EndOfTrack fires
        // almost immediately -> the frontend auto-advances (App.tsx end_of_track)
        // -> EVERY track in the queue "insta-skips". sleeping for the packet's real
        // duration makes silent playback advance at 1x, same backpressure the real
        // rodio sink applies. (only matters when this fallback is actually in use;
        // on a healthy device the real sink is used instead.)
        if let Ok(samples) = packet.samples() {
            let frames = samples.len() / NUM_CHANNELS as usize;
            if frames > 0 {
                let secs = frames as f64 / SAMPLE_RATE as f64;
                std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            }
        }
        Ok(())
    }
}

// real output sink - opens the default device at ITS OWN native sample rate.
//
// this is deliberately NOT librespot's built-in rodio backend. that backend
// forces a 44100 Hz stream, and cpal 0.16's macOS set_sample_rate() has a bug:
// when the requested rate differs from the device's CURRENT rate (a MacBook's
// built-in output defaults to 48000) it reads the AvailableNominalSampleRates
// size through a non-`mut` binding the optimizer assumes stays 0, then hands
// CoreAudio a 0-length Vec<u8> whose dangling pointer is 0x1 -> AudioObject-
// GetPropertyData writes to 0x1 -> UNCATCHABLE SIGSEGV in HALC_ProxyIOContext::
// GetPropertyData (crash FA686AC4, "no audio + crash on real M3/macOS 26").
// Opening at the device's OWN rate means current==target, so cpal early-returns
// and never runs that buggy branch. rodio resamples our 44100 source up to the
// device's rate. Works on Windows/Linux too (native rate is always fine there).


// macos only: are we running inside a hypervisor (a vm)?
//
// apples paravirtualized coreaudio hal (AppleParavirtGPU / VirtualMac2,1)
// segfaults inside AudioObjectGetPropertyData the second cpal opens the default
// output device. thats a SIGSEGV in a system framework, rusts catch_unwind
// cant catch it so it took the whole app down a few secs after launch
// ("crashes a lot on startup on macos", see the crash log: Thread 30 in
// HALC_ProxyIOContext::GetPropertyData). theres no real audio device to open in
// that env anyway so when we spot a vm we skip the real backend
// entirely and run the silent sink, playback "works" (silently) and the app
// never crashes. on real mac hardware kern.hv_vmm_present is 0 and the real
// audio backend is used like normal
#[cfg(target_os = "macos")]
fn running_under_hypervisor() -> bool {
    let name = match std::ffi::CString::new("kern.hv_vmm_present") {
        Ok(n) => n,
        Err(_) => return false,
    };
    let mut val: i32 = 0;
    let mut size = std::mem::size_of::<i32>();
    let rc = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            &mut val as *mut _ as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    rc == 0 && val != 0
}

// is opening the real audio output device even safe on this machine?
//
// CONFIRMED (real M3 MacBook, macOS 26.5.2, crash FA686AC4): cpal's
// AudioObjectGetPropertyData SIGSEGVs inside CoreAudio's HAL proxy
// (HALC_ProxyIOContext::GetPropertyData) the instant the default output device is
// opened - it writes through a dangling/near-null pointer (fault addr 0x1). This
// is a hard NATIVE fault in a system framework: catch_unwind CANNOT catch it, so
// if the MAIN app opens the device it takes the whole process down on first play
// (crash thread 28, a librespot player worker). It faults on ANY thread, in ANY
// process, so a run loop doesn't help.
//
// So we test the open in a THROWAWAY child process (`--audio-probe`) first and
// cache the verdict for this run. The child installs signal handlers that turn
// the fault into a clean _exit (see `audio_probe` in lib.rs), so: (a) it never
// pops a "quit unexpectedly" crash dialog, and (b) its nonzero exit tells us the
// device is unsafe -> the main app uses the (real-time-paced) NullSink and NEVER
// touches CoreAudio, staying alive and usable. On a healthy machine the child
// exits 0 and we use the real backend like normal.
//
// macOS ONLY. On Windows and Linux there is nothing to probe for: our own
// `RodioSink` opens the device lazily and a failure comes back as a clean
// `SinkError::ConnectionRefused` (see sink.rs), never a panic and never a
// native fault. Running the probe there just bolted a full process spawn -
// loading the whole app binary again, opening and closing the audio device,
// waiting for it to exit - onto the front of the FIRST PLAY, for a verdict that
// was always `true`. The uncatchable-SIGSEGV problem this guards against is
// specific to cpal + CoreAudio's HAL proxy, so the probe now runs only where
// that fault can actually happen.
#[cfg(target_os = "macos")]
fn audio_device_safe() -> bool {
    static SAFE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SAFE.get_or_init(|| {
        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => { eprintln!("[playback] current_exe failed: {e} - assuming audio ok"); return true; }
        };
        match std::process::Command::new(exe).arg("--audio-probe").status() {
            Ok(s) if s.success() => true,
            Ok(s) => { eprintln!("[playback] audio device probe failed ({s}) - using silent sink"); false }
            Err(e) => { eprintln!("[playback] audio probe spawn error: {e} - assuming audio ok"); true }
        }
    })
}

// playbackinner

pub struct PlaybackInner {
    pub player:            Arc<Player>,
    pub volume:            SharedVolume,
    pub loaded:            Arc<AtomicBool>,
    pub current_uri:       Arc<Mutex<Option<String>>>,
    pub is_ended:          Arc<AtomicBool>,
    pub is_playing_atomic: Arc<AtomicBool>,
    pub needs_rebuild:     Arc<AtomicBool>,
    pub output_latency:    Arc<AtomicI64>,
    spirc:                 Spirc,
    session:               Session,
    _event_task:           tauri::async_runtime::JoinHandle<()>,
    _spirc_task:           tauri::async_runtime::JoinHandle<()>,
    _dealer_task:          tauri::async_runtime::JoinHandle<()>,
    _state_task:           tauri::async_runtime::JoinHandle<()>,
}

// map a librespot control error into our IPC error type
fn spirc_err(e: LibrespotError) -> AppError {
    AppError::Playback(e.to_string())
}

impl PlaybackInner {
    /// Audio output latency in milliseconds (queued frames in sink + device buffer).
    /// Returns 0 for the null/silent fallback sink or if playback has not started.
    pub fn output_latency_ms(&self) -> i64 {
        self.output_latency.load(Ordering::Relaxed)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing_atomic.load(Ordering::Relaxed)
    }

    /// Tear the librespot session down for real.
    ///
    /// Dropping `PlaybackInner` is not enough. Both background tasks were
    /// spawned onto the runtime and dropping a `JoinHandle` only *detaches* the
    /// task, so after a logout the previous account's Connect device stayed
    /// registered, kept answering remote commands, and kept rewriting its
    /// credentials cache - which then got picked up as "cached credentials" on
    /// the next sign-in and put the old account straight back.
    pub async fn shutdown(&self) {
        if let Err(e) = self.spirc.shutdown() {
            eprintln!("[playback] spirc shutdown request failed: {e}");
        }
        self.player.stop();
        // give spirc a moment to send its goodbye and unregister the device
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        self._spirc_task.abort();
        self._event_task.abort();
        self._dealer_task.abort();
        self._state_task.abort();
    }

    pub fn is_ended(&self) -> bool {
        self.is_ended.load(Ordering::Relaxed)
    }

    pub fn current_uri(&self) -> Option<String> {
        self.current_uri.lock().unwrap().clone()
    }

    // a dropped/expired librespot session just eats load/play calls so
    // playback shows as "playing" with no actual audio. callers rebuild when this is
    // true (see ensure_inner)
    pub fn session_invalid(&self) -> bool {
        self.session.is_invalid()
    }

    // hand out the live librespot session so other subsystems (lyrics) can call
    // spclient endpoints with our real first-party identity. `Session` is an
    // Arc handle, so this clone is cheap and lets callers drop the state mutex
    // before doing network i/o.
    pub fn session(&self) -> Session {
        self.session.clone()
    }

    pub fn play_uri(&self, uri: String, position_ms: u32) -> Result<(), AppError> {
        eprintln!("[playback] play_uri uri={uri} pos={position_ms}");
        self.spirc.activate().map_err(spirc_err)?;
        self.spirc.load(LoadRequest::from_tracks(
            vec![uri.clone()],
            LoadRequestOptions {
                start_playing: true,
                seek_to: position_ms,
                ..Default::default()
            },
        )).map_err(spirc_err)?;
        *self.current_uri.lock().unwrap() = Some(uri);
        self.loaded.store(true, Ordering::Relaxed);
        self.is_ended.store(false, Ordering::Relaxed);
        self.is_playing_atomic.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn resume(&self) -> Result<(), AppError> {
        eprintln!("[playback] resume");
        self.spirc.play().map_err(spirc_err)?;
        self.is_playing_atomic.store(true, Ordering::Relaxed);
        self.is_ended.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn pause(&self) -> Result<(), AppError> {
        eprintln!("[playback] pause");
        self.spirc.pause().map_err(spirc_err)?;
        self.is_playing_atomic.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn seek(&self, position_ms: u32) -> Result<(), AppError> {
        eprintln!("[playback] seek pos={position_ms}");
        self.spirc.set_position_ms(position_ms).map_err(spirc_err)?;
        Ok(())
    }

    // push a volume change (0.0..=1.0) into Spirc so the connect-state it reports
    // matches. best-effort: a failure here never blocks the local volume change.
    pub fn report_volume(&self, level: f64) {
        let v = (level.clamp(0.0, 1.0) * u16::MAX as f64).round() as u16;
        let _ = self.spirc.set_volume(v);
    }

    // ── connect-driven playback (jam) ────────────────────────────────────────
    // in a jam the queue is spotify's, held by spirc and edited by everyone in
    // it. these drive spirc's own queue instead of the app's.

    /// what this device last reported to connect: the track, the queue, the
    /// context. in a jam that's the jam
    pub fn connect_state(&self) -> ConnectStateMsg {
        ConnectStateMsg::from(&*self.spirc.state_updates().borrow())
    }

    /// load a list as the context, starting at `index`. with `keep_stream`,
    /// the track already playing at that spot carries on instead of restarting
    pub fn load_tracks(&self, uris: Vec<String>, index: u32, position_ms: u32, start_playing: bool, keep_stream: bool) -> Result<(), AppError> {
        self.spirc.activate().map_err(spirc_err)?;
        let current = uris.get(index as usize).cloned();
        let request = LoadRequest::from_tracks(
            uris,
            LoadRequestOptions {
                start_playing,
                seek_to: position_ms,
                playing_track: Some(PlayingTrack::Index(index)),
                ..Default::default()
            },
        );
        if keep_stream {
            self.spirc.load_keep_stream(request).map_err(spirc_err)?;
        } else {
            self.spirc.load(request).map_err(spirc_err)?;
        }
        if let Some(uri) = current {
            *self.current_uri.lock().unwrap() = Some(uri);
        }
        self.loaded.store(true, Ordering::Relaxed);
        self.is_ended.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn add_to_queue(&self, uri: String) -> Result<(), AppError> {
        self.spirc.add_to_queue(uri).map_err(spirc_err)
    }

    pub fn skip_to(&self, uri: String) -> Result<(), AppError> {
        self.spirc.skip_to(uri).map_err(spirc_err)
    }

    pub fn next(&self) -> Result<(), AppError> {
        self.spirc.next().map_err(spirc_err)
    }

    pub fn prev(&self) -> Result<(), AppError> {
        self.spirc.prev().map_err(spirc_err)
    }

    pub fn set_shuffle(&self, on: bool) -> Result<(), AppError> {
        self.spirc.shuffle(on).map_err(spirc_err)
    }

    pub fn set_repeat(&self, context: bool, track: bool) -> Result<(), AppError> {
        self.spirc.repeat(context).map_err(spirc_err)?;
        self.spirc.repeat_track(track).map_err(spirc_err)
    }

    /// a jam guest pausing just for themselves, or rejoining where the jam is
    pub fn set_jam_hold(&self, hold: bool) -> Result<(), AppError> {
        self.spirc.set_jam_hold(hold).map_err(spirc_err)?;
        self.is_playing_atomic.store(!hold, Ordering::Relaxed);
        Ok(())
    }
}

/// one row of spirc's queue, as the frontend shows it
#[derive(Debug, Clone, Serialize)]
pub struct QueueEntry {
    pub uri:       String,
    pub uid:       String,
    /// "context", "queue" or "autoplay"
    pub provider:  String,
    /// in a jam, the username of whoever added it
    pub queued_by: Option<String>,
}

impl QueueEntry {
    fn from_track(t: &librespot_protocol::player::ProvidedTrack) -> Option<Self> {
        let hidden = t.metadata.get("hidden").is_some_and(|v| v == "true");
        if t.uri.is_empty() || t.uri == "spotify:delimiter" || hidden {
            return None;
        }
        Some(QueueEntry {
            uri:       t.uri.clone(),
            uid:       t.uid.clone(),
            provider:  t.provider.clone(),
            queued_by: t.metadata.get("queued_by").filter(|v| !v.is_empty()).cloned(),
        })
    }
}

/// spirc's reported connect state, trimmed to what the ui mirrors
#[derive(Debug, Clone, Serialize)]
pub struct ConnectStateMsg {
    pub active:        bool,
    pub context_uri:   String,
    pub track:         Option<QueueEntry>,
    pub next:          Vec<QueueEntry>,
    pub is_playing:    bool,
    pub is_paused:     bool,
    pub position_ms:   i64,
    pub timestamp:     i64,
    pub duration_ms:   i64,
    pub shuffle:       bool,
    pub repeat_context: bool,
    pub repeat_track:  bool,
    /// social-connect switched this device into jam mode
    pub jam_mode:      bool,
}

impl From<&librespot_connect::ConnectSnapshot> for ConnectStateMsg {
    fn from(s: &librespot_connect::ConnectSnapshot) -> Self {
        let p = &s.player;
        let options = p.options.as_ref();
        ConnectStateMsg {
            active:         s.active,
            context_uri:    p.context_uri.clone(),
            track:          p.track.as_ref().and_then(QueueEntry::from_track),
            next:           p.next_tracks.iter().filter_map(QueueEntry::from_track).collect(),
            is_playing:     p.is_playing,
            is_paused:      p.is_paused,
            position_ms:    p.position_as_of_timestamp,
            timestamp:      p.timestamp,
            duration_ms:    p.duration,
            shuffle:        options.is_some_and(|o| o.shuffling_context),
            repeat_context: options.is_some_and(|o| o.repeating_context),
            repeat_track:   options.is_some_and(|o| o.repeating_track),
            jam_mode:       options.is_some_and(|o| o.modes.get("jam").is_some_and(|v| v == "on")),
        }
    }
}

// event messages n stuff

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlayerMsg {
    Playing {
        track_id:    Option<String>,
        position_ms: u32,
    },
    Paused {
        track_id:    Option<String>,
        position_ms: u32,
    },
    PositionChanged {
        track_id:    Option<String>,
        position_ms: u32,
    },
    Stopped {
        track_id: Option<String>,
    },
    EndOfTrack {
        track_id: Option<String>,
    },
    Unavailable {
        track_id: Option<String>,
    },
    TimeToPreloadNextTrack {
        track_id: Option<String>,
    },
}

// session / player init stuff

pub async fn create_inner(
    app:            AppHandle,
    pool:           SqlitePool,
    auth_state:     Arc<RwLock<AuthState>>,
    initial_volume: f64,
    initial_muted:  bool,
    media_tx:       std::sync::mpsc::SyncSender<crate::media_controls::MediaMsg>,
    // false for background callers (the startup warm-up): they must never open
    // a browser authorization tab the user didn't ask for. When no silent
    // credential works they fail instead and the next real play recovers.
    interactive:    bool,
) -> Result<PlaybackInner, AppError> {
    let _ = auth::get_valid_token(&pool, &auth_state).await
        .map_err(|e| { eprintln!("[playback] auth token error: {e}"); e })?;

    let app_data = app.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let creds_dir = app_data.join("credentials");
    let vol_dir = app_data.join("volume");
    let cache_dir = app_data.join("audio_cache");
    let _ = std::fs::create_dir_all(&creds_dir);
    let _ = std::fs::create_dir_all(&vol_dir);
    let _ = std::fs::create_dir_all(&cache_dir);

    let creds_file = creds_dir.join("credentials.json");

    let cache_limit_mb: u64 = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM settings WHERE key = 'audio_cache_limit_mb'",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten()
    .and_then(|(v,)| v.parse::<u64>().ok())
    .unwrap_or(2048);

    let cache_bytes = cache_limit_mb.saturating_mul(1024 * 1024);
    eprintln!("[playback] configured cache limit = {cache_limit_mb} MB");

    let open_cache = || {
        Cache::new(
            Some(&creds_dir),
            Some(&vol_dir),
            Some(&cache_dir),
            Some(cache_bytes),
        ).ok()
    };

    let cache = open_cache();
    let cached_creds = cache.as_ref().and_then(|c| c.credentials());
    let used_cached_creds = cached_creds.is_some();
    let credentials = match cached_creds {
        Some(c) => {
            eprintln!("[playback] using cached librespot credentials from disk");
            c
        }
        None => {
            let playback_token = match auth::get_setting_value(&pool, "spotify_playback_token").await? {
                Some(t) if !t.trim().is_empty() => t,
                _ if !interactive => {
                    return Err(AppError::Auth(
                        "no playback credentials; skipping non-interactive session build".into(),
                    ));
                }
                _ => crate::commands::auth::authorize_playback_token(&app).await?,
            };
            Credentials::with_access_token(&playback_token)
        }
    };

    let device_name = auth::get_setting_value(&pool, "device_name")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| auth::DEFAULT_DEVICE_NAME.to_string());
    let device_id = auth::compute_device_id(&device_name);

    let session_config = SessionConfig {
        device_id,
        autoplay: Some(false),
        ..SessionConfig::default()
    };
    let mut session = Session::new(session_config.clone(), cache);
    eprintln!("[playback] librespot session client_id = {}", session.client_id());

    let volume    = SharedVolume::new(initial_volume, initial_muted);
    let vol_clone = volume.clone();

    let output_latency = Arc::new(AtomicI64::new(0));
    let latency_sink = Arc::clone(&output_latency);

    #[cfg(target_os = "macos")]
    let force_null_sink = running_under_hypervisor() || !audio_device_safe();
    #[cfg(not(target_os = "macos"))]
    let force_null_sink = false;
    if force_null_sink {
        eprintln!("[playback] audio device unavailable/unsafe - using silent sink");
    }

    let make_sink = move || {
        if force_null_sink {
            return Box::new(NullSink) as Box<dyn Sink>;
        }
        eprintln!("[playback] STEP opening audio device via RodioSink");
        let on_err: crate::sink::ErrorHook = Arc::new(|msg| {
            eprintln!("[playback error] {msg}");
        });
        Box::new(crate::sink::RodioSink::new(
            None,
            on_err,
            Box::new(vol_clone),
            crate::sink::DEFAULT_BUFFER_MS,
        ).with_latency_tracker(Arc::clone(&latency_sink))) as Box<dyn Sink>
    };

    let bitrate_setting: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'audio_quality'",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let bitrate = match bitrate_setting.as_ref().map(|(v,)| v.as_str()) {
        Some("96") => Bitrate::Bitrate96,
        Some("160") => Bitrate::Bitrate160,
        _ => Bitrate::Bitrate320,
    };
    eprintln!("[playback] configured bitrate = {bitrate:?}");

    let player_config = PlayerConfig {
        bitrate,
        gapless: true,
        ..Default::default()
    };
    let player = Player::new(
        player_config,
        session.clone(),
        Box::new(librespot_playback::mixer::NoOpVolume),
        make_sink,
    );
    eprintln!("[playback] STEP player built");

    let mixer: Arc<dyn Mixer> = Arc::new(SharedMixer(volume.clone()));
    let connect_config = ConnectConfig {
        name:           device_name.clone(),
        device_type:    DeviceType::Computer,
        initial_volume: (initial_volume.clamp(0.0, 1.0) * u16::MAX as f64).round() as u16,
        ..Default::default()
    };

    let mut spirc_attempt = Spirc::new(
        connect_config.clone(),
        session.clone(),
        credentials.clone(),
        player.clone(),
        mixer.clone(),
    )
    .await;

    if let Err(ref e) = spirc_attempt {
        let err_msg = e.to_string();
        eprintln!("[playback] spirc connect failed: {err_msg}");
        if err_msg.contains("INVALID_CREDENTIALS") || err_msg.contains("Login request was denied") {
            eprintln!("[playback] cached credentials rejected; deleting bad credentials file");
            let _ = std::fs::remove_file(&creds_file);

            // Recover in order of how much it costs the user. A credentials
            // file left behind by a previous account is the common case, and
            // the playback token we already hold usually fixes it silently -
            // going straight to the browser opened a surprise authorization tab
            // (and, when a sign-in was already in flight, one that could not
            // even bind its port).
            let stored_token = if used_cached_creds {
                auth::get_setting_value(&pool, "spotify_playback_token")
                    .await
                    .ok()
                    .flatten()
                    .filter(|t| !t.trim().is_empty())
            } else {
                None
            };

            enum Recovery {
                Stored(String),
                Interactive,
            }

            let mut attempts = Vec::new();
            if let Some(token) = stored_token {
                attempts.push(Recovery::Stored(token));
            }
            if interactive {
                attempts.push(Recovery::Interactive);
            }

            for attempt in attempts {
                let (label, token) = match attempt {
                    Recovery::Stored(token) => ("stored playback token", token),
                    Recovery::Interactive => {
                        eprintln!("[playback] initiating playback authorization flow");
                        match crate::commands::auth::authorize_playback_token(&app).await {
                            Ok(token) => ("browser authorization", token),
                            Err(e) => {
                                eprintln!("[playback] playback authorization failed: {e}");
                                break;
                            }
                        }
                    }
                };

                eprintln!("[playback] retrying spirc with {label}");
                let fresh_cache = open_cache();
                let fresh_session = Session::new(session_config.clone(), fresh_cache);
                spirc_attempt = Spirc::new(
                    connect_config.clone(),
                    fresh_session.clone(),
                    Credentials::with_access_token(&token),
                    player.clone(),
                    mixer.clone(),
                )
                .await;
                if spirc_attempt.is_ok() {
                    session = fresh_session;
                    break;
                }

                if label == "stored playback token" {
                    // It is spent; stop offering it to every later attempt.
                    eprintln!("[playback] stored playback token rejected; discarding it");
                    let _ = sqlx::query("DELETE FROM settings WHERE key = 'spotify_playback_token'")
                        .execute(&pool)
                        .await;
                    let _ = std::fs::remove_file(&creds_file);
                }
            }
        }
    }

    let (spirc, spirc_task) = spirc_attempt.map_err(|e| {
        eprintln!("[playback] spirc connect finally failed: {e}");
        AppError::Auth(e.to_string())
    })?;
    eprintln!("[playback] STEP spirc connected (Connect device 'Musique' registered)");
    let spirc_task = tauri::async_runtime::spawn(spirc_task);
    // spirc has the dealer connected now; tap it for device/jam/playlist pushes
    let dealer_task = crate::internal::dealer::spawn(app.clone(), session.clone());

    // mirror every state spirc reports. in a jam spotify drives this device
    // (social-connect transfers the jam here, skips, adds to its queue) and
    // this is the only place the ui can see what that queue now is
    let mut state_rx = spirc.state_updates();
    let state_app = app.clone();
    let state_task = tauri::async_runtime::spawn(async move {
        while state_rx.changed().await.is_ok() {
            let msg = ConnectStateMsg::from(&*state_rx.borrow_and_update());
            let _ = state_app.emit("connect:state", msg);
            // spirc reports in bursts (a jam edit is a transfer, then a
            // resolve, then an update); the last one is the one that matters
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
    });

    // the access point pushes CountryCode (and ProductInfo) as SEPARATE packets
    // that show up AFTER the session connects (Spirc::new above did the connect).
    // librespots availability filter (available_for_user) checks each tracks
    // allowed-countries whitelist against session.country() and while thats still
    // empty EVERY track gets rejected as NotWhitelisted -> PlayerEvent::Unavailable
    // ("content may not be available in your region"). a play fired right after a
    // fresh connect races those packets, so wait a sec for the country to land.
    // Poll finely. The country packet usually lands within a few ms of the
    // connect, but at 100ms granularity we slept out the rest of the tick every
    // time and added up to ~100ms of dead wait to every session build (and so to
    // the first play). Same 5s ceiling, 20x finer resolution.
    for _ in 0..1000 {
        if !session.country().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    eprintln!("[playback] STEP country = {:?}", session.country());

    let current_uri       = Arc::new(Mutex::new(None));
    let is_ended          = Arc::new(AtomicBool::new(false));
    let is_playing_atomic = Arc::new(AtomicBool::new(false));

    let is_ended_clone          = is_ended.clone();
    let is_playing_atomic_clone = is_playing_atomic.clone();

    let mut event_rx = player.get_player_event_channel();
    let event_app    = app.clone();
    let event_task   = tauri::async_runtime::spawn(async move {
        // coalesce PositionChanged: librespot can fire it very frequently, and every
        // one becomes a JSON-serialized IPC message + a React state update. We forward
        // at most ~4x/sec (or immediately on a real jump like a seek). The frontend
        // interpolates position between updates, so playback stays smooth.
        let mut last_pos_emit = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap_or_else(std::time::Instant::now);
        let mut last_pos_ms: u32 = 0;
        while let Some(event) = event_rx.recv().await {
            log::trace!("[playback event] {event:?}");
            // pass the playback state along to the os media controls
            match &event {
                PlayerEvent::Playing { position_ms, .. } => {
                    is_playing_atomic_clone.store(true, Ordering::Relaxed);
                    is_ended_clone.store(false, Ordering::Relaxed);
                    let _ = media_tx.try_send(
                        crate::media_controls::MediaMsg::Playing { position_ms: *position_ms as u64 }
                    );
                }
                PlayerEvent::Paused { position_ms, .. } => {
                    is_playing_atomic_clone.store(false, Ordering::Relaxed);
                    let _ = media_tx.try_send(
                        crate::media_controls::MediaMsg::Paused { position_ms: *position_ms as u64 }
                    );
                }
                PlayerEvent::Stopped { .. }
                | PlayerEvent::EndOfTrack { .. }
                | PlayerEvent::Unavailable { .. } => {
                    is_playing_atomic_clone.store(false, Ordering::Relaxed);
                    is_ended_clone.store(true, Ordering::Relaxed);
                    let _ = media_tx.try_send(crate::media_controls::MediaMsg::Stopped);
                }
                _ => {}
            }

            let msg = match event {
                PlayerEvent::Playing { track_id, position_ms, .. } =>
                    Some(PlayerMsg::Playing {
                        track_id:    track_id.to_id().ok(),
                        position_ms,
                    }),
                PlayerEvent::Paused { track_id, position_ms, .. } =>
                    Some(PlayerMsg::Paused {
                        track_id:    track_id.to_id().ok(),
                        position_ms,
                    }),
                PlayerEvent::PositionChanged { track_id, position_ms, .. } => {
                    let due   = last_pos_emit.elapsed() >= std::time::Duration::from_millis(1000);
                    let moved = position_ms.abs_diff(last_pos_ms) >= 1000;
                    if due || moved {
                        last_pos_emit = std::time::Instant::now();
                        last_pos_ms   = position_ms;
                        Some(PlayerMsg::PositionChanged {
                            track_id:    track_id.to_id().ok(),
                            position_ms,
                        })
                    } else {
                        None
                    }
                }
                PlayerEvent::Stopped { track_id, .. } =>
                    Some(PlayerMsg::Stopped { track_id: track_id.to_id().ok() }),
                PlayerEvent::EndOfTrack { track_id, .. } =>
                    Some(PlayerMsg::EndOfTrack { track_id: track_id.to_id().ok() }),
                PlayerEvent::Unavailable { track_id, .. } =>
                    Some(PlayerMsg::Unavailable { track_id: track_id.to_id().ok() }),
                PlayerEvent::TimeToPreloadNextTrack { track_id, .. } =>
                    Some(PlayerMsg::TimeToPreloadNextTrack { track_id: track_id.to_id().ok() }),
                _ => None,
            };

            if let Some(payload) = msg {
                if let Err(e) = event_app.emit("player:event", payload) {
                    eprintln!("[playback] emit failed: {e}");
                }
            }
        }
    });

    Ok(PlaybackInner {
        player,
        volume,
        loaded:            Arc::new(AtomicBool::new(false)),
        current_uri,
        is_ended,
        is_playing_atomic,
        needs_rebuild:     Arc::new(AtomicBool::new(false)),
        output_latency,
        spirc,
        session,
        _event_task:       event_task,
        _dealer_task:      dealer_task,
        _spirc_task:       spirc_task,
        _state_task:       state_task,
    })
}

// track id parsing stuff

pub fn parse_track_id(raw: &str) -> Result<SpotifyUri, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("Track id is required".into()));
    }

    // podcast episodes ride the same load path as tracks - spirc and the
    // player both take episode uris as-is
    if let Some(v) = trimmed.strip_prefix("spotify:episode:") {
        let spotify_id = SpotifyId::from_base62(v)
            .map_err(|_| AppError::InvalidInput(format!("Invalid Spotify episode id: {trimmed}")))?;
        return Ok(SpotifyUri::Episode { id: spotify_id });
    }

    let id = if let Some(v) = trimmed.strip_prefix("spotify:track:") {
        v
    } else if let Some((_, rest)) = trimmed.split_once("open.spotify.com/track/") {
        rest.split('?').next().unwrap_or(rest)
    } else {
        trimmed
    };

    let spotify_id = SpotifyId::from_base62(id)
        .map_err(|_| AppError::InvalidInput(format!("Invalid Spotify track id: {trimmed}")))?;
    Ok(SpotifyUri::Track { id: spotify_id })
}

// canonical "spotify:track:<base62>" string for Spirc load commands (which take
// URIs, not SpotifyUri). validates the id the same way parse_track_id does.
pub fn track_uri(raw: &str) -> Result<String, AppError> {
    parse_track_id(raw)?
        .to_uri()
        .map_err(|e| AppError::InvalidInput(e.to_string()))
}
