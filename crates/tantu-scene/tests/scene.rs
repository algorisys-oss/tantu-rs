//! Tests for `docs/specs/scene/scene.md`, one or more per rule SCENE-SCENE-NN.
//! SCENE-SCENE-24 (no allocation once warm) needs its own global allocator and lives in
//! `tests/alloc.rs`.

use std::fmt::Debug;
use std::mem::size_of;

use tantu_core::{Affine, Arena, Color, Point, Rect, Size, Vec2};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, Command, CustomKind, Damage, ElementId, Entry, FontId, Glyph,
    ImageDraw, ImageId, ImageSampling, Layer, RoundedRect, Scene, SceneError,
};

const SIZE: Size = Size::new(100.0, 100.0);

fn rect(l: f32, t: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(l, t, w, h)
}

fn el(raw: u64) -> Option<ElementId> {
    ElementId::from_raw(raw)
}

/// A gray level used to tell fills apart.
fn shade(i: u8) -> Color {
    Color::from_rgb8(i, i, i)
}

/// The gray level of each `Fill` entry, in order.
fn fill_shades(scene: &Scene) -> Vec<u8> {
    scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { color, .. } => Some(color.to_rgba8()[0]),
            _ => None,
        })
        .collect()
}

/// A short name for each entry, for comparing structure.
fn names(scene: &Scene) -> Vec<String> {
    scene
        .entries()
        .iter()
        .map(|e| match &e.command {
            Command::PushClip(_) => "push_clip".to_owned(),
            Command::PopClip => "pop_clip".to_owned(),
            Command::PushTransform(_) => "push_transform".to_owned(),
            Command::PopTransform => "pop_transform".to_owned(),
            Command::PushLayer(_) => "push_layer".to_owned(),
            Command::PopLayer => "pop_layer".to_owned(),
            Command::Fill { color, .. } => format!("fill{}", color.to_rgba8()[0]),
            Command::Stroke { .. } => "stroke".to_owned(),
            Command::BoxShadow(_) => "shadow".to_owned(),
            Command::Image(_) => "image".to_owned(),
            Command::GlyphRun(_) => "glyphs".to_owned(),
            Command::Custom(_) => "custom".to_owned(),
        })
        .collect()
}

/// Every push has exactly one matching pop of the same kind, properly nested.
fn assert_well_formed(entries: &[Entry]) {
    let mut stack = Vec::new();
    for e in entries {
        match e.command {
            Command::PushClip(_) => stack.push("clip"),
            Command::PushTransform(_) => stack.push("transform"),
            Command::PushLayer(_) => stack.push("layer"),
            Command::PopClip => assert_eq!(stack.pop(), Some("clip")),
            Command::PopTransform => assert_eq!(stack.pop(), Some("transform")),
            Command::PopLayer => assert_eq!(stack.pop(), Some("layer")),
            _ => {}
        }
    }
    assert!(stack.is_empty(), "unclosed scopes: {stack:?}");
}

/// A frame using every kind of command.
fn record_everything(scene: &mut Scene) {
    let font = FontId::from_raw(7).unwrap();
    let image = ImageId::from_raw(9).unwrap();
    let mut b = scene.begin(SIZE);
    b.set_element(el(1));
    b.push_layer(Layer {
        opacity: 0.5,
        overlay_color: Some(Color::WHITE),
    });
    b.push_transform(Affine::translate(Vec2::new(5.0, 5.0)));
    b.push_clip(Clip::RoundedRect(RoundedRect::new(
        rect(0.0, 0.0, 50.0, 50.0),
        BorderRadius::circular(4.0),
    )));
    b.fill_rect(rect(1.0, 2.0, 3.0, 4.0), shade(1));
    b.stroke(
        RoundedRect::from_rect(rect(0.0, 0.0, 9.0, 9.0)),
        1.0,
        shade(2),
    );
    b.box_shadow(BoxShadow {
        shape: RoundedRect::from_rect(rect(0.0, 0.0, 9.0, 9.0)),
        color: Color::BLACK,
        offset: Vec2::new(0.0, 2.0),
        blur_radius: 4.0,
        spread_radius: 1.0,
    });
    b.image(ImageDraw {
        image,
        src: None,
        dest: rect(0.0, 0.0, 16.0, 16.0),
        sampling: ImageSampling::Nearest,
        opacity: 1.0,
    });
    b.glyph_run(
        font,
        14.0,
        Color::BLACK,
        Point::new(0.0, 12.0),
        &[
            Glyph {
                id: 1,
                x: 0.0,
                y: 0.0,
            },
            Glyph {
                id: 2,
                x: 8.0,
                y: 0.0,
            },
        ],
    );
    b.custom(CustomKind(3), rect(0.0, 0.0, 20.0, 20.0), &[1, 2, 3]);
    b.pop();
    b.pop();
    b.pop();
    b.add_damage(rect(0.0, 0.0, 10.0, 10.0));
    b.finish().unwrap();
}

// ---- Recording ---------------------------------------------------------------------------

#[test]
fn scene_scene_01_new_scene_is_empty() {
    let scene = Scene::new();
    assert_eq!(scene, Scene::default());
    assert_eq!(scene.size(), Size::ZERO);
    assert!(scene.entries().is_empty());
    assert_eq!(scene.damage(), Damage::Full);
}

#[test]
fn scene_scene_02_begin_discards_previous_frame() {
    let mut scene = Scene::new();
    record_everything(&mut scene);
    let old_run = scene
        .entries()
        .iter()
        .find_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.clone()),
            _ => None,
        })
        .unwrap();
    let old_custom = scene
        .entries()
        .iter()
        .find_map(|e| match &e.command {
            Command::Custom(c) => Some(c.clone()),
            _ => None,
        })
        .unwrap();
    assert!(matches!(scene.damage(), Damage::Rects(_)));

    let size = Size::new(640.0, 480.0);
    scene.begin(size).finish().unwrap();
    assert_eq!(scene.size(), size);
    assert!(scene.entries().is_empty());
    assert!(scene.glyphs(&old_run).is_empty());
    assert!(scene.custom_data(&old_custom).is_empty());
    assert_eq!(scene.damage(), Damage::Full);
}

#[test]
fn scene_scene_03_dropped_builder_leaves_scene_empty() {
    let mut scene = Scene::new();
    {
        let mut b = scene.begin(SIZE);
        b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
        b.push_clip(Clip::Rect(rect(0.0, 0.0, 5.0, 5.0)));
    }
    assert!(scene.entries().is_empty());

    record_everything(&mut scene);
    {
        let mut b = scene.begin(SIZE);
        b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    }
    assert!(scene.entries().is_empty());
}

#[test]
fn scene_scene_04_entries_keep_recording_order() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    for i in [3, 1, 2, 9, 0] {
        b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(i));
    }
    b.finish().unwrap();
    assert_eq!(fill_shades(&scene), [3, 1, 2, 9, 0]);
}

#[test]
fn scene_scene_05_element_and_z_are_sticky() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(0));
    b.set_element(el(1));
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(2));
    b.set_element(el(2));
    b.set_z_index(3);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(3));
    b.finish().unwrap();
    let meta: Vec<_> = scene
        .entries()
        .iter()
        .map(|e| (e.element, e.z_index))
        .collect();
    assert_eq!(meta, [(None, 0), (el(1), 0), (el(1), 0), (el(2), 3)]);

    // `begin` resets both.
    let mut b = scene.begin(SIZE);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(0));
    b.finish().unwrap();
    assert_eq!(scene.entries()[0].element, None);
    assert_eq!(scene.entries()[0].z_index, 0);
}

#[test]
fn scene_scene_06_pop_restores_element_and_z() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    b.set_element(el(1));
    b.set_z_index(1);
    b.push_clip(Clip::Rect(rect(0.0, 0.0, 50.0, 50.0)));
    b.set_element(el(2));
    b.set_z_index(5);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    b.pop();
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(2));
    b.finish().unwrap();

    let e = scene.entries();
    assert_eq!(names(&scene), ["push_clip", "fill1", "pop_clip", "fill2"]);
    assert_eq!((e[1].element, e[1].z_index), (el(2), 5));
    assert_eq!((e[3].element, e[3].z_index), (el(1), 1));
}

#[test]
fn scene_scene_07_values_stored_as_given() {
    let nan = f32::NAN;
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(nan, -1.0));
    b.fill_rect(
        Rect::from_ltrb(nan, 0.0, f32::INFINITY, -5.0),
        Color::new(nan, 2.0, -1.0, 0.0),
    );
    b.fill_rect(Rect::from_ltrb(10.0, 10.0, 0.0, 0.0), Color::TRANSPARENT);
    b.stroke(
        RoundedRect::new(rect(0.0, 0.0, 1.0, 1.0), BorderRadius::circular(-3.0)),
        -2.0,
        Color::BLACK,
    );
    b.push_transform(Affine::new([nan; 6]));
    b.push_layer(Layer {
        opacity: nan,
        overlay_color: None,
    });
    b.push_clip(Clip::Rect(Rect::from_ltrb(nan, nan, nan, nan)));
    b.glyph_run(
        FontId::from_raw(1).unwrap(),
        nan,
        Color::BLACK,
        Point::new(nan, 0.0),
        &[Glyph {
            id: 0,
            x: nan,
            y: 0.0,
        }],
    );
    b.custom(CustomKind(0), Rect::from_ltrb(nan, 0.0, 0.0, 0.0), &[]);
    let _ = b.is_culled(Rect::from_ltrb(nan, nan, nan, nan));
    b.add_damage(Rect::from_ltrb(nan, 0.0, 0.0, 0.0));
    b.pop();
    b.pop();
    b.pop();
    b.finish().unwrap();

    let e = scene.entries();
    assert_eq!(e.len(), 11);
    match e[0].command {
        Command::Fill { shape, color } => {
            assert!(shape.rect.left.is_nan());
            assert_eq!(shape.rect.right, f32::INFINITY);
            assert_eq!(shape.rect.bottom, -5.0);
            assert!(color.r.is_nan());
            assert_eq!((color.g, color.b), (2.0, -1.0));
        }
        ref other => panic!("expected a fill, got {other:?}"),
    }
    match e[1].command {
        Command::Fill { shape, color } => {
            assert_eq!(shape.rect, Rect::from_ltrb(10.0, 10.0, 0.0, 0.0));
            assert_eq!(color, Color::TRANSPARENT);
        }
        ref other => panic!("expected a fill, got {other:?}"),
    }
    match e[2].command {
        Command::Stroke { shape, width, .. } => {
            assert_eq!(shape.radii, BorderRadius::circular(-3.0));
            assert_eq!(width, -2.0);
        }
        ref other => panic!("expected a stroke, got {other:?}"),
    }
    match e[3].command {
        Command::PushTransform(t) => assert!(t.coeffs().iter().all(|c| c.is_nan())),
        ref other => panic!("expected a transform, got {other:?}"),
    }
    match e[4].command {
        Command::PushLayer(layer) => assert!(layer.opacity.is_nan()),
        ref other => panic!("expected a layer, got {other:?}"),
    }
    match &e[6].command {
        Command::GlyphRun(run) => {
            assert!(run.font_size.is_nan());
            assert!(scene.glyphs(run)[0].x.is_nan());
        }
        other => panic!("expected a glyph run, got {other:?}"),
    }
    match scene.damage() {
        Damage::Rects(rects) => assert!(rects[0].left.is_nan()),
        Damage::Full => panic!("expected damage rects"),
    }
    assert!(scene.size().width.is_nan());
}

#[test]
fn scene_scene_08_draw_methods_record_given_values() {
    let shape = RoundedRect::new(
        rect(1.0, 2.0, 30.0, 40.0),
        BorderRadius {
            top_left: 1.0,
            top_right: 2.0,
            bottom_right: 3.0,
            bottom_left: 4.0,
        },
    );
    let shadow = BoxShadow {
        shape,
        color: shade(5),
        offset: Vec2::new(1.0, -2.0),
        blur_radius: 6.0,
        spread_radius: -1.0,
    };
    let image = ImageDraw {
        image: ImageId::from_raw(42).unwrap(),
        src: Some(rect(0.0, 0.0, 8.0, 8.0)),
        dest: rect(10.0, 10.0, 32.0, 32.0),
        sampling: ImageSampling::Nearest,
        opacity: 0.25,
    };
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    b.fill_rect(rect(1.0, 2.0, 3.0, 4.0), shade(1));
    b.fill(shape, shade(2));
    b.stroke(shape, 1.5, shade(3));
    b.box_shadow(shadow);
    b.image(image);
    b.finish().unwrap();

    let commands: Vec<_> = scene.entries().iter().map(|e| e.command.clone()).collect();
    assert_eq!(
        commands,
        [
            Command::Fill {
                shape: RoundedRect::from_rect(rect(1.0, 2.0, 3.0, 4.0)),
                color: shade(1)
            },
            Command::Fill {
                shape,
                color: shade(2)
            },
            Command::Stroke {
                shape,
                width: 1.5,
                color: shade(3)
            },
            Command::BoxShadow(shadow),
            Command::Image(image),
        ]
    );
    assert_eq!(
        RoundedRect::from_rect(rect(1.0, 2.0, 3.0, 4.0)).radii,
        BorderRadius::ZERO
    );
    assert_eq!(
        RoundedRect::new(rect(1.0, 2.0, 30.0, 40.0), shape.radii),
        shape
    );
}

// ---- Glyph runs and custom data ----------------------------------------------------------

#[test]
fn scene_scene_09_glyph_runs_copy_glyphs() {
    let font = FontId::from_raw(3).unwrap();
    let glyphs: Vec<Glyph> = (0..5)
        .map(|i| Glyph {
            id: i,
            x: i as f32 * 7.0,
            y: 0.5,
        })
        .collect();
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    b.glyph_run(font, 12.0, shade(4), Point::new(3.0, 15.0), &glyphs[..2]);
    b.glyph_run(font, 16.0, shade(5), Point::new(3.0, 35.0), &glyphs);
    b.glyph_run(font, 16.0, shade(6), Point::new(3.0, 55.0), &[]);
    b.finish().unwrap();

    let runs: Vec<_> = scene
        .entries()
        .iter()
        .map(|e| match &e.command {
            Command::GlyphRun(run) => run.clone(),
            other => panic!("expected a glyph run, got {other:?}"),
        })
        .collect();
    assert_eq!(scene.glyphs(&runs[0]), &glyphs[..2]);
    assert_eq!(scene.glyphs(&runs[1]), &glyphs[..]);
    assert!(scene.glyphs(&runs[2]).is_empty());
    assert_eq!(runs[1].font, font);
    assert_eq!(runs[1].font_size, 16.0);
    assert_eq!(runs[1].color, shade(5));
    assert_eq!(runs[1].origin, Point::new(3.0, 35.0));
}

#[test]
fn scene_scene_10_custom_commands_copy_data() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    b.custom(CustomKind(1), rect(0.0, 0.0, 10.0, 10.0), b"chart");
    b.custom(CustomKind(2), rect(5.0, 5.0, 10.0, 10.0), &[0, 255, 7]);
    b.finish().unwrap();

    let customs: Vec<_> = scene
        .entries()
        .iter()
        .map(|e| match &e.command {
            Command::Custom(c) => c.clone(),
            other => panic!("expected a custom command, got {other:?}"),
        })
        .collect();
    assert_eq!(scene.custom_data(&customs[0]), b"chart");
    assert_eq!(scene.custom_data(&customs[1]), &[0, 255, 7]);
    assert_eq!(customs[1].kind, CustomKind(2));
    assert_eq!(customs[1].bounds, rect(5.0, 5.0, 10.0, 10.0));
}

#[test]
fn scene_scene_11_foreign_runs_and_customs_read_empty() {
    let font = FontId::from_raw(1).unwrap();
    let glyph = Glyph {
        id: 1,
        x: 0.0,
        y: 0.0,
    };

    let mut big = Scene::new();
    let mut b = big.begin(SIZE);
    b.glyph_run(font, 10.0, Color::BLACK, Point::ZERO, &[glyph; 4]);
    b.glyph_run(font, 10.0, Color::BLACK, Point::ZERO, &[glyph; 4]);
    b.custom(CustomKind(0), Rect::ZERO, &[1; 8]);
    b.custom(CustomKind(0), Rect::ZERO, &[1; 8]);
    b.finish().unwrap();
    let late_run = match &big.entries()[1].command {
        Command::GlyphRun(run) => run.clone(),
        other => panic!("expected a glyph run, got {other:?}"),
    };
    let late_custom = match &big.entries()[3].command {
        Command::Custom(c) => c.clone(),
        other => panic!("expected a custom command, got {other:?}"),
    };

    // Another, smaller Scene.
    let mut small = Scene::new();
    let mut b = small.begin(SIZE);
    b.glyph_run(font, 10.0, Color::BLACK, Point::ZERO, &[glyph]);
    b.custom(CustomKind(0), Rect::ZERO, &[1]);
    b.finish().unwrap();
    assert!(small.glyphs(&late_run).is_empty());
    assert!(small.custom_data(&late_custom).is_empty());

    // The same Scene, an emptier later frame.
    big.begin(SIZE).finish().unwrap();
    assert!(big.glyphs(&late_run).is_empty());
    assert!(big.custom_data(&late_custom).is_empty());
}

// ---- Z-index and scopes ------------------------------------------------------------------

#[test]
fn scene_scene_12_finish_sorts_by_z_within_scopes() {
    // Root level: stable ascending sort, negative z first.
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    for (z, i) in [(2, 1), (0, 2), (1, 3), (0, 4), (-1, 5), (2, 6)] {
        b.set_z_index(z);
        b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(i));
    }
    b.finish().unwrap();
    assert_eq!(fill_shades(&scene), [5, 2, 4, 3, 1, 6]);

    // A nested scope moves as one unit, with the z of its push entry; its contents are sorted
    // among themselves and never leave it.
    let mut b = scene.begin(SIZE);
    b.set_z_index(5);
    b.push_clip(Clip::Rect(rect(0.0, 0.0, 50.0, 50.0)));
    b.set_z_index(-10);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    b.set_z_index(-20);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(2));
    b.pop();
    b.set_z_index(0);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(3));
    b.push_layer(Layer::default());
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(4));
    b.pop();
    b.set_z_index(7);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(5));
    b.finish().unwrap();
    assert_eq!(
        names(&scene),
        [
            "fill3",
            "push_layer",
            "fill4",
            "pop_layer",
            "push_clip",
            "fill2",
            "fill1",
            "pop_clip",
            "fill5"
        ]
    );
    assert_well_formed(scene.entries());
}

#[test]
fn scene_scene_13_push_and_pop_entries() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    let clip = Clip::Rect(rect(0.0, 0.0, 10.0, 10.0));
    let transform = Affine::scale(2.0);
    let layer = Layer {
        opacity: 0.5,
        overlay_color: Some(shade(9)),
    };
    b.set_element(el(1));
    b.set_z_index(1);
    b.push_layer(layer);
    b.set_element(el(2));
    b.push_transform(transform);
    b.set_element(el(3));
    b.push_clip(clip);
    b.set_element(el(4));
    b.set_z_index(1);
    b.pop();
    b.pop();
    b.pop();
    b.finish().unwrap();

    let e = scene.entries();
    assert_eq!(e[0].command, Command::PushLayer(layer));
    assert_eq!(e[1].command, Command::PushTransform(transform));
    assert_eq!(e[2].command, Command::PushClip(clip));
    assert_eq!(e[3].command, Command::PopClip);
    assert_eq!(e[4].command, Command::PopTransform);
    assert_eq!(e[5].command, Command::PopLayer);
    // Each pop carries its push entry's element id and z-index.
    for (push, pop) in [(0, 5), (1, 4), (2, 3)] {
        assert_eq!(e[pop].element, e[push].element);
        assert_eq!(e[pop].z_index, e[push].z_index);
    }
    assert_eq!(e[2].element, el(3));
}

#[test]
fn scene_scene_14_unmatched_pop_records_nothing() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    b.pop();
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    b.push_clip(Clip::Rect(rect(0.0, 0.0, 10.0, 10.0)));
    b.pop();
    b.pop();
    assert_eq!(
        b.finish(),
        Err(SceneError {
            unmatched_pops: 2,
            unclosed_scopes: 0
        })
    );
    assert_eq!(names(&scene), ["fill1", "push_clip", "pop_clip"]);
}

#[test]
fn scene_scene_15_finish_closes_open_scopes() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    b.set_element(el(1));
    b.push_clip(Clip::Rect(rect(0.0, 0.0, 10.0, 10.0)));
    b.set_element(el(2));
    b.push_layer(Layer::default());
    b.set_element(el(3));
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    assert_eq!(
        b.finish(),
        Err(SceneError {
            unmatched_pops: 0,
            unclosed_scopes: 2
        })
    );
    assert_eq!(
        names(&scene),
        ["push_clip", "push_layer", "fill1", "pop_layer", "pop_clip"]
    );
    assert_eq!(scene.entries()[3].element, el(2));
    assert_eq!(scene.entries()[4].element, el(1));
}

#[test]
fn scene_scene_16_finish_result_and_well_formed_scene() {
    let mut scene = Scene::new();
    record_everything(&mut scene); // unwraps finish: balanced is Ok
    assert_well_formed(scene.entries());

    let mut b = scene.begin(SIZE);
    b.pop();
    b.push_transform(Affine::IDENTITY);
    b.set_z_index(3);
    b.push_layer(Layer::default());
    b.set_z_index(-3);
    b.push_clip(Clip::Rect(Rect::ZERO));
    b.pop();
    b.set_z_index(-4);
    b.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    b.pop();
    b.pop();
    b.pop();
    b.push_layer(Layer::default());
    b.push_layer(Layer::default());
    let err = b.finish().unwrap_err();
    assert_eq!(
        err,
        SceneError {
            unmatched_pops: 2,
            unclosed_scopes: 2
        }
    );
    assert!(!err.to_string().is_empty());
    assert_well_formed(scene.entries());
}

// ---- Culling -----------------------------------------------------------------------------

#[test]
fn scene_scene_17_is_culled_against_scene_clips_and_transforms() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    assert!(!b.is_culled(rect(10.0, 10.0, 10.0, 10.0)));
    assert!(!b.is_culled(rect(-5.0, -5.0, 10.0, 10.0)), "partly visible");
    assert!(
        b.is_culled(rect(100.0, 0.0, 10.0, 10.0)),
        "touching the edge only"
    );
    assert!(b.is_culled(rect(200.0, 200.0, 10.0, 10.0)));

    b.push_transform(Affine::translate(Vec2::new(200.0, 0.0)));
    assert!(b.is_culled(rect(0.0, 0.0, 10.0, 10.0)));
    assert!(!b.is_culled(rect(-200.0, 0.0, 10.0, 10.0)));
    b.pop();

    b.push_transform(Affine::scale(10.0));
    assert!(!b.is_culled(rect(9.0, 9.0, 0.5, 0.5)));
    assert!(b.is_culled(rect(10.0, 10.0, 1.0, 1.0)));
    b.pop();

    // A clip pushed under a transform is mapped by that transform.
    b.push_transform(Affine::translate(Vec2::new(50.0, 50.0)));
    b.push_clip(Clip::Rect(rect(0.0, 0.0, 10.0, 10.0))); // visible: 50..60
    assert!(!b.is_culled(rect(5.0, 5.0, 1.0, 1.0)));
    assert!(b.is_culled(rect(20.0, 0.0, 5.0, 5.0)));
    b.push_transform(Affine::translate(Vec2::new(-50.0, -50.0)));
    assert!(!b.is_culled(rect(55.0, 55.0, 1.0, 1.0)));
    assert!(b.is_culled(rect(0.0, 0.0, 10.0, 10.0)));
    b.pop();
    b.pop();
    b.pop();

    // Clips intersect; popping restores the outer visible area.
    b.push_clip(Clip::Rect(rect(0.0, 0.0, 50.0, 100.0)));
    b.push_clip(Clip::RoundedRect(RoundedRect::new(
        rect(25.0, 0.0, 75.0, 100.0),
        BorderRadius::circular(5.0),
    )));
    assert!(b.is_culled(rect(0.0, 0.0, 20.0, 20.0)));
    assert!(b.is_culled(rect(60.0, 0.0, 20.0, 20.0)));
    assert!(!b.is_culled(rect(30.0, 0.0, 10.0, 10.0)));
    b.pop();
    assert!(!b.is_culled(rect(0.0, 0.0, 20.0, 20.0)));
    b.pop();
    assert!(!b.is_culled(rect(60.0, 0.0, 20.0, 20.0)));
    b.finish().unwrap();

    // An empty scene shows nothing.
    let b = scene.begin(Size::ZERO);
    assert!(b.is_culled(rect(0.0, 0.0, 10.0, 10.0)));
}

#[test]
fn scene_scene_18_is_culled_is_conservative() {
    let nan = f32::NAN;
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);

    // Rotated by 45° about (0, 0), the thin strip (0, -100, 200, 1) becomes a diagonal line
    // along y ≈ x - 141, above the scene, but its bounding box overlaps the scene: not culled.
    b.push_transform(Affine::rotate(std::f32::consts::FRAC_PI_4));
    assert!(!b.is_culled(rect(0.0, -100.0, 200.0, 1.0)));
    b.pop();

    // Zero opacity doesn't cull.
    b.push_layer(Layer {
        opacity: 0.0,
        overlay_color: None,
    });
    assert!(!b.is_culled(rect(10.0, 10.0, 10.0, 10.0)));
    b.pop();

    // Empty and non-finite bounds are culled.
    assert!(b.is_culled(rect(10.0, 10.0, 0.0, 10.0)));
    assert!(b.is_culled(Rect::from_ltrb(20.0, 20.0, 10.0, 10.0)));
    assert!(b.is_culled(Rect::from_ltrb(nan, 0.0, 10.0, 10.0)));
    assert!(b.is_culled(Rect::from_ltrb(0.0, 0.0, f32::INFINITY, 10.0)));

    // A non-finite transform or clip culls everything under it.
    b.push_transform(Affine::new([1.0, 0.0, 0.0, 1.0, nan, 0.0]));
    assert!(b.is_culled(rect(10.0, 10.0, 10.0, 10.0)));
    b.pop();
    b.push_clip(Clip::Rect(Rect::from_ltrb(0.0, 0.0, f32::INFINITY, 50.0)));
    assert!(b.is_culled(rect(10.0, 10.0, 10.0, 10.0)));
    b.pop();
    assert!(!b.is_culled(rect(10.0, 10.0, 10.0, 10.0)));
    b.finish().unwrap();
}

#[test]
fn scene_scene_19_culling_never_records_or_drops() {
    let mut scene = Scene::new();
    let mut b = scene.begin(SIZE);
    assert!(b.is_culled(rect(500.0, 500.0, 10.0, 10.0)));
    b.fill_rect(rect(500.0, 500.0, 10.0, 10.0), shade(1));
    assert!(!b.is_culled(rect(0.0, 0.0, 10.0, 10.0)));
    b.finish().unwrap();
    assert_eq!(names(&scene), ["fill1"]);
}

// ---- Damage ------------------------------------------------------------------------------

#[test]
fn scene_scene_20_damage() {
    let a = rect(0.0, 0.0, 10.0, 10.0);
    let c = rect(50.0, 50.0, -5.0, 10.0);
    let mut scene = Scene::new();

    scene.begin(SIZE).finish().unwrap();
    assert_eq!(scene.damage(), Damage::Full);

    let mut b = scene.begin(SIZE);
    b.add_damage(a);
    b.add_damage(c);
    b.finish().unwrap();
    assert_eq!(scene.damage(), Damage::Rects(&[a, c]));

    let mut b = scene.begin(SIZE);
    b.add_damage(a);
    b.damage_all();
    b.add_damage(c);
    b.finish().unwrap();
    assert_eq!(scene.damage(), Damage::Full);
}

// ---- Handles and values ------------------------------------------------------------------

#[test]
fn scene_scene_21_handles_round_trip() {
    let mut arena = Arena::new();
    let id = arena.insert(());
    assert_eq!(ElementId::from(id).to_raw(), id.to_bits());

    for x in [1, 2, 0xFFFF_FFFF, u64::MAX] {
        assert_eq!(ElementId::from_raw(x).unwrap().to_raw(), x);
        assert_eq!(ImageId::from_raw(x).unwrap().to_raw(), x);
        assert_eq!(FontId::from_raw(x).unwrap().to_raw(), x);
    }
    assert_eq!(ElementId::from_raw(0), None);
    assert_eq!(ImageId::from_raw(0), None);
    assert_eq!(FontId::from_raw(0), None);
    assert_eq!(size_of::<Option<ElementId>>(), 8);
}

#[test]
fn scene_scene_22_layer_and_radius_defaults() {
    let layer = Layer::default();
    assert_eq!(layer.opacity, 1.0);
    assert_eq!(layer.overlay_color, None);
    assert_eq!(
        BorderRadius::circular(3.0),
        BorderRadius {
            top_left: 3.0,
            top_right: 3.0,
            bottom_right: 3.0,
            bottom_left: 3.0
        }
    );
}

fn assert_plain_data<T: Clone + Debug + PartialEq + Send + Sync + 'static>() {}

#[test]
fn scene_scene_23_scenes_are_comparable_plain_data() {
    let mut a = Scene::new();
    let mut b = Scene::new();
    record_everything(&mut a);
    record_everything(&mut b);
    assert_eq!(a, b);
    assert_eq!(a.clone(), a);

    // A different frame compares unequal.
    let mut c = b.begin(SIZE);
    c.fill_rect(rect(0.0, 0.0, 1.0, 1.0), shade(1));
    c.finish().unwrap();
    assert_ne!(a, b);
    assert!(!format!("{b:?}").is_empty());

    assert_plain_data::<Scene>();
    assert_plain_data::<Entry>();
    assert_plain_data::<Command>();
    assert_plain_data::<Clip>();
    assert_plain_data::<Layer>();
    assert_plain_data::<BoxShadow>();
    assert_plain_data::<ImageDraw>();
    assert_plain_data::<tantu_scene::GlyphRun>();
    assert_plain_data::<tantu_scene::CustomDraw>();
}

/// Not a promise to users (see the spec's performance section), but a budget worth watching.
#[test]
fn entry_size_budget() {
    assert!(
        size_of::<Entry>() <= 96,
        "Entry is {} bytes",
        size_of::<Entry>()
    );
}
