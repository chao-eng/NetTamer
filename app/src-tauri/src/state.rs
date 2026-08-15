//! Global, shared application state managed by Tauri (`app.manage(AppState)`).
//!
//! Long-lived, shareable values are wrapped in `Arc` so they can be cheaply
//! cloned into background tasks (the 1s emit loop, the alert-emit thread) while
//! remaining the same instance the `#[tauri::command]` handlers read through
//! `tauri::State`.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::alert;
use crate::config;
use crate::etw;
use crate::monitor;
use crate::process;
use crate::store;
use crate::throttle;
use crate::windivert;

pub struct AppState {
    /// Active ETW trace session (started/stopped by the monitor commands).
    pub etw: Mutex<Option<etw::Session>>,

    /// Per-process rate aggregator. Snapshotted once per `refresh_interval`.
    pub aggregator: Arc<monitor::Aggregator>,

    /// Active WinDivert capture engine.
    pub windivert: Mutex<Option<Arc<windivert::WinDivertEngine>>>,

    /// Runtime throttle table (token buckets keyed by PID).
    pub throttle: Arc<throttle::ThrottleTable>,

    /// Alert rule engine.
    pub alert: Arc<alert::Engine>,

    /// SQLite-backed persistence layer.
    pub store: store::Db,

    /// Socket -> PID mapping used by the WinDivert engine.
    pub port_map: Arc<process::PortPidMap>,

    /// Process metadata resolver (name / path / icon).
    pub resolver: Arc<process::Resolver>,

    /// Typed config accessor over the `config` table.
    pub config: config::Config,

    /// Emit cadence in milliseconds (updated by `set_refresh_interval`).
    pub refresh_interval: Arc<AtomicU64>,

    /// Whether live monitoring (ETW + WinDivert) is currently active.
    pub running: AtomicBool,
}

impl AppState {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn set_running(&self, v: bool) {
        self.running.store(v, Ordering::Relaxed);
    }

    /// Dynamically syncs the WinDivert capture engine status (Plan A - Process Specific Port Filtering):
    /// Only runs if monitoring is active AND there is at least one active rate-limiting policy.
    /// Dedicated to target processes; completely stops WinDivert if no active policies exist.
    pub fn sync_windivert_state(&self) {
        if !self.is_running() {
            return;
        }

        let mgr = throttle::Manager::new(self.store.clone(), self.throttle.clone());
        let active_targets: Vec<String> = mgr
            .list()
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.active && p.rate_limit_bps > 0)
            .map(|p| p.process_name)
            .collect();

        let mut guard = self.windivert.lock().unwrap();
        if !active_targets.is_empty() {
            if let Some(engine) = guard.as_ref() {
                engine.update_targets(active_targets);
            } else {
                match windivert::WinDivertEngine::start(
                    active_targets,
                    self.throttle.clone(),
                    self.port_map.clone(),
                    self.resolver.clone(),
                ) {
                    Ok(engine) => {
                        *guard = Some(engine);
                        log::info!("WinDivert capture engine started on demand for targeted processes (Plan A)");
                    }
                    Err(e) => log::warn!("WinDivert engine failed to start: {e}"),
                }
            }
        } else if let Some(engine) = guard.take() {
            engine.stop();
            log::info!("WinDivert capture engine stopped (no active rate limit policies)");
        }
    }

    #[allow(dead_code)]
    pub fn refresh_ms(&self) -> u64 {
        self.refresh_interval.load(Ordering::Relaxed).max(100)
    }
}
