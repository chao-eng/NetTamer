//! Builds the system tray icon.
//!
//! Best-effort: any failure is mapped to an [`Error`](crate::models::Error) and
//! the caller logs it so the app still runs without a tray.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

/// Create the tray icon with right-click menu (仪表盘 / 退出).
pub fn setup(app: &AppHandle) -> Result<(), crate::models::Error> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| crate::models::Error::new("no default window icon available for tray"))?;

    let dashboard_item = MenuItem::with_id(app, "dashboard", "仪表盘", true, None::<&str>)
        .map_err(|e| crate::models::Error(e.to_string()))?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    let menu = Menu::with_items(app, &[&dashboard_item, &quit_item])
        .map_err(|e| crate::models::Error(e.to_string()))?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("NetTamer - 网络驯兽师")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "dashboard" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    Ok(())
}
