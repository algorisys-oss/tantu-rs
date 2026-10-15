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
        let _ = (year, month, day);
        todo!("WIDGETS-DATE-01")
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
        todo!("WIDGETS-DATE-01")
    }

    /// The number of days in `month` of `year` (0 for a month that doesn't exist).
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        let _ = (year, month);
        todo!("WIDGETS-DATE-01")
    }

    /// The date `days` later (earlier when negative); `None` past year 1 or 9999.
    pub fn add_days(self, days: i64) -> Option<Date> {
        let _ = days;
        todo!("WIDGETS-DATE-01")
    }

    /// The same day `months` later, clamped to the month's length; `None` out of range.
    pub fn add_months(self, months: i32) -> Option<Date> {
        let _ = months;
        todo!("WIDGETS-DATE-01")
    }

    /// Reads `YYYY-MM-DD`.
    pub fn parse_iso(text: &str) -> Option<Date> {
        let _ = text;
        todo!("WIDGETS-DATE-01")
    }
}

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
        todo!("WIDGETS-DATE-06")
    }

    /// "Mo", "Tu", …
    pub fn short_name(self) -> &'static str {
        todo!("WIDGETS-DATE-02")
    }
}
