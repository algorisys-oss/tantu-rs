//! Tests for `docs/specs/text/raster.md`, rules TEXT-RASTER-01..04 (Liberation Sans).

use std::sync::Arc;

use tantu_scene::{FontData, FontId};
use tantu_text::GlyphRasterizer;

const FONT: &[u8] = include_bytes!("fonts/LiberationSans-Regular.ttf");
/// Glyph ids in Liberation Sans.
const H: u32 = 43;
const SPACE: u32 = 3;

fn font() -> (FontId, FontData) {
    (
        FontId::from_raw(1).expect("non-zero"),
        FontData::new(FONT.to_vec(), 0).expect("non-empty"),
    )
}

#[test]
fn text_raster_01_visible_glyph() {
    let (id, data) = font();
    let mut r = GlyphRasterizer::new();
    let mask = r.mask(id, &data, H, 16.0, 0.0).expect("H has an outline");
    assert!(mask.width >= 1 && mask.height >= 1);
    assert_eq!(mask.coverage.len(), (mask.width * mask.height) as usize);
    assert!(mask.coverage.contains(&255));
    assert!(mask.coverage.iter().any(|&c| c > 0 && c < 255));
    assert!(mask.top < 0);
    assert!((mask.top + mask.height as i32).abs() <= 1);
    assert!((0..=3).contains(&mask.left), "left {}", mask.left);
}

#[test]
fn text_raster_02_sizes_and_absent_glyphs() {
    let (id, data) = font();
    let mut r = GlyphRasterizer::new();
    let small = r.mask(id, &data, H, 16.0, 0.0).expect("mask");
    let large = r.mask(id, &data, H, 32.0, 0.0).expect("mask");
    assert!((large.height as i32 - 2 * small.height as i32).abs() <= 2);
    assert!(r.mask(id, &data, SPACE, 16.0, 0.0).is_none());
    for size in [0.0, -4.0, f32::NAN, f32::INFINITY] {
        assert!(r.mask(id, &data, H, size, 0.0).is_none(), "{size}");
    }
    assert!(r.mask(id, &data, 60_000, 16.0, 0.0).is_none());
    let junk = FontData::new(vec![0u8; 64], 0).expect("non-empty");
    let junk_id = FontId::from_raw(2).expect("non-zero");
    assert!(r.mask(junk_id, &junk, H, 16.0, 0.0).is_none());
}

#[test]
fn text_raster_03_quarter_pixel_positions() {
    let (id, data) = font();
    let mut r = GlyphRasterizer::new();
    let zero = r.mask(id, &data, H, 16.0, 0.0).expect("mask");
    let same = r.mask(id, &data, H, 16.0, 0.1).expect("mask");
    let whole = r.mask(id, &data, H, 16.0, 3.0).expect("mask");
    let one = r.mask(id, &data, H, 16.0, 0.95).expect("mask");
    assert!(Arc::ptr_eq(&zero, &same));
    assert!(Arc::ptr_eq(&zero, &whole));
    assert!(Arc::ptr_eq(&zero, &one));
    let half = r.mask(id, &data, H, 16.0, 0.5).expect("mask");
    let also_half = r.mask(id, &data, H, 16.0, 7.45).expect("mask");
    assert!(Arc::ptr_eq(&half, &also_half));
    assert_ne!(zero.coverage, half.coverage);
}

#[test]
fn text_raster_04_cache() {
    let (id, data) = font();
    let mut r = GlyphRasterizer::new();
    assert!(r.is_empty());
    let a = r.mask(id, &data, H, 16.0, 0.0).expect("mask");
    let b = r.mask(id, &data, H, 16.0, 0.0).expect("mask");
    assert!(Arc::ptr_eq(&a, &b));
    r.mask(id, &data, SPACE, 16.0, 0.0);
    r.mask(id, &data, SPACE, 16.0, 0.0);
    assert_eq!(r.len(), 2);
    r.clear();
    assert!(r.is_empty());
    let c = r.mask(id, &data, H, 16.0, 0.0).expect("mask");
    assert!(!Arc::ptr_eq(&a, &c));
    assert_eq!(*a, *c);
}

#[test]
fn text_raster_05_readable() {
    let (id, data) = font();
    let mut r = GlyphRasterizer::new();
    assert!(r.readable(id, &data));
    assert!(r.readable(id, &data));
    let junk = FontData::new(vec![0u8; 64], 0).expect("non-empty");
    let junk_id = FontId::from_raw(9).expect("non-zero");
    assert!(!r.readable(junk_id, &junk));
}
