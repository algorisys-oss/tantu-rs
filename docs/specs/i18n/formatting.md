# i18n, part 3: locale-aware number and date formatting

- **Status:** Agreed (2026-10-16; the user asked to implement translations and locale
  formatting; the proposals are taken and marked review)
- **Crates:** `tantu-i18n` (formatting on `Localization`; `Date`, `Time`, `Weekday` moved
  here), `tantu-widgets` (re-exports; the calendar's names and first weekday),
  `examples/gallery`
- **Plan item:** Phase 4, i18n, part 3 (locale-aware formatting)
- **Related:** [ADR 0015](../../adr/0015-internationalization-crate.md),
  [localization](localization.md), [date picker](../widgets/date-picker.md),
  [time picker](../widgets/time-picker.md)

## Purpose

`1234567.5` is "1,234,567.5" in the US, "1.234.567,5" in Germany, "1 234 567,5" in France
and "12,34,567.5" in India; 16 October 2026 is "10/16/26", "16.10.26" or "16/10/2026";
weeks start on Sunday in the US and Monday in most of Europe. This part formats numbers,
dates and times for the current locale through ICU4X, and makes the calendar widget use
the locale's names and first weekday.

## Scope

In scope: integers and fixed-precision decimals; dates (short, medium, long, full), times
(short) and both together; month and weekday names; the locale's first day of the week;
`Date`, `Time` and `Weekday` moving to `tantu-i18n`; the calendar using all of these.

Out of scope: currencies, percentages, units and compact numbers (ICU4X's experimental
crates); parsing localized input (the date and time fields keep typing ISO `YYYY-MM-DD`
and `HH:MM`); non-Gregorian calendars (formatting always uses the Gregorian calendar);
time zones.

## Public API

```rust
// tantu-i18n (Date, Time and Weekday as in the date and time picker specs, moved here)
/// How much of a date to write.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DateLength { Short, #[default] Medium, Long, Full }

impl Localization {
    /// `value` with the locale's grouping and digits (a tracked read).
    pub fn format_integer(&self, value: i64) -> String;
    /// `value` rounded half-even to `fraction_digits` digits after the separator (shown even
    /// when zero); NaN and infinities are written as "NaN", "∞" and "-∞".
    pub fn format_decimal(&self, value: f64, fraction_digits: u8) -> String;
    pub fn format_date(&self, date: Date, length: DateLength) -> String;
    /// Hours and minutes, with the locale's 12- or 24-hour convention.
    pub fn format_time(&self, time: Time) -> String;
    pub fn format_date_time(&self, date: Date, time: Time, length: DateLength) -> String;
    /// "October" (`wide`) or "Oct" (abbreviated) in the locale.
    pub fn month_name(&self, month: u32, abbreviated: bool) -> String;
    /// "Friday" (`wide`) or the short form used in calendar headers ("Fr").
    pub fn weekday_name(&self, day: Weekday, short: bool) -> String;
    /// The month and year as a calendar header writes them ("October 2026").
    pub fn format_month_year(&self, year: i32, month: u32) -> String;
    /// The first day of the week.
    pub fn first_weekday(&self) -> Weekday;
}
```

## Behavior

- **I18N-FMT-01:** `format_integer` and `format_decimal` follow the locale's decimal
  format through ICU4X (grouping sizes, separators, minus sign); for example 1234567 is
  `1,234,567` (en), `1.234.567` (de), `1 234 567` with U+202F (fr) and `12,34,567`
  (hi). `format_decimal(2.5, 2)` is `2.50` (en) and `2,50` (de); rounding is half to even
  (`0.125` to 2 digits is `0.12`).
- **I18N-FMT-02:** `format_date` uses the locale's year-month-day pattern at the length:
  for 2026-10-16, en gives `10/16/26`, `Oct 16, 2026`, `October 16, 2026`,
  `Friday, October 16, 2026`; de gives `16.10.26`, `16.10.2026`, `16. Oktober 2026`,
  `Freitag, 16. Oktober 2026` (the exact strings are ICU4X's compiled data).
- **I18N-FMT-03:** `format_time` writes hours and minutes with the locale's hour cycle
  (`9:05 PM` en, `21:05` de); `format_date_time` joins them with the locale's glue
  pattern.
- **I18N-FMT-04:** `month_name`, `weekday_name`, `format_month_year` and `first_weekday`
  come from the locale's data (`Oktober`, `Freitag`, `Oktober 2026`; first weekday Sunday
  for en-US, Monday for de and fr); a month outside 1..=12 gives an empty string.
- **I18N-FMT-05:** Every formatting method reads the locale signal, so formatted text in a
  view follows `set_locale`. Without a `Localization`, widgets format as `en`.
- **I18N-FMT-06:** `Date`, `Time` and `Weekday` keep their API and rules (WIDGETS-DATE-01,
  WIDGETS-TIME-01) in `tantu-i18n`; `tantu-widgets` re-exports them, so existing paths keep
  working.
- **WIDGETS-I18N-03:** Under a `Localization`, `CalendarDatePicker` (and the date field's
  popup) writes its header with `format_month_year`, its weekday row with short weekday
  names, its day cells' names with `format_date(Full)`, and starts the week on
  `first_weekday` unless `first_day_of_week` was set; without one it behaves as before
  (English, Monday).
- **GALLERY-52:** The "i18n" kitchen-sink card also shows 1234567.891 with two digits, the
  date 2026-10-16 (long) and the time 21:05 in the gallery's language, echoed in
  `sink-i18n-format` ("1,234,567.89 · October 16, 2026 · 9:05 PM" in English).

## Performance and allocation

Formatters are created once per locale and kept in the localization; a format call
allocates its result string.

## Open questions

Decided on the user's request to implement (2026-10-16), for review:

1. **ICU4X 2.3 with compiled data** (all locales); the binary-size growth is recorded in
   HANDOFF.md, and trimming the data to chosen locales is a later feature if needed.
2. **Gregorian only** and **ISO typed input** in the date and time fields.
3. **`Date`/`Time`/`Weekday` move to `tantu-i18n`** with re-exports.
