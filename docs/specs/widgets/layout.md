# Layout widgets

- **Status:** Draft
- **Crate:** `tantu-widgets`
- **Plan item:** Phase 2, `tantu` facade → "Layout widgets"
- **Related:** [ADR 0002](../../adr/0002-flutter-layout-protocol.md),
  [ADR 0011](../../adr/0011-view-layer.md), [view tree](../view/tree.md),
  [frame](../view/frame.md) (props), the layout specs in `../layout/`

## Purpose

Apps compose layout the Flutter way (AGENTS.md: "Flutter structure is the API"). These widgets
are thin views over the existing `tantu-layout` render objects. Each builds one render element,
binds its props, and builds its children. Someone who knows Flutter should read them without
surprise: same names, same defaults, `snake_case`.

## Scope

In scope (in `tantu-widgets`):

| Widget | Render object | Builder |
|---|---|---|
| `Padding` | `RenderPadding` | `Padding::new(insets)`, `Padding::all(v)`, `.child(..)` |
| `Align`, `Center` | `RenderPositionedBox` | `Align::new(alignment)`, `.width_factor`, `.height_factor`, `Center::new()`, `.child(..)` |
| `SizedBox` | `RenderConstrainedBox` | `SizedBox::new(w, h)`, `::width(w)`, `::height(h)`, `::expand()`, `::shrink()`, `.child(..)` |
| `ConstrainedBox` | `RenderConstrainedBox` | `ConstrainedBox::new(constraints)`, `.child(..)` |
| `Row`, `Column` | `RenderFlex` | `::new()`, `.main_axis_alignment`, `.main_axis_size`, `.cross_axis_alignment`, `.spacing`, `.child`, `.children` |
| `Expanded`, `Flexible` | parent data `FlexParentData` | `Expanded::new(child)`, `Flexible::new(child)`, `.flex(n)`; `Flexible` also has `.fit(..)` |
| `Spacer` | an empty box with `FlexParentData { flex, fit: Tight }` | `Spacer::new()`, `.flex(n)` |
| `Stack` | `RenderStack` | `Stack::new()`, `.alignment`, `.fit`, `.child`, `.children` |
| `Positioned` | parent data `StackParentData` | `Positioned::new(child)`, `.left` / `.top` / `.right` / `.bottom` / `.width` / `.height`, `Positioned::fill(child)` |

Out of scope:

- `Wrap`, `FractionallySizedBox`, `AspectRatio`, `LayoutBuilder` widgets. Their render objects
  exist, so they follow in a small later spec, or when the gallery needs them.
- Decorated boxes (`Container`, `DecoratedBox`) need a decoration model, and come with
  `tantu-theme` in Phase 3.

## Public API

Every widget implements `View`. Builders take `self` and return `Self`. Props accept
`impl IntoProp<T>`, so a value, a closure, a `Signal` or a `Memo` all work, and a dynamic prop
updates the render object at the next frame through `ElementMut::update_render`. Children are
`impl View`, stored as `AnyView`. Representative signatures:

```rust
pub struct Column { /* props, children */ }

impl Column {
    /// A column with Flutter's defaults: `Start`, `Max`, `Center`, spacing 0, no children.
    pub fn new() -> Self;
    pub fn main_axis_alignment(self, value: impl IntoProp<MainAxisAlignment>) -> Self;
    pub fn main_axis_size(self, value: impl IntoProp<MainAxisSize>) -> Self;
    pub fn cross_axis_alignment(self, value: impl IntoProp<CrossAxisAlignment>) -> Self;
    pub fn spacing(self, value: impl IntoProp<f32>) -> Self;
    /// Appends a child.
    pub fn child(self, child: impl View) -> Self;
    /// Appends children.
    pub fn children<V: View>(self, children: impl IntoIterator<Item = V>) -> Self;
}

pub struct Padding { /* padding, child */ }

impl Padding {
    pub fn new(padding: impl IntoProp<EdgeInsets>) -> Self;
    /// `Padding::new(EdgeInsets::all(value))`.
    pub fn all(value: f32) -> Self;
    /// Sets the child (replacing a previous one).
    pub fn child(self, child: impl View) -> Self;
}

pub struct Expanded { /* child, flex */ }

impl Expanded {
    /// Makes `child` fill its share of a `Row`'s or `Column`'s free space (flex 1, tight fit).
    pub fn new(child: impl View) -> Self;
    pub fn flex(self, flex: u32) -> Self;
}
```

The other widgets follow the same pattern, with the builders from the table.
`Expanded`/`Flexible`/`Spacer` take a plain `u32` flex and `Positioned` takes plain `f32`
offsets, because parent data is set once while building (see open question 2).

## Behavior

- **WIDGETS-LAYOUT-01:** Each widget builds one render element holding its render object, with
  Flutter's defaults for props that aren't set, and builds its children under it in order.
  Single-child widgets without a child build no children.
- **WIDGETS-LAYOUT-02:** Layout matches the render objects' own specs. For example, a `Column`
  of two 50 px `SizedBox`es with spacing 8 under loose 200 × 200 constraints puts the second at
  y = 58; `Padding::all(16.0)` insets its child by 16 on each side; `Center` centers its child.
- **WIDGETS-LAYOUT-03:** A dynamic prop (closure or signal) is applied right away, and again at
  the next `ViewTree::frame` after its signals change. Only that render object changes, and the
  next layout reflects it. Setting the same value again doesn't mark layout dirty.
- **WIDGETS-LAYOUT-04:** `Expanded` and `Flexible` set `FlexParentData` (tight and loose fit) on
  their child's element, and `Positioned` sets `StackParentData`, so a `Row` with fixed 50 px,
  `Expanded` flex 1 and `Expanded` flex 2 in 350 px gives the flex children 100 and 200 px.
  When the child is a region (`Dyn`, `Show`, `For`), there is no element to hold the parent
  data. The widget then logs a warning and builds the child unwrapped, without panicking.
- **WIDGETS-LAYOUT-05:** `Spacer` takes `flex` shares of the free space and draws nothing.
  `SizedBox::shrink()` is 0 × 0 and `SizedBox::expand()` fills bounded constraints.
- **WIDGETS-LAYOUT-06:** Layout widgets paint nothing themselves (`NoPaint`). Their children are
  painted in order, so a `Stack` paints later children on top.

## Performance and allocation

Building allocates (boxed children, bindings). Layout and paint go through the render objects,
and a static prop adds no per-frame work.

## Open questions

1. **Every layout prop is reactive (`IntoProp`)**, as AGENTS.md asks ("Reactive props accept
   either a value or a closure"). Proposal: yes, through `bind` + `update_render`.
2. **Parent-data props (`flex`, `Positioned` offsets) are plain values** in Phase 2. Reactive
   parent data needs `ElementMut::update_parent_data`, which can come when an app needs it.
3. **A region child of `Expanded` warns and is ignored**, where Flutter would throw. Proposal:
   warn, and revisit when regions forward parent data.
