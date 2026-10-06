//! Tests for `docs/specs/render-headless/recorder.md`, one or more per rule RENDER-HEADLESS-NN.

use tantu_core::{Color, Rect, Size};
use tantu_render_headless::{HeadlessRenderer, RecordedFrame};
use tantu_scene::{
    Command, CustomKind, ElementId, ImageData, ImageDraw, ImageId, ImageSampling, RenderError,
    RenderReport, Renderer, Resources, Scene, SceneBuilder,
};

fn r(l: f32, t: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(l, t, w, h)
}

fn el(raw: u64) -> ElementId {
    ElementId::from_raw(raw).expect("non-zero")
}

/// A frame recorded by `paint`.
fn scene_with(paint: impl FnOnce(&mut SceneBuilder<'_>)) -> Scene {
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(100.0, 100.0));
    paint(&mut b);
    b.finish().expect("tests record balanced scopes");
    scene
}

/// A Scene with one image (missing from any `Resources`) and custom commands of kinds 1 and 2.
fn scene_with_problems() -> Scene {
    scene_with(|b| {
        b.image(ImageDraw {
            image: ImageId::from_raw(u64::MAX).expect("non-zero"),
            src: None,
            dest: r(0.0, 0.0, 10.0, 10.0),
            sampling: ImageSampling::Linear,
            opacity: 1.0,
        });
        b.custom(CustomKind(1), r(0.0, 0.0, 5.0, 5.0), &[]);
        b.custom(CustomKind(2), r(0.0, 0.0, 5.0, 5.0), &[]);
        b.fill_rect(Rect::from_ltrb(f32::NAN, 0.0, 1.0, 1.0), Color::BLACK);
    })
}

#[test]
fn render_headless_01_new_renderer() {
    let renderer = HeadlessRenderer::new(640, 480);
    assert_eq!(renderer.size(), (640, 480));
    assert_eq!(renderer.scale_factor(), 1.0);
    assert!(renderer.frames().is_empty());
    assert!(renderer.last_frame().is_none());
    assert_eq!(renderer.frame_count(), 0);
    assert_eq!(HeadlessRenderer::new(0, 0).size(), (0, 0));
}

#[test]
fn render_headless_02_resize() {
    let mut renderer = HeadlessRenderer::new(10, 10);
    renderer.resize(300, 200, 1.5);
    assert_eq!(renderer.size(), (300, 200));
    assert_eq!(renderer.scale_factor(), 1.5);
    for bad in [0.0, -2.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        renderer.resize(30, 20, bad);
        assert_eq!(renderer.size(), (30, 20));
        assert_eq!(renderer.scale_factor(), 1.0, "scale factor {bad}");
    }
}

#[test]
fn render_headless_03_render_records_a_frame() {
    let mut res = Resources::new();
    res.add_image(ImageData::rgba8(1, 1, vec![0u8; 4]).expect("1×1 RGBA8"));
    let mut scene = scene_with(|b| b.fill_rect(r(0.0, 0.0, 10.0, 10.0), Color::WHITE));
    let mut renderer = HeadlessRenderer::new(200, 100);
    renderer.resize(200, 100, 2.0);

    let report = renderer.render(&scene, &res).expect("headless never fails");
    assert_eq!(renderer.frame_count(), 1);
    let frame = renderer.last_frame().expect("one frame").clone();
    assert_eq!(
        frame,
        RecordedFrame {
            scene: scene.clone(),
            width: 200,
            height: 100,
            scale_factor: 2.0,
            resources_revision: 1,
            report,
        }
    );

    // Recording the next frame into the same Scene leaves the copy alone.
    scene
        .begin(Size::new(1.0, 1.0))
        .finish()
        .expect("empty frame");
    assert_eq!(renderer.last_frame(), Some(&frame));
    assert_ne!(frame.scene, scene);
}

#[test]
fn render_headless_04_report_matches_shared_counting() {
    let scene = scene_with_problems();
    let res = Resources::new();
    let mut renderer = HeadlessRenderer::new(100, 100);
    renderer.register_custom(CustomKind(2));

    let report = renderer.render(&scene, &res).expect("headless never fails");
    let expected = RenderReport::for_scene(&scene, &res, &|kind| kind == CustomKind(2));
    assert_eq!(report, expected);
    assert_eq!(
        report,
        RenderReport {
            missing_images: 1,
            missing_fonts: 0,
            unhandled_custom: 1,
            invalid_commands: 1
        }
    );
    assert_eq!(renderer.last_frame().map(|f| f.report), Some(report));
}

#[test]
fn render_headless_05_zero_size_target() {
    let scene = scene_with_problems();
    for (w, h) in [(0, 100), (100, 0), (0, 0)] {
        let mut renderer = HeadlessRenderer::new(w, h);
        let report = renderer
            .render(&scene, &Resources::new())
            .expect("never fails");
        assert_eq!(report, RenderReport::default());
        let frame = renderer.last_frame().expect("still recorded");
        assert_eq!((frame.width, frame.height), (w, h));
        assert_eq!(frame.report, RenderReport::default());
        // The Scene holds a NaN on purpose, so compare its Debug output (NaN != NaN).
        assert_eq!(format!("{:?}", frame.scene), format!("{scene:?}"));
    }
}

#[test]
fn render_headless_06_register_custom() {
    let scene = scene_with(|b| {
        b.custom(CustomKind(1), r(0.0, 0.0, 5.0, 5.0), &[]);
        b.custom(CustomKind(2), r(0.0, 0.0, 5.0, 5.0), &[]);
        b.custom(CustomKind(1), r(0.0, 0.0, 5.0, 5.0), &[]);
    });
    let res = Resources::new();
    let mut renderer = HeadlessRenderer::new(10, 10);
    let unhandled = |renderer: &mut HeadlessRenderer| {
        renderer
            .render(&scene, &res)
            .expect("never fails")
            .unhandled_custom
    };
    assert_eq!(unhandled(&mut renderer), 3);
    renderer.register_custom(CustomKind(1));
    assert_eq!(unhandled(&mut renderer), 1);
    renderer.register_custom(CustomKind(1));
    assert_eq!(unhandled(&mut renderer), 1);
    renderer.register_custom(CustomKind(2));
    assert_eq!(unhandled(&mut renderer), 0);
}

#[test]
fn render_headless_07_fail_next_render() {
    let scene = scene_with(|b| b.fill_rect(r(0.0, 0.0, 1.0, 1.0), Color::BLACK));
    let res = Resources::new();
    let mut renderer = HeadlessRenderer::new(10, 10);
    renderer.render(&scene, &res).expect("first frame");

    renderer.fail_next_render(RenderError::OutOfMemory);
    renderer.fail_next_render(RenderError::TargetLost); // replaces the first
    assert!(matches!(
        renderer.render(&scene, &res),
        Err(RenderError::TargetLost)
    ));
    assert_eq!(renderer.frame_count(), 1);
    assert_eq!(renderer.frames().len(), 1);

    assert!(renderer.render(&scene, &res).is_ok());
    assert_eq!(renderer.frame_count(), 2);
}

#[test]
fn render_headless_08_frames_and_taking() {
    let res = Resources::new();
    let mut renderer = HeadlessRenderer::new(10, 10);
    let scenes: Vec<Scene> = (1..=3)
        .map(|i| scene_with(|b| b.fill_rect(r(0.0, 0.0, i as f32, 1.0), Color::BLACK)))
        .collect();
    for scene in &scenes {
        renderer.render(scene, &res).expect("never fails");
    }
    let recorded: Vec<&Scene> = renderer.frames().iter().map(|f| &f.scene).collect();
    assert_eq!(recorded, scenes.iter().collect::<Vec<_>>());
    assert_eq!(renderer.last_frame().map(|f| &f.scene), Some(&scenes[2]));

    let taken = renderer.take_frames();
    assert_eq!(
        taken.iter().map(|f| &f.scene).collect::<Vec<_>>(),
        scenes.iter().collect::<Vec<_>>()
    );
    assert!(renderer.frames().is_empty());
    assert!(renderer.last_frame().is_none());
    assert_eq!(renderer.frame_count(), 3);

    renderer.render(&scenes[0], &res).expect("never fails");
    assert_eq!(renderer.frames().len(), 1);
    assert_eq!(renderer.frame_count(), 4);
}

#[test]
fn render_headless_09_entries_for_element() {
    let scene = scene_with(|b| {
        b.set_element(Some(el(1)));
        b.fill_rect(r(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        b.set_element(Some(el(2)));
        b.fill_rect(r(0.0, 0.0, 2.0, 2.0), Color::BLACK);
        b.set_element(None);
        b.fill_rect(r(0.0, 0.0, 3.0, 3.0), Color::BLACK);
        b.set_element(Some(el(1)));
        b.set_z_index(-1);
        b.fill_rect(r(0.0, 0.0, 4.0, 4.0), Color::BLACK);
    });
    let mut renderer = HeadlessRenderer::new(10, 10);
    renderer
        .render(&scene, &Resources::new())
        .expect("never fails");
    let frame = renderer.last_frame().expect("one frame");

    let widths: Vec<f32> = frame
        .entries_for(el(1))
        .map(|e| match e.command {
            Command::Fill { shape, .. } => shape.rect.width(),
            ref other => panic!("expected a fill, got {other:?}"),
        })
        .collect();
    assert_eq!(
        widths,
        [4.0, 1.0],
        "paint order: the z = -1 fill comes first"
    );
    assert_eq!(frame.entries_for(el(2)).count(), 1);
    assert_eq!(frame.entries_for(el(3)).count(), 0);
}

fn assert_send<T: Send>() {}

#[test]
fn render_headless_10_send_and_object_safe() {
    assert_send::<HeadlessRenderer>();
    assert_send::<RecordedFrame>();
    let mut boxed: Box<dyn Renderer> = Box::new(HeadlessRenderer::new(10, 10));
    boxed.resize(20, 20, 1.0);
    let scene = scene_with(|b| b.fill_rect(r(0.0, 0.0, 1.0, 1.0), Color::BLACK));
    assert!(boxed.render(&scene, &Resources::new()).is_ok());
}
