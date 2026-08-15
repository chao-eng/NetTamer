//! Alert rule commands.

use tauri::State;

use crate::models::{AlertEvent, AlertHistoryFilter, Rule};
use crate::state::AppState;

#[tauri::command]
pub fn create_alert_rule(state: State<'_, AppState>, rule: Rule) -> Result<(), String> {
    state.alert.create_rule(rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_alert_rule(state: State<'_, AppState>, rule: Rule) -> Result<(), String> {
    state.alert.update_rule(rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_alert_rule(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.alert.delete_rule(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_alert_rules(state: State<'_, AppState>) -> Result<Vec<Rule>, String> {
    Ok(state.alert.list_rules())
}

#[tauri::command]
pub fn get_alert_history(
    state: State<'_, AppState>,
    filter: Option<AlertHistoryFilter>,
) -> Result<Vec<AlertEvent>, String> {
    let f = filter.unwrap_or_default();
    state.alert.get_history(&f).map_err(|e| e.to_string())
}
