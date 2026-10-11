# ScrollView

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-12; review
  the Open questions)
- **Crates:** `tantu-widgets` (`ScrollView`, `ScrollController`), `tantu-test`
  (`WidgetTester::scroll`); layout in [RenderViewport](../layout/viewport.md)
- **Plan item:** Phase 3, "Scroll: `ScrollView`, scrollbars, kinetic/wheel/trackpad handling"
- **Related:** [events](../view/events.md) (`PointerKind::Scroll`), [paint](../view/paint.md)
  (clipping and culling), Flutter's `SingleChildScrollView`, `ScrollController`, `Scrollbar`

## Purpose

A box whose content can be taller (or wider) than it is: the wheel or trackpad scrolls it, a
scrollbar shows where you are and can be dragged, content outside the box is clipped and not
painted, and taps land on the scrolled content. The offset lives in a `ScrollController` the
app can read and set, like the text field's controller. Flutter's `SingleChildScrollView` plus
`Scrollbar`, for the desktop.

## Scope

In scope:

- `ScrollController` (offset, extents, `jump_to`), `ScrollView::new(child)`, `.horizontal()`,
  `.controller(..)`.
- Wheel and trackpad deltas, clamping, nested scroll views (the innermost that can move takes
  the event).
- A scrollbar (thumb on the trailing edge) that follows the offset and can be dragged.
- Clipping and culling; hit-testing through the offset; `WidgetTester::scroll`.

Out of scope (and where it goes):

- Kinetic scrolling, smooth wheel animation, overscroll bounce, `animate_to`: the animation
  item (timers). Until then every delta applies at once.
- Keyboard scrolling (PageUp/Down, arrows, Home/End) and `ensure_visible` (focus scrolling,
  the text field's caret following): with focus scopes, next.
- Scrollbar track clicks (paging), auto-hiding scrollbars, theming: `tantu-theme` and later.
- Virtualization (`ListView` builds only visible items): the next item, on this viewport.

## Public API

```rust
// tantu-widgets
/// A scroll view's offset and extents, shared between the app and the view. `Copy`; create it
/// inside a runtime.
#[derive(Clone, Copy)]
pub struct ScrollController { /* offset: Signal<f32>, extents: Signal<(f32, f32)> */ }

impl ScrollController {
    pub fn new() -> Self;
    /// The offset in logical pixels (a tracked read).
    pub fn offset(&self) -> f32;
    /// Sets the offset; clamped to `0..=max_scroll_extent` at the next frame.
    pub fn jump_to(&self, offset: f32);
    /// The largest offset: content minus viewport along the axis, at least 0 (0 before the
    /// first frame). A tracked read.
    pub fn max_scroll_extent(&self) -> f32;
    /// The viewport's extent along the axis, from the last frame.
    pub fn viewport_extent(&self) -> f32;
}

/// A scrollable box around one child (Flutter's `SingleChildScrollView` with a `Scrollbar`).
pub struct ScrollView { /* child, axis, controller */ }

impl ScrollView {
    /// A vertical scroll view around `child`, with its own controller.
    pub fn new(child: impl View) -> Self;
    pub fn horizontal(self) -> Self;
    pub fn controller(self, controller: ScrollController) -> Self;
}

// tantu-test
impl WidgetTester {
    /// Dispatches a wheel scroll of `delta` at the center of the found element, then pumps
    /// (TEST-WT-11).
    pub fn scroll(&mut self, finder: &Finder, delta: Vec2);
}
```

Structure: one render element with a `RenderViewport` (the axis, the controller's offset) and
the child under it; the element holds the pointer handler and a `ViewportPaint` that clips to
the bounds (`clips_children`), paints the child, then the scrollbar.

Scrollbar (desktop Material look, drawn when the content exceeds the viewport): a thumb of
width 8 px, 2 px in from the trailing edge (right for vertical, bottom for horizontal), corner
radius 4, color on-surface at 40 % (`0x661C1B1F`); the track is the viewport's extent along
the axis minus 4 px (2 px margins); the thumb's length is `track · viewport / content`, at
least 24 px; its position is `track_start + (track − thumb) · offset / max`.

## Behavior

- **WIDGETS-SCROLL-01:** `ScrollView` builds the structure above and sizes by
  LAYOUT-VIEWPORT-01 (a 300 × 1000 column in a 300 × 200 window shows 300 × 200). A fresh
  `ScrollController` has offset 0 and extents 0; after the first frame `max_scroll_extent` is
  content minus viewport (800) and `viewport_extent` 200; `jump_to(5000)` shows 800 after the
  next frame and `jump_to(-10)` 0. The child's position follows the offset (its rect moves
  up by the offset).
- **WIDGETS-SCROLL-02:** A `Scroll` pointer event over the view moves the offset by the
  delta's component along the axis (positive down/right), clamped to the extents, and stops
  the event when it moved; when it can't move (at an end, or no overflow) it continues, so an
  outer scroll view scrolls. The offset is reactive: the controller's `offset()` updates.
- **WIDGETS-SCROLL-03:** Painting: the child is painted offset and clipped to the view's
  bounds, and leaves outside the bounds emit no commands (a column of 50 boxes of 40 px in a
  200 px view paints 5 or 6 fills, not 50).
- **WIDGETS-SCROLL-04:** The scrollbar is painted per the formulas when `max_scroll_extent >
  0` (a 1000 px content in a 200 px view gives a thumb of `196 · 0.2 = 39.2` px at the top,
  moving to the track's end at the maximum offset) and not at all otherwise. A primary press
  on the thumb and a drag move the offset by `drag · content / track`, clamped; the press
  doesn't scroll the content under the pointer.
- **WIDGETS-SCROLL-05:** Taps reach scrolled content: a button at y = 500 in the content, after
  scrolling by 400, is tapped at y = 100 of the view, and a button scrolled out of the view
  isn't hit.
- **TEST-WT-11:** `scroll` dispatches `PointerKind::Scroll(delta)` at the element's center and
  pumps.

## Open questions

Decided by the agent while the user was away (review):

1. **Deltas apply at once**, no animation or kinetic phase until timers exist.
2. **The innermost view that can move takes the wheel**, by stopping the event only when it
   moved (Flutter nests the same way through its scroll physics).
3. **A controller of two signals** (offset and extents), `Copy`, like the text field's; the
   extents are written after layout when they change, so reading them is reactive.
4. **Only the axis's component** of a delta scrolls (a horizontal view ignores vertical wheel
   motion; trackpads give both components).
5. **A fixed-look scrollbar, always shown when needed**; auto-hide and the theme come later.
