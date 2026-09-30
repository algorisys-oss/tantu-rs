//! # Tantu: Core types
//!
//! Geometry, color, ids, arena and errors shared by every Tantu crate.
//!
//! So far this crate has the [`geometry`] types and [`Color`], re-exported at the crate root. Ids
//! and the arena follow (see `PLAN.md`, Phase 0).
//!
//! ```
//! use tantu_core::{Color, EdgeInsets, Point, Rect, Size, Vec2};
//!
//! let bounds = Rect::from_origin_size(Point::new(10.0, 10.0), Size::new(100.0, 40.0));
//! let content = EdgeInsets::all(8.0).deflate_rect(bounds);
//! assert!(content.contains(Point::new(18.0, 18.0)));
//! assert_eq!(content.translate(Vec2::new(-10.0, -10.0)).origin(), Point::new(8.0, 8.0));
//!
//! let background = Color::from_argb32(0xFF2196F3);
//! assert!(background.is_opaque());
//! ```

pub mod color;
pub mod geometry;

pub use color::Color;
pub use geometry::{Affine, EdgeInsets, Point, Rect, Size, Vec2};
