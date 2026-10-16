//! # Tantu: i18n
//!
//! Locales, translated messages and locale-aware formatting. A [`Localization`] holds the
//! current [`Locale`] as a signal and the app's [`Translations`] (Fluent resources); widgets
//! find it as an inherited value (the `Localizations` widget provides it), so switching the
//! locale re-renders every message and formatted number or date that reads it. Messages use
//! [Project Fluent](https://projectfluent.org); numbers and dates use ICU4X. Specs:
//! `docs/specs/i18n/localization.md` and `docs/specs/i18n/formatting.md`,
//! [ADR 0015](https://github.com/algorisys-oss/tantu-rs/blob/main/docs/adr/0015-internationalization-crate.md).
//!
//! ```
//! use tantu_i18n::{Args, Locale, Localization, Translations};
//! use tantu_view::ViewTree;
//!
//! let en = Locale::parse("en").expect("a locale");
//! let mut translations = Translations::new(en.clone());
//! translations
//!     .add(en.clone(), "inbox = { $count ->\n  [one] One message\n *[other] { $count } messages\n}\n")
//!     .expect("valid Fluent");
//! let tree = ViewTree::new(|| tantu_view::Builder::new(|_| Nothing));
//! tree.enter(|| {
//!     let loc = Localization::new(translations, en);
//!     assert_eq!(loc.text_with("inbox", &Args::new().set("count", 3)), "3 messages");
//!     assert_eq!(loc.format_integer(1234567), "1,234,567");
//! });
//! # struct Nothing;
//! # impl tantu_view::View for Nothing {
//! #     fn build(self, cx: &mut tantu_view::BuildCx<'_>) -> tantu_view::ElementId {
//! #         cx.region(|_| {})
//! #     }
//! # }
//! ```
#![forbid(unsafe_code)]

mod date;
mod formats;
mod locale;
mod localization;
mod messages;
mod time;

pub use date::{Date, Weekday};
pub use locale::Locale;
pub use localization::{DateLength, Localization};
pub use messages::{ArgValue, Args, ResourceError, Translations};
pub use time::Time;
