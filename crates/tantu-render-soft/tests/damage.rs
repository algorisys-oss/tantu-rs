//! Tests for `docs/specs/scene/damage.md`, rules RENDER-SOFT-29 and RENDER-SOFT-30.

use tantu_core::{Color, Rect, Size};
use tantu_render_conformance::reference_scenes;
use tantu_render_soft::SoftRenderer;
use tantu_scene::{Command, ElementId, Entry, Renderer, Resources, Scene, SceneBuilder};

fn r(l: f32, t: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(l, t, w, h)
}

/// Three 30 × 30 boxes (elements 1..=3) on a 100 × 100 frame, element `odd` in white.
fn boxes(odd: u64) -> Scene {
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(100.0, 100.0));
    for n in 1..=3u64 {
        b.set_element(ElementId::from_raw(n));
        let color = if n == odd { Color::WHITE } else { Color::BLACK };
        b.fill_rect(r(5.0 + 32.0 * (n - 1) as f32, 5.0, 30.0, 30.0), color);
    }
    b.finish().expect("balanced");
    scene
}

#[test]
fn render_soft_29_partial_repaint() {
    let res = Resources::new();
    let mut renderer = SoftRenderer::new(100, 100);
    renderer.resize(100, 100, 1.0);
    // The first frame (no damage recorded: full).
    let first = boxes(0);
    let report = renderer.render(&first, &res).expect("render");
    assert_eq!(report.repainted_pixels, 100 * 100);
    // One box changed: only its 30 × 30 pixels.
    let mut second = boxes(2);
    second.diff_damage(&first);
    let report = renderer.render(&second, &res).expect("render");
    assert_eq!(report.repainted_pixels, 30 * 30);
    // Nothing changed: nothing repainted.
    let mut third = boxes(2);
    third.diff_damage(&second);
    let report = renderer.render(&third, &res).expect("render");
    assert_eq!(report.repainted_pixels, 0);
    // A new scale factor repaints everything, even with damage rects (here 2 × 2 device
    // pixels per logical pixel).
    renderer.resize(200, 200, 2.0);
    let mut fourth = boxes(3);
    fourth.diff_damage(&third);
    let report = renderer.render(&fourth, &res).expect("render");
    assert_eq!(report.repainted_pixels, 200 * 200);
    // Then damage is in device pixels, rounded outward: element 1's box at 2×.
    let mut fifth = boxes(1);
    fifth.diff_damage(&fourth);
    let report = renderer.render(&fifth, &res).expect("render");
    // Elements 1 and 3 changed: two 60 × 60 device boxes.
    assert_eq!(report.repainted_pixels, 2 * 60 * 60);
}

/// Records `scene` again, recoloring `element`'s draws.
fn replay(scene: &Scene, element: Option<ElementId>) -> Scene {
    let recolor = |entry: &Entry, color: Color| {
        if entry.element == element {
            Color::new(1.0 - color.r, 1.0 - color.g, 0.5, 1.0)
        } else {
            color
        }
    };
    let mut out = Scene::new();
    let mut b: SceneBuilder<'_> = out.begin(scene.size());
    for entry in scene.entries() {
        b.set_element(entry.element);
        b.set_z_index(entry.z_index);
        match &entry.command {
            Command::PushClip(clip) => b.push_clip(*clip),
            Command::PushTransform(t) => b.push_transform(*t),
            Command::PushLayer(layer) => b.push_layer(*layer),
            Command::PopClip | Command::PopTransform | Command::PopLayer => b.pop(),
            Command::Fill { shape, color } => b.fill(*shape, recolor(entry, *color)),
            Command::Stroke {
                shape,
                width,
                color,
            } => b.stroke(*shape, *width, recolor(entry, *color)),
            Command::BoxShadow(shadow) => {
                let mut shadow = *shadow;
                shadow.color = recolor(entry, shadow.color);
                b.box_shadow(shadow);
            }
            Command::Image(image) => {
                let mut image = *image;
                if entry.element == element {
                    image.opacity *= 0.5;
                }
                b.image(image);
            }
            Command::GlyphRun(run) => b.glyph_run(
                run.font,
                run.font_size,
                recolor(entry, run.color),
                run.origin,
                scene.glyphs(run),
            ),
            Command::Custom(custom) => {
                b.custom(custom.kind, custom.bounds, scene.custom_data(custom));
            }
        }
    }
    b.finish().expect("balanced");
    out
}

#[test]
fn render_soft_30_partial_equals_full() {
    let mut checked = 0;
    for reference in reference_scenes() {
        let (scene, res) = reference.record();
        let (w, h) = reference.target_size();
        let scale = reference.scale_factor();
        let mut elements: Vec<Option<ElementId>> = Vec::new();
        for entry in scene.entries() {
            if !elements.contains(&entry.element) {
                elements.push(entry.element);
            }
        }
        for element in elements {
            let mut partial = SoftRenderer::new(w, h);
            partial.resize(w, h, scale);
            partial.render(&scene, &res).expect("render");
            let mut next = replay(&scene, element);
            next.diff_damage(&scene);
            partial.render(&next, &res).expect("render");
            let mut full = SoftRenderer::new(w, h);
            full.resize(w, h, scale);
            full.render(&replay(&scene, element), &res).expect("render");
            assert_eq!(
                partial.snapshot(),
                full.snapshot(),
                "{} with element {element:?} changed",
                reference.name()
            );
            checked += 1;
        }
    }
    assert!(checked >= 10, "only {checked} cases");
}
