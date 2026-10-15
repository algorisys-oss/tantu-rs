//! Shared fixtures: a runtime to create localizations in, and test resources.
#![allow(dead_code)]

use tantu_i18n::{Locale, Localization, Translations};
use tantu_view::{BuildCx, ElementId, View, ViewTree};

struct Nothing;

impl View for Nothing {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}

/// Runs `f` with a runtime current.
pub fn with<R>(f: impl FnOnce() -> R) -> R {
    let tree = ViewTree::new(|| Nothing);
    tree.enter(f)
}

pub fn locale(text: &str) -> Locale {
    Locale::parse(text).expect("a locale")
}

pub const EN: &str = "\
hello = Hello, { $name }!
inbox = { $count ->
    [one] You have one message
   *[other] You have { $count } messages
}
only-en = Only in English
";

pub const DE: &str = "\
hello = Hallo, { $name }!
inbox = { $count ->
    [one] Du hast eine Nachricht
   *[other] Du hast { $count } Nachrichten
}
";

pub const DE_CH: &str = "hello = Grüezi, { $name }!\n";

/// English (the fallback), German, Swiss German, and French and Hindi without messages.
pub fn translations() -> Translations {
    let mut t = Translations::new(locale("en"));
    t.add(locale("en"), EN).expect("valid");
    t.add(locale("de"), DE).expect("valid");
    t.add(locale("de-CH"), DE_CH).expect("valid");
    // Formats only: their messages fall back to English.
    t.add(locale("fr"), "").expect("valid");
    t.add(locale("hi"), "").expect("valid");
    t
}

/// A localization over [`translations`] in `locale`.
pub fn localization(locale_id: &str) -> Localization {
    Localization::new(translations(), locale(locale_id))
}
