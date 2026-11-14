fn main() {
    soloist_capture();
    tauri_build::build()
}

/// Soloist's sound, read back for the visualizer (src/soloist/capture.rs).
/// Only Windows has a way to read one process's sound: WASAPI process loopback.
fn soloist_capture() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let source = "src/soloist/native/windows.cpp";
    println!("cargo:rerun-if-changed={source}");
    println!("cargo:rerun-if-changed=src/soloist/native/audio.h");
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file(source)
        .flag_if_supported("/EHsc")
        .compile("musique_soloist_audio");
    for lib in ["mmdevapi", "ole32", "uuid"] {
        println!("cargo:rustc-link-lib={lib}");
    }
}
