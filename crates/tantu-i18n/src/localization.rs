//! [`Localization`]: the current locale, message lookup and formatting. Specs:
//! `docs/specs/i18n/localization.md` and `docs/specs/i18n/formatting.md`.

use tantu_view::BuildCx;
use tantu_view::reactive::Signal;

use crate::{Args, Date, Locale, Time, Translations, Weekday};

/// How much of a date to write.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DateLength {
    /// `10/16/26`.
    Short,
    /// `Oct 16, 2026`.
    #[default]
    Medium,
    /// `October 16, 2026`.
    Long,
    /// `Friday, October 16, 2026`.
    Full,
}

/// The current locale and the translations (`Copy`; provided by the `Localizations`
/// widget). Create it while a runtime is current (its locale is a signal).
#[derive(Clone, Copy)]
pub struct Localization {
    locale: Signal<Locale>,
}

impl Localization {
    /// `translations` shown in `locale` (negotiated against the available ones).
    pub fn new(translations: Translations, locale: Locale) -> Self {
        let _ = (translations, locale);
        todo!("I18N-MSG-04")
    }

    /// The current locale (a tracked read).
    pub fn locale(&self) -> Locale {
        self.locale.get()
    }

    /// Switches the locale (negotiated against the available ones).
    pub fn set_locale(&self, locale: Locale) {
        let _ = locale;
        todo!("I18N-MSG-04")
    }

    /// The message `id` in the current locale, or `id` itself when missing (a tracked read).
    pub fn text(&self, id: &str) -> String {
        let _ = id;
        todo!("I18N-MSG-02")
    }

    /// The message with arguments.
    pub fn text_with(&self, id: &str, args: &Args) -> String {
        let _ = (id, args);
        todo!("I18N-MSG-03")
    }

    /// The message, or `None` when no locale in the chain has it (a tracked read).
    pub fn message(&self, id: &str, args: Option<&Args>) -> Option<String> {
        let _ = (id, args);
        todo!("I18N-MSG-06")
    }

    /// The nearest localization above the element being built.
    pub fn of(cx: &BuildCx<'_>) -> Option<Localization> {
        cx.inherit::<Localization>()
    }

    /// `value` with the locale's grouping and digits (a tracked read).
    pub fn format_integer(&self, value: i64) -> String {
        let _ = value;
        todo!("I18N-FMT-01")
    }

    /// `value` rounded half-even to `fraction_digits` digits after the separator.
    pub fn format_decimal(&self, value: f64, fraction_digits: u8) -> String {
        let _ = (value, fraction_digits);
        todo!("I18N-FMT-01")
    }

    /// The date at `length` in the locale's pattern.
    pub fn format_date(&self, date: Date, length: DateLength) -> String {
        let _ = (date, length);
        todo!("I18N-FMT-02")
    }

    /// Hours and minutes, with the locale's 12- or 24-hour convention.
    pub fn format_time(&self, time: Time) -> String {
        let _ = time;
        todo!("I18N-FMT-03")
    }

    /// A date and a time joined in the locale's way.
    pub fn format_date_time(&self, date: Date, time: Time, length: DateLength) -> String {
        let _ = (date, time, length);
        todo!("I18N-FMT-03")
    }

    /// "October", or "Oct" when `abbreviated`; empty for a month outside 1..=12.
    pub fn month_name(&self, month: u32, abbreviated: bool) -> String {
        let _ = (month, abbreviated);
        todo!("I18N-FMT-04")
    }

    /// "Friday", or the short form calendar headers use when `short`.
    pub fn weekday_name(&self, day: Weekday, short: bool) -> String {
        let _ = (day, short);
        todo!("I18N-FMT-04")
    }

    /// The month and year as a calendar header writes them ("October 2026").
    pub fn format_month_year(&self, year: i32, month: u32) -> String {
        let _ = (year, month);
        todo!("I18N-FMT-04")
    }

    /// The first day of the week.
    pub fn first_weekday(&self) -> Weekday {
        todo!("I18N-FMT-04")
    }
}
