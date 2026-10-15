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
        let _ = (hour, minute);
        todo!("WIDGETS-TIME-01")
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
        let _ = minutes;
        todo!("WIDGETS-TIME-01")
    }

    /// Reads `HH:MM` (two digits each).
    pub fn parse(text: &str) -> Option<Time> {
        let _ = text;
        todo!("WIDGETS-TIME-01")
    }
}

/// Writes `HH:MM`.
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}
