//! [`CalendarDatePicker`] and [`DatePickerField`]. Spec:
//! `docs/specs/widgets/date-picker.md`.

use std::rc::Rc;

use tantu_view::reactive::Signal;
use tantu_view::{BuildCx, ElementId, View};

use crate::date::{Date, Weekday};

/// The range and markers a calendar shares with its field.
#[derive(Clone)]
struct Options {
    first: Date,
    last: Date,
    today: Option<Date>,
    first_day: Weekday,
    on_change: Option<Rc<dyn Fn(Date)>>,
}

impl Options {
    fn new() -> Self {
        // Placeholders until WIDGETS-DATE-02 (the stubbed `Date::new` can't run here).
        let _ = Date::new;
        Options {
            first: todo_date(),
            last: todo_date(),
            today: None,
            first_day: Weekday::Monday,
            on_change: None,
        }
    }
}

fn todo_date() -> Date {
    todo!("WIDGETS-DATE-02")
}

/// A month calendar for choosing a date (Flutter's `CalendarDatePicker`,
/// WIDGETS-DATE-02..06).
pub struct CalendarDatePicker {
    selected: Signal<Option<Date>>,
    options: Options,
}

impl CalendarDatePicker {
    /// A calendar choosing into `selected`.
    pub fn new(selected: Signal<Option<Date>>) -> Self {
        CalendarDatePicker {
            selected,
            options: Options::new(),
        }
    }

    /// The earliest choosable date (default 1900-01-01).
    pub fn first_date(mut self, date: Date) -> Self {
        self.options.first = date;
        self
    }

    /// The latest choosable date (default 2100-12-31).
    pub fn last_date(mut self, date: Date) -> Self {
        self.options.last = date;
        self
    }

    /// The date marked as today (default: none marked).
    pub fn today(mut self, date: Date) -> Self {
        self.options.today = Some(date);
        self
    }

    /// The first column's weekday (default Monday).
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.options.first_day = day;
        self
    }

    /// Called with the chosen date after the user picks one that differs.
    pub fn on_change(mut self, f: impl Fn(Date) + 'static) -> Self {
        self.options.on_change = Some(Rc::new(f));
        self
    }
}

impl View for CalendarDatePicker {
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.selected, self.options);
        todo!("WIDGETS-DATE-02")
    }
}

/// A date typed as `YYYY-MM-DD` or picked from a calendar popup (WIDGETS-DATE-07, -08).
pub struct DatePickerField {
    selected: Signal<Option<Date>>,
    options: Options,
    label: String,
    width: f32,
}

impl DatePickerField {
    /// A field choosing into `selected`.
    pub fn new(selected: Signal<Option<Date>>) -> Self {
        DatePickerField {
            selected,
            options: Options::new(),
            label: "Date".to_owned(),
            width: 240.0,
        }
    }

    /// The accessible name (default "Date").
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// The field's width, the button included (default 240 px).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// The earliest choosable date (default 1900-01-01).
    pub fn first_date(mut self, date: Date) -> Self {
        self.options.first = date;
        self
    }

    /// The latest choosable date (default 2100-12-31).
    pub fn last_date(mut self, date: Date) -> Self {
        self.options.last = date;
        self
    }

    /// The date marked as today in the calendar (default: none marked).
    pub fn today(mut self, date: Date) -> Self {
        self.options.today = Some(date);
        self
    }

    /// The calendar's first column (default Monday).
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.options.first_day = day;
        self
    }

    /// Called with the chosen date after the user picks or types one that differs.
    pub fn on_change(mut self, f: impl Fn(Date) + 'static) -> Self {
        self.options.on_change = Some(Rc::new(f));
        self
    }
}

impl View for DatePickerField {
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.selected, self.options, self.label, self.width);
        todo!("WIDGETS-DATE-07")
    }
}
