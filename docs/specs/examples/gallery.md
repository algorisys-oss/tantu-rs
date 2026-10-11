# Gallery

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-12; review
  the Open questions)
- **Crate:** `examples/gallery`
- **Plan item:** Phase 3, "`examples/gallery`"
- **Related:** every widget spec in `docs/specs/widgets/`, [layout demo](layout-demo.md),
  [WidgetTester](../test/widget-tester.md)

## Purpose

One app that shows every widget Tantu has, section by section, so a reader of the README can
run `cargo run -p gallery` and see what they get, and so every widget has the gallery entry
AGENTS.md asks for. It is also the first app composing scrolling, lists, text input and
toggles together, which is where integration bugs show.

## Scope

In scope: `examples/gallery` with `src/lib.rs` (`gallery() -> impl View`) and `src/main.rs`,
depending only on `tantu`; tests with `WidgetTester` and goldens at 960 × 640.

Out of scope: theming (fixed Material 3 look), a search box, source snippets per sample
(later, with the docs site).

## Public API

```rust
// examples/gallery/src/lib.rs
/// The gallery's view: a header, a sidebar of sections and the selected section's samples.
pub fn gallery() -> impl View;
/// The section names, in sidebar order.
pub const SECTIONS: [&str; 6] = ["Buttons", "Text", "Selection", "Text field", "Lists", "Media"];
```

Structure: a `Column` with a header `Row` (key `header`: the title "Tantu gallery" and, after a
`Spacer`, the selected section's name, key `current`), and an `Expanded` body `Row`: a 200 px
sidebar (key `sidebar`, a `Column` of one `Button` per section, keys `nav-<section>` with the
name lower-cased and spaces as dashes) and an `Expanded` content area (key `content`) showing
the selected section through a `Dyn`, each section starting with its name as a title `Text`
(key `heading`).

Sections and their samples:

| Section | Samples (keys) |
|---|---|
| Buttons | a `Button` counting presses into "Pressed N times" (`press`, `press-count`), a disabled `Button`, an `IconButton` (`icon-button`), a `Button` with a `Tooltip` |
| Text | `Text` in the body, title and label styles, a `RichText` with a bold red span, `Divider`s |
| Selection | a `Checkbox` (`checkbox`) with "Checkbox: on/off" (`checkbox-state`), a `Switch` (`switch`) with "Switch: on/off" (`switch-state`), three `Radio`s (`radio-a/b/c`) with "Radio: a" (`radio-state`), a `Slider` over 0..100 (`slider`) with "Slider: 50" (`slider-state`) |
| Text field | a `TextField` (`field`) with a placeholder and "You typed: …" (`echo`), a multi-line `TextField` (`notes`) |
| Lists | a `ListView` of 10 000 rows of 32 px ("Row N") in a 320 px tall box (`list`) |
| Media | a row of `Icon`s, an `Image` of a generated 64 × 64 gradient shown at 128 × 128 (`image`) |

## Behavior

- **GALLERY-01:** At 960 × 640 the header spans the width, the sidebar is 200 px, the content
  starts at x = 200 and fills the rest, and the "Buttons" section shows first (its heading and
  the header's current name read "Buttons"). Tapping a sidebar button shows that section: the
  heading and the current name change, and the previous section's samples are gone.
- **GALLERY-02:** The samples react: pressing the press button increments "Pressed N times";
  tapping the checkbox, switch and radios updates their state texts; dragging the slider
  updates "Slider: N".
- **GALLERY-03:** Typing into the text field (`enter_text`) updates "You typed: …"; the list
  shows "Row 0" first and, after scrolling, no longer shows it but shows a later row.
- **GALLERY-04:** Goldens at 960 × 640 of the Buttons section (`tests/goldens/gallery-buttons.png`)
  and the Selection section (`gallery-selection.png`).

## Open questions

Decided by the agent while the user was away (review): six sections keyed by name; samples
report their state as text so tests and readers see it; the image is generated in code (no
asset file).
