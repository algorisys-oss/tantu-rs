# Single-child layouts

- **Status:** Agreed
- **Crate:** `tantu-layout`
- **Plan item:** Phase 2, `tantu-layout` → "Single-child layouts"
- **Related:** [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md),
  [ADR 0009](../../adr/0009-layout-tree-in-tantu-layout.md), [constraints](constraints.md),
  [layout tree](tree.md)

## Purpose

The layouts behind Flutter's most common wrapper widgets: `Padding`, `Align`/`Center`,
`SizedBox`/`ConstrainedBox`, `FractionallySizedBox` and `AspectRatio`. Each is a `RenderBox`
with one child, under Flutter's render-object name, with Flutter's behavior, so a Flutter
developer's expectations (and Flutter's layout docs) carry over. `tantu-widgets` will wrap them
in views; that is where the widget names live.

Each is a small value type with public fields, deriving `PartialEq`, so `tantu-view` updates
them with `LayoutTree::set` and a node is re-laid out only when a property really changed.

## Scope

In scope:

- `Alignment` (Flutter's, `x`/`y` from -1 to 1).
- `RenderPadding`, `RenderPositionedBox` (`Align`, `Center`), `RenderConstrainedBox`
  (`SizedBox`, `ConstrainedBox`), `RenderFractionallySizedBox` (Flutter's
  `RenderFractionallySizedOverflowBox`), `RenderAspectRatio`: layout and intrinsic sizes.

Out of scope (and where it goes):

- `AlignmentDirectional` and `EdgeInsetsDirectional` (start/end, mirrored for RTL): with i18n
  and RTL layout mirroring (Phase 4); the layout objects take resolved values.
- `OverflowBox`, `SizedOverflowBox`, `LimitedBox`, `IntrinsicWidth`/`IntrinsicHeight`,
  `FittedBox`, `Baseline`: added when a widget needs them.
- Painting, clipping and overflow indicators: `tantu-view`.

## Public API

Crate root `tantu_layout`. All types are `Clone + Copy + Debug + PartialEq`.

```rust
use tantu_core::{EdgeInsets, Vec2};

/// A point in a rectangle: `x` and `y` run from -1 (left/top) through 0 (center) to 1
/// (right/bottom). Values outside -1..=1 are allowed and point outside. Flutter's `Alignment`.
pub struct Alignment { pub x: f32, pub y: f32 }

impl Alignment {
    pub const TOP_LEFT: Alignment;      // (-1, -1)
    pub const TOP_CENTER: Alignment;    // ( 0, -1)
    pub const TOP_RIGHT: Alignment;     // ( 1, -1)
    pub const CENTER_LEFT: Alignment;   // (-1,  0)
    pub const CENTER: Alignment;        // ( 0,  0)
    pub const CENTER_RIGHT: Alignment;  // ( 1,  0)
    pub const BOTTOM_LEFT: Alignment;   // (-1,  1)
    pub const BOTTOM_CENTER: Alignment; // ( 0,  1)
    pub const BOTTOM_RIGHT: Alignment;  // ( 1,  1)

    /// An alignment at `(x, y)`.
    pub const fn new(x: f32, y: f32) -> Self;
    /// The offset of this point in a free space of `free` (the parent's size minus the
    /// child's): `((x + 1) / 2 · free.x, (y + 1) / 2 · free.y)`. Negative free space gives a
    /// negative offset (the child overflows evenly for `CENTER`).
    pub fn along_offset(self, free: Vec2) -> Vec2;
}
impl Default for Alignment { /* CENTER */ }

/// Insets its child by `padding` (Flutter's `RenderPadding`).
pub struct RenderPadding { pub padding: EdgeInsets }
impl RenderPadding { pub const fn new(padding: EdgeInsets) -> Self; }

/// Positions its child within itself by `alignment`; sizes itself to the child times a factor
/// on an axis with a factor or unbounded constraints, else as large as allowed (Flutter's
/// `RenderPositionedBox`, behind `Align` and `Center`).
pub struct RenderPositionedBox {
    pub alignment: Alignment,
    pub width_factor: Option<f32>,
    pub height_factor: Option<f32>,
}
impl RenderPositionedBox {
    /// `alignment`, no factors.
    pub const fn new(alignment: Alignment) -> Self;
    /// `CENTER`, no factors (`Center`).
    pub const fn center() -> Self;
}
impl Default for RenderPositionedBox { /* center() */ }

/// Imposes `additional` constraints on its child, within the incoming ones (Flutter's
/// `RenderConstrainedBox`, behind `ConstrainedBox` and `SizedBox`).
pub struct RenderConstrainedBox { pub additional: BoxConstraints }
impl RenderConstrainedBox {
    pub const fn new(additional: BoxConstraints) -> Self;
    /// `SizedBox(width, height)`: `BoxConstraints::tight_for(width, height)`.
    pub const fn sized(width: Option<f32>, height: Option<f32>) -> Self;
    /// `SizedBox.expand()`: `BoxConstraints::expand(None, None)`.
    pub const fn expand() -> Self;
    /// `SizedBox.shrink()`: tight at 0 × 0.
    pub const fn shrink() -> Self;
}

/// Sizes its child to a fraction of the incoming maximum on an axis with a factor, and
/// positions it by `alignment`; the child may overflow (Flutter's
/// `RenderFractionallySizedOverflowBox`, behind `FractionallySizedBox`).
pub struct RenderFractionallySizedBox {
    pub alignment: Alignment,
    pub width_factor: Option<f32>,
    pub height_factor: Option<f32>,
}
impl RenderFractionallySizedBox {
    pub const fn new(width_factor: Option<f32>, height_factor: Option<f32>) -> Self; // CENTER
}

/// Sizes itself (and its child, tightly) to `aspect_ratio` = width / height, as large as the
/// constraints allow (Flutter's `RenderAspectRatio`).
pub struct RenderAspectRatio { pub aspect_ratio: f32 }
impl RenderAspectRatio { pub const fn new(aspect_ratio: f32) -> Self; }
```

## Behavior

"The child" is the node's first child. Every layout here lays out at most that child; further
children are never laid out (their size stays `None` and their offset unchanged). Sizes are
then constrained by the tree (LAYOUT-TREE-09); the rules below give the size before that,
which for normalized constraints already satisfies them. "Child intrinsics" are the child's
intrinsic sizes through the tree, 0 without a child.

### Alignment

- **LAYOUT-SINGLE-01:** The nine constants have the values listed; `default()` is `CENTER`;
  `new` stores as given.
- **LAYOUT-SINGLE-02:** `along_offset(free)` is `((x + 1) / 2 · free.x, (y + 1) / 2 · free.y)`:
  `TOP_LEFT` gives zero, `BOTTOM_RIGHT` gives `free`, `CENTER` gives `free / 2`, including for
  negative `free`.

### RenderPadding

- **LAYOUT-SINGLE-03:** With a child: the child is laid out with `constraints.deflate(padding)`,
  placed at `(padding.left, padding.top)`, and the size is the child's size plus
  `(padding.horizontal(), padding.vertical())`. Without a child: the size is
  `constraints.constrain((padding.horizontal(), padding.vertical()))`.
- **LAYOUT-SINGLE-04:** Intrinsics: the width ones are the child's at
  `max(0, height − vertical)` plus `horizontal`; the height ones are the child's at
  `max(0, width − horizontal)` plus `vertical` (with no child: just `horizontal` or `vertical`).
  An infinite argument stays infinite.

### RenderPositionedBox

- **LAYOUT-SINGLE-05:** An axis *shrink-wraps* when it has a factor or its incoming maximum is
  infinite. With a child: the child is laid out with `constraints.loosen()`; on a
  shrink-wrapping axis the size is the child's extent times the factor (1 without one), on
  the other axis it is as large as allowed (the incoming maximum); the child is placed at
  `alignment.along_offset(size − child size)`. Without a child: 0 on a shrink-wrapping axis,
  the maximum on the other.
- **LAYOUT-SINGLE-06:** A factor that is negative or NaN counts as 0. Intrinsics are the
  child's (factors don't scale them, as in Flutter).

### RenderConstrainedBox

- **LAYOUT-SINGLE-07:** With a child: the child is laid out with
  `additional.enforce(constraints)` and the size is the child's size. Without a child: the size
  is `additional.enforce(constraints).constrain(Size::ZERO)`. So `sized(Some(w), Some(h))`
  gives `w × h` within the incoming constraints, `expand()` fills bounded constraints, and
  `shrink()` is as small as allowed.
- **LAYOUT-SINGLE-08:** The `sized`, `expand` and `shrink` constructors use the constraints
  listed in the API.
- **LAYOUT-SINGLE-09:** Intrinsic width (min and max): if `additional` has a bounded, tight width,
  that width; otherwise the child's, passed through `additional.constrain_width` unless
  `additional` has an infinite minimum width. Heights mirror this.

### RenderFractionallySizedBox

- **LAYOUT-SINGLE-10:** The child's constraints: on an axis with a factor, tight at the incoming
  maximum times the factor; on an axis without one, the incoming range unchanged. With a
  child, the size is `constraints.constrain(child size)` and the child is placed at
  `alignment.along_offset(size − child size)`, so a factor above 1 overflows (negative offset
  for `CENTER`). Without a child, the size is `constraints.constrain` of the child's
  constraints' smallest size. A negative or NaN factor counts as 0.
- **LAYOUT-SINGLE-11:** Intrinsics: the child's divided by the factor on that axis (1 without
  one); a 0 factor gives an infinite intrinsic for a positive child intrinsic.

### RenderAspectRatio

- **LAYOUT-SINGLE-12:** The size: if the constraints are tight, their smallest size. Otherwise
  start from the maximum width (or, when it is infinite, from the maximum height times the
  ratio; when both are infinite, from the minimum width), derive the other side from the ratio,
  then fit it in this order, re-deriving the other side each time: width above the maximum,
  height above the maximum, width below the minimum, height below the minimum. The child is
  laid out tight at that size and placed at zero.
- **LAYOUT-SINGLE-13:** An `aspect_ratio` that is not finite and positive gives the smallest
  allowed size (the child is laid out tight at it).
- **LAYOUT-SINGLE-14:** Intrinsics: width (min and max) at a finite height is `height · ratio`;
  height (min and max) at a finite width is `width / ratio`; at an infinite argument, the
  child's. A ratio that isn't finite and positive gives the child's intrinsics.

### Shared

- **LAYOUT-SINGLE-15:** Every layout here lays out only its first child (LAYOUT-TREE-08 index 0),
  works without a child, and never panics for any constraints, properties (NaN, infinities,
  negative values) or child sizes. Equal property values compare equal, so `LayoutTree::set`
  with unchanged values doesn't re-lay out.

## Performance and allocation

Each layout is O(1) besides its child, allocates nothing, and is a few dozen bytes.

## Open questions

Resolved (2026-10-10, agreed with the user):

1. **One rule prefix,** `LAYOUT-SINGLE-NN`, for the whole spec.
2. **Malformed properties get defined fallbacks** (a negative or NaN factor counts as 0; a bad
   aspect ratio gives the smallest size), never a panic, in line with `BoxConstraints`.
3. **`RenderFractionallySizedBox`**, without Flutter's `Overflow`, to match the widget name.
