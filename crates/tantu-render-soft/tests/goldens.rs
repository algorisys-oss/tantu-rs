//! Golden images for the software renderer (spec `docs/specs/render-soft/renderer.md`,
//! "Golden tests", and `docs/specs/render-conformance/conformance.md`, RENDER-CONF-07 and 11).
//!
//! The reference Scenes and their goldens live in `tantu-render-conformance`. This renderer
//! produces the goldens, so it is held to them strictly: tolerance 2 per channel, no differing
//! pixels. Set `TANTU_UPDATE_GOLDENS=1` to write the PNGs instead. On a mismatch the actual
//! image is written next to the golden as `<name>.actual.png` (gitignored).

mod common;

use common::*;
use tantu_core::Color;
use tantu_render_conformance::{MatchTolerance, ReferenceScene, reference_scene, reference_scenes};
use tantu_render_soft::{diff_images, encode_png};
use tantu_scene::{BorderRadius, ImageData, Resources, RoundedRect, Scene};

const TOLERANCE: u8 = 2;

/// Renders `reference` at its target size and scale factor.
fn render_reference(reference: &ReferenceScene) -> ImageData {
    let (scene, resources) = reference.record();
    render_target(reference, &scene, &resources)
}

fn render_target(reference: &ReferenceScene, scene: &Scene, resources: &Resources) -> ImageData {
    let (w, h) = reference.target_size();
    let (image, report) = render_with(w, h, reference.scale_factor(), scene, resources);
    assert!(report.is_clean(), "{}: {report:?}", reference.name());
    image
}

#[test]
fn render_conf_11_soft_reproduces_every_golden() {
    let update = std::env::var_os("TANTU_UPDATE_GOLDENS").is_some_and(|v| v == "1");
    let mut failures = Vec::new();
    for reference in reference_scenes() {
        let image = render_reference(reference);
        let path = reference.golden_path();
        if update {
            std::fs::write(&path, encode_png(&image).expect("encode")).expect("write golden");
            continue;
        }
        let golden = reference.golden().expect("embedded golden decodes");
        let diff = diff_images(&golden, &image, TOLERANCE).expect("golden has the same size");
        if diff.differing_pixels > 0 {
            let actual = path.with_extension("actual.png");
            std::fs::write(&actual, encode_png(&image).expect("encode")).expect("write actual");
            failures.push(format!(
                "{}: {} pixels differ by more than {TOLERANCE} (max {}); actual written to {}",
                reference.name(),
                diff.differing_pixels,
                diff.max_channel_delta,
                actual.display()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const RED: Color = Color::from_rgb8(220, 40, 40);
const BLUE: Color = Color::from_rgb8(40, 80, 220);
const GREEN: Color = Color::from_rgb8(40, 180, 90);

/// One change to the `shapes_and_strokes` Scene.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mutation {
    None,
    /// The red rounded rect moved 1 logical pixel right.
    Moved,
    /// The red rounded rect's radius 8 → 12.
    Radius,
    /// The blue fill's red channel 40 → 52.
    Color,
    /// The green stroke 3 → 4 logical pixels wide.
    StrokeWidth,
}

/// A copy of the `shapes_and_strokes` reference Scene with `mutation` applied. With
/// `Mutation::None` it must reproduce the golden exactly, so this copy can't drift from the
/// reference.
fn shapes_and_strokes(mutation: Mutation) -> Scene {
    let m = |which: Mutation| mutation == which;
    scene(100.0, 100.0, |b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::WHITE);
        b.fill(
            RoundedRect::new(
                r(if m(Mutation::Moved) { 9.0 } else { 8.0 }, 8.0, 40.0, 30.0),
                BorderRadius::circular(if m(Mutation::Radius) { 12.0 } else { 8.0 }),
            ),
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
            if m(Mutation::Color) {
                Color::from_rgb8(52, 80, 220)
            } else {
                BLUE
            },
        );
        b.stroke(
            RoundedRect::new(r(8.0, 50.0, 40.0, 40.0), BorderRadius::circular(12.0)),
            if m(Mutation::StrokeWidth) { 4.0 } else { 3.0 },
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
    })
}

#[test]
fn render_conf_07_cross_backend_tolerance_catches_real_errors() {
    let reference = reference_scene("shapes_and_strokes").expect("exists");
    let golden = reference.golden().expect("decodes");
    let resources = Resources::new();

    let unchanged = render_target(reference, &shapes_and_strokes(Mutation::None), &resources);
    let diff = diff_images(&golden, &unchanged, TOLERANCE).expect("same size");
    assert_eq!(
        diff.differing_pixels, 0,
        "the copy drifted from the reference Scene"
    );

    for mutation in [
        Mutation::Moved,
        Mutation::Radius,
        Mutation::Color,
        Mutation::StrokeWidth,
    ] {
        let image = render_target(reference, &shapes_and_strokes(mutation), &resources);
        let m = reference
            .check(&image, &MatchTolerance::CROSS_BACKEND)
            .expect("same size");
        assert!(!m.passed, "{mutation:?} passed: {m:?}");
    }
}
