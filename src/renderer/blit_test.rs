use super::{blit_color_glyph, blit_glyph_pixels, blit_gray_glyph};

#[test]
fn blit_color_glyph_alpha_zero_skips_pixel() {
    let mut buf = vec![0xff_00_00_00u32; 10 * 10];
    let bitmap = vec![0u8; 4]; // 1×1 RGBA pixel, alpha=0
    blit_color_glyph(
        &mut buf,
        10,
        &bitmap,
        1,
        1,
        0,
        0,
        0,
        0xff_00_00_00,
        true,
        1.0,
        [0, 0, 10, 10],
    );
    assert_eq!(
        buf[0], 0xff_00_00_00,
        "alpha=0 must not overwrite the buffer"
    );
}

#[test]
fn blit_color_glyph_out_of_clip_skips_pixel() {
    let mut buf = vec![0xff_00_00_00u32; 10 * 10];
    let bitmap = vec![255u8, 0, 0, 255]; // opaque red 1×1
    // Pixel lands at (8, 8), but clip is [0, 0, 5, 5] → outside.
    blit_color_glyph(
        &mut buf,
        10,
        &bitmap,
        1,
        1,
        8,
        8,
        0,
        0xff_00_00_00,
        true,
        1.0,
        [0, 0, 5, 5],
    );
    assert_eq!(
        buf[88], 0xff_00_00_00,
        "out-of-clip pixel must not be written"
    );
}

#[test]
fn blit_color_glyph_active_pane_renders_full_brightness() {
    let mut buf = vec![0xff_00_00_00u32; 10 * 10]; // black background
    let bitmap = vec![255u8, 0, 0, 255]; // opaque red 1×1
    blit_color_glyph(
        &mut buf,
        10,
        &bitmap,
        1,
        1,
        0,
        0,
        0,
        0xff_00_00_00,
        true,
        1.0,
        [0, 0, 10, 10],
    );
    assert_eq!(
        (buf[0] >> 16) & 0xFF,
        255,
        "active pane: red channel must be 255"
    );
}

#[test]
fn blit_color_glyph_inactive_pane_dims_pixel() {
    let mut buf = vec![0xff_00_00_00u32; 10 * 10]; // black background
    let bitmap = vec![255u8, 0, 0, 255]; // opaque red 1×1
    blit_color_glyph(
        &mut buf,
        10,
        &bitmap,
        1,
        1,
        0,
        0,
        0,
        0xff_00_00_00,
        false, // inactive
        0.5,
        [0, 0, 10, 10],
    );
    let r = (buf[0] >> 16) & 0xFF;
    assert!(r < 255, "inactive pane must dim the pixel (got r={r})");
}

#[test]
fn blit_gray_glyph_alpha_zero_skips_pixel() {
    let mut buf = vec![0xff_00_ff_00u32; 10 * 10]; // green background
    let bitmap = vec![0u8]; // 1×1 pixel, alpha=0
    blit_gray_glyph(
        &mut buf,
        10,
        &bitmap,
        1,
        1,
        0,
        0,
        0,
        0xff_00_ff_00,
        0xff_ff_00_00,
        [0, 0, 10, 10],
    );
    assert_eq!(
        buf[0], 0xff_00_ff_00,
        "alpha=0 must not overwrite the buffer"
    );
}

#[test]
fn blit_gray_glyph_opaque_blends_to_fg() {
    let mut buf = vec![0xff_00_00_00u32; 10 * 10]; // black background
    let bitmap = vec![255u8]; // fully opaque 1×1
    blit_gray_glyph(
        &mut buf,
        10,
        &bitmap,
        1,
        1,
        0,
        0,
        0,
        0xff_00_00_00,
        0xff_ff_ff_ff, // white fg
        [0, 0, 10, 10],
    );
    let ch = buf[0] & 0xFF;
    assert!(
        ch > 200,
        "fully opaque gray glyph must render near-fg color (got {ch})"
    );
}

#[test]
fn blit_glyph_pixels_alpha_zero_skips_pixel() {
    let mut buf = vec![0xff_aa_bb_ccu32; 10 * 10];
    let bitmap = vec![0u8]; // 1×1, alpha=0
    blit_glyph_pixels(&mut buf, 10, 10, 0, 0, 1, 1, &bitmap, 0xff_ff_ff_ff);
    assert_eq!(
        buf[0], 0xff_aa_bb_cc,
        "alpha=0 must not overwrite the buffer"
    );
}

#[test]
fn blit_glyph_pixels_out_of_bounds_skips_pixel() {
    let mut buf = vec![0xff_00_00_00u32; 5 * 5];
    let bitmap = vec![255u8]; // 1×1, fully opaque
    // Try to write at (10, 10) on a 5×5 buffer → out of bounds.
    blit_glyph_pixels(&mut buf, 5, 5, 10, 10, 1, 1, &bitmap, 0xff_ff_ff_ff);
    // Buffer must be unchanged.
    assert!(buf.iter().all(|&p| p == 0xff_00_00_00));
}

#[test]
fn blit_glyph_pixels_opaque_writes_blended_pixel() {
    let mut buf = vec![0xff_00_00_00u32; 10 * 10]; // black background
    let bitmap = vec![255u8]; // fully opaque 1×1
    blit_glyph_pixels(&mut buf, 10, 10, 0, 0, 1, 1, &bitmap, 0xff_ff_ff_ff);
    let ch = buf[0] & 0xFF;
    assert!(
        ch > 200,
        "fully opaque pixel must blend near-white (got {ch})"
    );
}
