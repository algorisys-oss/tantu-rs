//! Tests for `docs/specs/scene/renderer.md`, rules SCENE-RENDER-NN (renderer types).

use std::error::Error;

use tantu_core::{Color, Rect, Size};
use tantu_scene::{RenderError, RenderReport, Renderer, ResourceError, Resources, Scene};

#[test]
fn scene_render_01_report_is_clean() {
    let report = RenderReport::default();
    assert_eq!(
        report,
        RenderReport {
            missing_images: 0,
            missing_fonts: 0,
            unhandled_custom: 0,
            invalid_commands: 0
        }
    );
    assert!(report.is_clean());
    for dirty in [
        RenderReport {
            missing_images: 1,
            ..report
        },
        RenderReport {
            missing_fonts: 1,
            ..report
        },
        RenderReport {
            unhandled_custom: 1,
            ..report
        },
        RenderReport {
            invalid_commands: 1,
            ..report
        },
    ] {
        assert!(!dirty.is_clean(), "{dirty:?}");
    }
}

/// Counts entries and remembers the last size it was given.
#[derive(Default)]
struct Counting {
    size: (u32, u32, f32),
    entries: usize,
}

impl Renderer for Counting {
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.size = (width, height, scale_factor);
    }

    fn render(&mut self, scene: &Scene, _: &Resources) -> Result<RenderReport, RenderError> {
        self.entries += scene.entries().len();
        Ok(RenderReport::default())
    }
}

#[test]
fn scene_render_02_renderer_is_object_safe() {
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(10.0, 10.0));
    b.fill_rect(Rect::from_ltwh(0.0, 0.0, 5.0, 5.0), Color::BLACK);
    b.finish().unwrap();

    let mut renderers: Vec<Box<dyn Renderer>> = vec![Box::new(Counting::default())];
    for r in &mut renderers {
        r.resize(20, 20, 2.0);
        assert!(r.render(&scene, &Resources::new()).unwrap().is_clean());
    }
}

#[test]
fn scene_render_03_error_messages_and_source() {
    for err in [
        RenderError::TargetLost,
        RenderError::OutOfMemory,
        RenderError::Backend(Box::new(ResourceError::EmptyFontData)),
    ] {
        assert!(!err.to_string().is_empty());
    }
    for err in [
        ResourceError::EmptyFontData,
        ResourceError::InvalidImageSize {
            width: 0,
            height: 1,
        },
        ResourceError::PixelDataLength {
            expected: 4,
            actual: 3,
        },
    ] {
        assert!(!err.to_string().is_empty());
    }

    let err = RenderError::Backend(Box::new(ResourceError::EmptyFontData));
    let source = err.source().expect("Backend has a source");
    assert_eq!(
        source.downcast_ref::<ResourceError>(),
        Some(&ResourceError::EmptyFontData)
    );
    assert!(RenderError::TargetLost.source().is_none());
}
