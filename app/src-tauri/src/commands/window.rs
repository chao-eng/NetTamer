//! Window management commands.

use tauri::{AppHandle, Manager};

/// Hide the main window so the app continues running in the system tray.
#[tauri::command]
pub fn minimize_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
