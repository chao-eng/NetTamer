//! Builds the system tray icon.
//!
//! Best-effort: any failure is mapped to an [`Error`](crate::models::Error) and
//! the caller logs it so the app still runs without a tray.
//!
//! TODO: add a show/hide/quit `Menu` once the Tauri 2 menu-builder API
//! (`tauri::menu::{MenuBuilder, MenuItemBuilder}`) is verified against the
//! pinned Tauri version — for now we register the icon + tooltip only to keep
//! the build robust.

use tauri::AppHandle;

/// Create the tray icon (icon + tooltip). Requires the `tray-icon` Tauri feature.
pub fn setup(app: &AppHandle) -> Result<(), crate::models::Error> {
    use tauri::tray::TrayIconBuilder;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| crate::models::Error::new("no default window icon available for tray"))?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("NetTamer - 网络驯兽师")
        .build(app)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    Ok(())
}
