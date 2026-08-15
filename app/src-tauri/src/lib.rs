//! NetTamer — Tauri 2.0 application entry point (Rust backend).
//!
//! Wires together the ETW monitor, the WFP process filtering engine, the alert
//! engine, the SQLite store and the typed config, then exposes everything to
//! the frontend through `#[tauri::command]`s and background `app.emit(...)` events.

mod alert;
mod commands;
mod config;
mod etw;
mod firewall;
mod models;
mod monitor;
mod notify;
mod process;
mod state;
mod store;
mod tray;
mod wfp;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Emitter, Manager};

// Bring every command into scope for `generate_handler!`.
use crate::commands::{alert as alert_cmds, config as config_cmds, firewall as firewall_cmds, monitor as monitor_cmds, window as window_cmds};

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
            let firewall = Arc::new(firewall::FirewallTable::new());
            let firewall_mgr = firewall::Manager::new(store.clone(), firewall.clone());
            if let Err(e) = firewall_mgr.load() {
                log::warn!("failed to load firewall rules on startup: {e}");
            }

            // Initialize WFP native filtering engine
            let wfp = Arc::new(wfp::WfpEngine::new().map_err(|e| e.to_string())?);

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
                wfp,
                firewall,
                alert,
                store,
                port_map,
                resolver,
                config,
                refresh_interval,
                running: std::sync::atomic::AtomicBool::new(false),
            };

            // Sync loaded rules to WFP
            app_state.sync_wfp_state();

            // ---- Background task: emit speed + system stats every N ms -------
            let agg = app_state.aggregator.clone();
            let alert_engine = app_state.alert.clone();
            let refresh = app_state.refresh_interval.clone();
            let store_for_tray = app_state.store.clone();
            let emit_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                use crate::store::ConfigStore;
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

                    let taskbar_speed_enabled = store_for_tray
                        .config_get("taskbar_speed")
                        .ok()
                        .flatten()
                        .map(|v| v == "true")
                        .unwrap_or(false);

                    let floating_speed_enabled = store_for_tray
                        .config_get("floating_speed")
                        .ok()
                        .flatten()
                        .map(|v| v == "true")
                        .unwrap_or(false);

                    tray::update_speed(&emit_handle, total_up, total_down, taskbar_speed_enabled, floating_speed_enabled);

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

                    let dir_str = match ev.direction {
                        0 => "上传",
                        1 => "下载",
                        _ => "网络",
                    };
                    let body = format!(
                        "进程「{}」当前{}速率 {}，超过设定阈值 {}！",
                        ev.process_name, dir_str, current_str, threshold_str
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
            monitor_cmds::get_system_stats,
            monitor_cmds::set_refresh_interval,
            alert_cmds::create_alert_rule,
            alert_cmds::update_alert_rule,
            alert_cmds::delete_alert_rule,
            alert_cmds::list_alert_rules,
            alert_cmds::get_alert_history,
            firewall_cmds::apply_firewall_rule,
            firewall_cmds::remove_firewall_rule,
            firewall_cmds::list_firewall_rules,
            config_cmds::get_config,
            config_cmds::set_config,
            config_cmds::get_all_config,
            window_cmds::minimize_to_tray,
            window_cmds::show_main_window,
            window_cmds::set_floating_click_through,
            window_cmds::show_floating_context_menu,
            window_cmds::open_url,
        ])
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "float_dashboard" | "dashboard" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "op_100" => {
                    let state = app.state::<crate::state::AppState>();
                    let _ = state.config.set("floating_opacity", "100");
                    let _ = app.emit("floating:opacity", 100);
                }
                "op_80" => {
                    let state = app.state::<crate::state::AppState>();
                    let _ = state.config.set("floating_opacity", "80");
                    let _ = app.emit("floating:opacity", 80);
                }
                "op_60" => {
                    let state = app.state::<crate::state::AppState>();
                    let _ = state.config.set("floating_opacity", "60");
                    let _ = app.emit("floating:opacity", 60);
                }
                "op_40" => {
                    let state = app.state::<crate::state::AppState>();
                    let _ = state.config.set("floating_opacity", "40");
                    let _ = app.emit("floating:opacity", 40);
                }
                "float_click_through" => {
                    let state = app.state::<crate::state::AppState>();
                    let cur = state.config.get("floating_click_through").map(|v| v == "true").unwrap_or(false);
                    let next = !cur;
                    let _ = state.config.set("floating_click_through", if next { "true" } else { "false" });
                    if let Some(window) = app.get_webview_window(crate::tray::FLOATING_LABEL) {
                        let _ = window.set_ignore_cursor_events(next);
                    }
                    let _ = app.emit("floating:click-through", next);
                }
                "float_hide" => {
                    let state = app.state::<crate::state::AppState>();
                    let _ = state.config.set("floating_speed", "false");
                    if let Some(window) = app.get_webview_window(crate::tray::FLOATING_LABEL) {
                        let _ = window.hide();
                    }
                    let _ = app.emit("config:sync", serde_json::json!({ "key": "floating_speed", "value": "false" }));
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let app = window.app_handle();
                    let state = app.state::<crate::state::AppState>();
                    let minimize_to_tray = state
                        .config
                        .get("minimize_to_tray")
                        .map(|v| v == "true")
                        .unwrap_or(true);

                    if minimize_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        app.exit(0);
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running NetTamer");
}
