# Theme

- **Status:** Agreed (the user said "continue" on the draft, 2026-10-11, taking its five
  proposals; additions made while agreeing are marked *review* in Open questions)
- **Crates:** `tantu-theme` (tokens and the default themes), `tantu-widgets` (reading them),
  `tantu` (providing the default), `tantu-test` (the same for tests), `examples/gallery` (the
  live switch)
- **Plan item:** Phase 3, "`tantu-theme`: design tokens, light/dark, density, live theme
  switching"
- **Related:** [ADR 0012](../../adr/0012-theme-tokens-through-inherited-signals.md),
  [inherited values](../view/inherited.md), every widget spec (the constants they list are
  the light theme's values), Flutter's `ThemeData`, `ColorScheme`, `Theme.of`

## Purpose

Every widget paints with Material 3 constants today. A theme turns those constants into
tokens an app can set once, switch at run time (light/dark), and tune (density). Widgets read
the tokens through inherited values: the runner (or the app, with `ThemeProvider`) provides a
`Signal<Theme>`, and each widget reads it in the reactive closures already driving its paint,
so switching the theme repaints exactly what reads it. With no theme provided, the built-in
light theme applies, which is today's look, so nothing changes for existing apps and tests.

## Scope

In scope:

- `tantu-theme`: `Theme`, `ColorScheme`, `TextTheme`, `Shape`, `Density`; `Theme::light()`,
  `Theme::dark()`; `Theme::of(cx)`, `Theme::signal(cx)`; `ThemeProvider`.
- The facade providing a `Signal<Theme>` (light) at the root of every window when the
  `default-theme` feature is on, and painting the window background in the theme's surface
  color unless `Window::background` was set.
- `WidgetTester` providing the same signal, reachable as `tester.theme()`.
- Every widget reading its colors, radii, text presets and minimum sizes from the theme.
- `Builder` in `tantu-view` (Flutter's `Builder`): a view built from a closure that gets the
  `BuildCx`, so app code can call `Theme::of(cx)` anywhere in a view tree.
- A dark-mode switch in the gallery, and a dark golden.

Out of scope: a Fluent theme (the structure allows it; later), per-widget theme overrides
(`ButtonTheme`), high-contrast and custom palettes beyond setting the fields, animated theme
transitions, reactive *text* presets (a widget's text style is fixed at build; colors and
sizes are reactive).

## Public API

```rust
// tantu-theme
/// Material 3 color roles (the subset the widgets use), as sRGB colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorScheme {
    pub primary: Color, pub on_primary: Color,
    pub surface: Color, pub on_surface: Color, pub on_surface_variant: Color,
    pub surface_container_highest: Color,
    pub outline: Color, pub outline_variant: Color,
    pub inverse_surface: Color, pub inverse_on_surface: Color,
    pub error: Color,
}

/// The three text presets the widgets use.
#[derive(Clone, Debug, PartialEq)]
pub struct TextTheme { pub body: TextStyle, pub title: TextStyle, pub label: TextStyle }

/// Corner radii (Material 3's shape scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape { pub extra_small: f32, pub small: f32, pub medium: f32, pub large: f32 }

/// How tight the controls are: 0 is Material's default, negative is denser (−1, −2, −3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Density(pub i8);

impl Density {
    /// Pixels added to a control's minimum extent: 4 per step, the step clamped to −3..=3.
    pub fn offset(self) -> f32;
    /// `extent` plus the offset, never below 0.
    pub fn apply(self, extent: f32) -> f32;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub colors: ColorScheme,
    pub text: TextTheme,
    pub shape: Shape,
    pub density: Density,
}

impl Theme {
    /// Material 3's baseline light scheme: the widgets' current constants.
    pub fn light() -> Self;
    /// Material 3's baseline dark scheme.
    pub fn dark() -> Self;
    /// The theme in effect while building: the provided signal's value (a tracked read), or
    /// `light()`.
    pub fn of(cx: &BuildCx<'_>) -> Theme;
    /// The provided signal, if any (for reactive closures, and for apps switching the theme).
    pub fn signal(cx: &BuildCx<'_>) -> Option<Signal<Theme>>;
}

/// Provides `theme` to its child (sugar over `Provide<Signal<Theme>, V>`).
pub struct ThemeProvider<V> { /* theme: Signal<Theme>, child */ }
impl<V: View> ThemeProvider<V> { pub fn new(theme: Signal<Theme>, child: V) -> Self; }

// tantu-view
/// Builds the view `f` returns, giving `f` the build context (Flutter's `Builder`).
pub struct Builder<F> { /* f */ }
impl<V: View, F: FnOnce(&mut BuildCx<'_>) -> V> Builder<F> { pub fn new(f: F) -> Self; }

// tantu (facade)
impl Window {
    /// The window's background color; the default follows the theme's `surface`.
    pub fn background(self, color: Color) -> Self;   // unchanged signature, new default
}

// tantu-test
impl WidgetTester {
    /// The theme signal the tester provides at the root (initially `Theme::light()`).
    pub fn theme(&self) -> Signal<Theme>;
}
```

Light values (`Theme::light()`), all today's constants:

| Role | Value | Used by |
|---|---|---|
| `primary` | `0xFF6750A4` | button fill and focus outline, selected toggles, slider active track and thumb, text field focused outline, caret, selection (40 %), composing underline |
| `on_primary` | `0xFFFFFFFF` | button label, check mark, selected switch thumb, slider active ticks (38 %), button state layers |
| `surface` | `0xFFFFFFFF` | the window background (Material's `0xFFFFFBFE` rounded to white, today's `Window::background` default) |
| `on_surface` | `0xFF1C1B1F` | `Text`, `RichText`, `Icon` default color; unselected state layers; disabled fills at 12 % and 38 %; scrollbar thumb at 40 % |
| `on_surface_variant` | `0xFF49454F` | unselected checkbox and radio outlines, icon button icon, placeholder, inactive slider ticks (38 %) |
| `surface_container_highest` | `0xFFE6E0E9` | unselected switch track, inactive slider track |
| `outline` | `0xFF79747E` | text field outline, unselected switch outline and thumb |
| `outline_variant` | `0xFFCAC4D0` | dividers |
| `inverse_surface` | `0xFF313033` | tooltip background |
| `inverse_on_surface` | `0xFFF4EFF4` | tooltip text |
| `error` | `0xFFB3261E` | reserved (forms, Phase 4) |

Dark values (`Theme::dark()`), Material 3's baseline dark scheme: `primary 0xFFD0BCFF`,
`on_primary 0xFF381E72`, `surface 0xFF141218`, `on_surface 0xFFE6E0E9`, `on_surface_variant
0xFFCAC4D0`, `surface_container_highest 0xFF36343B`, `outline 0xFF938F99`, `outline_variant
0xFF49454F`, `inverse_surface 0xFFE6E0E9`, `inverse_on_surface 0xFF322F35`, `error
0xFFF2B8B5`.

Text presets: `body = TextStyle::body()`, `title = TextStyle::title()`, `label =
TextStyle::label()` in both. Shape: `extra_small 4`, `small 8`, `medium 12`, `large 16`
(text field and tooltip use `extra_small`; pill buttons and the checkbox's 2 px box are
geometry, not shape tokens). Density: `Density(0)`.

## Behavior

### `tantu-theme`

- **THEME-01:** `Theme::light()` has the light values and `Theme::dark()` the dark values of
  the tables above, both with the body/title/label presets of `TextStyle`, the shape scale and
  `Density(0)`; the two differ only in `colors`.
- **THEME-02:** `Theme::of(cx)` is the provided signal's value, else `Theme::light()`;
  `Theme::signal(cx)` is the provided signal or `None`. `ThemeProvider::new(signal, child)`
  provides `signal` to `child` and everything under it (a nested provider shadows an outer
  one, as VIEW-INHERIT-01).
- **THEME-03:** `Density::offset` is 4 px per step with the step clamped to −3..=3
  (`Density(-1)` → −4, `Density(-5)` → −12, `Density(2)` → 8); `Density::apply(extent)` adds
  the offset and never returns less than 0 (`Density(-3).apply(10.0)` is 0).

### `tantu-widgets`

- **WIDGETS-THEME-01:** Every widget takes its colors from the theme in effect where it is
  built, by the roles in the table above: with `Theme::dark()` provided, a button fills with
  the dark `primary` and labels in the dark `on_primary`, `Text` paints in the dark
  `on_surface`, a divider in the dark `outline_variant`, a tooltip in the dark
  `inverse_surface`, the unselected switch track in the dark `surface_container_highest`,
  and so on for `IconButton`, `Checkbox`, `Radio`, `Slider`, `TextField`, `RichText`, `Icon`
  and `ScrollView`. Radii come from `shape` (`TextField` and `Tooltip`: `extra_small`). With
  the light theme provided or none, every existing golden is unchanged.
- **WIDGETS-THEME-02:** Setting a provided signal to another theme repaints the widgets below
  the provider in its colors at the next frame, widgets outside the provider keep theirs, and
  a widget's own explicit color (e.g. `Text::color`) is not overridden.
- **WIDGETS-THEME-03:** Density changes the minimum extents of controls through
  `Density::apply`, reactively: at `Density(-1)` a `Button` is at least 36 px tall (64 wide),
  a `Checkbox`, `Radio` and `IconButton` element is 36 × 36, a `Switch` 56 × 36 and a
  `TextField` at least 36 px tall; at `Density(-2)` 32, 32 × 32, 52 × 32 and 32. A button's
  or field's content can still make it taller than the minimum (a 14 px label with 10 px of
  padding above and below is 36 px). The slider and the layout widgets are unaffected.
- **WIDGETS-THEME-04:** `Text` defaults to the theme's `text.body`, `Button` labels use
  `text.label`, `TextField` uses `text.body` at 16 px and `Tooltip` `text.body` at 12 px, read
  from the theme in effect at build time (not reactive); a theme whose `body` is 20 px makes a
  default `Text` 20 px.

### Facade, tester, view and gallery

- **FACADE-APP-13:** With the `default-theme` feature, the runner provides a `Signal<Theme>`
  holding `Theme::light()` at the root of every window (so `Theme::signal(cx)` is `Some` in
  app code and setting it switches the whole window), and the window background follows the
  theme's `surface` unless `Window::background` was set. Without the feature nothing is
  provided and the background is `Window::background` (default white).
- **TEST-WT-12:** `WidgetTester` provides a `Signal<Theme>` holding `Theme::light()` around
  the app view; `tester.theme()` returns it, so a test can switch the theme and pump.
  `matches_golden` renders the frame over the theme's `surface` (white in the light theme,
  where goldens were transparent before), so a golden shows what a window shows.
- **VIEW-TREE-12:** `Builder::new(f)` builds the view `f(cx)` returns in place (same parent,
  same scope), so `f` can read inherited values and the text styles.
- **GALLERY-05:** The header holds a "Dark" `Switch` (key `dark`) that sets the provided theme
  signal to `Theme::dark()` or `Theme::light()`; after tapping it the press button fills with
  the dark primary and the background is the dark surface. Golden
  `tests/goldens/gallery-dark.png` (the Buttons section, dark).

## Performance and allocation

Each widget binds one or two reactive props to the theme signal (colors as a `Copy`
`ColorScheme`, density as a `Density`), so a theme switch costs one paint update per widget
plus a layout update for the controls whose size depends on density. No per-frame work.

## Open questions

Taken from the draft when the user said "continue":

1. **Dependency table change:** `tantu-widgets | view, theme`, `tantu-theme | view`
   (ADR 0012). Tokens stay in the theme crate, as AGENTS.md says.
2. **A signal as the theme's carrier** (`Signal<Theme>` through inherited values).
3. **The light theme provided by the facade and the tester by default.**
4. **Density as a small integer** (0, −1, −2, …) rather than Flutter's two-axis
   `VisualDensity`.
5. **Migration in one spec and one triple** for all widgets.

Added while agreeing (*review*):

- `tantu-test | view, theme, render-soft, render-headless`: the tester provides the signal
  (proposal 3) and exposes it, so tests of theme switching need no provider of their own.
- `Shape` has Material's four steps (`extra_small` to `large`) instead of `small/medium/full`;
  pill shapes are geometry (half the height), not a token.
- `Density::offset`/`apply` as public helpers, clamped to ±3 (Flutter allows ±4), so the
  minimum extents never go below 0.
- The light `surface` is white, not Material's `0xFFFFFBFE`, so the window background and
  every golden stay as they are.
- `Builder` (VIEW-TREE-12): the smallest way for app code to reach a `BuildCx`.
- The facade keeps `tantu-theme` as a plain dependency (widgets need it anyway); the
  `default-theme` feature now only controls whether the runner provides the signal.
- Text presets are read at build time; making them reactive would re-key every paragraph.
