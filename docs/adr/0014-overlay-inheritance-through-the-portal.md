# 0014. Overlay inheritance follows the portal target

- **Status:** Accepted (the user said "ok, then continue" on overlay inheritance, 2026-10-12;
  internal representation marked review)
- **Date:** 2026-10-12
- **Related:** [ADR 0011](0011-view-layer.md), [ADR 0012](0012-theme-tokens-through-inherited-signals.md),
  [overlay spec](../specs/view/overlay.md), [inherited values](../specs/view/inherited.md)

## Context

Popups live under the root overlay layer to escape ancestor clipping and paint on top.
Walking only their structural parents therefore misses app-local providers. Tooltip worked
around this by capturing a theme handle, but future menus, dialogs and forms need all
inherited types, including signal handles, with nearest-provider semantics.

## Decision

Keep the overlay wrapper's structural parent as the root overlay layer. Give that wrapper
one private inherited-parent redirect to its portal target, installed before content builds.
`BuildCx::inherit` follows that redirect only for inherited lookup; ordinary elements still
follow their structural parent. Popup-local providers shadow portal context normally, and
nested popups continue through their own targets. Do not copy inherited maps or values.

`ViewTree::parent`, layout, paint, hit-test/event paths, focus traversal and accessibility
continue to use structural ancestry. This decision adds no public API or dependency edge.
Existing overlay ownership/removal disposes consumers and prevents redirects outliving targets.

## Consequences

All inherited types work in popups, including reactive theme/form/locale signals. Tooltip
can resolve its theme where it builds. Lookup allocates nothing and remains linear in
inherited depth. A private optional element id adds fixed metadata per element; its default
is absent. The tree inspector must distinguish inheritance from structural ancestry if it
exposes this detail in future. No generic user-reparenting API is added; the private redirects
are acyclic by construction because each new wrapper points to an already built target.

## Alternatives considered

- Capture individual values at each portal: repeats Tooltip's workaround and misses generic
  types/local overrides; arbitrary erased values cannot all be cloned correctly.
- Structurally reparent popup content under the target: loses the root layer's clipping,
  layout, paint and hit-testing behavior.
- Copy provided maps onto the wrapper: adds work and retains snapshots while hiding the
  actual nearest-provider path; signal identity alone does not solve generic providers.
