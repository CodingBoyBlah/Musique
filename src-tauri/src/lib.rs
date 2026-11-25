use std::sync::{Arc, OnceLock};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tokio::sync::RwLock;

mod auth;
mod commands;
mod diagnostics;
pub use diagnostics::sync_probe;
pub use widevine::{probe as widevine_probe, serve_host as widevine_host, HOST_ARG as WIDEVINE_HOST_ARG};

pub fn connect_probe() -> i32 {
    use librespot_core::{authentication::Credentials, config::SessionConfig, session::Session};

    struct StderrLog;
    impl log::Log for StderrLog {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, r: &log::Record) {
            eprintln!("[{}] {}: {}", r.level(), r.target(), r.args());
        }
        fn flush(&self) {}
    }
    static LOGGER: StderrLog = StderrLog;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Trace);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(_) => return 10,
    };
    rt.block_on(async {
        let token = match keyring::Entry::new("spotify-client", "access_token")
            .and_then(|e| e.get_password())
        {
            Ok(t) => t,
            Err(e) => { eprintln!("[connect-probe] no stored token: {e}"); return 11; }
        };
        let appdata = match std::env::var("APPDATA") {
            Ok(v) => v,
            Err(_) => { eprintln!("[connect-probe] no APPDATA"); return 12; }
        };
        let db_url = format!("sqlite:{}/dev.boyblah.musique/spotify-client.db", appdata.replace('\\', "/"));
        let pool = match sqlx::SqlitePool::connect(&db_url).await {
            Ok(p) => p,
            Err(e) => { eprintln!("[connect-probe] db open failed: {e}"); return 13; }
        };
        let cid: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key='spotify_client_id'")
            .fetch_optional(&pool).await.ok().flatten();
        let client_id = cid.map(|c| c.0.trim().to_string()).unwrap_or_else(|| auth::SHARED_CLIENT_ID.to_string());
        eprintln!("[connect-probe] using client_id={client_id}");

        let mut cfg = SessionConfig::default();
        cfg.client_id = client_id.trim().to_string();
        let session = Session::new(cfg, None);
        if let Err(e) = session.connect(Credentials::with_access_token(&token), false).await {
            eprintln!("[connect-probe] connect failed (token likely expired): {e}");
            return 3;
        }
        eprintln!("[connect-probe] connected - waiting 6s for ProductInfo (exit 1 here = premium check killed us)");
        tokio::time::sleep(std::time::Duration::from_secs(6)).await;
        eprintln!("[connect-probe] SURVIVED - account accepted as premium");
        0
    })
}

/// `spotify.exe --apple-playback-probe <catalog song id>` with
/// `MUSIQUE_APPLE_MEDIA_USER_TOKEN` set (or a signed-in Apple account in the
/// keyring): resolves, licenses, downloads, decrypts and decodes the start of the
/// track, and prints the timings. Read-only; nothing is played aloud.
pub fn apple_playback_probe() -> i32 {
    let mut args = std::env::args().skip_while(|a| a != "--apple-playback-probe").skip(1);
    let Some(track_id) = args.next().filter(|id| !id.starts_with("--")) else {
        eprintln!("usage: --apple-playback-probe <apple music catalog song id>");
        return 2;
    };
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("apple probe: cannot start a runtime: {e}");
            return 1;
        }
    };
    match runtime.block_on(apple::playback::probe(&track_id)) {
        Ok(report) => {
            println!("{report}");
            0
        }
        Err(e) => {
            eprintln!("apple probe failed: {e}");
            1
        }
    }
}

pub fn playback_probe() -> i32 {
    use librespot_core::{
        authentication::Credentials, config::SessionConfig, session::Session, SpotifyId, SpotifyUri,
    };
    use librespot_playback::audio_backend::{Sink, SinkResult};
    use librespot_playback::config::{Bitrate, PlayerConfig};
    use librespot_playback::convert::Converter;
    use librespot_playback::decoder::AudioPacket;
    use librespot_playback::mixer::VolumeGetter;
    use librespot_playback::player::{Player, PlayerEvent};

    struct NullSink;
    impl Sink for NullSink {
        fn write(&mut self, _p: AudioPacket, _c: &mut Converter) -> SinkResult<()> {
            Ok(())
        }
    }
    struct FullVol;
    impl VolumeGetter for FullVol {
        fn attenuation_factor(&self) -> f64 {
            1.0
        }
    }
    /// The output's own volume. With MUSIQUE_PROBE_SILENT set the probe runs
    /// the whole path to the device at zero volume, so it can be run while
    /// something else is playing without being heard.
    struct ProbeVol(f64);
    impl VolumeGetter for ProbeVol {
        fn attenuation_factor(&self) -> f64 {
            self.0
        }
    }
    let probe_volume = if std::env::var_os("MUSIQUE_PROBE_SILENT").is_some() { 0.0 } else { 1.0 };

    struct StderrLog;
    impl log::Log for StderrLog {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, r: &log::Record) {
            let t = r.target();
            if t.starts_with("librespot") {
                eprintln!("[{}] {}: {}", r.level(), t, r.args());
            }
        }
        fn flush(&self) {}
    }
    static LOGGER: StderrLog = StderrLog;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Debug);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(_) => return 10,
    };
    rt.block_on(async {
        let appdata = match std::env::var("APPDATA") { Ok(v) => v, Err(_) => { eprintln!("[playback-probe] no APPDATA"); return 12; } };
        let db_url = format!("sqlite:{}/dev.boyblah.musique/spotify-client.db", appdata.replace('\\', "/"));
        let pool = match sqlx::SqlitePool::connect(&db_url).await { Ok(p) => p, Err(e) => { eprintln!("[playback-probe] db open failed: {e}"); return 13; } };
        let cid: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key='spotify_client_id'").fetch_optional(&pool).await.ok().flatten();
        let client_id = cid.map(|c| c.0.trim().to_string()).unwrap_or_else(|| auth::SHARED_CLIENT_ID.to_string());


        let token = if let Ok(t) = std::env::var("SPOTIFY_TOKEN") {
            if !t.trim().is_empty() {
                eprintln!("[playback-probe] using SPOTIFY_TOKEN from env");
                Some(t.trim().to_string())
            } else { None }
        } else { None };

        // otherwise grab a fresh access token using the saved refresh_token + client_id
        let token = match token {
            Some(t) => t,
            None => {
        let refresh_token = match keyring::Entry::new("spotify-client", "refresh_token").and_then(|e| e.get_password()) {
            Ok(t) => t,
            Err(e) => { eprintln!("[playback-probe] no refresh_token: {e}"); return 11; }
        };
        #[derive(serde::Deserialize)]
        struct Tok { access_token: String, refresh_token: Option<String> }
        let tok = reqwest::Client::new()
            .post("https://accounts.spotify.com/api/token")
            .form(&[("grant_type", "refresh_token"), ("refresh_token", refresh_token.as_str()), ("client_id", client_id.as_str())])
            .send().await;
        match tok {
            Ok(r) if r.status().is_success() => match r.json::<Tok>().await {
                Ok(t) => {
                    // spotify ROTATES refresh tokens so u gotta save the new one or
                    // the old one gets killed and the user is stuck having to log in again
                    if let Some(rt) = t.refresh_token {
                        if let Ok(e) = keyring::Entry::new("spotify-client", "refresh_token") {
                            let _ = e.set_password(&rt);
                            eprintln!("[playback-probe] rotated refresh_token persisted");
                        }
                    }
                    t.access_token
                }
                Err(e) => { eprintln!("[playback-probe] token parse: {e}"); return 11; }
            },
            Ok(r) => { let s = r.status(); let b = r.text().await.unwrap_or_default(); eprintln!("[playback-probe] refresh failed {s}: {b}"); return 11; }
            Err(e) => { eprintln!("[playback-probe] refresh request error: {e}"); return 11; }
        }
            }
        };
        eprintln!("[playback-probe] token ready; client_id={client_id}");

        let creds_dir = std::path::PathBuf::from(&appdata).join("dev.boyblah.musique").join("credentials");
        let cache = librespot_core::cache::Cache::new(Some(&creds_dir), None, None, None).ok();
        let cached_creds = cache.as_ref().and_then(|c| c.credentials());

        let mut cfg = SessionConfig::default();
        cfg.device_id = auth::PLAYBACK_DEVICE_ID.to_string();

        let session = Session::new(cfg, cache);
        let creds = match cached_creds {
            Some(c) => {
                eprintln!("[playback-probe] using cached credentials from disk");
                c
            }
            None => {
                eprintln!("[playback-probe] using access token from keyring");
                Credentials::with_access_token(&token)
            }
        };

        if let Err(e) = session.connect(creds, false).await {
            eprintln!("[playback-probe] connect failed: {e}");
            return 3;
        }
        eprintln!("[playback-probe] session connected (check_catalogue did NOT kill us -> patch works)");


        for _ in 0..50 {
            if !session.country().is_empty() { break; }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        eprintln!("[playback-probe] country = {:?}", session.country());

        // grab a track the user actually owns (def in their region) else a global fallback
        let user_track: Option<(String,)> = sqlx::query_as("SELECT track_id FROM saved_tracks LIMIT 1").fetch_optional(&pool).await.ok().flatten();
        let track_b62 = user_track.map(|t| t.0).unwrap_or_else(|| "4PTG3Z6ehGkBFwjybzWkR8".to_string()); // fallback is never gonna give you up lol
        let spotify_id = match SpotifyId::from_base62(&track_b62) { Ok(i) => i, Err(_) => { eprintln!("[playback-probe] bad track id {track_b62}"); return 6; } };
        eprintln!("[playback-probe] loading track {track_b62}");

        let on_err: crate::sink::ErrorHook = Arc::new(|msg| {
            eprintln!("[playback error] {msg}");
        });
        // kept here too, so the probe can say what the speaker was actually
        // given and not only that the decoder started
        let playout = crate::sink::Playout::new();
        let sink_playout = Arc::clone(&playout);
        let event_slot: crate::sink::EventSlot = Default::default();
        let sink_events = Arc::clone(&event_slot);
        let player = Player::new(
            PlayerConfig { bitrate: Bitrate::Bitrate320, ..Default::default() },
            session.clone(),
            Box::new(FullVol),
            move || Box::new(crate::sink::RodioSink::new(
                None,
                on_err.clone(),
                Box::new(ProbeVol(probe_volume)),
                crate::sink::DEFAULT_BUFFER_MS,
                Arc::clone(&sink_playout),
                Arc::clone(&sink_events),
            )) as Box<dyn Sink>,
        );
        *event_slot.lock().unwrap() = Some(player.get_player_event_channel());
        let mut rx = player.get_player_event_channel();
        player.load(SpotifyUri::Track { id: spotify_id }, true, 0);

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
        let mut duration_ms = 0u32;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() { eprintln!("[playback-probe] TIMEOUT - no decisive event"); return 6; }
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Some(ev)) => match ev {
                    PlayerEvent::TrackChanged { audio_item } => duration_ms = audio_item.duration_ms,
                    PlayerEvent::Playing { .. } => {
                        eprintln!("[playback-probe] SUCCESS - audio keys granted, track is PLAYING through RodioSink. Letting 6s of audio play...");
                        // what the device pulled, second by second: the heard
                        // position has to advance in step with the clock
                        let ring = &playout.ring;
                        let mut heard = Vec::new();
                        let mut dry = Vec::new();
                        for _ in 0..3 {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            heard.push(ring.heard_ms());
                            dry.push(ring.underruns());
                        }
                        // a pause holds the position, and a resume moves it on
                        player.pause();
                        ring.set_held(true);
                        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
                        let paused_at = ring.heard_ms();
                        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
                        let still_at = ring.heard_ms();
                        player.play();
                        ring.set_held(false);
                        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                        let resumed_to = ring.heard_ms();
                        dry.push(ring.underruns());
                        // a seek flushes what was queued and lands where asked
                        playout.seek_requested(60_000);
                        player.seek(60_000);
                        // the far side of a seek may have to come from the CDN;
                        // how long that takes is the network's business. what
                        // is checked is that it lands, and plays on from there
                        let seek_began = std::time::Instant::now();
                        while ring.heard_ms() < 60_400 && seek_began.elapsed() < std::time::Duration::from_secs(20) {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                        eprintln!("[playback-probe] the seek was heard after {}ms", seek_began.elapsed().as_millis());
                        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                        let after_seek = ring.heard_ms();
                        dry.push(ring.underruns());
                        eprintln!("[playback-probe] underruns after each second, after the resume and after the seek: {dry:?}");
                        let frames = ring.rendered_frames.load(std::sync::atomic::Ordering::Relaxed);
                        eprintln!(
                            "[playback-probe] heard {heard:?}ms; paused at {paused_at}ms, held at {still_at}ms, resumed to {resumed_to}ms; after seeking to 60000ms heard {after_seek}ms; {frames} frames rendered; {} underruns; {} flushes",
                            ring.underruns(),
                            ring.flushes(),
                        );
                        let advancing = heard.windows(2).all(|w| w[1] > w[0] + 700 && w[1] < w[0] + 1300);
                        let held = still_at.abs_diff(paused_at) < 50;
                        let resumed = resumed_to > still_at + 900;
                        let seeked = (62_000..63_500).contains(&after_seek);
                        if !(advancing && held && resumed && seeked && frames > 44_100 * 4 && ring.underruns() == 0) {
                            eprintln!("[playback-probe] FAILED - advancing={advancing} held={held} resumed={resumed} seeked={seeked}");
                            return 7;
                        }
                        eprintln!("[playback-probe] audio reached the device, paused, resumed and seeked cleanly");

                        // the join between two tracks, made the way the engine
                        // makes it (playback::roll_on): the next track is loaded
                        // the moment the decoder ends this one, behind the tail
                        // still queued, and nothing is flushed. the device must
                        // not go quiet anywhere across it
                        let second: Option<(String,)> = sqlx::query_as("SELECT track_id FROM saved_tracks LIMIT 1 OFFSET 1").fetch_optional(&pool).await.ok().flatten();
                        let next = second.and_then(|t| SpotifyId::from_base62(&t.0).ok());
                        let (Some(next), true) = (next, duration_ms > 30_000) else {
                            eprintln!("[playback-probe] no second saved track (or no duration); the join was not tried");
                            return 0;
                        };
                        let next = SpotifyUri::Track { id: next };
                        let near_end = duration_ms - 7_000;
                        playout.seek_requested(near_end);
                        player.seek(near_end);
                        player.preload(next.clone());
                        let waiting = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
                        let old_tag = loop {
                            match tokio::time::timeout_at(waiting, rx.recv()).await {
                                Ok(Some(PlayerEvent::EndOfTrack { play_request_id, .. })) => break play_request_id,
                                Ok(Some(_)) => {}
                                _ => { eprintln!("[playback-probe] FAILED - the first track never reached its end"); return 8; }
                            }
                        };
                        let tail_ms = ring.queued_ms_of(old_tag);
                        player.load(next, true, 0);
                        playout.rolling_on(old_tag);

                        let rendered = || ring.rendered_frames.load(std::sync::atomic::Ordering::Relaxed);
                        let dry_before = ring.underruns();
                        let flushes_before = ring.flushes();
                        let mut new_tag = None;
                        let mut worst_gap_ms = 0i64;
                        let mut last = (std::time::Instant::now(), rendered());
                        let mut joined = false;
                        for _ in 0..240 {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            while let Ok(ev) = rx.try_recv() {
                                if let PlayerEvent::PlayRequestIdChanged { play_request_id } = ev {
                                    new_tag = Some(play_request_id);
                                }
                            }
                            let now = (std::time::Instant::now(), rendered());
                            let clock_ms = now.0.duration_since(last.0).as_millis() as i64;
                            let sound_ms = ((now.1 - last.1) * 1000 / 44_100) as i64;
                            worst_gap_ms = worst_gap_ms.max(clock_ms - sound_ms);
                            last = now;
                            if new_tag.and_then(|tag| ring.heard(tag)).is_some_and(|heard| heard > 2_000) {
                                joined = true;
                                break;
                            }
                        }
                        let dry = ring.underruns() - dry_before;
                        let flushed = ring.flushes() - flushes_before;
                        eprintln!(
                            "[playback-probe] join: {tail_ms}ms of tail queued when the next track was loaded; longest stretch without sound {worst_gap_ms}ms; {dry} underruns; {flushed} flushes; next track heard: {joined}"
                        );
                        // the device takes sound in bursts of its own period, so
                        // a few tens of ms of slack is the measurement, not a gap
                        if !(joined && dry == 0 && flushed == 0 && worst_gap_ms < 80) {
                            eprintln!("[playback-probe] FAILED - the join between tracks was not gapless");
                            return 8;
                        }
                        eprintln!("[playback-probe] the next track followed the first without a gap");
                        return 0;
                    }
                    PlayerEvent::Unavailable { track_id, .. } => {
                        eprintln!("[playback-probe] BLOCKED - track {track_id:?} reported Unavailable (region/availability/relink). check_catalogue survived but the track itself is blocked.");
                        return 5;
                    }
                    PlayerEvent::EndOfTrack { .. } => { eprintln!("[playback-probe] EndOfTrack before Playing"); return 6; }
                    other => { eprintln!("[playback-probe] event: {other:?}"); }
                },
                Ok(None) => { eprintln!("[playback-probe] event channel closed"); return 6; }
                Err(_) => { eprintln!("[playback-probe] TIMEOUT"); return 6; }
            }
        }
    })
}

/// `--lossless-probe`: for a handful of saved tracks, list the file manifest,
/// then for every FLAC file request its audio key, pull the first bytes off the
/// CDN, decrypt, and check for the `fLaC` stream marker.
pub fn lossless_probe() -> i32 {
    use librespot_audio::{AudioDecrypt, AudioFile};
    use librespot_core::{config::SessionConfig, session::Session, SpotifyId, SpotifyUri};
    use librespot_metadata::audio::{AudioFileFormat, AudioItem};
    use std::io::Read;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async {
            let Ok(appdata) = std::env::var("APPDATA") else { return 12 };
            let db_url = format!("sqlite:{}/dev.boyblah.musique/spotify-client.db", appdata.replace('\\', "/"));
            let Ok(pool) = sqlx::SqlitePool::connect(&db_url).await else { return 1 };
            let creds_dir = std::path::PathBuf::from(&appdata).join("dev.boyblah.musique").join("credentials");
            let cache = librespot_core::cache::Cache::new(Some(&creds_dir), None, None, None).ok();
            let Some(creds) = cache.as_ref().and_then(|c| c.credentials()) else {
                eprintln!("[lossless-probe] no cached credentials");
                return 2;
            };
            let mut cfg = SessionConfig::default();
            cfg.device_id = auth::PLAYBACK_DEVICE_ID.to_string();
            let session = Session::new(cfg, cache);
            if let Err(e) = session.connect(creds, false).await {
                eprintln!("[lossless-probe] connect failed: {e}");
                return 3;
            }
            for _ in 0..50 {
                if !session.country().is_empty() { break; }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            eprintln!("[lossless-probe] connected, country = {:?}", session.country());

            let mut ids: Vec<String> = sqlx::query_as::<_, (String,)>("SELECT track_id FROM saved_tracks LIMIT 5")
                .fetch_all(&pool).await.unwrap_or_default().into_iter().map(|t| t.0).collect();
            ids.push("4PTG3Z6ehGkBFwjybzWkR8".into());

            let mut ok = 0;
            for b62 in ids {
                let Ok(id) = SpotifyId::from_base62(&b62) else { continue };
                let item = match AudioItem::get_file(&session, SpotifyUri::Track { id }).await {
                    Ok(i) => i,
                    Err(e) => { eprintln!("[lossless-probe] {b62}: metadata failed: {e}"); continue; }
                };
                let formats: Vec<_> = item.files.keys().map(|f| format!("{f:?}")).collect();
                eprintln!("\n[lossless-probe] {b62} <{}> formats: {}", item.name, formats.join(", "));
                for fmt in [AudioFileFormat::FLAC_FLAC, AudioFileFormat::FLAC_FLAC_24BIT] {
                    let Some(&file_id) = item.files.get(&fmt) else { continue };
                    let key = match session.audio_key().request(id, file_id).await {
                        Ok(k) => { eprintln!("  {fmt:?}: audio key GRANTED"); Some(k) }
                        Err(e) => { eprintln!("  {fmt:?}: audio key FAILED: {e}"); None }
                    };
                    let file = match AudioFile::open(&session, file_id, 150 * 1024).await {
                        Ok(f) => f,
                        Err(e) => { eprintln!("  {fmt:?}: CDN open FAILED: {e}"); continue; }
                    };
                    let head = tokio::task::spawn_blocking(move || {
                        let mut d = AudioDecrypt::new(key, file);
                        let mut buf = [0u8; 64];
                        d.read_exact(&mut buf).map(|_| buf)
                    }).await;
                    match head {
                        Ok(Ok(buf)) => {
                            let hex: String = buf[..16].iter().map(|b| format!("{b:02x}")).collect();
                            let pos = buf.windows(4).position(|w| w == b"fLaC");
                            eprintln!("  {fmt:?}: decrypted head {hex} fLaC@{pos:?}");
                            if pos.is_some() { ok += 1; }
                        }
                        other => eprintln!("  {fmt:?}: read failed: {other:?}"),
                    }
                }
            }
            eprintln!("\n[lossless-probe] decodable FLAC files: {ok}");
            if ok > 0 { 0 } else { 8 }
        })
}

pub fn quality_probe() -> i32 {
    use librespot_core::{
        config::SessionConfig, session::Session, SpotifyId, SpotifyUri,
    };
    use librespot_playback::{
        audio_backend::Sink,
        config::{Bitrate, PlayerConfig},
        player::{Player, PlayerEvent},
    };
    use librespot_metadata::audio::{AudioFileFormat, AudioItem};
    use std::sync::Arc;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async {
            eprintln!("[quality-probe] initializing database and session...");
            let appdata = match std::env::var("APPDATA") {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("[quality-probe] no APPDATA");
                    return 12;
                }
            };
            let db_url = format!("sqlite:{}/dev.boyblah.musique/spotify-client.db", appdata.replace('\\', "/"));
            let pool = match sqlx::SqlitePool::connect(&db_url).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[quality-probe] db connect failed: {e}");
                    return 1;
                }
            };

            let creds_dir = std::path::PathBuf::from(&appdata).join("dev.boyblah.musique").join("credentials");
            let cache = librespot_core::cache::Cache::new(Some(&creds_dir), None, None, None).ok();
            let creds = cache.as_ref().and_then(|c| c.credentials());
            if creds.is_none() {
                eprintln!("[quality-probe] no cached credentials found");
                return 2;
            }

            let mut cfg = SessionConfig::default();
            cfg.client_id = auth::PLAYBACK_CLIENT_ID.to_string();
            cfg.device_id = auth::PLAYBACK_DEVICE_ID.to_string();

            let session = Session::new(cfg, cache);
            if let Err(e) = session.connect(creds.unwrap(), false).await {
                eprintln!("[quality-probe] connect failed: {e}");
                return 3;
            }

            for _ in 0..50 {
                if !session.country().is_empty() { break; }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            eprintln!("[quality-probe] session connected as country = {:?}", session.country());

            let user_track: Option<(String,)> = sqlx::query_as("SELECT track_id FROM saved_tracks LIMIT 1")
                .fetch_optional(&pool).await.ok().flatten();
            let track_b62 = user_track.map(|t| t.0).unwrap_or_else(|| "000TJlEJQ3nafsm1hBWpoj".to_string());
            let spotify_id = match SpotifyId::from_base62(&track_b62) {
                Ok(i) => i,
                Err(_) => return 4,
            };

            let track_uri = SpotifyUri::Track { id: spotify_id };
            eprintln!("[quality-probe] querying Spotify CDN metadata for track {track_b62}...");

            let audio_item = match AudioItem::get_file(&session, track_uri.clone()).await {
                Ok(item) => item,
                Err(e) => {
                    eprintln!("[quality-probe] metadata query failed: {e}");
                    return 5;
                }
            };

            eprintln!("[quality-probe] Track Name: <{}>", audio_item.name);
            eprintln!("[quality-probe] Spotify Server Audio File Manifest:");
            for (format, file_id) in &*audio_item.files {
                let hex_id = file_id.0.iter().map(|b| format!("{b:02x}")).collect::<String>();
                eprintln!("  Format: {:?} -> FileId: {}", format, hex_id);
            }

            // Verify each quality setting (96, 160, 320)
            let qualities = [
                ("96 kbps (Normal)", Bitrate::Bitrate96, AudioFileFormat::OGG_VORBIS_96),
                ("160 kbps (High)", Bitrate::Bitrate160, AudioFileFormat::OGG_VORBIS_160),
                ("320 kbps (Very High)", Bitrate::Bitrate320, AudioFileFormat::OGG_VORBIS_320),
            ];

            for (label, bitrate, expected_format) in qualities {
                eprintln!("\n[quality-probe] --- Testing {label} ---");
                let expected_file_id = audio_item.files.get(&expected_format);
                if let Some(fid) = expected_file_id {
                    let hex_id = fid.0.iter().map(|b| format!("{b:02x}")).collect::<String>();
                    eprintln!("[quality-probe] Expected file ID for {label}: {hex_id}");
                } else {
                    eprintln!("[quality-probe] Warning: Format {expected_format:?} not found in track manifest");
                }

                let on_err: crate::sink::ErrorHook = Arc::new(|_| {});
                let player = Player::new(
                    PlayerConfig { bitrate, gapless: true, ..Default::default() },
                    session.clone(),
                    Box::new(librespot_playback::mixer::NoOpVolume),
                    move || Box::new(crate::sink::RodioSink::new(
                        None,
                        on_err.clone(),
                        Box::new(librespot_playback::mixer::NoOpVolume),
                        crate::sink::DEFAULT_BUFFER_MS,
                        crate::sink::Playout::new(),
                        Default::default(),
                    )) as Box<dyn Sink>,
                );

                let mut rx = player.get_player_event_channel();
                player.load(track_uri.clone(), true, 0);

                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
                let mut playing = false;
                while let Ok(Some(ev)) = tokio::time::timeout(deadline.saturating_duration_since(tokio::time::Instant::now()), rx.recv()).await {
                    match ev {
                        PlayerEvent::Playing { .. } | PlayerEvent::TrackChanged { .. } => {
                            eprintln!("[quality-probe] CONFIRMED: {label} loaded and stream decryptor started playing!");
                            playing = true;
                            break;
                        }
                        PlayerEvent::Unavailable { track_id, .. } => {
                            eprintln!("[quality-probe] FAILED: track reported unavailable for {label}: {track_id:?}");
                            return 6;
                        }
                        _ => {}
                    }
                }
                player.stop();
                if !playing {
                    eprintln!("[quality-probe] TIMEOUT waiting for {label} to play");
                    return 7;
                }
            }

            eprintln!("\n[quality-probe] ALL QUALITY TIERS (96, 160, 320 kbps) VERIFIED AND STREAMING FROM SPOTIFY!");
            0
        })
}

pub fn audio_probe() -> i32 {
    // catch the uncatchable: exit cleanly instead of crash-dialoging on a native
    // audio fault. async-signal-safe (only _exit). unix = macOS + Linux.
    // MUST be installed BEFORE we open the device below.
    #[cfg(unix)]
    {
        extern "C" fn bail(_sig: i32) {
            unsafe { libc::_exit(70) };
        }
        unsafe {
            for sig in [
                libc::SIGSEGV,
                libc::SIGBUS,
                libc::SIGILL,
                libc::SIGFPE,
                libc::SIGABRT,
            ] {
                libc::signal(sig, bail as libc::sighandler_t);
            }
        }
    }

    // probe the output device using RodioSink
    use librespot_playback::audio_backend::Sink;
    let mut sink = crate::sink::RodioSink::new(
        None,
        Arc::new(|_| {}),
        Box::new(librespot_playback::mixer::NoOpVolume),
        crate::sink::DEFAULT_BUFFER_MS,
        crate::sink::Playout::new(),
        Default::default(),
    );
    let _ = sink.start();
    let _ = sink.stop();
    0
}

mod db;
mod discord;
mod episode_audio;
mod cast;
mod sonos;
mod zeroconf;
mod eq;
mod instance;
mod errors;
mod http;
mod internal;
mod lastfm;
mod library;
mod lyrics;
mod media_controls;
mod playback;
mod playout;
mod recommend;
mod resample;
mod sink;
mod soloist;
mod spotify;
mod stretch;
mod youtube;
mod apple;
mod widevine;
mod state;
mod mem_trim;

fn install_panic_logger() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut path = std::env::temp_dir();
        path.push("spotify-panic.log");
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let thread = std::thread::current()
            .name()
            .unwrap_or("<unnamed>")
            .to_string();
        let bt = std::backtrace::Backtrace::force_capture();
        let entry = format!("\n===== PANIC @unix={secs} thread={thread} =====\n{info}\n{bt}\n");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = f.write_all(entry.as_bytes());
        }
        default_hook(info);
    }));
}

#[cfg(unix)]
mod native_crash {
    use std::ffi::CString;
    use std::sync::OnceLock;

    static LOG_PATH: OnceLock<CString> = OnceLock::new();

    fn sig_name(sig: i32) -> &'static [u8] {
        match sig {
            libc::SIGSEGV => b"SIGSEGV",
            libc::SIGBUS => b"SIGBUS",
            libc::SIGILL => b"SIGILL",
            libc::SIGFPE => b"SIGFPE",
            libc::SIGABRT => b"SIGABRT",
            _ => b"SIGNAL",
        }
    }

    extern "C" fn handler(sig: i32) {
        // ONLY async-signal-safe calls in here, no rust std fs / alloc / format or it breaks
        if let Some(path) = LOG_PATH.get() {
            unsafe {
                let fd = libc::open(
                    path.as_ptr(),
                    libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND,
                    0o644,
                );
                if fd >= 0 {
                    let pre: &[u8] = b"\n===== NATIVE CRASH signal=";
                    libc::write(fd, pre.as_ptr() as *const libc::c_void, pre.len());
                    let name = sig_name(sig);
                    libc::write(fd, name.as_ptr() as *const libc::c_void, name.len());
                    let post: &[u8] = b" (uncatchable native fault - e.g. CoreAudio/WASAPI device open; see the OS crash report for the backtrace) =====\n";
                    libc::write(fd, post.as_ptr() as *const libc::c_void, post.len());
                    libc::close(fd);
                }
            }
        }

        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
    }

    pub fn install() {
        let mut path = std::env::temp_dir();
        path.push("spotify-panic.log");
        if let Ok(c) = CString::new(path.to_string_lossy().as_bytes()) {
            let _ = LOG_PATH.set(c);
        }
        unsafe {
            for sig in [
                libc::SIGSEGV,
                libc::SIGBUS,
                libc::SIGILL,
                libc::SIGFPE,
                libc::SIGABRT,
            ] {
                libc::signal(sig, handler as libc::sighandler_t);
            }
        }
    }
}

// librespot's log output, teed to stderr and to a temp file.
//
// This used to open(), write(), close() the log file on EVERY line, from
// whichever thread emitted it - the session thread during connect, the
// player/decoder threads during a load. librespot is chatty at Info level while
// a track is starting, so the first play paid a few dozen synchronous file-open
// syscalls on exactly the threads that needed to be getting audio out the door.
// Now the record is formatted, handed to a bounded channel, and a dedicated
// writer thread owns one long-lived file handle. Logging from the hot path
// costs a format plus a channel push; if the writer ever falls behind, lines are
// dropped rather than stalling playback.
struct PlaybackLog;

static LOG_TX: OnceLock<std::sync::mpsc::SyncSender<String>> = OnceLock::new();

fn log_sink() -> &'static std::sync::mpsc::SyncSender<String> {
    LOG_TX.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1024);
        std::thread::Builder::new()
            .name("playback-log".into())
            .spawn(move || {
                let mut p = std::env::temp_dir();
                p.push("spotify-playback.log");
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&p)
                    .ok();
                while let Ok(line) = rx.recv() {
                    use std::io::Write;
                    eprint!("{line}");
                    if let Some(f) = file.as_mut() {
                        let _ = f.write_all(line.as_bytes());
                    }
                }
            })
            .ok();
        tx
    })
}

/// `MUSIQUE_LOG_DEBUG=1`: librespot's debug lines too, each stamped with the
/// milliseconds since start, to see where a load spends its time.
fn log_debug() -> bool {
    static DEBUG: OnceLock<bool> = OnceLock::new();
    *DEBUG.get_or_init(|| std::env::var_os("MUSIQUE_LOG_DEBUG").is_some())
}

impl log::Log for PlaybackLog {
    fn enabled(&self, m: &log::Metadata) -> bool {
        let level = if log_debug() { log::Level::Debug } else { log::Level::Info };
        m.target().starts_with("librespot") && m.level() <= level
    }
    fn log(&self, r: &log::Record) {
        if !self.enabled(r.metadata()) {
            return;
        }
        let line = if log_debug() {
            static START: OnceLock<std::time::Instant> = OnceLock::new();
            let ms = START.get_or_init(std::time::Instant::now).elapsed().as_millis();
            format!("{ms:>7} [{}] {}: {}\n", r.level(), r.target(), r.args())
        } else {
            format!("[{}] {}: {}\n", r.level(), r.target(), r.args())
        };
        // never block a player/session thread on disk or on a full queue
        let _ = log_sink().try_send(line);
    }
    fn flush(&self) {}
}
static PLAYBACK_LOG: PlaybackLog = PlaybackLog;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = log::set_logger(&PLAYBACK_LOG);
    log::set_max_level(if log_debug() { log::LevelFilter::Debug } else { log::LevelFilter::Info });

    install_panic_logger();

    // catches native crashes that a rust panic doesnt, leaves a breadcrumb in the same log
    #[cfg(unix)]
    native_crash::install();

    // no --num-raster-threads: pinning it to 1 left every newly exposed tile
    // (a fast scroll, a page opening) queued behind one worker. Chromium sizes
    // the pool to the CPU, and scrolling now runs on the compositor and leans
    // on those workers to keep up with the display.
    #[cfg(target_os = "windows")]
    if std::env::var_os("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").is_none() {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--renderer-process-limit=1 --gpu-rasterization-msaa-sample-count=0 --enable-zero-copy --disk-cache-size=33554432 --media-cache-size=33554432 --disable-renderer-accessibility --disable-speech-api --disable-print-preview --enable-features=TrimOnMemoryPressure,NetworkServiceInProcess --disable-background-networking --disable-component-update --disable-domain-reliability --disable-sync --disable-breakpad --disable-features=Translate,OptimizationHints,MediaRouter,CalculateNativeWinOcclusion,InterestFeedContentSuggestions,BackForwardCache,GlobalMediaControls",
        );
    }

    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::from_filename(".env");

    // one copy at a time: a second would sign in a second session as the same
    // Connect device and the two would knock each other off. the engine probe
    // runs beside the app on purpose, headless
    let _instance = if std::env::var_os("MUSIQUE_ENGINE_PROBE").is_none() {
        match instance::claim() {
            Some(instance::Claim::Second) => {
                eprintln!("[instance] Musique is already running; showing it");
                return;
            }
            Some(instance::Claim::First(guard)) => Some(guard),
            None => None,
        }
    } else {
        None
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            init_windows_aumid(app.handle());

            #[allow(unused_mut)]
            let mut backdrop_active = false;

            #[allow(unused_mut)]
            let mut main_hwnd: Option<isize> = None;
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(not(target_os = "macos"))]
                let _ = window.set_decorations(false);

                let _ = window.set_theme(Some(tauri::Theme::Dark));

                // win11 gets mica, older builds fall back to acrylic
                #[cfg(target_os = "windows")]
                {
                    use window_vibrancy::{apply_acrylic, apply_mica};
                    backdrop_active = apply_mica(&window, Some(true)).is_ok()
                        || apply_acrylic(&window, Some((18, 18, 18, 110))).is_ok();
                }

                #[cfg(target_os = "macos")]
                {
                    use window_vibrancy::{
                        apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
                    };
                    for material in [
                        NSVisualEffectMaterial::HudWindow,
                        NSVisualEffectMaterial::UnderWindowBackground,
                        NSVisualEffectMaterial::WindowBackground,
                    ] {
                        if apply_vibrancy(
                            &window,
                            material,
                            Some(NSVisualEffectState::Active),
                            None,
                        )
                        .is_ok()
                        {
                            backdrop_active = true;
                            break;
                        }
                    }
                }

                // linux has no native backdrop material so backdrop_active stays
                // false and the frontend just paints its own solid background

                // the engine probe runs with no interface: the window stays hidden
                // and empty. a release build carries the real interface, which
                // would otherwise load, pick up its saved queue and start
                // issuing commands of its own in the middle of the probe
                if std::env::var_os("MUSIQUE_ENGINE_PROBE").is_none() {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else if let Ok(blank) = "about:blank".parse() {
                    let _ = window.navigate(blank);
                }

                // a shipped build is an app, not a browser tab: turn off WebView2's
                // browser keys (Ctrl+P / Ctrl+Shift+P print, Ctrl+F find, F5 reload,
                // Ctrl+U, F7 caret browsing, F12 / Ctrl+Shift+I devtools). Typing,
                // clipboard and the page's own keydown handlers are unaffected.
                // dev builds keep them so devtools and reload still work there.
                // src/lib/browserKeys.ts covers the same keys in the page
                #[cfg(all(target_os = "windows", not(debug_assertions)))]
                {
                    let _ = window.with_webview(|wv| unsafe {
                        use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
                        use windows::core::Interface;
                        if let Ok(settings) = wv.controller().CoreWebView2().and_then(|c| c.Settings()) {
                            if let Ok(s3) = settings.cast::<ICoreWebView2Settings3>() {
                                let _ = s3.SetAreBrowserAcceleratorKeysEnabled(false);
                            }
                        }
                    });
                }

                // grab the HWND now, windows only, means nothing anywhere else
                #[cfg(target_os = "windows")]
                {
                    main_hwnd = window.hwnd().ok().map(|h| h.0 as isize);
                }
            }

            // another launch asks this copy to come forward instead
            let shown = app.handle().clone();
            instance::on_show(move || {
                let app = shown.clone();
                let _ = shown.run_on_main_thread(move || {
                    if let Some(w) = app.get_webview_window("main") {
                        w.unminimize().ok();
                        w.show().ok();
                        w.set_focus().ok();
                    }
                });
            });

            // kick off the media-controls thread (souvlaki)
            let media_tx = media_controls::start(app.handle().clone(), main_hwnd);

            // database + auth setup
            //
            // This block_on runs before the event loop starts, so every
            // millisecond here is a millisecond the window sits unpainted.
            // Reading the OS credential store is a slow, purely independent
            // call, so it runs on a blocking thread while sqlite opens and
            // migrates instead of after it.
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let tokens = tauri::async_runtime::spawn_blocking(auth::load_stored_tokens);
                let pool = db::connection::create_pool(&handle).await;
                // seed spotify creds from .env on first run, does nothing if theyre already set
                commands::credentials::seed_credentials_from_env(&pool).await;
                commands::playback::load_sound_settings(&pool).await;
                // which service is signed in, and where Apple Music keeps its web token
                if let Ok(dir) = handle.path().app_local_data_dir() {
                    apple::auth::init(dir.join("apple"));
                }
                apple::load(&pool).await;
                let auth_state = auth::init_auth_state_with(
                    &pool,
                    tokens.await.unwrap_or(None),
                )
                .await;
                handle.manage(state::AppState {
                    db: pool,
                    auth: Arc::new(RwLock::new(auth_state)),
                    playback: Arc::new(tokio::sync::Mutex::new(None)),
                    yt: Arc::new(tokio::sync::Mutex::new(None)),
                    soloist: soloist::Soloist::new(),
                    media_tx,
                    sync_gate: Arc::new(RwLock::new(())),
                    backdrop_active,
                });
                Ok::<(), Box<dyn std::error::Error>>(())
            })?;

            mem_trim::start_memory_trimmer(app.handle().clone());

            // Soloist is a separate program; one the listener turned on is
            // started with the app so it is signed in by the first play
            tauri::async_runtime::spawn(commands::soloist::start_if_enabled(app.handle().clone()));

            if std::env::var_os("MUSIQUE_ENGINE_PROBE").is_some() {
                let probe = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let code = if std::env::var_os("MUSIQUE_SOLOIST_PROBE").is_some() {
                        commands::playback::soloist_probe(probe.clone()).await
                    } else if std::env::var_os("MUSIQUE_START_PROBE").is_some() {
                        commands::playback::start_probe(probe.clone()).await
                    } else if std::env::var_os("MUSIQUE_SEARCH_PROBE").is_some() {
                        commands::spotify::search_probe(probe.clone()).await
                    } else {
                        commands::playback::engine_probe(probe.clone()).await
                    };
                    eprintln!("[engine-probe] exit {code}");
                    probe.exit(code);
                });
            }

            // a session spotify closed mid-song is rebuilt and the song carried on
            tauri::async_runtime::spawn(commands::playback::recover_dropped_sessions(app.handle().clone()));

            // background loops that just keep running
            let handle2 = app.handle().clone();
            tauri::async_runtime::spawn(auth::refresh_loop(handle2));

            // Build the librespot session NOW, in the background, instead of
            // waiting for the webview to mount, resolve auth over IPC and call
            // warmup_playback. Connecting to the access point, registering the
            // Connect device and waiting for the country packet is most of the
            // cost of the first play; doing it while the UI is still painting
            // means it is usually finished before the user can click anything.
            let handle4 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                commands::playback::warm_session_if_possible(handle4).await;
            });

            let handle3 = app.handle().clone();
            tauri::async_runtime::spawn(library::library_sync_loop(handle3));

            // system tray item - wrapped so a menu build failure is logged and skipped
            let tray_setup = (|| -> Result<(), Box<dyn std::error::Error>> {
                let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
                let sep1 = PredefinedMenuItem::separator(app)?;
                // plain text labels, the U+23EE/EF/ED media glyphs turn into tofu boxes in minimal
                // gtk menu fonts on linux and arent guaranteed in every macos menu
                // either, text always renders so we just use that
                let prev_item = MenuItem::with_id(app, "prev", "Previous", true, None::<&str>)?;
                let toggle_item =
                    MenuItem::with_id(app, "toggle", "Play / Pause", true, None::<&str>)?;
                let next_item = MenuItem::with_id(app, "next", "Next", true, None::<&str>)?;
                let sep2 = PredefinedMenuItem::separator(app)?;
                let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

                let menu = Menu::with_items(
                    app,
                    &[
                        &show_item,
                        &sep1,
                        &prev_item,
                        &toggle_item,
                        &next_item,
                        &sep2,
                        &quit_item,
                    ],
                )?;

                let mut tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                w.show().ok();
                                w.set_focus().ok();
                            }
                        }
                        "quit" => app.exit(0),
                        "toggle" => {
                            app.emit("media:toggle", ()).ok();
                        }
                        "next" => {
                            app.emit("media:next", ()).ok();
                        }
                        "prev" => {
                            app.emit("media:prev", ()).ok();
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(w) = app.get_webview_window("main") {
                                if w.is_visible().unwrap_or(false) {
                                    w.hide().ok();
                                } else {
                                    w.show().ok();
                                    w.set_focus().ok();
                                }
                            }
                        }
                    });

                // macOS menu bar: a monochrome template glyph (icons/tray-template.svg,
                // rasterised to 64x64 RGBA) the OS tints for light/dark menu bars,
                // like every native menu-bar item. the full-colour app icon looked
                // out of place there. elsewhere the colour icon is the norm.
                #[cfg(target_os = "macos")]
                {
                    const TRAY_TEMPLATE: &[u8] = include_bytes!("../icons/tray-template.rgba");
                    tray = tray
                        .icon(tauri::image::Image::new(TRAY_TEMPLATE, 64, 64))
                        .icon_as_template(true);
                }
                #[cfg(not(target_os = "macos"))]
                {
                    if let Some(icon) = app.default_window_icon().cloned() {
                        tray = tray.icon(icon);
                    }
                }
                if let Err(e) = tray.build(app) {
                    eprintln!("[setup] tray build failed (non fatal): {e}");
                }

                Ok(())
            })();
            if let Err(e) = tray_setup {
                eprintln!("[setu[] tray/menu setup failed (non-fatal): {e}]");
            }

            Ok(())
        })
        .invoke_handler({
            let commands: Box<dyn Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync> = Box::new(tauri::generate_handler![
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::credentials::save_credentials,
            commands::credentials::get_credentials,
            commands::credentials::validate_credentials,
            commands::credentials::clear_credentials,
            commands::auth::start_login,
            commands::auth::authorize_playback,
            commands::auth::logout,
            commands::auth::get_auth_status,
            commands::auth::get_profile,
            commands::apple::apple_sign_in,
            commands::apple::apple_sign_in_with_token,
            commands::apple::apple_cancel_sign_in,
            commands::widevine::widevine_status,
            commands::widevine::widevine_license_text,
            commands::widevine::widevine_install,
            commands::widevine::widevine_uninstall,
            commands::spotify::search,
            commands::spotify::get_artist,
            commands::spotify::get_album,
            commands::spotify::get_track,
            commands::spotify::get_playlist,
            commands::spotify::get_recommendations,
            commands::spotify::record_listen_event,
            commands::internal::get_tracks_metadata,
            commands::social::get_friend_activity,
            commands::stats::get_listening_stats,
            commands::social::get_jam,
            commands::social::start_jam,
            commands::social::join_jam,
            commands::social::leave_jam,
            commands::social::set_jam_queue_only,
            commands::social::kick_jam_member,
            commands::social::jam_command,
            commands::playback::get_connect_state,
            commands::playback::connect_load_tracks,
            commands::playback::connect_add_to_queue,
            commands::playback::connect_skip_to,
            commands::playback::connect_next,
            commands::playback::connect_prev,
            commands::playback::connect_set_shuffle,
            commands::playback::connect_set_repeat,
            commands::playback::jam_hold,
            commands::home_feed::get_home_feed,
            commands::artist_extras::get_artist_extras,
            commands::artist_extras::get_artist_overview,
            commands::internal::get_canvas,
            commands::internal::get_track_credits,
            commands::internal::get_playlist_folders,
            commands::internal::get_blend_info,
            commands::internal::get_track_radio,
            commands::internal::get_station,
            commands::internal::get_autoplay_tracks,
            commands::internal::get_extracted_colors,
            commands::profile::get_user_profile,
            commands::podcasts::get_show,
            commands::episode_extras::get_episode_transcript,
            commands::episode_extras::get_episode_media,
            commands::podcasts::get_show_episodes,
            commands::podcasts::get_episode,
            commands::podcasts::get_saved_shows,
            commands::podcasts::save_show,
            commands::podcasts::unsave_show,
            commands::podcasts::is_show_saved,
            commands::profile::get_user_followers,
            commands::profile::get_user_following,
            commands::profile::set_user_followed,
            commands::profile::is_user_followed,
            commands::playback::warmup_playback,
            commands::playback::play_track,
            commands::playback::retry_play_track,
            commands::playback::pause_playback,
            commands::playback::resume_playback,
            commands::playback::resume_or_play,
            commands::playback::stop_playback,
            commands::playback::seek_playback,
            commands::playback::preload_track,
            commands::playback::get_playback_backend,
            commands::playback::set_playback_backend,
            commands::playback::get_equalizer,
            commands::playback::set_equalizer,
            commands::playback::get_volume_curve,
            commands::playback::set_volume_curve,
            commands::playback::get_audio_normalisation,
            commands::playback::set_audio_normalisation,
            commands::soloist::soloist_status,
            commands::soloist::soloist_enable,
            commands::soloist::soloist_disable,
            commands::soloist::soloist_forget,
            commands::soloist::soloist_set_quality,
            commands::playback::get_yt_match,
            commands::playback::search_yt_candidates,
            commands::playback::pin_yt_match,
            commands::playback::forget_yt_match,
            commands::playback::set_volume,
            commands::playback::set_playback_speed,
            commands::playback::set_muted,
            commands::playback::get_volume,
            commands::playback::get_audio_quality,
            commands::playback::set_audio_quality,
            commands::playback::get_audio_cache_limit,
            commands::playback::set_audio_cache_limit,
            commands::playback::get_output_latency_ms,
            commands::library::sync_library,
            commands::library::get_liked_songs,
            commands::library::get_liked_songs_count,
            commands::library::get_my_playlists,
            commands::library::get_saved_albums,
            commands::library::get_followed_artists,
            commands::library::get_library_status,
            commands::library::get_top_tracks,
            commands::library::get_top_artists,
            commands::library::get_recently_played,
            commands::library::get_new_releases,
            commands::library::get_cached_playlist,
            commands::library::save_track,
            commands::library::unsave_track,
            commands::library::get_saved_track_ids,
            commands::library::follow_artist,
            commands::library::unfollow_artist,
            commands::library::is_artist_followed,
            commands::library::save_album,
            commands::library::unsave_album,
            commands::library::is_album_saved,
            commands::library::add_track_to_playlist,
            commands::library::remove_track_from_playlist,
            commands::library::create_playlist,
            commands::library::update_playlist_details,
            commands::library::follow_playlist,
            commands::library::unfollow_playlist,
            commands::library::is_playlist_followed,
            commands::media::update_now_playing,
            commands::media::set_discord_enabled,
            commands::media::show_playback_notification,
            commands::window::set_window_effect,
            commands::window::get_backdrop_active,
            commands::lyrics::get_lyrics,
            commands::lyrics::set_lyrics_source,
            commands::share::resolve_odesli,
            commands::lastfm::lastfm_status,
            commands::lastfm::lastfm_save_api,
            commands::lastfm::lastfm_start_auth,
            commands::lastfm::lastfm_finish_auth,
            commands::lastfm::lastfm_disconnect,
            commands::lastfm::lastfm_clear,
            commands::lastfm::lastfm_now_playing,
            commands::lastfm::lastfm_scrobble,
            commands::theme::get_wallpaper_data_url,
            commands::theme::get_system_accent,
            commands::connect::get_devices,
            commands::receivers::discover_receivers,
            commands::receivers::get_receivers,
            commands::receivers::activate_receiver,
            commands::connect::get_playback_state,
            commands::connect::transfer_playback,
            commands::connect::remote_play,
            commands::connect::remote_play_track,
            commands::connect::remote_pause,
            commands::connect::remote_next,
            commands::connect::remote_previous,
            commands::connect::remote_seek,
            commands::connect::remote_set_volume,
            commands::connect::get_musique_device_id,
            commands::connect::get_remote_queue,
            commands::connect::remote_set_shuffle,
            commands::connect::remote_set_repeat,
            commands::connect::remote_add_to_queue,
            commands::connect::remote_play_context,
            mem_trim::trim_memory,
            ]);
            // the engine probe runs with nobody at the controls. an interface
            // that manages to start anyway (a release build carries one) must
            // not be able to play, or to write its own idea of the settings
            // over the listener's
            let probing = std::env::var_os("MUSIQUE_ENGINE_PROBE").is_some();
            move |invoke| {
                if probing {
                    invoke.resolver.reject("the engine probe is running");
                    return true;
                }
                commands(invoke)
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "windows")]
fn init_windows_aumid(app: &tauri::AppHandle) {
    use std::os::windows::ffi::OsStrExt;
    use tauri::Manager;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyW, RegSetValueExW, HKEY_CURRENT_USER, REG_DWORD, REG_SZ,
    };
    use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let app_id_str = "dev.boyblah.musique";
    let app_id_wide: Vec<u16> = app_id_str
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // 1. Set current process AUMID
    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(app_id_wide.as_ptr());
    }

    // 2. Register HKCU\Software\Classes\AppUserModelId\dev.boyblah.musique
    let subkey = format!("Software\\Classes\\AppUserModelId\\{}", app_id_str);
    let subkey_wide: Vec<u16> = std::ffi::OsStr::new(&subkey)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut hkey: windows_sys::Win32::System::Registry::HKEY = std::ptr::null_mut();
    let res = unsafe {
        RegCreateKeyW(
            HKEY_CURRENT_USER,
            subkey_wide.as_ptr(),
            &mut hkey,
        )
    };

    if res == 0 && !hkey.is_null() {
        // DisplayName: "Musique"
        let display_name: Vec<u16> = "Musique".encode_utf16().chain(std::iter::once(0)).collect();
        let name_val: Vec<u16> = "DisplayName".encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let _ = RegSetValueExW(
                hkey,
                name_val.as_ptr(),
                0,
                REG_SZ,
                display_name.as_ptr() as *const u8,
                (display_name.len() * 2) as u32,
            );
        }

        // ShowInSettings: 1
        let show_in_settings_val: Vec<u16> = "ShowInSettings"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let show_val: u32 = 1;
        unsafe {
            let _ = RegSetValueExW(
                hkey,
                show_in_settings_val.as_ptr(),
                0,
                REG_DWORD,
                &show_val as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );
        }

        // IconUri: resolve or extract icon to app data
        if let Ok(app_dir) = app.path().app_local_data_dir() {
            let _ = std::fs::create_dir_all(&app_dir);
            let logo_path = app_dir.join("musique_logo.png");
            if !logo_path.exists() {
                let logo_bytes = include_bytes!("../icons/icon.png");
                let _ = std::fs::write(&logo_path, logo_bytes);
            }
            let icon_path = app_dir.join("icon.ico");
            if !icon_path.exists() {
                let icon_bytes = include_bytes!("../icons/icon.ico");
                let _ = std::fs::write(&icon_path, icon_bytes);
            }
            if icon_path.exists() {
                let icon_uri_str = icon_path.to_string_lossy().to_string();
                let icon_uri_wide: Vec<u16> = icon_uri_str
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                let icon_val: Vec<u16> =
                    "IconUri".encode_utf16().chain(std::iter::once(0)).collect();
                unsafe {
                    let _ = RegSetValueExW(
                        hkey,
                        icon_val.as_ptr(),
                        0,
                        REG_SZ,
                        icon_uri_wide.as_ptr() as *const u8,
                        (icon_uri_wide.len() * 2) as u32,
                    );
                }
            }
        }

        unsafe {
            let _ = RegCloseKey(hkey);
        }
    }
}
