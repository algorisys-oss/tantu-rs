# Docking, part 1: the dock layout model and persistence

- **Status:** Agreed (2026-10-16, the user said "continue" on the six proposals)
- **Crate:** `tantu-widgets`
- **Plan item:** Phase 4, "Docking layout manager", part 1 (the model)
- **Related:** [SplitView](split-view.md) (fraction cleaning, LAYOUT-SPLIT-01),
  [TabView](tab-view.md), part 2 (the `DockArea` widget) and part 3 (drag-to-dock), both to
  be specified after this one

## Purpose

IDEs, trading screens and admin consoles let users arrange panels: tabs grouped together,
groups split side by side or stacked, panels dragged from one group to another, and the
arrangement restored the next time the app starts. This part defines the arrangement
itself as plain data, a `DockLayout`: a tree of splits whose leaves are tab groups of
panels. Every change (activating, closing, adding, moving a panel, resizing a split) is an
operation on that value that keeps the tree in a normal form, and the layout saves to and
loads from a small text format so apps can persist it.

Keeping the model separate from the widget lets the operations be tested exhaustively
without a window, lets apps build and change layouts in code (a "Reset layout" command, a
"Show terminal" menu item), and gives part 3's drag-and-drop a single function to call
when a drop lands.

## Scope

In scope: the tree (splits and tab groups), app-chosen panel ids, group and split ids for
the widget, construction, queries, the operations, the normal form, and the text format
with error reporting.

Out of scope (and where it goes): drawing the layout, tab strips, divider dragging and
panel content (part 2, `DockArea`); dragging tabs and drop indicators (part 3); floating
panels in their own OS windows (the multi-window item); pinned/auto-hide side panels and
maximizing a group (later, if wanted); panel state beyond its id (the app keeps it, e.g.
in signals keyed by panel id).

## Public API

All items are exported from `tantu-widgets` and the facade prelude.

```rust
/// A panel's identity, chosen by the app ("explorer", "editor:src/main.rs"). Stable across
/// runs: it is what the text format stores.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PanelId(Arc<str>);
impl PanelId { pub fn new(id: impl Into<Arc<str>>) -> Self; pub fn as_str(&self) -> &str; }
impl From<&str> for PanelId; impl From<String> for PanelId;

/// A tab group's or a split's identity within one layout (the widget keys elements by it).
/// Not persisted; equality of layouts ignores it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DockGroupId(/* private */);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DockSplitId(/* private */);

/// A description of a layout, for building one in code.
#[derive(Clone, Debug, PartialEq)]
pub enum DockNode {
    /// A tab group: its panels in order and the index of the active one.
    Tabs { panels: Vec<PanelId>, active: usize },
    /// Children side by side (`Axis::Horizontal`) or stacked, with their fractions.
    Split { axis: Axis, children: Vec<(f32, DockNode)> },
}
impl DockNode {
    /// A group of `panels`, the first active.
    pub fn tabs<P: Into<PanelId>>(panels: impl IntoIterator<Item = P>) -> Self;
    /// Children side by side, each with its fraction.
    pub fn row(children: impl IntoIterator<Item = (f32, DockNode)>) -> Self;
    /// Children stacked top to bottom, each with its fraction.
    pub fn column(children: impl IntoIterator<Item = (f32, DockNode)>) -> Self;
}

/// Where a panel goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockTarget {
    /// Into a group's tabs at `index` (clamped to the end).
    Tabs { group: DockGroupId, index: usize },
    /// A new group beside `group`, on `side`.
    Beside { group: DockGroupId, side: DockSide },
    /// A new group along one edge of the whole layout.
    Edge(DockSide),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockSide { Left, Right, Top, Bottom }

/// Why an operation changed nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DockError {
    UnknownPanel(PanelId),
    UnknownGroup(DockGroupId),
    UnknownSplit(DockSplitId),
    /// Moving a group's only panel beside its own group.
    NoOp,
}

/// Why a saved layout couldn't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DockLoadError {
    /// The first line isn't `tantu-dock 1`.
    Version,
    /// Malformed text at this byte offset.
    Syntax { offset: usize, expected: &'static str },
}

/// One node of the normalized tree, for reading (the widget walks it).
pub enum DockView<'a> {
    Tabs { id: DockGroupId, panels: &'a [PanelId], active: usize },
    Split { id: DockSplitId, axis: Axis, fractions: &'a [f32], children: Vec<DockView<'a>> },
}

/// The arrangement: a normalized tree of splits and tab groups (possibly empty).
#[derive(Clone, Debug)]
pub struct DockLayout { /* private */ }

impl DockLayout {
    /// The empty layout.
    pub fn empty() -> Self;
    /// A layout from a description, normalized (WIDGETS-DOCK-01).
    pub fn new(root: DockNode) -> Self;

    // Queries
    pub fn is_empty(&self) -> bool;
    pub fn root(&self) -> Option<DockView<'_>>;
    /// Every panel, in tree order (depth first, groups' tabs in order).
    pub fn panels(&self) -> Vec<PanelId>;
    pub fn contains(&self, panel: &PanelId) -> bool;
    pub fn group_of(&self, panel: &PanelId) -> Option<DockGroupId>;
    /// A group's panels and active index.
    pub fn group(&self, group: DockGroupId) -> Option<(&[PanelId], usize)>;
    pub fn active(&self, group: DockGroupId) -> Option<&PanelId>;
    /// The groups in tree order (the first is the default target for new panels).
    pub fn groups(&self) -> Vec<DockGroupId>;

    // Operations: each keeps the normal form (WIDGETS-DOCK-02).
    /// Makes `panel` its group's active tab.
    pub fn activate(&mut self, panel: &PanelId) -> Result<(), DockError>;
    /// Removes `panel` from the layout.
    pub fn close(&mut self, panel: &PanelId) -> Result<(), DockError>;
    /// Puts a new panel at `target`; a panel already in the layout is moved instead.
    pub fn add(&mut self, panel: impl Into<PanelId>, target: DockTarget) -> Result<(), DockError>;
    /// Moves a panel that is in the layout to `target`.
    pub fn move_panel(&mut self, panel: &PanelId, target: DockTarget) -> Result<(), DockError>;
    /// Sets a split's fractions (cleaned as LAYOUT-SPLIT-01).
    pub fn set_fractions(&mut self, split: DockSplitId, fractions: &[f32]) -> Result<(), DockError>;

    // Persistence (WIDGETS-DOCK-10, -11)
    pub fn save(&self) -> String;
    /// Reads a saved layout, keeping only the panels `keep` accepts (a panel the app no
    /// longer has is dropped), then normalizes.
    pub fn load(text: &str, keep: impl Fn(&PanelId) -> bool) -> Result<DockLayout, DockLoadError>;
}

/// Compares the trees (axes, fractions, panels, active tabs), ignoring ids.
impl PartialEq for DockLayout;
impl Default for DockLayout; // empty
```

Apps hold a layout in a `Signal<DockLayout>` and change it with
`layout.update(|l| { l.move_panel(&id, target).ok(); })`; part 2's `DockArea` takes that
signal. No new dependencies or feature flags.

## Behavior

### Normal form

- **WIDGETS-DOCK-01:** `DockLayout::new` normalizes its description: a panel id appearing
  more than once is kept only at its first position in tree order; groups left without
  panels are removed; `active` is clamped to the group's last panel; each split's fractions
  are cleaned as LAYOUT-SPLIT-01 (negative or non-finite count as 0, normalized to sum to 1,
  all zero meaning equal shares) after its removed children's fractions are dropped. A
  description with no panels gives the empty layout. Nothing panics for any description.
- **WIDGETS-DOCK-02:** After construction, every operation and `load`, the layout is in
  normal form: no empty group; every split has at least two children; no split has a child
  split on the same axis (such a child is flattened into its parent, its fractions scaled by
  its own fraction); every split's fractions sum to 1 (within 1e-5) and are each ≥ 0; each
  panel appears once; each group's `active` is in range. A split left with one child is
  replaced by that child. An operation that fails leaves the layout unchanged.
- **WIDGETS-DOCK-03:** Every group and split has an id unique within the layout. Groups and
  splits that survive an operation keep their ids (a group keeps its id when panels move in
  or out of it, and when it moves up the tree because its parent split was replaced or
  flattened); new nodes get ids never used before by this layout value or the value it was
  cloned from. `clone` keeps ids. `==` ignores ids and compares fractions within 1e-5 (so
  layouts reached by different operation orders compare equal despite rounding), so
  `load(save(l)) == l`.

### Queries and operations

- **WIDGETS-DOCK-04:** Queries read the normalized tree: `panels` and `groups` in tree order
  (depth first, children in order); `group_of`, `group` and `active` return `None` for
  unknown ids; `root` describes the tree with fractions and ids.
- **WIDGETS-DOCK-05:** `activate` makes the panel its group's active tab;
  `UnknownPanel` if it isn't in the layout. Activating the active panel succeeds and changes
  nothing.
- **WIDGETS-DOCK-06:** `close` removes the panel. If it was active, the next tab (the one to
  its right) becomes active, or the previous one if it was last; an inactive panel's
  closing keeps the same panel active. A group left empty is removed and its share goes to
  its siblings in proportion to their fractions; then WIDGETS-DOCK-02 applies (a split left
  with one child is replaced by it). Closing the last panel gives the empty layout.
  `UnknownPanel` if it isn't in the layout.
- **WIDGETS-DOCK-07:** `Tabs { group, index }` inserts the panel into the group's tabs at
  `index`, clamped to the end, where the index counts the group's tabs without the moving
  panel; the panel becomes active. Moving a panel within its own group reorders it. If the
  panel's old group becomes empty (and isn't the target), it is removed as in
  WIDGETS-DOCK-06.
- **WIDGETS-DOCK-08:** `Beside { group, side }` puts the panel in a new group next to
  `group`: when `group`'s parent split runs along the side's axis (Left/Right horizontal,
  Top/Bottom vertical), the new group is inserted before (Left/Top) or after (Right/Bottom)
  it in that split and takes half of `group`'s fraction; otherwise `group` is replaced by a
  new split on that axis holding the two groups at 0.5 each (the new group first for
  Left/Top). The panel is removed from its old place first (WIDGETS-DOCK-06), and a target
  group removed by that is an error (`NoOp`): moving a group's only panel beside its own
  group changes nothing.
- **WIDGETS-DOCK-09:** `Edge(side)` puts the panel in a new group along that edge of the
  whole layout: when the root is a split along the side's axis, the new group becomes its
  first (Left/Top) or last child with fraction 0.25 and the others are scaled by 0.75;
  otherwise the root is replaced by a split holding the new group (0.25) and the old root
  (0.75), in edge order. In an empty layout any target that names no group (`Edge`) creates
  the root group; `Tabs`/`Beside` naming an unknown group fail with `UnknownGroup`.
  `add` of a panel already in the layout behaves as `move_panel`; `move_panel` of an
  unknown panel fails with `UnknownPanel`.
- **WIDGETS-DOCK-10:** `set_fractions` cleans the given fractions as LAYOUT-SPLIT-01 against
  the split's child count (missing entries count as an equal share) and stores them;
  `UnknownSplit` for an unknown id.

### Persistence

- **WIDGETS-DOCK-11:** `save` writes the first line `tantu-dock 1`, then the tree in this
  grammar on a second line ending with a newline (a single space between items):

  ```text
  layout := "empty" | node
  node   := "tabs(" active (" " panel)+ ")"
          | ("row(" | "column(") fraction " " node (" " fraction " " node)+ ")"
  panel  := '"' (any character, with '"' and '\' escaped by '\') '"'
  ```

  Fractions use Rust's shortest round-trip `f32` formatting, so `load(save(l), |_| true)`
  equals `l`. Example:
  `tantu-dock 1` / `row(0.2 tabs(0 "explorer" "search") 0.8 column(0.7 tabs(0 "editor") 0.3 tabs(1 "terminal" "problems")))`.
- **WIDGETS-DOCK-12:** `load` accepts any amount of whitespace between items and a final
  newline, and fails with `Version` when the first line isn't `tantu-dock 1`, or `Syntax`
  with the byte offset of the first unexpected character and what was expected there (one
  of the strings `"node"`, `"fraction"`, `"panel"`, `")"`, `"end of input"`). The parser
  is lenient where normalizing can repair the input: a split with one child and a group
  with no panels are accepted. Panels `keep` rejects are dropped,
  and the result is normalized as `new` (out-of-range `active`, bad fractions, duplicates
  and empty groups are repaired, not errors). No input panics, including empty text, deep
  nesting (the parser is iterative or bounded at 256 levels, returning `Syntax` beyond) and
  invalid UTF-8 boundaries in escapes.

## Performance and allocation

Operations and queries are O(nodes); layouts hold tens of panels, so clarity wins over
allocation here. `save`/`load` are linear in the text length.

## Open questions

Accepted by the user ("continue", 2026-10-16):

1. **Three parts:** this model; part 2, a `DockArea` widget drawing the layout (nested
   splits through `RenderSplit`, dynamic tab strips with close buttons, panel content kept
   alive when a panel moves between groups, keyboard and semantics); part 3, dragging tabs
   with drop indicators onto `DockTarget`s. Floating panels wait for multi-window.
2. **The layout is a plain value** in a `Signal<DockLayout>` that the app owns and may
   change in code, rather than a controller object; the widget writes the same signal.
3. **App-chosen string panel ids** (persistable, readable in saved text); the widget will
   build a panel's content and title from the id through app callbacks.
4. **Normal form flattens same-axis splits**, so a layout has one canonical shape (simpler
   persistence, equality and dividers).
5. **Docking beside a group halves it; docking at an edge takes a quarter**; closing gives
   the share to the siblings proportionally; closing the active tab activates its right
   neighbour (Visual Studio and JetBrains behavior), not the most recently used one.
6. **An in-house versioned text format** rather than a serde dependency: no new crate,
   human-readable, stable across Tantu versions through the version line.
