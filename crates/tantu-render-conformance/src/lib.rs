//! # Tantu: Renderer conformance
//!
//! What a renderer is tested against (ADR 0008): the reference Scenes, one per area of the
//! Scene command set, their goldens (rendered by `tantu-render-soft` and embedded in this
//! crate, so no file access is needed), and [`match_images`], a comparison that tolerates
//! anti-aliasing differences between backends but not real errors. A backend conforms when it
//! renders every [`reference_scenes`] entry so that [`ReferenceScene::check`] passes with
//! [`MatchTolerance::CROSS_BACKEND`]. The spec is `docs/specs/render-conformance/conformance.md`.
//!
//! This crate depends on no rasterizer, so using it doesn't pull one in.
//!
//! ```
//! use tantu_render_conformance::{MatchTolerance, reference_scenes};
//!
//! for reference in reference_scenes() {
//!     let (scene, resources) = reference.record();
//!     let (width, height) = reference.target_size();
//!     // A real backend renders `scene` with `resources` at `width × height` and
//!     // `reference.scale_factor()`; the golden itself stands in for its output here.
//!     let rendered = reference.golden().expect("embedded golden decodes");
//!     assert_eq!((rendered.width(), rendered.height()), (width, height));
//!     let result = reference.check(&rendered, &MatchTolerance::CROSS_BACKEND).expect("same size");
//!     assert!(result.passed, "{}: {result:?}", reference.name());
//!     let _ = (scene, resources);
//! }
//! ```

#![forbid(unsafe_code)]

mod compare;
mod reference;
mod scenes;

pub use compare::{ImageMatch, MatchTolerance, match_images};
pub use reference::{GoldenError, ReferenceScene, reference_scene, reference_scenes};
