//! Builds the system tray icon.
//!
//! Best-effort: any failure is mapped to an [`Error`](crate::models::Error) and
//! the caller logs it so the app still runs without a tray.

use tauri::tray::TrayIconEvent;
use tauri::{AppHandle, Manager};

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
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    Ok(())
}
