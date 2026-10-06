//! # Tantu: Software renderer
//!
//! [`SoftRenderer`] draws a Tantu [`Scene`](tantu_scene::Scene) on the CPU with
//! [tiny-skia](https://github.com/linebender/tiny-skia), into an in-memory RGBA8 target that
//! [`SoftRenderer::snapshot`] reads back. It is the fallback where there is no usable GPU, and the
//! reference renderer for golden-image tests, with lossless [`encode_png`] / [`decode_png`] and
//! [`diff_images`]. The spec is `docs/specs/render-soft/renderer.md`.
//!
//! ```
//! use tantu_core::{Color, Rect, Size};
//! use tantu_render_soft::SoftRenderer;
//! use tantu_scene::{Renderer, Resources, Scene};
//!
//! let mut scene = Scene::new();
//! let mut b = scene.begin(Size::new(40.0, 20.0));
//! b.fill_rect(Rect::from_ltwh(0.0, 0.0, 20.0, 20.0), Color::from_rgb8(255, 0, 0));
//! b.finish().expect("balanced scopes");
//!
//! let mut renderer = SoftRenderer::new(80, 40);
//! renderer.resize(80, 40, 2.0);
//! let report = renderer.render(&scene, &Resources::new()).expect("in-memory target");
//! assert!(report.is_clean());
//!
//! let image = renderer.snapshot().expect("non-empty target");
//! let pixel = |x: usize, y: usize| &image.pixels()[(y * 80 + x) * 4..][..4];
//! assert_eq!(pixel(10, 10), [255, 0, 0, 255]); // inside the fill, scaled by 2
//! assert_eq!(pixel(60, 10), [0, 0, 0, 0]); // outside it
//! ```

#![forbid(unsafe_code)]

mod blur;
mod diff;
mod png_io;
mod renderer;

pub use diff::{ImageDiff, diff_images};
pub use png_io::{PngError, decode_png, encode_png};
pub use renderer::{CustomCanvas, CustomHandler, SoftRenderer};
/// The tiny-skia version this crate draws with, for custom handlers.
pub use tiny_skia;
