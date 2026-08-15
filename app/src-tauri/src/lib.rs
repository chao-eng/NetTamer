//! NetTamer — Tauri 2.0 application entry point (Rust backend).
//!
//! Wires together the ETW monitor, the WinDivert throttle engine, the alert
//! engine, the SQLite store and the typed config, then exposes everything to
//! the frontend through `#[tauri::command]`s and background `app.emit(...)` events.

mod alert;
mod commands;
mod config;
mod etw;
mod models;
mod monitor;
mod notify;
mod process;
mod state;
mod store;
mod throttle;
mod tray;
mod windivert;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Emitter, Manager};

// Bring every command into scope for `generate_handler!`.
use crate::commands::{alert as alert_cmds, config as config_cmds, monitor as monitor_cmds, throttle as throttle_cmds};

use crate::models::{AlertEvent, SystemStats};
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle().clone();

            // ---- Resolve the on-disk database path ---------------------------
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("nettamer.db");

            // ---- Core services ----------------------------------------------
            let store = store::Db::new(&db_path)?;
            store::migrations::run(&store)?;

            let config = config::Config::new(store.clone());
            // Seed defaults if missing (idempotent).
            config.ensure_defaults();

            let resolver = Arc::new(process::Resolver::new());
            let port_map = Arc::new(process::PortPidMap::new());
            let aggregator = Arc::new(monitor::Aggregator::new(
                Duration::from_secs(1),
                resolver.clone(),
            ));
            let throttle = Arc::new(throttle::ThrottleTable::new());

            // Alert engine owns one side of the alert event channel; a dedicated
            // thread forwards `AlertEvent`s to the frontend as `alert:triggered`.
            let (alert_tx, alert_rx) = std::sync::mpsc::channel::<AlertEvent>();
            let alert = Arc::new(alert::Engine::new(store.clone(), alert_tx));
            alert.load_rules();

            let refresh_interval = Arc::new(std::sync::atomic::AtomicU64::new(
                config
                    .get("refresh_interval_ms")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(1000),
            ));

            let app_state = AppState {
                etw: Mutex::new(None),
                aggregator,
                windivert: Mutex::new(None),
                throttle,
                alert,
                store,
                port_map,
                resolver,
                config,
                refresh_interval,
                running: std::sync::atomic::AtomicBool::new(false),
            };

            // ---- Background task: emit speed + system stats every N ms -------
            let agg = app_state.aggregator.clone();
            let alert_engine = app_state.alert.clone();
            let refresh = app_state.refresh_interval.clone();
            let emit_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let ms = refresh.load(Ordering::Relaxed).max(100);
                    tokio::time::sleep(Duration::from_millis(ms)).await;

                    let stats = agg.snapshot();
                    let total_up: f64 = stats.iter().map(|s| s.upload_rate).sum();
                    let total_down: f64 = stats.iter().map(|s| s.download_rate).sum();

                    // Evaluate alert rules against the fresh snapshot (sends
                    // AlertEvents which the forwarding thread emits as
                    // `alert:triggered`).
                    alert_engine.evaluate(&stats);

                    let _ = emit_handle.emit("speed:update", stats);
                    let _ = emit_handle.emit(
                        "system:stats",
                        SystemStats {
                            total_upload_rate: total_up,
                            total_download_rate: total_down,
                        },
                    );
                }
            });

            // ---- Background thread: forward alert events to frontend + OS toast
            let alert_handle = handle.clone();
            std::thread::spawn(move || {
                for ev in alert_rx {
                    let current_str = if ev.current_rate >= 1_048_576.0 {
                        format!("{:.2} MB/s", ev.current_rate / 1_048_576.0)
                    } else {
                        format!("{:.2} KB/s", ev.current_rate / 1024.0)
                    };
                    let threshold_str = if ev.threshold >= 1_048_576.0 {
                        format!("{:.2} MB/s", ev.threshold / 1_048_576.0)
                    } else {
                        format!("{:.2} KB/s", ev.threshold / 1024.0)
                    };

                    let body = format!(
                        "进程「{}」当前速率 {}，超过设定阈值 {}！",
                        ev.process_name, current_str, threshold_str
                    );
                    notify::toast::notify(&alert_handle, "🐾 NetTamer 流量预警", &body);
                    let _ = alert_handle.emit("alert:triggered", ev);
                }
            });

            // ---- System tray -------------------------------------------------
            if let Err(e) = tray::setup(&handle) {
                log::warn!("tray setup failed: {e}");
            }

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            monitor_cmds::start_monitoring,
            monitor_cmds::stop_monitoring,
            monitor_cmds::get_process_list,
            monitor_cmds::set_refresh_interval,
            alert_cmds::create_alert_rule,
            alert_cmds::update_alert_rule,
            alert_cmds::delete_alert_rule,
            alert_cmds::list_alert_rules,
            alert_cmds::get_alert_history,
            throttle_cmds::apply_throttle_policy,
            throttle_cmds::remove_throttle_policy,
            throttle_cmds::list_throttle_policies,
            config_cmds::get_config,
            config_cmds::set_config,
            config_cmds::get_all_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running NetTamer");
}
