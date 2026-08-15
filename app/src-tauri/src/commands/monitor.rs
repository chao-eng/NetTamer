//! Monitoring commands: start/stop ETW+WinDivert, list processes, set interval.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::State;

use crate::etw;
use crate::models::ProcessStats;
use crate::state::AppState;

/// Start live monitoring: open the ETW session and the WinDivert engine.
#[tauri::command]
pub fn start_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    if state.is_running() {
        return Ok(());
    }

    // 1) ETW real-time session feeding the aggregator.
    let (session, rx) = etw::Session::start(1024).map_err(|e| e.to_string())?;
    let agg = state.aggregator.clone();
    std::thread::spawn(move || {
        for ev in rx {
            agg.ingest(ev);
        }
    });

    // 2) WinDivert capture engine (best-effort; needs admin + driver present).
    match crate::windivert::WinDivertEngine::open(
        "tcp or udp",
        state.throttle.clone(),
        state.port_map.clone(),
        state.resolver.clone(),
    ) {
        Ok(engine) => {
            let arc = Arc::new(engine);
            arc.run();
            *state.windivert.lock().unwrap() = Some(arc);
        }
        Err(e) => log::warn!("WinDivert engine not started: {e}"),
    }

    *state.etw.lock().unwrap() = Some(session);
    state.set_running(true);
    Ok(())
}

/// Stop live monitoring and release both engines.
#[tauri::command]
pub fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(session) = state.etw.lock().unwrap().take() {
        session.stop().map_err(|e| e.to_string())?;
    }
    if let Some(engine) = state.windivert.lock().unwrap().take() {
        engine.stop().map_err(|e| e.to_string())?;
    }
    state.set_running(false);
    Ok(())
}

/// Snapshot of all currently tracked processes.
#[tauri::command]
pub fn get_process_list(state: State<'_, AppState>) -> Result<Vec<ProcessStats>, String> {
    Ok(state.aggregator.snapshot())
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
