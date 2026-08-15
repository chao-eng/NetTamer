// Pre-link build script for Tauri 2. Generates the context (window/plugin/menu
// definitions, embedded icons, etc.) consumed by `tauri::generate_context!()`.
fn main() {
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
