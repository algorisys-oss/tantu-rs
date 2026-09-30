//! # Tantu: Core types
//!
//! Geometry, color, ids, arena and errors shared by every Tantu crate.
//!
//! It has the [`geometry`] types, [`Color`], and [`Id`] with the generational [`Arena`], all
//! re-exported at the crate root. No dependencies and no `unsafe`.
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

#![forbid(unsafe_code)]

pub mod arena;
pub mod color;
pub mod geometry;

pub use arena::{Arena, Id};
pub use color::Color;
pub use geometry::{Affine, EdgeInsets, Point, Rect, Size, Vec2};
