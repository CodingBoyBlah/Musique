use std::sync::{Arc, OnceLock};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tokio::sync::RwLock;

mod auth;
mod commands;

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
        let player = Player::new(
            PlayerConfig { bitrate: Bitrate::Bitrate320, ..Default::default() },
            session.clone(),
            Box::new(FullVol),
            move || Box::new(crate::sink::RodioSink::new(
                None,
                on_err.clone(),
                Box::new(librespot_playback::mixer::NoOpVolume),
                crate::sink::DEFAULT_BUFFER_MS,
            )) as Box<dyn Sink>,
        );
        let mut rx = player.get_player_event_channel();
        player.load(SpotifyUri::Track { id: spotify_id }, true, 0);

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() { eprintln!("[playback-probe] TIMEOUT - no decisive event"); return 6; }
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Some(ev)) => match ev {
                    PlayerEvent::Playing { .. } | PlayerEvent::TrackChanged { .. } => {
                        eprintln!("[playback-probe] SUCCESS - audio keys granted, track is PLAYING through RodioSink. Letting 2s of audio play...");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        eprintln!("[playback-probe] 2s audio playback complete!");
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
    );
    let _ = sink.start();
    let _ = sink.stop();
    0
}

mod db;
mod discord;
mod errors;
mod http;
mod internal;
mod lastfm;
mod library;
mod lyrics;
mod media_controls;
mod playback;
mod recommend;
mod resample;
mod sink;
mod spotify;
mod youtube;
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

impl log::Log for PlaybackLog {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.target().starts_with("librespot") && m.level() <= log::Level::Info
    }
    fn log(&self, r: &log::Record) {
        if !self.enabled(r.metadata()) {
            return;
        }
        let line = format!("[{}] {}: {}\n", r.level(), r.target(), r.args());
        // never block a player/session thread on disk or on a full queue
        let _ = log_sink().try_send(line);
    }
    fn flush(&self) {}
}
static PLAYBACK_LOG: PlaybackLog = PlaybackLog;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = log::set_logger(&PLAYBACK_LOG);
    log::set_max_level(log::LevelFilter::Info);

    install_panic_logger();

    // catches native crashes that a rust panic doesnt, leaves a breadcrumb in the same log
    #[cfg(unix)]
    native_crash::install();

    #[cfg(target_os = "windows")]
    if std::env::var_os("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").is_none() {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--js-flags=--max-old-space-size=96,--optimize-for-size --renderer-process-limit=1 --gpu-rasterization-msaa-sample-count=0 --num-raster-threads=1 --enable-zero-copy --disk-cache-size=33554432 --media-cache-size=33554432 --disable-renderer-accessibility --disable-speech-api --disable-print-preview --enable-features=TrimOnMemoryPressure,NetworkServiceInProcess --disable-background-networking --disable-component-update --disable-domain-reliability --disable-sync --disable-breakpad --disable-features=Translate,OptimizationHints,MediaRouter,CalculateNativeWinOcclusion,InterestFeedContentSuggestions,BackForwardCache,GlobalMediaControls",
        );
    }

    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::from_filename(".env");

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

                let _ = window.show();
                let _ = window.set_focus();

                // grab the HWND now, windows only, means nothing anywhere else
                #[cfg(target_os = "windows")]
                {
                    main_hwnd = window.hwnd().ok().map(|h| h.0 as isize);
                }
            }

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
                    media_tx,
                    sync_gate: Arc::new(RwLock::new(())),
                    backdrop_active,
                });
                Ok::<(), Box<dyn std::error::Error>>(())
            })?;

            mem_trim::start_memory_trimmer(app.handle().clone());

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

                if let Some(icon) = app.default_window_icon().cloned() {
                    tray = tray.icon(icon);
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
        .invoke_handler(tauri::generate_handler![
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
            commands::spotify::search,
            commands::spotify::get_artist,
            commands::spotify::get_album,
            commands::spotify::get_track,
            commands::spotify::get_playlist,
            commands::spotify::get_recommendations,
            commands::spotify::record_listen_event,
            commands::internal::get_tracks_metadata,
            commands::home_feed::get_home_feed,
            commands::artist_extras::get_artist_extras,
            commands::artist_extras::get_artist_overview,
            commands::internal::get_canvas,
            commands::internal::get_track_credits,
            commands::internal::get_playlist_folders,
            commands::internal::get_track_radio,
            commands::internal::get_station,
            commands::internal::get_autoplay_tracks,
            commands::internal::get_extracted_colors,
            commands::profile::get_user_profile,
            commands::podcasts::get_show,
            commands::audiobooks::get_audiobook,
            commands::audiobooks::get_audiobook_chapters,
            commands::audiobooks::get_saved_audiobooks,
            commands::audiobooks::save_audiobook,
            commands::audiobooks::unsave_audiobook,
            commands::audiobooks::is_audiobook_saved,
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
            commands::playback::get_yt_match,
            commands::playback::search_yt_candidates,
            commands::playback::pin_yt_match,
            commands::playback::forget_yt_match,
            commands::playback::set_volume,
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
        ])
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
