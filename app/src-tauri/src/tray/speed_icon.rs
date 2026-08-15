//! Generates dynamic 32x32 RGBA system tray speed icons with bold high-contrast font.
#![allow(dead_code)]

const WIDTH: usize = 32;
const HEIGHT: usize = 32;

/// 5x10 bold bitmap font for maximum taskbar legibility.
fn get_glyph_10(c: char) -> Option<&'static [u8; 10]> {
    match c {
        '0' => Some(&[
            0b01110, 0b11111, 0b11011, 0b11011, 0b11011, 0b11011, 0b11011, 0b11011, 0b11111,
            0b01110,
        ]),
        '1' => Some(&[
            0b00110, 0b01110, 0b11110, 0b00110, 0b00110, 0b00110, 0b00110, 0b00110, 0b11111,
            0b11111,
        ]),
        '2' => Some(&[
            0b01110, 0b11111, 0b11011, 0b00011, 0b00110, 0b01100, 0b11000, 0b11000, 0b11111,
            0b11111,
        ]),
        '3' => Some(&[
            0b11111, 0b11111, 0b00011, 0b00110, 0b01110, 0b00011, 0b00011, 0b11011, 0b11111,
            0b01110,
        ]),
        '4' => Some(&[
            0b00110, 0b01110, 0b11110, 0b10110, 0b11111, 0b11111, 0b00110, 0b00110, 0b00110,
            0b00110,
        ]),
        '5' => Some(&[
            0b11111, 0b11111, 0b11000, 0b11110, 0b11111, 0b00011, 0b00011, 0b11011, 0b11111,
            0b01110,
        ]),
        '6' => Some(&[
            0b01110, 0b11111, 0b11000, 0b11110, 0b11111, 0b11011, 0b11011, 0b11011, 0b11111,
            0b01110,
        ]),
        '7' => Some(&[
            0b11111, 0b11111, 0b00011, 0b00110, 0b00110, 0b01100, 0b01100, 0b11000, 0b11000,
            0b11000,
        ]),
        '8' => Some(&[
            0b01110, 0b11111, 0b11011, 0b11111, 0b01110, 0b11111, 0b11011, 0b11011, 0b11111,
            0b01110,
        ]),
        '9' => Some(&[
            0b01110, 0b11111, 0b11011, 0b11011, 0b11111, 0b01111, 0b00011, 0b00011, 0b11111,
            0b01110,
        ]),
        '.' => Some(&[
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100,
            0b01100,
        ]),
        'K' | 'k' => Some(&[
            0b11011, 0b11011, 0b11110, 0b11100, 0b11110, 0b11011, 0b11011, 0b11011, 0b11011,
            0b11011,
        ]),
        'M' | 'm' => Some(&[
            0b11011, 0b11111, 0b11111, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001,
            0b10001,
        ]),
        'G' | 'g' => Some(&[
            0b01110, 0b11111, 0b11000, 0b11000, 0b11011, 0b11011, 0b11011, 0b11011, 0b11111,
            0b01110,
        ]),
        '↑' => Some(&[
            0b00100, 0b01110, 0b11111, 0b11111, 0b01110, 0b01110, 0b01110, 0b01110, 0b01110,
            0b01110,
        ]),
        '↓' => Some(&[
            0b01110, 0b01110, 0b01110, 0b01110, 0b01110, 0b01110, 0b11111, 0b11111, 0b01110,
            0b00100,
        ]),
        ' ' => Some(&[
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
            0b00000,
        ]),
        _ => None,
    }
}

pub fn format_compact_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec <= 0.0 {
        "0K".to_string()
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        let kb = bytes_per_sec / 1024.0;
        if kb < 10.0 {
            format!("{:.1}K", kb) // "0.4K", "9.8K"
        } else if kb < 100.0 {
            format!("{:.0}K", kb) // "12K", "99K"
        } else {
            format!("{:.0}K", kb) // "350K"
        }
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        let mb = bytes_per_sec / (1024.0 * 1024.0);
        if mb < 10.0 {
            format!("{:.1}M", mb) // "1.2M", "8.5M"
        } else {
            format!("{:.0}M", mb) // "15M", "120M"
        }
    } else {
        let gb = bytes_per_sec / (1024.0 * 1024.0 * 1024.0);
        format!("{:.1}G", gb)
    }
}

pub fn format_tooltip_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        format!("{:.2} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.2} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Render a 32x32 RGBA icon showing bold upload and download speed.
pub fn generate_speed_icon(up_rate: f64, down_rate: f64) -> (Vec<u8>, u32, u32) {
    let mut buf = vec![0u8; WIDTH * HEIGHT * 4];

    let up_str = format!("↑{}", format_compact_speed(up_rate));
    let down_str = format!("↓{}", format_compact_speed(down_rate));

    // Upload color: Sky Blue (#38bdf8)
    let up_color = [56, 189, 248, 255];
    // Download color: Bright Green (#4ade80)
    let down_color = [74, 222, 128, 255];
    // Outline color: Dark Black
    let outline_color = [0, 0, 0, 255];

    // Measure widths to center text
    let up_width = measure_text_width(&up_str);
    let down_width = measure_text_width(&down_str);

    let up_x = (WIDTH.saturating_sub(up_width)) / 2;
    let down_x = (WIDTH.saturating_sub(down_width)) / 2;

    draw_line(&mut buf, &up_str, up_x, 3, up_color, outline_color);
    draw_line(&mut buf, &down_str, down_x, 18, down_color, outline_color);

    (buf, WIDTH as u32, HEIGHT as u32)
}

fn measure_text_width(text: &str) -> usize {
    let mut w: usize = 0;
    for c in text.chars() {
        if c == '.' {
            w += 3;
        } else {
            w += 6;
        }
    }
    w.saturating_sub(1)
}

fn draw_line(
    buf: &mut [u8],
    text: &str,
    start_x: usize,
    start_y: usize,
    fg: [u8; 4],
    outline: [u8; 4],
) {
    let mut cur_x = start_x;
    for c in text.chars() {
        if let Some(glyph) = get_glyph_10(c) {
            let char_w = if c == '.' { 3 } else { 5 };
            // First pass: 1px outline all around
            for row in 0..10 {
                let row_bits = glyph[row];
                for col in 0..char_w {
                    if (row_bits & (1 << (4 - col))) != 0 {
                        let px = cur_x + col;
                        let py = start_y + row;
                        set_pixel_safe(buf, px.wrapping_sub(1), py, outline);
                        set_pixel_safe(buf, px + 1, py, outline);
                        set_pixel_safe(buf, px, py.wrapping_sub(1), outline);
                        set_pixel_safe(buf, px, py + 1, outline);
                        set_pixel_safe(buf, px.wrapping_sub(1), py.wrapping_sub(1), outline);
                        set_pixel_safe(buf, px + 1, py + 1, outline);
                        set_pixel_safe(buf, px.wrapping_sub(1), py + 1, outline);
                        set_pixel_safe(buf, px + 1, py.wrapping_sub(1), outline);
                    }
                }
            }
            // Second pass: foreground
            for row in 0..10 {
                let row_bits = glyph[row];
                for col in 0..char_w {
                    if (row_bits & (1 << (4 - col))) != 0 {
                        let px = cur_x + col;
                        let py = start_y + row;
                        set_pixel_safe(buf, px, py, fg);
                    }
                }
            }
            cur_x += char_w + 1;
        } else {
            cur_x += 4;
        }
    }
}

fn set_pixel_safe(buf: &mut [u8], x: usize, y: usize, color: [u8; 4]) {
    if x < WIDTH && y < HEIGHT {
        let idx = (y * WIDTH + x) * 4;
        buf[idx] = color[0];
        buf[idx + 1] = color[1];
        buf[idx + 2] = color[2];
        buf[idx + 3] = color[3];
    }
}
