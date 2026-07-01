use std::sync::mpsc;
use std::time::Duration;

use souvlaki::{MediaControls, MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use tauri::{AppHandle, Emitter};

pub enum MediaMsg {
    NowPlaying {
        title:       String,
        artist:      String,
        album:       String,
        cover_url:   Option<String>,
        track_url:   Option<String>,
        duration_ms: u64,
        position_ms: u64,
    },
    Playing  { position_ms: u64 },
    Paused   { position_ms: u64 },
    Stopped,
    /// user flipped discord rich presence in settings
    SetDiscordEnabled(bool),
}

/// kicks off the media-controls background thread
/// gives back a `SyncSender` u can clone into AppState
/// if init fails the receiver just gets dropped right away,
/// `try_send` on the returned sender will quietly give back
/// `Err(Disconnected)` so callers can always just ignore errors
pub fn start(app: AppHandle, hwnd_raw: Option<isize>) -> mpsc::SyncSender<MediaMsg> {
    let (tx, rx) = mpsc::sync_channel::<MediaMsg>(64);

    std::thread::Builder::new()
        .name("media-controls".into())
        .spawn(move || {
            // souvlakis PlatformConfig always has an `hwnd` field but its only
            // meaningful on windows (SMTC needs the window handle) where souvlaki
            // `expect()`s it, a None there used to PANIC and crash the app at
            // launch. the handle gets grabbed on the main thread (lib.rs) and
            // passed in as an `isize` (Send), we rebuild the pointer here. on
            // macos (MPNowPlayingInfoCenter) and linux (MPRIS/D-Bus) its None
            let hwnd: Option<*mut std::ffi::c_void> =
                hwnd_raw.map(|v| v as *mut std::ffi::c_void);

            let config = PlatformConfig {
                dbus_name: "dev.boyblah.spotify",
                display_name: "Spotify Client",
                hwnd,
            };

            // os media controls are best effort, on a busted host (and
            // historically on some macos setups where touching MediaPlayer from
            // this thread acts up) init/attach can fail or straight up panic. keep it
            // OPTIONAL and isolated so a failure never kills the thread, which
            // also runs discord rich presence. `catch_unwind` (works now that
            // release uses panic=unwind) turns a panic into a clean `None`
            let ev_app = app.clone();
            let mut controls = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut c = MediaControls::new(config).map_err(|e| {
                    eprintln!("[media-ctrl] init failed: {e:?}");
                }).ok()?;
                if let Err(e) = c.attach(move |event: MediaControlEvent| {
                    let _ = match event {
                        MediaControlEvent::Play     => ev_app.emit("media:play",   ()),
                        MediaControlEvent::Pause    => ev_app.emit("media:pause",  ()),
                        MediaControlEvent::Toggle   => ev_app.emit("media:toggle", ()),
                        MediaControlEvent::Next     => ev_app.emit("media:next",   ()),
                        MediaControlEvent::Previous => ev_app.emit("media:prev",   ()),
                        MediaControlEvent::Stop     => ev_app.emit("media:stop",   ()),
                        _ => Ok(()),
                    };
                }) {
                    eprintln!("[media-ctrl] attach failed: {e:?}");
                    return None;
                }
                Some(c)
            }))
            .unwrap_or_else(|_| {
                eprintln!("[media-ctrl] init panicked — OS media controls disabled, Discord still active");
                None
            });

            // discord rich presence rides this same thread, it already sees
            // every NowPlaying / Playing / Paused / Stopped message so we dont
            // need a separate channel. fully best effort, if discord is
            // closed every call just does nothing
            let mut discord = crate::discord::DiscordPresence::new();
            // remember the last position/playing state so a settings toggle can
            // re-show presence mid track without waiting for the next event
            let mut last_pos: u64 = 0;
            let mut last_playing = false;

            // recv_timeout instead of a plain `for msg in rx` so we can poke
            // discord every so often, if it was closed at launch and opened later the
            // tick reconnects and shows presence without needing a track change
            loop {
                let msg = match rx.recv_timeout(Duration::from_secs(15)) {
                    Ok(m) => m,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        discord.tick();
                        continue;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                };
                match msg {
                    MediaMsg::NowPlaying {
                        ref title, ref artist, ref album,
                        ref cover_url, ref track_url, duration_ms, position_ms,
                    } => {
                        if let Some(c) = controls.as_mut() {
                            let _ = c.set_metadata(MediaMetadata {
                                title:     Some(title),
                                artist:    Some(artist),
                                album:     Some(album),
                                cover_url: cover_url.as_deref(),
                                duration:  Some(Duration::from_millis(duration_ms)),
                            });
                            let _ = c.set_playback(MediaPlayback::Playing {
                                progress: Some(MediaPosition(Duration::from_millis(position_ms))),
                            });
                        }
                        last_pos = position_ms;
                        last_playing = true;
                        discord.now_playing(
                            title.clone(), artist.clone(), album.clone(),
                            cover_url.clone(), track_url.clone(), duration_ms, position_ms, true,
                        );
                    }
                    MediaMsg::Playing { position_ms } => {
                        if let Some(c) = controls.as_mut() {
                            let _ = c.set_playback(MediaPlayback::Playing {
                                progress: Some(MediaPosition(Duration::from_millis(position_ms))),
                            });
                        }
                        last_pos = position_ms;
                        last_playing = true;
                        discord.set_state(position_ms, true);
                    }
                    MediaMsg::Paused { position_ms } => {
                        if let Some(c) = controls.as_mut() {
                            let _ = c.set_playback(MediaPlayback::Paused {
                                progress: Some(MediaPosition(Duration::from_millis(position_ms))),
                            });
                        }
                        last_pos = position_ms;
                        last_playing = false;
                        discord.set_state(position_ms, false);
                    }
                    MediaMsg::Stopped => {
                        if let Some(c) = controls.as_mut() {
                            let _ = c.set_playback(MediaPlayback::Stopped);
                            let _ = c.set_metadata(MediaMetadata {
                                title: None, artist: None, album: None,
                                cover_url: None, duration: None,
                            });
                        }
                        last_playing = false;
                        discord.clear();
                    }
                    MediaMsg::SetDiscordEnabled(enabled) => {
                        discord.set_enabled(enabled, last_pos, last_playing);
                    }
                }
            }
        })
        .expect("spawn media-controls thread");

    tx
}
