# 0013. Accessibility: semantics in the view tree, AccessKit trees from `tantu-a11y`

- **Status:** Proposed
- **Date:** 2026-10-12
- **Related:** [ADR 0004](0004-platform-trait-with-winit-by-default.md) (platform trait),
  [ADR 0011](0011-view-layer.md) (elements), spec [semantics](../specs/a11y/semantics.md),
  spec [a11y platform](../specs/a11y/platform.md), PLAN.md ("AccessKit as a first-class part of
  the pipeline")

## Context

AGENTS.md promises an AccessKit tree for every standard widget, and the 0.1 criteria say
screen readers (Narrator, VoiceOver, Orca) must be able to operate the gallery. AccessKit
needs, per window, a tree of nodes (role, name, value, state, bounds, children, focus) pushed
as `TreeUpdate`s, and sends back `ActionRequest`s (focus, click, increment, set value). The
platform adapter (`accesskit_winit`) hooks winit's event loop and window; the tree itself comes
from the UI. The dependency rule keeps widgets and `tantu-view` away from platform crates, and
`tantu-platform` away from the view layer.

## Decision

1. **Semantics are data on elements, in `tantu-view`.** A backend-neutral `Semantics` struct
   (`Role`, name, value, states, numeric range) is set at build (`BuildCx::set_semantics`) and
   updated reactively (`ElementMut::update_semantics`) like paints. `tantu-view` knows nothing
   of AccessKit.
2. **`tantu-a11y` builds AccessKit trees from the view tree.** It walks the elements, emits a
   node per element with semantics (unsemantic layout and region elements are skipped, their
   descendants re-parented), node ids are the element ids' bits (decision 19), bounds come
   from the layout tree, focus from the tree's focus. It also maps `ActionRequest`s to view
   actions (`ViewTree::perform_action`), which synthesize the same events a user would produce
   (a click is a press and release at the element's center; increment is an arrow key).
3. **AccessKit's data crate is a dependency of `tantu-a11y` and `tantu-platform`;
   `accesskit_winit` of `tantu-platform-winit` only.** The platform trait carries
   accessibility as events (`Activated`, `Deactivated`, `Action`) and one context call
   (`update_accessibility(window, TreeUpdate)`), as it does for IME. The runner (`tantu`)
   builds and sends a tree after each frame while a window's assistive technology is active,
   and routes actions to the tree.

## Consequences

- AGENTS.md: `tantu-a11y | view` gains the external `accesskit` dependency, and so does
  `tantu-platform`; `tantu-test` may depend on `tantu-a11y` so tests can inspect the tree.
  The dependency rule gains a line: only `tantu-platform-*` may depend on `accesskit_*`
  adapters.
- Every widget sets semantics; a widget without them is invisible to assistive technology,
  which the spec treats as a bug (the gallery test checks every widget has a node).
- Full-tree updates per frame for now (small trees; AccessKit diffs); incremental updates
  are a later optimization behind the same `update` call.
- Text editing semantics (caret, selection as AccessKit text ranges) are a later addition;
  v1 exposes a text field's value and lets `SetValue` replace it.

## Alternatives considered

- **AccessKit types directly in `tantu-view`** (`accesskit::Node` on elements): simplest, but
  ties the view layer to one accessibility API and its version churn. Rejected.
- **Widgets building AccessKit nodes themselves** (a `Semantics` trait with an
  `accesskit` dependency in `tantu-widgets`): the same coupling one level up. Rejected.
- **A separate semantics tree with its own identity** (Flutter's `SemanticsNode`s with
  merging): more flexible, but elements already have stable ids and a parent chain; merging
  can come later as a `Semantics` option. Deferred.
