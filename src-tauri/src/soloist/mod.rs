//! Spotify Soloist as an audio backend.
//!
//! Soloist is Spotify's own headless player (developer.spotify.com/
//! documentation/soloist): a separate program that signs in as a Spotify
//! Connect device, opens the audio output itself and is driven over a
//! WebSocket bound to this computer. It needs Premium and a personal key from
//! the listener's Spotify developer dashboard. The app signs it in with the
//! playback account by itself (see `pair`); picking it once in a Spotify app's
//! device list is only the fallback.
//!
//! This module downloads it (`install`), runs one copy as a child process,
//! restarts it when it falls over, and translates between its API and what the
//! rest of the app already speaks: `play`/`pause`/`seek` go out as commands,
//! and what it reports comes back as the same `player:event`s the librespot
//! and YouTube backends emit, so the interface cannot tell the difference.
//!
//! The app's queue stays the app's: each track is played by its URI and the
//! interface moves on when it is told the track ended, exactly as with the
//! other backends. Soloist gives no PCM, so nothing here touches the audio.
//!
//! What was learnt by driving the real player (1.3.10):
//! - its `timestamp_ms` is this computer's clock, and its position is what
//!   reaches the speaker about 50ms later (measured over a loopback capture);
//! - `play` takes no start position and activates the device by itself, so a
//!   start part-way through is `play` then `seek`;
//! - it reads `audio.play_bitrate_enumeration` from `settings/prefs` at
//!   launch: 3 and below stream Ogg Vorbis 160 kbps, 4 streams 320 kbps and 5
//!   asks for lossless, falling back to 320 kbps when the plan has none. It
//!   rewrites the file on exit, so the choice is written before every launch;
//! - with `--verbose` it logs the codec, bitrate and sample rate of each file
//!   it opens, which is the only place the format can be read from.

pub mod install;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::errors::AppError;
use crate::playback::PlayerMsg;
use install::Build;

const KEYRING_SERVICE: &str = "spotify-client";
const KEYRING_ACCOUNT: &str = "soloist_api_key";

/// How long Soloist gets to start listening after it is launched.
const START_LIMIT: Duration = Duration::from_secs(20);
/// How long after one restart the next waits: a player that keeps falling
/// over is given up on rather than restarted for ever.
const RESTART_AFTER: [Duration; 3] =
    [Duration::from_secs(2), Duration::from_secs(10), Duration::from_secs(30)];
/// A session that stayed up this long was healthy; the next failure starts
/// counting restarts afresh.
const HEALTHY_AFTER: Duration = Duration::from_secs(120);
/// How often sign-in and the build's age are looked at.
const TICK: Duration = Duration::from_secs(30);
/// How often the position is asked for while playing (see the poll in `drive`).
const POSITION_POLL: Duration = Duration::from_secs(1);
/// Soloist exits with this code when its build has expired.
const EXPIRED_EXIT: i32 = 10;
/// The most of Soloist's own output kept for a bug report.
const OUTPUT_LIMIT: usize = 1 << 20;
/// How long a play waits for a Soloist that is still starting or signing in.
/// A first run downloads about 20 MB, so this is generous.
const READY_LIMIT: Duration = Duration::from_secs(180);
/// How long a play waits on a Soloist that is up but not signed in before it
/// says so: the app signs it in within a few seconds when it can at all.
const SIGN_IN_LIMIT: Duration = Duration::from_secs(20);
/// What Soloist's reported position runs ahead of the speaker by.
pub const OUTPUT_LATENCY_MS: i64 = 50;

/// Where Soloist is, for the interface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    /// Not running.
    Off,
    /// Spotify publishes no build for this kind of computer.
    Unsupported,
    /// Chosen, but there is no personal key to start it with.
    NeedsKey,
    Downloading { received: u64, total: u64 },
    Starting,
    /// Running and waiting to be signed in.
    Pairing { name: String, version: String },
    /// Signed in and taking commands.
    Ready { name: String, version: String, expires: u64 },
    Failed { message: String },
}

/// The streaming quality asked of Soloist (see the module notes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    /// Ogg Vorbis, 160 kbps.
    High,
    /// Ogg Vorbis, 320 kbps.
    VeryHigh,
    /// FLAC when the plan includes it, otherwise 320 kbps.
    Lossless,
}

impl Quality {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "high" => Some(Self::High),
            "very_high" => Some(Self::VeryHigh),
            "lossless" => Some(Self::Lossless),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::VeryHigh => "very_high",
            Self::Lossless => "lossless",
        }
    }

    /// The value of Spotify's `audio.play_bitrate_enumeration` preference.
    fn enumeration(self) -> u8 {
        match self {
            Self::High => 3,
            Self::VeryHigh => 4,
            Self::Lossless => 5,
        }
    }
}

/// What Soloist is decoding, as its log reports it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Format {
    /// As Soloist names it: "vorbis", "flac", ...
    pub codec:       String,
    pub bitrate:     Option<u32>,
    pub sample_rate: Option<u32>,
}

/// What Soloist last said about playback, and what the app asked of it.
#[derive(Default)]
struct Play {
    /// The URI the app asked for.
    requested:   Option<String>,
    /// What Soloist calls that track, when it plays it under another URI
    /// (Spotify swaps a track for the copy licensed where the listener is).
    alias:       Option<String>,
    /// What Soloist is on.
    item:        Option<String>,
    status:      String,
    position_ms: u64,
    /// When `position_ms` was true, in Unix milliseconds. 0 when unknown.
    at_ms:       u64,
    speed:       f64,
    duration_ms: u64,
    /// The furthest into the requested track playback has been seen.
    peak_ms:     u64,
    /// The requested track has been heard playing.
    started:     bool,
    /// Its end has been reported.
    ended:       bool,
    /// The app paused it, so a pause near the end is not the end.
    user_paused: bool,
    /// Soloist has answered the `play` for the requested track. Until then
    /// what it reports is whatever it was on before (the account's last
    /// state, restored when it became active), which is not the app's track.
    acked:       bool,
    /// Where to start the requested track, when not at its top. Soloist
    /// drops a seek that arrives before the track has loaded, so it is sent
    /// once the track is heard playing; until then nobody hears of it.
    seek_to:     Option<u32>,
    /// That seek, sent at the time given, and not yet seen to land.
    seeking:     Option<(u32, u64)>,
    /// Soloist is the account's active device.
    active:      bool,
    /// The requested track has been heard at least once.
    heard:       bool,
    /// Another device took playback over: nothing Soloist reports from then
    /// on is about the app's track, until the app asks for one again.
    lost:        bool,
}

/// What to do about something Soloist said.
#[derive(Debug, PartialEq)]
enum Out {
    Event(PlayerMsg),
    Command(Value),
    /// Put the volume back: it was down while a start part-way through
    /// skipped the top of the track.
    Unmute,
}

/// How long a seek gets to land before what Soloist reports is taken as is.
const SEEK_LANDS_WITHIN_MS: u64 = 4000;

fn unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

/// The id the interface knows a URI by: the bare id of a track, the whole URI
/// of anything else (episodes are kept by their URI throughout the app).
fn id_of(uri: &str) -> String {
    uri.strip_prefix("spotify:track:").unwrap_or(uri).to_string()
}

fn ms32(ms: u64) -> u32 {
    ms.min(u64::from(u32::MAX)) as u32
}

impl Play {
    fn position_now(&self, now_ms: u64) -> u64 {
        let moved = if self.at_ms == 0 || self.speed <= 0.0 {
            0
        } else {
            (now_ms.saturating_sub(self.at_ms) as f64 * self.speed.min(4.0)) as u64
        };
        let position = self.position_ms.saturating_add(moved);
        if self.duration_ms > 0 { position.min(self.duration_ms) } else { position }
    }

    fn on_requested(&self) -> bool {
        let Some(item) = self.item.as_deref() else {
            return false;
        };
        self.requested.as_deref() == Some(item) || self.alias.as_deref() == Some(item)
    }

    /// The app asked for `uri`, from `start_ms` into it.
    fn request(&mut self, uri: &str, start_ms: u32) {
        self.requested = Some(uri.to_string());
        self.alias = None;
        self.peak_ms = 0;
        self.started = false;
        self.ended = false;
        self.user_paused = false;
        self.acked = false;
        self.seek_to = (start_ms > 0).then_some(start_ms);
        self.seeking = None;
        self.heard = false;
        self.lost = false;
    }

    /// Forget what Soloist said but keep the app's place in its track, for a
    /// restart: playing on afterwards starts the track again from there.
    fn keep_place(&mut self, now_ms: u64) {
        let position_ms = if self.ended { 0 } else { self.position_now(now_ms) };
        *self = Play {
            requested: self.requested.take(),
            duration_ms: self.duration_ms,
            position_ms,
            ..Play::default()
        };
    }

    /// Whether the position is worth asking for: the track plays, or a start
    /// part-way through is waiting to be seen to land.
    fn wants_poll(&self) -> bool {
        (self.started && !self.ended && matches!(self.status.as_str(), "playing" | "buffering"))
            || (self.acked && !self.ended && (self.seek_to.is_some() || self.seeking.is_some()))
    }

    /// Hold the position where it is: playback stopped here.
    fn freeze(&mut self, now_ms: u64) {
        self.position_ms = self.position_now(now_ms);
        self.at_ms = now_ms;
        self.speed = 0.0;
    }

    /// The track the app is on and where to start it again, when it is not
    /// loaded and playing on in Soloist.
    fn resume_point(&self) -> Option<(String, u32)> {
        let uri = self.requested.clone()?;
        let at = if self.ended { 0 } else { ms32(self.position_now(unix_ms())) };
        Some((uri, at))
    }

    fn set_position(&mut self, position: &Value) {
        self.position_ms = position["position_ms"].as_u64().unwrap_or(0);
        self.at_ms = position["timestamp_ms"].as_u64().unwrap_or(0);
        self.speed = position["speed"].as_f64().unwrap_or(0.0);
    }

    fn set_item(&mut self, item: &Value) {
        self.item = item["uri"].as_str().filter(|uri| !uri.is_empty()).map(str::to_owned);
        self.duration_ms = item["decorations"]["playback"]["duration_ms"].as_u64().unwrap_or(0);
    }

    /// Take in a message from Soloist and say what the app should hear of it.
    fn observe(&mut self, message: &Value, now_ms: u64) -> Vec<Out> {
        let kind = message["type"].as_str().unwrap_or_default();
        // a `playback_state` that leaves the status as it was is the answer to
        // the once-a-second position poll, not a change of state: it must
        // reach the interface as a position update, never as a fresh
        // `playing` (that event drives scrobbles and now-playing)
        let was = self.status.clone();
        if self.started && !self.ended && self.on_requested() {
            self.peak_ms = self.peak_ms.max(self.position_now(now_ms));
        }
        let was_active = self.active;
        // once another device has the account, where it is in whatever it
        // plays is not the app's place in its track
        let kept = self.lost.then(|| (self.position_ms, self.at_ms, self.speed, self.duration_ms));
        let outs = self.take_in(kind, message);
        if let Some((position_ms, at_ms, speed, duration_ms)) = kept {
            (self.position_ms, self.at_ms, self.speed, self.duration_ms) = (position_ms, at_ms, speed, duration_ms);
        }
        if let Some(outs) = outs {
            return outs;
        }
        self.report(kind, &was, was_active, now_ms)
    }

    /// Record what a message says. `Some` when it is answered by itself.
    fn take_in(&mut self, kind: &str, message: &Value) -> Option<Vec<Out>> {
        match kind {
            "playback_state" => {
                self.set_item(&message["item"]);
                self.status = message["status"].as_str().unwrap_or_default().to_string();
                self.set_position(&message["position"]);
                if let Some(active) = message["is_active"].as_bool() {
                    self.active = active;
                }
            }
            "track_changed" => {
                self.set_item(&message["item"]);
                // the first change after the play was taken is that track,
                // whatever Soloist calls it
                if self.requested.is_some() && self.acked && !self.heard && !self.lost && !self.ended && !self.on_requested() {
                    self.alias = self.item.clone();
                }
            }
            "playback_changed" => {
                self.status = message["status"].as_str().unwrap_or_default().to_string();
            }
            "position_sync" => self.set_position(&message["position"]),
            "device_changed" => {
                self.active = message["is_active"].as_bool().unwrap_or(false);
            }
            "command_result" => {
                match message["command"].as_str() {
                    Some("play") => self.acked = true,
                    _ => {}
                }
                return Some(Vec::new());
            }
            "error" => {
                // refused before it was taken: the track will not play, which
                // the interface treats like any track that did not load
                if self.requested.is_some() && !self.acked && !self.ended {
                    self.ended = true;
                    let track_id = self.requested.as_deref().map(id_of);
                    let unmute = self.seek_to.take().is_some();
                    let mut outs = vec![Out::Event(PlayerMsg::Unavailable { track_id })];
                    if unmute {
                        outs.push(Out::Unmute);
                    }
                    return Some(outs);
                }
                return Some(Vec::new());
            }
            _ => return Some(Vec::new()),
        }
        None
    }

    /// What the interface hears of the state as it now is.
    fn report(&mut self, kind: &str, was: &str, was_active: bool, now_ms: u64) -> Vec<Out> {
        let Some(requested) = self.requested.clone() else {
            return Vec::new();
        };
        if self.ended || self.lost || !self.acked {
            return Vec::new();
        }
        let mut outs = Vec::new();
        if let Some(target) = self.seek_to {
            // loaded and running: now a seek is taken
            if self.on_requested() && self.status == "playing" {
                self.seek_to = None;
                self.seeking = Some((target, now_ms));
                return vec![Out::Command(json!({"command": "seek", "position_ms": target}))];
            }
            return Vec::new();
        }
        if let Some((target, sent)) = self.seeking {
            let at = self.position_now(now_ms);
            let target = u64::from(target);
            let landed = self.on_requested()
                && self.status == "playing"
                && at.saturating_add(1500) >= target
                && at <= target.saturating_add(5000);
            if !landed && now_ms.saturating_sub(sent) < SEEK_LANDS_WITHIN_MS {
                return Vec::new();
            }
            self.seeking = None;
            outs.push(Out::Unmute);
        }
        outs.extend(self.report_state(&requested, kind, was, was_active, now_ms));
        outs
    }

    fn report_state(&mut self, requested: &str, kind: &str, was: &str, was_active: bool, now_ms: u64) -> Vec<Out> {
        let track_id = Some(id_of(requested));

        // another device took playback over: here, that is a pause. Playing
        // on starts the track again on this computer from the same place.
        if was_active && !self.active && self.started {
            self.freeze(now_ms);
            self.started = false;
            self.user_paused = true;
            self.lost = true;
            let position_ms = ms32(self.position_ms);
            return vec![Out::Event(PlayerMsg::Paused { track_id, position_ms })];
        }

        if !self.on_requested() {
            if self.started && self.item.is_some() {
                // Soloist went on to something of its own choosing (autoplay,
                // the rest of an album). The app's queue decides what is next.
                self.ended = true;
                self.started = false;
                let mut outs = Vec::new();
                // only while it is this account's player: a pause sent to one
                // that is not would pause whatever device has taken over
                if self.active {
                    outs.push(Out::Command(json!({"command": "pause"})));
                }
                outs.push(Out::Event(PlayerMsg::EndOfTrack { track_id }));
                return outs;
            }
            return Vec::new();
        }

        let position_ms = ms32(self.position_now(now_ms));
        match self.status.as_str() {
            "playing" => {
                let moved_only = self.started
                    && (kind == "position_sync" || (kind == "playback_state" && was == "playing"));
                self.started = true;
                self.heard = true;
                if moved_only {
                    vec![Out::Event(PlayerMsg::PositionChanged { track_id, position_ms })]
                } else {
                    vec![Out::Event(PlayerMsg::Playing { track_id, position_ms })]
                }
            }
            "paused" if self.started => {
                let at_end = self.duration_ms > 0
                    && self.peak_ms.saturating_add(3000) >= self.duration_ms
                    && (u64::from(position_ms).saturating_add(3000) >= self.duration_ms
                        || position_ms < 1500);
                if at_end && !self.user_paused {
                    self.ended = true;
                    self.started = false;
                    vec![Out::Event(PlayerMsg::EndOfTrack { track_id })]
                } else if kind == "position_sync" || (kind == "playback_state" && was == "paused") {
                    vec![Out::Event(PlayerMsg::PositionChanged { track_id, position_ms })]
                } else {
                    vec![Out::Event(PlayerMsg::Paused { track_id, position_ms })]
                }
            }
            "idle" if self.started => {
                self.ended = true;
                self.started = false;
                vec![Out::Event(PlayerMsg::EndOfTrack { track_id })]
            }
            _ => Vec::new(),
        }
    }
}

struct Shared {
    status: watch::Sender<Status>,
    play:   Mutex<Play>,
    format: Mutex<Option<Format>>,
    /// A track to start again once the player is back up: it was playing when
    /// the last one fell over.
    resume: Mutex<Option<(String, u32)>>,
    /// The device name this copy is signed in under.
    name:   Mutex<Option<String>>,
    /// The level the listener chose, 0 to 100.
    volume: AtomicU8,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
    fn play(&self) -> MutexGuard<'_, Play> {
        lock(&self.play)
    }

    fn status(&self) -> Status {
        self.status.borrow().clone()
    }

    fn set_status(&self, app: &AppHandle, status: Status) {
        let changed = self.status.send_if_modified(|current| {
            if *current == status {
                return false;
            }
            *current = status.clone();
            true
        });
        if !changed {
            return;
        }
        if !matches!(status, Status::Downloading { .. }) {
            log::info!("soloist: {status:?}");
        }
        let _ = app.emit("soloist:status", status);
    }
}

struct Running {
    stop:     watch::Sender<bool>,
    commands: mpsc::UnboundedSender<Value>,
    task:     tauri::async_runtime::JoinHandle<()>,
}

/// A play waiting for the player to come up. A later play, a pause or a
/// seek changes it rather than queueing behind it.
struct Pending {
    request: u64,
    uri:     String,
    at:      u32,
}

/// Set while Soloist is the backend: the built-in player's events are then
/// about nothing the listener hears, and must not reach the interface (a
/// Spotify Connect hand-over to Soloist stops the built-in player).
static OWNS_OUTPUT: AtomicBool = AtomicBool::new(false);

pub fn owns_output() -> bool {
    OWNS_OUTPUT.load(Ordering::Relaxed)
}

/// The one Soloist this app runs, or would run.
pub struct Soloist {
    shared:      Arc<Shared>,
    running:     Mutex<Option<Running>>,
    /// Numbers play requests: only the newest waiting one goes ahead.
    request:     AtomicU64,
    pending:     Mutex<Option<Pending>>,
    /// The quality changed while a track played: the player restarts, to
    /// read it, when the next track starts.
    restart_due: AtomicBool,
    /// Serialises starts and stops, so two never interleave.
    lifecycle:   tokio::sync::Mutex<()>,
    /// The backend that was playing when Soloist was chosen. It plays on
    /// while Soloist starts and signs in, and is silenced the moment Soloist
    /// takes the track over, so switching leaves no gap of silence.
    handover:    Mutex<Option<crate::commands::playback::Backend>>,
    app:         std::sync::OnceLock<AppHandle>,
}

impl Soloist {
    pub fn new() -> Arc<Self> {
        let (status, _) = watch::channel(Status::Off);
        Arc::new(Self {
            shared: Arc::new(Shared {
                status,
                play: Mutex::new(Play::default()),
                format: Mutex::new(None),
                resume: Mutex::new(None),
                name: Mutex::new(None),
                volume: AtomicU8::new(80),
            }),
            running: Mutex::new(None),
            request: AtomicU64::new(0),
            pending: Mutex::new(None),
            restart_due: AtomicBool::new(false),
            lifecycle: tokio::sync::Mutex::new(()),
            handover: Mutex::new(None),
            app: std::sync::OnceLock::new(),
        })
    }

    fn running(&self) -> MutexGuard<'_, Option<Running>> {
        lock(&self.running)
    }

    pub fn status(&self) -> Status {
        self.shared.status()
    }

    /// What the player is decoding, when it has said.
    pub fn format(&self) -> Option<Format> {
        lock(&self.shared.format).clone()
    }

    pub fn is_running(&self) -> bool {
        self.running().as_ref().is_some_and(|running| !running.task.inner().is_finished())
    }

    /// Start Soloist afresh, with `key` when the listener has just supplied
    /// one and the saved one otherwise. Replaces a copy already running.
    pub async fn start(&self, app: AppHandle, key: Option<String>) {
        let _turn = self.lifecycle.lock().await;
        self.halt().await;
        *self.shared.play() = Play::default();
        *lock(&self.shared.resume) = None;
        self.launch(app, key);
    }

    /// Start a new copy and keep the app's place: a paused track carries on
    /// from where it was, and a playing one starts again from there.
    async fn restart(&self, app: AppHandle) {
        let _turn = self.lifecycle.lock().await;
        let playing = {
            let play = self.shared.play();
            play.started && !play.ended && matches!(play.status.as_str(), "playing" | "buffering")
        };
        self.halt().await;
        let now = unix_ms();
        let mut play = self.shared.play();
        play.keep_place(now);
        *lock(&self.shared.resume) = if playing { play.resume_point() } else { None };
        drop(play);
        self.launch(app, None);
    }

    fn launch(&self, app: AppHandle, key: Option<String>) {
        let _ = self.app.set(app.clone());
        OWNS_OUTPUT.store(true, Ordering::Relaxed);
        if !install::supported() {
            self.shared.set_status(&app, Status::Unsupported);
            return;
        }
        let (stop, stopped) = watch::channel(false);
        let (commands, inbox) = mpsc::unbounded_channel();
        let shared = Arc::clone(&self.shared);
        shared.set_status(&app, Status::Starting);
        let task = tauri::async_runtime::spawn(supervise(app, shared, key, stopped, inbox));
        *self.running() = Some(Running { stop, commands, task });
    }

    /// Stop the running copy and wait for it to be gone.
    async fn halt(&self) {
        let running = self.running().take();
        if let Some(running) = running {
            let _ = running.stop.send(true);
            let _ = tokio::time::timeout(Duration::from_secs(5), running.task).await;
        }
    }

    /// Let `from` play on until Soloist takes over (see `handover`).
    pub fn hand_over_from(&self, from: crate::commands::playback::Backend) {
        *lock(&self.handover) = Some(from);
    }

    /// Silence the backend Soloist is taking over from, if it still plays.
    async fn finish_handover(&self) {
        let from = lock(&self.handover).take();
        if let (Some(from), Some(app)) = (from, self.app.get()) {
            crate::commands::playback::silence(app, from).await;
        }
    }

    /// Stop the player for good: Soloist is no longer the backend.
    pub async fn stop(&self) {
        *lock(&self.handover) = None;
        let _turn = self.lifecycle.lock().await;
        // a play still waiting for it must not start it again
        self.request.fetch_add(1, Ordering::SeqCst);
        *lock(&self.pending) = None;
        self.give_up_active().await;
        self.halt().await;
        OWNS_OUTPUT.store(false, Ordering::Relaxed);
        self.restart_due.store(false, Ordering::Relaxed);
        *self.shared.play() = Play::default();
        *lock(&self.shared.resume) = None;
        *lock(&self.shared.format) = None;
        let _ = self.shared.status.send_replace(Status::Off);
    }

    /// Hand the account's active-device role back before going: a player
    /// that is simply killed stays the active device in Spotify's eyes for
    /// a moment, long enough to knock the built-in player, starting the
    /// same song at that moment, back off again ("device became inactive").
    async fn give_up_active(&self) {
        if !self.ready() || !self.shared.play().active {
            return;
        }
        if self.send(json!({"command": "deactivate"})).is_err() {
            return;
        }
        let began = Instant::now();
        while self.shared.play().active && began.elapsed() < Duration::from_millis(2000) {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    fn ready(&self) -> bool {
        matches!(self.status(), Status::Ready { .. })
    }

    fn send(&self, command: Value) -> Result<(), AppError> {
        self.running()
            .as_ref()
            .and_then(|running| running.commands.send(command).ok())
            .ok_or_else(|| AppError::Playback("Soloist is not running".into()))
    }

    /// Wait for the player to be signed in. `Ok(false)` when a newer request
    /// (or a pause) took this one's place meanwhile.
    async fn wait_ready(&self, request: u64) -> Result<bool, AppError> {
        let mut status = self.shared.status.subscribe();
        let began = Instant::now();
        let mut signing_in: Option<Instant> = None;
        loop {
            if self.request.load(Ordering::SeqCst) != request {
                return Ok(false);
            }
            let now = status.borrow_and_update().clone();
            match now {
                Status::Ready { .. } => return Ok(true),
                Status::Failed { message } => return Err(AppError::Playback(format!("Soloist didn't start: {message}"))),
                Status::NeedsKey => {
                    return Err(AppError::Playback("Soloist needs your personal key. Add it in Settings → Playback.".into()));
                }
                Status::Unsupported => {
                    return Err(AppError::Playback("Spotify doesn't publish Soloist for this kind of computer.".into()));
                }
                Status::Off => return Err(AppError::Playback("Soloist isn't running.".into())),
                Status::Pairing { name, .. } => {
                    let since = *signing_in.get_or_insert_with(Instant::now);
                    if since.elapsed() > SIGN_IN_LIMIT {
                        return Err(AppError::Playback(format!(
                            "Soloist couldn't sign in by itself. Open Spotify on a phone or computer on this network and pick \"{name}\" from its device list once."
                        )));
                    }
                }
                Status::Downloading { .. } | Status::Starting => signing_in = None,
            }
            if began.elapsed() > READY_LIMIT {
                return Err(AppError::Playback("Soloist took too long to start. Try again in a moment.".into()));
            }
            tokio::select! {
                changed = status.changed() => if changed.is_err() {
                    return Err(AppError::Playback("Soloist isn't running.".into()));
                },
                _ = tokio::time::sleep(Duration::from_millis(500)) => {}
            }
        }
    }

    /// Play `uri` from the start, or from `position_ms`. Waits for a player
    /// that is still starting or signing in (starting one that should be up
    /// and is not), so the first play after choosing Soloist just works.
    pub async fn play(&self, app: &AppHandle, uri: &str, position_ms: u32) -> Result<(), AppError> {
        let asked = Instant::now();
        let request = self.request.fetch_add(1, Ordering::SeqCst) + 1;
        *lock(&self.pending) = Some(Pending { request, uri: uri.to_string(), at: position_ms });
        *lock(&self.shared.resume) = None;
        if self.restart_due.swap(false, Ordering::Relaxed) && self.is_running() {
            // the quality changed during the last track: read it now, in the
            // gap between two tracks, rather than cut a track off for it
            self.restart(app.clone()).await;
        } else if !self.is_running() || matches!(self.status(), Status::Failed { .. } | Status::Off) {
            self.start(app.clone(), None).await;
        }
        let ready = self.wait_ready(request).await;
        if !matches!(ready, Ok(true)) {
            // whatever happens, the old backend does not play on under this
            if !matches!(ready, Ok(false)) || lock(&self.pending).is_none() {
                self.finish_handover().await;
            }
            return ready.map(|_| ());
        }
        // a seek made while it waited moved where this starts
        let Some(Pending { uri, mut at, .. }) = lock(&self.pending).take().filter(|p| p.request == request) else {
            return Ok(());
        };
        // the track played on through the old backend while this waited:
        // carry on from where it got to, not from where it was asked
        if lock(&self.handover).is_some() && at > 0 {
            at = at.saturating_add(ms32(asked.elapsed().as_millis() as u64));
        }
        self.finish_handover().await;
        self.shared.play().request(&uri, at);
        // starting part-way, the top of the track plays for a moment before
        // the seek can be taken (see `Play::seek_to`): not out loud
        let volume = if at > 0 { 0 } else { soloist_volume(self.shared.volume.load(Ordering::Relaxed)) };
        self.send(json!({"command": "set_volume", "volume": volume}))?;
        self.send(json!({"command": "play", "uri": uri}))?;
        Ok(())
    }

    /// Carry on with the app's track: un-pause it when Soloist still has it,
    /// otherwise start it again from where it was.
    pub async fn resume(&self, app: &AppHandle) -> Result<(), AppError> {
        let point = {
            let play = self.shared.play();
            let loaded = play.started && !play.ended && play.on_requested();
            if loaded { None } else { play.resume_point() }
        };
        match point {
            Some((uri, at)) => self.play(app, &uri, at).await,
            None if self.ready() => {
                self.shared.play().user_paused = false;
                self.send(json!({"command": "play"}))
            }
            None => Ok(()),
        }
    }

    pub fn pause(&self) -> Result<(), AppError> {
        // the backend still playing while Soloist starts stops too
        if lock(&self.handover).is_some() {
            if let Some(app) = self.app.get() {
                let from = lock(&self.handover).take();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(from) = from {
                        crate::commands::playback::silence(&app, from).await;
                    }
                });
            }
        }
        // whatever was about to start, does not
        self.request.fetch_add(1, Ordering::SeqCst);
        *lock(&self.pending) = None;
        *lock(&self.shared.resume) = None;
        let unmute = {
            let mut play = self.shared.play();
            play.user_paused = true;
            // a start part-way through that has not landed never will now;
            // playing on starts there
            let target = play.seek_to.take().or(play.seeking.take().map(|(target, _)| target));
            if let Some(target) = target {
                play.position_ms = u64::from(target);
                play.at_ms = unix_ms();
                play.speed = 0.0;
            }
            target.is_some()
        };
        if !self.ready() {
            return Ok(());
        }
        self.send(json!({"command": "pause"}))?;
        if unmute {
            self.send(json!({"command": "set_volume", "volume": soloist_volume(self.shared.volume.load(Ordering::Relaxed))}))?;
        }
        Ok(())
    }

    pub fn seek(&self, position_ms: u32) -> Result<(), AppError> {
        if let Some(pending) = lock(&self.pending).as_mut() {
            pending.at = position_ms;
            return Ok(());
        }
        let loaded = {
            let mut play = self.shared.play();
            // a start part-way through that is still on its way goes there instead
            if play.seek_to.is_some() {
                play.seek_to = Some(position_ms.max(1));
                return Ok(());
            }
            let loaded = (play.started || play.seeking.is_some()) && !play.ended && play.on_requested();
            if !loaded {
                // nothing loaded to move: playing on starts from here
                play.position_ms = u64::from(position_ms);
                play.at_ms = unix_ms();
                play.speed = 0.0;
                play.ended = false;
            }
            loaded
        };
        if !loaded || !self.ready() {
            return Ok(());
        }
        self.send(json!({"command": "seek", "position_ms": position_ms}))
    }

    /// `percent` is 0 to 100. Quietly does nothing while Soloist is not up:
    /// the level is sent again when it signs in.
    pub fn set_volume(&self, percent: u8) {
        self.shared.volume.store(percent.min(100), Ordering::Relaxed);
        let quiet = {
            let play = self.shared.play();
            play.seek_to.is_some() || play.seeking.is_some()
        };
        if self.ready() && !quiet {
            let _ = self.send(json!({"command": "set_volume", "volume": soloist_volume(percent)}));
        }
    }

    /// Whether the track the app asked for is loaded and has not ended, so
    /// pressing play can carry it on rather than start it again.
    pub fn resumable(&self, uri: &str) -> bool {
        let play = self.shared.play();
        play.requested.as_deref() == Some(uri) && play.started && !play.ended && play.on_requested()
    }

    pub fn position_ms(&self) -> u32 {
        ms32(self.shared.play().position_now(unix_ms()))
    }

    /// The quality setting changed: a player that is not playing restarts now
    /// to read it; one that is reads it when the next track starts.
    pub async fn quality_changed(&self, app: &AppHandle) {
        if !self.is_running() {
            return;
        }
        let playing = {
            let play = self.shared.play();
            play.started && !play.ended && matches!(play.status.as_str(), "playing" | "buffering")
        };
        if playing {
            self.restart_due.store(true, Ordering::Relaxed);
        } else {
            self.restart(app.clone()).await;
        }
    }

    /// The name this computer's Soloist appears under in Spotify's device
    /// list, so the app can show it as this computer rather than another.
    pub fn device_name(&self, app: &AppHandle) -> Option<String> {
        if let Some(name) = lock(&self.shared.name).clone() {
            return Some(name);
        }
        let id = std::fs::read_to_string(root(app).join("instance-id")).ok()?;
        let id = id.trim();
        (id.len() == 8 && id.bytes().all(|b| b.is_ascii_hexdigit())).then(|| name_for(id))
    }
}

/// Soloist's volume against its loudness, measured over a loopback capture
/// with the same passage played at each step: close to half a decibel per
/// step, flatter at the very top.
const SOLOIST_CURVE_DB: [(u8, f64); 11] = [
    (100, 0.0), (90, -3.4), (80, -8.5), (70, -13.7), (60, -18.8), (50, -23.8),
    (40, -28.9), (30, -33.9), (20, -38.9), (10, -43.9), (5, -50.0),
];

/// The Soloist volume that is as loud as the app's `level` (0 to 100) is on
/// the built-in and YouTube players, which scale the signal by the level
/// (or its cube, with the cubic curve on). Sent as is, the app's 20% would be
/// about 25 dB quieter through Soloist than through the built-in player.
pub fn soloist_volume(level: u8) -> u8 {
    let level = f64::from(level.min(100)) / 100.0;
    let gain = if crate::playback::cubic_volume() { level * level * level } else { level };
    if gain <= 0.0 {
        return 0;
    }
    let db = 20.0 * gain.log10();
    for pair in SOLOIST_CURVE_DB.windows(2) {
        let ((high, high_db), (low, low_db)) = (pair[0], pair[1]);
        if db >= low_db {
            let t = (db - low_db) / (high_db - low_db);
            let volume = f64::from(low) + t * f64::from(high - low);
            return volume.round().clamp(1.0, 100.0) as u8;
        }
    }
    // quieter than the curve was measured: on down to silence
    let (low, low_db) = SOLOIST_CURVE_DB[SOLOIST_CURVE_DB.len() - 1];
    (f64::from(low) * (db / low_db).recip()).round().clamp(1.0, f64::from(low)) as u8
}

fn name_for(id: &str) -> String {
    format!("Musique Soloist ({id})")
}

/// Write the preferences Soloist reads at launch. Only the streaming quality:
/// its loudness normalisation is not read from here (`audio.normalize_v2`
/// was tried and ignored; the setting lives in the account's synced
/// settings now, which are left alone). Soloist rewrites the file when it
/// exits, so this runs before every start.
fn write_prefs(data: &Path, quality: Quality) {
    let settings = data.join("settings");
    let _ = std::fs::create_dir_all(&settings);
    let bitrate = quality.enumeration().to_string();
    let wanted = [
        ("audio.play_bitrate_enumeration", bitrate.clone()),
        ("audio.play_bitrate_non_metered_enumeration", bitrate),
    ];
    let set = |path: &Path, add: bool| {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let mut found = false;
        let mut lines: Vec<String> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| match wanted.iter().find(|(key, _)| line.starts_with(&format!("{key}="))) {
                Some((key, value)) => {
                    found = true;
                    format!("{key}={value}")
                }
                None => line.to_string(),
            })
            .collect();
        if !found && !add {
            return;
        }
        for (key, value) in &wanted {
            if add && !lines.iter().any(|line| line.starts_with(&format!("{key}="))) {
                lines.push(format!("{key}={value}"));
            }
        }
        let _ = std::fs::write(path, lines.join("
") + "
");
    };
    set(&settings.join("prefs"), true);
    // a per-account copy, should Soloist ever write one, must agree with it
    if let Ok(users) = std::fs::read_dir(settings.join("Users")) {
        for user in users.flatten() {
            let prefs = user.path().join("prefs");
            if prefs.is_file() {
                set(&prefs, false);
            }
        }
    }
}

/// The quality the listener chose for Soloist. Lossless by default: it costs
/// nothing on a plan without it, which streams 320 kbps instead.
pub async fn saved_quality(pool: &sqlx::SqlitePool) -> Quality {
    sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'soloist_quality'")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|(value,)| Quality::parse(&value))
        .unwrap_or(Quality::Lossless)
}

/// Read a format line out of Soloist's verbose log, into `format`. True when
/// it said something new.
fn read_format(line: &str, format: &mut Format) -> bool {
    let Some((_, said)) = line.split_once("] ") else {
        return false;
    };
    let said = said.trim();
    if let Some(codec) = said.strip_prefix("codec: ") {
        *format = Format { codec: codec.trim().to_ascii_lowercase(), ..Format::default() };
        return true;
    }
    if format.codec.is_empty() {
        return false;
    }
    let number = |text: &str, unit: &str| text.trim().strip_suffix(unit).unwrap_or(text).trim().parse::<u32>().ok();
    if let Some(bitrate) = said.strip_prefix("bitrate: ").and_then(|text| number(text, "bits/s")) {
        format.bitrate = Some(bitrate).filter(|b| *b > 0);
        return true;
    }
    if let Some(rate) = said.strip_prefix("sample rate: ").and_then(|text| number(text, "Hz")) {
        format.sample_rate = Some(rate).filter(|r| *r > 0);
        return true;
    }
    false
}

// ── the personal key ────────────────────────────────────────────────────────

/// A key is one word. Anything else is a mistake or an attempt to pass
/// Soloist another option.
fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 512
        && !key.starts_with('-')
        && key.chars().all(|c| c.is_ascii_graphic())
}

fn key_entry() -> Result<keyring::Entry, AppError> {
    Ok(keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?)
}

pub fn saved_key() -> Option<String> {
    match key_entry().ok()?.get_password() {
        Ok(key) if valid_key(key.trim()) => Some(key.trim().to_string()),
        _ => None,
    }
}

/// Save the personal key Soloist starts with.
pub fn save_key(key: &str) -> Result<(), String> {
    let key = key.trim();
    if !valid_key(key) {
        return Err("That doesn't look like a Soloist key.".into());
    }
    key_entry()
        .and_then(|entry| Ok(entry.set_password(key)?))
        .map_err(|e| format!("Couldn't save the Soloist key: {e}"))
}

pub fn forget_key() {
    if let Ok(entry) = key_entry() {
        let _ = entry.delete_password();
    }
}

fn root(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("soloist")
}

/// Remove Soloist's pairing, cache and key: what signing out leaves behind
/// otherwise. The downloaded builds stay; they belong to nobody.
pub async fn forget(app: &AppHandle) {
    forget_key();
    let _ = tokio::fs::remove_dir_all(root(app).join("data")).await;
}

// ── running it ──────────────────────────────────────────────────────────────

enum End {
    /// Asked to stop.
    Stopped,
    /// There is no key to start with.
    NeedsKey,
    /// A newer build is staged and the player was idle: start that one.
    Restart,
}

async fn supervise(
    app: AppHandle,
    shared: Arc<Shared>,
    key: Option<String>,
    mut stopped: watch::Receiver<bool>,
    mut inbox: mpsc::UnboundedReceiver<Value>,
) {
    let mut key = key;
    let mut failures = 0usize;
    loop {
        let began = Instant::now();
        let result = session(&app, &shared, key.take(), &mut stopped, &mut inbox).await;
        // the app's place in its track outlives the player: playing on after
        // a restart starts it again from there
        let (playing, point) = {
            let mut play = shared.play();
            let playing = play.started && !play.ended && matches!(play.status.as_str(), "playing" | "buffering");
            play.keep_place(unix_ms());
            (playing, play.resume_point())
        };
        match result {
            Ok(End::Stopped) => return,
            Ok(End::NeedsKey) => {
                shared.set_status(&app, Status::NeedsKey);
                return;
            }
            Ok(End::Restart) => continue,
            Err(message) => {
                log::warn!("soloist stopped: {message}");
                if let (true, Some((uri, at))) = (playing, point) {
                    // say so rather than leave the interface playing silence,
                    // and carry on by itself once the player is back
                    report(&app, PlayerMsg::Paused { track_id: Some(id_of(&uri)), position_ms: at });
                    *lock(&shared.resume) = Some((uri, at));
                }
                // a build that was only just moved to gets no second chance
                let _ = tokio::fs::remove_file(root(&app).join("builds").join("pending.json")).await;
                if began.elapsed() >= HEALTHY_AFTER {
                    failures = 0;
                }
                let Some(wait) = RESTART_AFTER.get(failures).copied() else {
                    *lock(&shared.resume) = None;
                    shared.set_status(&app, Status::Failed { message });
                    return;
                };
                failures += 1;
                shared.set_status(&app, Status::Starting);
                tokio::select! {
                    _ = stopped.changed() => return,
                    _ = tokio::time::sleep(wait) => {}
                }
            }
        }
        if *stopped.borrow() {
            return;
        }
    }
}

/// A name that tells this computer's Soloist from another's in Spotify's
/// device list: two computers often share a user name and folder layout.
async fn device_name(root: &Path) -> String {
    let path = root.join("instance-id");
    let id = match tokio::fs::read_to_string(&path).await {
        Ok(id) if id.trim().len() == 8 && id.trim().bytes().all(|b| b.is_ascii_hexdigit()) => id.trim().to_string(),
        _ => {
            let id = format!("{:08x}", rand::random::<u32>());
            let _ = tokio::fs::write(&path, &id).await;
            id
        }
    };
    name_for(&id)
}

/// One run of the player, from launch until it stops or fails.
async fn session(
    app: &AppHandle,
    shared: &Arc<Shared>,
    key: Option<String>,
    stopped: &mut watch::Receiver<bool>,
    inbox: &mut mpsc::UnboundedReceiver<Value>,
) -> Result<End, String> {
    let key = match key.map(|key| key.trim().to_string()) {
        Some(key) => {
            if !valid_key(&key) {
                return Err("That does not look like a Soloist key.".into());
            }
            key_entry()
                .and_then(|entry| Ok(entry.set_password(&key)?))
                .map_err(|e| format!("Could not save the Soloist key: {e}"))?;
            key
        }
        None => match saved_key() {
            Some(key) => key,
            None => return Ok(End::NeedsKey),
        },
    };

    let root = root(app);
    let builds = root.join("builds");
    let data = root.join("data");
    for dir in [&builds, &data] {
        tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    }
    let name = device_name(&root).await;
    *lock(&shared.name) = Some(name.clone());

    let mut build = match install::saved(&builds, "pending.json").await {
        Some(build) => build,
        None => match install::saved(&builds, "current.json").await {
            Some(build) => build,
            None => fetch(app, shared, &builds, stopped).await?,
        },
    };
    let mut keep = vec![build.digest.clone()];
    if let Some(previous) = install::saved(&builds, "current.json").await {
        keep.push(previous.digest);
    }
    install::collect(&builds, &keep).await;

    let mut redownloaded = false;
    loop {
        match run(app, shared, &key, &name, &builds, &data, &build, stopped, inbox).await? {
            Ran::Ended(end) => return Ok(end),
            Ran::Expired if !redownloaded => {
                redownloaded = true;
                let _ = tokio::fs::remove_file(builds.join("current.json")).await;
                build = fetch(app, shared, &builds, stopped).await?;
            }
            Ran::Expired => {
                return Err("Spotify has not published a Soloist build that still works. Try again later.".into());
            }
        }
    }
}

async fn fetch(
    app: &AppHandle,
    shared: &Shared,
    builds: &Path,
    stopped: &mut watch::Receiver<bool>,
) -> Result<Build, String> {
    let progress = |received, total| shared.set_status(app, Status::Downloading { received, total });
    tokio::select! {
        _ = stopped.changed() => Err("stopped".into()),
        build = install::download(builds, &progress) => build,
    }
}

enum Ran {
    Ended(End),
    /// Soloist said its build is too old to run.
    Expired,
}

type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(port: u16) -> Result<Socket, String> {
    tokio::time::timeout(
        Duration::from_secs(5),
        tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}")),
    )
    .await
    .map_err(|_| "Soloist's local API did not answer in time".to_string())?
    .map(|(socket, _)| socket)
    .map_err(|e| format!("Could not reach Soloist's local API: {e}"))
}

async fn send(socket: &mut Socket, mut command: Value) -> Result<(), String> {
    command["type"] = json!("command");
    socket
        .send(Message::Text(command.to_string().into()))
        .await
        .map_err(|e| format!("Soloist disconnected: {e}"))
}

/// The next JSON message from Soloist.
async fn receive(socket: &mut Socket) -> Result<Value, String> {
    loop {
        match socket.next().await {
            Some(Ok(Message::Text(text))) => {
                if let Ok(message) = serde_json::from_str(text.as_str()) {
                    return Ok(message);
                }
            }
            Some(Ok(Message::Close(_))) | None => return Err("Soloist disconnected".into()),
            Some(Ok(_)) => {}
            Some(Err(e)) => return Err(format!("Soloist disconnected: {e}")),
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run(
    app: &AppHandle,
    shared: &Arc<Shared>,
    key: &str,
    name: &str,
    builds: &Path,
    data: &Path,
    build: &Build,
    stopped: &mut watch::Receiver<bool>,
    inbox: &mut mpsc::UnboundedReceiver<Value>,
) -> Result<Ran, String> {
    shared.set_status(app, Status::Starting);
    // Soloist writes where it is listening into these; stale ones would send
    // us to a port something else has since taken
    for file in ["ws.addr", "ws.port"] {
        let _ = tokio::fs::remove_file(data.join(file)).await;
    }
    // a copy an earlier run left behind holds the data folder and the device
    // name; it has to go before this one can start
    let ours = builds.to_owned();
    let _ = tokio::task::spawn_blocking(move || reclaim(&ours)).await;

    // Soloist starts at a level of its own (40) unless it is told otherwise
    let volume = soloist_volume(saved_volume(app).await).to_string();
    let pool = app.state::<crate::state::AppState>().db.clone();
    let quality = saved_quality(&pool).await;
    // the disk cache limit in Settings is Soloist's too (it takes 100 MB at least)
    let cache_mb = sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = 'audio_cache_limit_mb'")
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten()
        .and_then(|(value,)| value.parse::<u64>().ok())
        .unwrap_or(2048)
        .clamp(100, 65_536)
        .to_string();
    let at = data.to_owned();
    let _ = tokio::task::spawn_blocking(move || write_prefs(&at, quality)).await;
    *lock(&shared.format) = None;
    let mut command = tokio::process::Command::new(build.path(builds));
    command
        .args(["--device-name", name, "--api-key", key, "--ws", "127.0.0.1:0", "--cache-size", &cache_mb])
        .args(["--initial-volume", &volume])
        .arg("--data-dir")
        .arg(data)
        .arg("--cache-dir")
        .arg(data.join("cache"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    // its verbose log is the only place the format it streams is reported
    // (see `read_format`); all of it lands in soloist-output.log
    command.arg("--verbose");
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // no console window
    // Soloist only takes the key as an argument. This command line and what
    // the child prints are never written to the app's own log.
    let mut child = command.spawn().map_err(|e| format!("Could not start Soloist: {e}"))?;
    let format_app = app.clone();
    let format_shared = Arc::clone(shared);
    let mut format = Format::default();
    keep_output(data.join("soloist-output.log"), key.to_string(), &mut child, move |line| {
        if read_format(line, &mut format) {
            *lock(&format_shared.format) = Some(format.clone());
            let _ = format_app.emit("soloist:format", format.clone());
        }
    });
    raise_priority(&child);
    bind_to_app(&child);

    let outcome = tokio::select! {
        _ = stopped.changed() => Ok(Ran::Ended(End::Stopped)),
        outcome = drive(app, shared, name, builds, data, build, &mut child, inbox) => outcome,
    };
    let _ = child.kill().await;
    let _ = child.wait().await;
    outcome
}

/// How many times a Soloist that is not signed in is signed in by the app,
/// a tick apart, before it is left to be picked in a Spotify app.
const PAIR_TRIES: usize = 5;

/// Signs Soloist in with the account the app plays with, over Spotify
/// Connect's local sign-in, as picking it in a Spotify app's device list
/// would. A moment after it starts listening it may not be advertised yet,
/// so this waits a little first.
fn pair(app: &AppHandle, name: &str) -> tokio::task::JoinHandle<Result<bool, AppError>> {
    let (app, name) = (app.clone(), name.to_string());
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        crate::commands::receivers::sign_in_local(&app, &name).await
    })
}

/// Soloist's stream is fed by threads at normal priority, so on a busy
/// machine one that wakes late leaves a hole in the music. Above normal is
/// enough to be served first without ever starving the desktop, as realtime
/// could in a player whose code is not ours.
#[cfg(windows)]
fn raise_priority(child: &tokio::process::Child) {
    use windows_sys::Win32::System::Threading::{SetPriorityClass, ABOVE_NORMAL_PRIORITY_CLASS};
    let Some(handle) = child.raw_handle() else { return };
    // SAFETY: the handle is the live child's, owned by `child` for this call
    if unsafe { SetPriorityClass(handle as _, ABOVE_NORMAL_PRIORITY_CLASS) } == 0 {
        log::debug!("soloist stays at normal priority: {}", std::io::Error::last_os_error());
    }
}

#[cfg(not(windows))]
fn raise_priority(_child: &tokio::process::Child) {}

/// Tie the player's life to the app's: on Windows a child outlives a parent
/// that is killed or crashes, and a Soloist left behind plays on with
/// nothing to pause it. A job that kills what is in it when its last handle
/// closes (the app's, at exit of any kind) prevents that. The next start
/// would reclaim it (see `reclaim`), but only once the app runs again.
#[cfg(windows)]
fn bind_to_app(child: &tokio::process::Child) {
    use std::sync::OnceLock;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    // one job for the app's lifetime; its handle is never closed by hand
    static JOB: OnceLock<usize> = OnceLock::new();
    let job = *JOB.get_or_init(|| {
        // SAFETY: plain Win32 calls on a handle this function owns
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return 0;
            }
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            job as usize
        }
    });
    let Some(handle) = child.raw_handle() else { return };
    // SAFETY: the job handle lives for the app; the child's for this call
    if job == 0 || unsafe { AssignProcessToJobObject(job as _, handle as _) } == 0 {
        log::debug!("soloist is not tied to the app: {}", std::io::Error::last_os_error());
    }
}

#[cfg(not(windows))]
fn bind_to_app(_child: &tokio::process::Child) {}

/// Wait for the child to listen, then relay between it and the app until
/// something ends the session.
#[allow(clippy::too_many_arguments)]
async fn drive(
    app: &AppHandle,
    shared: &Shared,
    name: &str,
    builds: &Path,
    data: &Path,
    build: &Build,
    child: &mut tokio::process::Child,
    inbox: &mut mpsc::UnboundedReceiver<Value>,
) -> Result<Ran, String> {
    let started = Instant::now();
    let port = loop {
        if let Some(exit) = child.try_wait().map_err(|e| e.to_string())? {
            if exit.code() == Some(EXPIRED_EXIT) {
                return Ok(Ran::Expired);
            }
            return Err(format!("Soloist exited before it was ready ({exit})"));
        }
        let port = tokio::fs::read_to_string(data.join("ws.port")).await.ok();
        if let Some(port) = port.and_then(|text| text.trim().parse::<u16>().ok()).filter(|port| *port != 0) {
            let address = tokio::fs::read_to_string(data.join("ws.addr")).await.unwrap_or_default();
            if address.trim() != "127.0.0.1" {
                return Err("Soloist did not keep its API to this computer".into());
            }
            break port;
        }
        if started.elapsed() > START_LIMIT {
            return Err("Soloist took too long to start".into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    let mut socket = connect(port).await?;

    // it runs: this is the build to start from now on
    install::remember(builds, "current.json", build).await.map_err(|e| e.to_string())?;
    let _ = tokio::fs::remove_file(builds.join("pending.json")).await;
    shared.set_status(app, Status::Pairing { name: name.into(), version: build.version.clone() });

    let mut logged_in = false;
    // until it is signed in, Soloist waits on the network to be picked from a
    // Spotify app's device list. the playback session signs it in the same
    // way instead, so it pairs by itself (see `pair`)
    let mut pairing = Some(pair(app, name));
    let mut pair_tries = 1usize;
    let mut reconnected: Option<Instant> = None;
    let mut checked = build.checked;
    let mut download: Option<tokio::task::JoinHandle<Result<Build, String>>> = None;
    let mut pending: Option<Build> = None;
    let mut tick = tokio::time::interval(TICK);
    // Soloist only speaks up when playback changes, never while it simply
    // plays on. Without asking, the interface extrapolates for minutes at a
    // time and every rebuffer or clock difference becomes lyric drift. Its
    // answer carries the exact position and when it was true.
    let mut position_poll = tokio::time::interval(POSITION_POLL);
    position_poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = position_poll.tick() => {
                let playing = logged_in && shared.play().wants_poll();
                if playing {
                    send(&mut socket, json!({"command": "get_state"})).await?;
                }
            }
            exit = child.wait() => {
                let exit = exit.map_err(|e| e.to_string())?;
                if exit.code() == Some(EXPIRED_EXIT) {
                    return Ok(Ran::Expired);
                }
                return Err(format!("Soloist exited ({exit})"));
            }
            command = inbox.recv() => {
                let Some(command) = command else {
                    return Ok(Ran::Ended(End::Stopped));
                };
                send(&mut socket, command).await?;
            }
            _ = tick.tick() => {
                send(&mut socket, json!({"command": "get_auth_state"})).await?;
                if pairing.as_ref().is_some_and(|task| task.is_finished()) {
                    match pairing.take().expect("finished pairing").await {
                        Ok(Ok(true)) => eprintln!("[soloist] handed it this account to sign in with"),
                        Ok(Ok(false)) => eprintln!("[soloist] not on the network yet"),
                        Ok(Err(error)) => eprintln!("[soloist] could not sign it in: {error}"),
                        Err(error) => eprintln!("[soloist] sign-in stopped: {error}"),
                    }
                }
                if !logged_in && pairing.is_none() && pair_tries < PAIR_TRIES {
                    pair_tries += 1;
                    pairing = Some(pair(app, name));
                }
                let mut current = build.clone();
                current.checked = checked;
                if current.due(install::now()) && download.is_none() && pending.is_none() {
                    checked = install::now();
                    let root = builds.to_owned();
                    download = Some(tokio::spawn(async move { install::download(&root, &|_, _| {}).await }));
                }
                if download.as_ref().is_some_and(|task| task.is_finished()) {
                    let fetched = download.take().expect("finished download").await;
                    match fetched {
                        Ok(Ok(mut next)) if next.digest != build.digest && next.built > build.built => {
                            next.checked = checked;
                            let _ = install::remember(builds, "pending.json", &next).await;
                            log::info!("soloist {} is ready to move to", next.version);
                            pending = Some(next);
                        }
                        Ok(Ok(_)) => {
                            // Spotify has nothing newer yet; ask again tomorrow
                            current.checked = checked;
                            let _ = install::remember(builds, "current.json", &current).await;
                        }
                        Ok(Err(error)) => log::warn!("soloist update check failed: {error}"),
                        Err(error) => log::warn!("soloist update check failed: {error}"),
                    }
                }
                // moving to it restarts the player, so only when nothing plays
                let idle = { let play = shared.play(); !matches!(play.status.as_str(), "playing" | "buffering" | "paused") };
                if pending.is_some() && idle {
                    return Ok(Ran::Ended(End::Restart));
                }
            }
            message = receive(&mut socket) => {
                let message = match message {
                    Ok(message) => message,
                    Err(error) => {
                        // one drop is a hiccup; a second straight after is a
                        // player that will not hold a connection
                        if reconnected.is_some_and(|at| at.elapsed() < Duration::from_secs(30)) {
                            return Err(error);
                        }
                        reconnected = Some(Instant::now());
                        socket = connect(port).await?;
                        continue;
                    }
                };
                if message["type"] == "auth_state" {
                    let now_in = message["logged_in"].as_bool().unwrap_or(false);
                    if now_in != logged_in {
                        logged_in = now_in;
                        if logged_in {
                            // Soloist starts at a level of its own every launch
                            let volume = saved_volume(app).await;
                            shared.volume.store(volume, Ordering::Relaxed);
                            // the track that was playing when the last copy
                            // fell over carries on from where it was
                            let resume = lock(&shared.resume).take();
                            let quiet = resume.as_ref().is_some_and(|(_, at)| *at > 0);
                            send(&mut socket, json!({"command": "set_volume", "volume": if quiet { 0 } else { soloist_volume(volume) }})).await?;
                            if let Some((uri, at)) = resume {
                                shared.play().request(&uri, at);
                                send(&mut socket, json!({"command": "play", "uri": uri})).await?;
                            }
                            send(&mut socket, json!({"command": "get_state"})).await?;
                            shared.set_status(app, Status::Ready {
                                name: name.into(),
                                version: build.version.clone(),
                                expires: build.expires(),
                            });
                        } else {
                            shared.set_status(app, Status::Pairing { name: name.into(), version: build.version.clone() });
                            pair_tries = 0;
                        }
                    }
                    continue;
                }
                if message["type"] == "error" {
                    log::warn!("soloist refused a command: {}", message["message"].as_str().unwrap_or_default());
                }
                let outs = shared.play().observe(&message, unix_ms());
                for out in outs {
                    match out {
                        Out::Command(command) => send(&mut socket, command).await?,
                        Out::Event(event) => report(app, event),
                        Out::Unmute => {
                            let volume = shared.volume.load(Ordering::Relaxed);
                            send(&mut socket, json!({"command": "set_volume", "volume": soloist_volume(volume)})).await?;
                        }
                    }
                }
            }
        }
    }
}

/// The level the listener left the volume at, as Soloist counts it.
async fn saved_volume(app: &AppHandle) -> u8 {
    let pool = app.state::<crate::state::AppState>().db.clone();
    let read = |key: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten()
                .map(|(value,)| value)
        }
    };
    if read("player_muted").await.as_deref() == Some("1") {
        return 0;
    }
    read("player_volume")
        .await
        .and_then(|value| value.parse::<f64>().ok())
        .map_or(80, |level| level.clamp(0.0, 100.0) as u8)
}

/// Pass a playback event on, to the interface and to the system's media
/// controls, the way the other backends do.
fn report(app: &AppHandle, event: PlayerMsg) {
    use crate::media_controls::MediaMsg;
    let media = match &event {
        PlayerMsg::Playing { position_ms, .. } => Some(MediaMsg::Playing { position_ms: u64::from(*position_ms) }),
        PlayerMsg::Paused { position_ms, .. } => Some(MediaMsg::Paused { position_ms: u64::from(*position_ms) }),
        PlayerMsg::EndOfTrack { .. } | PlayerMsg::Stopped { .. } => Some(MediaMsg::Stopped),
        _ => None,
    };
    if let Some(media) = media {
        let _ = app.state::<crate::state::AppState>().media_tx.try_send(media);
    }
    if let Err(e) = app.emit("player:event", event) {
        log::warn!("soloist: emit failed: {e}");
    }
}

/// Keep what Soloist prints, for a bug report, with the key taken out. Each
/// start replaces the file, and it stops growing at `OUTPUT_LIMIT`.
fn keep_output(
    path: PathBuf,
    key: String,
    child: &mut tokio::process::Child,
    mut on_line: impl FnMut(&str) + Send + 'static,
) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (lines, mut all) = mpsc::unbounded_channel::<String>();
    fn pump<R: tokio::io::AsyncRead + Unpin + Send + 'static>(reader: R, lines: mpsc::UnboundedSender<String>) {
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if lines.send(line).is_err() {
                    break;
                }
            }
        });
    }
    if let Some(stdout) = child.stdout.take() {
        pump(stdout, lines.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        pump(stderr, lines);
    }
    tokio::spawn(async move {
        let Ok(mut file) = tokio::fs::File::create(&path).await else {
            // still drained, or the child would block on a full pipe
            while let Some(line) = all.recv().await {
                on_line(&line);
            }
            return;
        };
        let mut written = 0usize;
        while let Some(line) = all.recv().await {
            on_line(&line);
            if written >= OUTPUT_LIMIT {
                continue;
            }
            let line = line.replace(&key, "[key]");
            written += line.len() + 1;
            let _ = file.write_all(line.as_bytes()).await;
            let _ = file.write_all(b"\n").await;
        }
    });
}

/// Stop any Soloist still running out of this app's own builds folder.
/// Nothing else called soloist is touched.
fn reclaim(builds: &Path) {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

    let Ok(builds) = builds.canonicalize() else {
        return;
    };
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );
    for process in system.processes().values() {
        let ours = process
            .exe()
            .and_then(|exe| exe.canonicalize().ok())
            .is_some_and(|exe| exe.starts_with(&builds));
        if ours && process.kill() {
            log::warn!("stopped a Soloist an earlier run left behind");
            process.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[tokio::test]
    async fn the_player_runs_above_normal_priority() {
        use windows_sys::Win32::System::Threading::{GetPriorityClass, ABOVE_NORMAL_PRIORITY_CLASS};
        let mut child = tokio::process::Command::new("cmd")
            .args(["/C", "ping -n 3 127.0.0.1 >NUL"])
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        super::raise_priority(&child);
        let class = unsafe { GetPriorityClass(child.raw_handle().unwrap() as _) };
        let _ = child.kill().await;
        assert_eq!(class, ABOVE_NORMAL_PRIORITY_CLASS);
    }

    use super::*;

    const A: &str = "spotify:track:aaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "spotify:track:bbbbbbbbbbbbbbbbbbbbbb";

    fn item(uri: &str) -> Value {
        json!({"uri": uri, "entity_type": "track", "decorations": {"playback": {"duration_ms": 200_000}}})
    }

    fn id(uri: &str) -> Option<String> {
        Some(id_of(uri))
    }

    /// The app asks for `uri` from its start, and Soloist takes the play.
    fn ask(play: &mut Play, uri: &str) {
        play.request(uri, 0);
        assert!(play.observe(&json!({"type": "command_result", "command": "play"}), 0).is_empty());
    }

    fn state(status: &str, uri: &str, position_ms: u64, at: u64, active: bool) -> Value {
        json!({"type": "playback_state", "status": status, "item": item(uri), "is_active": active,
            "position": {"position_ms": position_ms, "timestamp_ms": at, "speed": if status == "playing" { 1.0 } else { 0.0 }}})
    }

    #[test]
    fn what_soloist_was_on_before_the_play_is_taken_is_not_reported() {
        let mut play = Play::default();
        play.request(A, 0);
        // becoming active restores the account's last state: this very track,
        // a minute in. Reporting it would flash the wrong position
        assert!(play.observe(&state("playing", A, 64_923, 1000, true), 1000).is_empty());
        assert!(play.observe(&json!({"type": "command_result", "command": "play"}), 1001).is_empty());
        assert!(play.observe(&state("buffering", A, 0, 1100, true), 1100).is_empty());
        assert_eq!(
            play.observe(&state("playing", A, 3, 1200, true), 1200),
            vec![Out::Event(PlayerMsg::Playing { track_id: id(A), position_ms: 3 })]
        );
    }

    #[test]
    fn a_start_part_way_through_is_reported_from_where_it_starts() {
        let mut play = Play::default();
        play.request(A, 90_000);
        play.observe(&json!({"type": "command_result", "command": "play"}), 0);
        // what it restored on becoming active, and the load: no seek yet,
        // Soloist would drop it
        assert!(play.observe(&state("paused", B, 64_913, 50, true), 50).is_empty());
        assert!(play.observe(&state("buffering", A, 0, 80, true), 80).is_empty());
        // playing at the top: now the seek goes, and the lyrics hear nothing
        assert_eq!(
            play.observe(&state("playing", A, 3, 100, true), 100),
            vec![Out::Command(json!({"command": "seek", "position_ms": 90_000}))]
        );
        assert!(play.observe(&json!({"type": "command_result", "command": "seek"}), 101).is_empty());
        assert!(play.observe(&state("playing", A, 400, 120, true), 120).is_empty());
        assert!(play.observe(&state("buffering", A, 90_000, 150, true), 150).is_empty());
        assert_eq!(
            play.observe(&state("playing", A, 90_004, 260, true), 260),
            vec![Out::Unmute, Out::Event(PlayerMsg::Playing { track_id: id(A), position_ms: 90_004 })]
        );
    }

    #[test]
    fn a_seek_that_never_lands_is_given_up_on_and_the_sound_comes_back() {
        let mut play = Play::default();
        play.request(A, 90_000);
        play.observe(&json!({"type": "command_result", "command": "play"}), 0);
        assert_eq!(play.observe(&state("playing", A, 3, 100, true), 100).len(), 1);
        assert!(play.wants_poll());
        assert!(play.observe(&state("playing", A, 1_100, 1_200, true), 1_200).is_empty());
        let outs = play.observe(&state("playing", A, 4_300, 4_400, true), 4_400);
        assert_eq!(outs[0], Out::Unmute);
        assert_eq!(outs[1], Out::Event(PlayerMsg::Playing { track_id: id(A), position_ms: 4_300 }));
    }

    #[test]
    fn another_device_taking_over_is_a_pause_and_nothing_is_sent_to_it() {
        let mut play = Play::default();
        ask(&mut play, A);
        play.observe(&state("playing", A, 0, 1000, true), 1000);
        assert_eq!(
            play.observe(&json!({"type": "device_changed", "is_active": false}), 31_000),
            vec![Out::Event(PlayerMsg::Paused { track_id: id(A), position_ms: 30_000 })]
        );
        // what the other device then plays is not this track's end, and no
        // pause goes out that would stop the other device
        assert!(play.observe(&state("playing", B, 5_000, 32_000, false), 32_000).is_empty());
        assert!(play.observe(&json!({"type": "track_changed", "item": item(B)}), 32_100).is_empty());
        // playing on starts it again where it was taken away
        assert_eq!(play.resume_point(), Some((A.to_string(), 30_000)));
    }

    #[test]
    fn a_refused_play_is_a_track_that_did_not_load() {
        let mut play = Play::default();
        play.request(A, 0);
        assert_eq!(
            play.observe(&json!({"type": "error", "message": "track unavailable"}), 0),
            vec![Out::Event(PlayerMsg::Unavailable { track_id: id(A) })]
        );
        // and an error after the play was taken is not
        ask(&mut play, B);
        assert!(play.observe(&json!({"type": "error", "message": "x"}), 0).is_empty());
    }

    #[test]
    fn a_restart_keeps_the_place_in_the_track() {
        let mut play = Play::default();
        ask(&mut play, A);
        play.observe(&state("playing", A, 10_000, 1000, true), 1000);
        play.keep_place(6000);
        assert!(!play.started);
        assert_eq!(play.position_now(60_000), 15_000);
        assert_eq!(play.resume_point(), Some((A.to_string(), 15_000)));
        // an ended track starts again from the top
        play.ended = true;
        assert_eq!(play.resume_point(), Some((A.to_string(), 0)));
    }

    #[test]
    fn the_format_is_read_from_the_verbose_log() {
        let mut format = Format::default();
        let lines = [
            "10:02:45.178 I [/f:134                          ] FileStreamer file average bitrate: 160000",
            "10:02:48.986 I [/f:388                          ] vorbis_info.bitrate_nominal: 320000",
            "10:02:48.988 I [/f:347                          ]     codec: vorbis",
            "10:02:48.988 I [/f:348                          ]     bitrate: 320000 bits/s",
            "10:02:48.988 I [/f:349                          ]     sample rate: 44100 Hz",
            "10:02:48.988 I [/f:350                          ]     sample format: 32-bit-float-pcm",
        ];
        let changed: Vec<bool> = lines.iter().map(|line| read_format(line, &mut format)).collect();
        assert_eq!(changed, [false, false, true, true, true, false]);
        assert_eq!(format, Format { codec: "vorbis".into(), bitrate: Some(320_000), sample_rate: Some(44_100) });
        // the next file starts afresh
        assert!(read_format("x I [/f:347 ]     codec: FLAC", &mut format));
        assert_eq!(format, Format { codec: "flac".into(), bitrate: None, sample_rate: None });
    }

    #[test]
    fn the_preferences_are_written_where_soloist_reads_them_and_nothing_else_changes() {
        let data = std::env::temp_dir().join(format!("musique-soloist-prefs-{:08x}", rand::random::<u32>()));
        let user = data.join("settings").join("Users").join("someone-user");
        std::fs::create_dir_all(&user).unwrap();
        std::fs::write(data.join("settings").join("prefs"), "storage.size=512\ncore.clock_delta=-1\n").unwrap();
        std::fs::write(user.join("prefs"), "audio.gapless_migrated_to_ps=true\n").unwrap();
        write_prefs(&data, Quality::VeryHigh);
        assert_eq!(
            std::fs::read_to_string(data.join("settings").join("prefs")).unwrap(),
            "storage.size=512\ncore.clock_delta=-1\naudio.play_bitrate_enumeration=4\naudio.play_bitrate_non_metered_enumeration=4\n"
        );
        // a per-account file without the setting is left alone
        assert_eq!(std::fs::read_to_string(user.join("prefs")).unwrap(), "audio.gapless_migrated_to_ps=true\n");
        // changing it replaces the value rather than adding another line
        std::fs::write(user.join("prefs"), "audio.play_bitrate_enumeration=2\n").unwrap();
        write_prefs(&data, Quality::Lossless);
        let global = std::fs::read_to_string(data.join("settings").join("prefs")).unwrap();
        assert_eq!(global.matches("audio.play_bitrate_enumeration=").count(), 1);
        assert!(global.contains("audio.play_bitrate_enumeration=5"));
        assert!(std::fs::read_to_string(user.join("prefs")).unwrap().contains("audio.play_bitrate_enumeration=5"));
        // a first run, before Soloist has written anything
        let fresh = data.join("fresh");
        write_prefs(&fresh, Quality::High);
        assert!(std::fs::read_to_string(fresh.join("settings").join("prefs")).unwrap().contains("audio.play_bitrate_enumeration=3"));
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn keys_are_one_word_and_never_an_option() {
        assert!(valid_key("abcDEF123-_="));
        assert!(!valid_key(""));
        assert!(!valid_key("--pair"));
        assert!(!valid_key("key --ws 0.0.0.0:1"));
        assert!(!valid_key("key\n"));
    }

    #[test]
    fn a_requested_track_plays_pauses_and_ends() {
        let mut play = Play::default();
        // what Soloist was doing before is not the app's business
        assert!(play.observe(&json!({"type": "playback_changed", "status": "paused"}), 0).is_empty());

        ask(&mut play, A);
        assert!(play.observe(&json!({"type": "track_changed", "item": item(A)}), 1000).is_empty());
        let now = 10_000;
        let position = json!({"position_ms": 0, "timestamp_ms": now, "speed": 1.0});
        assert!(play.observe(&json!({"type": "position_sync", "position": position}), now).is_empty());
        assert_eq!(
            play.observe(&json!({"type": "playback_changed", "status": "playing"}), now + 500),
            vec![Out::Event(PlayerMsg::Playing { track_id: id(A), position_ms: 500 })]
        );
        assert_eq!(play.position_now(now + 60_000), 60_000);

        // a pause the app asked for, mid-track
        play.user_paused = true;
        let position = json!({"position_ms": 60_000, "timestamp_ms": now + 60_000, "speed": 0.0});
        assert_eq!(
            play.observe(&json!({"type": "position_sync", "position": position}), now + 60_000),
            vec![Out::Event(PlayerMsg::PositionChanged { track_id: id(A), position_ms: 60_000 })]
        );
        assert_eq!(
            play.observe(&json!({"type": "playback_changed", "status": "paused"}), now + 61_000),
            vec![Out::Event(PlayerMsg::Paused { track_id: id(A), position_ms: 60_000 })]
        );

        // going idle after playing is the end, told once
        play.user_paused = false;
        assert_eq!(
            play.observe(&json!({"type": "playback_changed", "status": "idle"}), now + 62_000),
            vec![Out::Event(PlayerMsg::EndOfTrack { track_id: id(A) })]
        );
        assert!(play.observe(&json!({"type": "playback_changed", "status": "idle"}), now + 63_000).is_empty());
    }

    #[test]
    fn a_polled_state_while_playing_is_only_a_position_update() {
        let mut play = Play::default();
        ask(&mut play, A);
        let first = play.observe(&json!({"type": "playback_state", "status": "playing", "item": item(A),
            "position": {"position_ms": 0, "timestamp_ms": 1000, "speed": 1.0}}), 1000);
        assert_eq!(first, vec![Out::Event(PlayerMsg::Playing { track_id: id(A), position_ms: 0 })]);
        // the once-a-second answer: Soloist fell 300ms behind the wall clock
        // (a rebuffer), and its own position is what gets reported
        assert_eq!(
            play.observe(&json!({"type": "playback_state", "status": "playing", "item": item(A),
                "position": {"position_ms": 9_700, "timestamp_ms": 11_000, "speed": 1.0}}), 11_000),
            vec![Out::Event(PlayerMsg::PositionChanged { track_id: id(A), position_ms: 9_700 })]
        );
    }

    #[test]
    fn soloist_moving_on_by_itself_is_the_end_of_the_track() {
        let mut play = Play::default();
        ask(&mut play, A);
        play.observe(&json!({"type": "playback_state", "status": "playing", "item": item(A), "is_active": true,
            "position": {"position_ms": 0, "timestamp_ms": 5, "speed": 1.0}}), 5);
        assert!(play.started);
        assert_eq!(
            play.observe(&json!({"type": "track_changed", "item": item(B)}), 200_000),
            vec![
                Out::Command(json!({"command": "pause"})),
                Out::Event(PlayerMsg::EndOfTrack { track_id: id(A) }),
            ]
        );
        // and nothing more of the track it chose reaches the interface
        assert!(play.observe(&json!({"type": "playback_changed", "status": "playing"}), 200_100).is_empty());

        // the next request starts clean
        ask(&mut play, B);
        assert_eq!(
            play.observe(&json!({"type": "playback_changed", "status": "playing"}), 200_200).len(),
            1
        );
    }

    #[test]
    fn a_pause_at_the_end_nobody_asked_for_is_the_end() {
        let mut play = Play::default();
        ask(&mut play, A);
        play.observe(&json!({"type": "playback_state", "status": "playing", "item": item(A),
            "position": {"position_ms": 199_000, "timestamp_ms": 1000, "speed": 1.0}}), 1000);
        // playing on to the end, then stopping there with the position reset
        play.observe(&json!({"type": "position_sync",
            "position": {"position_ms": 0, "timestamp_ms": 2500, "speed": 0.0}}), 2500);
        assert_eq!(
            play.observe(&json!({"type": "playback_changed", "status": "paused"}), 2600),
            vec![Out::Event(PlayerMsg::EndOfTrack { track_id: id(A) })]
        );
    }

    #[test]
    fn a_track_spotify_plays_under_another_uri_is_still_the_one_asked_for() {
        let mut play = Play::default();
        ask(&mut play, A);
        assert!(play.observe(&json!({"type": "track_changed", "item": item(B)}), 0).is_empty());
        assert_eq!(
            play.observe(&json!({"type": "playback_changed", "status": "playing"}), 0),
            vec![Out::Event(PlayerMsg::Playing { track_id: id(A), position_ms: 0 })]
        );
    }

    /// Starts Spotify's real player the way `run` does, with a key that is
    /// not one, and talks to its local API as far as that goes without an
    /// account. Point `MUSIQUE_SOLOIST_EXE` at a downloaded `soloist` and run
    /// `cargo test -- --ignored soloist`.
    #[tokio::test]
    #[ignore]
    async fn live_the_real_player_listens_where_it_says_and_speaks_the_protocol() {
        let Some(exe) = std::env::var_os("MUSIQUE_SOLOIST_EXE") else {
            return;
        };
        let data = std::env::temp_dir().join(format!("musique-soloist-run-{:016x}", rand::random::<u64>()));
        std::fs::create_dir_all(&data).unwrap();
        let mut command = tokio::process::Command::new(exe);
        command
            .args(["--device-name", "Musique Soloist (test)", "--api-key", "not-a-real-key", "--ws", "127.0.0.1:0", "--cache-size", "512", "--initial-volume", "50"])
            .arg("--data-dir")
            .arg(&data)
            .arg("--cache-dir")
            .arg(data.join("cache"))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x0800_0000);
        let mut child = command.spawn().unwrap();
        keep_output(data.join("soloist-output.log"), "not-a-real-key".into(), &mut child, |_| {});

        let started = Instant::now();
        let port = loop {
            if let Some(exit) = child.try_wait().unwrap() {
                let output = std::fs::read_to_string(data.join("soloist-output.log")).unwrap_or_default();
                panic!("soloist exited before listening ({exit}):
{output}");
            }
            let port = std::fs::read_to_string(data.join("ws.port")).ok().and_then(|text| text.trim().parse::<u16>().ok());
            if let Some(port) = port.filter(|port| *port != 0) {
                break port;
            }
            assert!(started.elapsed() < START_LIMIT, "no ws.port");
            tokio::time::sleep(Duration::from_millis(100)).await;
        };
        assert_eq!(std::fs::read_to_string(data.join("ws.addr")).unwrap().trim(), "127.0.0.1");

        let mut socket = connect(port).await.unwrap();
        let hello = tokio::time::timeout(Duration::from_secs(5), receive(&mut socket)).await.unwrap().unwrap();
        println!("on connect: {hello}");
        assert_eq!(hello["type"], "auth_state");
        assert_eq!(hello["logged_in"], false);

        send(&mut socket, json!({"command": "get_auth_state"})).await.unwrap();
        let state = tokio::time::timeout(Duration::from_secs(5), receive(&mut socket)).await.unwrap().unwrap();
        println!("get_auth_state: {state}");
        assert_eq!(state["type"], "auth_state");

        // nobody is signed in, so a play is refused, in the documented shape
        send(&mut socket, json!({"command": "play", "uri": A})).await.unwrap();
        let refused = tokio::time::timeout(Duration::from_secs(5), receive(&mut socket)).await.unwrap().unwrap();
        println!("play: {refused}");
        assert_eq!(refused["type"], "error");

        child.kill().await.unwrap();
        let _ = child.wait().await;
        let output = std::fs::read_to_string(data.join("soloist-output.log")).unwrap_or_default();
        println!("its output:
{output}");
        assert!(!output.contains("not-a-real-key"));
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn a_level_is_as_loud_through_soloist_as_through_the_built_in_player() {
        assert_eq!(soloist_volume(0), 0);
        assert_eq!(soloist_volume(100), 100);
        // the built-in player's 50% is 6 dB down; Soloist's 50 is 24 dB down
        assert_eq!(soloist_volume(50), 85);
        assert_eq!(soloist_volume(20), 69);
        assert_eq!(soloist_volume(10), 58);
        // quiet stays quiet but audible, and never goes the wrong way
        assert!((1..=20).contains(&soloist_volume(1)));
        let mut last = 0;
        for level in 0..=100 {
            let volume = soloist_volume(level);
            assert!(volume >= last, "{level} -> {volume} after {last}");
            last = volume;
        }
    }

    #[test]
    fn episodes_keep_their_whole_uri() {
        assert_eq!(id_of("spotify:track:abc"), "abc");
        assert_eq!(id_of("spotify:episode:abc"), "spotify:episode:abc");
    }
}
