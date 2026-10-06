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

// ---- Shared report counting (RenderReport::for_scene) ------------------------------------

use tantu_core::{Affine, Point, Vec2};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, CustomKind, FontData, FontId, Glyph, ImageData, ImageDraw,
    ImageId, ImageSampling, Layer, RoundedRect, SceneBuilder,
};

const NAN: f32 = f32::NAN;

fn r(l: f32, t: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(l, t, w, h)
}

/// A frame recorded by `paint`.
fn scene_with(paint: impl FnOnce(&mut SceneBuilder<'_>)) -> Scene {
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(100.0, 100.0));
    paint(&mut b);
    b.finish().expect("tests record balanced scopes");
    scene
}

/// Records some commands into a frame.
type Paint = Box<dyn Fn(&mut SceneBuilder<'_>)>;

fn no_custom(_: CustomKind) -> bool {
    false
}

fn report(scene: &Scene, resources: &Resources) -> RenderReport {
    RenderReport::for_scene(scene, resources, &no_custom)
}

fn image_draw(image: ImageId) -> ImageDraw {
    ImageDraw {
        image,
        src: None,
        dest: r(0.0, 0.0, 10.0, 10.0),
        sampling: ImageSampling::Linear,
        opacity: 1.0,
    }
}

fn shadow() -> BoxShadow {
    BoxShadow {
        shape: RoundedRect::from_rect(r(0.0, 0.0, 10.0, 10.0)),
        color: Color::BLACK,
        offset: Vec2::ZERO,
        blur_radius: 2.0,
        spread_radius: 0.0,
    }
}

/// Resources with one image and one font.
fn resources() -> (Resources, ImageId, FontId) {
    let mut res = Resources::new();
    let image = res.add_image(ImageData::rgba8(1, 1, vec![0u8; 4]).expect("1×1 RGBA8"));
    let font = res.add_font(FontData::new(vec![1u8], 0).expect("non-empty"));
    (res, image, font)
}

#[test]
fn scene_render_04_non_finite_scopes_hide_their_contents() {
    let (res, _, _) = resources();
    let missing = ImageId::from_raw(u64::MAX).unwrap();
    let scene = scene_with(|b| {
        b.push_transform(Affine::new([1.0, 0.0, 0.0, 1.0, NAN, 0.0]));
        b.image(image_draw(missing)); // hidden: not counted as missing
        b.fill_rect(Rect::from_ltrb(NAN, 0.0, 1.0, 1.0), Color::BLACK); // hidden: not invalid
        b.push_clip(Clip::Rect(Rect::from_ltrb(NAN, 0.0, 1.0, 1.0))); // hidden: not counted
        b.pop();
        b.pop();
        b.image(image_draw(missing)); // after the scope: counted
        b.push_clip(Clip::RoundedRect(RoundedRect::new(
            r(0.0, 0.0, 10.0, 10.0),
            BorderRadius::circular(f32::INFINITY),
        )));
        b.custom(CustomKind(1), r(0.0, 0.0, 1.0, 1.0), &[]); // hidden
        b.pop();
        b.push_layer(Layer {
            opacity: NAN,
            overlay_color: Some(Color::new(NAN, 0.0, 0.0, 1.0)),
        });
        b.image(image_draw(missing)); // layers never hide or count
        b.pop();
        b.push_transform(Affine::scale(2.0));
        b.push_clip(Clip::Rect(r(0.0, 0.0, 10.0, 10.0)));
        b.image(image_draw(missing)); // finite scopes: counted
        b.pop();
        b.pop();
    });
    assert_eq!(
        report(&scene, &res),
        RenderReport {
            missing_images: 3,
            invalid_commands: 2,
            ..RenderReport::default()
        }
    );
}

#[test]
fn scene_render_05_non_finite_draw_commands_are_invalid() {
    let (res, image, font) = resources();
    let missing_image = ImageId::from_raw(u64::MAX).unwrap();
    let missing_font = FontId::from_raw(u64::MAX).unwrap();
    let ok = r(0.0, 0.0, 10.0, 10.0);
    let bad = Rect::from_ltrb(0.0, 0.0, f32::INFINITY, 1.0);
    let glyph = Glyph {
        id: 1,
        x: 0.0,
        y: 0.0,
    };

    // Each closure records one draw command with one non-finite field.
    let cases: Vec<Paint> = vec![
        Box::new(move |b| b.fill_rect(bad, Color::BLACK)),
        Box::new(move |b| {
            b.fill(
                RoundedRect::new(ok, BorderRadius::circular(NAN)),
                Color::BLACK,
            )
        }),
        Box::new(move |b| b.fill_rect(ok, Color::new(0.0, 0.0, 0.0, NAN))),
        Box::new(move |b| b.stroke(RoundedRect::from_rect(ok), NAN, Color::BLACK)),
        Box::new(move |b| b.stroke(RoundedRect::from_rect(bad), 1.0, Color::BLACK)),
        Box::new(move |b| {
            b.box_shadow(BoxShadow {
                offset: Vec2::new(NAN, 0.0),
                ..shadow()
            })
        }),
        Box::new(move |b| {
            b.box_shadow(BoxShadow {
                blur_radius: NAN,
                ..shadow()
            })
        }),
        Box::new(move |b| {
            b.box_shadow(BoxShadow {
                spread_radius: f32::INFINITY,
                ..shadow()
            })
        }),
        Box::new(move |b| {
            b.box_shadow(BoxShadow {
                color: Color::new(NAN, 0.0, 0.0, 1.0),
                ..shadow()
            })
        }),
        Box::new(move |b| {
            b.box_shadow(BoxShadow {
                shape: RoundedRect::from_rect(bad),
                ..shadow()
            })
        }),
        // Invalid wins over missing: a missing image with a NaN dest is invalid only.
        Box::new(move |b| {
            b.image(ImageDraw {
                dest: bad,
                ..image_draw(missing_image)
            })
        }),
        Box::new(move |b| {
            b.image(ImageDraw {
                src: Some(bad),
                ..image_draw(image)
            })
        }),
        Box::new(move |b| {
            b.image(ImageDraw {
                opacity: NAN,
                ..image_draw(image)
            })
        }),
        Box::new(move |b| b.glyph_run(missing_font, NAN, Color::BLACK, Point::ZERO, &[glyph])),
        Box::new(move |b| {
            b.glyph_run(
                font,
                12.0,
                Color::new(0.0, NAN, 0.0, 1.0),
                Point::ZERO,
                &[glyph],
            )
        }),
        Box::new(move |b| b.glyph_run(font, 12.0, Color::BLACK, Point::new(NAN, 0.0), &[glyph])),
        Box::new(move |b| {
            b.glyph_run(
                font,
                12.0,
                Color::BLACK,
                Point::ZERO,
                &[
                    glyph,
                    Glyph {
                        id: 2,
                        x: 1.0,
                        y: NAN,
                    },
                ],
            )
        }),
        Box::new(move |b| b.custom(CustomKind(9), bad, &[])),
    ];
    for (i, paint) in cases.iter().enumerate() {
        let scene = scene_with(|b| paint(b));
        assert_eq!(
            report(&scene, &res),
            RenderReport {
                invalid_commands: 1,
                ..RenderReport::default()
            },
            "case {i}"
        );
    }
}

#[test]
fn scene_render_06_missing_resources_and_unhandled_custom() {
    let (res, image, font) = resources();
    let missing_image = ImageId::from_raw(u64::MAX).unwrap();
    let missing_font = FontId::from_raw(u64::MAX).unwrap();
    let glyph = Glyph {
        id: 1,
        x: 0.0,
        y: 0.0,
    };
    let scene = scene_with(|b| {
        b.image(image_draw(image));
        b.image(image_draw(missing_image));
        b.image(image_draw(missing_image));
        b.glyph_run(font, 12.0, Color::BLACK, Point::ZERO, &[glyph]);
        b.glyph_run(missing_font, 12.0, Color::BLACK, Point::ZERO, &[glyph]);
        b.custom(CustomKind(1), r(0.0, 0.0, 1.0, 1.0), &[1]);
        b.custom(CustomKind(2), r(0.0, 0.0, 1.0, 1.0), &[2]);
        b.custom(CustomKind(2), r(0.0, 0.0, 1.0, 1.0), &[3]);
    });
    let handles_one = |kind: CustomKind| kind == CustomKind(1);
    assert_eq!(
        RenderReport::for_scene(&scene, &res, &handles_one),
        RenderReport {
            missing_images: 2,
            missing_fonts: 1,
            unhandled_custom: 2,
            invalid_commands: 0
        }
    );
    // Everything present and handled: clean.
    let clean = scene_with(|b| {
        b.image(image_draw(image));
        b.glyph_run(font, 12.0, Color::BLACK, Point::ZERO, &[glyph]);
        b.custom(CustomKind(1), r(0.0, 0.0, 1.0, 1.0), &[]);
    });
    assert!(RenderReport::for_scene(&clean, &res, &handles_one).is_clean());
}

#[test]
fn scene_render_07_other_odd_values_are_valid() {
    let (res, _, font) = resources();
    let scene = scene_with(|b| {
        b.fill_rect(r(0.0, 0.0, 0.0, 0.0), Color::BLACK);
        b.fill_rect(Rect::from_ltrb(10.0, 10.0, 0.0, 0.0), Color::BLACK);
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), Color::TRANSPARENT);
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), Color::new(2.0, -1.0, 0.5, 3.0));
        b.stroke(
            RoundedRect::new(r(0.0, 0.0, 10.0, 10.0), BorderRadius::circular(-4.0)),
            -1.0,
            Color::BLACK,
        );
        b.box_shadow(BoxShadow {
            blur_radius: -3.0,
            spread_radius: -50.0,
            ..shadow()
        });
        b.push_layer(Layer {
            opacity: 0.0,
            overlay_color: None,
        });
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), Color::BLACK);
        b.pop();
        b.push_layer(Layer {
            opacity: NAN,
            overlay_color: None,
        });
        b.pop();
        b.glyph_run(font, 12.0, Color::BLACK, Point::ZERO, &[]);
        b.glyph_run(font, -12.0, Color::BLACK, Point::ZERO, &[]);
        b.push_transform(Affine::scale(0.0));
        b.push_clip(Clip::Rect(Rect::ZERO));
        b.fill_rect(r(0.0, 0.0, 10.0, 10.0), Color::BLACK);
        b.pop();
        b.pop();
    });
    assert!(
        report(&scene, &res).is_clean(),
        "{:?}",
        report(&scene, &res)
    );
}

#[test]
fn scene_render_08_counts_saturate_and_never_panic() {
    // Saturation can't be reached by recording 2^32 commands in a test; check that counting
    // a large, mixed Scene is exact, then that empty and foreign inputs don't panic.
    let missing = ImageId::from_raw(u64::MAX).unwrap();
    let scene = scene_with(|b| {
        for _ in 0..10_000 {
            b.image(image_draw(missing));
            b.fill_rect(Rect::from_ltrb(NAN, 0.0, 1.0, 1.0), Color::BLACK);
        }
    });
    let res = Resources::new();
    assert_eq!(
        report(&scene, &res),
        RenderReport {
            missing_images: 10_000,
            invalid_commands: 10_000,
            ..RenderReport::default()
        }
    );
    assert!(report(&Scene::new(), &res).is_clean());

    // A glyph run whose glyphs came from another Scene still counts without panicking.
    let font = FontId::from_raw(1).unwrap();
    let big = scene_with(|b| {
        b.glyph_run(
            font,
            12.0,
            Color::BLACK,
            Point::ZERO,
            &[Glyph {
                id: 0,
                x: 0.0,
                y: 0.0,
            }; 50],
        );
    });
    let small = scene_with(|b| {
        b.glyph_run(
            font,
            12.0,
            Color::BLACK,
            Point::ZERO,
            &[Glyph {
                id: 0,
                x: NAN,
                y: 0.0,
            }],
        );
    });
    let _ = report(&big, &res);
    assert_eq!(report(&small, &res).invalid_commands, 1);
}
