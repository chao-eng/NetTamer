//! Monitoring commands: start/stop ETW, list processes, set interval.

use std::sync::atomic::Ordering;

use tauri::State;

use crate::etw;
use crate::models::ProcessStats;
use crate::state::AppState;

/// Start live monitoring: open the ETW session.
#[tauri::command]
pub fn start_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    if state.is_running() {
        return Ok(());
    }

    // ETW real-time session feeding the aggregator.
    let (session, rx) = etw::Session::start(1024).map_err(|e| e.to_string())?;
    let agg = state.aggregator.clone();
    std::thread::spawn(move || {
        for ev in rx {
            agg.ingest(ev);
        }
    });

    *state.etw.lock().unwrap() = Some(session);
    state.set_running(true);
    Ok(())
}

/// Stop live monitoring and release the ETW session.
#[tauri::command]
pub fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(session) = state.etw.lock().unwrap().take() {
        session.stop().map_err(|e| e.to_string())?;
    }
    state.set_running(false);
    Ok(())
}

/// Snapshot of all currently tracked processes.
#[tauri::command]
pub fn get_process_list(state: State<'_, AppState>) -> Result<Vec<ProcessStats>, String> {
    Ok(state.aggregator.snapshot())
}

/// Snapshot of system-wide total upload and download speeds.
#[tauri::command]
pub fn get_system_stats(state: State<'_, AppState>) -> Result<crate::models::SystemStats, String> {
    let procs = state.aggregator.snapshot();
    let total_up: f64 = procs.iter().map(|s| s.upload_rate).sum();
    let total_down: f64 = procs.iter().map(|s| s.download_rate).sum();
    Ok(crate::models::SystemStats {
        total_upload_rate: total_up,
        total_download_rate: total_down,
    })
}

/// Update the emit cadence (also persisted to config).
#[tauri::command]
pub fn set_refresh_interval(state: State<'_, AppState>, ms: u64) -> Result<(), String> {
    let ms = ms.max(100);
    state.refresh_interval.store(ms, Ordering::Relaxed);
    state
        .config
        .set("refresh_interval_ms", &ms.to_string())
        .map_err(|e| e.to_string())?;
    Ok(())
}
