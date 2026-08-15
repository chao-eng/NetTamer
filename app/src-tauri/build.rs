// Pre-link build script for Tauri 2. Generates the context (window/plugin/menu
// definitions, embedded icons, etc.) consumed by `tauri::generate_context!()`.
fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let bin_dir = std::path::Path::new(&manifest_dir).join("bin");
    println!("cargo:rustc-link-search=native={}", bin_dir.display());

    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        if let Some(dir) = std::path::Path::new(&out_dir).ancestors().nth(3) {
            let _ = std::fs::copy(bin_dir.join("WinDivert.dll"), dir.join("WinDivert.dll"));
            let _ = std::fs::copy(bin_dir.join("WinDivert64.sys"), dir.join("WinDivert64.sys"));
            let _ = std::fs::copy(bin_dir.join("WinDivert64.sys"), dir.join("WinDivert.sys"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        use tauri_build::WindowsAttributes;
        let windows = WindowsAttributes::new().app_manifest(include_str!("manifest.xml"));
        let attrs = tauri_build::Attributes::new().windows_attributes(windows);
        tauri_build::try_build(attrs).expect("failed to run tauri-build");
    }
    #[cfg(not(target_os = "windows"))]
    {
        tauri_build::build();
    }
}
