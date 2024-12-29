#[cfg(target_os = "macos")]
fn main() {
    println!("Building for macOS...");
    println!("cargo:rustc-link-lib=framework=JavaScriptCore");
}

#[cfg(target_os = "linux")]
const POTENTIAL_LIBS: [&str; 3] = [
    "javascriptcoregtk-4.1",
    "javascriptcoregtk-4.0",
    "javascriptcoregtk-3.0",
];

#[cfg(target_os = "linux")]
fn main() {
    println!("Building for Linux...");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    for l in POTENTIAL_LIBS {
        if pkg_config::probe_library(l).is_ok() {
            return;
        }
    }
    panic!("libjavascriptcoregtk-4.0, 4.1, or 3.0 must be installed.");
}

#[cfg(target_os = "windows")]
fn main() {
    println!("Building for Windows...");
    println!("cargo:rustc-link-search=native=.build/windows/win/lib32");
    println!("cargo:rustc-link-lib=dylib=JavaScriptCore");
    println!("cargo:rustc-link-lib=dylib=WTF");

    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn main() {
    panic!("Only macOS, Linux, and Windows are supported currently.");
}
