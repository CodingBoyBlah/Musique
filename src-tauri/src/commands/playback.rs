use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    state::AppState,
};

// helper stuff

async fn read_vol(pool: &sqlx::SqlitePool) -> f64 {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'player_volume'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.and_then(|(v,)| v.parse::<f64>().ok())
        .map(|v| v / 100.0)
        .unwrap_or(0.8)
}

async fn read_muted(pool: &sqlx::SqlitePool) -> bool {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'player_muted'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|(v,)| v == "1").unwrap_or(false)
}

pub(crate) async fn save_setting(pool: &sqlx::SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_inner(app: &AppHandle) -> Result<(), AppError> {
    ensure_inner_with(app, true).await
}

/// podcast playback speed (0.5x - 3x, pitch kept). applies to whatever the
/// sink is playing; the frontend sets it back to 1x for music
#[tauri::command]
pub async fn set_playback_speed(app: AppHandle, speed: f32) -> Result<(), AppError> {
    if !(0.5..=3.0).contains(&speed) {
        return Err(AppError::InvalidInput(format!("speed out of range: {speed}")));
    }
    let before = crate::stretch::speed();
    crate::stretch::set_speed(speed);
    if (before - speed).abs() < 1e-3 {
        return Ok(());
    }
    // seconds of audio are queued ahead, stretched at the old speed. the
    // youtube backend's prebuffer starts over on its own; spotify's player
    // has to be sent back to what is being heard
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.speed_changed()?;
    }
    Ok(())
}

/// bring the librespot session up for the internal api layer (spclient calls)
/// without ever popping a browser authorization at the user
pub(crate) async fn warm_session(app: &AppHandle) -> Result<(), AppError> {
    ensure_inner_with(app, false).await
}

/// `interactive = false` never opens a browser authorization; see `create_inner`.
async fn ensure_inner_with(app: &AppHandle, interactive: bool) -> Result<(), AppError> {
    let epoch = crate::auth::auth_epoch();
    let s        = app.state::<AppState>();
    let db       = s.db.clone();
    let auth     = s.auth.clone();
    let playback = s.playback.clone();
    let media_tx = s.media_tx.clone();
    drop(s);

    let mut guard = playback.lock().await;
    // rebuild when theres no session yet OR the existing one went invalid
    // (ap disconnect / expiry). a stale session just eats load/play calls so
    // the ui says "playing" but no audio comes out. healing here fixes that
    let rebuild = match guard.as_ref() {
        None        => true,
        Some(inner) => inner.session_invalid() || inner.needs_rebuild.load(Ordering::Relaxed),
    };
    if rebuild {
        let vol   = read_vol(&db).await;
        // a probe run while something else is playing must not be heard
        let muted = read_muted(&db).await || std::env::var_os("MUSIQUE_PROBE_SILENT").is_some();
        // the session dropped under a player that is fine: the new session
        // takes the player over, still playing, instead of building another
        // and loading the song again on it. (a rebuild asked for, after a
        // setting the player is built with changed, gets a new player)
        let carry = guard
            .as_ref()
            .filter(|old| old.session_invalid() && !old.needs_rebuild.load(Ordering::Relaxed))
            .map(Carry::from);
        let reuse = carry.as_ref().map(|carry| carry.parts.clone());
        let inner = crate::playback::create_inner(app.clone(), db.clone(), auth, vol, muted, media_tx, interactive, reuse).await?;
        if !crate::auth::epoch_is_current(epoch) {
            if inner.kept_player {
                inner.shutdown_session().await;
            } else {
                inner.shutdown().await;
            }
            return Err(AppError::Auth("Account changed; playback cancelled".into()));
        }
        if let Some(carry) = carry.filter(|_| inner.kept_player) {
            if let Some(old) = guard.take() {
                tauri::async_runtime::spawn(async move { old.shutdown_session().await });
            }
            carry.hand_over(&inner);
            *guard = Some(inner);
            return Ok(());
        }
        // the player of a session that died plays on through what it had
        // downloaded, which carries the listener over the reconnect. left
        // running it would be heard under the new one, so it stops the
        // moment the new one is ready, and where it got to is kept for the
        // new one to carry on from (`recover_dropped_sessions`)
        if let Some(old) = guard.take() {
            if let Some(uri) = old.current_uri() {
                *REPLACED.lock().unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some((uri, old.position_ms(), std::time::Instant::now()));
            }
            old.stop_playing();
            tauri::async_runtime::spawn(async move { old.shutdown().await });
        }
        warm_last_played(&db, &inner).await;
        *guard = Some(inner);
    }
    Ok(())
}

/// What a player whose session dropped was doing, for the session that takes
/// it over.
struct Carry {
    parts:       crate::playback::PlayerParts,
    /// The track it is on, unless nothing is loaded or it played to the end.
    uri:         Option<String>,
    position_ms: u32,
    playing:     bool,
    jam:         bool,
    snapshot:    Option<std::sync::Arc<librespot_connect::PlaybackSnapshot>>,
}

impl Carry {
    fn from(old: &crate::playback::PlaybackInner) -> Self {
        Carry {
            parts:       old.player_parts(),
            uri:         old.current_uri().filter(|_| old.loaded.load(Ordering::Relaxed) && !old.is_ended()),
            position_ms: old.position_ms(),
            playing:     old.is_playing(),
            jam:         old.connect_state().jam_mode,
            snapshot:    old.disconnected_playback(),
        }
    }

    /// Spirc on the new session takes over what the player is on, where it
    /// is. The music never stopped; spirc and Spotify Connect learn of it.
    fn hand_over(self, inner: &crate::playback::PlaybackInner) {
        let Some(uri) = self.uri else {
            eprintln!("[playback] the player carries on, with nothing loaded, on a new session");
            return;
        };
        if self.jam {
            // a jam's queue is spirc's: what it kept of it puts it back, and
            // the track it is on is taken over where it is
            let restored = self.snapshot.map(|snapshot| {
                inner.mark_carry_on(&uri);
                inner.restore_playback(snapshot, uri)
            });
            match restored {
                Some(Ok(())) => eprintln!("[playback] the jam carried on, on a new session"),
                Some(Err(e)) => eprintln!("[playback] could not put the jam back after reconnecting: {e}"),
                None => eprintln!("[playback] nothing was kept of the jam to put back"),
            }
            return;
        }
        match inner.carry_on(uri, self.position_ms, self.playing) {
            Ok(()) => eprintln!("[playback] the player carried on at {}ms on a new session", self.position_ms),
            Err(e) => eprintln!("[playback] could not hand the track to the new session: {e}"),
        }
    }
}

/// On a new session, fetches ahead the start of the song last played: its
/// metadata, first bytes and audio key. Pressing play on it then starts at
/// once, and whatever is played first finds the CDN connection and the key
/// channel already open, which on a cold session cost a few hundred ms more.
async fn warm_last_played(pool: &sqlx::SqlitePool, inner: &crate::playback::PlaybackInner) {
    let last: Option<(String,)> =
        sqlx::query_as("SELECT track_id FROM recently_played ORDER BY played_at DESC LIMIT 1")
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    if let Some(id) = last.and_then(|(id,)| crate::playback::parse_track_id(&id).ok()) {
        inner.player.prefetch_also(vec![id]);
    }
}

// picking playback back up after the session drops
//
// Spotify's access point closes sessions now and then, and a network change
// (wifi to ethernet, a VPN coming up, the laptop waking) takes one with it. The
// player keeps playing what it had already downloaded and then goes quiet, and
// until now nothing happened until the listener pressed something: the next
// command noticed the dead session and rebuilt it. This notices by itself,
// builds a new session, and carries on from where the listener had got to.

/// How often the session is looked at.
const SESSION_CHECK: std::time::Duration = std::time::Duration::from_secs(3);
/// How long to leave it after a rebuild that failed (no network yet).
const SESSION_RETRY: std::time::Duration = std::time::Duration::from_secs(10);

/// What was playing when the session dropped.
struct Interrupted {
    uri:         String,
    /// Where the listener was in it.
    position_ms: u32,
    /// Playing rather than paused.
    playing:     bool,
    /// The queue was spirc's (a jam), so it is spirc's to bring back.
    jam:         bool,
    /// What spirc kept of its own queue and context as the session ended.
    snapshot:    Option<std::sync::Arc<librespot_connect::PlaybackSnapshot>>,
}

async fn interrupted(app: &AppHandle) -> Option<Interrupted> {
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    let inner = guard.as_ref()?;
    if !inner.session_invalid() || !inner.loaded.load(Ordering::Relaxed) || inner.is_ended() {
        return None;
    }
    Some(Interrupted {
        uri:         inner.current_uri()?,
        position_ms: inner.position_ms(),
        playing:     inner.is_playing(),
        jam:         inner.connect_state().jam_mode,
        snapshot:    inner.disconnected_playback(),
    })
}

/// The track a replaced player was on, and where it had got to when it was
/// stopped. Only good for a moment.
static REPLACED: std::sync::Mutex<Option<(String, u32, std::time::Instant)>> = std::sync::Mutex::new(None);

/// Where the replaced player had got to in `uri`, if it was on it just now.
fn replaced_at(uri: &str) -> Option<u32> {
    let replaced = REPLACED.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take()?;
    (replaced.0 == uri && replaced.2.elapsed() < std::time::Duration::from_secs(30)).then_some(replaced.1)
}

/// Runs for the life of the app (see `lib.rs`).
pub async fn recover_dropped_sessions(app: AppHandle) {
    loop {
        tokio::time::sleep(SESSION_CHECK).await;
        // the player's own state first: it is in memory, and nearly every
        // look finds nothing dropped. the backend is read from the database,
        // and only once there is something to bring back
        let Some(Interrupted { uri, position_ms, playing, jam, snapshot }) = interrupted(&app).await else {
            continue;
        };
        let pool = app.state::<AppState>().db.clone();
        if backend(&pool).await != Backend::Spotify {
            continue;
        }
        eprintln!("[playback] the session dropped at {position_ms}ms of {uri}; reconnecting");
        // non-interactive: never a browser tab the listener did not ask for
        if let Err(e) = ensure_inner_with(&app, false).await {
            eprintln!("[playback] could not reconnect yet: {e}");
            tokio::time::sleep(SESSION_RETRY).await;
            continue;
        }
        let playback = app.state::<AppState>().playback.clone();
        let guard = playback.lock().await;
        let Some(inner) = guard.as_ref() else {
            continue;
        };
        // the new session took the player over, still playing (`Carry`)
        if inner.kept_player {
            continue;
        }
        // the listener got there first and picked something on the new session
        if inner.loaded.load(Ordering::Relaxed) {
            continue;
        }
        if jam {
            // a jam's queue is spirc's and everyone's in it: only what spirc
            // kept of it can put it back, in order. without that there is
            // nothing of the app's to reload; the listener rejoins the jam
            match snapshot.map(|snapshot| inner.restore_playback(snapshot, uri)) {
                Some(Ok(())) => eprintln!("[playback] the jam carried on, on a new session"),
                Some(Err(e)) => eprintln!("[playback] could not put the jam back after reconnecting: {e}"),
                None => eprintln!("[playback] nothing was kept of the jam to put back"),
            }
            continue;
        }
        if !playing {
            // paused when it dropped: pressing play loads it again from there
            continue;
        }
        // the app's own queue: the track, from where the listener heard it
        // to (spirc's own record is where the decoder was, seconds further on).
        // the old player played on while the new session came up
        let position_ms = replaced_at(&uri).filter(|at| *at >= position_ms).unwrap_or(position_ms);
        match inner.play_uri(uri, position_ms) {
            Ok(()) => eprintln!("[playback] carried on from {position_ms}ms on a new session"),
            Err(e) => eprintln!("[playback] could not carry on after reconnecting: {e}"),
        }
    }
}

// ── engine probe ────────────────────────────────────────────────────────────
//
// `MUSIQUE_ENGINE_PROBE=1 spotify.exe` runs the real engine - the session,
// spirc, the player, the output - with the window left hidden and no interface
// attached, and plays the part of the interface itself: it names the next
// track, waits to be told the first one ended, and asks for the next the way
// `App.tsx` does. Then it drops the session and waits for the engine to pick
// the song back up. It checks what the device was given throughout.
//
// It only runs when nothing is playing on the account, since loading a track
// here would take playback away from whatever device has it.

/// Whether this run is the engine probe. Playback commands from the interface
/// are refused then: the probe plays the interface's part itself, and one that
/// managed to load must not be able to join in.
fn probing() -> bool {
    static PROBING: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *PROBING.get_or_init(|| std::env::var_os("MUSIQUE_ENGINE_PROBE").is_some())
}

fn refuse_while_probing() -> Result<(), AppError> {
    if probing() {
        return Err(AppError::Playback("the engine probe is running".into()));
    }
    Ok(())
}

async fn probe_inner<T>(
    app: &AppHandle,
    f: impl FnOnce(&crate::playback::PlaybackInner) -> T,
) -> Option<T> {
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    guard.as_ref().map(f)
}

async fn probe_wait(
    events: &mut tokio::sync::mpsc::UnboundedReceiver<serde_json::Value>,
    kind: &str,
    track: &str,
    within: std::time::Duration,
) -> Option<serde_json::Value> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        match tokio::time::timeout_at(deadline, events.recv()).await {
            Ok(Some(event)) if event["type"] == kind && event["track_id"] == track => return Some(event),
            Ok(Some(_)) => {}
            _ => return None,
        }
    }
}

/// `MUSIQUE_ENGINE_PROBE=1 MUSIQUE_START_PROBE=1 spotify.exe` times how long
/// a clicked song takes to be heard: six saved songs picked at random, each
/// started the way the interface starts one, from the click to the player
/// saying it plays and to the first 30 ms coming out of the device.
pub async fn start_probe(app: AppHandle) -> i32 {
    use std::time::{Duration, Instant};
    use tauri::Listener;
    let pool = app.state::<AppState>().db.clone();
    if backend(&pool).await != Backend::Spotify {
        eprintln!("[start-probe] the built-in Spotify player is not the selected backend");
        return 2;
    }
    // the song last played first: the new session fetched its start ahead
    let mut tracks: Vec<(String,)> =
        sqlx::query_as("SELECT track_id FROM recently_played ORDER BY played_at DESC LIMIT 1")
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
    tracks.extend(
        sqlx::query_as::<_, (String,)>("SELECT track_id FROM saved_tracks ORDER BY RANDOM() LIMIT 6")
            .fetch_all(&pool)
            .await
            .unwrap_or_default(),
    );
    if tracks.len() < 3 {
        eprintln!("[start-probe] needs saved tracks");
        return 2;
    }
    let (tx, mut events) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
    app.listen("player:event", move |event| {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            let _ = tx.send(value);
        }
    });
    let warm = Instant::now();
    if let Err(e) = ensure_inner_with(&app, false).await {
        eprintln!("[start-probe] no session: {e}");
        return 3;
    }
    eprintln!("[start-probe] session up in {}ms", warm.elapsed().as_millis());
    // as a listener would: the app is up a moment before the first click
    tokio::time::sleep(Duration::from_secs(4)).await;

    let (mut said, mut heard) = (Vec::new(), Vec::new());
    for (id,) in &tracks {
        let Ok(uri) = crate::playback::track_uri(id) else { continue };
        while events.try_recv().is_ok() {}
        // what the click costs before the player is asked: the backend checks
        let click = Instant::now();
        let _ = soloist(&app).await;
        let _ = uses_youtube(&app).await;
        let _ = ensure_inner_with(&app, false).await;
        let checks = click.elapsed();
        log::info!(target: "librespot_probe", "click {id}");
        let asked = probe_inner(&app, |inner| inner.play_requested(uri.clone(), None)).await;
        if !matches!(asked, Some(Ok(()))) {
            eprintln!("[start-probe] could not start {id}: {asked:?}");
            continue;
        }
        let playing = probe_wait(&mut events, "playing", id, Duration::from_secs(20)).await.map(|_| click.elapsed());
        let mut sound = None;
        while click.elapsed() < Duration::from_secs(20) {
            let heard_ms = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
            let current = probe_inner(&app, |inner| inner.current_uri()).await.flatten();
            if heard_ms >= 30 && current.as_deref() == Some(uri.as_str()) {
                sound = Some(click.elapsed().saturating_sub(Duration::from_millis(30)));
                log::info!(target: "librespot_probe", "heard {id}");
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        if sound.is_none() {
            let state = probe_inner(&app, |inner| inner.probe_state()).await;
            eprintln!("[start-probe] {id} never sounded: {state:?}");
        }
        eprintln!(
            "[start-probe] {id}: checks {}ms, playing event {:?}ms, first sound {:?}ms",
            checks.as_millis(),
            playing.map(|d| d.as_millis()),
            sound.map(|d| d.as_millis()),
        );
        if let (Some(p), Some(s)) = (playing, sound) {
            said.push(p.as_millis());
            heard.push(s.as_millis());
        }
        // a jump well past what has been fetched (the player logs how long it took)
        log::info!(target: "librespot_probe", "seek {id}");
        let _ = probe_inner(&app, |inner| inner.seek(90_000)).await;
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let _ = probe_inner(&app, |inner| inner.pause()).await;
        tokio::time::sleep(Duration::from_millis(800)).await;
    }
    let playback = app.state::<AppState>().playback.clone();
    if let Some(inner) = playback.lock().await.take() {
        inner.shutdown().await;
    }
    if heard.is_empty() {
        return 5;
    }
    let median = |v: &mut Vec<u128>| { v.sort_unstable(); v[v.len() / 2] };
    eprintln!("[start-probe] median: playing event {}ms, first sound {}ms", median(&mut said), median(&mut heard));
    0
}

/// What is playing on the account elsewhere, by device name, if anything.
async fn playing_elsewhere(pool: &sqlx::SqlitePool, auth: &std::sync::Arc<tokio::sync::RwLock<crate::state::AuthState>>) -> Result<Option<String>, String> {
    let token = crate::auth::get_valid_token(pool, auth).await.map_err(|e| format!("not signed in: {e}"))?;
    let response = crate::http::client()
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("could not read the player state: {e}"))?;
    if response.status().as_u16() == 204 {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("could not read the player state ({})", response.status()));
    }
    let body: serde_json::Value = response.json().await.unwrap_or_default();
    Ok((body["is_playing"] == true).then(|| body["device"]["name"].as_str().unwrap_or("another device").to_string()))
}

/// `MUSIQUE_ENGINE_PROBE=1 MUSIQUE_SOLOIST_PROBE=1 spotify.exe` runs Soloist
/// the way the app does, with the key in `MUSIQUE_SOLOIST_KEY` (saved like
/// one entered in the app) or the one already saved, waits for it to sign in
/// (the app pairs it by itself), and, when nothing is playing on the
/// account, plays a saved song on it and checks that it plays and pauses.
pub async fn soloist_probe(app: AppHandle) -> i32 {
    use std::time::{Duration, Instant};
    use tauri::Listener;
    let state = app.state::<AppState>();
    let (pool, auth, soloist) = (state.db.clone(), state.auth.clone(), state.soloist.clone());
    drop(state);
    // the playback session's account is what Soloist is signed in with
    if let Err(e) = ensure_inner_with(&app, false).await {
        eprintln!("[soloist-probe] no playback session: {e}");
        return 3;
    }
    let key = std::env::var("MUSIQUE_SOLOIST_KEY").ok();
    let started = Instant::now();
    soloist.start(app.clone(), key).await;
    let mut last = String::new();
    let ready = loop {
        let status = soloist.status();
        let shown = format!("{status:?}");
        if shown != last {
            eprintln!("[soloist-probe] {:>6}ms {shown}", started.elapsed().as_millis());
            last = shown;
        }
        match status {
            crate::soloist::Status::Ready { .. } => break true,
            crate::soloist::Status::Failed { .. }
            | crate::soloist::Status::NeedsKey
            | crate::soloist::Status::Unsupported => break false,
            _ if started.elapsed() > Duration::from_secs(240) => break false,
            _ => tokio::time::sleep(Duration::from_millis(200)).await,
        }
    };
    if !ready {
        eprintln!("[soloist-probe] FAILED - Soloist did not sign in");
        soloist.stop().await;
        return 6;
    }
    eprintln!("[soloist-probe] signed in {}ms after starting, without being picked in a Spotify app", started.elapsed().as_millis());

    match playing_elsewhere(&pool, &auth).await {
        Ok(None) => {}
        Ok(Some(device)) => {
            eprintln!("[soloist-probe] \"{device}\" is playing on this account; not taking playback away from it to play on Soloist");
            soloist.stop().await;
            return 4;
        }
        Err(e) => {
            eprintln!("[soloist-probe] {e}");
            soloist.stop().await;
            return 2;
        }
    }
    let track: Option<(String,)> = sqlx::query_as("SELECT track_id FROM saved_tracks ORDER BY RANDOM() LIMIT 1")
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();
    let Some((id,)) = track else {
        eprintln!("[soloist-probe] needs a saved track");
        soloist.stop().await;
        return 2;
    };
    let (tx, mut events) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
    app.listen("player:event", move |event| {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            let _ = tx.send(value);
        }
    });
    let uri = match crate::playback::track_uri(&id) {
        Ok(uri) => uri,
        Err(e) => { eprintln!("[soloist-probe] {e}"); soloist.stop().await; return 2; }
    };
    let asked = Instant::now();
    if let Err(e) = soloist.play(&uri, 0) {
        eprintln!("[soloist-probe] FAILED - Soloist would not play: {e}");
        soloist.stop().await;
        return 5;
    }
    let playing = probe_wait(&mut events, "playing", &id, Duration::from_secs(20)).await;
    let took = asked.elapsed().as_millis();
    eprintln!("[soloist-probe] play {id}: playing event after {took}ms: {}", playing.is_some());
    tokio::time::sleep(Duration::from_secs(4)).await;
    let paused = soloist.pause().is_ok() && probe_wait(&mut events, "paused", &id, Duration::from_secs(10)).await.is_some();
    eprintln!("[soloist-probe] paused: {paused}");
    soloist.stop().await;
    if playing.is_none() || !paused {
        eprintln!("[soloist-probe] FAILED - Soloist did not play and pause the song");
        return 5;
    }
    eprintln!("[soloist-probe] Soloist played and paused a song from the app's own queue");
    0
}

pub async fn engine_probe(app: AppHandle) -> i32 {
    use tauri::Listener;
    use std::time::{Duration, Instant};

    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let auth = state.auth.clone();
    drop(state);

    if backend(&pool).await != Backend::Spotify {
        eprintln!("[engine-probe] the built-in Spotify player is not the selected backend");
        return 2;
    }
    let token = match crate::auth::get_valid_token(&pool, &auth).await {
        Ok(token) => token,
        Err(e) => { eprintln!("[engine-probe] not signed in: {e}"); return 2; }
    };
    let now_playing = crate::http::client()
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&token)
        .send()
        .await;
    match now_playing {
        Ok(response) if response.status().as_u16() == 204 => {}
        Ok(response) if response.status().is_success() => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            if body["is_playing"] == true {
                eprintln!("[engine-probe] something is playing on this account; not taking playback away from it");
                return 4;
            }
        }
        Ok(response) => { eprintln!("[engine-probe] could not read the player state ({})", response.status()); return 2; }
        Err(e) => { eprintln!("[engine-probe] could not read the player state: {e}"); return 2; }
    }

    let tracks: Vec<(String,)> = sqlx::query_as("SELECT track_id FROM saved_tracks LIMIT 2")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    let [(a,), (b,)] = tracks.as_slice() else {
        eprintln!("[engine-probe] needs two saved tracks");
        return 2;
    };
    let (a, b) = (a.clone(), b.clone());
    let (uri_a, uri_b) = match (crate::playback::track_uri(&a), crate::playback::track_uri(&b)) {
        (Ok(a), Ok(b)) => (a, b),
        _ => { eprintln!("[engine-probe] bad saved track ids"); return 2; }
    };

    // what the interface would be sent
    let (tx, mut events) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
    app.listen("player:event", move |event| {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            let _ = tx.send(value);
        }
    });

    if let Err(e) = ensure_inner_with(&app, false).await {
        eprintln!("[engine-probe] no session: {e}");
        return 3;
    }
    eprintln!("[engine-probe] session up; playing {a}, then {b}");
    let started = probe_inner(&app, |inner| inner.play_requested(uri_a.clone(), None)).await;
    if !matches!(started, Some(Ok(()))) {
        eprintln!("[engine-probe] FAILED - could not start the first track: {started:?}");
        return 5;
    }
    if probe_wait(&mut events, "playing", &a, Duration::from_secs(25)).await.is_none() {
        eprintln!("[engine-probe] FAILED - the first track never started");
        return 5;
    }

    // the interface names what is next, as it does when a track starts
    let duration_ms = {
        let mut duration = 0;
        for _ in 0..50 {
            duration = probe_inner(&app, |inner| inner.connect_state().duration_ms).await.unwrap_or(0);
            if duration > 0 { break; }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        duration
    };
    if duration_ms < 30_000 {
        eprintln!("[engine-probe] FAILED - no duration for the first track ({duration_ms}ms)");
        return 5;
    }
    let lined_up = probe_inner(&app, |inner| -> Result<(), AppError> {
        inner.player.preload(crate::playback::parse_track_id(&b)?);
        inner.set_next(uri_b.clone());
        inner.seek((duration_ms - 9_000) as u32)
    })
    .await;
    if !matches!(lined_up, Some(Ok(()))) {
        eprintln!("[engine-probe] FAILED - could not line the next track up: {lined_up:?}");
        return 5;
    }
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let (dry_before, _, _) = probe_inner(&app, |inner| inner.audio_health()).await.unwrap_or_default();

    // watch the device from here to a few seconds into the next track
    let watching = app.clone();
    let stop_watch = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop = stop_watch.clone();
    let watch = tokio::spawn(async move {
        let mut worst = 0i64;
        let mut gaps: Vec<(u128, i64, u32)> = Vec::new();
        let began = Instant::now();
        let mut last: Option<(Instant, u64)> = None;
        while !stop.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let Some((_, _, frames)) = probe_inner(&watching, |inner| inner.audio_health()).await else { continue };
            let now = (Instant::now(), frames);
            if let Some(last) = last {
                let clock = now.0.duration_since(last.0).as_millis() as i64;
                let sound = (now.1.saturating_sub(last.1) * 1000 / 44_100) as i64;
                worst = worst.max(clock - sound);
                if clock - sound > 30 {
                    let at = probe_inner(&watching, |inner| inner.position_ms()).await.unwrap_or(0);
                    gaps.push((began.elapsed().as_millis(), clock - sound, at));
                }
            }
            last = Some(now);
        }
        (worst, gaps, began)
    });

    if probe_wait(&mut events, "end_of_track", &a, Duration::from_secs(30)).await.is_none() {
        eprintln!("[engine-probe] FAILED - the interface was never told the first track ended");
        return 6;
    }
    let told_at = Instant::now();
    // by now the engine is on the next track by itself
    let engine_on = probe_inner(&app, |inner| inner.current_uri()).await.flatten();
    let (_, flushes_before, _) = probe_inner(&app, |inner| inner.audio_health()).await.unwrap_or_default();
    // ...and the interface asks for it, late, exactly as App.tsx does
    tokio::time::sleep(Duration::from_millis(300)).await;
    let asked = probe_inner(&app, |inner| inner.play_requested(uri_b.clone(), None)).await;
    let playing_b = probe_wait(&mut events, "playing", &b, Duration::from_secs(15)).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    stop_watch.store(true, Ordering::Relaxed);
    let (worst_gap_ms, gaps, began) = watch.await.unwrap_or((i64::MAX, Vec::new(), Instant::now()));
    eprintln!(
        "[engine-probe] quiet stretches (ms into the watch, ms quiet, position): {gaps:?}; the interface was told at {}ms",
        told_at.duration_since(began).as_millis(),
    );
    let (dry_after, flushes_after, _) = probe_inner(&app, |inner| inner.audio_health()).await.unwrap_or_default();
    let position = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
    let since_told = told_at.elapsed().as_millis() as u32;
    eprintln!(
        "[engine-probe] hand-off: engine was on {engine_on:?} when the interface was told; its late play_track -> {asked:?}; next track at {position}ms {since_told}ms later; longest stretch without sound {worst_gap_ms}ms; {} underruns; {} flushes",
        dry_after - dry_before,
        flushes_after - flushes_before,
    );
    let on_next = engine_on.as_deref() == Some(uri_b.as_str());
    // had the late request restarted the track, it would be near its start
    // again and the queue would have been flushed
    let not_restarted = position + 1500 >= since_told && flushes_after == flushes_before;
    // spotify can hand playback to another device at any moment (the listener
    // pressing play somewhere else). the engine stopping then is it doing its
    // job, and says nothing about the hand-off
    if probe_inner(&app, |inner| !inner.connect_state().active).await.unwrap_or(false) {
        eprintln!("[engine-probe] INCONCLUSIVE - another device took playback during the probe");
        return 4;
    }
    if !(on_next && matches!(asked, Some(Ok(()))) && playing_b.is_some() && not_restarted && dry_after == dry_before && worst_gap_ms < 80) {
        eprintln!("[engine-probe] FAILED - hand-off: on_next={on_next} playing={} not_restarted={not_restarted}", playing_b.is_some());
        return 6;
    }
    eprintln!("[engine-probe] the engine moved on by itself, gaplessly, and the interface's late request left it alone");

    // the session drops mid-song
    let before = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
    let dropped_at = Instant::now();
    // what the device gave out across the reconnect, every 10ms for 10s
    let silence = tauri::async_runtime::spawn(async move {
        let (mut longest, mut quiet_since) = (Duration::ZERO, None::<Instant>);
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(10) {
            if crate::playout::sounding(Duration::from_millis(30)) {
                if let Some(since) = quiet_since.take() {
                    longest = longest.max(since.elapsed());
                }
            } else {
                quiet_since.get_or_insert_with(Instant::now);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        quiet_since.map_or(longest, |since| longest.max(since.elapsed()))
    });
    probe_inner(&app, |inner| inner.session().shutdown()).await;
    eprintln!("[engine-probe] session shut down at {before}ms; waiting for the engine to carry on");
    while events.try_recv().is_ok() {}
    let back = probe_wait(&mut events, "playing", &b, Duration::from_secs(45)).await;
    let took = dropped_at.elapsed().as_millis();
    let healthy = probe_inner(&app, |inner| !inner.session_invalid()).await.unwrap_or(false);
    let first = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
    tokio::time::sleep(Duration::from_secs(2)).await;
    let second = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
    let kept = probe_inner(&app, |inner| inner.kept_player).await.unwrap_or(false);
    let quiet = silence.await.unwrap_or(Duration::MAX);
    eprintln!(
        "[engine-probe] recovery: back after {took}ms on a {} session, at {first}ms (dropped at {before}ms), {second}ms two seconds later; player kept {kept}, longest silence {}ms",
        if healthy { "live" } else { "dead" },
        quiet.as_millis(),
    );
    let _ = probe_inner(&app, |inner| inner.pause()).await;
    // a kept player plays straight through the reconnect
    let recovered = back.is_some() && healthy && first + 500 >= before && first < before + 20_000 && second > first + 1200
        && (!kept || quiet < Duration::from_millis(250));
    if !recovered {
        eprintln!("[engine-probe] FAILED - playback did not carry on after the session dropped");
        return 7;
    }
    eprintln!("[engine-probe] the song carried on from where it was after the session dropped");
    // the old session's player must not be playing on under the new one
    tokio::time::sleep(Duration::from_secs(1)).await;
    let outputs = crate::sink::open_outputs();
    if outputs > 1 {
        eprintln!("[engine-probe] FAILED - {outputs} outputs open after the session was rebuilt; the old player plays on");
        return 7;
    }
    eprintln!("[engine-probe] one output open after the rebuild");

    // paused long enough, the output closes; the next play opens it again
    // and carries on from the same place
    let closes = crate::sink::idle_closes();
    let (underruns, _, _) = probe_inner(&app, |inner| inner.audio_health()).await.unwrap_or((0, 0, 0));
    tokio::time::sleep(Duration::from_millis(600)).await;
    let paused_at = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
    tokio::time::sleep(crate::sink::IDLE_CLOSE + Duration::from_secs(3)).await;
    let closed = crate::sink::idle_closes() > closes;
    while events.try_recv().is_ok() {}
    let _ = probe_inner(&app, |inner| inner.resume()).await;
    let resumed_at = Instant::now();
    let back = probe_wait(&mut events, "playing", &b, Duration::from_secs(10)).await;
    let took = resumed_at.elapsed().as_millis();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let first = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
    tokio::time::sleep(Duration::from_secs(2)).await;
    let second = probe_inner(&app, |inner| inner.position_ms()).await.unwrap_or(0);
    let (underruns_after, _, _) = probe_inner(&app, |inner| inner.audio_health()).await.unwrap_or((0, 0, 0));
    let _ = probe_inner(&app, |inner| inner.pause()).await;
    eprintln!(
        "[engine-probe] idle close: closed={closed}; resumed in {took}ms at {first}ms (paused at {paused_at}ms), {second}ms two seconds later, underruns {underruns} -> {underruns_after}",
    );
    if !closed || back.is_none() || first + 300 < paused_at || first > paused_at + 1500 || second < first + 1200 || underruns_after > underruns {
        eprintln!("[engine-probe] FAILED - the output did not close while paused, or did not carry on after");
        return 8;
    }
    eprintln!("[engine-probe] the output closed while paused and carried on from the same place on resume");

    let playback = app.state::<AppState>().playback.clone();
    if let Some(inner) = playback.lock().await.take() {
        inner.shutdown().await;
    }
    0
}

// backend routing
//
// Free Spotify accounts cannot stream through librespot - Spotify gates audio
// delivery on Premium, so every track comes back Unavailable no matter how
// healthy the session is. Those accounts get audio from YouTube Music instead
// (see `crate::youtube`), while Spotify continues to supply all metadata,
// artwork, lyrics and library data.
//
// The librespot session is still built on a free account: Connect device
// registration and the spclient calls the lyrics pipeline makes both keep
// working. Only the audio path changes.

/// Which backend supplies audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Spotify,
    YouTube,
    /// Spotify's own Soloist player, run beside the app (see `crate::soloist`).
    Soloist,
}

async fn setting_value(pool: &sqlx::SqlitePool, key: &str) -> Option<String> {
    sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|(v,)| v)
        .filter(|v| !v.is_empty())
}

/// Whether the signed-in account can stream through Spotify at all.
///
/// Spotify only delivers audio to Premium, so this is what makes the Spotify
/// backend available rather than a setting. A missing subscription field is
/// unknown, since Spotify can omit it. Only a confirmed Free plan disables
/// the choice; Spotify still enforces streaming entitlement on the server.
pub(crate) async fn spotify_available(pool: &sqlx::SqlitePool) -> bool {
    !matches!(setting_value(pool, "spotify_product").await.as_deref(), Some("free" | "open"))
}

/// Decide which audio backend supplies audio.
///
/// Free accounts have no choice - Spotify cannot stream to them, so they are
/// always on YouTube. Premium accounts pick with `playback_backend` and default
/// to Spotify, since that is the service they are paying for.
///
/// Any unrecognised stored value (including the legacy `"auto"` this setting
/// used to accept) falls through to that default, so old settings rows resolve
/// sensibly instead of needing a migration.
async fn backend(pool: &sqlx::SqlitePool) -> Backend {
    if !spotify_available(pool).await {
        return Backend::YouTube;
    }
    match setting_value(pool, "playback_backend").await.as_deref() {
        Some("youtube") => Backend::YouTube,
        // Soloist is another way of playing from Spotify, so it sits under
        // that choice rather than beside it. Once it is turned on, playback
        // never quietly falls back to the built-in player: a Soloist that is
        // not ready says so instead.
        _ if soloist_enabled(pool).await => Backend::Soloist,
        _ => Backend::Spotify,
    }
}

pub(crate) async fn soloist_enabled(pool: &sqlx::SqlitePool) -> bool {
    setting_value(pool, "soloist_enabled").await.as_deref() == Some("1")
}

/// Soloist, when it is what plays.
async fn soloist(app: &AppHandle) -> Option<std::sync::Arc<crate::soloist::Soloist>> {
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let soloist = state.soloist.clone();
    drop(state);
    (backend(&pool).await == Backend::Soloist).then_some(soloist)
}

/// Pause whatever `which` is playing: the other backends each own an output
/// of their own and would otherwise carry on underneath the new one.
pub(crate) async fn silence(app: &AppHandle, which: Backend) {
    match which {
        Backend::Spotify => {
            let playback = app.state::<AppState>().playback.clone();
            let guard = playback.lock().await;
            if let Some(inner) = guard.as_ref() {
                if let Err(e) = inner.pause() {
                    eprintln!("[playback] pausing spotify on backend switch: {e}");
                }
            }
        }
        Backend::YouTube => {
            let yt = app.state::<AppState>().yt.clone();
            let guard = yt.lock().await;
            if let Some(player) = guard.as_ref() {
                if let Err(e) = player.pause() {
                    eprintln!("[playback] pausing youtube on backend switch: {e}");
                }
            }
        }
        Backend::Soloist => {
            let soloist = app.state::<AppState>().soloist.clone();
            // not being up yet is not being heard either
            let _ = soloist.pause();
        }
    }
}

async fn uses_youtube(app: &AppHandle) -> bool {
    let pool = app.state::<AppState>().db.clone();
    backend(&pool).await == Backend::YouTube
}

/// Build the YouTube audio backend if it does not exist yet.
async fn ensure_yt(app: &AppHandle) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let db       = s.db.clone();
    let yt       = s.yt.clone();
    let media_tx = s.media_tx.clone();
    drop(s);

    let mut guard = yt.lock().await;
    if guard.is_none() {
        let vol   = read_vol(&db).await;
        let muted = read_muted(&db).await;
        *guard = Some(crate::playback::youtube::YtPlayback::new(
            app.clone(),
            crate::playback::SharedVolume::new(vol, muted),
            media_tx,
        )?);
    }
    Ok(())
}

/// Build the match query for a track, fetching it from Spotify if the local
/// catalog doesn't have it yet.
///
/// The local `tracks` table only holds what library sync and page loads have
/// written. Plenty of playable tracks never pass through either - anything
/// started from search results, radio, a queue entry or an autoplay
/// continuation can be pressed play on while its row does not exist. Treating
/// that as "track not found" made every such play fail on the YouTube backend.
///
/// The query is built from the API response directly rather than by upserting
/// and re-reading, so playback does not depend on the cache write succeeding.
/// The upsert still happens, best-effort, so the next play skips the round trip.
async fn match_query(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<crate::youtube::TrackQuery, AppError> {
    match crate::youtube::query_for_track(pool, id).await {
        Ok(q) => return Ok(q),
        // Only a missing/incomplete row is worth a network fallback; a real
        // database error should surface as itself.
        Err(AppError::NotFound(_)) => {}
        Err(e) => return Err(e),
    }

    let token = crate::commands::spotify::tok(app).await?;
    let track: crate::spotify::types::SpTrack = crate::spotify::spotify_get(
        &token,
        &format!("{}/tracks/{id}", crate::commands::spotify::BASE),
    )
    .await?;

    if track.artists.is_empty() {
        // Without an artist the matcher can't tell two identically titled
        // songs apart, so refuse rather than match on title alone.
        return Err(AppError::NotFound(format!(
            "\"{}\" has no artist information; can't match it safely",
            track.name
        )));
    }

    let query = crate::youtube::TrackQuery {
        title:       track.name.clone(),
        artists:     track.artists.iter().map(|a| a.name.clone()).collect(),
        album:       track.album.as_ref().map(|a| a.name.clone()),
        duration_ms: track.duration_ms.max(0) as u64,
        explicit:    track.explicit,
    };

    // Best-effort catalog fill. Failures here are logged, never fatal - the
    // query above is already complete.
    if let Err(e) = cache_track(pool, &track).await {
        eprintln!("[playback] caching {id} for later failed: {e}");
    }

    Ok(query)
}

/// Write a fetched track into the local catalog: album, artists, track and the
/// join rows `query_for_track` reads back. The album has to go first -
/// `tracks.album_id` is a foreign key, so upserting the track alone fails for
/// exactly the not-yet-cached tracks this fallback exists for.
async fn cache_track(
    pool: &sqlx::SqlitePool,
    track: &crate::spotify::types::SpTrack,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    crate::library::upsert_track_with_deps(&mut tx, track).await?;
    tx.commit().await?;
    Ok(())
}

/// Latest YouTube transport request. `yt_play` downloads before it takes the
/// player lock, so a slow load can finish after the user has already moved on
/// (clicked another track, paused). Every transport command claims a new
/// ticket; a load whose ticket is no longer current when its download finishes
/// is dropped instead of replacing what the user asked for last.
static YT_REQUEST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn yt_claim() -> u64 {
    YT_REQUEST.fetch_add(1, Ordering::SeqCst) + 1
}

/// Resolve, fetch and play a Spotify track through YouTube Music.
///
/// Resolution, extraction and download all happen *before* the state lock is
/// taken. The download is ~1s of network work; holding the playback mutex
/// across it would block every transport command for its duration.
async fn yt_play(app: &AppHandle, id: &str, position_ms: u32) -> Result<(), AppError> {
    if let Some(episode_id) = id.strip_prefix("spotify:episode:") {
        return yt_play_episode(app, id, episode_id, position_ms).await;
    }
    let ticket = yt_claim();
    ensure_yt(app).await?;
    let pool = app.state::<AppState>().db.clone();

    {
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        if YT_REQUEST.load(Ordering::SeqCst) != ticket { return Ok(()); }
        if let Some(player) = guard.as_ref() {
            player.pause()?;
        }
    }
    // Uses the prefetched copy when `preload_track` already fetched this track,
    // which is the difference between an instant track change and a ~1s gap.
    let prepared = match crate::youtube::take_prepared(id).await {
        Some(prepared) => prepared,
        None => {
            let query = match_query(app, &pool, id).await?;
            crate::youtube::prepare(&pool, id, &query).await?
        }
    };

    let yt    = app.state::<AppState>().yt.clone();
    let guard = yt.lock().await;
    // Checked under the lock: a newer yt_play also plays under it, so this
    // cannot interleave with the load that superseded us.
    if YT_REQUEST.load(Ordering::SeqCst) != ticket {
        eprintln!("[youtube] dropping superseded load of {id}");
        return Ok(());
    }
    if let Some(player) = guard.as_ref() {
        player.play_track(
            id,
            &prepared.stream,
            std::sync::Arc::clone(&prepared.audio),
            position_ms,
        )?;
    }
    Ok(())
}

/// Play a podcast episode on the YouTube backend. Spotify only streams episodes
/// to its own session, so the audio comes from the show's RSS feed (or YouTube
/// Music) instead - see `episode_audio`. Episodes stream rather than download
/// whole, so this starts on the first chunk.
async fn yt_play_episode(
    app: &AppHandle,
    id: &str,
    episode_id: &str,
    position_ms: u32,
) -> Result<(), AppError> {
    use rodio::Source;

    let ticket = yt_claim();
    let current = || YT_REQUEST.load(Ordering::SeqCst) == ticket;
    ensure_yt(app).await?;
    {
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        if !current() { return Ok(()); }
        if let Some(player) = guard.as_ref() {
            player.pause()?;
        }
    }
    let pool = app.state::<AppState>().db.clone();
    let token = crate::commands::spotify::tok(app).await?;

    let found = crate::episode_audio::resolve(&pool, &token, episode_id).await?;
    if !current() {
        return Ok(());
    }
    let opened = match crate::episode_audio::remote::open(&found.url, found.user_agent).await {
        Ok(opened) => opened,
        Err(e) => {
            // a remembered url that stopped working shouldn't stick
            crate::episode_audio::forget(&pool, episode_id).await;
            return Err(e);
        }
    };
    if !current() {
        return Ok(());
    }
    eprintln!("[youtube] episode {episode_id} via {} ({} bytes)", found.via, opened.len);

    let handle = opened.handle.clone();
    if position_ms > 0 && found.duration_ms > 0 {
        // rough (see EpisodeStream::byte_for); the exact spot for an mp4 is
        // only known once its index is read, below
        let byte = (opened.len as u128 * position_ms as u128 / found.duration_ms as u128) as u64;
        handle.prefetch(byte.saturating_sub(64 * 1024), std::time::Duration::from_secs(8)).await;
    }

    // the container type without its codec parameters, which the probe
    // doesn't match on; the feed's word first, the server's second
    let mime = found
        .mime
        .or(opened.content_type)
        .map(|m| m.split(';').next().unwrap_or_default().trim().to_string())
        .filter(|m| m.starts_with("audio/"));
    let reader = opened.reader;
    let clock = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let source_clock = std::sync::Arc::clone(&clock);
    // probing and the first seek read the file, which can wait on the network,
    // so they run on a blocking thread rather than an async worker
    let (source, segments, started_at) = tokio::task::spawn_blocking(move || -> Result<_, AppError> {
        let mut decoder = crate::episode_audio::source::EpisodeDecoder::open(reader, mime)
            .map_err(|e| AppError::Playback(format!("episode decode: {e}")))?;
        let segments = decoder.segments();
        let mut start = 0u32;
        if position_ms > 0 {
            match decoder.try_seek(std::time::Duration::from_millis(position_ms as u64)) {
                Ok(()) => start = position_ms,
                Err(e) => eprintln!("[youtube] episode seek to {position_ms}ms on load failed: {e}"),
            }
        }
        Ok((crate::episode_audio::source::EpisodeSource::new(decoder, source_clock, start as u64), segments, start))
    })
    .await
    .map_err(|e| AppError::Playback(format!("episode decode: {e}")))??;

    let stream = crate::playback::youtube::EpisodeStream { handle, duration_ms: found.duration_ms, segments };
    let yt = app.state::<AppState>().yt.clone();
    let guard = yt.lock().await;
    if !current() {
        eprintln!("[youtube] dropping superseded load of {id}");
        return Ok(());
    }
    if let Some(player) = guard.as_ref() {
        player.play_episode(id, source, clock, stream, started_at);
    }
    Ok(())
}

/// Seek on the YouTube backend. Music is fully in memory, so that's instant;
/// a streaming episode first pulls the bytes around the target so the audio
/// thread doesn't sit waiting on the network mid-seek.
async fn yt_seek(app: &AppHandle, position_ms: u32) -> Result<(), AppError> {
    let yt = app.state::<AppState>().yt.clone();
    let maybe_episode = {
        let guard = yt.lock().await;
        let Some(player) = guard.as_ref() else { return Ok(()); };
        // Seek the same player whose stream we snapshot, before any network
        // wait can let another track take its place.
        player.seek(position_ms)?;
        player.episode_stream()
    };
    if let Some(episode) = maybe_episode {
        if let Some(byte) = episode.byte_for(position_ms) {
            episode.handle.prefetch(byte, std::time::Duration::from_secs(6)).await;
        }
    }
    Ok(())
}

/// Resolve, extract and download a track ahead of playing it.
///
/// Fire-and-forget: a preload failing must never surface to the user or block
/// anything, because the track may never actually be played. If it does get
/// played, `yt_play` simply pays the cost then instead.
async fn yt_preload(app: &AppHandle, id: &str) {
    let pool = app.state::<AppState>().db.clone();
    if let Some(episode_id) = id.strip_prefix("spotify:episode:") {
        // not the audio (that's hundreds of MB) - just find where it lives,
        // so pressing play skips the feed lookup
        if let Ok(token) = crate::commands::spotify::tok(app).await {
            if let Err(e) = crate::episode_audio::resolve(&pool, &token, episode_id).await {
                eprintln!("[youtube] preload {id}: {e}");
            }
        }
        return;
    }
    let query = match match_query(app, &pool, id).await {
        Ok(q) => q,
        Err(e) => {
            eprintln!("[youtube] preload {id}: {e}");
            return;
        }
    };
    match crate::youtube::prepare(&pool, id, &query).await {
        Ok(p) => eprintln!(
            "[youtube] preloaded {id} ({} bytes, itag {})",
            p.audio.len(),
            p.stream.format.itag
        ),
        Err(e) => eprintln!("[youtube] preload {id}: {e}"),
    }
}

// backend / match inspection commands
//
// These exist so the YouTube path is never a black box to the user. When audio
// comes from somewhere other than Spotify, they can see that it does, see which
// upload was chosen, and correct it. A wrong match the user cannot fix would be
// worse than no match at all.

#[derive(Debug, Clone, Serialize)]
pub struct BackendState {
    /// Backend actually in use right now: "spotify" or "youtube".
    pub active:            String,
    /// What plays under that choice: "librespot", "soloist" or "youtube".
    pub engine:            String,
    /// Spotify account product, when known.
    pub product:           Option<String>,
    /// Whether Spotify is a selectable option. False on a free account, which
    /// is what the settings UI disables the Spotify choice on.
    pub spotify_available: bool,
}

#[tauri::command]
pub async fn get_playback_backend(app: AppHandle) -> Result<BackendState, AppError> {
    let pool = app.state::<AppState>().db.clone();

    // `active` doubles as the selected value in the UI. There is no separate
    // "setting" to report any more: with the automatic option gone, the
    // resolved backend *is* the selection, and a free account's forced
    // YouTube shows up as YouTube being selected rather than as a stored
    // preference that disagrees with reality.
    let backend = backend(&pool).await;
    Ok(BackendState {
        active: match backend {
            Backend::Spotify | Backend::Soloist => "spotify".into(),
            Backend::YouTube => "youtube".into(),
        },
        engine: match backend {
            Backend::Spotify => "librespot".into(),
            Backend::Soloist => "soloist".into(),
            Backend::YouTube => "youtube".into(),
        },
        product:           setting_value(&pool, "spotify_product").await,
        spotify_available: spotify_available(&pool).await,
    })
}

/// Choose the audio backend. Accepts "spotify" or "youtube".
#[tauri::command]
pub async fn set_playback_backend(app: AppHandle, mode: String) -> Result<(), AppError> {
    if !matches!(mode.as_str(), "spotify" | "youtube") {
        return Err(AppError::InvalidInput(format!(
            "unknown playback backend {mode:?}; expected spotify or youtube"
        )));
    }
    let pool = app.state::<AppState>().db.clone();

    // The UI disables this choice on a free account, but enforce it here too -
    // storing "spotify" for an account that cannot stream would produce silent
    // playback with no visible cause.
    if mode == "spotify" && !spotify_available(&pool).await {
        return Err(AppError::InvalidInput(
            "Spotify playback requires Premium".into(),
        ));
    }

    // Captured before the write, since `backend()` resolves from the setting.
    let previous = backend(&pool).await;
    save_setting(&pool, "playback_backend", &mode).await?;
    let next = backend(&pool).await;

    if previous != next {
        yt_claim();
        // Silence whichever backend was playing. Each owns its own audio
        // device and neither knows about the other, so without this the old
        // one keeps playing underneath the new one and you hear both tracks
        // at once.
        //
        // Pause rather than stop: the track and position survive, so pressing
        // play resumes where it left off, now through the new backend.
        silence(&app, previous).await;
    }

    // Prepared YouTube buffers are useless once audio comes from Spotify, and
    // holding several megabytes for a backend that is no longer running is
    // pure waste.
    if next != Backend::YouTube {
        crate::youtube::clear_prepared().await;
    }
    Ok(())
}

/// The cached YouTube mapping for a track, if any.
#[tauri::command]
pub async fn get_yt_match(
    app: AppHandle,
    track_id: String,
) -> Result<Option<crate::db::repos::yt_match::YtMatch>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    crate::db::repos::yt_match::get(&pool, &track_id).await
}

#[derive(Debug, Clone, Serialize)]
pub struct YtCandidate {
    pub video_id:    String,
    pub title:       String,
    pub artists:     Vec<String>,
    pub album:       Option<String>,
    pub duration_ms: Option<u64>,
    pub explicit:    bool,
}

/// Search YouTube Music for a track and return the raw candidates, ungated.
///
/// This powers the manual-override picker, so it deliberately does *not* apply
/// the matcher's gates: the whole point is to show the user what was rejected
/// so they can choose for themselves. Automatic playback never uses this.
#[tauri::command]
pub async fn search_yt_candidates(
    app: AppHandle,
    track_id: String,
) -> Result<Vec<YtCandidate>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    // Same fallback as playback - the picker must work for a track that is not
    // in the local catalog, which is exactly when a user is most likely to be
    // investigating a failed match.
    let query = match_query(&app, &pool, &track_id).await?;
    let terms = format!(
        "{} {}",
        query.title,
        query.artists.first().map(String::as_str).unwrap_or_default()
    );

    Ok(crate::youtube::search::songs(terms.trim())
        .await?
        .into_iter()
        .map(|r| YtCandidate {
            video_id:    r.video_id,
            title:       r.title,
            artists:     r.artists,
            album:       r.album,
            duration_ms: r.duration_ms,
            explicit:    r.explicit,
        })
        .collect())
}

/// Pin a user-chosen video for a track. Survives re-resolution.
#[tauri::command]
pub async fn pin_yt_match(
    app: AppHandle,
    track_id: String,
    video_id: String,
) -> Result<(), AppError> {
    let pool = app.state::<AppState>().db.clone();
    crate::db::repos::yt_match::pin(&pool, &track_id, &video_id).await?;
    // Otherwise the next play reuses the already-downloaded wrong upload.
    crate::youtube::evict_prepared(&track_id).await;
    Ok(())
}

/// Forget a cached mapping so the next play re-resolves it.
#[tauri::command]
pub async fn forget_yt_match(app: AppHandle, track_id: String) -> Result<(), AppError> {
    let pool = app.state::<AppState>().db.clone();
    crate::db::repos::yt_match::forget(&pool, &track_id).await?;
    crate::youtube::evict_prepared(&track_id).await;
    Ok(())
}

// commands

// pre warms the librespot session so the first play_track is instant.
// call it after login. idempotent so its safe to call over and over
#[tauri::command]
pub async fn warmup_playback(app: AppHandle) -> Result<(), AppError> {
    if uses_youtube(&app).await {
        // Scraping the visitor id costs a ~380 KB page fetch, measured at
        // ~630ms. Cached process-wide once fetched, so doing it here moves it
        // off the first play entirely. Detached because nothing below needs it
        // and a failure is non-fatal (see visitor.rs).
        tokio::spawn(async { crate::youtube::visitor::get().await; });

        // Opening the audio device is the other one-time cost on this path.
        if let Err(e) = ensure_yt(&app).await {
            eprintln!("[playback] youtube warmup failed: {e}");
        }
    }
    ensure_inner_with(&app, false).await
}

/// Startup warm-up, fired from `setup()` the moment app state exists.
///
/// The frontend also calls `warmup_playback`, but only after the webview has
/// booted, mounted React, round-tripped `get_auth_status` over IPC and flipped
/// `isLoggedIn`. Building the session is the expensive part of the first play -
/// connecting to the access point, registering the Connect device, waiting on
/// the country packet - and none of it depends on the UI. Starting it here runs
/// it concurrently with webview startup, so by the time there is something to
/// click, the session is usually already up.
///
/// Deliberately conservative: this returns early unless everything needed is
/// already on disk, and builds the session non-interactively. A stored playback
/// token is usually expired by the next launch, so the on-disk check alone does
/// not stop `create_inner` from falling back to a browser authorization - the
/// `interactive = false` flag does. A background task must never do that
/// unprompted; if no silent credential works, the user's first play recovers.
pub async fn warm_session_if_possible(app: AppHandle) {
    use tauri::Manager;

    let s = app.state::<AppState>();
    let db = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    // not logged in yet -> nothing to warm, and get_valid_token would just fail
    if auth.read().await.refresh_token.is_none() {
        return;
    }

    // would create_inner have to go interactive? if so, leave it to the user's
    // first real play (or the frontend's warmup after an explicit login).
    let has_cached_creds = app
        .path()
        .app_data_dir()
        .map(|d| d.join("credentials").join("credentials.json").exists())
        .unwrap_or(false);
    if !has_cached_creds {
        let has_token = crate::auth::get_setting_value(&db, "spotify_playback_token")
            .await
            .ok()
            .flatten()
            .is_some_and(|t| !t.trim().is_empty());
        if !has_token {
            return;
        }
    }

    if let Err(e) = ensure_inner_with(&app, false).await {
        eprintln!("[playback] startup warm-up skipped: {e}");
    }
}

#[tauri::command]
pub async fn play_track(app: AppHandle, id: String, context_uri: Option<String>) -> Result<(), AppError> {
    refuse_while_probing()?;
    eprintln!("[playback cmd] play_track id={id} context={context_uri:?}");

    if let Some(soloist) = soloist(&app).await {
        soloist.play(&crate::playback::track_uri(&id)?, 0)?;
    } else if uses_youtube(&app).await {
        // A resolution failure here is a real, expected outcome (no acceptable
        // YouTube match). It propagates to the frontend as an error naming the
        // track rather than silently playing something else.
        yt_play(&app, &id, 0).await?;
    } else {
        ensure_inner(&app).await?;
        let uri      = crate::playback::track_uri(&id)?;
        let playback = app.state::<AppState>().playback.clone();
        let guard    = playback.lock().await;
        if let Some(inner) = guard.as_ref() {
            inner.play_requested(uri, context_uri)?;
        }
    }

    let pool = app.state::<AppState>().db.clone();
    let track_id = id.clone();
    tokio::spawn(async move {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let _ = sqlx::query("INSERT INTO recently_played (track_id, played_at) VALUES (?, ?)")
            .bind(&track_id)
            .bind(now)
            .execute(&pool)
            .await;
    });

    Ok(())
}

// recovery path for a track that came back Unavailable. every so often the
// access point just stops answering audio key requests for a given librespot session
// (the key request times out even after in session retries) which makes an
// otherwise playable track fail to load. a brand new session almost always gets
// a healthy key channel so here we tear the current session down and rebuild a
// fresh one before loading the track again. the frontend calls this AT MOST once
// per track (see App.tsx) so a genuinely unavailable track cant loop. safe
// cuz it only runs when the failed track is the current one, i.e nothings
// actually playing to interrupt
#[tauri::command]
pub async fn retry_play_track(app: AppHandle, id: String, context_uri: Option<String>) -> Result<(), AppError> {
    refuse_while_probing()?;
    if let Some(soloist) = soloist(&app).await {
        // Soloist keeps its own session; there is nothing of ours to rebuild
        return soloist.play(&crate::playback::track_uri(&id)?, 0);
    }
    {
        // drop the wedged session so ensure_inner just builds a fresh one
        let playback = app.state::<AppState>().playback.clone();
        let mut guard = playback.lock().await;
        *guard = None;
    }
    ensure_inner(&app).await?;

    let uri      = crate::playback::track_uri(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.set_play_context(&uri, context_uri);
        inner.play_uri(uri, 0)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pause_playback(app: AppHandle) -> Result<(), AppError> {
    refuse_while_probing()?;
    if let Some(soloist) = soloist(&app).await {
        return soloist.pause();
    }
    if uses_youtube(&app).await {
        // cancels a load still downloading, so it can't start audio after this
        yt_claim();
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        if let Some(player) = guard.as_ref() {
            player.pause()?;
        }
        return Ok(());
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.pause()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_playback(app: AppHandle) -> Result<(), AppError> {
    refuse_while_probing()?;
    if let Some(soloist) = soloist(&app).await {
        // a track that played to its end has nothing left to carry on with
        return match soloist.current_uri() {
            Some(uri) if !soloist.resumable(&uri) => soloist.play(&uri, 0),
            _ => soloist.resume(),
        };
    }
    if uses_youtube(&app).await {
        // After end-of-track the decoder queue is drained, so there is nothing
        // to un-pause - the track has to be fetched and decoded again.
        let (ended, track) = {
            let yt = app.state::<AppState>().yt.clone();
            let guard = yt.lock().await;
            match guard.as_ref() {
                Some(p) => (p.is_ended(), p.current_track()),
                None => (false, None),
            }
        };

        match (ended, track) {
            (true, Some(id)) => yt_play(&app, &id, 0).await?,
            _ => {
                yt_claim();
                let yt = app.state::<AppState>().yt.clone();
                let guard = yt.lock().await;
                if let Some(player) = guard.as_ref() {
                    player.resume()?;
                }
            }
        }
        return Ok(());
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        if !inner.is_ended() {
            if let Err(e) = inner.resume() {
                eprintln!("[playback cmd] resume failed ({e}), falling back to play_uri");
                if let Some(uri) = inner.current_uri() {
                    inner.play_uri(uri, 0)?;
                }
            }
        } else if let Some(uri) = inner.current_uri() {
            inner.play_uri(uri, 0)?;
        }
    }
    Ok(())
}

// the ▶ button. resumes if the matching track is loaded and not ended,
// otherwise loads and plays the requested track at position_ms.
#[tauri::command]
pub async fn resume_or_play(
    app:         AppHandle,
    id:          String,
    position_ms: u32,
    context_uri: Option<String>,
) -> Result<(), AppError> {
    refuse_while_probing()?;
    eprintln!("[playback cmd] resume_or_play id={id} pos={position_ms}");

    if let Some(soloist) = soloist(&app).await {
        let uri = crate::playback::track_uri(&id)?;
        if !soloist.resumable(&uri) {
            return soloist.play(&uri, position_ms);
        }
        // the same track, still loaded: carry on, moving only if another
        // device took it somewhere else meanwhile
        if position_ms > 0 && soloist.position_ms().abs_diff(position_ms) > 2000 {
            soloist.seek(position_ms)?;
        }
        return soloist.resume();
    }

    if uses_youtube(&app).await {
        // Un-pause when the very same track is still loaded and live; anything
        // else is a fresh load. The frontend passes the current position on
        // every resume, so a non-zero position must not force a reload - the
        // whole track is in memory, so seek in place when it has drifted.
        let resumable = {
            let yt = app.state::<AppState>().yt.clone();
            let guard = yt.lock().await;
            guard.as_ref().is_some_and(|p| {
                p.current_track().as_deref() == Some(id.as_str()) && !p.is_ended()
            })
        };

        if resumable {
            yt_claim();
            let yt = app.state::<AppState>().yt.clone();
            let drifted = yt
                .lock()
                .await
                .as_ref()
                .is_some_and(|p| position_ms > 0 && p.position_ms().abs_diff(position_ms) > 2000);
            if drifted {
                if let Err(e) = yt_seek(&app, position_ms).await {
                    eprintln!("[youtube] seek on resume failed: {e}");
                }
            }
            let guard = yt.lock().await;
            if let Some(player) = guard.as_ref() {
                player.resume()?;
            }
        } else {
            yt_play(&app, &id, position_ms).await?;
        }
        return Ok(());
    }

    ensure_inner(&app).await?;

    let uri      = crate::playback::track_uri(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        let loaded = inner.loaded.load(Ordering::Relaxed);
        let same_uri = inner.current_uri().as_deref() == Some(&uri);
        let ended = inner.is_ended();
        inner.set_play_context(&uri, context_uri);

        // Ordinary resume carries the UI's current position. Reloading here
        // discards buffered audio and waits for metadata/audio keys again.
        if loaded && same_uri && !ended {
            // A transfer back from another device may have moved the same
            // track. Seek in place then, while normal resume keeps its buffer.
            if position_ms > 0 && inner.position_ms().abs_diff(position_ms) > 2000 {
                inner.seek(position_ms)?;
            }
            if let Err(e) = inner.resume() {
                eprintln!("[playback cmd] resume failed ({e}), falling back to play_uri");
                inner.play_uri(uri, position_ms)?;
            }
        } else {
            inner.play_uri(uri, position_ms)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_playback(app: AppHandle) -> Result<(), AppError> {
    refuse_while_probing()?;
    if let Some(soloist) = soloist(&app).await {
        return soloist.pause();
    }
    if uses_youtube(&app).await {
        yt_claim();
        // Unlike the Spirc path there is no connect-state to keep consistent,
        // so pausing is the right analogue: it keeps the decoded track loaded
        // so pressing play again is instant instead of a re-download.
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        if let Some(player) = guard.as_ref() {
            player.pause()?;
        }
        return Ok(());
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        // Spirc has no hard "stop"; pause keeps the connect-state consistent
        // (device stays active/paused) instead of desyncing by poking the player
        // directly. The frontend uses this rarely (mostly pause is what's wanted).
        inner.pause()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn seek_playback(app: AppHandle, position_ms: u32) -> Result<(), AppError> {
    refuse_while_probing()?;
    if let Some(soloist) = soloist(&app).await {
        return soloist.seek(position_ms);
    }
    if uses_youtube(&app).await {
        return yt_seek(&app, position_ms).await;
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.seek(position_ms)?;
    }
    Ok(())
}

// preload the next track so it kicks in instantly when the current one ends
#[tauri::command]
pub async fn preload_track(app: AppHandle, id: String) -> Result<(), AppError> {
    refuse_while_probing()?;
    if soloist(&app).await.is_some() {
        // Soloist fetches what it plays itself, and anything put in its
        // queue could never be taken out again if the app's queue changed
        return Ok(());
    }
    if uses_youtube(&app).await {
        // Returns immediately - the frontend fires this on a timer while a
        // track is playing, so it must not hold up the IPC call or the caller.
        let app = app.clone();
        tokio::spawn(async move { yt_preload(&app, &id).await });
        return Ok(());
    }

    let track_id = crate::playback::parse_track_id(&id)?;
    let uri      = crate::playback::track_uri(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.player.preload(track_id);
        // the interface only preloads what its queue plays next, so this is
        // also what the engine moves on to by itself when this track ends
        inner.set_next(uri);
    }
    Ok(())
}

#[tauri::command]
pub async fn set_volume(app: AppHandle, level: u8) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let pool     = s.db.clone();
    let playback = s.playback.clone();
    let yt       = s.yt.clone();
    drop(s);

    let level_f = level as f64 / 100.0;

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.volume.set_level(level_f);
        // keep Spotify's reported device volume in sync with ours
        inner.report_volume(level_f);
    }
    drop(guard);

    // The two backends own separate volume state, so both are updated
    // unconditionally rather than only the active one - otherwise switching
    // backends would jump the volume back to whatever the other last saw.
    let yt_guard = yt.lock().await;
    if let Some(player) = yt_guard.as_ref() {
        player.set_level(level_f);
    }
    drop(yt_guard);

    // Soloist has one level and no mute of its own
    if !read_muted(&pool).await {
        app.state::<AppState>().soloist.set_volume(level.min(100));
    }

    save_setting(&pool, "player_volume", &level.to_string()).await
}

#[tauri::command]
pub async fn set_muted(app: AppHandle, muted: bool) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let pool     = s.db.clone();
    let playback = s.playback.clone();
    let yt       = s.yt.clone();
    drop(s);

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.volume.set_muted(muted);
    }
    drop(guard);

    let yt_guard = yt.lock().await;
    if let Some(player) = yt_guard.as_ref() {
        player.set_muted(muted);
    }
    drop(yt_guard);

    let level = if muted { 0 } else { (read_vol(&pool).await * 100.0).round().clamp(0.0, 100.0) as u8 };
    app.state::<AppState>().soloist.set_volume(level);

    save_setting(&pool, "player_muted", if muted { "1" } else { "0" }).await
}

#[derive(Debug, Clone, Serialize)]
pub struct VolumeState {
    pub level: u8,   // 0-100
    pub muted: bool,
}

#[tauri::command]
pub async fn get_volume(app: AppHandle) -> Result<VolumeState, AppError> {
    let s        = app.state::<AppState>();
    let pool     = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        return Ok(VolumeState {
            level: (inner.volume.level() * 100.0).round() as u8,
            muted: inner.volume.is_muted(),
        });
    }
    drop(guard);

    let level = (read_vol(&pool).await * 100.0).round() as u8;
    let muted = read_muted(&pool).await;
    Ok(VolumeState { level, muted })
}

#[tauri::command]
pub async fn get_audio_quality(app: AppHandle) -> Result<String, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    drop(s);

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'audio_quality'",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    Ok(row.map(|(v,)| v).unwrap_or_else(|| "320".to_string()))
}

#[tauri::command]
pub async fn set_audio_quality(app: AppHandle, quality: String) -> Result<(), AppError> {
    let val = match quality.as_str() {
        "96" => "96",
        "160" => "160",
        _ => "320",
    };

    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    save_setting(&pool, "audio_quality", val).await?;

    // Reset playback session if idle so the new bitrate takes effect on next playback.
    // If music is actively playing, keep the session so it doesn't cut off mid-track.
    let mut guard = playback.lock().await;
    let playing = guard.as_ref().map(|i| i.is_playing()).unwrap_or(false);
    if !playing {
        *guard = None;
    } else if let Some(inner) = guard.as_ref() {
        inner.needs_rebuild.store(true, Ordering::Relaxed);
    }

    Ok(())
}

/// What the equalizer is set to, and the bands it has.
#[derive(Debug, Clone, Serialize)]
pub struct EqualizerState {
    pub enabled:     bool,
    /// One gain per band, in dB.
    pub gains:       Vec<f32>,
    /// The centre of each band, in Hz.
    pub bands:       Vec<f32>,
    /// How far a band can be moved either way, in dB.
    pub max_gain_db: f32,
}

fn equalizer_state() -> EqualizerState {
    let settings = crate::eq::settings();
    EqualizerState {
        enabled:     settings.enabled,
        gains:       settings.gains,
        bands:       crate::eq::BANDS.to_vec(),
        max_gain_db: crate::eq::MAX_GAIN_DB,
    }
}

#[tauri::command]
pub async fn get_equalizer() -> Result<EqualizerState, AppError> {
    Ok(equalizer_state())
}

/// Set the equalizer. It is heard at once, on whichever backend is playing
/// through the app's own output (not Soloist, which owns its own).
#[tauri::command]
pub async fn set_equalizer(app: AppHandle, enabled: bool, gains: Vec<f32>) -> Result<EqualizerState, AppError> {
    crate::eq::set(enabled, &gains);
    let pool = app.state::<AppState>().db.clone();
    save_setting(&pool, "equalizer", &crate::eq::encode(&crate::eq::settings())).await?;
    Ok(equalizer_state())
}

/// "linear" or "cubic": how the volume slider maps to loudness.
#[tauri::command]
pub async fn get_volume_curve() -> Result<String, AppError> {
    Ok(if crate::playback::cubic_volume() { "cubic" } else { "linear" }.into())
}

#[tauri::command]
pub async fn set_volume_curve(app: AppHandle, curve: String) -> Result<(), AppError> {
    let cubic = match curve.as_str() {
        "cubic" => true,
        "linear" => false,
        other => {
            return Err(AppError::InvalidInput(format!(
                "unknown volume curve {other:?}; expected linear or cubic"
            )))
        }
    };
    crate::playback::set_cubic_volume(cubic);
    let pool = app.state::<AppState>().db.clone();
    save_setting(&pool, "volume_curve", if cubic { "cubic" } else { "linear" }).await
}

/// The sound settings that live outside any one session, read once at start.
pub async fn load_sound_settings(pool: &sqlx::SqlitePool) {
    if let Some(saved) = setting_value(pool, "equalizer").await.as_deref().and_then(crate::eq::decode) {
        crate::eq::set(saved.enabled, &saved.gains);
    }
    crate::playback::set_cubic_volume(setting_value(pool, "volume_curve").await.as_deref() == Some("cubic"));
}

/// Whether tracks are levelled to the same loudness.
#[tauri::command]
pub async fn get_audio_normalisation(app: AppHandle) -> Result<bool, AppError> {
    let pool = app.state::<AppState>().db.clone();
    Ok(setting_value(&pool, "audio_normalisation").await.as_deref() == Some("1"))
}

/// Turn loudness normalisation on or off. Like the bitrate, it is the
/// player's from the moment it is built, so it takes hold with the next
/// session: at once when nothing is playing, after this track otherwise.
#[tauri::command]
pub async fn set_audio_normalisation(app: AppHandle, enabled: bool) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    save_setting(&pool, "audio_normalisation", if enabled { "1" } else { "0" }).await?;

    let mut guard = playback.lock().await;
    let playing = guard.as_ref().map(|i| i.is_playing()).unwrap_or(false);
    if !playing {
        *guard = None;
    } else if let Some(inner) = guard.as_ref() {
        inner.needs_rebuild.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_audio_cache_limit(app: AppHandle) -> Result<u64, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    drop(s);

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'audio_cache_limit_mb'",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let limit = row.and_then(|(v,)| v.parse::<u64>().ok()).unwrap_or(2048);
    Ok(limit)
}

#[tauri::command]
pub async fn set_audio_cache_limit(app: AppHandle, limit_mb: u64) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    save_setting(&pool, "audio_cache_limit_mb", &limit_mb.to_string()).await?;

    let mut guard = playback.lock().await;
    let playing = guard.as_ref().map(|i| i.is_playing()).unwrap_or(false);
    if !playing {
        *guard = None;
    } else if let Some(inner) = guard.as_ref() {
        inner.needs_rebuild.store(true, Ordering::Relaxed);
    }

    Ok(())
}

/// Returns the current audio output latency in milliseconds.
///
/// Accounts for both the audio chunks queued in rodio's sink and the
/// negotiated hardware buffer duration granted to cpal. `None` means "no
/// reading" (playback not initialised, or the null sink): the frontend keeps
/// its last good figure. `Some(0)` is a real answer - Soloist reports
/// positions as heard - and must not be confused with "unknown", or the
/// frontend's 250ms placeholder stays in and every lyric runs late.
#[tauri::command]
pub async fn get_output_latency_ms(app: AppHandle) -> Result<Option<i64>, AppError> {
    // Has to follow the active backend. The librespot sink only updates its
    // figure while it is actually writing audio, so on the YouTube path it
    // reports 0, which is "no reading", not "no latency".
    if soloist(&app).await.is_some() {
        // Soloist reports positions as it plays them and owns its own output
        return Ok(Some(0));
    }
    if uses_youtube(&app).await {
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        return Ok(guard.as_ref().map(|p| p.output_latency_ms()).filter(|&ms| ms > 0));
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    Ok(guard.as_ref().map(|inner| inner.output_latency_ms()).filter(|&ms| ms > 0))
}

// ── connect-driven playback (spotify jam) ───────────────────────────────────
//
// in a jam the queue belongs to spotify: spirc holds it and social-connect
// edits it for everyone. these drive spirc's queue directly instead of the
// app's own, and the ui mirrors it from the "connect:state" event.

async fn with_spirc<T>(
    app: &AppHandle,
    f: impl FnOnce(&crate::playback::PlaybackInner) -> Result<T, AppError>,
) -> Result<T, AppError> {
    if uses_youtube(app).await {
        return Err(AppError::Playback(
            "Jams play through Spotify. Switch the audio source to Spotify in Settings.".into(),
        ));
    }
    if soloist(app).await.is_some() {
        return Err(AppError::Playback(
            "Jams play through the built-in player. Turn Soloist off to join one.".into(),
        ));
    }
    ensure_inner(app).await?;
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    let inner = guard
        .as_ref()
        .ok_or_else(|| AppError::Playback("spotify session unavailable".into()))?;
    f(inner)
}

/// the queue, current track and context this device last reported
#[tauri::command]
pub async fn get_connect_state(app: AppHandle) -> Result<Option<crate::playback::ConnectStateMsg>, AppError> {
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    Ok(guard.as_ref().map(|inner| inner.connect_state()))
}

/// play a list through spirc so it becomes the connect queue (a jam host's
/// music is what the jam hears). `keep_stream` carries on the track already
/// playing when it's the one at `index`
#[tauri::command]
pub async fn connect_load_tracks(
    app: AppHandle,
    ids: Vec<String>,
    index: u32,
    position_ms: u32,
    start_playing: bool,
    keep_stream: bool,
) -> Result<(), AppError> {
    let uris = ids
        .iter()
        .map(|id| crate::playback::track_uri(id))
        .collect::<Result<Vec<_>, _>>()?;
    if uris.is_empty() {
        return Err(AppError::InvalidInput("nothing to play".into()));
    }
    let index = index.min(uris.len() as u32 - 1);
    with_spirc(&app, |inner| inner.load_tracks(uris, index, position_ms, start_playing, keep_stream)).await
}

#[tauri::command]
pub async fn connect_add_to_queue(app: AppHandle, id: String) -> Result<(), AppError> {
    let uri = crate::playback::track_uri(&id)?;
    with_spirc(&app, |inner| inner.add_to_queue(uri)).await
}

#[tauri::command]
pub async fn connect_skip_to(app: AppHandle, id: String) -> Result<(), AppError> {
    let uri = crate::playback::track_uri(&id)?;
    with_spirc(&app, |inner| inner.skip_to(uri)).await
}

#[tauri::command]
pub async fn connect_next(app: AppHandle) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.next()).await
}

#[tauri::command]
pub async fn connect_prev(app: AppHandle) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.prev()).await
}

#[tauri::command]
pub async fn connect_set_shuffle(app: AppHandle, on: bool) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.set_shuffle(on)).await
}

#[tauri::command]
pub async fn connect_set_repeat(app: AppHandle, context: bool, track: bool) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.set_repeat(context, track)).await
}

/// a jam guest's pause is theirs alone (hold = true); resuming rejoins the
/// jam where it has got to meanwhile
#[tauri::command]
pub async fn jam_hold(app: AppHandle, hold: bool) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.set_jam_hold(hold)).await
}
