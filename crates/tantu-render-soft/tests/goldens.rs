//! Golden images for the software renderer (spec `docs/specs/render-soft/renderer.md`,
//! "Golden tests"). Each test renders a reference Scene and compares it with
//! `tests/goldens/<name>.png` (tolerance 2 per channel, no differing pixels). Set
//! `TANTU_UPDATE_GOLDENS=1` to write the PNGs instead. On a mismatch the actual image is written
//! next to the golden as `<name>.actual.png` (gitignored).

mod common;

use std::path::PathBuf;

use common::*;
use tantu_core::{Affine, Color, Vec2};
use tantu_render_soft::{decode_png, diff_images, encode_png};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, ImageData, ImageDraw, ImageSampling, Layer, Resources,
    RoundedRect, Scene, SceneBuilder,
};

const TOLERANCE: u8 = 2;

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens")
        .join(format!("{name}.png"))
}

/// Renders `scene` at 2× into a 200 × 200 target and checks it against the golden `name`.
fn check_golden(name: &str, scene: &Scene, resources: &Resources) {
    let (image, report) = render_with(200, 200, 2.0, scene, resources);
    assert!(report.is_clean(), "{name}: {report:?}");
    let path = golden_path(name);
    if std::env::var_os("TANTU_UPDATE_GOLDENS").is_some_and(|v| v == "1") {
        std::fs::create_dir_all(path.parent().expect("goldens dir")).expect("create goldens dir");
        std::fs::write(&path, encode_png(&image).expect("encode")).expect("write golden");
        return;
    }
    let bytes = std::fs::read(&path).unwrap_or_else(|_| {
        panic!(
            "missing golden {}: run with TANTU_UPDATE_GOLDENS=1 to create it",
            path.display()
        )
    });
    let golden = decode_png(&bytes).expect("golden is a PNG");
    let diff = diff_images(&golden, &image, TOLERANCE).expect("golden has the same size");
    if diff.differing_pixels > 0 {
        let actual = path.with_extension("actual.png");
        std::fs::write(&actual, encode_png(&image).expect("encode")).expect("write actual");
        panic!(
            "{name}: {} pixels differ by more than {TOLERANCE} (max {}); actual written to {}",
            diff.differing_pixels,
            diff.max_channel_delta,
            actual.display()
        );
    }
}

fn golden_scene(paint: impl FnOnce(&mut SceneBuilder<'_>)) -> Scene {
    scene(100.0, 100.0, paint)
}

const RED: Color = Color::from_rgb8(220, 40, 40);
const BLUE: Color = Color::from_rgb8(40, 80, 220);
const GREEN: Color = Color::from_rgb8(40, 180, 90);

#[test]
fn golden_shapes_and_strokes() {
    let s = golden_scene(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::WHITE);
        b.fill(
            RoundedRect::new(r(8.0, 8.0, 40.0, 30.0), BorderRadius::circular(8.0)),
            RED,
        );
        b.fill(
            RoundedRect::new(
                r(55.0, 8.0, 38.0, 30.0),
                BorderRadius {
                    top_left: 0.0,
                    top_right: 15.0,
                    bottom_right: 4.0,
                    bottom_left: 30.0,
                },
            ),
            BLUE,
        );
        b.stroke(
            RoundedRect::new(r(8.0, 50.0, 40.0, 40.0), BorderRadius::circular(12.0)),
            3.0,
            GREEN,
        );
        b.stroke(
            RoundedRect::from_rect(r(55.5, 50.5, 37.0, 37.0)),
            1.0,
            Color::BLACK,
        );
        b.fill(
            RoundedRect::new(r(62.0, 57.0, 24.0, 12.0), BorderRadius::circular(100.0)),
            RED.with_alpha(0.5),
        );
    });
    check_golden("shapes_and_strokes", &s, &Resources::new());
}

#[test]
fn golden_shadows() {
    let s = golden_scene(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::from_rgb8(240, 240, 240));
        for (i, blur) in [0.0f32, 3.0, 10.0].into_iter().enumerate() {
            let shape = RoundedRect::new(
                r(10.0 + i as f32 * 30.0, 15.0, 22.0, 22.0),
                BorderRadius::circular(4.0),
            );
            b.box_shadow(BoxShadow {
                shape,
                color: Color::BLACK.with_alpha(0.5),
                offset: Vec2::new(2.0, 4.0),
                blur_radius: blur,
                spread_radius: 1.0,
            });
            b.fill(shape, Color::WHITE);
        }
        b.push_transform(Affine::translate(Vec2::new(50.0, 70.0)) * Affine::rotate(0.4));
        let shape = RoundedRect::new(r(-20.0, -10.0, 40.0, 20.0), BorderRadius::circular(6.0));
        b.box_shadow(BoxShadow {
            shape,
            color: BLUE.with_alpha(0.6),
            offset: Vec2::ZERO,
            blur_radius: 6.0,
            spread_radius: 0.0,
        });
        b.fill(shape, Color::WHITE);
        b.pop();
    });
    check_golden("shadows", &s, &Resources::new());
}

#[test]
fn golden_images() {
    let mut res = Resources::new();
    let pixels: Vec<u8> = (0..16u8)
        .flat_map(|i| [i * 16, 255 - i * 16, (i % 4) * 80, 255 - (i / 4) * 40])
        .collect();
    let id = res.add_image(ImageData::rgba8(4, 4, pixels).expect("4×4 RGBA8"));
    let s = golden_scene(|b| {
        b.image(ImageDraw {
            image: id,
            src: None,
            dest: r(5.0, 5.0, 40.0, 40.0),
            sampling: ImageSampling::Nearest,
            opacity: 1.0,
        });
        b.image(ImageDraw {
            image: id,
            src: None,
            dest: r(55.0, 5.0, 40.0, 40.0),
            sampling: ImageSampling::Linear,
            opacity: 1.0,
        });
        b.image(ImageDraw {
            image: id,
            src: Some(r(1.0, 1.0, 2.0, 2.0)),
            dest: r(5.0, 55.0, 40.0, 40.0),
            sampling: ImageSampling::Nearest,
            opacity: 0.6,
        });
        b.push_transform(Affine::translate(Vec2::new(75.0, 75.0)) * Affine::rotate(0.5));
        b.image(ImageDraw {
            image: id,
            src: None,
            dest: r(-15.0, -15.0, 30.0, 30.0),
            sampling: ImageSampling::Linear,
            opacity: 1.0,
        });
        b.pop();
    });
    check_golden("images", &s, &res);
}

#[test]
fn golden_clips_and_transforms() {
    let s = golden_scene(|b| {
        b.push_clip(Clip::RoundedRect(RoundedRect::new(
            r(5.0, 5.0, 90.0, 90.0),
            BorderRadius::circular(20.0),
        )));
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::from_rgb8(250, 230, 200));
        b.push_transform(Affine::translate(Vec2::new(50.0, 50.0)));
        for i in 0..6 {
            b.push_transform(Affine::rotate(i as f32 * std::f32::consts::FRAC_PI_6));
            b.push_clip(Clip::Rect(r(0.0, -4.0, 60.0, 8.0)));
            b.fill_rect(
                r(5.0, -10.0, 50.0, 20.0),
                if i % 2 == 0 { RED } else { BLUE },
            );
            b.pop();
            b.pop();
        }
        b.push_transform(Affine::scale_non_uniform(2.0, 0.5));
        b.fill(
            RoundedRect::new(r(-8.0, -8.0, 16.0, 16.0), BorderRadius::circular(8.0)),
            GREEN,
        );
        b.pop();
        b.pop();
        b.pop();
    });
    check_golden("clips_and_transforms", &s, &Resources::new());
}

#[test]
fn golden_layers() {
    let s = golden_scene(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::WHITE);
        b.push_layer(Layer {
            opacity: 0.5,
            overlay_color: None,
        });
        b.fill(
            RoundedRect::new(r(10.0, 10.0, 50.0, 50.0), BorderRadius::circular(10.0)),
            RED,
        );
        b.fill(
            RoundedRect::new(r(35.0, 35.0, 50.0, 50.0), BorderRadius::circular(25.0)),
            BLUE,
        );
        b.pop();
        b.push_layer(Layer {
            opacity: 1.0,
            overlay_color: Some(Color::from_rgb8(255, 200, 0).with_alpha(0.6)),
        });
        b.fill(
            RoundedRect::new(r(60.0, 5.0, 35.0, 25.0), BorderRadius::circular(5.0)),
            GREEN,
        );
        b.push_layer(Layer {
            opacity: 0.5,
            overlay_color: None,
        });
        b.fill_rect(r(70.0, 15.0, 25.0, 25.0), BLUE);
        b.pop();
        b.pop();
    });
    check_golden("layers", &s, &Resources::new());
}
