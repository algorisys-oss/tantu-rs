//! # Tantu
//!
//! A cross-platform desktop UI framework with Flutter-style authoring and pluggable renderers.
//! *Compose once. Render your way.*
//!
//! This is the facade crate. It will provide the `App` runner and `tantu::prelude`. For now it is
//! a placeholder in the workspace skeleton; see `PLAN.md` for the roadmap.
//!
//! ## Features
//!
//! - `wgpu` (default): GPU renderer
//! - `winit` (default): winit platform shell
//! - `default-theme` (default): the default themes
//! - `soft`: CPU renderer built on tiny-skia

#![forbid(unsafe_code)]
