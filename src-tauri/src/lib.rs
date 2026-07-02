use std::sync::Arc;
use tauri::{
    Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter,
};
use tokio::sync::RwLock;

mod auth;
mod commands;


pub fn connect_probe() -> i32 {
    use librespot_core::{authentication::Credentials, config::SessionConfig, session::Session};

  
    struct StderrLog;
    impl log::Log for StderrLog {
        fn enabled(&self, _: &log::Metadata) -> bool { true }
        fn log(&self, r: &log::Record) { eprintln!("[{}] {}: {}", r.level(), r.target(), r.args()); }
        fn flush(&self) {}
    }
    static LOGGER: StderrLog = StderrLog;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Trace);

    let rt = match tokio::runtime::Runtime::new() { Ok(r) => r, Err(_) => return 10 };
    rt.block_on(async {
        let token = match keyring::Entry::new("musique", "access_token")
            .and_then(|e| e.get_password())
        {
            Ok(t) => t,
            Err(e) => { eprintln!("[connect-probe] no stored token: {e}"); return 11; }
        };
        let appdata = match std::env::var("APPDATA") {
            Ok(v) => v,
            Err(_) => { eprintln!("[connect-probe] no APPDATA"); return 12; }
        };
        let db_url = format!("sqlite:{}/dev.boyblah.musique/musique.db", appdata.replace('\\', "/"));
        let pool = match sqlx::SqlitePool::connect(&db_url).await {
            Ok(p) => p,
            Err(e) => { eprintln!("[connect-probe] db open failed: {e}"); return 13; }
        };
        let cid: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key='spotify_client_id'")
            .fetch_optional(&pool).await.ok().flatten();
        let Some((client_id,)) = cid else { eprintln!("[connect-probe] no client_id in settings"); return 14; };
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
    use librespot_core::{authentication::Credentials, config::SessionConfig, session::Session, SpotifyId, SpotifyUri};
    use librespot_playback::audio_backend::{Sink, SinkResult};
    use librespot_playback::config::{Bitrate, PlayerConfig};
    use librespot_playback::convert::Converter;
    use librespot_playback::decoder::AudioPacket;
    use librespot_playback::mixer::VolumeGetter;
    use librespot_playback::player::{Player, PlayerEvent};

    struct NullSink;
    impl Sink for NullSink {
        fn write(&mut self, _p: AudioPacket, _c: &mut Converter) -> SinkResult<()> { Ok(()) }
    }
    struct FullVol;
    impl VolumeGetter for FullVol { fn attenuation_factor(&self) -> f64 { 1.0 } }

    struct StderrLog;
    impl log::Log for StderrLog {
        fn enabled(&self, _: &log::Metadata) -> bool { true }
        fn log(&self, r: &log::Record) {
            let t = r.target();
            if t.starts_with("librespot") { eprintln!("[{}] {}: {}", r.level(), t, r.args()); }
        }
        fn flush(&self) {}
    }
    static LOGGER: StderrLog = StderrLog;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Debug);

    let rt = match tokio::runtime::Runtime::new() { Ok(r) => r, Err(_) => return 10 };
    rt.block_on(async {
        let appdata = match std::env::var("APPDATA") { Ok(v) => v, Err(_) => { eprintln!("[playback-probe] no APPDATA"); return 12; } };
        let db_url = format!("sqlite:{}/dev.boyblah.musique/musique.db", appdata.replace('\\', "/"));
        let pool = match sqlx::SqlitePool::connect(&db_url).await { Ok(p) => p, Err(e) => { eprintln!("[playback-probe] db open failed: {e}"); return 13; } };
        let cid: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key='spotify_client_id'").fetch_optional(&pool).await.ok().flatten();
        let Some((client_id,)) = cid else { eprintln!("[playback-probe] no client_id in settings"); return 14; };
        let client_id = client_id.trim().to_string();

        
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

        let mut cfg = SessionConfig::default();
       
        let session_cid = std::env::var("SPOTIFY_SESSION_CLIENT_ID").ok().filter(|s| !s.trim().is_empty());
        cfg.client_id = session_cid.clone().unwrap_or(client_id);
        eprintln!("[playback-probe] session client_id override = {session_cid:?}");
        let session = Session::new(cfg, None);
        if let Err(e) = session.connect(Credentials::with_access_token(&token), false).await {
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

        let player = Player::new(
            PlayerConfig { bitrate: Bitrate::Bitrate320, ..Default::default() },
            session.clone(),
            Box::new(FullVol),
            move || Box::new(NullSink) as Box<dyn Sink>,
        );
        let mut rx = player.get_player_event_channel();
        player.load(SpotifyUri::Track { id: spotify_id }, true, 0);

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() { eprintln!("[playback-probe] TIMEOUT — no decisive event"); return 6; }
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Some(ev)) => match ev {
                    PlayerEvent::Playing { .. } | PlayerEvent::TrackChanged { .. } => {
                        eprintln!("[playback-probe] SUCCESS — audio keys granted, track is PLAYING. Region/unavailable error is GONE.");
                        return 0;
                    }
                    PlayerEvent::Unavailable { track_id, .. } => {
                        eprintln!("[playback-probe] BLOCKED — track {track_id:?} reported Unavailable (region/availability/relink). check_catalogue survived but the track itself is blocked.");
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
                libc::SIGSEGV, libc::SIGBUS, libc::SIGILL, libc::SIGFPE, libc::SIGABRT,
            ] {
                libc::signal(sig, bail as libc::sighandler_t);
            }
        }
    }

    // open EXACTLY the way the app will: default device at its NATIVE rate via
    // DeviceSink. that path avoids cpal's set_sample_rate() bug, so this returns 0
    // and the app uses the real device. a graceful Err is fine (parent -> NullSink);
    // a hard fault hits the handler above -> clean _exit(70), no crash dialog.
    let _ = crate::playback::DeviceSink::open();
    0
}

mod db;
mod discord;
mod errors;
mod lastfm;
mod library;
mod lyrics;
mod media_controls;
mod playback;
mod spotify;
mod state;


fn install_panic_logger() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut path = std::env::temp_dir();
        path.push("spotify-panic.log");
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let thread = std::thread::current().name().unwrap_or("<unnamed>").to_string();
        let bt = std::backtrace::Backtrace::force_capture();
        let entry = format!("\n===== PANIC @unix={secs} thread={thread} =====\n{info}\n{bt}\n");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
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
            libc::SIGBUS  => b"SIGBUS",
            libc::SIGILL  => b"SIGILL",
            libc::SIGFPE  => b"SIGFPE",
            libc::SIGABRT => b"SIGABRT",
            _             => b"SIGNAL",
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
                libc::SIGSEGV, libc::SIGBUS, libc::SIGILL, libc::SIGFPE, libc::SIGABRT,
            ] {
                libc::signal(sig, handler as libc::sighandler_t);
            }
        }
    }
}


struct PlaybackLog;
impl log::Log for PlaybackLog {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.target().starts_with("librespot") && m.level() <= log::Level::Info
    }
    fn log(&self, r: &log::Record) {
        if !self.enabled(r.metadata()) {
            return;
        }
        let line = format!("[{}] {}: {}\n", r.level(), r.target(), r.args());
        eprint!("{line}");
        let mut p = std::env::temp_dir();
        p.push("spotify-playback.log");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
            use std::io::Write;
            let _ = f.write_all(line.as_bytes());
        }
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

    
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    
    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::from_filename(".env");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            
            let mut backdrop_active = false;
            
            let mut main_hwnd: Option<isize> = None;
            if let Some(window) = app.get_webview_window("main") {
                
                #[cfg(not(target_os = "macos"))]
                let _ = window.set_decorations(false);

                
                let _ = window.set_theme(Some(tauri::Theme::Dark));

                // win11 gets mica, older builds fall back to acrylic
                #[cfg(target_os = "windows")]
                {
                    use window_vibrancy::{apply_mica, apply_acrylic};
                    backdrop_active = apply_mica(&window, Some(true)).is_ok()
                        || apply_acrylic(&window, Some((18, 18, 18, 110))).is_ok();
                }

                
                #[cfg(target_os = "macos")]
                {
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
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
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pool       = db::connection::create_pool(&handle).await?;
                // seed spotify creds from .env on first run, does nothing if theyre already set
                commands::credentials::seed_credentials_from_env(&pool).await;
                let auth_state = auth::init_auth_state(&pool).await;
                handle.manage(state::AppState {
                    db:       pool,
                    auth:     Arc::new(RwLock::new(auth_state)),
                    playback: Arc::new(tokio::sync::Mutex::new(None)),
                    media_tx,
                    backdrop_active,
                });
                Ok::<(), Box<dyn std::error::Error>>(())
            })?;

            // background loops that just keep running
            let handle2 = app.handle().clone();
            tauri::async_runtime::spawn(auth::refresh_loop(handle2));

            let handle3 = app.handle().clone();
            tauri::async_runtime::spawn(library::library_sync_loop(handle3));

            // system tray stuff
            let show_item   = MenuItem::with_id(app, "show",   "Show",         true, None::<&str>)?;
            let sep1        = PredefinedMenuItem::separator(app)?;
            // plain text labels, the U+23EE/EF/ED media glyphs turn into tofu boxes in minimal
            // gtk menu fonts on linux and arent guaranteed in every macos menu
            // either, text always renders so we just use that
            let prev_item   = MenuItem::with_id(app, "prev",   "Previous",   true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Play / Pause", true, None::<&str>)?;
            let next_item   = MenuItem::with_id(app, "next",   "Next",       true, None::<&str>)?;
            let sep2        = PredefinedMenuItem::separator(app)?;
            let quit_item   = MenuItem::with_id(app, "quit",   "Quit",         true, None::<&str>)?;

            let menu = Menu::with_items(app, &[
                &show_item, &sep1,
                &prev_item, &toggle_item, &next_item,
                &sep2, &quit_item,
            ])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                w.show().ok();
                                w.set_focus().ok();
                            }
                        }
                        "quit"   => app.exit(0),
                        "toggle" => { app.emit("media:toggle", ()).ok(); }
                        "next"   => { app.emit("media:next",   ()).ok(); }
                        "prev"   => { app.emit("media:prev",   ()).ok(); }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button:       MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
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
                })
                .build(app)?;

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
            commands::auth::logout,
            commands::auth::get_auth_status,
            commands::auth::get_profile,
            commands::spotify::search,
            commands::spotify::get_artist,
            commands::spotify::get_album,
            commands::spotify::get_track,
            commands::spotify::get_playlist,
            commands::spotify::get_recommendations,
            commands::playback::warmup_playback,
            commands::playback::play_track,
            commands::playback::retry_play_track,
            commands::playback::pause_playback,
            commands::playback::resume_playback,
            commands::playback::resume_or_play,
            commands::playback::stop_playback,
            commands::playback::seek_playback,
            commands::playback::preload_track,
            commands::playback::set_volume,
            commands::playback::set_muted,
            commands::playback::get_volume,
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
            commands::library::add_track_to_playlist,
            commands::library::remove_track_from_playlist,
            commands::library::create_playlist,
            commands::media::update_now_playing,
            commands::media::set_discord_enabled,
            commands::window::set_window_effect,
            commands::window::get_backdrop_active,
            commands::lyrics::get_lyrics,
            commands::share::resolve_odesli,
            commands::lastfm::lastfm_status,
            commands::lastfm::lastfm_save_api,
            commands::lastfm::lastfm_start_auth,
            commands::lastfm::lastfm_finish_auth,
            commands::lastfm::lastfm_disconnect,
            commands::lastfm::lastfm_clear,
            commands::lastfm::lastfm_now_playing,
            commands::lastfm::lastfm_scrobble,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
