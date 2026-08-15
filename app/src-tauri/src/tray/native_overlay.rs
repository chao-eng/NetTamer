//! Pure native Win32 Taskbar Speed Overlay window.
//!
//! Uses a lightweight per-pixel alpha layered window (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE).
//! Renders speed text with GDI double-buffered DIBSection and UpdateLayeredWindow.
//! Memory footprint: < 100 KB (compared to ~40MB for a Chromium WebView2 process).
//! CPU usage: ~0.00%.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
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
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct SIZE {
    cx: i32,
    cy: i32,
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct BITMAPINFOHEADER {
    bi_size: u32,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_xpels_per_meter: i32,
    bi_ypels_per_meter: i32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

#[repr(C)]
struct BITMAPINFO {
    bmi_header: BITMAPINFOHEADER,
    bmi_colors: [u32; 1],
}

#[repr(C)]
struct BLENDFUNCTION {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}

const WS_POPUP: u32 = 0x80000000;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_TRANSPARENT: u32 = 0x00000020;
const WS_EX_LAYERED: u32 = 0x00080000;
const WS_EX_NOACTIVATE: u32 = 0x08000000;

const CS_VREDRAW: u32 = 0x0001;
const CS_HREDRAW: u32 = 0x0002;

const SW_HIDE: i32 = 0;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SWP_NOACTIVATE: u32 = 0x0010;

const GWLP_HWNDPARENT: i32 = -8;
const DIB_RGB_COLORS: u32 = 0;
const TRANSPARENT: i32 = 1;
const FW_SEMIBOLD: i32 = 600;

const DT_CENTER: u32 = 0x00000001;
const DT_VCENTER: u32 = 0x00000004;
const DT_SINGLELINE: u32 = 0x00000020;
const DT_NOPREFIX: u32 = 0x00000800;

const ULW_ALPHA: u32 = 0x00000002;
const AC_SRC_OVER: u8 = 0x00;
const AC_SRC_ALPHA: u8 = 0x01;

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
    fn DestroyWindow(hWnd: usize) -> i32;
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> usize;
    fn SetWindowLongPtrW(hWnd: usize, nIndex: i32, dwNewLong: isize) -> isize;
    fn SetWindowPos(
        hWnd: usize,
        hWndInsertAfter: usize,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
    fn GetDC(hWnd: usize) -> usize;
    fn ReleaseDC(hWnd: usize, hDC: usize) -> i32;
    fn UpdateLayeredWindow(
        hWnd: usize,
        hdcDst: usize,
        pptDst: *const POINT,
        psize: *const SIZE,
        hdcSrc: usize,
        pptSrc: *const POINT,
        crKey: u32,
        pblend: *const BLENDFUNCTION,
        dwFlags: u32,
    ) -> i32;
    fn DrawTextW(
        hdc: usize,
        lpchText: *const u16,
        cchText: i32,
        lprc: *mut RECT,
        format: u32,
    ) -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateCompatibleDC(hdc: usize) -> usize;
    fn DeleteDC(hdc: usize) -> i32;
    fn CreateDIBSection(
        hdc: usize,
        pbmi: *const BITMAPINFO,
        usage: u32,
        ppvBits: *mut *mut std::ffi::c_void,
        hSection: usize,
        offset: u32,
    ) -> usize;
    fn SelectObject(hdc: usize, h: usize) -> usize;
    fn DeleteObject(ho: usize) -> i32;
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

pub struct NativeTaskbarOverlay {
    hwnd: Option<usize>,
    is_visible: bool,
}

unsafe impl Send for NativeTaskbarOverlay {}
unsafe impl Sync for NativeTaskbarOverlay {}

static OVERLAY: Lazy<Mutex<NativeTaskbarOverlay>> = Lazy::new(|| {
    Mutex::new(NativeTaskbarOverlay {
        hwnd: None,
        is_visible: false,
    })
});

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn wnd_proc(
    hwnd: usize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl NativeTaskbarOverlay {
    fn ensure_created(&mut self) -> Option<usize> {
        if let Some(hwnd) = self.hwnd {
            return Some(hwnd);
        }

        let class_name = to_wide("NetTamerTaskbarOverlay");

        unsafe {
            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.style = CS_HREDRAW | CS_VREDRAW;
            wc.lpfn_wnd_proc = Some(wnd_proc);
            wc.lpsz_class_name = class_name.as_ptr();

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                to_wide("NetTamer Speed").as_ptr(),
                WS_POPUP,
                0,
                0,
                160,
                40,
                0,
                0,
                0,
                std::ptr::null_mut(),
            );

            if hwnd == 0 {
                return None;
            }

            // Set Taskbar (Shell_TrayWnd) as the Owner Window
            let tray_class = to_wide("Shell_TrayWnd");
            let hwnd_tray = FindWindowW(tray_class.as_ptr(), std::ptr::null());
            if hwnd_tray != 0 {
                SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, hwnd_tray as isize);
            }

            self.hwnd = Some(hwnd);
            Some(hwnd)
        }
    }
}

/// Update or render the taskbar speed overlay.
pub fn update_native_taskbar_speed(
    upload_rate: f64,
    download_rate: f64,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    is_fullscreen: bool,
) {
    let mut overlay = OVERLAY.lock().unwrap();

    if is_fullscreen {
        if let Some(hwnd) = overlay.hwnd {
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
            overlay.is_visible = false;
        }
        return;
    }

    let hwnd = match overlay.ensure_created() {
        Some(h) => h,
        None => return,
    };

    let up_str = format_speed_compact(upload_rate);
    let down_str = format_speed_compact(download_rate);
    let text = format!("↑ {}  ↓ {}", up_str, down_str);

    render_layered_text(hwnd, &text, x, y, w as i32, h as i32);
    overlay.is_visible = true;
}

/// Hide native taskbar speed overlay.
pub fn hide_native_taskbar_speed() {
    let mut overlay = OVERLAY.lock().unwrap();
    if let Some(hwnd) = overlay.hwnd {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
        overlay.is_visible = false;
    }
}

/// Destroy overlay on shutdown.
#[allow(dead_code)]
pub fn destroy_native_taskbar_speed() {
    let mut overlay = OVERLAY.lock().unwrap();
    if let Some(hwnd) = overlay.hwnd.take() {
        unsafe {
            DestroyWindow(hwnd);
        }
        overlay.is_visible = false;
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

fn render_layered_text(hwnd: usize, text: &str, x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 || h <= 0 {
        return;
    }

    unsafe {
        let hdc_screen = GetDC(0);
        let hdc_mem = CreateCompatibleDC(hdc_screen);

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmi_header.bi_width = w;
        bmi.bmi_header.bi_height = -h; // Top-down DIB
        bmi.bmi_header.bi_planes = 1;
        bmi.bmi_header.bi_bit_count = 32;
        bmi.bmi_header.bi_compression = 0; // BI_RGB

        let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbmp = CreateDIBSection(
            hdc_mem,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            0,
            0,
        );

        if hbmp == 0 || bits_ptr.is_null() {
            DeleteDC(hdc_mem);
            ReleaseDC(0, hdc_screen);
            return;
        }

        let old_bmp = SelectObject(hdc_mem, hbmp);

        // 1. Clear background to 100% transparent (ARGB: 0)
        std::ptr::write_bytes(bits_ptr, 0, (w * h * 4) as usize);

        // 2. Select sleek Segoe UI font
        let font_name = to_wide("Segoe UI Variable Text");
        let font_name_fallback = to_wide("Segoe UI");
        let mut hfont = CreateFontW(
            -12, // 9pt / 12px
            0,
            0,
            0,
            FW_SEMIBOLD,
            0,
            0,
            0,
            1, // DEFAULT_CHARSET
            4, // OUT_TT_PRECIS
            0,
            5, // CLEARTYPE_QUALITY
            0,
            font_name.as_ptr(),
        );

        if hfont == 0 {
            hfont = CreateFontW(
                -12,
                0,
                0,
                0,
                FW_SEMIBOLD,
                0,
                0,
                0,
                1,
                4,
                0,
                5,
                0,
                font_name_fallback.as_ptr(),
            );
        }

        let old_font = SelectObject(hdc_mem, hfont);
        SetBkMode(hdc_mem, TRANSPARENT);
        SetTextColor(hdc_mem, 0x00FFFFFF); // Pure crisp white

        let wide_text = to_wide(text);
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };

        DrawTextW(
            hdc_mem,
            wide_text.as_ptr(),
            (wide_text.len() - 1) as i32,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );

        // 3. Post-process Alpha channel for GDI text rendering
        let total_pixels = (w * h) as usize;
        let slice = std::slice::from_raw_parts_mut(bits_ptr as *mut u32, total_pixels);
        for pixel in slice.iter_mut() {
            let val = *pixel;
            if (val & 0x00FFFFFF) != 0 {
                // Set alpha = 255 (fully opaque crisp text)
                *pixel = val | 0xFF000000;
            }
        }

        // 4. Submit to Desktop Window Manager (DWM) with UpdateLayeredWindow
        let pt_dst = POINT { x, y };
        let size_dst = SIZE { cx: w, cy: h };
        let pt_src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            blend_op: AC_SRC_OVER,
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };

        UpdateLayeredWindow(
            hwnd,
            hdc_screen,
            &pt_dst,
            &size_dst,
            hdc_mem,
            &pt_src,
            0,
            &blend,
            ULW_ALPHA,
        );

        SetWindowPos(
            hwnd,
            0,
            x,
            y,
            w,
            h,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );

        // Cleanup GDI objects
        SelectObject(hdc_mem, old_font);
        DeleteObject(hfont);
        SelectObject(hdc_mem, old_bmp);
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(0, hdc_screen);
    }
}
