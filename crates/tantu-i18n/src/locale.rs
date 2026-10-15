//! [`Locale`]: BCP 47 identifiers and their script direction. Spec:
//! `docs/specs/i18n/localization.md` (I18N-LOCALE-01).

use std::fmt;
use std::str::FromStr;

use tantu_view::layout::TextDirection;

/// A BCP 47 locale identifier such as `en`, `de-CH` or `ar-EG`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locale {
    id: icu_locale_core::Locale,
}

impl Locale {
    /// Parses an identifier (case-insensitive, `_` accepted as `-`); `None` if it isn't one.
    pub fn parse(text: &str) -> Option<Locale> {
        let _ = text;
        todo!("I18N-LOCALE-01")
    }

    /// The language subtag (`"de"` for `de-CH`).
    pub fn language(&self) -> &str {
        todo!("I18N-LOCALE-01")
    }

    /// `Rtl` for right-to-left scripts (Arabic, Hebrew, Persian, Urdu, …), else `Ltr`.
    pub fn direction(&self) -> TextDirection {
        todo!("I18N-LOCALE-01")
    }

    /// The ICU4X identifier.
    #[allow(dead_code)] // Used once formatting is implemented.
    pub(crate) fn icu(&self) -> &icu_locale_core::Locale {
        &self.id
    }
}

/// Writes the canonical form (`de-CH`).
impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.id, f)
    }
}

impl FromStr for Locale {
    type Err = ();

    /// As [`Locale::parse`].
    fn from_str(text: &str) -> Result<Locale, ()> {
        Locale::parse(text).ok_or(())
    }
}
