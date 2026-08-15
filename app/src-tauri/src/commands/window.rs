//! Window management commands.

use tauri::{AppHandle, Manager};

/// Hide the main window so the app continues running in the system tray.
#[tauri::command]
pub fn minimize_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Show and focus the main window.
#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    Ok(())
}

/// Toggle mouse cursor ignore (click-through) for the floating widget.
#[tauri::command]
pub fn set_floating_click_through(app: AppHandle, enabled: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(crate::tray::FLOATING_LABEL) {
        let _ = window.set_ignore_cursor_events(enabled);
    }
    Ok(())
}

/// Open native popup context menu for the floating widget outside webview bounds.
#[tauri::command]
pub fn show_floating_context_menu(app: AppHandle) -> Result<(), String> {
    use tauri::menu::{CheckMenuItem, ContextMenu, Menu, MenuItem, Submenu};

    let state = app.state::<crate::state::AppState>();
    let current_opacity = state.config.get("floating_opacity").unwrap_or_else(|| "100".into());
    let is_click_through = state.config.get("floating_click_through").map(|v| v == "true").unwrap_or(false);

    let dashboard_item = MenuItem::with_id(&app, "float_dashboard", "打开主面板", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    // Opacity Submenu
    let op_100 = CheckMenuItem::with_id(&app, "op_100", "100% 不透明", true, current_opacity == "100", None::<&str>)
        .map_err(|e| e.to_string())?;
    let op_80 = CheckMenuItem::with_id(&app, "op_80", "80% 轻微透", true, current_opacity == "80", None::<&str>)
        .map_err(|e| e.to_string())?;
    let op_60 = CheckMenuItem::with_id(&app, "op_60", "60% 半透明", true, current_opacity == "60", None::<&str>)
        .map_err(|e| e.to_string())?;
    let op_40 = CheckMenuItem::with_id(&app, "op_40", "40% 高透明", true, current_opacity == "40", None::<&str>)
        .map_err(|e| e.to_string())?;

    let opacity_sub = Submenu::with_items(&app, "透明度", true, &[&op_100, &op_80, &op_60, &op_40])
        .map_err(|e| e.to_string())?;

    let click_through_item = CheckMenuItem::with_id(&app, "float_click_through", "鼠标穿透模式", true, is_click_through, None::<&str>)
        .map_err(|e| e.to_string())?;

    let hide_item = MenuItem::with_id(&app, "float_hide", "隐藏悬浮窗", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(&app, &[
        &dashboard_item,
        &opacity_sub,
        &click_through_item,
        &hide_item,
    ])
    .map_err(|e| e.to_string())?;

    if let Some(widget) = app.get_webview_window(crate::tray::FLOATING_LABEL) {
        let _ = menu.popup(widget.as_ref().window());
    }

    Ok(())
}
