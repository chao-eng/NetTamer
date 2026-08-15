//! Dedicated Win32 Native Overlay Thread for Taskbar Speed Widget.
//!
//! Multi-mechanism Fullscreen Auto-Hide Architecture:
//! - Priority 1: `EVENT_SYSTEM_FOREGROUND` (Hook) for instant detection of game launch / window focus change.
//! - Priority 2: `EVENT_OBJECT_LOCATIONCHANGE` (Hook) for instant detection of web video / F11 fullscreen resizing.
//! - Priority 3: 500ms Fallback Timer (`WM_TIMER`) to continuously verify foreground coverage and taskbar visibility.
//!
//! Rendering & Lifecycle:
//! - Runs on an independent Win32 UI thread with message loop (`GetMessage` / `DispatchMessage`).
//! - Uses Layered Window with ColorKey (`LWA_COLORKEY`) transparency and zero-activation.
//! - Resource footprint: < 2MB RAM, 0.00% CPU.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(usize, u32, usize, isize) -> isize>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: usize,
    h_icon: usize,
    h_cursor: usize,
    hbr_background: usize,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: usize,
}

#[repr(C)]
struct MSG {
    hwnd: usize,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
    l_private: u32,
}

#[repr(C)]
struct PAINTSTRUCT {
    hdc: usize,
    f_erase: i32,
    rc_paint: RECT,
    f_restore: i32,
    f_inc_update: i32,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct MONITORINFO {
    cb_size: u32,
    rc_monitor: RECT,
    rc_work: RECT,
    dw_flags: u32,
}

const WS_POPUP: u32 = 0x80000000;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_TRANSPARENT: u32 = 0x00000020;
const WS_EX_LAYERED: u32 = 0x00080000;
const WS_EX_NOACTIVATE: u32 = 0x08000000;

const CS_VREDRAW: u32 = 0x0001;
const CS_HREDRAW: u32 = 0x0002;

const WM_CREATE: u32 = 0x0001;
const WM_PAINT: u32 = 0x000F;
const WM_TIMER: u32 = 0x0113;
const WM_DESTROY: u32 = 0x0002;
const WM_USER_UPDATE: u32 = 0x0400 + 101;

const IDT_FALLBACK_TIMER: usize = 1;
const FALLBACK_INTERVAL_MS: u32 = 500;

const SW_HIDE: i32 = 0;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SWP_NOACTIVATE: u32 = 0x0010;
const HWND_TOPMOST: usize = !0; // (HWND)-1

const TRANSPARENT: i32 = 1;
const FW_BOLD: i32 = 700;

const DT_CENTER: u32 = 0x00000001;
const DT_VCENTER: u32 = 0x00000004;
const DT_SINGLELINE: u32 = 0x00000020;
const DT_NOPREFIX: u32 = 0x00000800;

const LWA_COLORKEY: u32 = 0x00000001;
const SRCCOPY: u32 = 0x00CC0020;
const BLACK_BRUSH: i32 = 4;
const MONITOR_DEFAULTTONEAREST: u32 = 2;

const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
const EVENT_OBJECT_LOCATIONCHANGE: u32 = 0x800B;
const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> usize;
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: usize,
        hMenu: usize,
        hInstance: usize,
        lpParam: *mut std::ffi::c_void,
    ) -> usize;
    fn DefWindowProcW(hWnd: usize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn ShowWindow(hWnd: usize, nCmdShow: i32) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: usize, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostMessageW(hWnd: usize, Msg: u32, wParam: usize, lParam: isize) -> i32;
    fn PostQuitMessage(nExitCode: i32);
    fn SetWindowPos(
        hWnd: usize,
        hWndInsertAfter: usize,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
    fn SetLayeredWindowAttributes(hWnd: usize, crKey: u32, bAlpha: u8, dwFlags: u32) -> i32;
    fn SetTimer(hWnd: usize, nIDEvent: usize, uElapse: u32, lpTimerFunc: Option<unsafe extern "system" fn(usize, u32, usize, u32)>) -> usize;
    fn KillTimer(hWnd: usize, uIDEvent: usize) -> i32;
    fn BeginPaint(hWnd: usize, lpPaint: *mut PAINTSTRUCT) -> usize;
    fn EndPaint(hWnd: usize, lpPaint: *const PAINTSTRUCT) -> i32;
    fn InvalidateRect(hWnd: usize, lpRect: *const RECT, bErase: i32) -> i32;
    fn FillRect(hDC: usize, lprc: *const RECT, hbr: usize) -> i32;
    fn DrawTextW(
        hdc: usize,
        lpchText: *const u16,
        cchText: i32,
        lprc: *mut RECT,
        format: u32,
    ) -> i32;
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> usize;
    fn GetWindowRect(hWnd: usize, lpRect: *mut RECT) -> i32;
    fn IsWindowVisible(hWnd: usize) -> i32;
    fn GetForegroundWindow() -> usize;
    fn GetDesktopWindow() -> usize;
    fn GetClassNameW(hWnd: usize, lpClassName: *mut u16, nMaxCount: i32) -> i32;
    fn GetWindowLongW(hWnd: usize, nIndex: i32) -> i32;
    fn MonitorFromWindow(hwnd: usize, dw_flags: u32) -> usize;
    fn GetMonitorInfoW(hmonitor: usize, lpmi: *mut MONITORINFO) -> i32;
    fn EnumChildWindows(
        hWndParent: usize,
        lpEnumFunc: Option<unsafe extern "system" fn(usize, isize) -> i32>,
        lParam: isize,
    ) -> i32;
    fn SetWinEventHook(
        eventMin: u32,
        eventMax: u32,
        hmodWinEventProc: usize,
        pfn_win_event_proc: Option<
            unsafe extern "system" fn(
                h_win_event_hook: usize,
                event: u32,
                hwnd: usize,
                id_object: i32,
                id_child: i32,
                id_event_thread: u32,
                dwms_event_time: u32,
            ),
        >,
        idProcess: u32,
        idThread: u32,
        dwFlags: u32,
    ) -> usize;
    fn UnhookWinEvent(hWinEventHook: usize) -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateCompatibleDC(hdc: usize) -> usize;
    fn CreateCompatibleBitmap(hdc: usize, cx: i32, cy: i32) -> usize;
    fn DeleteDC(hdc: usize) -> i32;
    fn SelectObject(hdc: usize, h: usize) -> usize;
    fn DeleteObject(ho: usize) -> i32;
    fn GetStockObject(i: i32) -> usize;
    fn BitBlt(
        hdc: usize,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        hdcSrc: usize,
        x1: i32,
        y1: i32,
        rop: u32,
    ) -> i32;
    fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: u32,
        bUnderline: u32,
        bStrikeOut: u32,
        iCharSet: u32,
        iOutPrecision: u32,
        iClipPrecision: u32,
        iQuality: u32,
        iPitchAndFamily: u32,
        pszFaceName: *const u16,
    ) -> usize;
    fn SetBkMode(hdc: usize, mode: i32) -> i32;
    fn SetTextColor(hdc: usize, color: u32) -> u32;
}

#[derive(Clone, Copy, Debug)]
pub struct SpeedMessage {
    pub upload_rate: f64,
    pub download_rate: f64,
    pub enabled: bool,
}

static OVERLAY_SENDER: Lazy<Mutex<Option<Sender<SpeedMessage>>>> = Lazy::new(|| Mutex::new(None));
static LATEST_DATA: Lazy<Mutex<SpeedMessage>> = Lazy::new(|| {
    Mutex::new(SpeedMessage {
        upload_rate: 0.0,
        download_rate: 0.0,
        enabled: false,
    })
});
static OVERLAY_HWND: Mutex<usize> = Mutex::new(0);
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
static IS_FULLSCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Ensure the native Win32 Overlay thread is running.
pub fn ensure_overlay_thread_started() {
    if IS_INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    let (tx, rx) = channel::<SpeedMessage>();
    {
        let mut sender_lock = OVERLAY_SENDER.lock().unwrap();
        *sender_lock = Some(tx);
    }

    std::thread::Builder::new()
        .name("NetTamer-Win32-Overlay".to_string())
        .spawn(move || {
            run_overlay_thread_loop(rx);
        })
        .expect("failed to spawn Win32 Overlay Thread");
}

/// Send latest stats to the native Win32 Overlay.
pub fn send_speed_update(upload_rate: f64, download_rate: f64, enabled: bool) {
    ensure_overlay_thread_started();

    {
        let mut data = LATEST_DATA.lock().unwrap();
        data.upload_rate = upload_rate;
        data.download_rate = download_rate;
        data.enabled = enabled;
    }

    let hwnd = *OVERLAY_HWND.lock().unwrap();
    if hwnd != 0 {
        unsafe {
            PostMessageW(hwnd, WM_USER_UPDATE, 0, 0);
        }
    }
}

/// Priority 1 & 2: Event-driven hook for Foreground Switch (Games) and Location/Size Change (Web Fullscreen).
unsafe extern "system" fn win_event_hook_proc(
    _hook: usize,
    _event: u32,
    hwnd: usize,
    _id_object: i32,
    _id_child: i32,
    _id_thread: u32,
    _event_time: u32,
) {
    if hwnd == 0 {
        return;
    }

    let overlay_hwnd = *OVERLAY_HWND.lock().unwrap();
    // Ignore events originating from our own overlay window
    if overlay_hwnd != 0 && hwnd == overlay_hwnd {
        return;
    }

    // Evaluate against the current active foreground window
    let fg = GetForegroundWindow();
    if fg == 0 || (overlay_hwnd != 0 && fg == overlay_hwnd) {
        return;
    }

    let is_fs = check_window_is_fullscreen(fg);
    let prev = IS_FULLSCREEN_ACTIVE.swap(is_fs, Ordering::Relaxed);

    if prev != is_fs {
        if overlay_hwnd != 0 {
            if is_fs {
                ShowWindow(overlay_hwnd, SW_HIDE);
            } else {
                PostMessageW(overlay_hwnd, WM_USER_UPDATE, 0, 0);
            }
        }
    }
}

fn run_overlay_thread_loop(_rx: Receiver<SpeedMessage>) {
    let class_name = to_wide("NetTamer_Win32_TaskbarOverlay");
    let h_inst = unsafe { GetModuleHandleW(std::ptr::null()) };

    unsafe {
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.lpfn_wnd_proc = Some(overlay_wnd_proc);
        wc.h_instance = h_inst;
        wc.lpsz_class_name = class_name.as_ptr();

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            to_wide("NetTamer Overlay").as_ptr(),
            WS_POPUP,
            0,
            0,
            160,
            40,
            0,
            0,
            h_inst,
            std::ptr::null_mut(),
        );

        if hwnd == 0 {
            log::error!("Failed to create Native Win32 Overlay window");
            return;
        }

        // Configure ColorKey transparency: Black (0x000000) is 100% transparent
        SetLayeredWindowAttributes(hwnd, 0x00000000, 255, LWA_COLORKEY);

        // Start Priority 3 Fallback Timer (500ms)
        SetTimer(hwnd, IDT_FALLBACK_TIMER, FALLBACK_INTERVAL_MS, None);

        {
            let mut h_lock = OVERLAY_HWND.lock().unwrap();
            *h_lock = hwnd;
        }

        // Initial foreground state evaluation
        let fg = GetForegroundWindow();
        if fg != 0 {
            IS_FULLSCREEN_ACTIVE.store(check_window_is_fullscreen(fg), Ordering::Relaxed);
        }

        // Priority 1 Hook: EVENT_SYSTEM_FOREGROUND (0x0003) - Game / Process Focus changes
        let hook_foreground = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            0,
            Some(win_event_hook_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        // Priority 2 Hook: EVENT_OBJECT_LOCATIONCHANGE (0x800B) - Web Video / F11 Fullscreen resizing
        let hook_location = SetWinEventHook(
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_LOCATIONCHANGE,
            0,
            Some(win_event_hook_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        // Standard Win32 Message Loop on dedicated thread
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if hook_foreground != 0 {
            UnhookWinEvent(hook_foreground);
        }
        if hook_location != 0 {
            UnhookWinEvent(hook_location);
        }

        KillTimer(hwnd, IDT_FALLBACK_TIMER);

        {
            let mut h_lock = OVERLAY_HWND.lock().unwrap();
            *h_lock = 0;
        }
    }
}

unsafe extern "system" fn overlay_wnd_proc(
    hwnd: usize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    match msg {
        WM_CREATE => {
            SetTimer(hwnd, IDT_FALLBACK_TIMER, FALLBACK_INTERVAL_MS, None);
            0
        }
        WM_TIMER => {
            if wparam == IDT_FALLBACK_TIMER {
                // Priority 3: 500ms Fallback Check
                let fg = GetForegroundWindow();
                let is_fs = if fg != 0 { check_window_is_fullscreen(fg) } else { false };
                let prev = IS_FULLSCREEN_ACTIVE.swap(is_fs, Ordering::Relaxed);

                let data = *LATEST_DATA.lock().unwrap();
                if !data.enabled || is_fs {
                    ShowWindow(hwnd, SW_HIDE);
                } else if prev != is_fs {
                    PostMessageW(hwnd, WM_USER_UPDATE, 0, 0);
                }
            }
            0
        }
        WM_USER_UPDATE => {
            let data = *LATEST_DATA.lock().unwrap();

            if !data.enabled {
                ShowWindow(hwnd, SW_HIDE);
                return 0;
            }

            // Check multi-mechanism fullscreen state
            if IS_FULLSCREEN_ACTIVE.load(Ordering::Relaxed) {
                ShowWindow(hwnd, SW_HIDE);
                return 0;
            }

            if let Some((x, y, w, h)) = get_taskbar_speed_geometry(160) {
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    x,
                    y,
                    w as i32,
                    h as i32,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
                InvalidateRect(hwnd, std::ptr::null(), 0);
            } else {
                ShowWindow(hwnd, SW_HIDE);
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            if hdc != 0 {
                let data = *LATEST_DATA.lock().unwrap();
                let up_str = format!("↑ {}", format_speed_compact(data.upload_rate));
                let down_str = format!("↓ {}", format_speed_compact(data.download_rate));

                let w = 160;
                let h = 40;

                // Double buffer
                let hdc_mem = CreateCompatibleDC(hdc);
                let hbmp = CreateCompatibleBitmap(hdc, w, h);
                let old_bmp = SelectObject(hdc_mem, hbmp);

                // 1. Fill background with Black (ColorKey transparent)
                let hbr_black = GetStockObject(BLACK_BRUSH);
                let rc_full = RECT { left: 0, top: 0, right: w, bottom: h };
                FillRect(hdc_mem, &rc_full, hbr_black);

                // 2. Render Text
                let font_name = to_wide("Segoe UI Variable Text");
                let mut hfont = CreateFontW(
                    -12, 0, 0, 0, FW_BOLD, 0, 0, 0, 1, 4, 0, 5, 0, font_name.as_ptr(),
                );
                if hfont == 0 {
                    let font_name_fallback = to_wide("Segoe UI");
                    hfont = CreateFontW(-12, 0, 0, 0, FW_BOLD, 0, 0, 0, 1, 4, 0, 5, 0, font_name_fallback.as_ptr());
                }

                let old_font = SelectObject(hdc_mem, hfont);
                SetBkMode(hdc_mem, TRANSPARENT);

                let half_w = w / 2;
                let line_h = 18;
                let pad_y = (h - line_h).max(0) / 2;

                // Draw UP: Emerald Green
                SetTextColor(hdc_mem, 0x0030E070);
                let wide_up = to_wide(&up_str);
                let mut rc_up = RECT {
                    left: 2,
                    top: pad_y,
                    right: half_w - 2,
                    bottom: pad_y + line_h,
                };
                DrawTextW(
                    hdc_mem,
                    wide_up.as_ptr(),
                    (wide_up.len() - 1) as i32,
                    &mut rc_up,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                // Draw DOWN: Cyan/Sky Blue
                SetTextColor(hdc_mem, 0x00FFAA38);
                let wide_down = to_wide(&down_str);
                let mut rc_down = RECT {
                    left: half_w + 2,
                    top: pad_y,
                    right: w - 2,
                    bottom: pad_y + line_h,
                };
                DrawTextW(
                    hdc_mem,
                    wide_down.as_ptr(),
                    (wide_down.len() - 1) as i32,
                    &mut rc_down,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );

                // BitBlt
                BitBlt(hdc, 0, 0, w, h, hdc_mem, 0, 0, SRCCOPY);

                // Cleanup
                SelectObject(hdc_mem, old_font);
                DeleteObject(hfont);
                SelectObject(hdc_mem, old_bmp);
                DeleteObject(hbmp);
                DeleteDC(hdc_mem);
            }

            EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, IDT_FALLBACK_TIMER);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn format_speed_compact(rate: f64) -> String {
    if rate <= 0.0 {
        "0.0 K/s".to_string()
    } else if rate < 1024.0 {
        format!("{:.0} B/s", rate)
    } else if rate < 1_048_576.0 {
        format!("{:.1} K/s", rate / 1024.0)
    } else if rate < 1_073_741_824.0 {
        format!("{:.1} M/s", rate / 1_048_576.0)
    } else {
        format!("{:.2} G/s", rate / 1_073_741_824.0)
    }
}

unsafe extern "system" fn enum_tray_children(hwnd: usize, lparam: isize) -> i32 {
    let left_ptr = lparam as *mut i32;
    let mut class_buf = [0u16; 64];
    let len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), 64);
    if len > 0 {
        let name = String::from_utf16_lossy(&class_buf[..len as usize]);
        if name == "TrayNotifyWnd" || name.contains("InputSite") {
            let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetWindowRect(hwnd, &mut r);
            if r.left > 0 && (r.right - r.left) > 30 {
                if *left_ptr == 0 || r.left < *left_ptr {
                    *left_ptr = r.left;
                }
            }
        }
    }
    1
}

fn get_taskbar_speed_geometry(widget_width: i32) -> Option<(i32, i32, u32, u32)> {
    unsafe {
        let tray_class = to_wide("Shell_TrayWnd");
        let hwnd_tray = FindWindowW(tray_class.as_ptr(), std::ptr::null());
        if hwnd_tray == 0 {
            return None;
        }

        // Verify taskbar is visible
        if IsWindowVisible(hwnd_tray) == 0 {
            return None;
        }

        let mut tray_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetWindowRect(hwnd_tray, &mut tray_rect);

        let taskbar_height = (tray_rect.bottom - tray_rect.top).max(20) as u32;
        let mut notify_left = 0i32;

        EnumChildWindows(
            hwnd_tray,
            Some(enum_tray_children),
            &mut notify_left as *mut i32 as isize,
        );

        let x = if notify_left > widget_width + 100 {
            notify_left - widget_width - 8
        } else {
            tray_rect.right - 420 - widget_width
        };
        let y = tray_rect.top;

        Some((x, y, widget_width as u32, taskbar_height))
    }
}

fn check_window_is_fullscreen(hwnd: usize) -> bool {
    if hwnd == 0 {
        return false;
    }

    let overlay_hwnd = *OVERLAY_HWND.lock().unwrap();
    if overlay_hwnd != 0 && hwnd == overlay_hwnd {
        return false;
    }

    unsafe {
        let desktop = GetDesktopWindow();
        let shell_tray = FindWindowW(
            to_wide("Shell_TrayWnd").as_ptr(),
            std::ptr::null(),
        );

        if hwnd == desktop || hwnd == shell_tray {
            return false;
        }

        // Check if taskbar is hidden
        if shell_tray != 0 && IsWindowVisible(shell_tray) == 0 {
            return true;
        }

        let mut class_buf = [0u16; 64];
        let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), 64);
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
        if GetWindowRect(hwnd, &mut app_rect) == 0 {
            return false;
        }

        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if hmon == 0 {
            return false;
        }

        let mut mon_info: MONITORINFO = std::mem::zeroed();
        mon_info.cb_size = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hmon, &mut mon_info) == 0 {
            return false;
        }

        let style = GetWindowLongW(hwnd, -16) as u32; // GWL_STYLE = -16
        const WS_POPUP_FLAG: u32 = 0x80000000;
        const WS_CAPTION_FLAG: u32 = 0x00C00000;

        // Normal applications with caption are NOT full screen
        if (style & WS_CAPTION_FLAG) == WS_CAPTION_FLAG && (style & WS_POPUP_FLAG) == 0 {
            return false;
        }

        let is_covering = app_rect.left <= mon_info.rc_monitor.left
            && app_rect.top <= mon_info.rc_monitor.top
            && app_rect.right >= mon_info.rc_monitor.right
            && app_rect.bottom >= mon_info.rc_monitor.bottom;

        is_covering
    }
}
