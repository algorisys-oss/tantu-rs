# Offstage: retained but hidden subtrees

- **Status:** Draft
- **Crates:** `tantu-view` (the flag and every tree walk), `tantu-a11y` (the semantics walk),
  `tantu-widgets` (the `Offstage` widget)
- **Plan item:** Phase 4, "`TabView`", part 1 (`Offstage`)
- **Related:** [view tree](tree.md), [paint](paint.md), [events](events.md),
  [focus](focus.md), [focus scopes](focus-scope.md), [overlay](overlay.md),
  [semantics](../a11y/semantics.md), [TabView](../widgets/tab-view.md)

## Purpose

Keep a subtree's elements, reactive state and controllers alive while it is not shown.
Tab panels, wizard pages and collapsed docking panels need this: a half-filled form must
survive switching to another tab and back. Today a subtree is either built (and then laid
out, painted, hit-tested, focusable and announced) or disposed; there is nothing between.

## Scope

In scope: an element flag, its effect on layout, paint, hit-testing, hover and pointer
capture, focus and traversal, the IME, `ensure_visible`, overlay portals and the
accessibility tree; a Flutter-named `Offstage` widget.

Out of scope: pausing timers, tickers and effects inside hidden content (they keep running;
a widget that wants to pause watches its own visibility); `maintain_size` / `Visibility`
variants that keep the space; animation of the transition (compose with `Opacity`).

## Public API

```rust
// tantu-view
impl BuildCx<'_> {
    /// Marks `element` (render or region) offstage or onstage. Applies at once while
    /// building; reactive callers bind a `Prop<bool>` and use `ElementMut::set_offstage`.
    pub fn set_offstage(&mut self, element: ElementId, offstage: bool);
}
impl ElementMut<'_> {
    /// As `BuildCx::set_offstage`, for updates applied at a frame.
    pub fn set_offstage(&mut self, offstage: bool);
}
impl ViewTree {
    /// Whether `element` or any ancestor is offstage. False for removed ids.
    pub fn is_offstage(&self, element: ElementId) -> bool;
}

// tantu-widgets (and the prelude)
/// Flutter's `Offstage`: builds its child once and keeps it, hiding it while `offstage`.
pub struct Offstage { /* private */ }
impl Offstage {
    /// `offstage` defaults to true.
    pub fn new(offstage: impl IntoProp<bool>) -> Self;
    /// The kept child.
    pub fn child(self, child: impl View) -> Self;
}
```

`Offstage` is a region element, so it adds no layout node.

## Behavior

- **VIEW-OFFSTAGE-01:** An offstage element and its descendants keep their element ids,
  reactive scopes, signals, controllers and `For`/`Dyn` content; effects inside keep running
  and updates to them are applied at frames as usual. Coming back onstage shows the current
  state, not a rebuilt one. Removing an offstage subtree disposes it like any other.
- **VIEW-OFFSTAGE-02:** Layout: an offstage element's render nodes (its own, or a region's
  flattened render children) are left out of its render ancestor's layout children, so the
  parent lays out as if the subtree were absent (a `Column` gives no space, a `Stack` sizes
  without it) and nothing inside is laid out. Going onstage re-attaches the nodes in tree
  order and marks the parent for layout. Hidden content is laid out when it returns (this
  differs from Flutter's `Offstage`, which lays its child out; review).
- **VIEW-OFFSTAGE-03:** Paint and hit-testing skip offstage subtrees: no Scene commands,
  no hit-test entries, no cursor. An offstage element under the hover path gets `Leave`;
  a pointer capture whose path contains it is released without delivering `Up`.
- **VIEW-OFFSTAGE-04:** Focus: offstage focusables are skipped by `focus_next`/
  `focus_previous`, `focus_into` and a focusing press, and `focus(element)` on one returns
  false. If the focus is in a subtree when it goes offstage, the focus is cleared (focus
  change handlers run, the IME is disabled) before the next layout. Focus scopes inside keep
  their remembered focus, so `focus_into` after coming back onstage restores it.
- **VIEW-OFFSTAGE-05:** `ensure_visible` and `perform_action` on an offstage element return
  false and deliver nothing. Overlay portals whose target is offstage hide their overlay
  (no layout, paint, hit-test or semantics) and show it again when the target returns,
  without rebuilding the overlay content.
- **VIEW-OFFSTAGE-06:** `is_offstage` is true for an element whose own flag or an
  ancestor's is set; nested flags compose (inner onstage under an offstage parent stays
  hidden). Setting the current value again changes nothing (no relayout).
- **A11Y-TREE-06:** The accessibility tree omits offstage subtrees (no nodes, not counted in
  a parent's children) and an `active_descendant` that is offstage is dropped.
- **WIDGETS-OFFSTAGE-01:** `Offstage::new(signal)` follows the prop at frames; its child is
  built exactly once, at `Offstage`'s build, whether or not it starts offstage.

## Performance and allocation

Toggling is O(depth + direct render children) for the layout attachment and O(1) per walk
step for the skips; hidden subtrees cost nothing per frame in layout, paint, hit-testing or
the accessibility tree beyond their running effects. No allocation in the walks.

## Open questions

Proposals, to accept or change:

1. **One element flag honored by every walk**, rather than a layout object (`IndexedStack`)
   plus separate paint/hit/focus/a11y switches: the hiding is a view-tree property, and
   every consumer (tabs, wizards, docking) needs all of it together.
2. **Hidden content is not laid out** (deviation from Flutter, which lays out offstage
   children). Many hidden tabs then cost nothing; the price is a relayout when a tab
   returns and that `LayoutBuilder`s inside see no constraints while hidden.
3. **Focus is cleared, not moved**, when it goes offstage; the owner of the switch (TabView)
   decides where it goes. Timers keep running (the caret blink requests frames only while
   its field is focused, which an offstage field cannot be).
