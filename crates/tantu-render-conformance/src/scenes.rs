//! The reference Scenes. Each draws a 100 × 100 logical frame; they are rendered at scale 2
//! into a 200 × 200 target. Changing one means regenerating its golden with
//! `TANTU_UPDATE_GOLDENS=1 cargo test -p tantu-render-soft --test goldens`.

use tantu_core::{Affine, Color, Point, Rect, Size, Vec2};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, FontData, Glyph, ImageData, ImageDraw, ImageSampling, Layer,
    Resources, RoundedRect, Scene, SceneBuilder,
};

use crate::reference::ReferenceScene;

/// Logical size of every reference frame.
pub(crate) const LOGICAL_SIZE: f32 = 100.0;
/// Scale factor every reference frame is rendered at.
pub(crate) const SCALE_FACTOR: f32 = 2.0;

macro_rules! reference {
    ($name:ident) => {
        ReferenceScene {
            name: stringify!($name),
            golden_png: include_bytes!(concat!("../goldens/", stringify!($name), ".png")),
            record: $name,
        }
    };
}

/// The reference Scenes, in the order of the spec (RENDER-CONF-08).
pub(crate) static REFERENCE_SCENES: [ReferenceScene; 6] = [
    reference!(shapes_and_strokes),
    reference!(shadows),
    reference!(images),
    reference!(clips_and_transforms),
    reference!(layers),
    reference!(text),
];

fn r(l: f32, t: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(l, t, w, h)
}

/// A 100 × 100 frame recorded by `paint`.
fn golden_scene(paint: impl FnOnce(&mut SceneBuilder<'_>)) -> Scene {
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(LOGICAL_SIZE, LOGICAL_SIZE));
    paint(&mut b);
    b.finish()
        .expect("reference scenes record balanced scopes (RENDER-CONF-09)");
    scene
}

const RED: Color = Color::from_rgb8(220, 40, 40);
const BLUE: Color = Color::from_rgb8(40, 80, 220);
const GREEN: Color = Color::from_rgb8(40, 180, 90);

fn shapes_and_strokes(_resources: &mut Resources) -> Scene {
    golden_scene(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::WHITE);
        b.fill(
            RoundedRect::new(r(8.0, 8.0, 40.0, 30.0), BorderRadius::circular(8.0)),
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
            BLUE,
        );
        b.stroke(
            RoundedRect::new(r(8.0, 50.0, 40.0, 40.0), BorderRadius::circular(12.0)),
            3.0,
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

fn shadows(_resources: &mut Resources) -> Scene {
    golden_scene(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::from_rgb8(240, 240, 240));
        for (i, blur) in [0.0f32, 3.0, 10.0].into_iter().enumerate() {
            let shape = RoundedRect::new(
                r(10.0 + i as f32 * 30.0, 15.0, 22.0, 22.0),
                BorderRadius::circular(4.0),
            );
            b.box_shadow(BoxShadow {
                shape,
                color: Color::BLACK.with_alpha(0.5),
                offset: Vec2::new(2.0, 4.0),
                blur_radius: blur,
                spread_radius: 1.0,
            });
            b.fill(shape, Color::WHITE);
        }
        b.push_transform(Affine::translate(Vec2::new(50.0, 70.0)) * Affine::rotate(0.4));
        let shape = RoundedRect::new(r(-20.0, -10.0, 40.0, 20.0), BorderRadius::circular(6.0));
        b.box_shadow(BoxShadow {
            shape,
            color: BLUE.with_alpha(0.6),
            offset: Vec2::ZERO,
            blur_radius: 6.0,
            spread_radius: 0.0,
        });
        b.fill(shape, Color::WHITE);
        b.pop();
    })
}

fn images(resources: &mut Resources) -> Scene {
    let pixels: Vec<u8> = (0..16u8)
        .flat_map(|i| [i * 16, 255 - i * 16, (i % 4) * 80, 255 - (i / 4) * 40])
        .collect();
    let id = resources.add_image(ImageData::rgba8(4, 4, pixels).expect("4×4 RGBA8"));
    golden_scene(|b| {
        b.image(ImageDraw {
            image: id,
            src: None,
            dest: r(5.0, 5.0, 40.0, 40.0),
            sampling: ImageSampling::Nearest,
            opacity: 1.0,
        });
        b.image(ImageDraw {
            image: id,
            src: None,
            dest: r(55.0, 5.0, 40.0, 40.0),
            sampling: ImageSampling::Linear,
            opacity: 1.0,
        });
        b.image(ImageDraw {
            image: id,
            src: Some(r(1.0, 1.0, 2.0, 2.0)),
            dest: r(5.0, 55.0, 40.0, 40.0),
            sampling: ImageSampling::Nearest,
            opacity: 0.6,
        });
        b.push_transform(Affine::translate(Vec2::new(75.0, 75.0)) * Affine::rotate(0.5));
        b.image(ImageDraw {
            image: id,
            src: None,
            dest: r(-15.0, -15.0, 30.0, 30.0),
            sampling: ImageSampling::Linear,
            opacity: 1.0,
        });
        b.pop();
    })
}

fn clips_and_transforms(_resources: &mut Resources) -> Scene {
    golden_scene(|b| {
        b.push_clip(Clip::RoundedRect(RoundedRect::new(
            r(5.0, 5.0, 90.0, 90.0),
            BorderRadius::circular(20.0),
        )));
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::from_rgb8(250, 230, 200));
        b.push_transform(Affine::translate(Vec2::new(50.0, 50.0)));
        for i in 0..6 {
            b.push_transform(Affine::rotate(i as f32 * std::f32::consts::FRAC_PI_6));
            b.push_clip(Clip::Rect(r(0.0, -4.0, 60.0, 8.0)));
            b.fill_rect(
                r(5.0, -10.0, 50.0, 20.0),
                if i % 2 == 0 { RED } else { BLUE },
            );
            b.pop();
            b.pop();
        }
        b.push_transform(Affine::scale_non_uniform(2.0, 0.5));
        b.fill(
            RoundedRect::new(r(-8.0, -8.0, 16.0, 16.0), BorderRadius::circular(8.0)),
            GREEN,
        );
        b.pop();
        b.pop();
        b.pop();
    })
}

fn layers(_resources: &mut Resources) -> Scene {
    golden_scene(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::WHITE);
        b.push_layer(Layer {
            opacity: 0.5,
            overlay_color: None,
        });
        b.fill(
            RoundedRect::new(r(10.0, 10.0, 50.0, 50.0), BorderRadius::circular(10.0)),
            RED,
        );
        b.fill(
            RoundedRect::new(r(35.0, 35.0, 50.0, 50.0), BorderRadius::circular(25.0)),
            BLUE,
        );
        b.pop();
        b.push_layer(Layer {
            opacity: 1.0,
            overlay_color: Some(Color::from_rgb8(255, 200, 0).with_alpha(0.6)),
        });
        b.fill(
            RoundedRect::new(r(60.0, 5.0, 35.0, 25.0), BorderRadius::circular(5.0)),
            GREEN,
        );
        b.push_layer(Layer {
            opacity: 0.5,
            overlay_color: None,
        });
        b.fill_rect(r(70.0, 15.0, 25.0, 25.0), BLUE);
        b.pop();
        b.pop();
    })
}

/// Liberation Sans Regular (SIL Open Font License 1.1, see `fonts/LICENSE-OFL.txt`).
const LIBERATION_SANS: &[u8] = include_bytes!("../fonts/LiberationSans-Regular.ttf");

/// "Tantu" at 18 px: (glyph id, x, y) from `tantu_text::TextSystem`, frozen here so the Scene
/// doesn't depend on shaping (RENDER-CONF-14).
const TANTU: [(u32, f32, f32); 5] = [
    (55, 0.0, 16.0),
    (68, 9.0, 16.0),
    (81, 19.0107, 16.0),
    (87, 29.0215, 16.0),
    (88, 34.0225, 16.0),
];

/// "Ag 0.5px" at 10 px.
const SMALL: [(u32, f32, f32); 8] = [
    (36, 0.0, 9.0),
    (74, 6.6699, 9.0),
    (3, 12.2314, 9.0),
    (19, 15.0098, 9.0),
    (17, 20.5713, 9.0),
    (24, 23.3496, 9.0),
    (83, 28.9111, 9.0),
    (91, 34.4727, 9.0),
];

fn glyphs(data: &[(u32, f32, f32)]) -> Vec<Glyph> {
    data.iter().map(|&(id, x, y)| Glyph { id, x, y }).collect()
}

fn text(resources: &mut Resources) -> Scene {
    let font = resources
        .add_font(FontData::new(LIBERATION_SANS, 0).expect("the embedded font is not empty"));
    let (tantu, small) = (glyphs(&TANTU), glyphs(&SMALL));
    golden_scene(|b| {
        b.fill_rect(r(0.0, 0.0, 100.0, 100.0), Color::WHITE);
        b.glyph_run(font, 18.0, Color::BLACK, Point::new(6.0, 8.0), &tantu);
        b.glyph_run(font, 10.0, BLUE, Point::new(6.0, 40.0), &small);
        b.push_transform(Affine::translate(Vec2::new(6.0, 60.0)) * Affine::scale(1.5));
        b.glyph_run(font, 10.0, RED, Point::ZERO, &small);
        b.pop();
    })
}
