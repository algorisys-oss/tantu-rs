# Icon and IconButton

- **Status:** Agreed (decided by the agent while the user was away, 2026-10-11; review the
  Open questions)
- **Crates:** `tantu-widgets` (plus `TextStyles::ensure_font` in `tantu-text` and
  `BuildCx::ensure_font` in `tantu-view`)
- **Plan item:** Phase 3, "Widgets" → "`Icon` and `IconButton` (an icon font)"
- **Related:** [shared text styles](../text/styles.md), [text system](../text/system.md),
  [text in views](../view/text.md), [toggles](toggles.md) (the shared pressable core),
  [Text and Button](basic.md), Flutter's `Icon`, `Icons` and `IconButton`

## Purpose

Icons are glyphs of an icon font. `Icon` draws one at a size and color, and `IconButton` is
the Material icon button: a 40 px circle target with an icon, a state layer and `on_press`.
Flutter ships Material Icons and names every glyph (`Icons.check`); Tantu does the same with
the Material Icons Regular font (Apache 2.0) behind the default feature `material-icons`.

A widget that uses a font must be able to bring it. The text system registers fonts, but a
widget only sees the shared style table, so that table gains a queue: `BuildCx::ensure_font`
queues font data, and the text system registers whatever is queued before it next measures
or paints. Apps and `WidgetTester` need no setup, and theme packs can bring fonts the same
way.

## Scope

In scope:

- `IconData` (a code point, a family and optionally the font to register), `Icons` (one
  constant per Material Icons glyph), `MATERIAL_ICONS_FONT`.
- `Icon::new(icon)`, `.size(..)`, `.color(..)`.
- `IconButton::new(icon)`, `.on_press(..)`, `.enabled(..)`.
- `TextStyles::ensure_font`, `TextStyles::take_pending_fonts`, `BuildCx::ensure_font`.

Out of scope (and where it goes):

- Material Symbols (the variable successor, several MB) and icon weights/fills: later, with
  the theme, if wanted.
- Filled/tonal/outlined icon button variants, toggled icon buttons, tooltips: `tantu-theme`
  (variants), compose with `Tooltip` for now.
- Regenerating `icons.rs` with a tool (`cargo xtask icons`): dev tooling, when the font is
  next updated. The module says how it was generated.
- Accessibility (semantic labels): `tantu-a11y`.

## Public API

```rust
// tantu-widgets
/// A glyph of an icon font.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IconData {
    /// The glyph's code point.
    pub code_point: char,
    /// The font family holding it.
    pub family: &'static str,
    /// The font file to register before use, if the widget must bring it.
    pub font: Option<&'static [u8]>,
}

impl IconData {
    pub const fn new(code_point: char, family: &'static str) -> Self;
    pub const fn with_font(self, font: &'static [u8]) -> Self;
}
impl IntoProp<IconData> for IconData {}

/// Material Icons Regular, as a TTF (feature `material-icons`).
pub const MATERIAL_ICONS_FONT: &[u8];
/// Its family name.
pub const MATERIAL_ICONS_FAMILY: &str = "Material Icons";

/// Every Material Icons glyph, named as in the font's codepoints file in upper snake case
/// (`Icons::CHECK`, `Icons::ARROW_BACK`; names starting with a digit get an `N`, as in
/// `Icons::N3D_ROTATION`). Generated from `fonts/material-icons/MaterialIcons-Regular.codepoints`.
pub struct Icons;
impl Icons { pub const CHECK: IconData = /* '\u{e5ca}' */; /* … */ }

/// Draws an icon glyph (Flutter's `Icon`).
pub struct Icon { /* icon, size, color */ }
impl Icon {
    /// `icon` at 24 px in on-surface `0xFF1C1B1F`.
    pub fn new(icon: impl IntoProp<IconData>) -> Self;
    /// The icon's size, a square (default 24; not reactive, like `Text::style`).
    pub fn size(self, size: f32) -> Self;
    pub fn color(self, color: impl IntoProp<Color>) -> Self;
}

/// A pressable icon (Flutter's `IconButton`, Material 3 standard icon button).
pub struct IconButton { /* icon, on_press, enabled */ }
impl IconButton {
    /// An icon button, enabled, reporting nothing (so drawn disabled until `on_press` is set).
    pub fn new(icon: impl IntoProp<IconData>) -> Self;
    pub fn on_press(self, f: impl Fn() + 'static) -> Self;
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
}

// tantu-text
impl TextStyles {
    /// Queues `font` for registration by the text system before its next measure or paint.
    /// The same data (by address) is queued once per table.
    pub fn ensure_font(&self, font: &'static [u8]);
    /// Takes the queued fonts (what the text system calls).
    pub fn take_pending_fonts(&self) -> Vec<&'static [u8]>;
}

// tantu-view
impl BuildCx<'_> {
    /// `TextStyles::ensure_font` on the tree's table.
    pub fn ensure_font(&mut self, font: &'static [u8]);
}
```

`Icon` is one render element with a `RenderConstrainedBox` tight at `size × size` and an
`IconPaint` that paints the code point as a one-line paragraph in `TextStyle { family:
Named(icon.family), size, line_height: Some(1.0), .. }`, at the element's origin, in `color`.
Material Icons glyphs are square and advance one em, so the glyph fills the box.

`IconButton` is built on the toggles' pressable core (WIDGETS-TOGGLE-05..08: press and
release inside, Space/Enter, hover/focus/pressed state layer in on-surface, disabled rules):
a 40 × 40 element with a centered 24 px `Icon` in on-surface-variant `0xFF49454F` (disabled:
on-surface 38 % `0x611C1B1F`), `CursorIcon::Pointer`.

## Behavior

- **TEXT-STYLES-05:** `ensure_font` queues the data once per table (a second call with the
  same slice does nothing); `take_pending_fonts` returns and clears the queue. A
  `TextSystem` registers every queued font of its table (as `register_font`) before it
  measures, paints or computes intrinsics, so a family a queued font provides resolves by
  name from the next measurement on. (Added to [shared text styles](../text/styles.md).)
- **VIEW-STYLES-02:** `BuildCx::ensure_font` queues on the tree's table. (Added to the same
  spec.)
- **WIDGETS-ICON-01:** `Icons` has one constant per line of the codepoints file (the first of
  two lines with the same name wins), with that line's code point, the Material Icons
  family and `MATERIAL_ICONS_FONT`. `IconData::new` has no font; `with_font` sets one.
- **WIDGETS-ICON-02:** An `Icon` is `size × size` and paints one glyph run: font size `size`,
  the `color`, one glyph that isn't the missing glyph (id 0), from the icon's font, which the
  widget ensured. Checked against a golden (`crates/tantu-widgets/tests/goldens/icon.png`).
- **WIDGETS-ICON-03:** `icon` and `color` are reactive; a change repaints the icon. The
  family and its font come from the first `icon` value (the style key is made at build
  time, like `size`); a later value contributes its code point only.
- **WIDGETS-ICON-04:** An `IconButton` is 40 × 40 with the icon centered; a primary press
  released inside, or Space/Enter while focused (no repeats), calls `on_press`; it has the
  pressable core's state layer and focus rules. Disabled (`enabled(false)` or no `on_press`)
  draws the icon in on-surface 38 %, has no layer, and ignores input. Golden
  `icon-button.png`.

## Performance and allocation

An icon paints one glyph run (the text system's layout cache keyed by text and style serves
every icon of the same glyph). `ensure_font` is a pointer comparison once the font is queued.

## Open questions

Decided by the agent while the user was away (review):

1. **Material Icons Regular (357 KB TTF, Apache 2.0) embedded in `tantu-widgets`** behind a
   default feature, with a generated constant table (2 234 names) rather than a subset.
   The facade forwards the feature. Material Symbols can replace it later.
2. **Fonts queued through `TextStyles`** so a widget (or a theme) registers its own font with
   no app or tester setup. The alternative, `App` and `WidgetTester` registering the icon font
   themselves, needs `tantu-test` to reach the font (a dependency on widgets, or a copy).
3. **Digit-leading names get an `N` prefix** (`N3D_ROTATION`, `N360`); Flutter spells them out
   (`threed_rotation`), which can't be generated.
4. **The icon button uses the toggles' core** (so an `IconButton` without `on_press` is
   disabled, as Flutter's), including the on-surface state layer; Material's icon button
   uses on-surface-variant for it, a detail for the theme.
5. **`Icon::size` is not reactive** (a style key is made at build time, like `Text::style`).
