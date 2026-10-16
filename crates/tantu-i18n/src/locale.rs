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
        let text = text.trim().replace('_', "-");
        if text.is_empty() {
            return None;
        }
        icu_locale_core::Locale::try_from_str(&text)
            .ok()
            .map(|id| Locale { id })
    }

    /// The language subtag (`"de"` for `de-CH`).
    pub fn language(&self) -> &str {
        self.id.id.language.as_str()
    }

    /// `Rtl` for right-to-left scripts (Arabic, Hebrew, Persian, Urdu, …), else `Ltr`.
    pub fn direction(&self) -> TextDirection {
        let script = self.id.id.script.map(|s| s.to_tinystr());
        let rtl = match script.as_ref().map(|s| s.as_str()) {
            Some(script) => RTL_SCRIPTS.contains(&script),
            None => RTL_LANGUAGES.contains(&self.language()),
        };
        if rtl {
            TextDirection::Rtl
        } else {
            TextDirection::Ltr
        }
    }

    /// The ICU4X identifier.
    pub(crate) fn icu(&self) -> &icu_locale_core::Locale {
        &self.id
    }

    /// The identifier as Fluent's `unic-langid` type (language, script, region, variants).
    pub(crate) fn langid(&self) -> unic_langid::LanguageIdentifier {
        self.id
            .id
            .to_string()
            .parse()
            .unwrap_or_else(|_| unic_langid::LanguageIdentifier::default())
    }

    /// This locale and its parents, most specific first (`de-CH`, `de`) (I18N-MSG-02).
    pub(crate) fn chain(&self) -> Vec<Locale> {
        let mut chain = vec![self.clone()];
        let id = &self.id.id;
        let mut parent =
            icu_locale_core::LanguageIdentifier::from((id.language, id.script, id.region));
        if parent != *id {
            chain.push(Locale::from_langid(parent.clone()));
        }
        if parent.region.is_some() {
            parent.region = None;
            chain.push(Locale::from_langid(parent.clone()));
        }
        if parent.script.is_some() {
            parent.script = None;
            chain.push(Locale::from_langid(parent));
        }
        chain.dedup();
        chain
    }

    fn from_langid(id: icu_locale_core::LanguageIdentifier) -> Locale {
        Locale {
            id: icu_locale_core::Locale::from(id),
        }
    }
}

/// Languages written right to left when no script subtag says otherwise (I18N-LOCALE-01).
const RTL_LANGUAGES: [&str; 13] = [
    "ar", "arc", "ckb", "dv", "fa", "he", "ks", "ps", "sd", "syr", "ug", "ur", "yi",
];

/// Right-to-left scripts.
const RTL_SCRIPTS: [&str; 5] = ["Arab", "Hebr", "Syrc", "Thaa", "Nkoo"];

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
