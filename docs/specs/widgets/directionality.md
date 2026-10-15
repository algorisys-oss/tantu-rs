# i18n, part 1: right-to-left layout mirroring

- **Status:** Implemented (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-layout` (`TextDirection`, `AlignmentDirectional`, `EdgeInsetsDirectional`,
  `RenderFlex`/`RenderWrap` direction), `tantu-view` (`ElementMut::set_parent_data`),
  `tantu-widgets` (`Directionality`, directional `Padding`/`Align`/`Positioned`, `Row` and
  `Column` following the direction), `examples/gallery`
- **Plan item:** Phase 4, "i18n: Fluent-based localisation, RTL layout mirroring,
  locale-aware formatting", part 1 (RTL layout mirroring)
- **Related:** [flex](../layout/flex.md), [wrap](../layout/wrap.md),
  [layout widgets](layout.md), [inherited values](../view/inherited.md),
  [bidi and fallback](../text/scripts.md), [gallery](../examples/gallery.md)

## Purpose

Arabic, Hebrew, Persian and Urdu users read right to left, and expect the whole interface
to mirror: a row's first child on the right, a column's "start" edge on the right, leading
padding on the right. Flutter does this with a `Directionality` ancestor and directional
variants of its layout properties (`AlignmentDirectional`, `EdgeInsetsDirectional`,
`PositionedDirectional`), resolved against the nearest direction. Tantu adopts the same
model. Paragraphs already lay out bidirectional text (TEXT-SCRIPT-01..04); this part
mirrors the layout around them.

## Scope

In scope: a `TextDirection` and the directional alignment and insets in `tantu-layout`;
`RenderFlex` and `RenderWrap` laying out right to left; a `Directionality` widget providing
the direction to a subtree (reactively); `Row` and `Column` following it; directional
`Padding`, `Align` and `Positioned`; a gallery switch.

Out of scope (later parts or later): mirroring the insides of widgets that paint their own
geometry (the slider's track direction, the text field's caret movement keys, chevron
icons, scrollbar sides); a paragraph's alignment and base direction following
`Directionality` (paragraphs keep detecting their direction from their text); Fluent
localization (part 2) and locale formats (part 3); a `verticalDirection`.

## Public API

```rust
// tantu-layout
/// The direction text and horizontal layout run in (Flutter's `TextDirection`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextDirection { #[default] Ltr, Rtl }

/// An alignment whose x runs from the start edge (-1) to the end edge (1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlignmentDirectional { pub start: f32, pub y: f32 }
impl AlignmentDirectional {
    pub const TOP_START: Self; pub const TOP_CENTER: Self; pub const TOP_END: Self;
    pub const CENTER_START: Self; pub const CENTER: Self; pub const CENTER_END: Self;
    pub const BOTTOM_START: Self; pub const BOTTOM_CENTER: Self; pub const BOTTOM_END: Self;
    pub const fn new(start: f32, y: f32) -> Self;
    /// The alignment for `direction`: start is left for `Ltr`, right for `Rtl`.
    pub fn resolve(self, direction: TextDirection) -> Alignment;
}

/// Insets with start and end sides instead of left and right.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeInsetsDirectional { pub start: f32, pub top: f32, pub end: f32, pub bottom: f32 }
impl EdgeInsetsDirectional {
    pub const fn new(start: f32, top: f32, end: f32, bottom: f32) -> Self;
    pub fn resolve(self, direction: TextDirection) -> EdgeInsets;
}

pub struct RenderFlex { /* existing, */ pub text_direction: TextDirection }
pub struct RenderWrap { /* existing, */ pub text_direction: TextDirection }

// tantu-view
impl ElementMut<'_> {
    /// Replaces the element's parent data (as `BuildCx::set_parent_data`).
    pub fn set_parent_data<T: Any>(&mut self, data: T) -> bool;
}

// tantu-widgets
/// Gives its subtree a text direction (Flutter's `Directionality`).
pub struct Directionality { /* private */ }
impl Directionality {
    pub fn new(direction: impl IntoProp<TextDirection>, child: impl View) -> Self;
    /// The nearest direction above the element being built (`Ltr` without one); a tracked
    /// read.
    pub fn of(cx: &BuildCx<'_>) -> TextDirection;
}
impl Padding { pub fn directional(padding: impl IntoProp<EdgeInsetsDirectional>) -> Self; }
impl Align { pub fn directional(alignment: impl IntoProp<AlignmentDirectional>) -> Self; }
/// `Positioned` with start and end instead of left and right (Flutter's `PositionedDirectional`).
pub struct PositionedDirectional { /* private */ }
impl PositionedDirectional {
    pub fn new(child: impl View) -> Self;
    pub fn start(self, value: f32) -> Self;
    pub fn end(self, value: f32) -> Self;
    pub fn top(self, value: f32) -> Self;
    pub fn bottom(self, value: f32) -> Self;
    pub fn width(self, value: f32) -> Self;
    pub fn height(self, value: f32) -> Self;
}
```

## Behavior

- **LAYOUT-DIR-01:** `AlignmentDirectional::resolve` gives `Alignment::new(start, y)` for
  `Ltr` and `Alignment::new(-start, y)` for `Rtl`; `EdgeInsetsDirectional::resolve` puts
  `start` on the left for `Ltr` and on the right for `Rtl` (`end` opposite).
- **LAYOUT-FLEX-15:** A `RenderFlex` with `text_direction: Rtl` lays out as with `Ltr` and
  then mirrors each child's horizontal offset within its own width (`x' = width - x -
  child width`): a row runs right to left (`Start` packs children at the right, its first
  child rightmost) and a column's cross-axis `Start` and `End` are the right and left
  edges. The default is `Ltr`, so existing layouts are unchanged; the overflow is the same.
- **LAYOUT-WRAP-10:** A `RenderWrap` with `text_direction: Rtl` mirrors the same way: a
  horizontal wrap's runs fill from the right, a vertical wrap's runs advance from the
  right.
- **VIEW-FRAME-09:** `ElementMut::set_parent_data` replaces a render element's parent data
  and marks its layout parent for layout; false for a region (so directional parent data
  can follow a reactive direction).
- **WIDGETS-DIR-01:** `Directionality` provides its direction (a value, closure or signal)
  to every element built inside it, nearest provider first; `Directionality::of` returns
  it, `Ltr` when there is none. Overlay content gets the direction of its portal's target
  (ADR 0014).
- **WIDGETS-DIR-02:** `Row` and `Column` take the nearest direction as their flex's
  `text_direction`, and follow it when it changes.
- **WIDGETS-DIR-03:** `Padding::directional`, `Align::directional` and
  `PositionedDirectional` resolve their directional values against the nearest direction,
  and follow both the values and the direction when they change.
- **GALLERY-46:** The gallery's header gains an "RTL" switch before "Dark"; on, the window's
  content below the menu bar is laid out right to left (the sidebar moves to the right).
  Test: switching it on puts the sidebar's buttons right of the section title; golden
  `gallery-rtl.png`.

## Performance and allocation

Mirroring is one pass over a flex's or wrap's children after layout, only for `Rtl`;
nothing allocates.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **Flutter's model**: an inherited direction plus directional property types, resolved
   by the widgets; layout objects only know a resolved `TextDirection`.
2. **Mirror after layout** in `RenderFlex` and `RenderWrap`, which gives Flutter's results
   for every alignment without duplicating the placement code.
3. **Widgets' own painted geometry is not mirrored yet** (slider, caret keys, icons);
   each widget gets its own amendment when it is done.
4. **The gallery mirrors the content, not the menu bar**, so the switch stays reachable
   where it was.
