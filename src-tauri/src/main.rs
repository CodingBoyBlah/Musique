// stops a random extra console window popping up on windows release builds. DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // The Widevine CDM host: this exe started by the app as a child with
    // `--widevine-host <module>`. Handled before anything else so Tauri, WebView2
    // and the single-instance lock never start in it.
    {
        let mut args = std::env::args_os().skip(1);
        if args.next().is_some_and(|a| a == spotify_lib::WIDEVINE_HOST_ARG) {
            let module = args.next().map(std::path::PathBuf::from).unwrap_or_default();
            std::process::exit(spotify_lib::widevine_host(&module));
        }
    }
    if std::env::args().any(|a| a == "--widevine-probe") {
        std::process::exit(spotify_lib::widevine_probe());
    }
    if std::env::args().any(|a| a == "--sync-probe") {
        std::process::exit(spotify_lib::sync_probe());
    }
   
    if std::env::args().any(|a| a == "--audio-probe"){
        std::process::exit(spotify_lib::audio_probe());
    }
    if std::env::args().any(|a| a == "--connect-probe") {
        std::process::exit(spotify_lib::connect_probe());
    }
    if std::env::args().any(|a| a == "--playback-probe") {
        std::process::exit(spotify_lib::playback_probe());
    }
    if std::env::args().any(|a| a == "--quality-probe") {
        std::process::exit(spotify_lib::quality_probe());
    }
    if std::env::args().any(|a| a == "--lossless-probe") {
        std::process::exit(spotify_lib::lossless_probe());
    }
    spotify_lib::run()
}
