//! Tests for `docs/specs/examples/scene-window.md`, rules SCENE-WINDOW-01..03.

use scene_window::{demo_image, demo_scene};
use tantu_core::Size;
use tantu_render_soft::SoftRenderer;
use tantu_scene::{RenderReport, Renderer, Resources, Scene};

#[test]
fn scene_window_01_demo_image() {
    let image = demo_image();
    assert_eq!((image.width(), image.height()), (64, 64));
    assert!(image.pixels().chunks_exact(4).all(|p| p[3] == 255));
}

#[test]
fn scene_window_02_frames_are_clean_at_any_size_and_time() {
    let mut resources = Resources::new();
    let image = resources.add_image(demo_image());
    let mut scene = Scene::new();
    for (w, h) in [
        (1.0, 1.0),
        (40.0, 30.0),
        (900.0, 600.0),
        (4000.0, 3000.0),
        (3000.0, 50.0),
    ] {
        for time in [0.0, 0.016, 1.5, 1000.0, -3.0] {
            demo_scene(&mut scene, Size::new(w, h), time, image);
            assert_eq!(scene.size(), Size::new(w, h));
            assert!(!scene.entries().is_empty());
            let report = RenderReport::for_scene(&scene, &resources, &|_| false);
            assert!(report.is_clean(), "{w}×{h} at {time}: {report:?}");
            // Balanced: every push has its pop.
            let pushes = scene
                .entries()
                .iter()
                .filter(|e| format!("{:?}", e.command).starts_with("Push"))
                .count();
            let pops = scene
                .entries()
                .iter()
                .filter(|e| format!("{:?}", e.command).starts_with("Pop"))
                .count();
            assert_eq!(pushes, pops);
        }
    }
}

#[test]
fn scene_window_03_renders_full_window() {
    let mut resources = Resources::new();
    let image = resources.add_image(demo_image());
    for (physical, scale) in [((900, 600), 1.0), ((900, 600), 2.0)] {
        let logical = Size::new(physical.0 as f32 / scale, physical.1 as f32 / scale);
        let mut scene = Scene::new();
        demo_scene(&mut scene, logical, 0.5, image);
        let mut renderer = SoftRenderer::new(physical.0, physical.1);
        renderer.resize(physical.0, physical.1, scale);
        let report = renderer.render(&scene, &resources).unwrap();
        assert!(report.is_clean(), "{report:?}");
        let frame = renderer.snapshot().unwrap();
        let px = |x: u32, y: u32| {
            let i = ((y * frame.width() + x) * 4) as usize;
            [
                frame.pixels()[i],
                frame.pixels()[i + 1],
                frame.pixels()[i + 2],
                frame.pixels()[i + 3],
            ]
        };
        for (x, y) in [
            (0, 0),
            (physical.0 - 1, 0),
            (0, physical.1 - 1),
            (physical.0 - 1, physical.1 - 1),
        ] {
            assert_eq!(px(x, y)[3], 255, "corner ({x}, {y}) at scale {scale}");
        }
        let first = px(0, 0);
        assert!(
            frame.pixels().chunks_exact(4).any(|p| p != first),
            "not a single color"
        );
    }
}
