//! [`Time`]: a time of day for the time picker. Spec: `docs/specs/widgets/time-picker.md`
//! (WIDGETS-TIME-01).

use std::fmt;

/// A time of day, 00:00 to 23:59.
///
/// ```
/// use tantu_widgets::Time;
/// let time = Time::new(23, 45).expect("exists");
/// assert_eq!(time.add_minutes(30).to_string(), "00:15");
/// assert_eq!(Time::parse("23:45"), Some(time));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    hour: u8,
    minute: u8,
}

impl Time {
    /// `None` unless `hour < 24` and `minute < 60`.
    pub fn new(hour: u32, minute: u32) -> Option<Time> {
        (hour < 24 && minute < 60).then_some(Time {
            // Both fit: checked above.
            hour: hour as u8,
            minute: minute as u8,
        })
    }

    /// The hour, 0 to 23.
    pub fn hour(self) -> u32 {
        u32::from(self.hour)
    }

    /// The minute, 0 to 59.
    pub fn minute(self) -> u32 {
        u32::from(self.minute)
    }

    /// `minutes` later (earlier when negative), wrapping around midnight.
    pub fn add_minutes(self, minutes: i32) -> Time {
        let total = (self.minutes_of_day() + i64::from(minutes)).rem_euclid(MINUTES_PER_DAY);
        Time {
            // Both fit: `total` is below a day.
            hour: (total / 60) as u8,
            minute: (total % 60) as u8,
        }
    }

    /// Reads `HH:MM` (two digits each).
    pub fn parse(text: &str) -> Option<Time> {
        let bytes = text.as_bytes();
        if bytes.len() != 5 || bytes[2] != b':' {
            return None;
        }
        let digits = |range: std::ops::Range<usize>| {
            bytes[range.clone()]
                .iter()
                .all(u8::is_ascii_digit)
                .then(|| text[range].parse::<u32>().ok())?
        };
        Time::new(digits(0..2)?, digits(3..5)?)
    }

    /// The time `minutes` after midnight (wrapping).
    pub(crate) fn from_minutes(minutes: i64) -> Time {
        Time { hour: 0, minute: 0 }.add_minutes((minutes.rem_euclid(MINUTES_PER_DAY)) as i32)
    }

    /// Minutes since midnight.
    pub(crate) fn minutes_of_day(self) -> i64 {
        i64::from(self.hour) * 60 + i64::from(self.minute)
    }
}

/// Minutes in a day.
const MINUTES_PER_DAY: i64 = 24 * 60;

/// Writes `HH:MM`.
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}
