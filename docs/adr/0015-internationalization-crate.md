# 0015. Internationalization in a `tantu-i18n` crate over Fluent and ICU4X

- **Status:** Accepted (2026-10-16; the user asked to implement translations and locale
  formatting; drafted and taken by the agent, review)
- **Date:** 2026-10-16
- **Related:** [ADR 0005](0005-text-stack-parley-swash-fontique.md), [ADR 0012](0012-theme-tokens-through-inherited-signals.md),
  [localization spec](../specs/i18n/localization.md), [formatting spec](../specs/i18n/formatting.md),
  [directionality](../specs/widgets/directionality.md)

## Context

PLAN.md's i18n item asks for Fluent-based localisation and locale-aware formatting (RTL
mirroring is done). Apps need translated strings with plurals and arguments, numbers and
dates in the user's conventions, and the built-in widgets need their own strings ("No
matches", "Invalid date", month names) translated. None of this belongs in the layout or
text crates, and the widgets must be able to use it, so it can't live in `tantu-widgets`
or above it.

ICU4X 2.3 is already in the dependency graph (parley uses its locale, segmentation and
property crates), so its formatting crates share most of their code with what we ship.

## Decision

1. **A new crate `tantu-i18n`**, depending on `tantu-view` (for the inherited locale and
   reactive reads) and `tantu-core`. `tantu-widgets` gains an edge to it, and the facade
   re-exports it. AGENTS.md table changes:
   - new row: `tantu-i18n | locales, Fluent messages, number and date formatting | view`
   - `tantu-widgets | view, theme, i18n`
2. **Messages: `fluent-bundle` 0.16** (Project Fluent's Rust implementation, used by
   Firefox's Rust code and Bevy/egui localization crates), with `fluent-langneg` for
   locale negotiation. Fluent's number formatting inside messages is routed to ICU4X.
3. **Formatting: ICU4X 2.3** (`icu_locale_core`, `icu_decimal`, `icu_datetime`,
   `icu_calendar` for week information) with compiled data, the same major version as the
   copy parley already pulls in.
4. **The locale is reactive and inherited**, like the theme (ADR 0012): a `Localization`
   handle carries a `Signal<Locale>` and the loaded translations; `Localizations` (a widget,
   Flutter's name) provides it and a `Directionality` derived from the locale.
5. **`Date`, `Time` and `Weekday` move from `tantu-widgets` to `tantu-i18n`** (re-exported
   from `tantu-widgets` unchanged), so formatting can take them without a cycle.

## Consequences

- Translated apps write `Text::new(move || loc.text("greeting"))`; changing the locale
  re-renders every reader, flips the direction for RTL languages and re-formats numbers and
  dates.
- Built-in widget strings become message ids with English defaults compiled in; apps can
  override them per locale in their own Fluent resources.
- Binary size: ICU4X compiled data for decimals and date patterns covers every locale. The
  formatting spec records the measured growth; a later feature can trim it to selected
  locales (ICU4X's data generator) if it matters for small utilities.
- Dependency count grows by `fluent-bundle`, `fluent-syntax`, `fluent-langneg`,
  `intl-memoizer`, `intl_pluralrules`, `unic-langid` and the ICU4X formatting crates
  (licenses: Apache-2.0/MIT for Fluent, Unicode-3.0 for ICU4X, as already accepted for
  parley's ICU4X crates).

## Alternatives considered

- **Formatting inside `tantu-widgets`:** the widgets would own locale state and apps would
  depend on the widget crate for plain formatting; rejected for layering.
- **In-house number and date formatting:** small, but wrong for most locales (grouping in
  India, non-Gregorian defaults, 12/24-hour conventions, translated month names); rejected.
- **`chrono` + `num-format`:** two partial solutions with their own locale models and no
  plural rules; ICU4X is already present and complete.
- **Fluent's own number formatting only:** fluent-bundle formats numbers without locale
  grouping; we plug ICU4X in instead.
