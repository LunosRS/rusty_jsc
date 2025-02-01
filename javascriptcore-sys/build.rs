#[cfg(target_os = "windows")]
fn main() {
    use std::process::Command;

    println!("Building for Windows (oh no)...");

    println!("cargo:rerun-if-changed=build.rs");

    let libraries = ["JavaScriptCore.lib", "WTF.lib"];
    for lib in &libraries {
        let output = Command::new("where").arg(lib).output();

        if let Ok(output) = output {
            if output.status.success() {
                println!("Found library: {}", lib);
                continue;
            }
        }

        panic!(
            "Required library '{}' not found. Please ensure it is installed and accessible.",
            lib
        );
    }

    println!("cargo:rustc-link-search=native=.build/windows/win/lib32");
    println!("cargo:rustc-link-lib=dylib=JavaScriptCore");
    println!("cargo:rustc-link-lib=dylib=WTF");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
