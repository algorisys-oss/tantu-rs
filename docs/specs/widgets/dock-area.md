# Docking, part 2: the DockArea widget

- **Status:** Agreed (2026-10-16, the user said "continue" on the seven proposals)
- **Crates:** `tantu-layout` (`RenderDock`), `tantu-widgets` (`DockArea`), `examples/gallery`
- **Plan item:** Phase 4, "Docking layout manager", part 2 (`DockArea`)
- **Related:** [dock layout model](dock-layout.md) (part 1), [SplitView](split-view.md),
  [TabView](tab-view.md), [Offstage](../view/offstage.md), [focus](../view/focus.md),
  [shortcuts](../view/shortcuts.md), [semantics](../a11y/semantics.md),
  [cursor icons](../platform/cursor.md), [gallery](../examples/gallery.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

`DockArea` shows a `DockLayout` (part 1) and lets the user work with it: each tab group
gets a strip of document tabs with close buttons, the active panel's content fills the
rest of the group, and dividers between groups resize them. Every change the user makes
goes back into the app's `Signal<DockLayout>`, so the app sees and persists it, and every
change the app makes (opening a panel, resetting the layout) shows at the next frame.

Panels keep their state for as long as they are in the layout: switching tabs hides a
panel without disposing it, and moving a panel to another group (by code now, by dragging
in part 3) keeps its fields, scroll offsets and selections.

## Scope

In scope: laying out the tree (groups, strips, panel content, dividers) in one render
object; building panel content and titles through app callbacks; tab activation and
closing (with an optional veto); divider resizing; keyboard navigation and shortcuts; the
active group; focus when panels hide or close; semantics; a gallery section and a
kitchen-sink card.

Out of scope (and where it goes): dragging tabs to reorder or dock them, and drop
indicators (part 3); floating panels (multi-window); tab overflow menus, pinned tabs,
maximizing a group and auto-hide panels (later, if wanted); context menus on tabs (the
menus item); RTL mirroring (i18n).

## Public API

```rust
// tantu-layout
/// The geometry of a dock layout: node 0 is the root; groups are leaves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DockPlan { pub nodes: Vec<DockPlanNode> }
#[derive(Clone, Debug, PartialEq)]
pub enum DockPlanNode {
    Group,
    Split { axis: Axis, fractions: Vec<f32>, children: Vec<usize> },
}

/// Which part of the plan a child of a `RenderDock` is (parent data).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockSlot {
    /// The tab strip of group node `node`.
    Strip(usize),
    /// A panel's content in group node `node` (only the active panel is onstage).
    Panel(usize),
    /// The divider between children `index` and `index + 1` of split node `node`.
    Divider { node: usize, index: usize },
}

/// Lays out a dock plan: groups sized as split panes, each with a strip on top.
#[derive(Clone, Debug)]
pub struct RenderDock {
    pub plan: Arc<DockPlan>,
    pub strip_height: f32, // 36
    pub min_group: f32,    // 80, on every split axis
    pub gap: f32,          // 1
    pub handle: f32,       // 8
    // private: the last layout's node rects (not part of equality)
}
impl RenderDock {
    pub fn new(plan: Arc<DockPlan>) -> Self;
    /// Each plan node's rect from the last layout, indexed like `plan.nodes` (empty before).
    pub fn node_rects(&self) -> &[Rect];
}

// tantu-widgets
/// Shows a dock layout and lets the user change it.
pub struct DockArea { /* private */ }
impl DockArea {
    /// `content` builds a panel's view the first time it appears in the layout.
    pub fn new<V: View>(layout: Signal<DockLayout>, content: impl Fn(&PanelId) -> V + 'static) -> Self;
    /// A tab's title (default: the panel id). Reactive: read inside a closure.
    pub fn title(self, title: impl Fn(&PanelId) -> String + 'static) -> Self;
    /// Whether a panel shows a close button and can be closed (default: all).
    pub fn closable(self, closable: impl Fn(&PanelId) -> bool + 'static) -> Self;
    /// Asked before the user closes a panel; false keeps it (unsaved changes). Default: close.
    pub fn on_close_request(self, f: impl Fn(&PanelId) -> bool + 'static) -> Self;
    /// Shares the active group (the one the user last worked in) with the app.
    pub fn active_group(self, group: Signal<Option<DockGroupId>>) -> Self;
    /// What to show when the layout is empty (default: nothing).
    pub fn placeholder<V: View>(self, view: impl Fn() -> V + 'static) -> Self;
}
```

`DockArea` implements `View` and is in the facade prelude. No new dependencies.

## Behavior

### Layout (`RenderDock`)

- **LAYOUT-DOCK-01:** On bounded axes the dock takes the maximum size; an unbounded axis
  takes 0 with a debug `tracing` warning. Node 0 gets the whole rect. A split's children
  share its main extent as `RenderSplit` panes do (LAYOUT-SPLIT-01, -02) with `gap` between
  neighbours, every child's minimum being `min_group` (no collapsing), and each child gets
  the split's full cross extent. An empty plan lays out nothing.
- **LAYOUT-DOCK-02:** A group's `Strip` child is laid out tight at the group's width ×
  `strip_height` (clamped to the group's height) at its top; its `Panel` children tight
  at the rest of the group below the strip. Children are placed at their rects; offstage
  ones (inactive panels) are absent from layout (VIEW-OFFSTAGE-02).
- **LAYOUT-DOCK-03:** A `Divider { node, index }` child is laid out tight at `handle` ×
  the split's cross extent, centred on the gap after child `index`, oriented along the
  split's axis. Dividers come after the other children so they win the hit test.
- **LAYOUT-DOCK-04:** A child whose slot names a missing node, a non-group for
  `Strip`/`Panel`, or a non-split or out-of-range boundary for `Divider` gets zero size at
  the origin. Nothing panics for malformed plans (cycles are impossible: children indices
  must be greater than their parent's, others are ignored).
- **LAYOUT-DOCK-05:** `node_rects` returns every node's rect in the dock's coordinates
  after a layout, splits included (part 3 hit-tests drop zones with them).

### Widget (`DockArea`)

- **WIDGETS-DOCKAREA-01:** The area mirrors the layout signal: one strip per group, keyed by
  `DockGroupId`; one content host per panel, keyed by `PanelId`, holding
  `content(&id)` built the first time the panel appears; the active panel's host is
  onstage and the others offstage. A panel keeps its content (same elements, same state)
  while it stays in the layout, whether it becomes inactive or moves to another group; a
  panel that leaves the layout is disposed, and built again if it comes back. Layout
  changes rebuild no other panel.
- **WIDGETS-DOCKAREA-02:** Look (from the theme): the strip has a
  `surface_container_highest` background; each tab shows its title in the `label` style
  with 12 px horizontal padding, `on_surface` when active and `on_surface_variant`
  otherwise, on a `surface` background when active; the active tab has a 2 px bottom
  indicator in `primary` when its group is the active group, else `outline`; closable
  tabs show a 16 px close icon (`Icons::CLOSE`) when active or hovered (others keep the
  space, so tabs don't move on hover). Hovered tabs and close buttons get an `on_surface`
  layer at 8 %. When the tabs are wider than the strip, the strip scrolls horizontally
  (wheel, no scrollbar) and reveals the active tab whenever it changes.
- **WIDGETS-DOCKAREA-03:** Pointer: a primary press and release on a tab activates its
  panel and focuses the strip; a press on its close button, or a middle-button release on
  a closable tab, requests closing. A close request calls `on_close_request` (if set) and
  closes the panel in the layout only when it returns true. Pressing anywhere in a group
  (strip or content) makes it the active group.
- **WIDGETS-DOCKAREA-04:** The active group (shared via `active_group` or owned) is the
  group the user last pressed in or moved focus into; it starts as the first group and,
  when its group leaves the layout, falls back to the first group (`None` when empty).
- **WIDGETS-DOCKAREA-05:** Keys: each strip is one focus stop; Left/Right activate the
  previous/next tab (wrapping), Home/End the first/last. From anywhere inside the area,
  Control+Tab / Control+Shift+Tab and Control+PageDown / Control+PageUp activate the
  next/previous tab of the active group (wrapping), and Control+W / Control+F4 request
  closing its active panel. Modified keys the area doesn't bind pass through.
- **WIDGETS-DOCKAREA-06:** Dividers behave as `SplitView` dividers (WIDGETS-SPLIT-03, -05,
  -06, -07, without collapsing or Enter): dragging or keys move the boundary between two
  neighbours, clamped at `min_group`, and write the split's new fractions with
  `DockLayout::set_fractions` through the layout signal.
- **WIDGETS-DOCKAREA-07:** When the focus is inside a panel that becomes inactive or is
  closed, it moves to that group's strip if the group remains, else is cleared; a leaving
  field's IME is disabled. Other focus is unchanged.
- **WIDGETS-DOCKAREA-08:** Semantics: each strip is a `TabList`; each tab a `Tab` named by
  its title, `selected` when active, with one-based position and set size, `Click` and
  `Focus`; each close button a `Button` named "Close <title>"; the onstage panel host a
  `TabPanel` named by its title; dividers `Splitter`s named "Resize <title of the active
  panel before it>" with the boundary's position as a percentage (WIDGETS-SPLIT-08).
- **WIDGETS-DOCKAREA-09:** An empty layout shows the `placeholder` view filling the area
  (nothing without one); it is built when the layout becomes empty and disposed when a
  panel arrives.

### Gallery

- **GALLERY-26:** A "Docking" section before Split view (thirteen sections): a `DockArea` (key `dock`, 720 ×
  400) on the IDE layout (Explorer, Search | Editor / Terminal, Problems) with simple
  per-panel content (the editor a `TextField`), Explorer not closable, "Reset layout",
  "Save" and "Restore" buttons (Restore loads the last saved text), and an echo
  (`dock-echo`) "Active: <title> · Panels: <count>". Tests: clicking the Search tab
  activates it; text typed in the editor survives switching to another tab and back;
  closing Problems updates the echo; Save, close Terminal, Restore brings
  it back. Golden `gallery-docking.png` at 960 × 640.
- **GALLERY-27:** A "dock" kitchen-sink card after "split" (sixteen `SINK_CARDS` entries, sample `dock_area`): a 600 ×
  220 `DockArea` of two groups (Notes | Tasks, Log), echoed by `sink-dock-state`.

## Performance and allocation

Activating a tab or resizing writes the layout signal; the plan is rebuilt from the
layout (O(nodes)) and no panel is rebuilt. Inactive panels cost nothing per frame
(offstage). No allocation per frame beyond the existing widgets'.

## Open questions

Accepted by the user ("continue", 2026-10-16):

1. **One flat render object (`RenderDock`)** holding every strip, panel host and divider,
   positioned from a plan, so a panel moving between groups keeps its elements and state
   (the view tree cannot reparent); inactive panels are offstage.
2. **App callbacks keyed by `PanelId`** for content, titles and closability, plus an
   `on_close_request` veto for unsaved changes.
3. **Compact document tabs** (36 px strip, close icon on the active and hovered tabs,
   scrolling strips) rather than reusing `TabBar`'s 48 px Material tabs; an overflow menu
   waits for the menus item.
4. **An active group** tracked from presses and focus and shareable with the app, so
   "open file" commands can target it.
5. **Keyboard:** strips as single focus stops with arrows; Control+Tab /
   Control+PageDown cycling and Control+W / Control+F4 closing from anywhere in the area.
6. **Group minimum 80 px**, dividers as `SplitView`'s without collapsing.
7. **Gallery "Docking" section and a kitchen-sink card**; dragging tabs (reorder and dock)
   is part 3.
