# i18n, part 1b: right-to-left inside widgets

- **Status:** Agreed (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (`Switch`, `Slider`, `ScrollView`, `TreeView`, menus,
  `ComboBox`, `CalendarDatePicker`), `examples/gallery`
- **Plan item:** Phase 4, i18n, part 1 follow-up (RTL layout mirroring)
- **Related:** [directionality](directionality.md), [toggles](toggles.md),
  [slider](slider.md), [scroll](scroll.md), [tree view](tree-view.md), [menus](menu.md),
  [combo box](combo-box.md), [date picker](date-picker.md)

## Purpose

Part 1 mirrored the layout widgets, so every `Row` and `Column` already runs right to left
under `Directionality(Rtl)`. Widgets that paint their own geometry, place children
themselves or map Left and Right keys still behave left to right. Flutter mirrors these
inside each widget (`Switch`, `Slider`, `Scrollbar`, `TreeSliver` indentation, cascading
menus, directional icons). This part does the same for the affected Tantu widgets, each
reading the nearest direction and following it when it changes.

## Scope

In scope: the switch's thumb; the slider's track, pointer mapping and arrow keys; the
vertical scrollbar's side; the tree view's indentation, expander and arrow keys;
submenus' side, indicator and arrow keys, and the menu bar's arrow keys; the combo box's
arrow; the calendar's month chevrons.

Out of scope: text editing (caret movement over bidirectional text, which follows the
paragraph's visual order and needs its own spec); paragraph alignment; the data grid,
split view and dock (their own layouts; later amendments); the checkbox's check mark and
radio dot (symmetric).

## Behavior

Each rule applies under the nearest `Directionality` being `Rtl` and is reversed again
when it becomes `Ltr`.

- **WIDGETS-RTL-01:** `Switch`: the thumb sits at the left when on and at the right when
  off.
- **WIDGETS-RTL-02:** `Slider`: the minimum is at the right end: the active track fills
  from the right, the thumb's position and the value under a press or drag are mirrored,
  ArrowLeft increases and ArrowRight decreases (ArrowUp and ArrowDown are unchanged).
- **WIDGETS-RTL-03:** `ScrollView`: a vertical scrollbar is painted, and dragged, at the
  left edge.
- **WIDGETS-RTL-04:** `TreeView`: rows are indented from the right edge with the expander
  at the start (right) of the content; ArrowLeft expands a collapsed node or moves to its
  first child, ArrowRight collapses an expanded node or moves to its parent.
- **WIDGETS-RTL-05:** Menus: a submenu opens to the left of its item (its top-right corner
  at the item's top-left corner, flipping as before when there is no room) and the
  submenu indicator is `CHEVRON_LEFT`; ArrowLeft opens a submenu and ArrowRight closes it
  (or moves to the previous menu of a menu bar); in a menu bar, ArrowLeft moves to the
  next menu and ArrowRight to the previous one.
- **WIDGETS-RTL-06:** `ComboBox`: the drop-down arrow sits at the left end of the field.
  `CalendarDatePicker`: the previous-month button shows `CHEVRON_RIGHT` and the
  next-month button `CHEVRON_LEFT`.
- **GALLERY-48:** With the gallery's RTL switch on, the header's switches show their
  thumbs mirrored and the sidebar's scrollbar is at its left edge; the `gallery-rtl.png`
  golden shows it.

## Performance and allocation

Nothing new per frame; the direction is read through the same bindings as the widgets'
other props.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **Flutter's per-widget mirroring**, rather than mirroring painted output wholesale
   (a mirrored transform would flip text and icons too).
2. **Directional arrow keys** in sliders, trees and menus follow Flutter (and the
   WAI-ARIA practices for RTL), so Left always points toward "more" in RTL sliders.
