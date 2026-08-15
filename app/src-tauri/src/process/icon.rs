//! Process icon extraction and Base64 caching for Windows.
//!
//! Uses Win32 `SHGetFileInfoW` to extract High-DPI icons from executables.
//! Combines color and mask bitmaps to construct genuine 32-bit ARGB transparency,
//! eliminating black borders on legacy or 1-bit masked icons.
//! Results are encoded with BITMAPV5HEADER and cached in memory.

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
const SHGFI_LARGEICON: u32 = 0x000000000;
const SHGFI_SMALLICON: u32 = 0x000000001;
const DIB_RGB_COLORS: u32 = 0;
const BI_BITFIELDS: u32 = 3;

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

    // Cache the result
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
    // Request HD Large Icon (32x32 / 48x48) for crystal-clear High-DPI rendering
    let mut res = unsafe {
        SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut shfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if res == 0 || shfi.h_icon == 0 {
        // Fallback to small icon if large icon is unavailable
        res = unsafe {
            SHGetFileInfoW(
                wide_path.as_ptr(),
                0,
                &mut shfi,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_SMALLICON,
            )
        };
    }

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
    let color_hbm = if icon_info.hbm_color != 0 {
        icon_info.hbm_color
    } else {
        icon_info.hbm_mask
    };

    if unsafe {
        GetObjectW(
            color_hbm,
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
    let mut height = bm.bm_height;
    // If only hbm_mask is provided, height includes both AND mask and XOR mask
    if icon_info.hbm_color == 0 {
        height /= 2;
    }

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
    let mut mask_pixels = vec![0u8; pixel_count * 4];

    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmi_header.bi_width = width;
    bmi.bmi_header.bi_height = height; // bottom-up
    bmi.bmi_header.bi_planes = 1;
    bmi.bmi_header.bi_bit_count = 32;
    bmi.bmi_header.bi_compression = 0; // BI_RGB

    let mut lines = 0;
    if icon_info.hbm_color != 0 {
        lines = unsafe {
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
    }

    let mut has_mask = false;
    if icon_info.hbm_mask != 0 {
        let mut mask_bmi = bmi;
        let mask_lines = unsafe {
            GetDIBits(
                hdc,
                icon_info.hbm_mask,
                0,
                height as u32,
                mask_pixels.as_mut_ptr() as *mut std::ffi::c_void,
                &mut mask_bmi,
                DIB_RGB_COLORS,
            )
        };
        if mask_lines > 0 {
            has_mask = true;
        }
    }

    unsafe {
        if hdc != 0 { DeleteDC(hdc); }
        if icon_info.hbm_color != 0 { DeleteObject(icon_info.hbm_color); }
        if icon_info.hbm_mask != 0 { DeleteObject(icon_info.hbm_mask); }
        DestroyIcon(h_icon);
    }

    if lines == 0 && !has_mask {
        return None;
    }

    // Determine if color bitmap has real Alpha channel values (> 0 and < 255)
    let has_real_alpha = pixels.chunks(4).any(|p| p[3] > 0 && p[3] < 255);

    if has_real_alpha {
        // Modern 32-bit icon with alpha channel
        if has_mask {
            for i in 0..pixel_count {
                // If mask bit is 1 (white), pixel is transparent
                let mask_val = mask_pixels[i * 4];
                if mask_val != 0 {
                    pixels[i * 4 + 3] = 0;
                }
            }
        }
    } else {
        // Legacy icon without 32-bit alpha channel: reconstruct alpha from mask
        for i in 0..pixel_count {
            let mask_val = if has_mask { mask_pixels[i * 4] } else { 0 };
            if mask_val != 0 {
                // Transparent
                pixels[i * 4] = 0;
                pixels[i * 4 + 1] = 0;
                pixels[i * 4 + 2] = 0;
                pixels[i * 4 + 3] = 0;
            } else {
                // Opaque
                pixels[i * 4 + 3] = 255;
            }
        }
    }

    // Construct 32-bit BMP with BITMAPV5HEADER for 100% genuine alpha transparency
    let file_header_size = 14;
    let v5_header_size = 124;
    let image_size = pixels.len();
    let total_file_size = file_header_size + v5_header_size + image_size;
    let off_bits = file_header_size + v5_header_size;

    let mut bmp = Vec::with_capacity(total_file_size);

    // 1. BITMAPFILEHEADER (14 bytes)
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(total_file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes()); // reserved1
    bmp.extend_from_slice(&0u16.to_le_bytes()); // reserved2
    bmp.extend_from_slice(&(off_bits as u32).to_le_bytes());

    // 2. BITMAPV5HEADER (124 bytes)
    bmp.extend_from_slice(&(v5_header_size as u32).to_le_bytes()); // bV5Size (124)
    bmp.extend_from_slice(&width.to_le_bytes()); // bV5Width
    bmp.extend_from_slice(&height.to_le_bytes()); // bV5Height (bottom-up)
    bmp.extend_from_slice(&1u16.to_le_bytes()); // bV5Planes
    bmp.extend_from_slice(&32u16.to_le_bytes()); // bV5BitCount
    bmp.extend_from_slice(&BI_BITFIELDS.to_le_bytes()); // bV5Compression (BI_BITFIELDS = 3)
    bmp.extend_from_slice(&(image_size as u32).to_le_bytes()); // bV5SizeImage
    bmp.extend_from_slice(&0i32.to_le_bytes()); // bV5XPelsPerMeter
    bmp.extend_from_slice(&0i32.to_le_bytes()); // bV5YPelsPerMeter
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5ClrUsed
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5ClrImportant

    // Color Masks (RGBA / BGRA channel bitmasks)
    bmp.extend_from_slice(&0x00FF0000u32.to_le_bytes()); // bV5RedMask
    bmp.extend_from_slice(&0x0000FF00u32.to_le_bytes()); // bV5GreenMask
    bmp.extend_from_slice(&0x000000FFu32.to_le_bytes()); // bV5BlueMask
    bmp.extend_from_slice(&0xFF000000u32.to_le_bytes()); // bV5AlphaMask

    // Color Space & Gamma
    bmp.extend_from_slice(&0x73524742u32.to_le_bytes()); // bV5CSType ('sRGB')
    bmp.extend_from_slice(&[0u8; 36]); // bV5Endpoints (CIEXYZTRIPLE - 36 bytes zeroed)
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5GammaRed
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5GammaGreen
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5GammaBlue
    bmp.extend_from_slice(&4u32.to_le_bytes()); // bV5Intent (LCS_GM_IMAGES = 4)
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5ProfileData
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5ProfileSize
    bmp.extend_from_slice(&0u32.to_le_bytes()); // bV5Reserved

    // 3. Pixel Data (BGRA)
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
