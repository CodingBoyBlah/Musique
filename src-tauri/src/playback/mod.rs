use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};

use librespot_connect::{ConnectConfig, LoadRequest, LoadRequestOptions, Spirc};
use librespot_core::{
    authentication::Credentials,
    config::{DeviceType, SessionConfig},
    session::Session,
    Error as LibrespotError,
    SpotifyId,
    SpotifyUri,
};
use librespot_playback::{
    audio_backend::{Sink, SinkError, SinkResult},
    config::{Bitrate, PlayerConfig},
    convert::Converter,
    decoder::AudioPacket,
    mixer::{Mixer, MixerConfig, VolumeGetter},
    player::{Player, PlayerEvent},
    NUM_CHANNELS, SAMPLE_RATE,
};
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};
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
pub(crate) struct DeviceSink {
    sink:    rodio::Sink,
    _stream: rodio::OutputStream,
}

impl DeviceSink {
    pub(crate) fn open() -> Result<DeviceSink, String> {
        let mut stream = rodio::OutputStreamBuilder::from_default_device()
            .map_err(|e| e.to_string())?
            // open_stream() uses ONLY the device's native config. we intentionally
            // do NOT use open_stream_or_fallback(): its fallback would try other
            // sample rates and could re-trigger the cpal set_sample_rate() crash.
            .open_stream()
            .map_err(|e| e.to_string())?;
        stream.log_on_drop(false);
        let sink = rodio::Sink::connect_new(stream.mixer());
        Ok(DeviceSink { sink, _stream: stream })
    }
}

impl Sink for DeviceSink {
    fn start(&mut self) -> SinkResult<()> {
        self.sink.play();
        Ok(())
    }

    fn stop(&mut self) -> SinkResult<()> {
        self.sink.sleep_until_end();
        self.sink.pause();
        Ok(())
    }

    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let samples = packet.samples().map_err(|e| SinkError::OnWrite(e.to_string()))?;
        let samples_f32: &[f32] = &converter.f64_to_f32(samples);
        // 44100 source; rodio's mixer resamples to the device's native rate
        let source = rodio::buffer::SamplesBuffer::new(NUM_CHANNELS as u16, SAMPLE_RATE, samples_f32);
        self.sink.append(source);
        // backpressure: keep at most ~0.5s queued so playback tracks real time
        // instead of buffering the whole track at once (mirrors librespot's rodio)
        while self.sink.len() > 26 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }
}

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
fn audio_device_safe() -> bool {
    static SAFE: OnceLock<bool> = OnceLock::new();
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
    pub player:  Arc<Player>,
    pub volume:  SharedVolume,
    // whether a track has ever been loaded into this session. a freshly warmed
    // session has a live player but nothing loaded so player.play() would do
    // nothing, callers gotta load first. set to true on every load
    pub loaded:  Arc<AtomicBool>,
    // Spotify Connect controller. ALL playback control (load/play/pause/seek/
    // volume) goes through this instead of straight to `player` so that every
    // action is reflected in the connect-state Spirc PUTs to Spotify's backend
    // - that's what makes plays land in "recently played" and shows Musique as
    // the active device. `player` is still held for preload() (pure audio
    // pre-cache, no state change) and for the event channel.
    spirc:       Spirc,
    session:     Session,
    _event_task: tauri::async_runtime::JoinHandle<()>,
    // the Spirc state-machine future. dropping the handle aborts it, tearing the
    // connect device down, so it must live exactly as long as this inner.
    _spirc_task: tauri::async_runtime::JoinHandle<()>,
}

// map a librespot control error into our IPC error type
fn spirc_err(e: LibrespotError) -> AppError {
    AppError::Playback(e.to_string())
}

impl PlaybackInner {
    // a dropped/expired librespot session just eats load/play calls so
    // playback shows as "playing" with no actual audio. callers rebuild when this is
    // true (see ensure_inner)
    pub fn session_invalid(&self) -> bool {
        self.session.is_invalid()
    }

    // load + start a single track through Spotify Connect. `activate()` MUST come
    // first: Spirc ignores load/play/etc while the device is inactive, and since
    // both commands ride the same ordered channel the activate is processed first.
    // We load a one-track ad-hoc context (not a real album/playlist context) so
    // the frontend stays the source of truth for the queue: at end-of-track Spirc
    // finds no next track and stops, and the frontend advances by calling this
    // again for the next id. Each load registers as a play on Spotify's backend.
    pub fn play_uri(&self, uri: String, position_ms: u32) -> Result<(), AppError> {
        self.spirc.activate().map_err(spirc_err)?;
        self.spirc
            .load(LoadRequest::from_tracks(
                vec![uri],
                LoadRequestOptions {
                    start_playing: true,
                    seek_to: position_ms,
                    ..Default::default()
                },
            ))
            .map_err(spirc_err)?;
        self.loaded.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn resume(&self) -> Result<(), AppError> {
        self.spirc.play().map_err(spirc_err)
    }

    pub fn pause(&self) -> Result<(), AppError> {
        self.spirc.pause().map_err(spirc_err)
    }

    pub fn seek(&self, position_ms: u32) -> Result<(), AppError> {
        self.spirc.set_position_ms(position_ms).map_err(spirc_err)
    }

    // push a volume change (0.0..=1.0) into Spirc so the connect-state it reports
    // matches. best-effort: a failure here never blocks the local volume change.
    pub fn report_volume(&self, level: f64) {
        let v = (level.clamp(0.0, 1.0) * u16::MAX as f64).round() as u16;
        let _ = self.spirc.set_volume(v);
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
) -> Result<PlaybackInner, AppError> {
    let token = auth::get_valid_token(&pool, &auth_state).await
        .map_err(|e| { eprintln!("[playback] token error: {e}"); e })?;

    // librespots SESSION uses its built in default client_id. spotifys
    // streaming pipeline (clienttoken -> login5 -> extended-metadata ->
    // storage-resolve) is locked to first party client identities, a users own
    // web api "developer" client_id gets rejected by clienttoken.spotify.com
    // with HTTP 400 so login5 (which NEEDS a client token) fails and EVERY
    // track comes back Unavailable, basically forcing the users own client_id into
    // the session breaks playback completely. the users own client_id is still
    // what mints the oauth token below (web api + auth), only the librespot
    // streaming session uses the default. this matches the config that streamed
    // fine before the (reverted) "use the users client_id" change. see
    // vendor/librespot-audio for the matching cdn-fallthrough fix
    // (learned this the hard way, do NOT touch this lol)
    // client_id stays the librespot default (keymaster) - see the big comment
    // above; only `autoplay` is overridden. With Spirc driving playback we load a
    // one-track context per play and let the FRONTEND own queue advancement. If
    // autoplay were on, Spirc would resolve a recommended-radio "next" track for
    // the synthetic context and start playing it at end-of-track, fighting the
    // frontend (which also advances on EndOfTrack). Forcing autoplay off makes
    // Spirc simply stop at track end, so the frontend stays the single source of
    // truth for what plays next.
    let session = Session::new(
        SessionConfig { autoplay: Some(false), ..SessionConfig::default() },
        None,
    );
    eprintln!("[playback] librespot session client_id = {}", session.client_id());
    // NOTE: we deliberately do NOT session.connect() here anymore. Spirc::new()
    // below registers its dealer listeners (connection-id / cluster / player
    // commands) FIRST and then connects the session itself - that ordering is
    // required for the connect-state machinery, and connecting here too would be
    // redundant and could race those listeners. The client_id stays the librespot
    // default (see the big comment above): Spirc reuses THIS session so the
    // connect device is registered under the same first-party identity that the
    // streaming pipeline needs.

    let volume    = SharedVolume::new(initial_volume, initial_muted);
    let vol_clone = volume.clone();


    // decide up front whether to even touch the real audio device. two guards,
    // both because opening cpal's device can HARD-FAULT (SIGSEGV in a system
    // framework) which catch_unwind cannot catch:
    //  1) macos-in-a-vm: Apple's paravirt CoreAudio HAL segfaults on open (sysctl)
    //  2) any os: a throwaway child process (`--audio-probe`) actually opens the
    //     device; if it can't without faulting, we fall back to silence here
    //     instead of taking the whole app down. CONFIRMED needed on real M3 Mac
    //     hardware / macOS 26 where the open SIGSEGVs even in the main process.
    // When forced, the (real-time-paced) NullSink keeps the app alive + usable
    // (silent) and playback advances normally instead of insta-skipping.
    #[cfg(target_os = "macos")]
    let force_null_sink = running_under_hypervisor() || !audio_device_safe();
    #[cfg(not(target_os = "macos"))]
    let force_null_sink = !audio_device_safe();
    if force_null_sink {
        eprintln!("[playback] audio device unavailable/unsafe - using silent sink");
    }

    let player = Player::new(
        PlayerConfig {
            bitrate: Bitrate::Bitrate320,
            gapless: true,
            ..Default::default()
        },
        session.clone(),
        Box::new(vol_clone),
        // opening the output device can panic on systems with no/broken audio.
        // catch it and fall back to a silent sink so the player thread survives
        // and the app stays usable instead of dying mid track
        move || {
            // in a vm never touch the real device, the open SIGSEGVs (cant be
            // caught) and would crash the whole app. just go straight to silence
            if force_null_sink {
                return Box::new(NullSink) as Box<dyn Sink>;
            }
            eprintln!("[playback] STEP opening audio device (native rate)");
            // DeviceSink opens at the device's native rate so it can't hit cpal's
            // set_sample_rate() SIGSEGV. catch_unwind is just a belt-and-suspenders
            // for any *unwinding* failure; a graceful Err also falls back to silence.
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(DeviceSink::open)) {
                Ok(Ok(s))  => { eprintln!("[playback] STEP audio device opened"); Box::new(s) as Box<dyn Sink> }
                Ok(Err(e)) => { eprintln!("[playback] device open failed: {e} - using silent sink"); Box::new(NullSink) as Box<dyn Sink> }
                Err(_)     => { eprintln!("[playback] device open panicked - using silent sink"); Box::new(NullSink) as Box<dyn Sink> }
            }
        },
    );
    eprintln!("[playback] STEP player built");

    // Register Musique as a Spotify Connect device and route ALL playback control
    // through it (see PlaybackInner helpers). Spirc::new() connects the session
    // (after wiring its dealer listeners) and starts a state machine that PUTs
    // connect-state to Spotify on every change - this is the whole point: it makes
    // plays appear in Spotify's "recently played", drives now-playing, and lets
    // other Spotify clients see/control this device.
    let mixer: Arc<dyn Mixer> = Arc::new(SharedMixer(volume.clone()));
    let connect_config = ConnectConfig {
        name:           "Musique".to_string(),
        device_type:    DeviceType::Computer,
        // seed Spirc's initial volume with the user's persisted level. Spirc sets
        // the mixer to this on startup; since our mixer IS the SharedVolume, using
        // the real level stops it from resetting playback volume to the 50% default.
        initial_volume: (initial_volume.clamp(0.0, 1.0) * u16::MAX as f64).round() as u16,
        ..Default::default()
    };
    let (spirc, spirc_task) = Spirc::new(
        connect_config,
        session.clone(),
        Credentials::with_access_token(&token),
        player.clone(),
        mixer,
    )
    .await
    .map_err(|e| {
        eprintln!("[playback] spirc/session connect failed: {e}");
        AppError::Auth(e.to_string())
    })?;
    eprintln!("[playback] STEP spirc connected (Connect device 'Musique' registered)");
    let spirc_task = tauri::async_runtime::spawn(spirc_task);

    // the access point pushes CountryCode (and ProductInfo) as SEPARATE packets
    // that show up AFTER the session connects (Spirc::new above did the connect).
    // librespots availability filter (available_for_user) checks each tracks
    // allowed-countries whitelist against session.country() and while thats still
    // empty EVERY track gets rejected as NotWhitelisted -> PlayerEvent::Unavailable
    // ("content may not be available in your region"). a play fired right after a
    // fresh connect races those packets, so wait a sec for the country to land.
    for _ in 0..50 {
        if !session.country().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    eprintln!("[playback] STEP country = {:?}", session.country());

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
            // pass the playback state along to the os media controls
            match &event {
                PlayerEvent::Playing { position_ms, .. } => {
                    let _ = media_tx.try_send(
                        crate::media_controls::MediaMsg::Playing { position_ms: *position_ms as u64 }
                    );
                }
                PlayerEvent::Paused { position_ms, .. } => {
                    let _ = media_tx.try_send(
                        crate::media_controls::MediaMsg::Paused { position_ms: *position_ms as u64 }
                    );
                }
                PlayerEvent::Stopped { .. }
                | PlayerEvent::EndOfTrack { .. }
                | PlayerEvent::Unavailable { .. } => {
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
                    let due   = last_pos_emit.elapsed() >= std::time::Duration::from_millis(250);
                    let moved = position_ms.abs_diff(last_pos_ms) >= 250;
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
        loaded:      Arc::new(AtomicBool::new(false)),
        spirc,
        session,
        _event_task: event_task,
        _spirc_task: spirc_task,
    })
}

// track id parsing stuff

pub fn parse_track_id(raw: &str) -> Result<SpotifyUri, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("Track id is required".into()));
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
