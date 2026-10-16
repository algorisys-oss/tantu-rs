//! ICU4X formatters for one locale. Spec: `docs/specs/i18n/formatting.md`
//! (I18N-FMT-01..04).

use std::str::FromStr;

use icu_calendar::Gregorian;
use icu_datetime::options::TimePrecision;
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};
use icu_decimal::DecimalFormatter;
use icu_decimal::input::Decimal;

use crate::{Date, DateLength, Locale, Time, Weekday};

/// Formats numbers in one locale (also Fluent's number formatter, I18N-MSG-03).
pub(crate) struct NumberFormatter {
    decimal: DecimalFormatter,
}

impl NumberFormatter {
    fn new(locale: &icu_locale_core::Locale) -> Option<Self> {
        DecimalFormatter::try_new(locale.into(), Default::default())
            .ok()
            .map(|decimal| NumberFormatter { decimal })
    }

    /// `value` rounded half to even at `digits` fraction digits (I18N-FMT-01).
    pub(crate) fn format_fixed(&self, value: f64, digits: u8) -> String {
        if let Some(special) = special(value) {
            return special;
        }
        let Ok(mut decimal) = Decimal::from_str(&format!("{value}")) else {
            return format!("{value:.*}", usize::from(digits));
        };
        let position = -i16::from(digits);
        decimal.round(position);
        decimal.absolute.pad_end(position);
        self.decimal.format(&decimal).to_string()
    }

    /// `value` with as many fraction digits as it has.
    pub(crate) fn format_shortest(&self, value: f64) -> String {
        if let Some(special) = special(value) {
            return special;
        }
        match Decimal::from_str(&format!("{value}")) {
            Ok(decimal) => self.decimal.format(&decimal).to_string(),
            Err(_) => format!("{value}"),
        }
    }
}

/// NaN and the infinities, which ICU4X's decimals can't hold.
fn special(value: f64) -> Option<String> {
    if value.is_nan() {
        Some("NaN".to_owned())
    } else if value == f64::INFINITY {
        Some("∞".to_owned())
    } else if value == f64::NEG_INFINITY {
        Some("-∞".to_owned())
    } else {
        None
    }
}

impl intl_memoizer::Memoizable for NumberFormatter {
    type Args = ();
    type Error = ();

    fn construct(lang: unic_langid::LanguageIdentifier, _: ()) -> Result<Self, ()> {
        let locale = icu_locale_core::Locale::try_from_str(&lang.to_string()).map_err(|_| ())?;
        NumberFormatter::new(&locale).ok_or(())
    }
}

type Formatter<F> = Option<FixedCalendarDateTimeFormatter<Gregorian, F>>;

/// Every formatter one locale needs, created once.
pub(crate) struct Formats {
    numbers: Option<NumberFormatter>,
    dates: [Formatter<fieldsets::YMD>; 3],
    full_date: Formatter<fieldsets::YMDE>,
    time: Formatter<fieldsets::T>,
    date_times: [Formatter<fieldsets::YMDT>; 3],
    full_date_time: Formatter<fieldsets::YMDET>,
    month: Formatter<fieldsets::M>,
    month_short: Formatter<fieldsets::M>,
    month_year: Formatter<fieldsets::YM>,
    weekday: Formatter<fieldsets::E>,
    weekday_short: Formatter<fieldsets::E>,
    first_weekday: Weekday,
}

impl Formats {
    pub(crate) fn new(locale: &Locale) -> Self {
        let id = locale.icu();
        let prefs = || id.into();
        let dt = |set| FixedCalendarDateTimeFormatter::try_new(prefs(), set).ok();
        let minute = TimePrecision::Minute;
        Formats {
            numbers: NumberFormatter::new(id),
            dates: [
                dt(fieldsets::YMD::short()),
                dt(fieldsets::YMD::medium()),
                dt(fieldsets::YMD::long()),
            ],
            full_date: FixedCalendarDateTimeFormatter::try_new(prefs(), fieldsets::YMDE::long())
                .ok(),
            time: FixedCalendarDateTimeFormatter::try_new(prefs(), fieldsets::T::hm()).ok(),
            date_times: [
                FixedCalendarDateTimeFormatter::try_new(
                    prefs(),
                    fieldsets::YMDT::short().with_time_precision(minute),
                )
                .ok(),
                FixedCalendarDateTimeFormatter::try_new(
                    prefs(),
                    fieldsets::YMDT::medium().with_time_precision(minute),
                )
                .ok(),
                FixedCalendarDateTimeFormatter::try_new(
                    prefs(),
                    fieldsets::YMDT::long().with_time_precision(minute),
                )
                .ok(),
            ],
            full_date_time: FixedCalendarDateTimeFormatter::try_new(
                prefs(),
                fieldsets::YMDET::long().with_time_precision(minute),
            )
            .ok(),
            month: FixedCalendarDateTimeFormatter::try_new(prefs(), fieldsets::M::long()).ok(),
            month_short: FixedCalendarDateTimeFormatter::try_new(prefs(), fieldsets::M::medium())
                .ok(),
            month_year: FixedCalendarDateTimeFormatter::try_new(prefs(), fieldsets::YM::long())
                .ok(),
            weekday: FixedCalendarDateTimeFormatter::try_new(prefs(), fieldsets::E::long()).ok(),
            weekday_short: FixedCalendarDateTimeFormatter::try_new(prefs(), fieldsets::E::short())
                .ok(),
            first_weekday: icu_calendar::week::WeekInformation::try_new(
                icu_calendar::week::WeekPreferences::from(id),
            )
            .map(|w| weekday(w.first_weekday))
            .unwrap_or(Weekday::Monday),
        }
    }

    pub(crate) fn integer(&self, value: i64) -> String {
        match &self.numbers {
            Some(n) => n.format_fixed(value as f64, 0),
            None => value.to_string(),
        }
    }

    pub(crate) fn decimal(&self, value: f64, digits: u8) -> String {
        match &self.numbers {
            Some(n) => n.format_fixed(value, digits),
            None => format!("{value:.*}", usize::from(digits)),
        }
    }

    pub(crate) fn date(&self, date: Date, length: DateLength) -> String {
        let Some(icu) = gregorian(date) else {
            return date.to_string();
        };
        let text = match length {
            DateLength::Short => self.dates[0].as_ref().map(|f| f.format(&icu).to_string()),
            DateLength::Medium => self.dates[1].as_ref().map(|f| f.format(&icu).to_string()),
            DateLength::Long => self.dates[2].as_ref().map(|f| f.format(&icu).to_string()),
            DateLength::Full => self.full_date.as_ref().map(|f| f.format(&icu).to_string()),
        };
        text.unwrap_or_else(|| date.to_string())
    }

    pub(crate) fn time(&self, time: Time) -> String {
        match (&self.time, clock(time)) {
            (Some(f), Some(icu)) => f.format(&icu).to_string(),
            _ => time.to_string(),
        }
    }

    pub(crate) fn date_time(&self, date: Date, time: Time, length: DateLength) -> String {
        let (Some(date_icu), Some(time_icu)) = (gregorian(date), clock(time)) else {
            return format!("{date} {time}");
        };
        let both = icu_time::DateTime {
            date: date_icu,
            time: time_icu,
        };
        let text = match length {
            DateLength::Short => self.date_times[0]
                .as_ref()
                .map(|f| f.format(&both).to_string()),
            DateLength::Medium => self.date_times[1]
                .as_ref()
                .map(|f| f.format(&both).to_string()),
            DateLength::Long => self.date_times[2]
                .as_ref()
                .map(|f| f.format(&both).to_string()),
            DateLength::Full => self
                .full_date_time
                .as_ref()
                .map(|f| f.format(&both).to_string()),
        };
        text.unwrap_or_else(|| format!("{date} {time}"))
    }

    pub(crate) fn month_name(&self, month: u32, abbreviated: bool) -> String {
        let Some(icu) = Date::new(2026, month, 1).and_then(gregorian) else {
            return String::new();
        };
        let formatter = if abbreviated {
            &self.month_short
        } else {
            &self.month
        };
        formatter
            .as_ref()
            .map_or_else(String::new, |f| f.format(&icu).to_string())
    }

    pub(crate) fn weekday_name(&self, day: Weekday, short: bool) -> String {
        // 2026-10-12 is a Monday.
        let Some(icu) = Date::new(2026, 10, 12 + day.index() as u32).and_then(gregorian) else {
            return String::new();
        };
        let formatter = if short {
            &self.weekday_short
        } else {
            &self.weekday
        };
        formatter
            .as_ref()
            .map_or_else(|| day.name().to_owned(), |f| f.format(&icu).to_string())
    }

    pub(crate) fn month_year(&self, year: i32, month: u32) -> String {
        let Some(icu) = Date::new(year, month, 1).and_then(gregorian) else {
            return String::new();
        };
        self.month_year
            .as_ref()
            .map_or_else(String::new, |f| f.format(&icu).to_string())
    }

    pub(crate) fn first_weekday(&self) -> Weekday {
        self.first_weekday
    }
}

fn gregorian(date: Date) -> Option<icu_calendar::Date<Gregorian>> {
    let (month, day) = (
        u8::try_from(date.month()).ok()?,
        u8::try_from(date.day()).ok()?,
    );
    icu_calendar::Date::try_new_gregorian(date.year(), month, day).ok()
}

fn clock(time: Time) -> Option<icu_time::Time> {
    let (hour, minute) = (
        u8::try_from(time.hour()).ok()?,
        u8::try_from(time.minute()).ok()?,
    );
    icu_time::Time::try_new(hour, minute, 0, 0).ok()
}

fn weekday(day: icu_calendar::types::Weekday) -> Weekday {
    use icu_calendar::types::Weekday as W;
    match day {
        W::Monday => Weekday::Monday,
        W::Tuesday => Weekday::Tuesday,
        W::Wednesday => Weekday::Wednesday,
        W::Thursday => Weekday::Thursday,
        W::Friday => Weekday::Friday,
        W::Saturday => Weekday::Saturday,
        W::Sunday => Weekday::Sunday,
    }
}
