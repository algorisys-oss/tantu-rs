//! Tests for `docs/specs/render-wgpu/renderer.md`, rules RENDER-WGPU-01..17. The pixel probes are
//! the software renderer's (`crates/tantu-render-soft/tests/rules.rs`), so both backends are
//! held to the same expectations. Each test is skipped when there is no GPU adapter (see
//! `common::gpu`).

mod common;

use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

use common::*;
use tantu_core::{Affine, Color, Point, Rect, Vec2};
use tantu_render_wgpu::{CustomCanvas, CustomHandler};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, CustomKind, FontData, FontId, Glyph, ImageData, ImageDraw,
    ImageId, ImageSampling, Layer, RenderReport, Renderer, Resources, RoundedRect, SceneBuilder,
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
fn render_wgpu_01_new_offscreen() {
    let Some(renderer) = gpu(50, 40) else { return };
    assert_eq!(renderer.size(), (50, 40));
    assert_eq!(renderer.scale_factor(), 1.0);
    let image = renderer.snapshot().expect("offscreen");
    assert_eq!((image.width(), image.height()), (50, 40));
    assert!(is_blank(&image));
    let empty = gpu(0, 10).expect("an adapter was found above");
    assert!(empty.snapshot().is_none());
    assert!(!renderer.adapter_info().name.is_empty() || renderer.adapter_info().vendor == 0);
}

#[test]
fn render_wgpu_02_resize() {
    let Some(mut renderer) = gpu(10, 10) else {
        return;
    };
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
fn render_wgpu_03_each_frame_starts_clear() {
    let Some(mut renderer) = gpu(100, 100) else {
        return;
    };
    let res = Resources::new();
    let full = scene(100.0, 100.0, |b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), RED)
    });
    renderer.render(&full, &res).unwrap();
    assert!(!is_blank(&renderer.snapshot().unwrap()));
    renderer.render(&scene(100.0, 100.0, |_| {}), &res).unwrap();
    assert!(is_blank(&renderer.snapshot().unwrap()));
}

/// Draws nothing.
struct Noop;

impl CustomHandler for Noop {
    fn draw(&mut self, _: CustomCanvas<'_>) {}
}

#[test]
fn render_wgpu_04_report_and_zero_size() {
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
        b.glyph_run(font, 12.0, Color::BLACK, Point::new(10.0, 20.0), &glyph);
        b.glyph_run(font, 12.0, Color::BLACK, Point::new(10.0, 40.0), &glyph);
        b.glyph_run(missing_font, 12.0, Color::BLACK, Point::ZERO, &glyph);
        b.glyph_run(font, NAN, Color::BLACK, Point::ZERO, &glyph);
        b.push_transform(Affine::new([NAN; 6]));
        b.glyph_run(font, 12.0, Color::BLACK, Point::ZERO, &glyph);
        b.pop();
        b.image(image_draw(missing_image, r(0.0, 0.0, 5.0, 5.0)));
        b.custom(CustomKind(1), r(0.0, 0.0, 5.0, 5.0), &[]);
        b.custom(CustomKind(2), r(0.0, 0.0, 5.0, 5.0), &[]);
    });
    let Some(mut renderer) = gpu(100, 100) else {
        return;
    };
    renderer.register_custom(CustomKind(1), Noop);
    let report = renderer.render(&s, &res).unwrap();
    let mut expected = RenderReport::for_scene(&s, &res, &|k| k == CustomKind(1));
    expected.missing_fonts += 2;
    assert_eq!(report, expected);
    assert!(
        is_blank(&renderer.snapshot().unwrap()),
        "glyph runs draw nothing"
    );

    let mut zero = gpu(0, 10).expect("an adapter was found above");
    assert_eq!(zero.render(&s, &res).unwrap(), RenderReport::default());
}

#[test]
fn render_wgpu_05_scale_factor_and_clipping_to_target() {
    let s = scene(50.0, 50.0, |b| {
        b.fill_rect(r(10.0, 10.0, 20.0, 20.0), RED);
        b.fill_rect(r(40.0, 40.0, 100.0, 100.0), BLUE);
    });
    let Some((image, _)) = render_with(100, 100, 2.0, &s, &Resources::new()) else {
        return;
    };
    let red = [255, 0, 0, 255];
    assert_px(&image, 21, 21, red, 0);
    assert_px(&image, 58, 58, red, 0);
    assert_px(&image, 18, 18, CLEAR, 0);
    assert_px(&image, 61, 61, CLEAR, 0);
    assert_px(&image, 99, 99, [0, 0, 255, 255], 0);
    assert_px(&image, 70, 10, CLEAR, 0);
}

#[test]
fn render_wgpu_06_rendering_is_deterministic() {
    let mut res = Resources::new();
    let image = res.add_image(
        ImageData::rgba8(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 128,
            ],
        )
        .unwrap(),
    );
    let s = scene(100.0, 100.0, |b| {
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
    });
    let Some(mut renderer) = gpu(120, 120) else {
        return;
    };
    renderer.resize(120, 120, 1.25);
    renderer.render(&s, &res).unwrap();
    let first = renderer.snapshot().unwrap();
    renderer.render(&s, &res).unwrap();
    assert_eq!(renderer.snapshot().unwrap(), first);
    assert!(!is_blank(&first));
}

// ---- Fills, blending and strokes ---------------------------------------------------------

#[test]
fn render_wgpu_07_fills_and_blending() {
    let Some(image) = render100(|b| {
        b.fill_rect(r(0.0, 0.0, 50.0, 50.0), Color::from_rgba8(10, 200, 30, 255));
        b.fill_rect(
            r(50.0, 0.0, 50.0, 50.0),
            Color::from_rgba8(200, 100, 50, 128),
        );
        b.fill_rect(r(0.0, 50.0, 100.0, 50.0), Color::BLACK);
        b.fill_rect(r(0.0, 50.0, 100.0, 50.0), Color::new(1.0, 1.0, 1.0, 0.5));
    }) else {
        return;
    };
    assert_px(&image, 25, 25, [10, 200, 30, 255], 0);
    assert_px(&image, 75, 25, [200, 100, 50, 128], 1);
    assert_px(&image, 50, 75, [128, 128, 128, 255], 1);
}

#[test]
fn render_wgpu_08_rounded_corners_and_radius_clamping() {
    let red = [255, 0, 0, 255];
    let Some(image) = render100(|b| {
        b.fill(
            RoundedRect::new(r(10.0, 10.0, 40.0, 40.0), BorderRadius::circular(10.0)),
            RED,
        );
        b.fill(
            RoundedRect::new(r(10.0, 60.0, 40.0, 20.0), BorderRadius::circular(100.0)),
            RED,
        );
        b.fill(
            RoundedRect::new(r(60.0, 10.0, 30.0, 30.0), BorderRadius::circular(-5.0)),
            RED,
        );
    }) else {
        return;
    };
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
fn render_wgpu_08_radius_over_half_a_side() {
    // A 20-pixel bottom-left radius on a 20-pixel-tall rect with a square top-left corner: the
    // radii fit (CSS rule), so the corner curves along the whole left side: its circle is centered at (30, 10) and
    // passes through the rect's top-left corner.
    let red = [255, 0, 0, 255];
    let Some(image) = render100(|b| {
        b.fill(
            RoundedRect::new(
                r(10.0, 10.0, 40.0, 20.0),
                BorderRadius {
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_right: 0.0,
                    bottom_left: 20.0,
                },
            ),
            RED,
        );
    }) else {
        return;
    };
    assert_px(&image, 14, 11, red, 0);
    assert_px(&image, 10, 18, CLEAR, 0);
    assert_px(&image, 10, 19, CLEAR, 0);
    assert_px(&image, 20, 12, red, 0);
    assert_px(&image, 30, 29, red, 0);
}

#[test]
fn render_wgpu_09_strokes_and_empty_draws() {
    let red = [255, 0, 0, 255];
    let Some(image) = render100(|b| {
        b.stroke(RoundedRect::from_rect(r(10.0, 10.0, 40.0, 40.0)), 4.0, RED);
        b.stroke(RoundedRect::from_rect(r(60.0, 10.0, 30.0, 30.0)), 15.0, RED);
    }) else {
        return;
    };
    assert_px(&image, 11, 30, red, 0);
    assert_px(&image, 30, 11, red, 0);
    assert_px(&image, 48, 30, red, 0);
    assert_px(&image, 16, 30, CLEAR, 0);
    assert_px(&image, 30, 30, CLEAR, 0);
    assert_px(&image, 8, 30, CLEAR, 0);
    assert_px(&image, 75, 25, red, 0);

    let Some(none) = render100(|b| {
        b.stroke(RoundedRect::from_rect(r(10.0, 10.0, 40.0, 40.0)), 0.0, RED);
        b.stroke(RoundedRect::from_rect(r(10.0, 10.0, 40.0, 40.0)), -2.0, RED);
        b.fill_rect(r(10.0, 10.0, 0.0, 20.0), RED);
        b.fill_rect(Rect::from_ltrb(50.0, 50.0, 10.0, 10.0), RED);
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::TRANSPARENT);
        b.box_shadow(shadow(RoundedRect::from_rect(r(10.0, 10.0, 0.0, 0.0))));
    }) else {
        return;
    };
    assert!(is_blank(&none));
}

// ---- Box shadows -------------------------------------------------------------------------

#[test]
fn render_wgpu_10_box_shadows() {
    let black = [0, 0, 0, 255];
    let Some(hard) = render100(|b| {
        b.box_shadow(BoxShadow {
            offset: Vec2::new(10.0, 0.0),
            spread_radius: 5.0,
            ..shadow(RoundedRect::from_rect(r(20.0, 20.0, 20.0, 20.0)))
        });
    }) else {
        return;
    };
    assert_px(&hard, 26, 16, black, 0);
    assert_px(&hard, 53, 43, black, 0);
    assert_px(&hard, 23, 30, CLEAR, 0);
    assert_px(&hard, 56, 30, CLEAR, 0);

    let Some(grown) = render100(|b| {
        b.box_shadow(BoxShadow {
            spread_radius: 10.0,
            ..shadow(RoundedRect::new(
                r(30.0, 30.0, 40.0, 40.0),
                BorderRadius::circular(10.0),
            ))
        });
        b.box_shadow(BoxShadow {
            spread_radius: -10.0,
            ..shadow(RoundedRect::new(
                r(0.0, 80.0, 40.0, 20.0),
                BorderRadius::circular(5.0),
            ))
        });
    }) else {
        return;
    };
    assert_px(&grown, 23, 23, CLEAR, 0);
    assert_px(&grown, 50, 50, black, 0);

    let blurred = |blur: f32| {
        move |b: &mut SceneBuilder<'_>| {
            b.box_shadow(BoxShadow {
                blur_radius: blur,
                ..shadow(RoundedRect::from_rect(r(20.0, 20.0, 60.0, 60.0)))
            });
        }
    };
    let Some(soft) = render100(blurred(5.0)) else {
        return;
    };
    assert_px(&soft, 50, 50, [0, 0, 0, 255], 1);
    assert_px(&soft, 5, 50, CLEAR, 1);
    let edge = (px(&soft, 19, 50)[3] as i32 + px(&soft, 20, 50)[3] as i32) / 2;
    assert!((102..=153).contains(&edge), "edge alpha {edge}");
    let s = scene(100.0, 100.0, blurred(5.0));
    let Some((double, _)) = render_with(200, 200, 2.0, &s, &Resources::new()) else {
        return;
    };
    let (near1, near2) = (px(&soft, 14, 50)[3], px(&double, 28, 100)[3]);
    assert!(near1 > 0, "6 px outside is still inside the blur");
    assert!(
        near1.abs_diff(near2) <= 4,
        "scale 1: {near1}, scale 2: {near2}"
    );
    let Some(negative) = render100(blurred(-3.0)) else {
        return;
    };
    assert_px(&negative, 19, 50, CLEAR, 0);
    assert_px(&negative, 20, 50, black, 0);
}

// ---- Images ------------------------------------------------------------------------------

#[test]
fn render_wgpu_11_images() {
    let mut res = Resources::new();
    let checker = res.add_image(
        ImageData::rgba8(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        )
        .unwrap(),
    );
    let white = res.add_image(ImageData::rgba8(1, 1, vec![255, 255, 255, 255]).unwrap());
    let s = scene(100.0, 100.0, |b| {
        b.image(image_draw(checker, r(10.0, 10.0, 20.0, 20.0)));
        b.image(ImageDraw {
            src: Some(r(1.0, 0.0, 1.0, 1.0)),
            ..image_draw(checker, r(50.0, 10.0, 20.0, 20.0))
        });
        b.image(ImageDraw {
            sampling: ImageSampling::Linear,
            ..image_draw(checker, r(0.0, 50.0, 40.0, 40.0))
        });
        b.image(ImageDraw {
            opacity: 0.5,
            ..image_draw(white, r(50.0, 50.0, 20.0, 20.0))
        });
        b.image(ImageDraw {
            opacity: 2.0,
            ..image_draw(white, r(75.0, 50.0, 20.0, 20.0))
        });
        b.image(ImageDraw {
            opacity: -1.0,
            ..image_draw(white, r(50.0, 75.0, 20.0, 20.0))
        });
    });
    let Some((image, report)) = render_with(100, 100, 1.0, &s, &res) else {
        return;
    };
    assert!(report.is_clean());
    for (x, y, color) in [
        (11, 11, [255, 0, 0, 255]),
        (18, 18, [255, 0, 0, 255]),
        (21, 11, [0, 255, 0, 255]),
        (11, 21, [0, 0, 255, 255]),
        (28, 28, [255, 255, 255, 255]),
        (51, 11, [0, 255, 0, 255]),
        (68, 28, [0, 255, 0, 255]),
        (60, 60, [255, 255, 255, 128]),
        (85, 60, [255, 255, 255, 255]),
        (60, 85, CLEAR),
    ] {
        assert_px(&image, x, y, color, 1);
    }
    let mid = px(&image, 20, 70);
    for c in &mid[..3] {
        assert!((60..=200).contains(c), "blended pixel {mid:?}");
    }

    // Removed images draw nothing and are counted; new ones draw.
    let Some(mut renderer) = gpu(100, 100) else {
        return;
    };
    let red = res.add_image(ImageData::rgba8(1, 1, vec![255, 0, 0, 255]).unwrap());
    let s = scene(100.0, 100.0, |b| {
        b.image(image_draw(red, r(0.0, 0.0, 50.0, 50.0)))
    });
    renderer.render(&s, &res).unwrap();
    assert_px(&renderer.snapshot().unwrap(), 25, 25, [255, 0, 0, 255], 0);
    res.remove_image(red);
    assert_eq!(renderer.render(&s, &res).unwrap().missing_images, 1);
    assert!(is_blank(&renderer.snapshot().unwrap()));
}

// ---- Transforms, clips and layers --------------------------------------------------------

#[test]
fn render_wgpu_12_transforms() {
    let red = [255, 0, 0, 255];
    let Some(image) = render100(|b| {
        b.push_transform(Affine::translate(Vec2::new(30.0, 0.0)));
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED);
        b.pop();
        b.push_transform(Affine::scale(2.0));
        b.fill_rect(r(5.0, 20.0, 10.0, 5.0), RED);
        b.pop();
        b.push_transform(Affine::translate(Vec2::new(90.0, 0.0)));
        b.push_transform(Affine::rotate(FRAC_PI_2));
        b.fill_rect(r(0.0, 0.0, 20.0, 10.0), RED);
        b.pop();
        b.fill_rect(r(0.0, 60.0, 5.0, 5.0), RED);
        b.pop();
    }) else {
        return;
    };
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
fn render_wgpu_13_clips() {
    let red = [255, 0, 0, 255];
    let Some(image) = render100(|b| {
        b.push_clip(Clip::Rect(r(0.0, 0.0, 50.0, 50.0)));
        b.push_clip(Clip::Rect(r(25.0, 0.0, 75.0, 100.0)));
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), RED);
        b.pop();
        b.fill_rect(r(0.0, 0.0, 100.0, 20.0), BLUE);
        b.pop();
        b.push_transform(Affine::translate(Vec2::new(60.0, 60.0)));
        b.push_clip(Clip::RoundedRect(RoundedRect::new(
            r(0.0, 0.0, 40.0, 40.0),
            BorderRadius::circular(15.0),
        )));
        b.push_transform(Affine::translate(Vec2::new(-60.0, -60.0)));
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), RED);
        b.pop();
        b.pop();
        b.pop();
    }) else {
        return;
    };
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
fn render_wgpu_14_layers_and_overlay() {
    let Some(image) = render100(|b| {
        b.push_layer(Layer {
            opacity: 0.5,
            overlay_color: None,
        });
        b.fill_rect(r(0.0, 0.0, 60.0, 60.0), RED);
        b.fill_rect(r(40.0, 40.0, 60.0, 60.0), BLUE);
        b.pop();
    }) else {
        return;
    };
    assert_px(&image, 50, 50, [0, 0, 255, 128], 1);
    assert_px(&image, 10, 10, [255, 0, 0, 128], 1);
    assert_px(&image, 90, 10, CLEAR, 0);

    let Some(special) = render100(|b| {
        b.push_layer(Layer {
            opacity: NAN,
            overlay_color: None,
        });
        b.fill_rect(r(0.0, 0.0, 30.0, 30.0), RED);
        b.pop();
        b.push_layer(Layer {
            opacity: 3.0,
            overlay_color: Some(BLUE),
        });
        b.fill_rect(r(40.0, 0.0, 30.0, 30.0), RED);
        b.pop();
        b.push_layer(Layer {
            opacity: 1.0,
            overlay_color: Some(BLUE.with_alpha(0.5)),
        });
        b.fill_rect(r(0.0, 40.0, 30.0, 30.0), RED);
        b.pop();
        b.push_layer(Layer {
            opacity: 1.0,
            overlay_color: Some(Color::new(NAN, 0.0, 1.0, 1.0)),
        });
        b.fill_rect(r(40.0, 40.0, 30.0, 30.0), RED);
        b.pop();
    }) else {
        return;
    };
    assert_px(&special, 15, 15, CLEAR, 0); // NaN opacity: nothing
    assert_px(&special, 55, 15, [0, 0, 255, 255], 0); // opacity clamped, opaque overlay
    assert_px(&special, 80, 15, CLEAR, 0); // left transparent by the group
    assert_px(&special, 15, 55, [128, 0, 128, 255], 1); // half overlay
    assert_px(&special, 55, 55, [255, 0, 0, 255], 0); // non-finite overlay ignored
}

// ---- Custom commands, invalid values, robustness ------------------------------------------

/// What a recording handler saw.
#[derive(Clone, Debug, PartialEq)]
struct Call {
    bounds: Rect,
    data: Vec<u8>,
    transform: [f32; 6],
    clip: Option<[u32; 4]>,
}

struct Recorder(Rc<RefCell<Vec<Call>>>);

impl CustomHandler for Recorder {
    fn draw(&mut self, canvas: CustomCanvas<'_>) {
        self.0.borrow_mut().push(Call {
            bounds: canvas.bounds,
            data: canvas.data.to_vec(),
            transform: canvas.transform,
            clip: canvas.clip,
        });
    }
}

#[test]
fn render_wgpu_15_custom_handlers() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let Some(mut renderer) = gpu(200, 200) else {
        return;
    };
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
                transform: [2.0, 0.0, 0.0, 2.0, 20.0, 0.0],
                clip: None
            },
            Call {
                bounds: r(20.0, 20.0, 10.0, 10.0),
                data: vec![2, 3],
                transform: [2.0, 0.0, 0.0, 2.0, 0.0, 0.0],
                clip: Some([0, 0, 100, 100])
            },
        ]
    );
    let later = Rc::new(RefCell::new(Vec::new()));
    renderer.register_custom(CustomKind(1), Recorder(later.clone()));
    renderer.render(&s, &Resources::new()).unwrap();
    assert_eq!(calls.borrow().len(), 2);
    assert_eq!(later.borrow().len(), 2);
}

#[test]
fn render_wgpu_16_invalid_values_draw_nothing() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let Some(mut renderer) = gpu(100, 100) else {
        return;
    };
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
fn render_wgpu_17_never_panics() {
    let mut res = Resources::new();
    let big = res.add_image(ImageData::rgba8(2000, 1500, vec![200u8; 2000 * 1500 * 4]).unwrap());
    let too_wide = res.add_image(ImageData::rgba8(70_000, 1, vec![9u8; 70_000 * 4]).unwrap());
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
        b.image(image_draw(too_wide, r(0.0, 0.0, 30.0, 30.0)));
        b.push_transform(Affine::scale(1e-30));
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED);
        b.pop();
        b.push_transform(Affine::scale(huge));
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), RED);
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
    let Some(mut renderer) = gpu(100, 100) else {
        return;
    };
    renderer.resize(100, 100, 3.0);
    let report = renderer.render(&s, &res).unwrap();
    assert_eq!(
        report.missing_images, 1,
        "an image wider than the texture limit is missing"
    );
    renderer.resize(1, 1, 0.01);
    renderer.render(&s, &res).unwrap();
}
