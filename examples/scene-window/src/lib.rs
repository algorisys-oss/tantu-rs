//! # scene-window: the Phase 1 demo
//!
//! A hand-built, animated Scene with every Phase 1 command, drawn with wgpu in a winit window:
//! run it with `cargo run -p scene-window`. The frame itself is built by [`demo_scene`], which
//! the tests also render with the software renderer. The spec is
//! `docs/specs/examples/scene-window.md`.
//!
//! This depends on the Tantu crates directly; in Phase 2 it moves to the `tantu` facade.

#![forbid(unsafe_code)]

use tantu_core::Size;
use tantu_scene::{ImageData, ImageId, Scene};

/// The 64 × 64 RGBA8 image the demo draws (a hue gradient with a checker pattern).
pub fn demo_image() -> ImageData {
    todo!()
}

/// Records the demo frame into `scene` for a window of `size` logical pixels at animation time
/// `time` (seconds), drawing `image` (registered from [`demo_image`]).
pub fn demo_scene(scene: &mut Scene, size: Size, time: f32, image: ImageId) {
    todo!()
}
