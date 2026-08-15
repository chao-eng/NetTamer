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
    state.config.set(&key, &value).map_err(|e| e.to_string())?;

    if key == "taskbar_speed" {
        let enabled = value == "true";
        crate::tray::update_speed(&app, 0.0, 0.0, enabled);
    }

    Ok(())
}

#[tauri::command]
pub fn get_all_config(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    Ok(state.config.get_all())
}
