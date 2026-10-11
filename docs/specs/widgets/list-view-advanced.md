# ListView: variable extents, separators and keyboard navigation

- **Status:** Draft (for review; proposals in Open questions)
- **Crates:** `tantu-layout` (`RenderListContent` with an extent table), `tantu-widgets`
  (`ListView::builder_with_extents`, `ListView::separated`, selection and keys),
  `examples/gallery`
- **Plan item:** Phase 3, "Virtualized `ListView`", sub-item "variable item extents,
  separators, keyboard navigation"
- **Related:** [ListView](list-view.md), [RenderListContent](../layout/list.md),
  [ScrollView](scroll.md) (keys, `animate_to`), [focus](../view/focus.md), Flutter's
  `ListView.builder(itemExtentBuilder:)`, `ListView.separated`

## Purpose

`ListView::builder` virtualizes lists whose items all have one extent. Real lists have
headers, multi-line rows and separators, and desktop users move through them with the
keyboard. This item adds per-item extents the app declares (so positions stay a lookup, not a
layout), separators as a special case of that, and a selection the arrow keys move with the
list scrolling to keep it in view. Measured extents (Flutter's lazy slivers) are not needed for
this: the app knows its row heights, and an extent builder keeps 100 000 rows cheap.

## Scope

In scope:

- `ListExtents` on `RenderListContent`: a fixed extent or a table of extents with prefix
  sums; positions and the visible range by binary search.
- `ListView::builder_with_extents(count, extent_of, build)` and `ListView::separated(count,
  item_extent, build, separator_extent, separator)`.
- Selection and keys: `ListView::selection(signal)`, `on_activate`, a press selects, arrows,
  Home/End, PageUp/PageDown move it and scroll it into view, Enter activates.
- A gallery list with a header row, separators and keyboard selection.

Out of scope: extents measured from the items' intrinsic sizes (a later "prototype item" or
measured mode; it needs layout-driven building), multiple selection, `ensure_visible` for
arbitrary children (scroll leftovers), item enter/exit animations (`AnimatedList`), reordering.

## Public API

```rust
// tantu-layout
/// How long each item of a `RenderListContent` is along the axis.
#[derive(Clone, Debug, PartialEq)]
pub enum ListExtents {
    /// Every item the same; negative or NaN counts as 0.
    Fixed(f32),
    /// Item `i` ends at `ends[i]` (prefix sums, non-decreasing; `ends.len()` is the count).
    Table(Arc<[f32]>),
}

impl ListExtents {
    /// Prefix sums of `extents` (negative or NaN entries count as 0).
    pub fn table(extents: impl IntoIterator<Item = f32>) -> ListExtents;
    /// Where item `index` starts (0 for a fixed extent times index).
    pub fn start(&self, index: usize) -> f32;
    /// Item `index`'s extent.
    pub fn extent(&self, index: usize) -> f32;
    /// The total length of `count` items.
    pub fn total(&self, count: usize) -> f32;
    /// The index of the item containing `position` (the last item at or past the end, 0
    /// before the start), among `count` items; `None` when `count` is 0.
    pub fn index_at(&self, position: f32, count: usize) -> Option<usize>;
}

pub struct RenderListContent {
    pub axis: ScrollAxis,
    pub count: usize,
    /// Was `item_extent: f32`; `RenderListContent::new` still takes an `f32` (`Fixed`).
    pub extents: ListExtents,
}

// tantu-widgets
impl ListView {
    /// `count` items whose extent `extent_of(index)` gives, built by `build(index)` when in
    /// view. `extent_of` runs once per index when the count changes (and at build).
    pub fn builder_with_extents<V: View>(
        count: impl IntoProp<usize>,
        extent_of: impl Fn(usize) -> f32 + 'static,
        build: impl Fn(usize) -> V + 'static,
    ) -> Self;
    /// `count` items of `item_extent` with a separator of `separator_extent` between each
    /// pair, built by `separator(index)` for the separator after item `index`
    /// (Flutter's `ListView.separated`).
    pub fn separated<V: View, S: View>(
        count: impl IntoProp<usize>,
        item_extent: f32,
        build: impl Fn(usize) -> V + 'static,
        separator_extent: f32,
        separator: impl Fn(usize) -> S + 'static,
    ) -> Self;
    /// The selected item, shared with the app (read it in `build` to highlight the item).
    /// Enables keyboard navigation and press-to-select.
    pub fn selection(self, selection: Signal<Option<usize>>) -> Self;
    /// Called with the selected index on Enter, or on a double-click of an item.
    pub fn on_activate(self, f: impl Fn(usize) + 'static) -> Self;
}
```

## Behavior

### Layout

- **LAYOUT-LIST-03:** `ListExtents::table([10, 20, 30])` has starts 0, 10, 30, extents 10, 20,
  30 and total 60; `index_at` is 0 for positions below 10 (and negative), 1 at 10, 2 at 30 and
  at 100 (the last item); `Fixed(40)` behaves as before (`start(i) = 40 i`). `RenderListContent`
  with a table places child `i` at `start(i)` with the main axis tight at `extent(i)`, and its
  main-axis size is `total(count)` (a table shorter than `count` treats the missing items as
  0; a longer one ignores the rest). LAYOUT-LIST-01 and -02 hold with `Fixed`.

### Widget

- **WIDGETS-LIST-04:** `builder_with_extents(3, |i| [10, 20, 30][i], ..)` in a 300 × 200
  window scrolls over 60 px; item 1 is placed at y = 10 and is 20 px tall; the visible range
  comes from `index_at` (one item of buffer each side), so a 100 000-item list with extents
  alternating 20 and 60 keeps at most a viewport's worth plus two of items built; changing
  the count re-runs `extent_of` for every index.
- **WIDGETS-LIST-05:** `separated(3, 40, build, 1, separator)` lays out item 0 at 0, a
  separator at 40 (1 px), item 1 at 41, a separator at 81, item 2 at 82, with no separator after
  the last item (total 122); `Finder::text` finds separators built by `separator(index)` with
  the index of the item before them; a count of 1 has no separator and 0 nothing.
- **WIDGETS-LIST-06:** With `selection(signal)`: a primary press on an item sets the signal to
  its index (the press still reaches the item's own handlers); while the list is focused (a
  press focuses it, as a scroll view), ArrowDown/ArrowUp (or Right/Left when horizontal) move
  the selection by one from the current one (or select the first/last item from `None`),
  Home/End select the first/last, PageDown/PageUp move by the number of items that fit the
  viewport, all clamped; each change scrolls the list the least amount that makes the
  selected item fully visible (at once, `jump_to`); Enter calls `on_activate` with the
  selection, and a double-click on an item (two presses within 500 ms and 4 px) calls it with
  that item. Without `selection`, keys scroll as WIDGETS-SCROLL-06.
- **GALLERY-08:** The Lists section gains a second list (key `contacts`): 200 rows of two
  heights (a 24 px group header every ten rows, 32 px rows) with 1 px separators, a
  selection highlighted in the row's paint, and a "Selected: N" text (key `selected`);
  pressing a row and pressing ArrowDown update it and the list scrolls when the selection
  leaves the view.

## Performance and allocation

An extent table is built once per count change: O(n) time and 4 n bytes for n items (400 KB
for 100 000). The visible range is two binary searches per frame; placement is a lookup per
built item. Keys and presses touch one signal.

## Open questions

Proposals (say "continue" to take them, or change them):

1. **Extents come from the app** (`extent_of`), not from measuring the items: positions stay
   a prefix-sum lookup, 100 000 rows cost what a dozen do, and headers and multi-line rows
   are covered. Measured extents (Flutter's lazy slivers, or a prototype item) are a later
   mode on the same engine.
2. **Separators are entries of the same virtual list** (child `2 i` an item, `2 i + 1` a
   separator) with their own extent, so `separated` reuses the table and the `For`; the
   builder's `index` for an item stays the item index.
3. **Selection is a `Signal<Option<usize>>` the app owns** (as `Radio`'s group): the list moves
   it, the app's `build` reads it to highlight, no `selected` parameter in the builder and no
   built-in highlight paint.
4. **Keys follow the desktop list conventions** (arrows, Home/End, PageUp/Down, Enter) and
   scrolling the selection into view jumps rather than animates; `ensure_visible` for other
   content comes with the scroll leftovers and focus scopes.
5. **`RenderListContent::new` keeps its `f32` signature** (a `Fixed` extent); the table goes
   through the public `extents` field, so the existing layout and widget tests are untouched.
