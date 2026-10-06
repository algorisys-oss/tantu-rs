//! Helpers shared by the wgpu renderer tests.

#![allow(dead_code)]

use tantu_core::{Rect, Size};
use tantu_render_wgpu::{CreateError, WgpuRenderer};
use tantu_scene::{ImageData, RenderReport, Renderer, Resources, Scene, SceneBuilder};

pub fn r(l: f32, t: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(l, t, w, h)
}

/// A frame of `w × h` logical pixels recorded by `paint`.
pub fn scene(w: f32, h: f32, paint: impl FnOnce(&mut SceneBuilder<'_>)) -> Scene {
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(w, h));
    paint(&mut b);
    b.finish().expect("tests record balanced scopes");
    scene
}

/// An offscreen renderer, or `None` (test skipped) when there is no GPU adapter, unless
/// `TANTU_REQUIRE_GPU=1` asks for a failure instead.
#[allow(clippy::print_stderr)] // A helper, not a #[test]: say why the test did nothing.
pub fn gpu(w: u32, h: u32) -> Option<WgpuRenderer> {
    match WgpuRenderer::new_offscreen(w, h) {
        Ok(renderer) => Some(renderer),
        Err(CreateError::NoAdapter)
            if std::env::var_os("TANTU_REQUIRE_GPU").is_none_or(|v| v != "1") =>
        {
            eprintln!("skipped: no GPU adapter (set TANTU_REQUIRE_GPU=1 to fail instead)");
            None
        }
        Err(e) => panic!("creating a wgpu renderer failed: {e}"),
    }
}

/// Renders `scene` into a fresh `w × h` renderer at `scale`; `None` when skipped.
pub fn render_with(
    w: u32,
    h: u32,
    scale: f32,
    scene: &Scene,
    resources: &Resources,
) -> Option<(ImageData, RenderReport)> {
    let mut renderer = gpu(w, h)?;
    renderer.resize(w, h, scale);
    let report = renderer.render(scene, resources).expect("offscreen target");
    Some((renderer.snapshot().expect("offscreen, non-empty"), report))
}

/// Renders a 100 × 100 frame at scale 1 with no resources; `None` when skipped.
pub fn render100(paint: impl FnOnce(&mut SceneBuilder<'_>)) -> Option<ImageData> {
    let s = scene(100.0, 100.0, paint);
    render_with(100, 100, 1.0, &s, &Resources::new()).map(|(image, _)| image)
}

/// The straight-alpha RGBA8 pixel at (x, y).
pub fn px(image: &ImageData, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * image.width() + x) * 4) as usize;
    let p = &image.pixels()[i..i + 4];
    [p[0], p[1], p[2], p[3]]
}

/// Asserts each channel is within `tol` of `expected`.
#[track_caller]
pub fn assert_px(image: &ImageData, x: u32, y: u32, expected: [u8; 4], tol: u8) {
    let actual = px(image, x, y);
    let close = actual
        .iter()
        .zip(expected)
        .all(|(a, e)| a.abs_diff(e) <= tol);
    assert!(
        close,
        "pixel ({x}, {y}) is {actual:?}, expected {expected:?} ±{tol}"
    );
}

pub const CLEAR: [u8; 4] = [0, 0, 0, 0];

/// True if every pixel is fully transparent.
pub fn is_blank(image: &ImageData) -> bool {
    image.pixels().iter().all(|&b| b == 0)
}
