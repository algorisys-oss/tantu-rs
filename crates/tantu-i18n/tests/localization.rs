//! Tests for `docs/specs/i18n/localization.md`, rules I18N-LOCALE-01 and I18N-MSG-01..06.

mod common;

use std::cell::RefCell;
use std::rc::Rc;

use common::*;
use tantu_i18n::{Args, Locale, Translations};
use tantu_view::layout::TextDirection;
use tantu_view::reactive::effect;

#[test]
fn i18n_locale_01_parse_and_direction() {
    assert_eq!(locale("EN_us").to_string(), "en-US");
    assert_eq!(locale("de-ch").to_string(), "de-CH");
    assert_eq!(Locale::parse("not a locale!"), None);
    assert_eq!(Locale::parse(""), None);
    assert_eq!("fr".parse::<Locale>(), Ok(locale("fr")));
    assert_eq!(locale("de-CH").language(), "de");
    for rtl in [
        "ar", "ar-EG", "he", "he-IL", "fa", "ur", "yi", "az-Arab", "ckb",
    ] {
        assert_eq!(locale(rtl).direction(), TextDirection::Rtl, "{rtl}");
    }
    for ltr in ["en", "de-CH", "hi", "az", "az-Latn", "zh-Hant"] {
        assert_eq!(locale(ltr).direction(), TextDirection::Ltr, "{ltr}");
    }
}

#[test]
fn i18n_msg_01_resources() {
    let mut t = Translations::new(locale("en"));
    // A syntax error: reported with its offset, the valid messages kept.
    let errors = t
        .add(locale("en"), "good = Good\nbad = { $\nalso = Also\n")
        .expect_err("a syntax error");
    assert!(!errors.is_empty());
    assert!(errors[0].offset >= "good = Good\n".len(), "{errors:?}");
    // Duplicate ids in one resource are reported too.
    let errors = t
        .add(locale("en"), "twice = One\ntwice = Two\n")
        .expect_err("a duplicate");
    assert!(!errors.is_empty());
    // A later resource replaces earlier messages.
    t.add(locale("en"), "good = Better\n").expect("valid");
    with(move || {
        let loc = tantu_i18n::Localization::new(t, locale("en"));
        assert_eq!(loc.text("good"), "Better");
        assert_eq!(loc.text("also"), "Also");
    });
}

#[test]
fn i18n_msg_02_fallback_chain() {
    with(|| {
        let loc = localization("de-CH");
        let ada = Args::new().set("name", "Ada");
        assert_eq!(loc.text_with("hello", &ada), "Grüezi, Ada!");
        assert_eq!(
            loc.text_with("inbox", &Args::new().set("count", 1)),
            "Du hast eine Nachricht"
        );
        assert_eq!(loc.text("only-en"), "Only in English");
        assert_eq!(loc.text("nowhere"), "nowhere");
        assert_eq!(loc.text("hello"), "Grüezi, {$name}!");
    });
}

#[test]
fn i18n_msg_03_arguments_and_plurals() {
    with(|| {
        let en = localization("en");
        let count = |n: i64| Args::new().set("count", n);
        assert_eq!(en.text_with("inbox", &count(1)), "You have one message");
        assert_eq!(en.text_with("inbox", &count(5)), "You have 5 messages");
        assert_eq!(
            en.text_with("inbox", &count(1234)),
            "You have 1,234 messages"
        );
        let de = localization("de");
        assert_eq!(
            de.text_with("inbox", &count(1234)),
            "Du hast 1.234 Nachrichten"
        );
        // No bidi isolation marks.
        let hello = en.text_with("hello", &Args::new().set("name", "Ada"));
        assert!(!hello.contains('\u{2068}') && !hello.contains('\u{2069}'));
    });
}

#[test]
fn i18n_msg_04_reactive_locale() {
    with(|| {
        let loc = localization("en");
        let seen: Rc<RefCell<Vec<String>>> = Rc::default();
        {
            let seen = seen.clone();
            effect(move || {
                seen.borrow_mut().push(
                    loc.text("only-en")
                        + "/"
                        + &loc.text_with("hello", &Args::new().set("name", "A")),
                )
            });
        }
        loc.set_locale(locale("de-AT"));
        assert_eq!(loc.locale(), locale("de"), "negotiated");
        loc.set_locale(locale("ja"));
        assert_eq!(loc.locale(), locale("en"), "the fallback");
        assert_eq!(
            *seen.borrow(),
            [
                "Only in English/Hello, A!",
                "Only in English/Hallo, A!",
                "Only in English/Hello, A!"
            ]
        );
    });
}

#[test]
fn i18n_msg_05_negotiation() {
    let t = translations();
    let mut locales: Vec<String> = t.locales().iter().map(ToString::to_string).collect();
    locales.sort();
    assert_eq!(locales, ["de", "de-CH", "en", "fr", "hi"]);
    assert_eq!(
        t.negotiate(&[locale("ja"), locale("de-AT"), locale("en")]),
        locale("de")
    );
    assert_eq!(t.negotiate(&[locale("de-CH")]), locale("de-CH"));
    assert_eq!(t.negotiate(&[locale("ja")]), locale("en"));
    assert_eq!(t.negotiate(&[]), locale("en"));
}

#[test]
fn i18n_msg_06_message() {
    with(|| {
        let loc = localization("de");
        assert_eq!(loc.message("nowhere", None), None);
        assert_eq!(
            loc.message("hello", Some(&Args::new().set("name", "Ada"))),
            Some("Hallo, Ada!".to_owned())
        );
        assert_eq!(
            loc.message("only-en", None),
            Some("Only in English".to_owned())
        );
    });
}
