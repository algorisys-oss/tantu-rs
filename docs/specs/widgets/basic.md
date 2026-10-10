# Text and Button

- **Status:** Draft
- **Crate:** `tantu-widgets` (plus one field added to `tantu-view`'s `PointerCx`)
- **Plan item:** Phase 2, `tantu` facade → "`Text` and `Button`"
- **Related:** [shared text styles](../text/styles.md), [text in views](../view/text.md),
  [event dispatch](../view/events.md), [frame](../view/frame.md), [layout widgets](layout.md)

## Purpose

The two widgets the counter needs, and enough for a first layout demo. `Text` shows a reactive
string in a style and color. `Button` is a pressable label that calls `on_press` when a press is
released over it, and shows hover and pressed states. Both are Phase 2 versions: the look is
fixed until `tantu-theme` (Phase 3), and keyboard activation and accessibility come with focus
and `tantu-a11y`.

## Scope

In scope:

- `Text`: text (reactive), style, color (reactive), `max_lines`, `soft_wrap`.
- `Button`: label (reactive), `on_press`, `enabled` (reactive), hover and pressed visuals.
- `PointerCx::size`, the element's size from the last layout, so a handler can tell whether a
  release lands inside the element (amends [event dispatch](../view/events.md)).

Out of scope:

- `RichText` and text spans, selection: Phase 3 "Widgets".
- Button variants (filled, outlined, text, icon), theming and density: `tantu-theme`.
- Keyboard activation, focus ring and the accessibility node: Phase 3 focus and `tantu-a11y`.
- Cursor shapes: `Button` sets `CursorIcon::Pointer`, but the platform can't show cursors yet.

## Public API

```rust
// tantu-widgets
pub struct Text { /* text, style, color, max_lines, soft_wrap */ }

impl Text {
    /// Text showing `text` (a value, a closure or a signal), in `TextStyle::body()` and
    /// near-black (`Color::from_argb32(0xFF1C1B1F)`), wrapping at the available width.
    pub fn new(text: impl IntoProp<String>) -> Self;
    pub fn style(self, style: TextStyle) -> Self;
    pub fn color(self, color: impl IntoProp<Color>) -> Self;
    pub fn max_lines(self, max_lines: u32) -> Self;
    pub fn soft_wrap(self, soft_wrap: bool) -> Self;
}

pub struct Button { /* label, on_press, enabled */ }

impl Button {
    /// A button labelled `label`, enabled, doing nothing when pressed.
    pub fn new(label: impl IntoProp<String>) -> Self;
    /// Called when a primary-button press that started on the button is released over it.
    pub fn on_press(self, f: impl Fn() + 'static) -> Self;
    /// A disabled button ignores the pointer and draws dimmed.
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
}

// tantu-view (amendment)
pub struct PointerCx<'a> {
    // ... existing fields ...
    /// The element's size from the last layout.
    pub size: Size,
}
```

`Button` builds a render element with a `RenderPadding` of 24 horizontal and 10 vertical,
inside a `RenderConstrainedBox` with a minimum height of 40 and a minimum width of 64 (Material
3's filled button). A `Text` label in `TextStyle::label()` sits under it, centered. A
`ButtonPaint` fills a rounded rect with radius 20 (fully rounded at 40 px):

| State | Fill | Label |
|---|---|---|
| idle | `0xFF6750A4` | white |
| hovered | the idle fill with 8 % white over it | white |
| pressed | the idle fill with 12 % white over it | white |
| disabled | `0x1F1C1B1F` | `0x611C1B1F` |

## Behavior

- **WIDGETS-TEXT-01:** `Text` builds a render element with a `RenderParagraph` (text, the style's
  key from the tree's table, `max_lines`, `soft_wrap`) and a `ParagraphPaint` with the color.
  Laid out and painted with a text system, it emits glyph runs for its text in its color.
- **WIDGETS-TEXT-02:** A dynamic text or color is applied at the next frame after its signals
  change. A new text re-measures the paragraph, and a new color only repaints it.
- **WIDGETS-BUTTON-01:** A primary `Down` on an enabled button, then an `Up` with the pointer
  inside its bounds, calls `on_press` once. The callback runs with the tree's runtime current,
  so it can write signals. An `Up` outside the bounds, any other button, a disabled button or a
  `Down` that started elsewhere calls nothing.
- **WIDGETS-BUTTON-02:** The button lays out as described above: its label's size plus padding,
  at least 64 × 40.
- **WIDGETS-BUTTON-03:** Visual state: hovered after `Enter` until `Leave`, pressed from a primary
  `Down` until the `Up`, and the fill follows the table at the next frame. Disabled shows the
  disabled colors and ignores the pointer, and becoming disabled clears hovered and pressed.
- **WIDGETS-BUTTON-04:** The label is reactive like `Text`'s.
- **VIEW-EVENT-07:** `PointerCx::size` is the element's size from the last layout (`Size::ZERO` if
  never laid out).

## Performance and allocation

A state change rewrites the paint struct in place (`update_paint`). Nothing is re-laid out
unless the label changes.

## Open questions

1. **Material 3 filled-button metrics and colors as the fixed Phase 2 look.** Proposal: yes.
   They match the Flutter defaults readers know, and the theme replaces them in Phase 3.
2. **`on_press` on release inside**, with no gesture arena. Proposal: yes. Gestures (tap
   slop, long press) come in Phase 3.
