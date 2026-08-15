//! Configuration commands.

use std::collections::HashMap;

use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    Ok(state.config.get(&key))
}

#[tauri::command]
pub fn set_config(app: tauri::AppHandle, state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    use tauri::{Emitter, Manager};
    state.config.set(&key, &value).map_err(|e| e.to_string())?;

    if key == "taskbar_speed" || key == "floating_speed" {
        let taskbar_enabled = state.config.get("taskbar_speed").map(|v| v == "true").unwrap_or(false);
        let floating_enabled = state.config.get("floating_speed").map(|v| v == "true").unwrap_or(false);
        crate::tray::update_speed(&app, 0.0, 0.0, taskbar_enabled, floating_enabled);
    } else if key == "floating_click_through" {
        let enabled = value == "true";
        if let Some(window) = app.get_webview_window(crate::tray::FLOATING_LABEL) {
            let _ = window.set_ignore_cursor_events(enabled);
        }
        let _ = app.emit("floating:click-through", enabled);
    } else if key == "floating_opacity" {
        if let Ok(op) = value.parse::<u32>() {
            let _ = app.emit("floating:opacity", op);
        }
    } else if key == "theme" {
        let _ = app.emit("theme:sync", &value);
    }

    let _ = app.emit("config:sync", serde_json::json!({ "key": &key, "value": &value }));

    Ok(())
}

#[tauri::command]
pub fn get_all_config(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    Ok(state.config.get_all())
}
