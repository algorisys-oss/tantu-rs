//! # Tantu: Scene
//!
//! The backend-neutral [`Scene`] display list. Paint code records one frame into a `Scene`
//! through a [`SceneBuilder`]; renderers read [`Scene::entries`] in order and draw. A Scene is
//! plain data: no callbacks into the UI, images and fonts referenced by handle, glyphs and
//! custom-command bytes copied into the Scene's own buffers.
//!
//! Every [`Entry`] carries the element it was painted for and a z-index. Clips, transforms and
//! layers are scopes (`push_*` … `pop`); [`SceneBuilder::finish`] orders entries by z-index
//! within each scope. The spec is `docs/specs/scene/scene.md`.
//!
//! Backends implement [`Renderer`] and look the Scene's image and font handles up in
//! [`Resources`] (spec `docs/specs/scene/renderer.md`).
//!
//! ```
//! use tantu_core::{Color, Rect, Size};
//! use tantu_scene::{Clip, Command, Scene};
//!
//! let mut scene = Scene::new();
//! let mut builder = scene.begin(Size::new(200.0, 100.0));
//! builder.push_clip(Clip::Rect(Rect::from_ltwh(0.0, 0.0, 100.0, 100.0)));
//! builder.fill_rect(Rect::from_ltwh(10.0, 10.0, 50.0, 20.0), Color::BLACK);
//! assert!(builder.is_culled(Rect::from_ltwh(150.0, 0.0, 10.0, 10.0)));
//! builder.pop();
//! builder.finish().expect("scopes are balanced");
//!
//! assert_eq!(scene.entries().len(), 3);
//! assert!(matches!(scene.entries()[1].command, Command::Fill { .. }));
//! ```

#![forbid(unsafe_code)]

pub mod command;
pub mod handles;
pub mod renderer;
pub mod resources;
pub mod scene;

pub use command::{
    BorderRadius, BoxShadow, Clip, Command, CustomDraw, Entry, Glyph, GlyphRun, ImageDraw,
    ImageSampling, Layer, RoundedRect,
};
pub use handles::{CustomKind, ElementId, FontId, ImageId};
pub use renderer::{RenderError, RenderReport, Renderer};
pub use resources::{FontData, ImageData, ResourceError, Resources};
pub use scene::{Damage, Scene, SceneBuilder, SceneError};
