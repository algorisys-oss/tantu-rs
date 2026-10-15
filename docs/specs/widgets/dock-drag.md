# Docking, part 3: dragging tabs to dock them

- **Status:** Draft
- **Crate:** `tantu-widgets` (`DockArea`), `examples/gallery`
- **Plan item:** Phase 4, "Docking layout manager", part 3 (drag-to-dock)
- **Related:** [dock layout model](dock-layout.md) (part 1, `DockTarget`, `move_panel`),
  [DockArea](dock-area.md) (part 2), [events](../view/events.md) (pointer capture),
  [DataGrid columns](data-grid-columns.md) (drag slop and drop line),
  [gallery](../examples/gallery.md)

## Purpose

Users arrange a docked workspace by dragging: a tab dropped elsewhere in its strip
reorders it, dropped on another strip joins that group, dropped on a group's content edge
splits the group, and dropped near the dock's own edge creates a full-length group along
it. While dragging, an indicator shows exactly where the panel will go. The drop is one
`DockLayout::move_panel` call (part 1), so the app's layout signal, persistence and
undo-by-reset all keep working. A keyboard command reorders tabs for users who don't drag.

## Scope

In scope: starting a drag from a tab; computing the `DockTarget` under the pointer
(strip insertion points, five zones per group, four dock edges); painting the drop
indicator and dimming the dragged tab; dropping and cancelling; a keyboard reorder; a
gallery test and golden.

Out of scope (and where it goes): dragging a panel out of the window into a floating
window (multi-window); drag-and-drop between separate `DockArea`s or with the OS (the
drag-and-drop item); a ghost image following the pointer and a grabbing cursor (cursors
are set per element at build, so a drag-time cursor needs a view amendment; later);
auto-scrolling an overflowing strip while dragging over its ends (later); animations.

## Public API

No new public items. `DockArea` gains the behavior below. (Internally the area keeps
each strip's tab places, scroll offset and the group content rects it already records.)

## Behavior

- **WIDGETS-DOCKDRAG-01:** A primary press on a tab followed by a pointer move of more
  than 4 px (the grid's slop) starts a drag of that tab's panel; a release before that is
  the part 2 click (activation). Middle and secondary buttons never drag. While dragging,
  the dragged tab is drawn at 38 % opacity and nothing is activated, hovered or resized
  by the drag's moves.
- **WIDGETS-DOCKDRAG-02:** Over a group's strip, the target is `Tabs { group, index }`
  with `index` the insertion point nearest the pointer: the number of that strip's tabs,
  not counting the dragged one, whose centre lies left of the pointer (the strip's scroll
  offset included). The indicator is a 2 px `primary` vertical line the strip's height at
  that boundary.
- **WIDGETS-DOCKDRAG-03:** Over a group's content (below its strip), the pointer's
  distances to the content's four edges, as fractions of its width or height, pick the
  zone: the nearest edge when that fraction is under 0.25 (ties: left, right, top, bottom),
  giving `Beside { group, side }`, else the centre, giving `Tabs { group, index: end }`.
  The indicator is a `primary` fill at 16 % with a 2 px `primary` outline over the half of
  the content on that side, or the whole content for the centre.
- **WIDGETS-DOCKDRAG-04:** Within 16 px of the dock's outer edge (and not over a strip),
  the target is `Edge(side)` for the nearest edge (same tie order), taking priority over
  the group zones. The indicator is the zone fill over the quarter of the dock along that
  edge.
- **WIDGETS-DOCKDRAG-05:** There is no target, and no indicator, outside the dock, over a
  divider, or when the move would change nothing (`move_panel` would return `NoOp`, or the
  tab would land where it is: its own group's strip at its own index, or its own
  single-panel group's centre).
- **WIDGETS-DOCKDRAG-06:** Releasing the primary button with a target moves the panel
  there with `move_panel` through the layout signal; the panel becomes its new group's
  active tab, that group becomes the active group, and the focus moves to its strip.
  Releasing without a target, pressing Escape, or the panel leaving the layout during the
  drag cancels it: the layout is unchanged and the indicator and dimming go away.
- **WIDGETS-DOCKDRAG-07:** On a focused strip, Control+Shift+ArrowLeft / ArrowRight move
  the active tab one place left or right within its group (`move_panel` with a `Tabs`
  target), stopping at the ends (no wrapping); the tab stays active and focused.
- **GALLERY-28:** The Docking section's hint mentions dragging tabs. A gallery test drags
  Terminal onto the editor strip (it joins Editor and Readme), then drags Problems to the
  right edge of the editor content (a new group beside it), checking the echo and the
  saved text, and a golden `gallery-docking-drag.png` taken mid-drag shows a zone
  indicator.

## Performance and allocation

Target computation is O(groups + tabs in the strip under the pointer) per pointer move,
using rects and tab places recorded at the last frame; the indicator is a repaint of the
dock only. No allocation per move beyond the indicator state.

## Open questions

Proposals, to accept or change:

1. **The dock root follows the drag** in its capture phase (it is on the captured pointer
   path, and sees positions in its own coordinates), so tabs only start it.
2. **Five zones per group, quarter bands, plus 16 px dock-edge bands** (the Visual Studio
   and VS Code arrangement, without the floating "compass" buttons).
3. **Indicators drawn by the dock** over its children: a drop line on strips, a 16 %
   fill with an outline for zones; the dragged tab dimmed rather than a ghost image.
4. **No-op drops show nothing**, so an indicator always means the layout will change.
5. **Control+Shift+Left/Right** reorder tabs from the keyboard; moving panels between
   groups by keyboard waits for menus ("Move to group…").
6. **Tested through the gallery** with a mid-drag golden, besides widget tests per rule.
