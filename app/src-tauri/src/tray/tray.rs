use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use super::speed_icon;

pub const TRAY_ID: &str = "main-tray";
pub const FLOATING_LABEL: &str = "floating-widget";

/// Create the tray icon with right-click menu (仪表盘 / 退出) and desktop floating widget.
/// Initializes the dedicated Win32 Overlay Thread for taskbar speed widget.
pub fn setup(app: &AppHandle) -> Result<(), crate::models::Error> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| crate::models::Error::new("no default window icon available for tray"))?;

    let dashboard_item = MenuItem::with_id(app, "dashboard", "仪表盘", true, None::<&str>)
        .map_err(|e| crate::models::Error(e.to_string()))?;
    let restore_click_item = MenuItem::with_id(app, "tray_restore_click", "恢复悬浮窗可点击 (关闭穿透)", true, None::<&str>)
        .map_err(|e| crate::models::Error(e.to_string()))?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    let menu = Menu::with_items(app, &[&dashboard_item, &restore_click_item, &quit_item])
        .map_err(|e| crate::models::Error(e.to_string()))?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("NetTamer - 网络驯兽师")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app, event| {
            use tauri::Emitter;
            match event.id().as_ref() {
                "dashboard" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "tray_restore_click" => {
                    let state = app.state::<crate::state::AppState>();
                    let _ = state.config.set("floating_click_through", "false");
                    if let Some(window) = app.get_webview_window(FLOATING_LABEL) {
                        let _ = window.set_ignore_cursor_events(false);
                    }
                    let _ = app.emit("floating:click-through", false);
                    let _ = app.emit("config:sync", serde_json::json!({ "key": "floating_click_through", "value": "false" }));
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    // Create the desktop floating speed widget (draggable, always-on-top, theme-adaptive)
    let _ = WebviewWindowBuilder::new(
        app,
        FLOATING_LABEL,
        WebviewUrl::App("index.html#/floating-widget".into()),
    )
    .title("NetTamer Floating Speed")
    .inner_size(185.0, 36.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .visible(false)
    .build();

    // Start dedicated Native Win32 Overlay Thread
    #[cfg(target_os = "windows")]
    super::native_overlay::ensure_overlay_thread_started();

    Ok(())
}

/// Dynamically update system tray tooltip, native taskbar speed overlay, and desktop floating widget.
pub fn update_speed(app: &AppHandle, upload_rate: f64, download_rate: f64, taskbar_enabled: bool, floating_enabled: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = format!(
            "🐾 网络驯兽师\n↑ 上传: {}\n↓ 下载: {}",
            speed_icon::format_tooltip_speed(upload_rate),
            speed_icon::format_tooltip_speed(download_rate)
        );
        let _ = tray.set_tooltip(Some(tooltip));
    }

    #[cfg(target_os = "windows")]
    super::native_overlay::send_speed_update(upload_rate, download_rate, taskbar_enabled);

    if let Some(floating) = app.get_webview_window(FLOATING_LABEL) {
        if floating_enabled {
            if !floating.is_visible().unwrap_or(false) {
                if let Some((x, y)) = get_floating_default_position(app) {
                    let _ = floating.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                }
            }
            let _ = floating.show();
        } else {
            let _ = floating.hide();
        }
    }
}

/// Calculate default initial position in the bottom-right corner of the primary screen.
pub fn get_floating_default_position(app: &AppHandle) -> Option<(i32, i32)> {
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let scale = monitor.scale_factor();
        let size = monitor.size();
        let pos = monitor.position();

        let widget_w = (185.0 * scale) as i32;
        let widget_h = (36.0 * scale) as i32;
        let margin_x = (20.0 * scale) as i32;
        let margin_y = (65.0 * scale) as i32; // above standard Windows taskbar

        let x = pos.x + (size.width as i32) - widget_w - margin_x;
        let y = pos.y + (size.height as i32) - widget_h - margin_y;
        Some((x, y))
    } else {
        None
    }
}
