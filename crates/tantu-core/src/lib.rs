//! # Tantu: Core types
//!
//! Geometry, color, ids, arena and errors shared by every Tantu crate.
//!
//! So far this crate has the [`geometry`] types, re-exported at the crate root. Color, ids and
//! the arena follow (see `PLAN.md`, Phase 0).
//!
//! ```
//! use tantu_core::{EdgeInsets, Point, Rect, Size, Vec2};
//!
//! let bounds = Rect::from_origin_size(Point::new(10.0, 10.0), Size::new(100.0, 40.0));
//! let content = EdgeInsets::all(8.0).deflate_rect(bounds);
//! assert!(content.contains(Point::new(18.0, 18.0)));
//! assert_eq!(content.translate(Vec2::new(-10.0, -10.0)).origin(), Point::new(8.0, 8.0));
//! ```

pub mod geometry;

pub use geometry::{Affine, EdgeInsets, Point, Rect, Size, Vec2};
