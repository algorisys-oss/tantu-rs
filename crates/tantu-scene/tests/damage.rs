//! Tests for `docs/specs/scene/damage.md`, rules SCENE-DAMAGE-01..04 and SCENE-RENDER-09.

use tantu_core::{Affine, Color, Point, Rect, Size, Vec2};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, CustomKind, Damage, ElementId, FontId, Glyph, ImageDraw,
    ImageId, ImageSampling, RenderReport, RoundedRect, Scene, SceneBuilder,
};

const W: f32 = 400.0;
const H: f32 = 300.0;

fn scene(f: impl FnOnce(&mut SceneBuilder<'_>)) -> Scene {
    sized(Size::new(W, H), f)
}

fn sized(size: Size, f: impl FnOnce(&mut SceneBuilder<'_>)) -> Scene {
    let mut scene = Scene::new();
    let mut b = scene.begin(size);
    f(&mut b);
    b.finish().expect("balanced");
    scene
}

fn id(n: u64) -> Option<ElementId> {
    ElementId::from_raw(n)
}

fn r(l: f32, t: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(l, t, w, h)
}

/// The damage of `next` against `previous`: `None` for full damage, else the rects.
fn damage(previous: &Scene, mut next: Scene) -> Option<Vec<Rect>> {
    next.diff_damage(previous);
    match next.damage() {
        Damage::Full => None,
        Damage::Rects(rects) => Some(rects.to_vec()),
    }
}

/// Three 50 × 50 boxes, elements 1, 2 and 3, at x 10, 110 and 210.
fn three(f: impl Fn(u64, &mut SceneBuilder<'_>)) -> Scene {
    scene(|b| {
        for n in 1..=3 {
            b.set_element(id(n));
            f(n, b);
        }
    })
}

fn box_at(n: u64, b: &mut SceneBuilder<'_>, color: Color) {
    b.fill_rect(r(10.0 + 100.0 * (n - 1) as f32, 10.0, 50.0, 50.0), color);
}

#[test]
fn scene_damage_01_identical_and_resized() {
    let a = three(|n, b| box_at(n, b, Color::BLACK));
    let same = three(|n, b| box_at(n, b, Color::BLACK));
    assert_eq!(damage(&a, same), Some(vec![]));
    let bigger = sized(Size::new(W + 1.0, H), |b| {
        b.set_element(id(1));
        box_at(1, b, Color::BLACK);
    });
    assert_eq!(damage(&a, bigger), None);
}

#[test]
fn scene_damage_02_groups_by_element() {
    let a = three(|n, b| box_at(n, b, Color::BLACK));
    // One element's color: its box only.
    let recolored = three(|n, b| box_at(n, b, if n == 2 { Color::WHITE } else { Color::BLACK }));
    assert_eq!(
        damage(&a, recolored),
        Some(vec![r(110.0, 10.0, 50.0, 50.0)])
    );
    // A new element in front of the others: only its box, though every command moved.
    let inserted = scene(|b| {
        b.set_element(id(9));
        b.fill_rect(r(10.0, 100.0, 20.0, 20.0), Color::BLACK);
        for n in 1..=3 {
            b.set_element(id(n));
            box_at(n, b, Color::BLACK);
        }
    });
    assert_eq!(damage(&a, inserted), Some(vec![r(10.0, 100.0, 20.0, 20.0)]));
    // A changed context (a transform around element 3): where it was and where it is.
    let moved = three(|n, b| {
        if n == 3 {
            b.push_transform(Affine::translate(Vec2::new(0.0, 100.0)));
            box_at(n, b, Color::BLACK);
            b.pop();
        } else {
            box_at(n, b, Color::BLACK);
        }
    });
    assert_eq!(damage(&a, moved), Some(vec![r(210.0, 10.0, 50.0, 150.0)]));
    // Commands without an element form one group.
    let plain = |color| {
        scene(|b| {
            b.fill_rect(r(0.0, 200.0, 30.0, 30.0), color);
        })
    };
    assert_eq!(
        damage(&plain(Color::BLACK), plain(Color::WHITE)),
        Some(vec![r(0.0, 200.0, 30.0, 30.0)])
    );
    // Swapping two overlapping elements repaints both.
    let pair = |first: u64| {
        scene(|b| {
            for n in [first, 3 - first] {
                b.set_element(id(n));
                b.fill_rect(r(10.0 * n as f32, 0.0, 40.0, 40.0), Color::BLACK);
            }
        })
    };
    let swapped = damage(&pair(1), pair(2)).expect("rects");
    let covered = swapped.iter().fold(Rect::ZERO, |acc, rect| {
        if acc == Rect::ZERO {
            *rect
        } else {
            acc.union(*rect)
        }
    });
    assert_eq!(covered, r(10.0, 0.0, 50.0, 40.0));
}

#[test]
fn scene_damage_03_bounds() {
    let font = FontId::from_raw(1).expect("1 is not 0");
    let image = ImageId::from_raw(1).expect("1 is not 0");
    let glyphs = [
        Glyph {
            id: 1,
            x: 0.0,
            y: 0.0,
        },
        Glyph {
            id: 2,
            x: 30.0,
            y: 0.0,
        },
    ];
    // Each case: what one element draws, and the damage when it appears.
    let cases: Vec<(&str, Box<dyn Fn(&mut SceneBuilder<'_>)>, Rect)> = vec![
        (
            "translated fill",
            Box::new(|b| {
                b.push_transform(Affine::translate(Vec2::new(5.0, 7.0)));
                b.fill_rect(r(0.0, 0.0, 10.0, 10.0), Color::BLACK);
                b.pop();
            }),
            r(5.0, 7.0, 10.0, 10.0),
        ),
        (
            "stroke",
            Box::new(|b| {
                b.stroke(
                    RoundedRect::new(r(20.0, 20.0, 10.0, 10.0), BorderRadius::ZERO),
                    4.0,
                    Color::BLACK,
                )
            }),
            r(18.0, 18.0, 14.0, 14.0),
        ),
        (
            "shadow",
            Box::new(|b| {
                b.box_shadow(BoxShadow {
                    shape: RoundedRect::new(r(50.0, 50.0, 20.0, 20.0), BorderRadius::ZERO),
                    color: Color::BLACK,
                    offset: Vec2::new(4.0, 0.0),
                    blur_radius: 0.0,
                    spread_radius: 2.0,
                })
            }),
            // Spread 2, sigma 0.5 with no blur (3 sigma = 1.5): inflate 3.5, moved by 4.
            r(50.5, 46.5, 27.0, 27.0),
        ),
        (
            "image",
            Box::new(move |b| {
                b.image(ImageDraw {
                    image,
                    src: None,
                    dest: r(100.0, 100.0, 32.0, 16.0),
                    sampling: ImageSampling::Linear,
                    opacity: 1.0,
                })
            }),
            r(100.0, 100.0, 32.0, 16.0),
        ),
        (
            "glyph run",
            Box::new(move |b| {
                b.glyph_run(font, 10.0, Color::BLACK, Point::new(100.0, 200.0), &glyphs)
            }),
            // Glyph positions (100..130, 200) widened by the 10 px font size.
            r(90.0, 190.0, 50.0, 20.0),
        ),
        (
            "custom",
            Box::new(|b| b.custom(CustomKind(1), r(300.0, 10.0, 20.0, 20.0), &[1, 2])),
            r(300.0, 10.0, 20.0, 20.0),
        ),
        (
            "clipped",
            Box::new(|b| {
                b.push_clip(Clip::Rect(r(0.0, 0.0, 15.0, 15.0)));
                b.fill_rect(r(10.0, 10.0, 20.0, 20.0), Color::BLACK);
                b.pop();
            }),
            r(10.0, 10.0, 5.0, 5.0),
        ),
        (
            "off the frame",
            Box::new(|b| b.fill_rect(r(390.0, 290.0, 20.0, 20.0), Color::BLACK)),
            r(390.0, 290.0, 10.0, 10.0),
        ),
    ];
    let empty = scene(|_| {});
    for (name, draw, expected) in cases {
        let next = scene(|b| {
            b.set_element(id(1));
            draw(b);
        });
        assert_eq!(damage(&empty, next), Some(vec![expected]), "{name}");
    }
}

#[test]
fn scene_damage_04_coalescing() {
    let empty = scene(|_| {});
    // Overlapping and touching rects merge.
    let touching = scene(|b| {
        b.set_element(id(1));
        b.fill_rect(r(0.0, 0.0, 20.0, 20.0), Color::BLACK);
        b.set_element(id(2));
        b.fill_rect(r(10.0, 10.0, 20.0, 20.0), Color::BLACK);
        b.set_element(id(3));
        b.fill_rect(r(30.0, 0.0, 10.0, 10.0), Color::BLACK);
    });
    assert_eq!(
        damage(&empty, touching),
        Some(vec![r(0.0, 0.0, 40.0, 30.0)])
    );
    // More than 16 separate rects: full.
    let many = |count: u64| {
        scene(|b| {
            for n in 0..count {
                b.set_element(id(n + 1));
                b.fill_rect(
                    r((n % 8) as f32 * 40.0, (n / 8) as f32 * 40.0, 10.0, 10.0),
                    Color::BLACK,
                );
            }
        })
    };
    assert_eq!(damage(&empty, many(16)).map(|r| r.len()), Some(16));
    assert_eq!(damage(&empty, many(17)), None);
    // More than 60 % of the frame: full.
    let big = scene(|b| {
        b.set_element(id(1));
        b.fill_rect(r(0.0, 0.0, W, H * 0.61), Color::BLACK);
    });
    assert_eq!(damage(&empty, big), None);
}

#[test]
fn scene_damage_05_replaces_builder_damage() {
    let a = three(|n, b| box_at(n, b, Color::BLACK));
    let mut next = three(|n, b| {
        box_at(n, b, Color::BLACK);
        b.add_damage(r(0.0, 0.0, 1.0, 1.0));
    });
    let entries = next.entries().to_vec();
    next.diff_damage(&a);
    assert_eq!(next.damage(), Damage::Rects(&[]));
    assert_eq!(next.entries(), &entries[..]);
}

#[test]
fn scene_render_09_repainted_pixels_is_a_measurement() {
    let report = RenderReport::default();
    assert_eq!(report.repainted_pixels, 0);
    let measured = RenderReport {
        repainted_pixels: 1234,
        ..report
    };
    assert!(measured.is_clean(), "a measurement, not a problem");
    let scene = scene(|b| b.fill_rect(r(0.0, 0.0, 10.0, 10.0), Color::BLACK));
    let counted = RenderReport::for_scene(&scene, &tantu_scene::Resources::new(), &|_| false);
    assert_eq!(counted.repainted_pixels, 0, "for_scene doesn't measure");
}
