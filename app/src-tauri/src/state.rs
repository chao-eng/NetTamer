//! Global, shared application state managed by Tauri (`app.manage(AppState)`).
//!
//! Long-lived, shareable values are wrapped in `Arc` so they can be cheaply
//! cloned into background tasks (the 1s emit loop, the alert-emit thread) while
//! remaining the same instance the `#[tauri::command]` handlers read through
//! `tauri::State`.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::alert;
use crate::config;
use crate::etw;
use crate::firewall;
use crate::monitor;
use crate::process;
use crate::store;
use crate::wfp;

pub struct AppState {
    /// Active ETW trace session (started/stopped by the monitor commands).
    pub etw: Mutex<Option<etw::Session>>,

    /// Per-process rate aggregator. Snapshotted once per `refresh_interval`.
    pub aggregator: Arc<monitor::Aggregator>,

    /// Native WFP Engine for process-level network isolation.
    pub wfp: Arc<wfp::WfpEngine>,

    /// Firewall rule table for process network isolation.
    pub firewall: Arc<firewall::FirewallTable>,

    /// Alert rule engine.
    pub alert: Arc<alert::Engine>,

    /// SQLite-backed persistence layer.
    pub store: store::Db,

    /// Socket -> PID mapping (used for connection metadata).
    #[allow(dead_code)]
    pub port_map: Arc<process::PortPidMap>,

    /// Process metadata resolver (name / path / icon / exe lookup).
    pub resolver: Arc<process::Resolver>,

    /// Typed config accessor over the `config` table.
    pub config: config::Config,

    /// Emit cadence in milliseconds (updated by `set_refresh_interval`).
    pub refresh_interval: Arc<AtomicU64>,

    /// Whether live monitoring (ETW) is currently active.
    pub running: AtomicBool,
}

impl AppState {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn set_running(&self, v: bool) {
        self.running.store(v, Ordering::Relaxed);
    }

    /// Synchronize active firewall rules with the native WFP filtering engine.
    /// Blocks all active target processes at the kernel ALE layer.
    pub fn sync_wfp_state(&self) {
        let mgr = firewall::Manager::new(self.store.clone(), self.firewall.clone());
        let _ = mgr.load();
        let active_rules: Vec<crate::models::FirewallRule> = mgr
            .list()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.active)
            .collect();

        let mut target_paths: HashSet<String> = HashSet::new();

        for rule in &active_rules {
            let found_paths = self.resolver.find_exe_paths_by_name(&rule.process_name);
            if found_paths.is_empty() {
                // If the process isn't running yet but is an absolute path, add it directly.
                if std::path::Path::new(&rule.process_name).is_absolute() {
                    target_paths.insert(rule.process_name.to_lowercase());
                } else {
                    log::debug!(
                        "WFP sync: target process '{}' has no active running instances yet",
                        rule.process_name
                    );
                }
            } else {
                for path in found_paths {
                    target_paths.insert(path.to_lowercase());
                }
            }
        }

        let currently_blocked: HashSet<String> = self.wfp.list_blocked().into_iter().collect();

        // 1. Block new target processes
        for path in target_paths.difference(&currently_blocked) {
            if let Err(e) = self.wfp.block_process(path) {
                log::warn!("WFP sync: failed to block process '{}': {e}", path);
            }
        }

        // 2. Unblock processes no longer in active rules
        for path in currently_blocked.difference(&target_paths) {
            if let Err(e) = self.wfp.unblock_process(path) {
                log::warn!("WFP sync: failed to unblock process '{}': {e}", path);
            }
        }
    }

    #[allow(dead_code)]
    pub fn refresh_ms(&self) -> u64 {
        self.refresh_interval.load(Ordering::Relaxed).max(100)
    }
}
