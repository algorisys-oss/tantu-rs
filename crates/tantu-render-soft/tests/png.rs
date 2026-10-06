//! Tests for `docs/specs/render-soft/renderer.md`, rules RENDER-SOFT-25 (PNG) and -26 (diff).

use std::error::Error;

use tantu_render_soft::{ImageDiff, PngError, decode_png, diff_images, encode_png};
use tantu_scene::ImageData;

/// A PNG of the given color type, written with the `png` crate directly.
fn raw_png(width: u32, height: u32, color: png::ColorType, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("valid header");
    writer.write_image_data(data).expect("matching data length");
    writer.finish().expect("finish");
    out
}

#[test]
fn render_soft_25_png_round_trip_and_formats() {
    // Every alpha value, with colors that premultiplication would round away.
    let pixels: Vec<u8> = (0..=255u8).flat_map(|a| [251, 7, 129, a]).collect();
    let image = ImageData::rgba8(16, 16, pixels).unwrap();
    let decoded = decode_png(&encode_png(&image).unwrap()).unwrap();
    assert_eq!(decoded, image);

    let rgb = raw_png(2, 1, png::ColorType::Rgb, &[1, 2, 3, 4, 5, 6]);
    assert_eq!(
        decode_png(&rgb).unwrap().pixels(),
        &[1, 2, 3, 255, 4, 5, 6, 255]
    );
    let gray = raw_png(2, 1, png::ColorType::Grayscale, &[10, 200]);
    assert_eq!(
        decode_png(&gray).unwrap().pixels(),
        &[10, 10, 10, 255, 200, 200, 200, 255]
    );
    let gray_alpha = raw_png(1, 1, png::ColorType::GrayscaleAlpha, &[50, 60]);
    assert_eq!(decode_png(&gray_alpha).unwrap().pixels(), &[50, 50, 50, 60]);

    for junk in [&b""[..], b"not a png", &[0x89, b'P', b'N', b'G', 0, 0]] {
        assert!(matches!(decode_png(junk), Err(PngError::Decode(_))));
    }
    let err = decode_png(b"nope").unwrap_err();
    assert!(!err.to_string().is_empty());
    assert!(err.source().is_some());
}

#[test]
fn render_soft_26_diff_images() {
    let a = ImageData::rgba8(2, 1, vec![10, 20, 30, 255, 0, 0, 0, 0]).unwrap();
    assert_eq!(diff_images(&a, &a, 0), Some(ImageDiff::default()));

    let b = ImageData::rgba8(2, 1, vec![13, 20, 30, 255, 0, 0, 0, 1]).unwrap();
    assert_eq!(
        diff_images(&a, &b, 0),
        Some(ImageDiff {
            differing_pixels: 2,
            max_channel_delta: 3
        })
    );
    assert_eq!(
        diff_images(&a, &b, 1),
        Some(ImageDiff {
            differing_pixels: 1,
            max_channel_delta: 3
        })
    );
    assert_eq!(
        diff_images(&a, &b, 3),
        Some(ImageDiff {
            differing_pixels: 0,
            max_channel_delta: 3
        })
    );
    let c = ImageData::rgba8(1, 2, vec![0; 8]).unwrap();
    assert_eq!(diff_images(&a, &c, 255), None);
}
