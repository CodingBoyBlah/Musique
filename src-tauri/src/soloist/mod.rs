//! Spotify Soloist as an audio backend.
//!
//! Soloist is Spotify's own headless player (developer.spotify.com/
//! documentation/soloist): a separate program that signs in as a Spotify
//! Connect device, opens the audio output itself and is driven over a
//! WebSocket bound to this computer. It needs Premium and a personal key from
//! the listener's Spotify developer dashboard, and it has to be picked once
//! from the device list of a Spotify app on the same network to pair.
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

pub mod install;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
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
/// Soloist exits with this code when its build has expired.
const EXPIRED_EXIT: i32 = 10;
/// The most of Soloist's own output kept for a bug report.
const OUTPUT_LIMIT: usize = 1 << 20;

/// Where Soloist is, for the interface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    /// Not turned on.
    Off,
    /// Spotify publishes no build for this kind of computer.
    Unsupported,
    /// Turned on, but there is no personal key to start it with.
    NeedsKey,
    Downloading { received: u64, total: u64 },
    Starting,
    /// Running and waiting to be picked in a Spotify app's device list.
    Pairing { name: String, version: String },
    /// Signed in and taking commands.
    Ready { name: String, version: String, expires: u64 },
    Failed { message: String },
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
}

/// What to do about something Soloist said.
#[derive(Debug, PartialEq)]
enum Out {
    Event(PlayerMsg),
    Command(Value),
}

fn unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

/// The id the interface knows a URI by: the bare id of a track, the whole URI
/// of anything else (episodes are kept by their URI throughout the app).
fn id_of(uri: &str) -> String {
    uri.strip_prefix("spotify:track:").unwrap_or(uri).to_string()
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

    /// The app asked for `uri` from `position_ms`.
    fn request(&mut self, uri: &str) {
        self.requested = Some(uri.to_string());
        self.alias = None;
        self.peak_ms = 0;
        self.started = false;
        self.ended = false;
        self.user_paused = false;
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
        if self.started && !self.ended && self.on_requested() {
            self.peak_ms = self.peak_ms.max(self.position_now(now_ms));
        }
        match kind {
            "playback_state" => {
                self.set_item(&message["item"]);
                self.status = message["status"].as_str().unwrap_or_default().to_string();
                self.set_position(&message["position"]);
            }
            "track_changed" => {
                self.set_item(&message["item"]);
                // the first change after asking for a track is that track,
                // whatever Soloist calls it
                if self.requested.is_some() && !self.started && !self.ended && !self.on_requested() {
                    self.alias = self.item.clone();
                }
            }
            "playback_changed" => {
                self.status = message["status"].as_str().unwrap_or_default().to_string();
            }
            "position_sync" => self.set_position(&message["position"]),
            _ => return Vec::new(),
        }

        let Some(requested) = self.requested.clone() else {
            return Vec::new();
        };
        if self.ended {
            return Vec::new();
        }
        let track_id = Some(id_of(&requested));
        if !self.on_requested() {
            if self.started && self.item.is_some() {
                // Soloist went on to something of its own choosing (autoplay,
                // the rest of an album). The app's queue decides what is next.
                self.ended = true;
                self.started = false;
                return vec![
                    Out::Command(json!({"command": "pause"})),
                    Out::Event(PlayerMsg::EndOfTrack { track_id }),
                ];
            }
            return Vec::new();
        }

        let position_ms = self.position_now(now_ms).min(u64::from(u32::MAX)) as u32;
        match self.status.as_str() {
            "playing" => {
                let moved_only = kind == "position_sync" && self.started;
                self.started = true;
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
                } else if kind == "position_sync" {
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
    status: Mutex<Status>,
    play:   Mutex<Play>,
}

impl Shared {
    fn play(&self) -> MutexGuard<'_, Play> {
        self.play.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn status(&self) -> Status {
        self.status.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    fn set_status(&self, app: &AppHandle, status: Status) {
        let mut current = self.status.lock().unwrap_or_else(PoisonError::into_inner);
        if *current == status {
            return;
        }
        *current = status.clone();
        drop(current);
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

/// The one Soloist this app runs, or would run.
pub struct Soloist {
    shared:  Arc<Shared>,
    running: Mutex<Option<Running>>,
}

impl Soloist {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            shared: Arc::new(Shared {
                status: Mutex::new(Status::Off),
                play:   Mutex::new(Play::default()),
            }),
            running: Mutex::new(None),
        })
    }

    fn running(&self) -> MutexGuard<'_, Option<Running>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn status(&self) -> Status {
        self.shared.status()
    }

    /// Start Soloist, with `key` when the listener has just supplied one and
    /// the saved one otherwise. Replaces a copy already running.
    pub async fn start(&self, app: AppHandle, key: Option<String>) {
        self.stop().await;
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

    /// Stop the player and wait for it to be gone.
    pub async fn stop(&self) {
        let running = self.running().take();
        if let Some(running) = running {
            let _ = running.stop.send(true);
            let _ = tokio::time::timeout(Duration::from_secs(5), running.task).await;
        }
        *self.shared.play() = Play::default();
        *self.shared.status.lock().unwrap_or_else(PoisonError::into_inner) = Status::Off;
    }

    fn send(&self, command: Value) -> Result<(), AppError> {
        match self.status() {
            Status::Ready { .. } => {}
            Status::Pairing { name, .. } => {
                return Err(AppError::Playback(format!(
                    "Soloist is not connected yet. Play a song in the Spotify app and pick \"{name}\" from its device list."
                )));
            }
            Status::NeedsKey => {
                return Err(AppError::Playback("Soloist needs your personal key before it can play.".into()));
            }
            Status::Failed { message } => return Err(AppError::Playback(message)),
            _ => return Err(AppError::Playback("Soloist is still starting. Try again in a moment.".into())),
        }
        self.running()
            .as_ref()
            .and_then(|running| running.commands.send(command).ok())
            .ok_or_else(|| AppError::Playback("Soloist is not running".into()))
    }

    /// Play `uri` from the start, or from `position_ms`.
    pub fn play(&self, uri: &str, position_ms: u32) -> Result<(), AppError> {
        // checked before anything is recorded, so a refused play leaves the
        // state describing what is really playing
        self.send(json!({"command": "activate"}))?;
        self.shared.play().request(uri);
        self.send(json!({"command": "play", "uri": uri}))?;
        if position_ms > 0 {
            self.send(json!({"command": "seek", "position_ms": position_ms}))?;
        }
        Ok(())
    }

    pub fn resume(&self) -> Result<(), AppError> {
        self.shared.play().user_paused = false;
        self.send(json!({"command": "play"}))
    }

    pub fn pause(&self) -> Result<(), AppError> {
        self.shared.play().user_paused = true;
        self.send(json!({"command": "pause"}))
    }

    pub fn seek(&self, position_ms: u32) -> Result<(), AppError> {
        self.send(json!({"command": "seek", "position_ms": position_ms}))
    }

    /// `percent` is 0 to 100. Quietly does nothing while Soloist is not up:
    /// the level is sent again when it signs in.
    pub fn set_volume(&self, percent: u8) {
        let _ = self.send(json!({"command": "set_volume", "volume": percent.min(100)}));
    }

    /// The URI the app last asked for.
    pub fn current_uri(&self) -> Option<String> {
        self.shared.play().requested.clone()
    }

    /// Whether the track the app asked for is loaded and has not ended, so
    /// pressing play can carry it on rather than start it again.
    pub fn resumable(&self, uri: &str) -> bool {
        let play = self.shared.play();
        play.requested.as_deref() == Some(uri) && play.started && !play.ended && play.on_requested()
    }

    pub fn position_ms(&self) -> u32 {
        self.shared.play().position_now(unix_ms()).min(u64::from(u32::MAX)) as u32
    }
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
        *shared.play() = Play::default();
        match result {
            Ok(End::Stopped) => return,
            Ok(End::NeedsKey) => {
                shared.set_status(&app, Status::NeedsKey);
                return;
            }
            Ok(End::Restart) => continue,
            Err(message) => {
                log::warn!("soloist stopped: {message}");
                // a build that was only just moved to gets no second chance
                let _ = tokio::fs::remove_file(root(&app).join("builds").join("pending.json")).await;
                if began.elapsed() >= HEALTHY_AFTER {
                    failures = 0;
                }
                let Some(wait) = RESTART_AFTER.get(failures).copied() else {
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
        Ok(id) if id.len() == 8 && id.bytes().all(|b| b.is_ascii_hexdigit()) => id,
        _ => {
            let id = format!("{:08x}", rand::random::<u32>());
            let _ = tokio::fs::write(&path, &id).await;
            id
        }
    };
    format!("Musique Soloist ({id})")
}

/// One run of the player, from launch until it stops or fails.
async fn session(
    app: &AppHandle,
    shared: &Shared,
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
    shared: &Shared,
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
    let volume = saved_volume(app).await.to_string();
    let mut command = tokio::process::Command::new(build.path(builds));
    command
        .args(["--device-name", name, "--api-key", key, "--ws", "127.0.0.1:0", "--cache-size", "512"])
        .args(["--initial-volume", &volume])
        .arg("--data-dir")
        .arg(data)
        .arg("--cache-dir")
        .arg(data.join("cache"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // no console window
    // Soloist only takes the key as an argument. This command line and what
    // the child prints are never written to the app's own log.
    let mut child = command.spawn().map_err(|e| format!("Could not start Soloist: {e}"))?;
    keep_output(data.join("soloist-output.log"), key.to_string(), &mut child);
    raise_priority(&child);

    let outcome = tokio::select! {
        _ = stopped.changed() => Ok(Ran::Ended(End::Stopped)),
        outcome = drive(app, shared, name, builds, data, build, &mut child, inbox) => outcome,
    };
    let _ = child.kill().await;
    let _ = child.wait().await;
    outcome
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
    let mut reconnected: Option<Instant> = None;
    let mut checked = build.checked;
    let mut download: Option<tokio::task::JoinHandle<Result<Build, String>>> = None;
    let mut pending: Option<Build> = None;
    let mut tick = tokio::time::interval(TICK);
    loop {
        tokio::select! {
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
                            shared.set_status(app, Status::Ready {
                                name: name.into(),
                                version: build.version.clone(),
                                expires: build.expires(),
                            });
                            // Soloist starts at a level of its own every launch
                            send(&mut socket, json!({"command": "set_volume", "volume": saved_volume(app).await})).await?;
                            send(&mut socket, json!({"command": "get_state"})).await?;
                        } else {
                            shared.set_status(app, Status::Pairing { name: name.into(), version: build.version.clone() });
                        }
                    }
                    continue;
                }
                if message["type"] == "error" {
                    log::warn!("soloist refused a command: {}", message["message"].as_str().unwrap_or_default());
                    continue;
                }
                let outs = shared.play().observe(&message, unix_ms());
                for out in outs {
                    match out {
                        Out::Command(command) => send(&mut socket, command).await?,
                        Out::Event(event) => report(app, event),
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
fn keep_output(path: PathBuf, key: String, child: &mut tokio::process::Child) {
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
            while all.recv().await.is_some() {}
            return;
        };
        let mut written = 0usize;
        while let Some(line) = all.recv().await {
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

        play.request(A);
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
    fn soloist_moving_on_by_itself_is_the_end_of_the_track() {
        let mut play = Play::default();
        play.request(A);
        play.observe(&json!({"type": "playback_state", "status": "playing", "item": item(A),
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
        play.request(B);
        assert_eq!(
            play.observe(&json!({"type": "playback_changed", "status": "playing"}), 200_200).len(),
            1
        );
    }

    #[test]
    fn a_pause_at_the_end_nobody_asked_for_is_the_end() {
        let mut play = Play::default();
        play.request(A);
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
        play.request(A);
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
        keep_output(data.join("soloist-output.log"), "not-a-real-key".into(), &mut child);

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
    fn episodes_keep_their_whole_uri() {
        assert_eq!(id_of("spotify:track:abc"), "abc");
        assert_eq!(id_of("spotify:episode:abc"), "spotify:episode:abc");
    }
}
