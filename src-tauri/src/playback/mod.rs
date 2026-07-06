use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::AtomicBool;

use librespot_core::{
    authentication::Credentials,
    config::SessionConfig,
    session::Session,
    SpotifyId,
    SpotifyUri,
};
use librespot_playback::{
    audio_backend::{Sink, SinkError, SinkResult},
    config::{Bitrate, PlayerConfig},
    convert::Converter,
    decoder::AudioPacket,
    mixer::VolumeGetter,
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

// silent fallback sink
//
// throws away all audio. used when opening the real output device panics (a
// headless/busted linux box with no alsa/pulse device, or a broken audio stack)
// and in a macos vm. only catches unwinding failures; a hard SIGSEGV inside a
// system audio framework is an environment fault we cant catch.
struct NullSink;

impl Sink for NullSink {
    fn write(&mut self, packet: AudioPacket, _converter: &mut Converter) -> SinkResult<()> {
        // throw the audio away BUT pace at real time. librespot's player thread
        // calls write() as fast as we return, so returning instantly makes the
        // decoder race through the whole track in a few ms -> EndOfTrack fires
        // almost immediately -> the frontend auto-advances -> EVERY track
        // "insta-skips". sleeping for the packet's real duration makes silent
        // playback advance at 1x, same backpressure the real rodio sink applies.
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

pub(crate) struct DeviceSink {
    sink: rodio::Sink,
    _stream: rodio::OutputStream,
}

impl DeviceSink {
    pub(crate) fn open() -> Result<DeviceSink, String> {
        let mut stream = rodio::OutputStreamBuilder::from_default_device().map_err(|e| e.to_string())?
        .open_stream()
        .map_err(|e| e.to_string())?;
    stream.log_on_drop(false);
    let sink = rodio::Sink::connect_new(stream.mixer());
    Ok(DeviceSink { sink, _stream: stream})
    }
}

impl Sink for DeviceSink {
    fn start(&mut self) -> SinkResult<()> {
        self.sink.play();
        Ok(())
    }
    fn stop(&mut self) -> SinkResult <()> {
        self.sink.sleep_until_end();
        self.sink.pause();
        Ok(())
}
    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let samples = packet.samples().map_err(|e| SinkError::OnWrite(e.to_string()))?;
        let samples_f32: &[f32] = &converter.f64_to_f32(samples);
        let source = rodio::buffer::SamplesBuffer::new(NUM_CHANNELS as u16, SAMPLE_RATE, samples_f32);
        self.sink.append(source);
        while self.sink.len() > 26{
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
}
}
    

// macos only: are we running inside a hypervisor (a vm)?
//
// apples paravirtualized coreaudio hal segfaults inside AudioObjectGetPropertyData
// the second cpal opens the default output device. thats a SIGSEGV in a system
// framework, rusts catch_unwind cant catch it. theres no real audio device in a vm
// anyway, so when we spot one we skip the real backend and run the silent sink. on
// real mac hardware kern.hv_vmm_present is 0 and the real backend is used normally.
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


fn audio_device_safe() -> bool {
    static SAFE: OnceLock<bool> = OnceLock::new();
    *SAFE.get_or_init(|| {
        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => { eprintln!("[playback] current_exe failed: {e} - assuming audio ok");
return true; }
        };
        match std::process::Command::new(exe).arg("--audio-probe").status(){
            Ok(s) if s.success() => true,
            Ok(s) => {eprintln!("[playback] audio device probe failed ({s}) - using silent sink"); false}
            Err(e) => {eprintln!("[playback] audio probe spawn error: {e} - assuming audio ok"); true}
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
    session:     Session,
    _event_task: tauri::async_runtime::JoinHandle<()>,
}

impl PlaybackInner {
    // a dropped/expired librespot session just eats load/play calls so
    // playback shows as "playing" with no actual audio. callers rebuild when this is
    // true (see ensure_inner)
    pub fn session_invalid(&self) -> bool {
        self.session.is_invalid()
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
    let session = Session::new(SessionConfig::default(), None);
    eprintln!("[playback] librespot session client_id = {}", session.client_id());
    session
        .connect(Credentials::with_access_token(&token), false)
        .await
        .map_err(|e| {
            eprintln!("[playback] session connect failed: {e}");
            AppError::Auth(e.to_string())
        })?;
    eprintln!("[playback] STEP session connected");

    // the access point pushes CountryCode (and ProductInfo) as SEPARATE packets
    // that show up AFTER connect() resolves. librespots availability filter
    // (available_for_user) checks each tracks allowed countries whitelist
    // against session.country() and while thats still empty EVERY track gets
    // rejected as NotWhitelisted and shows up as PlayerEvent::Unavailable ->
    // "content may not be available in your region". a play fired right after a
    // fresh connect (e.g play_track rebuilding a dead session) races those
    // packets and fails. so wait a sec for the country to land so availability is
    // figured out against the real region
    for _ in 0..50 {
        if !session.country().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    eprintln!("[playback] STEP country = {:?}", session.country());

    let volume    = SharedVolume::new(initial_volume, initial_muted);
    let vol_clone = volume.clone();

   ;

    // ONLY force silence for the macos-in-a-vm case: apples paravirt coreaudio hal
    // SIGSEGVs on device open and thats uncatchable, so we must never reach the
    // open there. on real hardware (mac/win/linux) we DO open the real device; the
    // catch_unwind below turns any *unwinding* open failure into a (real-time-paced)
    // silent sink so the app stays usable. no more child-process --audio-probe: a
    // bare helper process has no run loop and coreaudios hal proxy faulted when
    // opened without one, so the probe crashed healthy macs then wrongly forced
    // NullSink -> no audio + insta-skip.
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
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(DeviceSink::open)) {
                Ok(Ok(s)) => { eprintln!("[playback] STEP audio device opened"); Box::new(s) as Box<dyn Sink> }
                Ok(Err(e)) => { eprintln!("[playback] device open failed: {e} - using silent sink"); Box::new(NullSink) as Box<dyn Sink> }
                Err(_) => { eprintln!("[playback] device open panicked, using silent sink"); Box::new(NullSink) as Box<dyn Sink> }            
}
        },
    );
    eprintln!("[playback] STEP player built");

    let mut event_rx = player.get_player_event_channel();
    let event_app    = app.clone();
    let event_task   = tauri::async_runtime::spawn(async move {
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
        session,
        _event_task: event_task,
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
