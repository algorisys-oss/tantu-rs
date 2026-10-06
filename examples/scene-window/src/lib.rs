//! # scene-window: the Phase 1 demo
//!
//! A hand-built, animated Scene with every Phase 1 command, drawn with wgpu in a winit window:
//! run it with `cargo run -p scene-window`. The frame itself is built by [`demo_scene`], which
//! the tests also render with the software renderer. The spec is
//! `docs/specs/examples/scene-window.md`.
//!
//! This depends on the Tantu crates directly; in Phase 2 it moves to the `tantu` facade.

#![forbid(unsafe_code)]

use std::f32::consts::TAU;

use tantu_core::{Affine, Color, Rect, Size, Vec2};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, ImageData, ImageDraw, ImageId, ImageSampling, Layer,
    RoundedRect, Scene,
};

/// The size the demo is designed at; it is scaled to fit the window.
const DESIGN: Size = Size::new(900.0, 600.0);

const BACKGROUND: Color = Color::from_rgb8(236, 239, 244);
const CARD: Color = Color::from_rgb8(255, 255, 255);
const INK: Color = Color::from_rgb8(46, 52, 64);
const ACCENT: Color = Color::from_rgb8(94, 129, 172);
const WARM: Color = Color::from_rgb8(208, 135, 112);
const GREEN: Color = Color::from_rgb8(163, 190, 140);
const PURPLE: Color = Color::from_rgb8(180, 142, 173);

/// The 64 × 64 RGBA8 image the demo draws (a hue gradient with a checker pattern).
pub fn demo_image() -> ImageData {
    let mut pixels = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64u32 {
        for x in 0..64u32 {
            let hue = x as f32 / 64.0;
            let value = 1.0 - y as f32 / 128.0;
            let checker = (x / 8 + y / 8) % 2 == 0;
            let [r, g, b] = hsv(hue, 0.65, if checker { value } else { value * 0.85 });
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }
    ImageData::rgba8(64, 64, pixels).expect("64 × 64 × 4 bytes")
}

/// HSV (each 0..=1) to 8-bit RGB.
fn hsv(h: f32, s: f32, v: f32) -> [u8; 3] {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - f * s), v * (1.0 - (1.0 - f) * s));
    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [r, g, b].map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn card(b: &mut tantu_scene::SceneBuilder<'_>, rect: Rect) {
    let shape = RoundedRect::new(rect, BorderRadius::circular(16.0));
    b.box_shadow(BoxShadow {
        shape,
        color: Color::BLACK.with_alpha(0.18),
        offset: Vec2::new(0.0, 6.0),
        blur_radius: 18.0,
        spread_radius: 0.0,
    });
    b.fill(shape, CARD);
    b.stroke(shape, 1.0, Color::BLACK.with_alpha(0.08));
}

/// Records the demo frame into `scene` for a window of `size` logical pixels at animation time
/// `time` (seconds), drawing `image` (registered from [`demo_image`]).
pub fn demo_scene(scene: &mut Scene, size: Size, time: f32, image: ImageId) {
    let mut b = scene.begin(size);
    b.fill_rect(
        Rect::from_ltwh(0.0, 0.0, size.width, size.height),
        BACKGROUND,
    );

    // Fit the 900 × 600 design into the window, centered.
    let scale = (size.width / DESIGN.width).min(size.height / DESIGN.height);
    let offset = Vec2::new(
        (size.width - DESIGN.width * scale) / 2.0,
        (size.height - DESIGN.height * scale) / 2.0,
    );
    b.push_transform(Affine::translate(offset) * Affine::scale(scale));
    // Keep the animation phase small so it stays precise for any time.
    let t = time.rem_euclid(60.0);

    // Card 1: shapes and strokes.
    card(&mut b, Rect::from_ltwh(40.0, 40.0, 390.0, 250.0));
    for (i, color) in [ACCENT, WARM, GREEN, PURPLE].into_iter().enumerate() {
        let x = 70.0 + i as f32 * 85.0;
        let radius = 4.0 + i as f32 * 10.0;
        b.fill(
            RoundedRect::new(
                Rect::from_ltwh(x, 80.0, 70.0, 70.0),
                BorderRadius::circular(radius),
            ),
            color,
        );
        b.stroke(
            RoundedRect::new(
                Rect::from_ltwh(x, 170.0, 70.0, 70.0),
                BorderRadius::circular(radius),
            ),
            2.0 + i as f32 * 2.0,
            color,
        );
    }
    b.fill(
        RoundedRect::new(
            Rect::from_ltwh(70.0, 255.0, 330.0, 14.0),
            BorderRadius::circular(7.0),
        ),
        INK.with_alpha(0.12),
    );
    let progress = 0.5 + 0.5 * (t * 1.3).sin();
    b.fill(
        RoundedRect::new(
            Rect::from_ltwh(70.0, 255.0, 330.0 * progress.max(0.05), 14.0),
            BorderRadius::circular(7.0),
        ),
        ACCENT,
    );

    // Card 2: clips and transforms; a spinning fan of bars inside a rounded clip.
    card(&mut b, Rect::from_ltwh(470.0, 40.0, 390.0, 250.0));
    b.push_clip(Clip::RoundedRect(RoundedRect::new(
        Rect::from_ltwh(490.0, 60.0, 350.0, 210.0),
        BorderRadius::circular(12.0),
    )));
    b.fill_rect(
        Rect::from_ltwh(490.0, 60.0, 350.0, 210.0),
        ACCENT.with_alpha(0.12),
    );
    b.push_transform(Affine::translate(Vec2::new(665.0, 165.0)) * Affine::rotate(t * 0.8));
    for i in 0..12 {
        b.push_transform(Affine::rotate(i as f32 * TAU / 12.0));
        let color = if i % 2 == 0 { ACCENT } else { WARM };
        b.fill(
            RoundedRect::new(
                Rect::from_ltwh(20.0, -6.0, 160.0, 12.0),
                BorderRadius::circular(6.0),
            ),
            color.with_alpha(0.85),
        );
        b.pop();
    }
    b.fill(
        RoundedRect::new(
            Rect::from_ltwh(-18.0, -18.0, 36.0, 36.0),
            BorderRadius::circular(18.0),
        ),
        INK,
    );
    b.pop();
    b.pop();

    // Card 3: images, nearest and bilinear, one gently rotating.
    card(&mut b, Rect::from_ltwh(40.0, 320.0, 390.0, 240.0));
    b.image(ImageDraw {
        image,
        src: None,
        dest: Rect::from_ltwh(70.0, 350.0, 96.0, 96.0),
        sampling: ImageSampling::Nearest,
        opacity: 1.0,
    });
    b.image(ImageDraw {
        image,
        src: None,
        dest: Rect::from_ltwh(70.0, 456.0, 96.0, 80.0),
        sampling: ImageSampling::Linear,
        opacity: 1.0,
    });
    b.image(ImageDraw {
        image,
        src: Some(Rect::from_ltwh(16.0, 16.0, 32.0, 32.0)),
        dest: Rect::from_ltwh(186.0, 350.0, 96.0, 96.0),
        sampling: ImageSampling::Nearest,
        opacity: 0.6,
    });
    b.push_transform(
        Affine::translate(Vec2::new(340.0, 440.0)) * Affine::rotate((t * 0.6).sin() * 0.4),
    );
    b.box_shadow(BoxShadow {
        shape: RoundedRect::new(
            Rect::from_ltwh(-60.0, -60.0, 120.0, 120.0),
            BorderRadius::circular(10.0),
        ),
        color: Color::BLACK.with_alpha(0.25),
        offset: Vec2::new(0.0, 8.0),
        blur_radius: 14.0,
        spread_radius: 0.0,
    });
    b.push_clip(Clip::RoundedRect(RoundedRect::new(
        Rect::from_ltwh(-60.0, -60.0, 120.0, 120.0),
        BorderRadius::circular(10.0),
    )));
    b.image(ImageDraw {
        image,
        src: None,
        dest: Rect::from_ltwh(-60.0, -60.0, 120.0, 120.0),
        sampling: ImageSampling::Linear,
        opacity: 1.0,
    });
    b.pop();
    b.pop();

    // Card 4: layers; group opacity and an overlay color pulsing in.
    card(&mut b, Rect::from_ltwh(470.0, 320.0, 390.0, 240.0));
    b.push_layer(Layer {
        opacity: 0.75,
        overlay_color: None,
    });
    for (i, color) in [ACCENT, WARM, GREEN].into_iter().enumerate() {
        let x = 510.0 + i as f32 * 70.0;
        b.fill(
            RoundedRect::new(
                Rect::from_ltwh(x, 350.0, 120.0, 120.0),
                BorderRadius::circular(60.0),
            ),
            color,
        );
    }
    b.pop();
    let tint = 0.5 + 0.5 * (t * 2.0).sin();
    b.push_layer(Layer {
        opacity: 1.0,
        overlay_color: Some(PURPLE.with_alpha(tint * 0.8)),
    });
    for i in 0..4 {
        let x = 500.0 + i as f32 * 85.0;
        b.fill(
            RoundedRect::new(
                Rect::from_ltwh(x, 490.0, 70.0, 40.0),
                BorderRadius::circular(20.0),
            ),
            GREEN,
        );
    }
    b.pop();

    b.pop();
    b.finish().expect("the demo records balanced scopes");
}
