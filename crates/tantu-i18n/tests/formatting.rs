//! Tests for `docs/specs/i18n/formatting.md`, rules I18N-FMT-01..06.

mod common;

use std::cell::RefCell;
use std::rc::Rc;

use common::*;
use tantu_i18n::{Date, DateLength, Time, Weekday};
use tantu_view::reactive::effect;

const NNBSP: char = '\u{202f}';

fn date() -> Date {
    Date::new(2026, 10, 16).expect("exists")
}

fn time() -> Time {
    Time::new(21, 5).expect("exists")
}

#[test]
fn i18n_fmt_01_numbers() {
    with(|| {
        let n = |l: &str| localization(l).format_integer(1_234_567);
        assert_eq!(n("en"), "1,234,567");
        assert_eq!(n("de"), "1.234.567");
        assert_eq!(n("fr"), format!("1{NNBSP}234{NNBSP}567"));
        assert_eq!(n("hi"), "12,34,567");
        let en = localization("en");
        assert_eq!(en.format_integer(-42), "-42");
        assert_eq!(en.format_decimal(2.5, 2), "2.50");
        assert_eq!(localization("de").format_decimal(2.5, 2), "2,50");
        assert_eq!(en.format_decimal(0.125, 2), "0.12", "half to even");
        assert_eq!(en.format_decimal(1234.5678, 0), "1,235");
        assert_eq!(en.format_decimal(f64::NAN, 2), "NaN");
        assert_eq!(en.format_decimal(f64::INFINITY, 2), "∞");
        assert_eq!(en.format_decimal(f64::NEG_INFINITY, 2), "-∞");
    });
}

#[test]
fn i18n_fmt_02_dates() {
    with(|| {
        let all = |l: &str| {
            let loc = localization(l);
            [
                DateLength::Short,
                DateLength::Medium,
                DateLength::Long,
                DateLength::Full,
            ]
            .map(|length| loc.format_date(date(), length))
        };
        assert_eq!(
            all("en"),
            [
                "10/16/26",
                "Oct 16, 2026",
                "October 16, 2026",
                "Friday, October 16, 2026"
            ]
        );
        assert_eq!(
            all("de"),
            [
                "16.10.26",
                "16.10.2026",
                "16. Oktober 2026",
                "Freitag, 16. Oktober 2026"
            ]
        );
        assert_eq!(DateLength::default(), DateLength::Medium);
    });
}

#[test]
fn i18n_fmt_03_times() {
    with(|| {
        let en = localization("en");
        assert_eq!(en.format_time(time()), format!("9:05{NNBSP}PM"));
        assert_eq!(localization("de").format_time(time()), "21:05");
        assert_eq!(
            en.format_date_time(date(), time(), DateLength::Medium),
            format!("Oct 16, 2026, 9:05{NNBSP}PM")
        );
        assert_eq!(
            localization("de").format_date_time(date(), time(), DateLength::Medium),
            "16.10.2026, 21:05"
        );
    });
}

#[test]
fn i18n_fmt_04_names_and_weeks() {
    with(|| {
        let en = localization("en");
        let de = localization("de");
        assert_eq!(en.month_name(10, false), "October");
        assert_eq!(en.month_name(10, true), "Oct");
        assert_eq!(de.month_name(10, false), "Oktober");
        assert_eq!(en.month_name(13, false), "");
        assert_eq!(en.weekday_name(Weekday::Friday, false), "Friday");
        assert_eq!(en.weekday_name(Weekday::Friday, true), "Fri");
        assert_eq!(de.weekday_name(Weekday::Friday, true), "Fr");
        assert_eq!(de.weekday_name(Weekday::Monday, false), "Montag");
        assert_eq!(en.format_month_year(2026, 10), "October 2026");
        assert_eq!(de.format_month_year(2026, 10), "Oktober 2026");
        assert_eq!(localization("en").first_weekday(), Weekday::Sunday);
        assert_eq!(de.first_weekday(), Weekday::Monday);
    });
}

#[test]
fn i18n_fmt_05_reactive() {
    with(|| {
        let loc = localization("en");
        let seen: Rc<RefCell<Vec<String>>> = Rc::default();
        {
            let seen = seen.clone();
            effect(move || seen.borrow_mut().push(loc.format_integer(1000)));
        }
        loc.set_locale(locale("de"));
        assert_eq!(*seen.borrow(), ["1,000", "1.000"]);
    });
}

#[test]
fn i18n_fmt_06_moved_values() {
    // The same types through both crates.
    let d: tantu_widgets_path::Date = date();
    assert_eq!(d.to_string(), "2026-10-16");
    assert_eq!(d.month_name(), "October");
    assert_eq!(Weekday::from_index(8), Weekday::Tuesday);
    assert_eq!(Weekday::Friday.index(), 4);
    assert_eq!(Weekday::Sunday.plus(1), Weekday::Monday);
    assert_eq!(Time::from_minutes(-5), Time::new(23, 55).expect("exists"));
    assert_eq!(time().minutes_of_day(), 21 * 60 + 5);
}

/// `tantu-widgets` re-exports the moved types (checked in its own tests); here the path is
/// this crate's.
mod tantu_widgets_path {
    pub use tantu_i18n::Date;
}
