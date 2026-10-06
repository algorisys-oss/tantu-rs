//! # Tantu: wgpu renderer
//!
//! [`WgpuRenderer`] draws a Tantu [`Scene`](tantu_scene::Scene) on the GPU with
//! [wgpu](https://wgpu.rs) (Vulkan, Metal, DX12, GL), into a window surface
//! ([`WgpuRenderer::for_window`]) or an offscreen texture ([`WgpuRenderer::new_offscreen`]) that
//! [`WgpuRenderer::snapshot`] reads back. Its rules mirror the software renderer's, so both
//! backends are held to the same contract. The spec is `docs/specs/render-wgpu/renderer.md`.
//!
//! ```no_run
//! use tantu_core::{Color, Rect, Size};
//! use tantu_render_wgpu::WgpuRenderer;
//! use tantu_scene::{Renderer, Resources, Scene};
//!
//! let mut scene = Scene::new();
//! let mut b = scene.begin(Size::new(40.0, 20.0));
//! b.fill_rect(Rect::from_ltwh(0.0, 0.0, 20.0, 20.0), Color::from_rgb8(255, 0, 0));
//! b.finish().expect("balanced scopes");
//!
//! let mut renderer = WgpuRenderer::new_offscreen(80, 40)?; // needs a GPU adapter
//! renderer.resize(80, 40, 2.0);
//! renderer.render(&scene, &Resources::new()).expect("rendered");
//! let image = renderer.snapshot().expect("offscreen target");
//! assert_eq!(&image.pixels()[..4], [255, 0, 0, 255]);
//! # Ok::<(), tantu_render_wgpu::CreateError>(())
//! ```

#![forbid(unsafe_code)]

mod gpu;
mod renderer;

pub use renderer::{CreateError, CustomCanvas, CustomHandler, WgpuRenderer};
/// The wgpu version this crate draws with, for custom handlers.
pub use wgpu;
