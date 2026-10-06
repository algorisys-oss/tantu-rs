//! Tests for `docs/specs/render-soft/renderer.md`, rules RENDER-SOFT-01..24.
//! PNG I/O and image diff (-25, -26) are in `tests/png.rs`; goldens in `tests/goldens.rs`.

mod common;

use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

use common::*;
use tantu_core::{Affine, Color, Point, Rect, Vec2};
use tantu_render_soft::{CustomCanvas, CustomHandler, SoftRenderer, tiny_skia};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, CustomKind, FontData, FontId, Glyph, ImageData, ImageDraw,
    ImageId, ImageSampling, Layer, RenderReport, Renderer, Resources, RoundedRect,
};

const RED: Color = Color::from_rgb8(255, 0, 0);
const BLUE: Color = Color::from_rgb8(0, 0, 255);
const NAN: f32 = f32::NAN;

fn image_draw(image: ImageId, dest: Rect) -> ImageDraw {
    ImageDraw {
        image,
        src: None,
        dest,
        sampling: ImageSampling::Nearest,
        opacity: 1.0,
    }
}

fn shadow(shape: RoundedRect) -> BoxShadow {
    BoxShadow {
        shape,
        color: Color::BLACK,
        offset: Vec2::ZERO,
        blur_radius: 0.0,
        spread_radius: 0.0,
    }
}

// ---- Target and frame --------------------------------------------------------------------

#[test]
fn render_soft_01_new_renderer() {
    let renderer = SoftRenderer::new(50, 40);
    assert_eq!(renderer.size(), (50, 40));
    assert_eq!(renderer.scale_factor(), 1.0);
    let image = renderer.snapshot().expect("non-empty");
    assert_eq!((image.width(), image.height()), (50, 40));
    assert!(is_blank(&image));
    assert!(SoftRenderer::new(0, 10).snapshot().is_none());
    assert!(SoftRenderer::new(10, 0).snapshot().is_none());
}

#[test]
fn render_soft_02_resize() {
    let mut renderer = SoftRenderer::new(10, 10);
    let s = scene(10.0, 10.0, |b| b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED));
    renderer.render(&s, &Resources::new()).unwrap();
    renderer.resize(30, 20, 2.0);
    assert_eq!(renderer.size(), (30, 20));
    assert_eq!(renderer.scale_factor(), 2.0);
    let image = renderer.snapshot().unwrap();
    assert_eq!((image.width(), image.height()), (30, 20));
    assert!(is_blank(&image));
    for bad in [0.0, -1.0, NAN, f32::INFINITY] {
        renderer.resize(30, 20, bad);
        assert_eq!(renderer.scale_factor(), 1.0, "scale {bad}");
    }
}

#[test]
fn render_soft_03_each_frame_starts_clear() {
    let mut renderer = SoftRenderer::new(100, 100);
    let res = Resources::new();
    renderer
        .render(
            &scene(100.0, 100.0, |b| {
                b.fill_rect(r(0.0, 0.0, 100.0, 100.0), RED)
            }),
            &res,
        )
        .unwrap();
    assert!(!is_blank(&renderer.snapshot().unwrap()));
    renderer.render(&scene(100.0, 100.0, |_| {}), &res).unwrap();
    assert!(is_blank(&renderer.snapshot().unwrap()));
}

#[test]
fn render_soft_04_report_counts_glyph_runs_as_missing_fonts() {
    let mut res = Resources::new();
    let font = res.add_font(FontData::new(vec![1u8], 0).unwrap());
    let missing_font = FontId::from_raw(u64::MAX).unwrap();
    let missing_image = ImageId::from_raw(u64::MAX - 1).unwrap();
    let glyph = [Glyph {
        id: 1,
        x: 0.0,
        y: 0.0,
    }];
    let s = scene(100.0, 100.0, |b| {
        b.glyph_run(font, 12.0, Color::BLACK, Point::new(10.0, 20.0), &glyph); // +1 (no text yet)
        b.glyph_run(font, 12.0, Color::BLACK, Point::new(10.0, 40.0), &glyph); // +1
        b.glyph_run(missing_font, 12.0, Color::BLACK, Point::ZERO, &glyph); // missing anyway
        b.glyph_run(font, NAN, Color::BLACK, Point::ZERO, &glyph); // invalid, not missing
        b.push_transform(Affine::new([NAN; 6]));
        b.glyph_run(font, 12.0, Color::BLACK, Point::ZERO, &glyph); // hidden: not counted
        b.pop();
        b.image(image_draw(missing_image, r(0.0, 0.0, 5.0, 5.0)));
        b.custom(CustomKind(1), r(0.0, 0.0, 5.0, 5.0), &[]);
        b.custom(CustomKind(2), r(0.0, 0.0, 5.0, 5.0), &[]);
    });
    let mut renderer = SoftRenderer::new(100, 100);
    renderer.register_custom(CustomKind(1), Noop);
    let report = renderer.render(&s, &res).unwrap();
    let mut expected = RenderReport::for_scene(&s, &res, &|k| k == CustomKind(1));
    expected.missing_fonts += 2;
    assert_eq!(report, expected);
    assert_eq!(
        report,
        RenderReport {
            missing_images: 1,
            missing_fonts: 3,
            unhandled_custom: 1,
            invalid_commands: 2
        }
    );
    // Glyph runs draw nothing.
    let text_only = scene(100.0, 100.0, |b| {
        b.glyph_run(font, 40.0, Color::BLACK, Point::new(10.0, 60.0), &glyph);
    });
    assert!(is_blank(&render_with(100, 100, 1.0, &text_only, &res).0));
}

#[test]
fn render_soft_05_zero_size_target() {
    let s = scene(100.0, 100.0, |b| {
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED);
        b.image(image_draw(
            ImageId::from_raw(u64::MAX).unwrap(),
            r(0.0, 0.0, 5.0, 5.0),
        ));
    });
    for (w, h) in [(0, 10), (10, 0)] {
        let mut renderer = SoftRenderer::new(w, h);
        assert_eq!(
            renderer.render(&s, &Resources::new()).unwrap(),
            RenderReport::default()
        );
        assert!(renderer.snapshot().is_none());
    }
}

#[test]
fn render_soft_06_scale_factor_and_clipping_to_target() {
    let s = scene(50.0, 50.0, |b| {
        b.fill_rect(r(10.0, 10.0, 20.0, 20.0), RED);
        b.fill_rect(r(40.0, 40.0, 100.0, 100.0), BLUE); // runs off the target
    });
    let (image, _) = render_with(100, 100, 2.0, &s, &Resources::new());
    let red = [255, 0, 0, 255];
    assert_px(&image, 21, 21, red, 0);
    assert_px(&image, 58, 58, red, 0);
    assert_px(&image, 18, 18, CLEAR, 0);
    assert_px(&image, 61, 61, CLEAR, 0);
    assert_px(&image, 99, 99, [0, 0, 255, 255], 0);
    assert_px(&image, 70, 10, CLEAR, 0);
}

/// A Scene using most commands.
fn busy_scene(image: ImageId) -> tantu_scene::Scene {
    scene(100.0, 100.0, |b| {
        b.box_shadow(BoxShadow {
            blur_radius: 6.0,
            offset: Vec2::new(3.0, 4.0),
            ..shadow(RoundedRect::new(
                r(10.0, 10.0, 40.0, 30.0),
                BorderRadius::circular(6.0),
            ))
        });
        b.fill(
            RoundedRect::new(r(10.0, 10.0, 40.0, 30.0), BorderRadius::circular(6.0)),
            RED,
        );
        b.push_transform(Affine::rotate(0.3));
        b.push_clip(Clip::RoundedRect(RoundedRect::new(
            r(40.0, 0.0, 50.0, 60.0),
            BorderRadius::circular(12.0),
        )));
        b.push_layer(Layer {
            opacity: 0.7,
            overlay_color: Some(Color::new(0.0, 1.0, 0.0, 0.3)),
        });
        b.stroke(RoundedRect::from_rect(r(45.0, 5.0, 40.0, 40.0)), 3.0, BLUE);
        b.image(ImageDraw {
            sampling: ImageSampling::Linear,
            ..image_draw(image, r(50.0, 20.0, 30.0, 30.0))
        });
        b.pop();
        b.pop();
        b.pop();
    })
}

fn checker_image() -> ImageData {
    ImageData::rgba8(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 128,
        ],
    )
    .unwrap()
}

#[test]
fn render_soft_07_rendering_is_deterministic() {
    let mut res = Resources::new();
    let image = res.add_image(checker_image());
    let s = busy_scene(image);
    let mut a = SoftRenderer::new(120, 120);
    a.resize(120, 120, 1.25);
    a.render(&s, &res).unwrap();
    let first = a.snapshot().unwrap();
    a.render(&s, &res).unwrap();
    assert_eq!(a.snapshot().unwrap(), first);
    let mut b = SoftRenderer::new(120, 120);
    b.resize(120, 120, 1.25);
    b.render(&s, &res).unwrap();
    assert_eq!(b.snapshot().unwrap(), first);
    assert!(!is_blank(&first));
}

// ---- Fills, blending and strokes ---------------------------------------------------------

#[test]
fn render_soft_08_fill_color() {
    let image = render100(|b| {
        b.fill_rect(r(0.0, 0.0, 50.0, 50.0), Color::from_rgba8(10, 200, 30, 255));
        b.fill_rect(
            r(50.0, 0.0, 50.0, 50.0),
            Color::from_rgba8(200, 100, 50, 128),
        );
    });
    assert_px(&image, 25, 25, [10, 200, 30, 255], 0);
    assert_px(&image, 75, 25, [200, 100, 50, 128], 1);
}

#[test]
fn render_soft_09_blending_in_encoded_srgb() {
    let image = render100(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::BLACK);
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::new(1.0, 1.0, 1.0, 0.5));
    });
    assert_px(&image, 50, 50, [128, 128, 128, 255], 1);
}

#[test]
fn render_soft_10_rounded_corners_and_radius_clamping() {
    let red = [255, 0, 0, 255];
    let image = render100(|b| {
        b.fill(
            RoundedRect::new(r(10.0, 10.0, 40.0, 40.0), BorderRadius::circular(10.0)),
            RED,
        );
        // 40 × 20 with radius 100 becomes a pill of radius 10.
        b.fill(
            RoundedRect::new(r(10.0, 60.0, 40.0, 20.0), BorderRadius::circular(100.0)),
            RED,
        );
        // Negative radii count as 0.
        b.fill(
            RoundedRect::new(r(60.0, 10.0, 30.0, 30.0), BorderRadius::circular(-5.0)),
            RED,
        );
    });
    assert_px(&image, 10, 10, CLEAR, 0);
    assert_px(&image, 30, 30, red, 0);
    assert_px(&image, 10, 60, CLEAR, 0);
    assert_px(&image, 11, 61, CLEAR, 0);
    assert_px(&image, 12, 70, red, 0);
    assert_px(&image, 30, 70, red, 0);
    assert_px(&image, 60, 10, red, 0);
    assert_px(&image, 89, 39, red, 0);
}

#[test]
fn render_soft_11_strokes_lie_inside() {
    let red = [255, 0, 0, 255];
    let image = render100(|b| {
        b.stroke(RoundedRect::from_rect(r(10.0, 10.0, 40.0, 40.0)), 4.0, RED);
        b.stroke(RoundedRect::from_rect(r(60.0, 10.0, 30.0, 30.0)), 15.0, RED); // fills
    });
    assert_px(&image, 11, 30, red, 0);
    assert_px(&image, 30, 11, red, 0);
    assert_px(&image, 48, 30, red, 0);
    assert_px(&image, 16, 30, CLEAR, 0);
    assert_px(&image, 30, 30, CLEAR, 0);
    assert_px(&image, 8, 30, CLEAR, 0);
    assert_px(&image, 75, 25, red, 0);

    let none = render100(|b| {
        b.stroke(RoundedRect::from_rect(r(10.0, 10.0, 40.0, 40.0)), 0.0, RED);
        b.stroke(RoundedRect::from_rect(r(10.0, 10.0, 40.0, 40.0)), -2.0, RED);
    });
    assert!(is_blank(&none));
}

#[test]
fn render_soft_12_empty_reversed_and_transparent_draw_nothing() {
    let image = render100(|b| {
        b.fill_rect(r(10.0, 10.0, 0.0, 20.0), RED);
        b.fill_rect(Rect::from_ltrb(50.0, 50.0, 10.0, 10.0), RED);
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::TRANSPARENT);
        b.stroke(
            RoundedRect::from_rect(Rect::from_ltrb(50.0, 50.0, 10.0, 10.0)),
            2.0,
            RED,
        );
        b.box_shadow(shadow(RoundedRect::from_rect(r(10.0, 10.0, 0.0, 0.0))));
    });
    assert!(is_blank(&image));
}

// ---- Box shadows -------------------------------------------------------------------------

#[test]
fn render_soft_13_hard_shadow_geometry() {
    let black = [0, 0, 0, 255];
    let image = render100(|b| {
        b.box_shadow(BoxShadow {
            offset: Vec2::new(10.0, 0.0),
            spread_radius: 5.0,
            ..shadow(RoundedRect::from_rect(r(20.0, 20.0, 20.0, 20.0)))
        });
    });
    // Inflated by 5 and moved by (10, 0): x 25..55, y 15..45.
    assert_px(&image, 26, 16, black, 0);
    assert_px(&image, 53, 43, black, 0);
    assert_px(&image, 23, 30, CLEAR, 0);
    assert_px(&image, 56, 30, CLEAR, 0);
    assert_px(&image, 40, 13, CLEAR, 0);

    // Radii grow with the spread: radius 10 + spread 10 = 20.
    let grown = render100(|b| {
        b.box_shadow(BoxShadow {
            spread_radius: 10.0,
            ..shadow(RoundedRect::new(
                r(30.0, 30.0, 40.0, 40.0),
                BorderRadius::circular(10.0),
            ))
        });
    });
    assert_px(&grown, 23, 23, CLEAR, 0);
    assert_px(&grown, 50, 50, black, 0);
    // A negative spread shrinks radii, but not below 0.
    let shrunk = render100(|b| {
        b.box_shadow(BoxShadow {
            spread_radius: -10.0,
            ..shadow(RoundedRect::new(
                r(30.0, 30.0, 40.0, 40.0),
                BorderRadius::circular(5.0),
            ))
        });
    });
    assert_px(&shrunk, 40, 40, black, 0);
    assert_px(&shrunk, 38, 50, CLEAR, 0);
}

#[test]
fn render_soft_14_blurred_shadow() {
    let blurred = |blur: f32| {
        move |b: &mut tantu_scene::SceneBuilder<'_>| {
            b.box_shadow(BoxShadow {
                blur_radius: blur,
                ..shadow(RoundedRect::from_rect(r(20.0, 20.0, 60.0, 60.0)))
            });
        }
    };
    let image = render100(blurred(5.0)); // sigma = 3.39, 3 sigma = 10.2
    assert_px(&image, 50, 50, [0, 0, 0, 255], 1);
    assert_px(&image, 5, 50, CLEAR, 1);
    let edge = (px(&image, 19, 50)[3] as i32 + px(&image, 20, 50)[3] as i32) / 2;
    assert!((102..=153).contains(&edge), "edge alpha {edge}");

    // At scale 2 the blur is twice as wide in physical pixels.
    let s = scene(100.0, 100.0, blurred(5.0));
    let (double, _) = render_with(200, 200, 2.0, &s, &Resources::new());
    let near1 = px(&image, 14, 50)[3];
    let near2 = px(&double, 28, 100)[3];
    assert!(near1 > 0, "6 px outside is still inside the blur");
    assert!(
        near1.abs_diff(near2) <= 4,
        "scale 1: {near1}, scale 2: {near2}"
    );

    // Negative blur is a hard edge.
    let hard = render100(blurred(-3.0));
    assert_px(&hard, 19, 50, CLEAR, 0);
    assert_px(&hard, 20, 50, [0, 0, 0, 255], 0);
}

// ---- Images ------------------------------------------------------------------------------

#[test]
fn render_soft_15_image_scaling_and_sampling() {
    let mut res = Resources::new();
    let checker = ImageData::rgba8(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ],
    )
    .unwrap();
    let id = res.add_image(checker);
    let s = scene(100.0, 100.0, |b| {
        b.image(image_draw(id, r(10.0, 10.0, 20.0, 20.0)));
        b.image(ImageDraw {
            src: Some(r(1.0, 0.0, 1.0, 1.0)),
            ..image_draw(id, r(50.0, 10.0, 20.0, 20.0))
        });
        b.image(ImageDraw {
            sampling: ImageSampling::Linear,
            ..image_draw(id, r(0.0, 50.0, 40.0, 40.0))
        });
    });
    let (image, report) = render_with(100, 100, 1.0, &s, &res);
    assert!(report.is_clean());
    for (x, y, color) in [
        (11, 11, [255, 0, 0, 255]),
        (18, 18, [255, 0, 0, 255]),
        (21, 11, [0, 255, 0, 255]),
        (28, 18, [0, 255, 0, 255]),
        (11, 21, [0, 0, 255, 255]),
        (28, 28, [255, 255, 255, 255]),
        (51, 11, [0, 255, 0, 255]),
        (68, 28, [0, 255, 0, 255]),
    ] {
        assert_px(&image, x, y, color, 1);
    }
    // Bilinear: the middle is a blend of all four source pixels.
    let mid = px(&image, 20, 70);
    for c in &mid[..3] {
        assert!((60..=200).contains(c), "blended pixel {mid:?}");
    }
}

#[test]
fn render_soft_16_image_opacity() {
    let mut res = Resources::new();
    let white = res.add_image(ImageData::rgba8(1, 1, vec![255, 255, 255, 255]).unwrap());
    let s = scene(100.0, 100.0, |b| {
        b.image(ImageDraw {
            opacity: 0.5,
            ..image_draw(white, r(0.0, 0.0, 50.0, 50.0))
        });
        b.image(ImageDraw {
            opacity: 2.0,
            ..image_draw(white, r(50.0, 0.0, 50.0, 50.0))
        });
        b.image(ImageDraw {
            opacity: -1.0,
            ..image_draw(white, r(0.0, 50.0, 50.0, 50.0))
        });
    });
    let (image, _) = render_with(100, 100, 1.0, &s, &res);
    assert_px(&image, 25, 25, [255, 255, 255, 128], 1);
    assert_px(&image, 75, 25, [255, 255, 255, 255], 0);
    assert_px(&image, 25, 75, CLEAR, 0);
}

#[test]
fn render_soft_17_missing_and_removed_images() {
    let mut res = Resources::new();
    let red = res.add_image(ImageData::rgba8(1, 1, vec![255, 0, 0, 255]).unwrap());
    let s = scene(100.0, 100.0, |b| {
        b.image(image_draw(red, r(0.0, 0.0, 50.0, 50.0)))
    });
    let mut renderer = SoftRenderer::new(100, 100);
    assert!(renderer.render(&s, &res).unwrap().is_clean());
    assert_px(&renderer.snapshot().unwrap(), 25, 25, [255, 0, 0, 255], 0);

    res.remove_image(red);
    let report = renderer.render(&s, &res).unwrap();
    assert_eq!(report.missing_images, 1);
    assert!(is_blank(&renderer.snapshot().unwrap()));

    let green = res.add_image(ImageData::rgba8(1, 1, vec![0, 255, 0, 255]).unwrap());
    let s2 = scene(100.0, 100.0, |b| {
        b.image(image_draw(green, r(0.0, 0.0, 50.0, 50.0)))
    });
    renderer.render(&s2, &res).unwrap();
    assert_px(&renderer.snapshot().unwrap(), 25, 25, [0, 255, 0, 255], 0);
}

// ---- Transforms, clips and layers --------------------------------------------------------

#[test]
fn render_soft_18_transforms() {
    let red = [255, 0, 0, 255];
    let image = render100(|b| {
        b.push_transform(Affine::translate(Vec2::new(30.0, 0.0)));
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED); // 30..40, 0..10
        b.pop();
        b.push_transform(Affine::scale(2.0));
        b.fill_rect(r(5.0, 20.0, 10.0, 5.0), RED); // 10..30, 40..50
        b.pop();
        b.push_transform(Affine::translate(Vec2::new(90.0, 0.0)));
        b.push_transform(Affine::rotate(FRAC_PI_2));
        b.fill_rect(r(0.0, 0.0, 20.0, 10.0), RED); // x 80..90, y 0..20
        b.pop();
        b.fill_rect(r(0.0, 60.0, 5.0, 5.0), RED); // outer translate only: 90..95, 60..65
        b.pop();
    });
    assert_px(&image, 35, 5, red, 0);
    assert_px(&image, 5, 5, CLEAR, 0);
    assert_px(&image, 11, 41, red, 0);
    assert_px(&image, 28, 48, red, 0);
    assert_px(&image, 9, 45, CLEAR, 0);
    assert_px(&image, 85, 15, red, 0);
    assert_px(&image, 85, 25, CLEAR, 0);
    assert_px(&image, 92, 62, red, 0);
    assert_px(&image, 2, 62, CLEAR, 0);
}

#[test]
fn render_soft_19_clips() {
    let red = [255, 0, 0, 255];
    let image = render100(|b| {
        b.push_clip(Clip::Rect(r(0.0, 0.0, 50.0, 50.0)));
        b.push_clip(Clip::Rect(r(25.0, 0.0, 75.0, 100.0)));
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), RED); // only 25..50 × 0..50
        b.pop();
        b.fill_rect(r(0.0, 0.0, 100.0, 20.0), BLUE); // 0..50 × 0..20
        b.pop();
        b.push_transform(Affine::translate(Vec2::new(60.0, 60.0)));
        b.push_clip(Clip::RoundedRect(RoundedRect::new(
            r(0.0, 0.0, 40.0, 40.0),
            BorderRadius::circular(15.0),
        )));
        b.push_transform(Affine::translate(Vec2::new(-60.0, -60.0)));
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), RED); // rounded 60..100 × 60..100
        b.pop();
        b.pop();
        b.pop();
    });
    assert_px(&image, 30, 30, red, 0);
    assert_px(&image, 10, 30, CLEAR, 0);
    assert_px(&image, 60, 30, CLEAR, 0);
    assert_px(&image, 10, 10, [0, 0, 255, 255], 0);
    assert_px(&image, 70, 10, CLEAR, 0);
    assert_px(&image, 80, 80, red, 0);
    assert_px(&image, 60, 60, CLEAR, 0);
    assert_px(&image, 99, 99, CLEAR, 0);
    assert_px(&image, 55, 80, CLEAR, 0);
}

#[test]
fn render_soft_20_layers_are_groups() {
    let image = render100(|b| {
        b.push_layer(Layer {
            opacity: 0.5,
            overlay_color: None,
        });
        b.fill_rect(r(0.0, 0.0, 60.0, 60.0), RED);
        b.fill_rect(r(40.0, 40.0, 60.0, 60.0), BLUE);
        b.pop();
    });
    assert_px(&image, 50, 50, [0, 0, 255, 128], 1);
    assert_px(&image, 10, 10, [255, 0, 0, 128], 1);
    assert_px(&image, 90, 10, CLEAR, 0);

    let nan = render100(|b| {
        b.push_layer(Layer {
            opacity: NAN,
            overlay_color: None,
        });
        b.fill_rect(r(0.0, 0.0, 60.0, 60.0), RED);
        b.pop();
    });
    assert!(is_blank(&nan));
    let over = render100(|b| {
        b.push_layer(Layer {
            opacity: 3.0,
            overlay_color: None,
        });
        b.fill_rect(r(0.0, 0.0, 60.0, 60.0), RED);
        b.pop();
    });
    assert_px(&over, 10, 10, [255, 0, 0, 255], 0);
}

#[test]
fn render_soft_21_overlay_color() {
    let image = render100(|b| {
        b.push_layer(Layer {
            opacity: 1.0,
            overlay_color: Some(BLUE),
        });
        b.fill_rect(r(10.0, 10.0, 30.0, 30.0), RED);
        b.pop();
        b.push_layer(Layer {
            opacity: 1.0,
            overlay_color: Some(BLUE.with_alpha(0.5)),
        });
        b.fill_rect(r(60.0, 10.0, 30.0, 30.0), RED);
        b.pop();
        b.push_layer(Layer {
            opacity: 1.0,
            overlay_color: Some(Color::new(NAN, 0.0, 1.0, 1.0)),
        });
        b.fill_rect(r(10.0, 60.0, 30.0, 30.0), RED);
        b.pop();
    });
    assert_px(&image, 20, 20, [0, 0, 255, 255], 0);
    assert_px(&image, 50, 20, CLEAR, 0); // left transparent by the group
    assert_px(&image, 75, 25, [128, 0, 128, 255], 1);
    assert_px(&image, 25, 75, [255, 0, 0, 255], 0); // non-finite overlay ignored
}

// ---- Custom commands and invalid values --------------------------------------------------

/// What a recording handler saw.
#[derive(Clone, Debug, PartialEq)]
struct Call {
    bounds: Rect,
    data: Vec<u8>,
    transform: tiny_skia::Transform,
    clipped: bool,
}

/// Records each call and fills the bounds with green.
struct Recorder(Rc<RefCell<Vec<Call>>>);

impl CustomHandler for Recorder {
    fn draw(&mut self, mut canvas: CustomCanvas<'_>) {
        self.0.borrow_mut().push(Call {
            bounds: canvas.bounds,
            data: canvas.data.to_vec(),
            transform: canvas.transform,
            clipped: canvas.clip.is_some(),
        });
        let b = canvas.bounds;
        if let Some(rect) = tiny_skia::Rect::from_ltrb(b.left, b.top, b.right, b.bottom) {
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(0, 255, 0, 255);
            canvas
                .pixmap
                .fill_rect(rect, &paint, canvas.transform, canvas.clip);
        }
    }
}

/// Draws nothing.
struct Noop;

impl CustomHandler for Noop {
    fn draw(&mut self, _: CustomCanvas<'_>) {}
}

#[test]
fn render_soft_22_custom_handlers() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut renderer = SoftRenderer::new(200, 200);
    renderer.resize(200, 200, 2.0);
    renderer.register_custom(CustomKind(1), Recorder(calls.clone()));
    let s = scene(100.0, 100.0, |b| {
        b.push_transform(Affine::translate(Vec2::new(10.0, 0.0)));
        b.custom(CustomKind(1), r(0.0, 0.0, 10.0, 10.0), &[1]);
        b.pop();
        b.push_clip(Clip::Rect(r(0.0, 0.0, 50.0, 50.0)));
        b.custom(CustomKind(1), r(20.0, 20.0, 10.0, 10.0), &[2, 3]);
        b.pop();
        b.custom(CustomKind(2), r(60.0, 60.0, 10.0, 10.0), &[4]);
    });
    let report = renderer.render(&s, &Resources::new()).unwrap();
    assert_eq!(report.unhandled_custom, 1);
    assert_eq!(
        *calls.borrow(),
        [
            Call {
                bounds: r(0.0, 0.0, 10.0, 10.0),
                data: vec![1],
                transform: tiny_skia::Transform::from_row(2.0, 0.0, 0.0, 2.0, 20.0, 0.0),
                clipped: false
            },
            Call {
                bounds: r(20.0, 20.0, 10.0, 10.0),
                data: vec![2, 3],
                transform: tiny_skia::Transform::from_scale(2.0, 2.0),
                clipped: true
            },
        ]
    );
    let image = renderer.snapshot().unwrap();
    assert_px(&image, 25, 5, [0, 255, 0, 255], 0);
    assert_px(&image, 50, 50, [0, 255, 0, 255], 0);
    assert_px(&image, 130, 130, CLEAR, 0);

    // A later registration replaces the handler.
    let later = Rc::new(RefCell::new(Vec::new()));
    renderer.register_custom(CustomKind(1), Recorder(later.clone()));
    renderer.render(&s, &Resources::new()).unwrap();
    assert_eq!(calls.borrow().len(), 2);
    assert_eq!(later.borrow().len(), 2);
}

#[test]
fn render_soft_23_invalid_values_draw_nothing() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut renderer = SoftRenderer::new(100, 100);
    renderer.register_custom(CustomKind(1), Recorder(calls.clone()));
    let s = scene(100.0, 100.0, |b| {
        b.fill_rect(Rect::from_ltrb(NAN, 0.0, 50.0, 50.0), RED);
        b.fill_rect(r(0.0, 0.0, 50.0, 50.0), Color::new(1.0, 0.0, 0.0, NAN));
        b.push_transform(Affine::new([1.0, 0.0, 0.0, 1.0, NAN, 0.0]));
        b.fill_rect(r(0.0, 0.0, 50.0, 50.0), RED);
        b.custom(CustomKind(1), r(0.0, 0.0, 10.0, 10.0), &[]);
        b.pop();
        b.push_clip(Clip::Rect(Rect::from_ltrb(0.0, 0.0, f32::INFINITY, 50.0)));
        b.fill_rect(r(0.0, 0.0, 50.0, 50.0), RED);
        b.pop();
        b.fill_rect(r(60.0, 60.0, 10.0, 10.0), BLUE);
    });
    let report = renderer.render(&s, &Resources::new()).unwrap();
    assert_eq!(report.invalid_commands, 4);
    assert!(calls.borrow().is_empty());
    let image = renderer.snapshot().unwrap();
    assert_px(&image, 25, 25, CLEAR, 0);
    assert_px(&image, 65, 65, [0, 0, 255, 255], 0);
}

#[test]
fn render_soft_24_never_panics() {
    let mut res = Resources::new();
    let big = res.add_image(ImageData::rgba8(2000, 1500, vec![200u8; 2000 * 1500 * 4]).unwrap());
    let huge = 1e30_f32;
    let s = scene(100.0, 100.0, |b| {
        b.fill_rect(Rect::from_ltrb(-huge, -huge, huge, huge), RED);
        b.fill_rect(r(10.0, 10.0, 1e-30, 1e-30), RED);
        b.fill(
            RoundedRect::new(r(0.0, 0.0, 50.0, 50.0), BorderRadius::circular(huge)),
            RED,
        );
        b.stroke(RoundedRect::from_rect(r(0.0, 0.0, 50.0, 50.0)), huge, RED);
        b.box_shadow(BoxShadow {
            blur_radius: 1e9,
            spread_radius: huge,
            ..shadow(RoundedRect::from_rect(r(0.0, 0.0, 10.0, 10.0)))
        });
        b.box_shadow(BoxShadow {
            blur_radius: 1e9,
            ..shadow(RoundedRect::from_rect(r(0.0, 0.0, 10.0, 10.0)))
        });
        b.image(image_draw(big, r(0.0, 0.0, 30.0, 30.0)));
        b.image(ImageDraw {
            src: Some(r(-50.0, -50.0, 1e9, 3.0)),
            ..image_draw(big, r(0.0, 0.0, 30.0, 30.0))
        });
        b.push_transform(Affine::scale(1e-30));
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED);
        b.pop();
        b.push_transform(Affine::scale(huge));
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED);
        b.box_shadow(BoxShadow {
            blur_radius: 3.0,
            ..shadow(RoundedRect::from_rect(r(0.0, 0.0, 10.0, 10.0)))
        });
        b.pop();
        for i in 0..64 {
            b.push_layer(Layer {
                opacity: 0.9,
                overlay_color: None,
            });
            b.push_clip(Clip::Rect(r(i as f32, 0.0, 100.0, 100.0)));
        }
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), BLUE);
        for _ in 0..128 {
            b.pop();
        }
    });
    let mut renderer = SoftRenderer::new(100, 100);
    renderer.resize(100, 100, 3.0);
    renderer.render(&s, &res).unwrap();
    renderer.resize(1, 1, 0.01);
    renderer.render(&s, &res).unwrap();
}
