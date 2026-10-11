# Theme

- **Status:** Draft (for review; proposals in Open questions, one of which changes the
  AGENTS.md dependency table)
- **Crates:** `tantu-theme` (tokens and the default themes), `tantu-widgets` (reading them),
  `tantu` (providing the default)
- **Plan item:** Phase 3, "`tantu-theme`: design tokens, light/dark, density, live theme
  switching"
- **Related:** [inherited values](../view/inherited.md), every widget spec (the constants they
  fix today), Flutter's `ThemeData`, `ColorScheme`, `Theme.of`

## Purpose

Every widget paints with Material 3 constants today. A theme turns those constants into
tokens an app can set once, switch at run time (light/dark), and tune (density). Widgets read
the tokens through inherited values: an app provides a `Signal<Theme>` with `Provide`, and
each widget reads it in its reactive closures, so switching the theme repaints exactly what
reads it. With no theme provided, the built-in light theme applies, which is today's look, so
nothing changes for existing apps and tests.

## Scope

In scope:

- `tantu-theme`: `Theme`, `ColorScheme`, `TextTheme`, `Shape`, `Density`; `Theme::light()`,
  `Theme::dark()`; `Theme::of(cx)`.
- `ThemeProvider` (sugar over `Provide<Signal<Theme>, _>`), and the facade providing the
  light theme at the root of every window when the `default-theme` feature is on.
- Every widget reading its colors, radii and text styles from the theme.

Out of scope: a Fluent theme (the structure allows it; later), per-widget theme overrides
(`ButtonTheme`), high-contrast and custom palettes beyond setting the fields, animated theme
transitions.

## Public API

```rust
// tantu-theme
/// Material 3 roles (the subset the widgets use), as sRGB colors.
#[derive(Clone, Debug, PartialEq)]
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

/// Corner radii.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape { pub small: f32, pub medium: f32, pub full: f32 }

/// How tight the controls are: 0 is Material's default, negative is denser (−1, −2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Density(pub i8);

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
    /// The theme in effect while building: the provided `Signal<Theme>` (a tracked read), or
    /// `light()`.
    pub fn of(cx: &BuildCx<'_>) -> Theme;
    /// The provided signal, if any (for reactive closures).
    pub fn signal(cx: &BuildCx<'_>) -> Option<Signal<Theme>>;
}

/// Provides `theme` to its child.
pub struct ThemeProvider<V> { /* theme: Signal<Theme>, child */ }
impl<V: View> ThemeProvider<V> { pub fn new(theme: Signal<Theme>, child: V) -> Self; }
```

Widgets: each widget's color constants become reads of the theme at build time
(`Theme::signal(cx)`), used inside the closures already driving their paints, with
`Theme::light()`'s values as the fallback. Density offsets the widgets' minimum sizes by
4 px per step (a density of −1 makes a 40 px button 36 px), as Material's `VisualDensity`.

## Behavior (draft)

- **THEME-01:** `Theme::light()` equals the widgets' current constants (the `0xFF6750A4`
  primary and the rest listed in the widget specs), `Theme::dark()` is Material 3's baseline
  dark scheme, and both have the body/title/label presets of `TextStyle`.
- **THEME-02:** `Theme::of(cx)` is the provided signal's value, else `light()`; `Theme::signal`
  is the provided signal or `None`. `ThemeProvider` provides.
- **THEME-03:** With a `ThemeProvider` holding a signal, setting the signal to `dark()`
  repaints the widgets below in the dark colors at the next frame (a button's fill becomes
  the dark primary), and widgets outside the provider keep the light colors.
- **THEME-04:** Density: at `Density(-1)` a button is 36 px tall and a toggle's target 36 px;
  at `Density(-2)` 32 px.
- **FACADE-APP-13:** With the `default-theme` feature, the runner wraps each window's content
  in a `ThemeProvider` holding `Theme::light()`, so apps can read the theme without providing
  one; `WidgetTester` does the same.
- **WIDGETS-THEME-01:** Every widget (`Button`, `IconButton`, `Checkbox`, `Radio`, `Switch`,
  `Slider`, `Divider`, `TextField`, `Tooltip`, `ScrollView`, `Text`, `RichText`, `Icon`) takes
  its colors and radii from the theme; with the light theme provided or none, every existing
  golden is unchanged.

## Open questions

Proposals (say "continue" to take them, or change them):

1. **Dependency table change.** `tantu-widgets` must read the tokens, so either
   `tantu-widgets` depends on `tantu-theme` (the proposal: AGENTS.md row `tantu-widgets |
   view, theme`, with `tantu-theme | view`), or the token types move into `tantu-widgets` and
   `tantu-theme` keeps only the default themes (then `tantu-theme | widgets`). The first keeps
   "tokens live in the theme crate" as AGENTS.md says. The architecture doc and diagram
   change with it.
2. **A signal as the theme's carrier** (`Signal<Theme>` through inherited values), not a
   theme-specific reactive mechanism.
3. **The light theme provided by the facade and the tester by default**, so widgets never
   need the fallback in real apps; the fallback stays for bare `ViewTree` use.
4. **Density as a small integer** (0, −1, −2) rather than Flutter's two-axis `VisualDensity`.
5. **Migration in one spec and one triple** for all widgets, since each change is mechanical
   (a constant becomes a closure) and the goldens pin the result.
