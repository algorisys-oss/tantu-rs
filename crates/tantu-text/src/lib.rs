//! # Tantu: Text
//!
//! The text stack behind `tantu_layout::TextMeasure` (ADR 0005, ADR 0010): a [`TextSystem`]
//! owns fonts (registered files and, optionally, the system's), turns [`TextStyle`]s into
//! `TextStyleKey`s, measures paragraphs with parley (shaping, bidi, line breaking, fallback)
//! and paints them into a Scene as glyph runs. Spec: `docs/specs/text/system.md`.
//!
//! ```no_run
//! use tantu_layout::TextMeasure;
//! use tantu_text::{TextStyle, TextSystem};
//!
//! let mut text = TextSystem::new();
//! let body = text.style(TextStyle::default());
//! let metrics = text.measure("Hello, world", body, 200.0, None);
//! assert_eq!(metrics.line_count, 1);
//! ```

#![forbid(unsafe_code)]

mod system;

pub use system::{FontFamily, TextStyle, TextSystem};
