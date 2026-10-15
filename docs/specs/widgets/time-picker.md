# Forms, part 4b: time picker

- **Status:** Implemented (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (`Time`, `TimePickerField`), `tantu-view` (`Role::SpinButton`),
  `tantu-a11y` (mapping), `examples/gallery`
- **Plan item:** Phase 4, "Forms: …, date/time pickers", part 4b (times)
- **Related:** [date picker](date-picker.md), [masked input](masked-input.md),
  [semantics](../a11y/semantics.md), [gallery](../examples/gallery.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Appointments, shifts and reminders need a time of day. On the desktop the common control is
a spin field: the user types `09:30`, or puts the caret in the hours or the minutes and
presses Up and Down (the Windows `DateTimePicker` in time mode, the HTML `<input
type="time">`). Tantu provides that as `TimePickerField`, over a plain `Time` value.

## Scope

In scope: a `Time` of day (hours and minutes) with wrapping arithmetic and `HH:MM` text; a
24-hour field typed as `HH:MM` with Up/Down stepping of the segment under the caret and a
minute step; spin-button semantics; gallery samples.

Out of scope: 12-hour clocks and AM/PM (the i18n item, with locale formats); seconds; a
popup list or dial of times (later, if wanted); time zones.

## Public API

```rust
// tantu-widgets
/// A time of day, 00:00 to 23:59.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time { /* private */ }
impl Time {
    /// `None` unless `hour < 24` and `minute < 60`.
    pub fn new(hour: u32, minute: u32) -> Option<Time>;
    pub fn hour(self) -> u32;
    pub fn minute(self) -> u32;
    /// `minutes` later (earlier when negative), wrapping around midnight.
    pub fn add_minutes(self, minutes: i32) -> Time;
    /// Reads `HH:MM` (two digits each).
    pub fn parse(text: &str) -> Option<Time>;
}
/// Writes `HH:MM`.
impl std::fmt::Display for Time;

/// A time typed as `HH:MM` or stepped with Up and Down.
pub struct TimePickerField { /* private */ }
impl TimePickerField {
    pub fn new(selected: Signal<Option<Time>>) -> Self;
    /// The accessible name (default "Time").
    pub fn label(self, label: impl Into<String>) -> Self;
    /// The field's width (default 120 px).
    pub fn width(self, width: f32) -> Self;
    /// How far Up/Down move the minutes (default 1; clamped to 1..=60).
    pub fn minute_step(self, minutes: u32) -> Self;
    /// Called with the time after the user types or steps one that differs.
    pub fn on_change(self, f: impl Fn(Time) + 'static) -> Self;
}

// tantu-view
pub enum Role { /* existing, */ SpinButton }
```

## Behavior

- **WIDGETS-TIME-01:** `Time` holds 00:00 to 23:59 and orders by the clock. `add_minutes`
  wraps around midnight in both directions. `parse` reads exactly two hour digits, `:` and
  two minute digits; `Display` writes the same.
- **WIDGETS-TIME-02:** `TimePickerField` is a `TextField` masked `##:##` (placeholder
  `HH:MM`), `width` wide, showing the selected time (empty for `None`) and following
  `selected` while it doesn't have the focus. Typing a complete valid time sets `selected`
  (and calls `on_change` if it changed); a complete text that isn't a time shows the error
  outline with the description "Invalid time". Losing the focus restores the selected
  time's text.
- **WIDGETS-TIME-03:** Stepping: Up and Down (no modifiers) change the selected time and
  the text, keeping the caret where it was: with the caret before the colon (offsets 0 to
  2) by an hour; after it by the minute step, snapping to the step first (with a 15-minute
  step, Up from 10:07 gives 10:15 and Down gives 10:00); both wrap around midnight. With no
  time selected (or an incomplete or invalid text), Up and Down select 00:00. Each step
  calls `on_change`.
- **WIDGETS-TIME-04:** Semantics: the field is a `SpinButton` named by the label, with the
  text as its value and the Increment and Decrement actions, which step the minutes as Up
  and Down do.
- **VIEW-SEM-05:** `Role::SpinButton` exists; Increment and Decrement on a `SpinButton` are
  synthesized as Up and Down (not Down and Up as for other non-slider roles).
- **A11Y-TREE-14:** `Role::SpinButton` maps to AccessKit's `SpinButton`.
- **GALLERY-44:** The Selection section gains a "Start" `TimePickerField` (15-minute step)
  under the date field, with a "Start: …" echo (`time-echo`: "Start: none", "Start:
  09:30"). Test: typing `0930` sets it; Up with the caret at the end gives 09:45.
- **GALLERY-45:** A "time" kitchen-sink card after "date" (twenty-four `SINK_CARDS`
  entries, sample `time_picker_field`): a field starting at 12:00, echoed by
  `sink-time-state` ("Time: 12:00").

## Performance and allocation

Nothing per frame; stepping and typing allocate the new text once.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **A spin field, not a popup list or dial**: the desktop convention, and the keyboard
   path is complete. A list of times can join later as an option.
2. **24-hour `HH:MM`** until the i18n item brings 12-hour locales.
3. **Up/Down from an empty field select 00:00**, rather than reading the system clock
   (the date picker doesn't read it either).
4. **`Role::SpinButton`** with Increment as Up, so screen readers' increment means "later".
