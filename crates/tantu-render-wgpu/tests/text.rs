//! Glyph runs in the wgpu renderer: rules RENDER-WGPU-18 and RENDER-WGPU-19 (spec
//! `docs/specs/render-wgpu/renderer.md`), the software renderer's text tests on the GPU. Text comes from `tantu_text::TextSystem` with the
//! committed Liberation Sans.

mod common;

use common::*;
use tantu_core::{Affine, Color, Point, Size, Vec2};
use tantu_scene::{Clip, ImageData, Layer, Resources, Scene, SceneBuilder};
use tantu_text::{FontFamily, TextStyle, TextSystem};

const FONT: &[u8] = include_bytes!("../../tantu-text/tests/fonts/LiberationSans-Regular.ttf");

/// A text system with only Liberation Sans, and a 40 px style.
fn text() -> (TextSystem, tantu_layout::TextStyleKey) {
    let mut system = TextSystem::without_system_fonts();
    system.register_font(FONT.to_vec());
    system.set_default_family(FontFamily::Named("Liberation Sans".into()));
    let style = system.style(TextStyle {
        size: 40.0,
        ..TextStyle::default()
    });
    (system, style)
}

/// A 100 × 100 frame: white, then whatever `paint` draws with the text system.
fn frame(
    paint: impl FnOnce(
        &mut SceneBuilder<'_>,
        &mut TextSystem,
        tantu_layout::TextStyleKey,
        &mut Resources,
    ),
) -> (Scene, Resources) {
    let (mut system, style) = text();
    let mut resources = Resources::new();
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(100.0, 100.0));
    b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::WHITE);
    paint(&mut b, &mut system, style, &mut resources);
    b.finish().expect("balanced");
    (scene, resources)
}

/// Draws "H" in black with its top-left at `origin`.
fn h(
    b: &mut SceneBuilder<'_>,
    system: &mut TextSystem,
    style: tantu_layout::TextStyleKey,
    res: &mut Resources,
    origin: Point,
) {
    system.paint(
        b,
        res,
        "H",
        style,
        f32::INFINITY,
        None,
        Color::BLACK,
        origin,
    );
}

/// Pixels darker than `level` (red channel), as (x, y).
fn dark(image: &ImageData, level: u8) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for y in 0..image.height() {
        for x in 0..image.width() {
            if px(image, x, y)[0] < level {
                out.push((x, y));
            }
        }
    }
    out
}

/// Renders on the GPU; `None` (test skipped) without an adapter.
fn render(scene: &Scene, res: &Resources, size: u32, scale: f32) -> Option<ImageData> {
    let (image, report) = render_with(size, size, scale, scene, res)?;
    assert!(report.is_clean(), "{report:?}");
    Some(image)
}

#[test]
fn render_wgpu_18_glyphs_are_drawn() {
    let (scene, res) = frame(|b, s, st, res| h(b, s, st, res, Point::new(10.0, 10.0)));
    let Some(image) = render(&scene, &res, 100, 1.0) else {
        return;
    };
    let ink = dark(&image, 64);
    assert!(!ink.is_empty(), "no dark pixels");
    // Inside the glyph's box (40 px text: H about 29 wide, cap height about 29).
    assert!(
        ink.iter()
            .all(|&(x, y)| (8..45).contains(&x) && (10..60).contains(&y)),
        "{ink:?}"
    );
    assert_eq!(px(&image, 90, 90), [255, 255, 255, 255]);
    // At scale 2 the glyph is about twice as tall.
    let Some(image2) = render(&scene, &res, 200, 2.0) else {
        return;
    };
    let rows = |pts: &[(u32, u32)]| {
        let ys: Vec<u32> = pts.iter().map(|p| p.1).collect();
        ys.iter().max().unwrap_or(&0) - ys.iter().min().unwrap_or(&0) + 1
    };
    let (h1, h2) = (rows(&ink), rows(&dark(&image2, 64)));
    assert!((h2 as i32 - 2 * h1 as i32).abs() <= 3, "{h1} vs {h2}");
    // Clips apply.
    let (clipped, res) = frame(|b, s, st, res| {
        b.push_clip(Clip::Rect(r(0.0, 0.0, 25.0, 100.0)));
        h(b, s, st, res, Point::new(10.0, 10.0));
        b.pop();
    });
    let Some(image) = render(&clipped, &res, 100, 1.0) else {
        return;
    };
    let ink = dark(&image, 250);
    assert!(!ink.is_empty());
    assert!(ink.iter().all(|&(x, _)| x < 25), "{ink:?}");
    // Layers apply: half opacity can't get darker than mid-grey.
    let (faded, res) = frame(|b, s, st, res| {
        b.push_layer(Layer {
            opacity: 0.5,
            overlay_color: None,
        });
        h(b, s, st, res, Point::new(10.0, 10.0));
        b.pop();
    });
    let Some(image) = render(&faded, &res, 100, 1.0) else {
        return;
    };
    let darkest = (0..100)
        .flat_map(|y| (0..100).map(move |x| (x, y)))
        .map(|(x, y)| px(&image, x, y)[0])
        .min()
        .unwrap_or(255);
    assert!((120..=135).contains(&darkest), "{darkest}");
}

#[test]
fn render_wgpu_19_glyphs_follow_transforms() {
    let leftmost = |image: &ImageData| dark(image, 64).iter().map(|p| p.0).min();
    let (plain, res) = frame(|b, s, st, res| h(b, s, st, res, Point::new(10.0, 10.0)));
    let Some(plain_image) = render(&plain, &res, 100, 1.0) else {
        return;
    };
    let base = leftmost(&plain_image).expect("ink");
    let (moved, res) = frame(|b, s, st, res| {
        b.push_transform(Affine::translate(Vec2::new(30.0, 0.0)));
        h(b, s, st, res, Point::new(10.0, 10.0));
        b.pop();
    });
    let Some(moved_image) = render(&moved, &res, 100, 1.0) else {
        return;
    };
    assert_eq!(leftmost(&moved_image), Some(base + 30));
    // Rotated: still drawn (upright, at the transformed position).
    let (rotated, res) = frame(|b, s, st, res| {
        b.push_transform(Affine::translate(Vec2::new(50.0, 50.0)) * Affine::rotate(0.3));
        h(b, s, st, res, Point::new(-10.0, -20.0));
        b.pop();
    });
    let Some(rotated_image) = render(&rotated, &res, 100, 1.0) else {
        return;
    };
    assert!(!dark(&rotated_image, 64).is_empty());
    // Off-target: nothing drawn, no panic.
    let (outside, res) = frame(|b, s, st, res| {
        h(b, s, st, res, Point::new(500.0, -300.0));
        h(b, s, st, res, Point::new(1e30, 1e30));
    });
    let Some(outside_image) = render(&outside, &res, 100, 1.0) else {
        return;
    };
    assert!(dark(&outside_image, 250).is_empty());
}
