//! Tests for `docs/specs/render-conformance/conformance.md`, rules RENDER-CONF-01..06 and
//! 08..10. RENDER-CONF-07 and 11 are in `tantu-render-soft` (they need a rasterizer),
//! RENDER-CONF-12 in `tantu-render-wgpu` and RENDER-CONF-13 in `scene-window`.

use tantu_render_conformance::{
    GoldenError, ImageMatch, MatchTolerance, match_images, reference_scene, reference_scenes,
};
use tantu_scene::{Command, ImageData, RenderReport};

/// A `w × h` image with every pixel `px`.
fn flat(w: u32, h: u32, px: [u8; 4]) -> ImageData {
    let pixels: Vec<u8> = (0..w * h).flat_map(|_| px).collect();
    ImageData::rgba8(w, h, pixels).expect("valid size")
}

/// A `w × h` image whose pixel at (x, y) is `f(x, y)`.
fn image(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> ImageData {
    let pixels: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| f(x, y))
        .collect();
    ImageData::rgba8(w, h, pixels).expect("valid size")
}

/// Every pixel within tolerance of everything: only the counts are interesting.
const LOOSE: MatchTolerance = MatchTolerance {
    edge_threshold: 8,
    interior: 255,
    edge: 255,
    loose_edge: 255,
    loose_edge_fraction: 1.0,
};

/// Nothing may differ at all.
const STRICT: MatchTolerance = MatchTolerance {
    edge_threshold: 8,
    interior: 0,
    edge: 0,
    loose_edge: 0,
    loose_edge_fraction: 0.0,
};

const WHITE: [u8; 4] = [255, 255, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];

/// White on the left half (x < 5), black on the right, 10 × 10.
fn split() -> ImageData {
    image(10, 10, |x, _| if x < 5 { WHITE } else { BLACK })
}

#[test]
fn render_conf_01_size_mismatch_is_none() {
    let a = flat(4, 4, WHITE);
    assert_eq!(match_images(&a, &flat(4, 5, WHITE), &STRICT), None);
    assert_eq!(match_images(&a, &flat(5, 4, WHITE), &STRICT), None);
    let one = flat(1, 1, WHITE);
    let m = match_images(&one, &one, &STRICT).expect("same size");
    assert!(m.passed);
    assert_eq!(m.edge_pixels, 0);
    // A 1 × 1 image differing by a little is an interior failure, not an edge.
    let m = match_images(&one, &flat(1, 1, [250, 255, 255, 255]), &STRICT).expect("same size");
    assert!(!m.passed);
    assert_eq!(
        (m.edge_pixels, m.interior_failures, m.max_interior_delta),
        (0, 1, 5)
    );
}

#[test]
fn render_conf_02_compares_premultiplied() {
    // Fully transparent pixels never differ, whatever their color channels hold.
    let clear_red = flat(3, 3, [255, 0, 0, 0]);
    let clear_blue = flat(3, 3, [0, 0, 255, 0]);
    let m = match_images(&clear_red, &clear_blue, &STRICT).expect("same size");
    assert!(m.passed, "{m:?}");
    // Alpha 6 white vs. transparent: premultiplied (6, 6, 6, 6) vs. 0, so the difference is 6,
    // not 255.
    let faint = flat(3, 3, [255, 255, 255, 6]);
    let m = match_images(&flat(3, 3, [0, 0, 0, 0]), &faint, &LOOSE).expect("same size");
    assert_eq!(m.max_interior_delta, 6);
    // Rounding: 255 × 128 / 255 = 128; 100 × 128 / 255 = 50.196 → 50; 101 × 128 / 255 =
    // 50.698 → 51.
    let a = flat(3, 3, [100, 0, 0, 128]);
    let b = flat(3, 3, [101, 0, 0, 128]);
    assert_eq!(
        match_images(&a, &b, &LOOSE)
            .expect("same size")
            .max_interior_delta,
        1
    );
    // The difference is the largest over the four channels.
    let a = flat(3, 3, [10, 20, 30, 255]);
    let b = flat(3, 3, [13, 11, 31, 250]);
    assert_eq!(
        match_images(&a, &b, &LOOSE)
            .expect("same size")
            .max_interior_delta,
        9
    );
}

#[test]
fn render_conf_03_edges_come_from_the_reference() {
    // A sharp vertical edge between x = 4 and x = 5 gives a band two pixels wide.
    let m = match_images(&split(), &split(), &LOOSE).expect("same size");
    assert_eq!(m.edge_pixels, 2 * 10);
    // The edge count doesn't depend on the other image.
    let m2 = match_images(&split(), &flat(10, 10, WHITE), &LOOSE).expect("same size");
    assert_eq!(m2.edge_pixels, 20);
    // A flat reference has no edges, whatever the other image looks like.
    let m3 = match_images(&flat(10, 10, WHITE), &split(), &LOOSE).expect("same size");
    assert_eq!(m3.edge_pixels, 0);
    // A change of exactly the threshold is not an edge; one more is.
    let step = |d: u8| {
        image(6, 6, move |x, _| {
            if x < 3 {
                [100, 100, 100, 255]
            } else {
                [100 + d, 100, 100, 255]
            }
        })
    };
    let at = match_images(&step(8), &step(8), &LOOSE).expect("same size");
    assert_eq!(at.edge_pixels, 0);
    let above = match_images(&step(9), &step(9), &LOOSE).expect("same size");
    assert_eq!(above.edge_pixels, 12);
    // Neighbourhoods are cut off at the border: a single odd pixel in a corner makes it and
    // its three neighbours edge pixels.
    let dot = image(5, 5, |x, y| if (x, y) == (0, 0) { BLACK } else { WHITE });
    assert_eq!(
        match_images(&dot, &dot, &LOOSE)
            .expect("same size")
            .edge_pixels,
        4
    );
}

#[test]
fn render_conf_04_pass_conditions_and_counts() {
    let tol = MatchTolerance {
        edge_threshold: 8,
        interior: 3,
        edge: 50,
        loose_edge: 10,
        loose_edge_fraction: 0.1,
    };
    let reference = split();
    // Interior pixels off by 3 pass, by 4 fail.
    let shifted = |d: u8| {
        image(10, 10, move |x, _| {
            if x == 0 {
                [255 - d, 255, 255, 255]
            } else if x < 5 {
                WHITE
            } else {
                BLACK
            }
        })
    };
    let m = match_images(&reference, &shifted(3), &tol).expect("same size");
    assert!(m.passed, "{m:?}");
    assert_eq!(m.max_interior_delta, 3);
    let m = match_images(&reference, &shifted(4), &tol).expect("same size");
    assert!(!m.passed);
    assert_eq!((m.interior_failures, m.max_interior_delta), (10, 4));

    // Edge pixels: 20 of them, allowance floor(0.1 × 20) = 2 loose ones.
    let edge_off = |rows: u32, d: u8| {
        image(10, 10, move |x, y| {
            if x == 4 && y < rows {
                [255 - d, 255 - d, 255 - d, 255]
            } else if x < 5 {
                WHITE
            } else {
                BLACK
            }
        })
    };
    let m = match_images(&reference, &edge_off(2, 20), &tol).expect("same size");
    assert!(m.passed, "{m:?}");
    assert_eq!(
        (
            m.edge_pixels,
            m.loose_edge_pixels,
            m.loose_edge_allowance,
            m.max_edge_delta
        ),
        (20, 2, 2, 20)
    );
    let m = match_images(&reference, &edge_off(3, 20), &tol).expect("same size");
    assert!(!m.passed);
    assert_eq!((m.loose_edge_pixels, m.edge_failures), (3, 0));
    // One edge pixel beyond `edge` fails even within the loose allowance.
    let m = match_images(&reference, &edge_off(1, 51), &tol).expect("same size");
    assert!(!m.passed);
    assert_eq!((m.edge_failures, m.max_edge_delta), (1, 51));

    // The fraction is clamped: NaN and negative as 0, above 1 as 1.
    for (fraction, allowance) in [(f32::NAN, 0), (-0.5, 0), (2.0, 20), (0.0, 0)] {
        let tol = MatchTolerance {
            loose_edge_fraction: fraction,
            ..tol
        };
        let m = match_images(&reference, &edge_off(1, 20), &tol).expect("same size");
        assert_eq!(m.loose_edge_allowance, allowance, "fraction {fraction}");
        assert_eq!(m.passed, allowance >= 1, "fraction {fraction}");
    }
}

#[test]
fn render_conf_05_an_image_matches_itself() {
    let images = [
        split(),
        flat(7, 3, [12, 34, 56, 78]),
        image(16, 16, |x, y| {
            [
                (x * 16) as u8,
                (y * 16) as u8,
                ((x ^ y) * 9) as u8,
                (x * y) as u8,
            ]
        }),
    ];
    for img in &images {
        for tol in [STRICT, LOOSE, MatchTolerance::CROSS_BACKEND] {
            let m = match_images(img, img, &tol).expect("same size");
            assert_eq!(
                m,
                ImageMatch {
                    passed: true,
                    edge_pixels: m.edge_pixels,
                    loose_edge_allowance: m.loose_edge_allowance,
                    ..ImageMatch::default()
                }
            );
        }
    }
}

#[test]
fn render_conf_06_cross_backend_values() {
    let expected = MatchTolerance {
        edge_threshold: 8,
        interior: 10,
        edge: 96,
        loose_edge: 16,
        loose_edge_fraction: 0.05,
    };
    assert_eq!(MatchTolerance::CROSS_BACKEND, expected);
    assert_eq!(MatchTolerance::default(), expected);
}

#[test]
fn render_conf_08_the_reference_set() {
    let names: Vec<&str> = reference_scenes().iter().map(|r| r.name()).collect();
    assert_eq!(
        names,
        [
            "shapes_and_strokes",
            "shadows",
            "images",
            "clips_and_transforms",
            "layers",
            "text"
        ]
    );
    for reference in reference_scenes() {
        assert_eq!(reference.target_size(), (200, 200), "{}", reference.name());
        assert_eq!(reference.scale_factor(), 2.0, "{}", reference.name());
        let found = reference_scene(reference.name()).expect("findable by name");
        assert!(std::ptr::eq(found, reference));
    }
    for other in ["", "Shadows", "shadows ", "text", "shapes"] {
        assert!(reference_scene(other).is_none(), "{other:?}");
    }
}

#[test]
fn render_conf_09_scenes_and_goldens_are_valid() {
    for reference in reference_scenes() {
        let name = reference.name();
        let (scene, resources) = reference.record();
        assert_eq!(scene.size(), tantu_core::Size::new(100.0, 100.0), "{name}");
        assert!(!scene.entries().is_empty(), "{name}");
        let report = RenderReport::for_scene(&scene, &resources, &|_| false);
        assert!(report.is_clean(), "{name}: {report:?}");

        // Balanced scopes: every push has its pop.
        let depth = scene.entries().iter().try_fold(0i32, |depth, e| {
            let depth = match e.command {
                Command::PushClip(_) | Command::PushTransform(_) | Command::PushLayer(_) => {
                    depth + 1
                }
                Command::PopClip | Command::PopTransform | Command::PopLayer => depth - 1,
                _ => depth,
            };
            (depth >= 0).then_some(depth)
        });
        assert_eq!(depth, Some(0), "{name}");

        // Recording again gives the same Scene, apart from fresh image handles.
        let (again, again_resources) = reference.record();
        assert_eq!(scene.entries().len(), again.entries().len(), "{name}");
        for (a, b) in scene.entries().iter().zip(again.entries()) {
            match (&a.command, &b.command) {
                (Command::Image(da), Command::Image(db)) => {
                    assert_eq!(
                        resources.image(da.image),
                        again_resources.image(db.image),
                        "{name}"
                    );
                    let (mut da, db) = (*da, *db);
                    da.image = db.image;
                    assert_eq!(da, db, "{name}");
                    assert_eq!((a.element, a.z_index), (b.element, b.z_index), "{name}");
                }
                (Command::GlyphRun(ga), Command::GlyphRun(gb)) => {
                    assert_eq!(
                        resources.font(ga.font).map(|f| f.bytes().len()),
                        again_resources.font(gb.font).map(|f| f.bytes().len()),
                        "{name}"
                    );
                    let (mut ga, gb) = (ga.clone(), gb.clone());
                    ga.font = gb.font;
                    assert_eq!(ga, gb, "{name}");
                }
                _ => assert_eq!(a, b, "{name}"),
            }
        }

        let golden = reference.golden().expect("embedded golden decodes");
        assert_eq!(
            (golden.width(), golden.height()),
            reference.target_size(),
            "{name}"
        );
        assert!(!reference.golden_png().is_empty());
        assert!(
            reference
                .golden_path()
                .ends_with(format!("goldens/{name}.png")),
            "{name}: {}",
            reference.golden_path().display()
        );
        assert!(
            reference.golden_path().is_file(),
            "{name}: source tree checkout"
        );
    }
}

#[test]
fn render_conf_10_check_uses_the_golden() {
    let reference = reference_scene("layers").expect("exists");
    let golden = reference.golden().expect("decodes");
    let tol = MatchTolerance::CROSS_BACKEND;
    assert_eq!(
        reference.check(&golden, &tol).expect("same size"),
        match_images(&golden, &golden, &tol).expect("same size")
    );
    let blank = flat(200, 200, [0, 0, 0, 0]);
    let m = reference.check(&blank, &tol).expect("same size");
    assert_eq!(m, match_images(&golden, &blank, &tol).expect("same size"));
    assert!(!m.passed);
    match reference.check(&flat(100, 200, WHITE), &tol) {
        Err(GoldenError::SizeMismatch { golden, actual }) => {
            assert_eq!((golden, actual), ((200, 200), (100, 200)));
        }
        other => panic!("expected SizeMismatch, got {other:?}"),
    }
}

#[test]
fn render_conf_14_text_scene() {
    let text = reference_scene("text").expect("a text reference scene");
    let (scene, resources) = text.record();
    let runs: Vec<_> = scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(runs.len(), 3);
    assert_eq!(scene.glyphs(&runs[0]).len(), 5);
    assert_eq!(scene.glyphs(&runs[1]).len(), 8);
    assert_eq!((runs[0].font_size, runs[1].font_size), (18.0, 10.0));
    for run in &runs {
        let font = resources
            .font(run.font)
            .expect("the font is in the resources");
        assert!(font.bytes().len() > 100_000);
    }
    assert!(
        scene
            .entries()
            .iter()
            .any(|e| matches!(e.command, Command::PushTransform(_)))
    );
}
