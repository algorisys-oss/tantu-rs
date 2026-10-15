# Forms, part 4a: date picker

- **Status:** Agreed (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (`Date`, `Weekday`, `CalendarDatePicker`, `DatePickerField`),
  `examples/gallery`
- **Plan item:** Phase 4, "Forms: …, date/time pickers", part 4a (dates; times are part 4b)
- **Related:** [combo box](combo-box.md), [masked input](masked-input.md),
  [forms part 1](form.md), [DataGrid selection](data-grid-selection.md),
  [semantics](../a11y/semantics.md), [gallery](../examples/gallery.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Business forms ask for dates all the time: due dates, birthdays, report ranges. Users either
type them or pick them from a month calendar. Flutter has `CalendarDatePicker` (the month
grid) and `InputDatePickerFormField` plus `showDatePicker` (typing, or a calendar popup).
Tantu provides the same two pieces, desktop-style: a calendar grid driven by the keyboard
and the pointer, and a date field with a calendar popup.

## Scope

In scope: a plain `Date` value (proleptic Gregorian, years 1 to 9999) with the arithmetic a
calendar needs and ISO 8601 text; a month calendar with a selectable range, a "today"
marker and a configurable first day of the week; a date field typed as `YYYY-MM-DD` with a
calendar popup; semantics; gallery samples.

Out of scope (and where it goes): localized month and weekday names and date formats (the
i18n item; English names until then); date ranges (two-date selection); a year grid;
time zones and reading the system clock (apps pass `today`); times (part 4b).

## Public API

```rust
// tantu-widgets
/// A calendar date (proleptic Gregorian, years 1 to 9999).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date { /* private */ }
impl Date {
    /// `None` unless the date exists.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Date>;
    pub fn year(self) -> i32;
    pub fn month(self) -> u32;
    pub fn day(self) -> u32;
    pub fn weekday(self) -> Weekday;
    /// The number of days in `month` of `year` (0 for a month that doesn't exist).
    pub fn days_in_month(year: i32, month: u32) -> u32;
    /// `None` past year 1 or 9999.
    pub fn add_days(self, days: i64) -> Option<Date>;
    /// The same day `months` later, clamped to the month's length; `None` out of range.
    pub fn add_months(self, months: i32) -> Option<Date>;
    /// Reads `YYYY-MM-DD`.
    pub fn parse_iso(text: &str) -> Option<Date>;
}
/// Writes `YYYY-MM-DD`.
impl std::fmt::Display for Date;

/// A day of the week.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weekday { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
impl Weekday {
    /// "Monday", …
    pub fn name(self) -> &'static str;
    /// "Mo", "Tu", …
    pub fn short_name(self) -> &'static str;
}

/// A month calendar for choosing a date (Flutter's `CalendarDatePicker`).
pub struct CalendarDatePicker { /* private */ }
impl CalendarDatePicker {
    pub fn new(selected: Signal<Option<Date>>) -> Self;
    /// The earliest choosable date (default 1900-01-01).
    pub fn first_date(self, date: Date) -> Self;
    /// The latest choosable date (default 2100-12-31).
    pub fn last_date(self, date: Date) -> Self;
    /// The date marked as today (default: none marked).
    pub fn today(self, date: Date) -> Self;
    /// The first column's weekday (default Monday).
    pub fn first_day_of_week(self, day: Weekday) -> Self;
    /// Called with the chosen date after the user picks one that differs.
    pub fn on_change(self, f: impl Fn(Date) + 'static) -> Self;
}

/// A date typed as `YYYY-MM-DD` or picked from a calendar popup.
pub struct DatePickerField { /* private */ }
impl DatePickerField {
    pub fn new(selected: Signal<Option<Date>>) -> Self;
    /// The accessible name (default "Date").
    pub fn label(self, label: impl Into<String>) -> Self;
    /// The field's width, the button included (default 240 px).
    pub fn width(self, width: f32) -> Self;
    pub fn first_date(self, date: Date) -> Self;
    pub fn last_date(self, date: Date) -> Self;
    pub fn today(self, date: Date) -> Self;
    pub fn first_day_of_week(self, day: Weekday) -> Self;
    pub fn on_change(self, f: impl Fn(Date) + 'static) -> Self;
}
```

## Behavior

- **WIDGETS-DATE-01:** `Date` holds existing proleptic Gregorian dates from 0001-01-01 to
  9999-12-31 (leap years: divisible by 4, except centuries not divisible by 400). Dates
  order chronologically. `add_days` and `add_months` return `None` outside that range;
  `add_months` clamps the day (2024-01-31 plus one month is 2024-02-29). `parse_iso` reads
  exactly four year digits, two month digits and two day digits separated by `-`, and
  `Display` writes the same.
- **WIDGETS-DATE-02:** Layout: a `CalendarDatePicker` is 280 × 320 px: a 48 px header with
  the displayed month's name and year ("October 2026", theme `title` style at 16 px) on the
  left and previous/next month `IconButton`s (`CHEVRON_LEFT`, `CHEVRON_RIGHT`) on the
  right; a 32 px row of weekday short names (`on_surface_variant`) starting with the first
  day of the week; and six 40 px week rows of seven 40 px columns, each day of the
  displayed month in its weekday's column (other months' days are blank). The displayed
  month starts as the selected date's, else today's, else the month of the first date (each
  clamped into the range); the buttons move it by one month and are disabled where the range
  ends.
- **WIDGETS-DATE-03:** Appearance: each day is a 36 px circle centered in its cell with its
  number (theme `body`); the selected day is filled `primary` with `on_primary` text; today
  is outlined `primary` (1 px) with `primary` text; days outside the range are drawn at
  38 % and can't be chosen; a hovered day has an `on_surface` 8 % layer, the keyboard
  cursor (while the picker or its field has the focus) a 12 % layer.
- **WIDGETS-DATE-04:** Choosing: a primary click on a choosable day sets `selected` and
  moves the cursor there; `on_change` is called when the date changed.
- **WIDGETS-DATE-05:** Keyboard: the picker is one Tab stop. Its cursor starts on the
  selected date if displayed, else today if displayed, else the displayed month's first
  choosable day. Left/Right move it by a day, Up/Down by a week, PageUp/PageDown by a month
  (with Shift, a year), Home/End to the first/last day of its month; every move is clamped
  to the range, and the displayed month follows the cursor. Enter or Space chooses the
  cursor's day.
- **WIDGETS-DATE-06:** Semantics: the picker is a `Grid` named by the displayed month and
  year, with seven columns and six rows; the weekday names are `ColumnHeader`s named by
  their full names; each day is a `Cell` named like "Friday, October 16, 2026", with its
  row and column index, `selected` for the selected date, `disabled` outside the range; the
  grid's `active_descendant` is the cursor's cell. The buttons are named "Previous month"
  and "Next month".
- **WIDGETS-DATE-07:** `DatePickerField`: a `TextField` masked `####-##-##` (placeholder
  `YYYY-MM-DD`) followed by a 40 px calendar `IconButton` (`CALENDAR_TODAY`, named "Choose
  date"), together `width` wide. It shows the selected date (empty for `None`) and follows
  `selected` while it doesn't have the focus. Typing a complete date that exists and lies in
  the range sets `selected` (and calls `on_change` if it changed); a complete text that
  isn't such a date shows the error outline with the description "Invalid date". Losing
  the focus restores the selected date's text.
- **WIDGETS-DATE-08:** The field's popup: the button (which also focuses the field) or
  Alt+Down in the field opens a `CalendarDatePicker` over the same selection and options
  below the field (`Anchor::below()`, flip and clamp), on the theme's `surface` with a 4 px
  radius, a level-2 shadow and 8 px padding. The focus stays in the field: while the popup
  is open the calendar keys of WIDGETS-DATE-05 move its cursor, Enter chooses, Escape or a
  press outside closes it; choosing (by key or click) closes it. The field is a `ComboBox`
  with `expanded` while open and the cursor's cell as its `active_descendant`.
- **WIDGETS-DATE-09:** The popup's portal target is the field and its button together, so
  a press on the button doesn't count as outside the popup (the button toggles it); the
  field's active descendant is accepted through A11Y-TREE-13's enclosing target (found
  while implementing, review).
- **GALLERY-42:** The Selection section gains a "Due date" `DatePickerField` (range
  2026-01-01 to 2027-12-31, today 2026-10-16) with a "Due: …" echo (`date-echo`: "Due:
  none", "Due: 2026-10-20"). Tests: typing `20261020` sets it; Alt+Down opens the popup on
  October 2026; Right, Enter picks 2026-10-21. Golden `gallery-date.png` with the popup open.
- **GALLERY-43:** A "date" kitchen-sink card after "mask" (twenty-three `SINK_CARDS`
  entries, sample `calendar_date_picker`): a standalone calendar starting on 2026-10-16,
  echoed by `sink-date-state` ("Picked: 2026-10-16").

## Performance and allocation

The calendar builds 42 day cells per displayed month and rebuilds them only when the month
changes; selection and cursor changes repaint.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **An in-house `Date`** rather than a `chrono` or `time` dependency: a calendar needs
   only Gregorian arithmetic, and apps convert at the edge.
2. **Monday first by default** (ISO 8601); apps in the US pass `Weekday::Sunday`.
3. **ISO `YYYY-MM-DD` text** until the i18n item brings locale formats.
4. **The focus stays in the field** while its popup is open (the WAI-ARIA combobox pattern
   with a grid popup), as in the combo box.
5. **No reading of the system clock**: `today` is passed by the app.
