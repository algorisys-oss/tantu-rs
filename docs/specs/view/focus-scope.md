# Focus scopes and scrolling into view

- **Status:** Agreed (decided by the agent while the user was away, 2026-10-12; the five
  proposals under Open questions are marked for review)
- **Crates:** `tantu-view` (`ensure_visible`, focus scopes, traversal), `tantu-widgets`
  (`ScrollView` reveals, `FocusScope`), `tantu-test` (`ensure_visible`)
- **Plan item:** Phase 3, "Scroll" → "`ensure_visible` (with focus scopes)"
- **Related:** [focus](focus.md) (traversal, which this spec confines to a scope),
  [semantics](../a11y/semantics.md) (`ScrollIntoView`, which shares the mechanism),
  [scroll](../widgets/scroll.md) (the scroll view that does the scrolling),
  [WidgetTester](../test/widget-tester.md)

## Purpose

Keyboard navigation through content larger than the window needs two things Flutter provides
and the focus spec deferred:

- **Scrolling the focus into view.** When Tab moves the focus to a button at the bottom of a
  form inside a `ScrollView`, the view must scroll so the button shows. Flutter's
  `Scrollable.ensureVisible` walks up every enclosing scrollable; the focus traversal policy
  calls it for the newly focused node. The accessibility `ScrollIntoView` action already asks
  for the same thing, so one mechanism serves both.
- **Focus scopes.** A dialog, a popup menu or a docked panel wants Tab to cycle inside it and
  to remember which of its children had the focus. Flutter's `FocusScope` does that: traversal
  stays within the innermost enclosing scope, and a scope remembers its last focused
  descendant so the focus can be restored to it.

## Scope

In scope:

- `ViewTree::ensure_visible`, called by `focus_next`/`focus_previous` (so by Tab) and by the
  `ScrollIntoView` action.
- `ScrollView` revealing a descendant along its axis.
- `BuildCx::focus_scope`, `ViewTree::focus_scope_of`, `ViewTree::focus_into`; traversal
  confined to the innermost scope; the remembered descendant.
- The `FocusScope` widget.
- `WidgetTester::ensure_visible`.

Out of scope (and where it goes):

- Animated reveals and Flutter's alignment policies (`ScrollPositionAlignmentPolicy`): the
  reveal is immediate and uses one rule (below). Animation can come with a `duration` later.
- Directional (arrow-key) traversal and `FocusTraversalGroup` ordering: Phase 4, with grids.
- Restoring focus when a window regains it, and modal barriers: Phase 4, dialogs.
- Revealing virtualized `ListView` items that aren't built: the list does its own
  (WIDGETS-LIST-06); an item that is built is an ordinary descendant and works here.
- Widgets scrolling themselves into view when focused by a pointer or by `focus` (Flutter's
  `EditableText` does for its caret): later, if needed.

## Public API

```rust
// tantu-view
impl ViewTree {
    /// Asks every container enclosing `element` to scroll it into view, innermost first
    /// (VIEW-REVEAL-01); true when one did.
    pub fn ensure_visible(&mut self, element: ElementId) -> bool;

    /// The innermost focus scope among `element` and its ancestors; the root when there is
    /// none; `None` for an unknown element (VIEW-SCOPE-01).
    pub fn focus_scope_of(&self, element: ElementId) -> Option<ElementId>;

    /// Focuses the element `scope` last had the focus on, else its first traversable
    /// descendant; returns the focus, or `None` (changing nothing) when there is neither or
    /// `scope` isn't a scope (VIEW-SCOPE-03).
    pub fn focus_into(&mut self, scope: ElementId) -> Option<ElementId>;
}

impl BuildCx<'_> {
    /// Makes `element` a focus scope: Tab traversal from inside it stays inside it, and it
    /// remembers its last focused descendant (VIEW-SCOPE-01). Render or region elements.
    pub fn focus_scope(&mut self, element: ElementId);
}

// tantu-widgets
/// A subtree whose Tab traversal stays inside it and which remembers its last focused
/// descendant (Flutter's `FocusScope`). Paints nothing.
pub struct FocusScope { /* child */ }

impl FocusScope {
    /// A scope around `child`.
    pub fn new(child: impl View) -> Self;
}

// tantu-test
impl WidgetTester {
    /// Calls `ensure_visible` on the single element `finder` matches, then pumps
    /// (TEST-WT-15).
    pub fn ensure_visible(&mut self, finder: &Finder);
}
```

No new registration API: a container that can reveal descendants handles the
`ScrollIntoView` action with `BuildCx::on_action` (VIEW-SEM-03), as `ScrollView` already does.

## Behavior

### Scrolling into view (`tantu-view`)

- **VIEW-REVEAL-01:** `ensure_visible(element)` visits the element's ancestors from its parent
  to the root. Each ancestor's `on_action` handlers get a `ScrollIntoView` request whose
  `target` is the element to reveal; when a handler returns true, that ancestor becomes the
  target for the ancestors above it (each container reveals the nearest container below it,
  whose bounds are current, rather than the original element, whose bounds change only at
  the next frame). Every ancestor is visited; the result is whether any handler returned true.
  Handlers run with the tree's runtime current. An unknown element returns false and calls
  nothing.
- **VIEW-REVEAL-02:** `focus_next` and `focus_previous` call `ensure_visible` on the element
  they focus (so an unhandled Tab reveals the new focus). `focus`, `focus_into`,
  `focus_on_press` and `set_focusable` don't scroll.
- **VIEW-REVEAL-03:** `perform_action(element, ScrollIntoView)` is `ensure_visible(element)`
  for an element with semantics (amends VIEW-SEM-03: every enclosing container reveals, not
  only the first that returns true). An element without semantics still returns false.

### `ScrollView` (`tantu-widgets`)

- **WIDGETS-SCROLL-09:** A scroll view handles `ScrollIntoView` for a descendant by comparing
  the target's bounds with its own along its axis: when the target is longer than the
  viewport, or starts before the view's start, the offset moves so the target's start is at
  the view's start; otherwise, when the target ends after the view's end, so the target's end
  is at the view's end; otherwise the offset doesn't change. The move is immediate (a running
  animation stops) and clamped to the extents; the cross axis is ignored. The handler returns
  true in every case, so enclosing containers reveal the scroll view itself (VIEW-REVEAL-01).
  Nested scroll views thus both show the target after the next frame.

### Focus scopes (`tantu-view`)

- **VIEW-SCOPE-01:** `focus_scope(element)` marks an element (render or region) as a scope;
  an unknown element is ignored. `focus_scope_of(element)` returns the innermost scope among
  the element itself and its ancestors, the root when none is marked (the root is always a
  scope), and `None` for an unknown element.
- **VIEW-SCOPE-02:** `focus_next` and `focus_previous` visit the traversable focusable elements
  inside the innermost scope enclosing the focused element (the scope's descendants in tree
  order, nested scopes' contents included), wrapping within that scope. With nothing focused
  they use the root scope, as VIEW-FOCUS-03. A scope with no traversable element leaves the
  focus unchanged.
- **VIEW-SCOPE-03:** Every scope enclosing the focused element remembers it. `focus_into(scope)`
  focuses the remembered element when it still exists and is focusable, else the scope's first
  traversable descendant in tree order, and returns the new focus; with neither it changes
  nothing and returns `None`, as it does for an element that isn't a scope.
- **VIEW-SCOPE-04:** Removing an element drops it as a scope and from every scope's memory, so
  `focus_into` falls back to the first traversable descendant.

### `FocusScope` (`tantu-widgets`)

- **WIDGETS-FOCUSSCOPE-01:** `FocusScope::new(child)` builds a region marked as a focus scope
  around `child`; Tab from a focused element inside it cycles through the scope's traversable
  elements and never leaves it, while `focus` on an element outside still works. It paints
  nothing, so it has no golden and no gallery entry (review: dialogs and menus in Phase 4 are
  its first visible users).

### `tantu-test`

- **TEST-WT-15:** `ensure_visible` calls `ViewTree::ensure_visible` on the found element and
  pumps (panics when the finder doesn't match exactly one element, as the other finders).

## Performance and allocation

`ensure_visible` is linear in the element's depth times its ancestors' handler counts; a
reveal that changes nothing costs one bounds query per enclosing container. Traversal stays
linear in the tree, as before (a preorder walk, filtered to the scope). Neither runs per
frame.

## Open questions

Resolved (2026-10-12; decided by the agent while the user was away, review):

1. **One mechanism for accessibility and focus.** `ensure_visible` delivers the
   `ScrollIntoView` action through the `on_action` handlers the semantics spec introduced,
   rather than a second "reveal" registration. Flutter's walk-up (each container reveals the
   container below it) replaces the semantics spec's "first handler wins", so nested scroll
   views work.
2. **Only traversal reveals.** `focus` and a focusing press don't scroll (a press is already
   where the user looks, and a programmatic focus may be setting up state); widgets that want
   more call `ensure_visible`. Flutter's traversal policy makes the same split.
3. **A target longer than the viewport aligns its start**, so the user sees its beginning;
   Flutter's default alignment of 0 does the same.
4. **Scope traversal includes nested scopes' contents** and every enclosing scope remembers
   the focus, as Flutter's `FocusScopeNode.traversalDescendants` and `focusedChild` chain.
   Nested scopes are reached by Tab from the outer scope; leaving an inner scope by Tab needs
   an explicit `focus_into` or `focus` (Flutter has the same rule).
5. **`FocusScope` has no golden or gallery entry** because it paints nothing; AGENTS.md's
   "golden and gallery entry" rule is read as applying to visible widgets.
