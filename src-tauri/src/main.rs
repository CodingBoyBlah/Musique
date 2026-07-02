// stops a random extra console window popping up on windows release builds. DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
   
    
    if std::env::args().any(|a| a == "--connect-probe") {
        std::process::exit(spotify_lib::connect_probe());
    }
    if std::env::args().any(|a| a == "--playback-probe") {
        std::process::exit(spotify_lib::playback_probe());
    }
    spotify_lib::run()
}
