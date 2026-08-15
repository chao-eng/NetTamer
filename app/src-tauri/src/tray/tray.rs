use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use super::speed_icon;

pub const TRAY_ID: &str = "main-tray";
pub const FLOATING_LABEL: &str = "floating-widget";
pub const WIDGET_WIDTH: i32 = 160;

/// Create the tray icon with right-click menu (仪表盘 / 退出), taskbar speed widget, and desktop floating widget.
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
    {
        if taskbar_enabled {
            let is_fullscreen = is_fullscreen_running();
            if let Some((x, y, w, h)) = get_taskbar_speed_geometry(WIDGET_WIDTH) {
                super::native_overlay::update_native_taskbar_speed(
                    upload_rate,
                    download_rate,
                    x,
                    y,
                    w,
                    h,
                    is_fullscreen,
                );
            }
        } else {
            super::native_overlay::hide_native_taskbar_speed();
        }
    }

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

/// Calculate the screen position and exact full taskbar height from Windows OS.
#[cfg(target_os = "windows")]
pub fn get_taskbar_speed_geometry(widget_width: i32) -> Option<(i32, i32, u32, u32)> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowExW, FindWindowW, GetWindowRect};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }

    unsafe {
        let tray_class = to_wide("Shell_TrayWnd");
        let notify_class = to_wide("TrayNotifyWnd");

        let hwnd_tray = FindWindowW(tray_class.as_ptr(), std::ptr::null());
        if hwnd_tray == 0 {
            return None;
        }

        let mut tray_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetWindowRect(hwnd_tray, &mut tray_rect);

        let taskbar_height = (tray_rect.bottom - tray_rect.top).max(20) as u32;

        let hwnd_notify = FindWindowExW(hwnd_tray, 0, notify_class.as_ptr(), std::ptr::null());
        let mut notify_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if hwnd_notify != 0 {
            GetWindowRect(hwnd_notify, &mut notify_rect);
        } else {
            notify_rect.left = tray_rect.right - 140;
            notify_rect.top = tray_rect.top;
            notify_rect.bottom = tray_rect.bottom;
        }

        // Place widget immediately to the left of the tray icons area, full taskbar height
        let x = notify_rect.left - widget_width - 8;
        let y = tray_rect.top;

        Some((x, y, widget_width as u32, taskbar_height))
    }
}

/// Detect if any application is currently running in full screen (e.g. video players, games).
#[cfg(target_os = "windows")]
pub fn is_fullscreen_running() -> bool {
    use windows_sys::Win32::Foundation::{HWND, RECT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetClassNameW, GetDesktopWindow, GetForegroundWindow, GetWindowRect,
    };

    #[repr(C)]
    struct MONITORINFO {
        cb_size: u32,
        rc_monitor: RECT,
        rc_work: RECT,
        dw_flags: u32,
    }

    const MONITOR_DEFAULTTONEAREST: u32 = 2;

    #[link(name = "user32")]
    extern "system" {
        fn MonitorFromWindow(hwnd: HWND, dw_flags: u32) -> usize;
        fn GetMonitorInfoW(hmonitor: usize, lpmi: *mut MONITORINFO) -> i32;
    }

    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if fg_hwnd == 0 {
            return false;
        }

        let desktop = GetDesktopWindow();
        let shell_tray = FindWindowW(
            [83, 104, 101, 108, 108, 95, 84, 114, 97, 121, 87, 110, 100, 0].as_ptr(), // "Shell_TrayWnd"
            std::ptr::null(),
        );

        if fg_hwnd == desktop || fg_hwnd == shell_tray {
            return false;
        }

        let mut class_buf = [0u16; 64];
        let class_len = GetClassNameW(fg_hwnd, class_buf.as_mut_ptr(), 64);
        if class_len > 0 {
            let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);
            if class_name == "Progman"
                || class_name == "WorkerW"
                || class_name == "Shell_TrayWnd"
                || class_name == "Shell_SecondaryTrayWnd"
            {
                return false;
            }
        }

        let mut app_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if GetWindowRect(fg_hwnd, &mut app_rect) == 0 {
            return false;
        }

        let hmon = MonitorFromWindow(fg_hwnd, MONITOR_DEFAULTTONEAREST);
        if hmon == 0 {
            return false;
        }

        let mut mon_info: MONITORINFO = std::mem::zeroed();
        mon_info.cb_size = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hmon, &mut mon_info) == 0 {
            return false;
        }

        // Check if the foreground window bounds completely cover or exceed the monitor screen
        let is_covering = app_rect.left <= mon_info.rc_monitor.left
            && app_rect.top <= mon_info.rc_monitor.top
            && app_rect.right >= mon_info.rc_monitor.right
            && app_rect.bottom >= mon_info.rc_monitor.bottom;

        is_covering
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_taskbar_speed_geometry(_w: i32) -> Option<(i32, i32, u32, u32)> {
    None
}
