// stops a random extra console window popping up on windows release builds. DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    if std::env::var_os("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").is_none() {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--js-flags=--max-old-space-size=64,--optimize-for-size,--expose-gc \
             --renderer-process-limit=1 \
             --enable-features=TrimOnMemoryPressure,NetworkServiceInProcess \
             --gpu-rasterization-msaa-sample-count=0 \
             --num-raster-threads=1 \
             --disk-cache-size=16777216 \
             --media-cache-size=16777216 \
             --disable-background-networking \
             --disable-component-update \
             --disable-domain-reliability \
             --disable-sync \
             --disable-breakpad \
             --disable-features=Translate,OptimizationHints,MediaRouter,CalculateNativeWinOcclusion,InterestFeedContentSuggestions,BackForwardCache,GlobalMediaControls",
        );
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
    spotify_lib::run()
}
