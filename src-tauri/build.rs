fn main() {
    widevine_shim();
    tauri_build::build()
}

/// The native half of the Widevine CDM host (src/widevine/shim/host.cc), a C++ adapter between
/// the module's C++ interface and a flat C API the Rust host calls.
fn widevine_shim() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if !matches!(os.as_str(), "windows" | "macos" | "linux") {
        return;
    }
    println!("cargo:rerun-if-changed=src/widevine/shim/host.cc");
    println!("cargo:rerun-if-changed=src/widevine/shim/cdm/content_decryption_module.h");
    println!("cargo:rerun-if-changed=src/widevine/shim/cdm/content_decryption_module_export.h");
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("src/widevine/shim/host.cc")
        .include("src/widevine/shim")
        .warnings(false);
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        build.flag("/std:c++14").flag("/EHsc");
    } else {
        build.flag("-std=c++14").flag("-fvisibility=hidden");
    }
    build.compile("widevine_host");
    if os == "linux" {
        println!("cargo:rustc-link-lib=dylib=dl");
    }
}
