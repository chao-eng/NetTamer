//! Process icon extraction and Base64 caching for Windows.
//!
//! Uses Win32 `SHGetFileInfoW` to extract the embedded icon from an executable,
//! rasterizes it into a 32-bit RGBA BMP in-memory stream, and encodes it as a Base64 data URI.
//! Results are cached in a thread-safe LRU/HashMap to avoid any duplicate disk or GDI operations.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static ICON_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[repr(C)]
struct SHFILEINFOW {
    h_icon: usize,
    i_icon: i32,
    dw_attributes: u32,
    sz_display_name: [u16; 260],
    sz_type_name: [u16; 80],
}

#[repr(C)]
struct ICONINFO {
    f_icon: i32,
    x_hotspot: u32,
    y_hotspot: u32,
    hbm_mask: usize,
    hbm_color: usize,
}

#[repr(C)]
struct BITMAP {
    bm_type: i32,
    bm_width: i32,
    bm_height: i32,
    bm_width_bytes: i32,
    bm_planes: u16,
    bm_bits_pixel: u16,
    bm_bits: *mut std::ffi::c_void,
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

#[link(name = "shell32")]
extern "system" {
    fn SHGetFileInfoW(
        pszPath: *const u16,
        dwFileAttributes: u32,
        psfi: *mut SHFILEINFOW,
        cbFileInfo: u32,
        uFlags: u32,
    ) -> usize;
}

#[link(name = "user32")]
extern "system" {
    fn GetIconInfo(hIcon: usize, piconinfo: *mut ICONINFO) -> i32;
    fn DestroyIcon(hIcon: usize) -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetObjectW(h: usize, c: i32, pv: *mut std::ffi::c_void) -> i32;
    fn CreateCompatibleDC(hdc: usize) -> usize;
    fn DeleteDC(hdc: usize) -> i32;
    fn DeleteObject(ho: usize) -> i32;
    fn GetDIBits(
        hdc: usize,
        hbm: usize,
        start: u32,
        cLines: u32,
        lpvBits: *mut std::ffi::c_void,
        lpbmi: *mut BITMAPINFO,
        usage: u32,
    ) -> i32;
}

const SHGFI_ICON: u32 = 0x000000100;
const SHGFI_SMALLICON: u32 = 0x000000001;
const DIB_RGB_COLORS: u32 = 0;

/// Extract the icon of an executable as a Base64 Data URI string.
/// Cached by normalized path.
pub fn get_process_icon_b64(exe_path: &str) -> String {
    let norm_path = exe_path.trim().to_lowercase();
    if norm_path.is_empty() || norm_path.starts_with('[') {
        return String::new();
    }

    // Check cache
    {
        let cache = ICON_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&norm_path) {
            return cached.clone();
        }
    }

    let b64 = extract_icon_as_bmp_base64(exe_path).unwrap_or_default();

    // Cache the result (even if empty to prevent repeated disk queries on missing files)
    let mut cache = ICON_CACHE.lock().unwrap();
    cache.insert(norm_path, b64.clone());
    b64
}

fn extract_icon_as_bmp_base64(exe_path: &str) -> Option<String> {
    let wide_path: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut shfi: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let res = unsafe {
        SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut shfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        )
    };

    if res == 0 || shfi.h_icon == 0 {
        return None;
    }

    let h_icon = shfi.h_icon;
    let mut icon_info: ICONINFO = unsafe { std::mem::zeroed() };
    if unsafe { GetIconInfo(h_icon, &mut icon_info) } == 0 {
        unsafe { DestroyIcon(h_icon) };
        return None;
    }

    let mut bm: BITMAP = unsafe { std::mem::zeroed() };
    if unsafe {
        GetObjectW(
            icon_info.hbm_color,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut _ as *mut std::ffi::c_void,
        )
    } == 0
    {
        unsafe {
            if icon_info.hbm_color != 0 { DeleteObject(icon_info.hbm_color); }
            if icon_info.hbm_mask != 0 { DeleteObject(icon_info.hbm_mask); }
            DestroyIcon(h_icon);
        }
        return None;
    }

    let width = bm.bm_width;
    let height = bm.bm_height;
    if width <= 0 || height <= 0 || width > 256 || height > 256 {
        unsafe {
            if icon_info.hbm_color != 0 { DeleteObject(icon_info.hbm_color); }
            if icon_info.hbm_mask != 0 { DeleteObject(icon_info.hbm_mask); }
            DestroyIcon(h_icon);
        }
        return None;
    }

    let hdc = unsafe { CreateCompatibleDC(0) };
    let pixel_count = (width * height) as usize;
    let mut pixels = vec![0u8; pixel_count * 4];

    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmi_header.bi_width = width;
    bmi.bmi_header.bi_height = height; // bottom-up for standard BMP
    bmi.bmi_header.bi_planes = 1;
    bmi.bmi_header.bi_bit_count = 32;
    bmi.bmi_header.bi_compression = 0; // BI_RGB

    let lines = unsafe {
        GetDIBits(
            hdc,
            icon_info.hbm_color,
            0,
            height as u32,
            pixels.as_mut_ptr() as *mut std::ffi::c_void,
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };

    unsafe {
        if hdc != 0 { DeleteDC(hdc); }
        if icon_info.hbm_color != 0 { DeleteObject(icon_info.hbm_color); }
        if icon_info.hbm_mask != 0 { DeleteObject(icon_info.hbm_mask); }
        DestroyIcon(h_icon);
    }

    if lines == 0 {
        return None;
    }

    // Check if alpha channel is all 0 (common in legacy 32-bit icons where alpha is unset)
    let has_alpha = pixels.chunks(4).any(|p| p[3] > 0);
    if !has_alpha {
        for chunk in pixels.chunks_mut(4) {
            chunk[3] = 255;
        }
    }

    // Construct 32-bit BMP file
    let file_header_size = 14;
    let info_header_size = 40;
    let image_size = pixels.len();
    let total_file_size = file_header_size + info_header_size + image_size;

    let mut bmp = Vec::with_capacity(total_file_size);
    // BITMAPFILEHEADER
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(total_file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes()); // reserved1
    bmp.extend_from_slice(&0u16.to_le_bytes()); // reserved2
    bmp.extend_from_slice(&(54u32).to_le_bytes()); // off_bits

    // BITMAPINFOHEADER
    bmp.extend_from_slice(&(info_header_size as u32).to_le_bytes());
    bmp.extend_from_slice(&width.to_le_bytes());
    bmp.extend_from_slice(&height.to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&32u16.to_le_bytes()); // bit_count
    bmp.extend_from_slice(&0u32.to_le_bytes()); // compression (BI_RGB)
    bmp.extend_from_slice(&(image_size as u32).to_le_bytes());
    bmp.extend_from_slice(&0i32.to_le_bytes()); // x_pels
    bmp.extend_from_slice(&0i32.to_le_bytes()); // y_pels
    bmp.extend_from_slice(&0u32.to_le_bytes()); // clr_used
    bmp.extend_from_slice(&0u32.to_le_bytes()); // clr_important

    // Pixel data
    bmp.extend_from_slice(&pixels);

    let b64 = base64_encode(&bmp);
    Some(format!("data:image/bmp;base64,{}", b64))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(CHARSET[(b0 >> 2) as usize] as char);
        result.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[(b2 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
