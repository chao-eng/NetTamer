//! Windows toast notifications via `tauri-plugin-notification`.

use tauri::AppHandle;

/// Show a toast notification. Best-effort: failures are swallowed.
pub fn notify(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}
