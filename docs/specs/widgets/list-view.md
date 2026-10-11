# ListView

- **Status:** Agreed (decided by the agent while the user was away, 2026-10-12; review the
  Open questions)
- **Crate:** `tantu-widgets`; layout in [RenderListContent](../layout/list.md)
- **Plan item:** Phase 3, "Virtualized `ListView` (100k items at 60fps)"
- **Related:** [ScrollView](scroll.md), [dynamic content](../view/dynamic.md) (`For`),
  Flutter's `ListView.builder` with `itemExtent`

## Purpose

A list of many items that builds only the ones in view. `ListView::builder` takes an item
count, a fixed item extent and a builder; a `ScrollView` scrolls over content that is as long
as the whole list, and a keyed `For` region builds the items whose indices are visible (plus
one on each side), removing the ones that scrolled out and keeping the identity of the ones
that stay. 100 000 items cost what a dozen do.

## Scope

In scope: `ListView::builder`, `.horizontal()`, `.controller(..)`; a reactive count; the
visible range from the controller's offset and viewport extent.

Out of scope (and where it goes): variable item extents, separators, `ListView::new` over
fixed children (a `ScrollView` around a `Column` does that), item animations, keyboard
navigation and selection (with focus scopes and the data grid), horizontal lists' cross
sizing from items.

## Public API

```rust
/// A scrolling list that builds only its visible items (Flutter's `ListView.builder` with
/// `itemExtent`).
pub struct ListView { /* count, item_extent, build, horizontal, controller */ }

impl ListView {
    /// `count` items (a value or a signal) of `item_extent` logical pixels each, built by
    /// `build(index)` when they come into view.
    pub fn builder<V: View>(count: impl IntoProp<usize>, item_extent: f32, build: impl Fn(usize) -> V + 'static) -> Self;
    pub fn horizontal(self) -> Self;
    pub fn controller(self, controller: ScrollController) -> Self;
}
```

Structure: a `ScrollView` (the list's axis and controller) around a render element with a
`RenderListContent` (axis, count, extent) whose only child is a `For` over the visible indices,
keyed by index; each built item is wrapped so its element carries `ListIndex(index)`. The
visible range, from the controller: with offset `o`, viewport extent `v` and extent `e`,
indices from `max(⌊o / e⌋ − 1, 0)` to `min(⌈(o + v) / e⌉ + 1, count)` (exclusive). Before the
first frame recorded the extents, `v` is 0, so the first items appear at the second frame.

## Behavior

- **WIDGETS-LIST-01:** A `ListView` of 50 items of 40 px in a 300 × 200 window scrolls over
  2000 px of content (the controller's `max_scroll_extent` is 1800) and places item `i` at
  `y = i · 40 − offset`; `Finder::text` finds the visible items, each 300 × 40.
- **WIDGETS-LIST-02:** Virtualization: after the second frame only the visible items plus one
  before and after exist (a 100 000-item list in a 200 px window with 40 px items has 6 item
  elements at offset 0 and at most 8 anywhere); scrolling removes the items that left and
  keeps the element ids of the ones still in view; at the maximum offset the last item is
  visible; the builder was called once per item that appeared.
- **WIDGETS-LIST-03:** `count` is reactive: a larger count extends the content and the range,
  a count of 0 shows nothing and has no scroll extent; a horizontal list scrolls along x with
  items placed by x.

## Open questions

Decided by the agent while the user was away (review):

1. **Fixed item extents** (`itemExtent`), the fast path; variable extents later.
2. **One item of buffer** on each side, instead of Flutter's 250 px cache extent.
3. **Items keyed by index**, so a stable index keeps its element across scrolls; reordering
   data under a stable index rebuilds nothing (the item's own signals should carry its data).
4. **The first frame shows no items** (the extents come from it); the runner's startup already
   draws two frames, and the tester pumps as needed.
