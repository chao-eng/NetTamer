//! Throttle policy commands.

use tauri::State;

use crate::models::Policy;
use crate::state::AppState;
use crate::throttle::Manager;

#[tauri::command]
pub fn apply_throttle_policy(state: State<'_, AppState>, policy: Policy) -> Result<(), String> {
    let mgr = Manager::new(state.store.clone(), state.throttle.clone());
    mgr.apply(policy).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_throttle_policy(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mgr = Manager::new(state.store.clone(), state.throttle.clone());
    mgr.remove(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_throttle_policies(state: State<'_, AppState>) -> Result<Vec<Policy>, String> {
    let mgr = Manager::new(state.store.clone(), state.throttle.clone());
    mgr.list().map_err(|e| e.to_string())
}
