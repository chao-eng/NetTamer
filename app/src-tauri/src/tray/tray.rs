use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use super::speed_icon;

pub const TRAY_ID: &str = "main-tray";
pub const WIDGET_LABEL: &str = "taskbar-widget";
pub const WIDGET_WIDTH: i32 = 160;

/// Create the tray icon with right-click menu (仪表盘 / 退出) and taskbar speed widget.
pub fn setup(app: &AppHandle) -> Result<(), crate::models::Error> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| crate::models::Error::new("no default window icon available for tray"))?;

    let dashboard_item = MenuItem::with_id(app, "dashboard", "仪表盘", true, None::<&str>)
        .map_err(|e| crate::models::Error(e.to_string()))?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    let menu = Menu::with_items(app, &[&dashboard_item, &quit_item])
        .map_err(|e| crate::models::Error(e.to_string()))?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("NetTamer - 网络驯兽师")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "dashboard" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| crate::models::Error(e.to_string()))?;

    // Create the taskbar speed widget transparent window (clicks pass through)
    if let Ok(widget) = WebviewWindowBuilder::new(
        app,
        WIDGET_LABEL,
        WebviewUrl::App("index.html#/taskbar-widget".into()),
    )
    .title("NetTamer Speed Widget")
    .inner_size(WIDGET_WIDTH as f64, 40.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .visible(false)
    .build() {
        let _ = widget.set_ignore_cursor_events(true);
    }

    Ok(())
}

/// Dynamically update system tray tooltip and taskbar speed widget.
pub fn update_speed(app: &AppHandle, upload_rate: f64, download_rate: f64, enabled: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = format!(
            "🐾 网络驯兽师\n↑ 上传: {}\n↓ 下载: {}",
            speed_icon::format_tooltip_speed(upload_rate),
            speed_icon::format_tooltip_speed(download_rate)
        );
        let _ = tray.set_tooltip(Some(tooltip));
    }

    if let Some(widget) = app.get_webview_window(WIDGET_LABEL) {
        if enabled {
            if let Some((x, y, w, h)) = get_taskbar_speed_geometry(WIDGET_WIDTH) {
                #[cfg(target_os = "windows")]
                pin_taskbar_widget_window(&widget, x, y, w, h);

                #[cfg(not(target_os = "windows"))]
                {
                    let _ = widget.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: w, height: h }));
                    let _ = widget.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                    let _ = widget.show();
                }
            }
        } else {
            let _ = widget.hide();
        }
    }
}

/// Pin taskbar widget permanently on top of taskbar with click-through and no-activation styles.
#[cfg(target_os = "windows")]
pub fn pin_taskbar_widget_window(widget: &tauri::WebviewWindow, x: i32, y: i32, w: u32, h: u32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOACTIVATE,
        SWP_SHOWWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    };

    if let Ok(hwnd) = widget.hwnd() {
        let hwnd_raw = hwnd.0 as windows_sys::Win32::Foundation::HWND;
        unsafe {
            let ex_style = GetWindowLongW(hwnd_raw, GWL_EXSTYLE);
            let target_style = ex_style | (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT) as i32;
            if ex_style != target_style {
                SetWindowLongW(hwnd_raw, GWL_EXSTYLE, target_style);
            }

            SetWindowPos(
                hwnd_raw,
                HWND_TOPMOST,
                x,
                y,
                w as i32,
                h as i32,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
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

#[cfg(not(target_os = "windows"))]
pub fn get_taskbar_speed_geometry(_w: i32) -> Option<(i32, i32, u32, u32)> {
    None
}
