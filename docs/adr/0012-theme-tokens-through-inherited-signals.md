# 0012. Theme tokens reach widgets through inherited signals

- **Status:** Accepted (the user said "continue" on the proposal in the theme spec draft,
  2026-10-11)
- **Date:** 2026-10-11
- **Related:** [ADR 0001](0001-retained-tree-and-fine-grained-reactivity.md) (signals),
  [ADR 0011](0011-view-layer.md) (run-once components, reactive props),
  spec [theme](../specs/theme/theme.md), spec [inherited values](../specs/view/inherited.md)

## Context

AGENTS.md gives `tantu-theme` the design tokens and the default themes, and `tantu-widgets`
the standard widgets, with both depending only on `tantu-view`. The widgets must read the
tokens (colors, radii, text presets, density), so one of the two must know the other, or the
tokens must live somewhere both can see. Live switching (light to dark at run time) must
repaint every widget without rebuilding the tree, and a tree built with no theme at all
(tests, bare `ViewTree` use) must still look right.

`tantu-view` already has inherited values (`BuildCx::provide`/`inherit`, spec
`view/inherited.md`): a value attached to an element, found by walking up at build time, with
reactivity coming from providing a `Copy` signal handle.

## Decision

1. **`tantu-widgets` depends on `tantu-theme`.** The AGENTS.md table rows become
   `tantu-theme | view` and `tantu-widgets | view, theme`. Tokens stay in the theme crate, as
   AGENTS.md intends; the widgets are their consumers. `tantu-test` depends on `tantu-theme`
   too, so the tester can provide a theme signal and hand it to tests.
2. **The theme's carrier is a `Signal<Theme>` provided as an inherited value.** There is no
   theme-specific reactive mechanism. `Theme::signal(cx)` returns the provided handle;
   widgets read the fields they need inside the reactive closures already driving their
   paints and sizes, so a switch repaints (and, for density, re-lays out) exactly the widgets
   that read it. `Theme::of(cx)` is the build-time snapshot, with `Theme::light()` as the
   fallback when nothing is provided.
3. **The runner and the tester provide the light theme by default.** Apps switch the theme by
   setting the provided signal, and can scope another theme to a subtree with
   `ThemeProvider`. The window background follows the theme's `surface` unless the window set
   its own color.

## Consequences

- The dependency table and the architecture diagram change (`tantu-widgets → tantu-theme`,
  `tantu-test → tantu-theme`); the rule "nothing below `tantu-view` knows about widgets" is
  untouched, since `tantu-theme` sits beside the widgets, not below the view layer.
- Every widget's color constants become reads of the theme; the light theme reproduces them,
  so goldens and existing apps are unchanged.
- A theme switch costs one update per widget that reads the theme (its paint snapshot of the
  `ColorScheme`), which is the fine-grained cost ADR 0001 aims for.
- Text presets are read at build time (a `TextStyleKey` is baked into each paragraph);
  switching font sizes live would need re-keying paragraphs and is deferred.
- A Fluent theme, per-widget theme overrides and animated transitions can be added without
  changing this structure.

## Alternatives considered

- **Token types in `tantu-widgets`, defaults in `tantu-theme`** (`tantu-theme | widgets`).
  Keeps the widgets dependency-free but splits "what a theme is" from "the themes", and every
  non-default theme would import the widgets crate to name a color role. Rejected.
- **Tokens in `tantu-view`.** The view layer would carry Material-shaped color roles it never
  uses, against the dependency rule's spirit. Rejected.
- **A theme-specific reactive mechanism** (a theme listener per element, Flutter's
  `InheritedWidget` dependency tracking). Duplicates what signals already do. Rejected.
- **Rebuilding the tree on a theme change.** Loses focus, scroll and editing state, and is
  the whole-tree rebuild ADR 0001 rejects. Rejected.
