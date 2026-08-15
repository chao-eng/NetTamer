//! Process firewall and network isolation commands.

use tauri::State;

use crate::firewall::Manager;
use crate::models::FirewallRule;
use crate::state::AppState;

#[tauri::command]
pub fn apply_firewall_rule(state: State<'_, AppState>, rule: FirewallRule) -> Result<(), String> {
    let mgr = Manager::new(state.store.clone(), state.firewall.clone());
    mgr.apply(rule).map_err(|e| e.to_string())?;
    state.sync_wfp_state();
    Ok(())
}

#[tauri::command]
pub fn remove_firewall_rule(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mgr = Manager::new(state.store.clone(), state.firewall.clone());
    mgr.remove(&id).map_err(|e| e.to_string())?;
    state.sync_wfp_state();
    Ok(())
}

#[tauri::command]
pub fn list_firewall_rules(state: State<'_, AppState>) -> Result<Vec<FirewallRule>, String> {
    let mgr = Manager::new(state.store.clone(), state.firewall.clone());
    mgr.list().map_err(|e| e.to_string())
}
