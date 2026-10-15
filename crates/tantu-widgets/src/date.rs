//! [`Date`] and [`Weekday`]: calendar dates for the date picker. Spec:
//! `docs/specs/widgets/date-picker.md` (WIDGETS-DATE-01).

use std::fmt;

/// A calendar date (proleptic Gregorian, years 1 to 9999).
///
/// ```
/// use tantu_widgets::{Date, Weekday};
/// let date = Date::new(2026, 10, 16).expect("exists");
/// assert_eq!(date.weekday(), Weekday::Friday);
/// assert_eq!(date.add_months(4).map(|d| d.to_string()).as_deref(), Some("2027-02-16"));
/// assert_eq!(Date::parse_iso("2026-10-16"), Some(date));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// `None` unless the date exists (and lies in years 1 to 9999).
    pub fn new(year: i32, month: u32, day: u32) -> Option<Date> {
        let exists =
            (1..=9999).contains(&year) && day >= 1 && day <= Date::days_in_month(year, month);
        // Both fit: the month is 1 to 12 and the day at most 31.
        exists.then_some(Date {
            year,
            month: month as u8,
            day: day as u8,
        })
    }

    /// A date known to exist (no checks).
    pub(crate) const fn from_ymd_unchecked(year: i32, month: u8, day: u8) -> Date {
        Date { year, month, day }
    }

    /// The month's English name ("October").
    pub(crate) fn month_name(self) -> &'static str {
        MONTHS[usize::from(self.month) - 1]
    }

    /// The year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 to 12.
    pub fn month(self) -> u32 {
        u32::from(self.month)
    }

    /// The day of the month, from 1.
    pub fn day(self) -> u32 {
        u32::from(self.day)
    }

    /// The day of the week.
    pub fn weekday(self) -> Weekday {
        // 1970-01-01 was a Thursday.
        WEEKDAYS[(self.days() + 3).rem_euclid(7) as usize]
    }

    /// The number of days in `month` of `year` (0 for a month that doesn't exist).
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => 0,
        }
    }

    /// The date `days` later (earlier when negative); `None` past year 1 or 9999.
    pub fn add_days(self, days: i64) -> Option<Date> {
        Date::from_days(self.days().checked_add(days)?)
    }

    /// The same day `months` later, clamped to the month's length; `None` out of range.
    pub fn add_months(self, months: i32) -> Option<Date> {
        let index = i64::from(self.year) * 12 + i64::from(self.month) - 1 + i64::from(months);
        let year = i32::try_from(index.div_euclid(12)).ok()?;
        let month = index.rem_euclid(12) as u32 + 1;
        let day = self.day().min(Date::days_in_month(year, month));
        Date::new(year, month, day)
    }

    /// Reads `YYYY-MM-DD`.
    pub fn parse_iso(text: &str) -> Option<Date> {
        let bytes = text.as_bytes();
        let digits = |range: std::ops::Range<usize>| {
            let part = bytes.get(range.clone())?;
            part.iter()
                .all(u8::is_ascii_digit)
                .then(|| text[range].parse::<u32>().ok())?
        };
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year = i32::try_from(digits(0..4)?).ok()?;
        Date::new(year, digits(5..7)?, digits(8..10)?)
    }

    /// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
    fn days(self) -> i64 {
        let (month, day) = (i64::from(self.month), i64::from(self.day));
        let year = i64::from(self.year) - i64::from(month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// The date `days` after 1970-01-01, if in range (`civil_from_days`).
    fn from_days(days: i64) -> Option<Date> {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let day_of_era = z - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * shifted + 2) / 5 + 1;
        let month = if shifted < 10 {
            shifted + 3
        } else {
            shifted - 9
        };
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        Date::new(i32::try_from(year).ok()?, month as u32, day as u32)
    }
}

/// The months' English names (localized names wait for the i18n item).
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The weekdays from Monday.
const WEEKDAYS: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

/// Writes `YYYY-MM-DD`.
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A day of the week.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl Weekday {
    /// "Monday", …
    pub fn name(self) -> &'static str {
        [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ][self.index()]
    }

    /// "Mo", "Tu", …
    pub fn short_name(self) -> &'static str {
        &self.name()[..2]
    }

    /// 0 for Monday to 6 for Sunday.
    pub(crate) fn index(self) -> usize {
        self as usize
    }

    /// The weekday `n` days after this one.
    pub(crate) fn plus(self, n: usize) -> Weekday {
        WEEKDAYS[(self.index() + n) % 7]
    }
}
