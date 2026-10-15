# i18n, part 2: locales and Fluent messages

- **Status:** Agreed (2026-10-16; the user asked to implement translations and locale
  formatting; the proposals below are taken and marked review)
- **Crates:** `tantu-i18n` (new: `Locale`, `Translations`, `Localization`, `Args`),
  `tantu-widgets` (`Localizations`, built-in widget strings), `tantu` (re-exports),
  `examples/gallery`
- **Plan item:** Phase 4, i18n, part 2 (Fluent-based localisation)
- **Related:** [ADR 0015](../../adr/0015-internationalization-crate.md),
  [formatting](formatting.md), [directionality](../widgets/directionality.md),
  [inherited values](../view/inherited.md), [theme](../theme/theme.md)

## Purpose

An enterprise app ships in many languages. Its strings live in translation files that
translators edit, with plurals ("1 file" / "3 files"), arguments ("Hello, {name}") and
gender or other selectors, and the user can switch the language while the app runs.
[Project Fluent](https://projectfluent.org) is designed for exactly this and PLAN.md
names it. This part adds locales, Fluent resources, a reactive inherited `Localization`
and translated built-in widget strings.

## Scope

In scope: `Locale` (BCP 47 identifiers, script direction); `Translations` (Fluent
resources per locale, a fallback locale, negotiation of a requested list); `Localization`
(the current locale as a signal, message lookup with arguments, reactive reads);
`Localizations` widget (provides the localization and the locale's direction); built-in
widget strings as message ids with English defaults; a gallery language switch.

Out of scope (and where it goes): number and date formatting (part 3,
[formatting](formatting.md)); reading the operating system's preferred languages (a later
platform amendment; apps pass a locale); loading resources from disk or watching them for
changes (apps read files and pass strings; hot reload is Phase 5); Fluent message
attributes beyond `.attr` lookup; bidi isolation marks inside formatted messages (off, as
our paragraphs already handle bidi).

## Public API

```rust
// tantu-i18n
/// A BCP 47 locale identifier such as `en`, `de-CH` or `ar-EG`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locale { /* private */ }
impl Locale {
    /// Parses an identifier; `None` if it isn't one.
    pub fn parse(text: &str) -> Option<Locale>;
    /// The language subtag (`"de"` for `de-CH`).
    pub fn language(&self) -> &str;
    /// `Rtl` for right-to-left scripts (Arabic, Hebrew, Persian, Urdu, …), else `Ltr`.
    pub fn direction(&self) -> TextDirection;
}
/// Writes the canonical form (`de-CH`).
impl std::fmt::Display for Locale;
impl std::str::FromStr for Locale;

/// A problem in a Fluent resource (syntax error or duplicate id), with its byte offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceError { pub message: String, pub offset: usize }

/// Fluent resources for a set of locales.
#[derive(Default)]
pub struct Translations { /* private */ }
impl Translations {
    pub fn new(fallback: Locale) -> Self;
    /// Adds a resource (Fluent syntax) for `locale`; a later resource's messages replace
    /// earlier ones with the same id. Errors are returned and the valid messages kept.
    pub fn add(&mut self, locale: Locale, source: &str) -> Result<(), Vec<ResourceError>>;
    /// The locales with resources, the fallback included.
    pub fn locales(&self) -> Vec<Locale>;
    /// The best available locale for `requested` (in preference order), else the fallback.
    pub fn negotiate(&self, requested: &[Locale]) -> Locale;
}

/// Message arguments.
#[derive(Clone, Debug, Default)]
pub struct Args { /* private */ }
impl Args {
    pub fn new() -> Self;
    pub fn set(self, name: &str, value: impl Into<ArgValue>) -> Self;
}
/// A string or number argument.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue { Text(String), Number(f64) }
// From<&str>, From<String>, From<i32/i64/u32/u64/usize/f32/f64> for ArgValue

/// The current locale and the translations (`Copy`; provided by `Localizations`).
#[derive(Clone, Copy)]
pub struct Localization { /* private */ }
impl Localization {
    pub fn new(translations: Translations, locale: Locale) -> Self;
    /// The current locale (a tracked read).
    pub fn locale(&self) -> Locale;
    /// Switches the locale (negotiated against the available ones).
    pub fn set_locale(&self, locale: Locale);
    /// The message `id` in the current locale (a tracked read).
    pub fn text(&self, id: &str) -> String;
    /// The message with arguments.
    pub fn text_with(&self, id: &str, args: &Args) -> String;
    /// The message, or `None` when no locale in the chain has it (a tracked read).
    pub fn message(&self, id: &str, args: Option<&Args>) -> Option<String>;
    /// The nearest localization above the element being built.
    pub fn of(cx: &BuildCx<'_>) -> Option<Localization>;
}

// tantu-widgets
/// Provides a localization and its locale's text direction to `child` (Flutter's
/// `Localizations`).
pub struct Localizations { /* private */ }
impl Localizations {
    pub fn new(localization: Localization, child: impl View) -> Self;
}
```

## Behavior

- **I18N-LOCALE-01:** `Locale::parse` accepts BCP 47 language tags (case-insensitive,
  `_` accepted as `-`) and canonicalizes them (`EN_us` → `en-US`); anything else is
  `None`. `direction` is `Rtl` for the languages `ar`, `arc`, `ckb`, `dv`, `fa`, `he`,
  `ks`, `ps`, `sd`, `syr`, `ug`, `ur` and `yi`, and for any locale whose script subtag is
  `Arab`, `Hebr`, `Syrc`, `Thaa` or `Nkoo`; else `Ltr`.
- **I18N-MSG-01:** `Translations::add` parses Fluent syntax; syntax errors and duplicate
  ids in one resource are returned with byte offsets, and the valid messages are kept.
  Messages added later for the same locale and id replace earlier ones.
- **I18N-MSG-02:** Lookup tries the current locale, then its parents (`de-CH` → `de`),
  then the fallback locale. A message missing everywhere returns its id (and logs a
  `tracing` warning once per id and locale); a missing argument formats as `{$name}`.
- **I18N-MSG-03:** Arguments: text values are inserted as is; numbers select plural
  variants by the locale's plural rules (`[one]`, `[other]`, …) and are formatted with the
  locale's number format (part 3) when placed in text. Fluent's bidi isolation marks are
  off.
- **I18N-MSG-04:** `Localization::text` and `text_with` read the locale signal, so a
  `Text` built from them updates when `set_locale` runs; `set_locale` stores the
  negotiated locale (an unavailable `fr-CA` becomes `fr` when only `fr` exists, or the
  fallback).
- **I18N-MSG-06:** `message` follows the same chain as `text` but returns `None` for a
  missing message (no warning), so a caller can supply its own default; the built-in
  widget strings use it (found while implementing, review).
- **I18N-MSG-05:** `Translations::negotiate` follows Fluent's filtering negotiation:
  exact matches, then language-and-region, then language, in the order requested; the
  fallback if nothing matches.
- **WIDGETS-I18N-01:** `Localizations` provides its `Localization` to its subtree (overlays
  included, ADR 0014) and a `Directionality` following the locale's direction; a
  `Directionality` inside it still wins below that point.
- **WIDGETS-I18N-02:** Built-in widget strings are messages with English defaults compiled
  into `tantu-widgets`, read through the nearest `Localization` (English without one), so
  an app overrides them by adding the same ids to its resources; the ids and English texts
  are the table below. Month and weekday names come from the locale's data instead (part 3).
  The strings follow a locale switch; to make that possible `IconButton::semantics_label`
  (and `Icon::semantics_label`) accept a reactive label (`impl IntoProp<String>`, so
  existing `&str` and `String` callers are unchanged). Placeholders and default accessible
  names of the date and time fields are read when the field is built (found while
  implementing, review).
- **GALLERY-50:** The gallery's header gains a "Language" combo box (English, Deutsch,
  Français, עברית); the header title, the sidebar section names and the status bar text
  come from the gallery's own Fluent resources (embedded strings), and choosing Hebrew
  turns the window right to left. Test: choosing Deutsch shows "Schaltflächen" for
  Buttons; choosing עברית mirrors the sidebar.
- **GALLERY-51:** An "i18n" kitchen-sink card: a message with a count argument and plural
  variants ("You have 1 new message" / "You have 5 new messages") and a stepper, echoed by
  `sink-i18n-state`, following the gallery's language.

## Built-in widget strings (English defaults)

| Id | English | Used by |
|---|---|---|
| `tantu-combo-no-matches` | No matches | `ComboBox`: the empty list's row |
| `tantu-date-label` | Date | `DatePickerField`: default accessible name |
| `tantu-date-placeholder` | YYYY-MM-DD | `DatePickerField`: placeholder (the typed format stays ISO) |
| `tantu-date-invalid` | Invalid date | `DatePickerField`: error description |
| `tantu-date-choose` | Choose date | `DatePickerField`: calendar button |
| `tantu-date-previous-month` | Previous month | `CalendarDatePicker`: header button |
| `tantu-date-next-month` | Next month | `CalendarDatePicker`: header button |
| `tantu-time-label` | Time | `TimePickerField`: default accessible name |
| `tantu-time-placeholder` | HH:MM | `TimePickerField`: placeholder |
| `tantu-time-invalid` | Invalid time | `TimePickerField`: error description |
| `tantu-split-resize` | Resize { $pane } | `SplitView` divider with a labelled pane |
| `tantu-split-resize-pane` | Resize pane { $number } | `SplitView` divider without a label |
| `tantu-dock-close` | Close { $title } | `DockArea` tab close button |
| `tantu-dock-resize` | Resize { $title } | `DockArea` divider (`Resize` alone without a panel) |

## Performance and allocation

Lookups format into a new `String` (Fluent's model); a `Text` re-formats only when the
locale or its own inputs change. Bundles are built once per locale and kept.

## Open questions

Decided on the user's request to implement (2026-10-16); the proposals are taken, for review:

1. **The crate split and dependency edges in ADR 0015** (new `tantu-i18n`, `tantu-widgets`
   → `i18n`, `Date`/`Time`/`Weekday` moving into it). Proposal: accept.
2. **`fluent-bundle` 0.16 + `fluent-langneg`.** Proposal: accept (the reference
   implementation; there is no smaller maintained alternative).
3. **Message lookup API:** `loc.text("id")` / `text_with("id", &Args)` plus reactive use
   through closures, rather than a `tr!` macro. Proposal: methods now; a macro could be
   sugar later (as with `view!{}`).
4. **System locale:** apps pass the locale for now; reading the OS's preferred languages
   (the `sys-locale` crate in the winit shell, through the `Platform` trait) is a small
   follow-up. Proposal: follow-up, not part of this item.
5. **Gallery languages:** English, German, French, Hebrew (Hebrew because the test fonts
   include Noto Sans Hebrew, so the RTL goldens render real glyphs). Proposal: accept.
