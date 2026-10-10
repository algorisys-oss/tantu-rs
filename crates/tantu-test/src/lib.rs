//! # Tantu: Testing
//!
//! [`WidgetTester`] builds a view in a window-sized tree with a bundled test font, pumps frames
//! into a headless recorder, simulates pointer input at found elements and compares frames with
//! golden PNGs through the software renderer. Spec: `docs/specs/test/widget-tester.md`.
//!
//! ```
//! use tantu_test::{Finder, WidgetTester};
//! use tantu_view::Keyed;
//! use tantu_view::reactive::signal;
//! use tantu_widgets::{Button, Column, Text};
//!
//! let mut tester = WidgetTester::new(|| {
//!     let count = signal(0);
//!     Column::new()
//!         .child(Text::new(move || format!("Count: {}", count.get())))
//!         .child(Keyed::new("inc", Button::new("+").on_press(move || count.update(|c| *c += 1))))
//! });
//! tester.tap(&Finder::key("inc"));
//! assert_eq!(tester.find_all(&Finder::text("Count: 1")).len(), 1);
//! ```

#![forbid(unsafe_code)]

mod tester;

pub use tester::{Finder, WidgetTester};
