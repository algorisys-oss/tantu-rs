# RenderImage

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-11; review)
- **Crate:** `tantu-layout`
- **Plan item:** Phase 3, "Widgets" → "`Image` (decoding, fit, sizing)" (the sizing part)
- **Related:** [constraints](constraints.md) (LAYOUT-CONS-18, added here), [layout tree](tree.md),
  [Image](../widgets/image.md), Flutter's `RenderImage`

## Purpose

How an image sizes itself: its natural (pixel) size, optionally overridden by an explicit
width and/or height, fitted into the constraints while keeping its aspect ratio where the
constraints allow. Flutter's `RenderImage` sizing, without the painting (the widget paints).

## Public API

```rust
/// Sizes itself to an image's natural size, or an explicit width/height, keeping the aspect
/// ratio within the constraints (Flutter's `RenderImage`, sizing only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderImage {
    /// The image's size in logical pixels (its pixel size at scale 1).
    pub natural: Size,
    /// An explicit width (tight), if any.
    pub width: Option<f32>,
    /// An explicit height (tight), if any.
    pub height: Option<f32>,
}

impl RenderImage {
    pub const fn new(natural: Size) -> Self; // no explicit size
}

impl BoxConstraints {
    /// Flutter's `constrainSizeAndAttemptToPreserveAspectRatio` (LAYOUT-CONS-18).
    pub fn constrain_size_and_attempt_to_preserve_aspect_ratio(self, size: Size) -> Size;
}
```

## Behavior

- **LAYOUT-CONS-18:** `constrain_size_and_attempt_to_preserve_aspect_ratio(size)`: tight
  constraints give their size. Otherwise, with `aspect = width / height`, the width is capped at
  `max_width` (height following), then the height at `max_height` (width following), then the
  width raised to `min_width` (height following), then the height to `min_height` (width
  following), and the result is `constrain`ed. A `size` with a dimension that isn't finite and
  positive counts as `Size::ZERO` (so the result is the smallest size).
- **LAYOUT-IMAGE-01:** `RenderImage` sizes to
  `BoxConstraints::tight_for(width, height).enforce(constraints)
  .constrain_size_and_attempt_to_preserve_aspect_ratio(natural)`: a 60 × 40 image under
  loose 300 × 200 is 60 × 40, with `width` 120 it is 120 × 80, with both set it is tight,
  under tight constraints it fills them, and under loose 30 × 100 it is 30 × 20. It lays out
  no children.
- **LAYOUT-IMAGE-02:** Intrinsics follow Flutter: the max intrinsic width at a height is the
  width of the size for `tight_for_finite(f32::INFINITY, height)` (so 30 at height 20 for a
  60 × 40 image, 60 at an infinite height), and the min intrinsic width is 0 unless `width` or
  `height` is set, in which case it equals the max. Heights mirror this.

## Open questions

Decided by the agent (review): `RenderImage` lives in `tantu-layout` with the other
Flutter render objects, and the fit (how the pixels are placed in the box) is paint-only, so
it stays in the widget.
