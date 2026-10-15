# TreeView: expansion, selection and virtualized rows

- **Status:** Implemented (agreed 2026-10-12, the user said "implement it and also update kitchen sink with this example"; implemented 2026-10-15)
- **Crates:** `tantu-widgets`, `tantu-view` and `tantu-a11y` (tree semantics),
  `examples/gallery`
- **Plan item:** Phase 4, "TreeView"
- **Related:** [ListView](list-view.md), [list extents and navigation](list-view-advanced.md),
  [scroll](scroll.md), [focus](../view/focus.md), [semantics](../a11y/semantics.md),
  [gallery](../examples/gallery.md), [kitchen sink](../examples/kitchen-sink.md),
  [ADR 0013](../../adr/0013-accessibility-pipeline.md)

## Purpose

Display project explorers, navigation hierarchies and other ordered trees without creating
a widget for every node. Applications supply stable identities and queryable children;
TreeView owns the viewport, disclosure controls, selection paint and desktop navigation.
Expansion and selection can be shared with the application through ordinary signals.

## Scope

In scope: a synchronous data-source trait, stable node IDs, fixed-height vertical rows,
an expanded preorder index, viewport virtualization, indentation, disclosure controls,
single selection, activation, shared scrolling, model revisions, themed paint, tree
semantics and accessibility actions, a gallery section and a kitchen-sink sample.

Out of scope: multiple/range selection, checkbox trees, drag/reorder, editing, filtering,
sorting, asynchronous loading UI, typeahead, variable/measured heights, horizontal scrolling,
automatic expansion of a hidden externally selected node, animated expansion, and persistence
formats. Apps may load children themselves and then bump `revision`; this feature adds no
platform or renderer dependency, crate, pipeline stage or new ADR.

## Public API

All widget items below are exported from `tantu-widgets` and the facade prelude.

```rust
/// Stable, application-assigned identity, unique throughout one source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeNodeId(pub u64);

/// An ordered forest. Queries must be cheap, synchronous and mutually consistent until
/// the next revision; IDs must not be reused for unrelated nodes during the view's lifetime.
pub trait TreeDataSource: 'static {
    /// Number of roots.
    fn root_count(&self) -> usize;
    /// Root at a zero-based index, or None for a missing entry.
    fn root(&self, index: usize) -> Option<TreeNodeId>;
    /// Number of immediate children; zero for a leaf or unknown node.
    fn child_count(&self, node: TreeNodeId) -> usize;
    /// Immediate child at a zero-based index, or None for a missing entry.
    fn child(&self, node: TreeNodeId, index: usize) -> Option<TreeNodeId>;
    /// Display/accessibility label; empty for an unknown node.
    fn label(&self, node: TreeNodeId) -> String;
}

/// Snapshot supplied to a custom row-content builder; chrome remains TreeView's job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeRowState {
    /// Stable identity of this row.
    pub node: TreeNodeId,
    /// Zero-based depth (roots are zero).
    pub depth: usize,
    /// Whether the source reports children.
    pub expandable: bool,
    /// Whether this expandable node is open.
    pub expanded: bool,
    /// Whether this is the selection.
    pub selected: bool,
}

/// A vertically scrolling, virtualized tree with a single keyboard focus stop.
pub struct TreeView { /* private */ }
impl TreeView {
    /// Uses Text for node labels; defaults to 32 px rows and 20 px indentation.
    pub fn new(source: Rc<dyn TreeDataSource>) -> Self;
    /// Replaces label content only. Called for mounted rows and again when their state
    /// or model revision changes; content should not introduce additional focus stops.
    pub fn builder<V: View>(
        source: Rc<dyn TreeDataSource>,
        build: impl Fn(TreeRowState) -> V + 'static,
    ) -> Self;
    /// Row height; a non-positive or non-finite value falls back to 32 px.
    pub fn item_extent(self, extent: f32) -> Self;
    /// Per-depth indentation; a negative or non-finite value falls back to 20 px.
    pub fn indentation(self, indentation: f32) -> Self;
    /// Shared expansion state; otherwise an empty owner-scoped signal is created.
    pub fn expanded(self, expanded: Signal<BTreeSet<TreeNodeId>>) -> Self;
    /// Shared single selection; otherwise an owner-scoped None signal is created.
    pub fn selection(self, selection: Signal<Option<TreeNodeId>>) -> Self;
    /// Refreshes structure and mounted labels/content when the value changes; default 0.
    pub fn revision(self, revision: impl IntoProp<u64>) -> Self;
    /// Shares the vertical scroll controller; otherwise uses a private controller.
    pub fn controller(self, controller: ScrollController) -> Self;
    /// Called with the selected ID on a non-repeated Enter press.
    pub fn on_activate(self, activate: impl Fn(TreeNodeId) + 'static) -> Self;
}

// Additions to tantu-view; other Semantics fields and enum variants remain available.
pub enum Role { /* existing variants, */ Tree, TreeItem }
pub struct Semantics {
    /* existing fields, */
    /// Open/closed for an expandable tree item; None for a leaf.
    pub expanded: Option<bool>,
    /// One-based tree depth, sibling position and sibling count.
    pub level: Option<usize>,
    pub position_in_set: Option<usize>,
    pub size_of_set: Option<usize>,
}
pub struct SemanticsActions { /* existing fields, */ pub expand: bool, pub collapse: bool }
pub enum SemanticsAction { /* existing variants, */ Expand, Collapse }
```

`TreeView` implements `View`. `TreeNodeId(0)` is valid; no sentinel ID is reserved.
`Rc`, `BTreeSet`, `Signal`, `IntoProp`, `ScrollController` and `View` retain their existing
types. There are no new feature flags.

## Behavior

- **WIDGETS-TREE-01:** With no expansion, roots appear in source order at depth zero.
  Expanding a branch inserts its children immediately after it in depth-first preorder;
  closed branches contribute just their own row. Children of closed nodes are never queried
  through `child`. Leaves show no disclosure indicator even if their IDs are in `expanded`.
  Empty sources produce an empty scrollable tree with no row or activation.
- **WIDGETS-TREE-02:** Rows fill the finite viewport width and have exactly `item_extent`
  height. Content starts after `depth * indentation` and a 20 px disclosure slot, including
  that slot for leaves. Defaults and invalid-value fallbacks follow the API; zero indentation
  is allowed. Excess indentation clamps to available width, leaving no negative constraints
  or non-finite positions. The viewport clips content and reuses ScrollView's sizing,
  wheel, scrollbar and controller behavior, including its existing unbounded-constraint
  behavior. Scrolling and layout build at most the intersecting rows plus one buffered row
  on each side; labels and custom content are queried only for those mounted rows.
- **WIDGETS-TREE-03:** A primary press in a branch's disclosure slot focuses the tree,
  selects that node and toggles only its expansion membership, once per press. A press
  elsewhere in a row selects it and focuses the tree without expansion or activation.
  Leaf disclosure slots select only; empty viewport space does not select a row; secondary
  presses do not change state. Chrome does not add a separate Tab stop. Custom content may
  handle its own pointer events; disclosure presses are consumed before content handlers.
- **WIDGETS-TREE-04:** While the tree itself has focus, unmodified Up/Down select the previous/
  next flattened row, clamped; from None, Down selects first and Up last. Home/End select
  first/last. PageUp/PageDown move by `max(1, floor(viewport / item_extent))`, clamped, with
  None starting at first/last respectively. Changes jump the least distance needed to reveal
  the full row; oversized rows align their start. Tab traverses normally; key releases and
  Control/Command/Alt/Shift combinations pass through. Navigation repeats are supported.
- **WIDGETS-TREE-05:** Right expands a closed selected branch; on an already open branch it
  selects its first displayed child; leaves do nothing. Left collapses an open selected
  branch, otherwise selects its displayed parent; roots do nothing. With None, Right/Left
  select the first row without expanding. Enter calls `on_activate` once on a non-repeated
  press with a displayed selection, on either branch or leaf, without toggling expansion;
  repeated Enter is consumed without activating. Space toggles the selected branch once
  on a non-repeated press; leaves do nothing. No double-click activation is added in this
  first feature, so disclosure clicks cannot accidentally activate a node.
- **WIDGETS-TREE-06:** If a collapse (pointer, keyboard or external expansion change) hides
  the selection, select its nearest remaining displayed ancestor. Selection of the branch
  itself is unchanged by collapsing it. Descendant expansion IDs remain in the set, so
  re-expanding restores their open states. An externally selected ID not in the displayed
  index becomes None at the next frame; it does not scan or open hidden branches. A valid
  external selection is revealed. Selection normalization never invokes activation.
- **WIDGETS-TREE-07:** A revision change rebuilds the displayed index and refreshes mounted
  labels/custom content even when IDs and counts are unchanged. IDs keep selection through
  sibling reorder or insertion. If a selected node disappears or becomes hidden through a
  revision, selection becomes None; no whole-source search is made. Unseen expansion IDs
  remain harmless in the set (the application may prune them). A shorter tree clamps the
  scroll offset. Changes to `expanded` rebuild structure; selection, hover and scrolling
  alone do not re-enumerate the model. ScrollView's fallback scrolling keys also ignore
  modified chords so they can reach app shortcuts (WIDGETS-SCROLL-06 amendment). Removing the view disposes its internal state and
  handlers; externally supplied signals survive with their owning application scope.
- **WIDGETS-TREE-08:** Surviving mounted rows retain their wrapper element identity by
  TreeNodeId when rows are inserted, removed or reordered above them; their position/depth
  updates. Mounted content may rebuild when state/revision changes. Rows leaving the buffered
  viewport are disposed, including their reactive scopes; focus remains on the tree.
  Re-entry may create new row elements. No hidden selected row is pinned in the element tree.
- **WIDGETS-TREE-09:** Malformed source entries returning None are skipped. Repeated IDs,
  including ancestor cycles, are emitted only at their first preorder encounter; subsequent
  occurrences and their subtrees are skipped and diagnosed with tracing. Traversal uses an
  explicit stack, so a deeply expanded chain does not overflow the Rust call stack. Sibling
  positions/counts describe source slots (one-based position, declared count), even when
  invalid entries are skipped. The source must report finite, practical counts; arbitrary
  panics or blocking inside app callbacks cannot be repaired by the widget.
- **WIDGETS-TREE-10:** Default labels use the current theme's body text/on-surface color;
  disclosure chevrons use on-surface-variant, point right when closed and down when open,
  and paint through Scene geometry without an icon-font feature requirement. Selection uses
  primary at 12% alpha, hover on-surface at 8% alpha; a focused tree has a 2 px primary
  outline inside its viewport. Theme color changes update mounted chrome and default text
  without resetting expansion, selection or scrolling. Goldens cover an indented open branch
  with a selected leaf, a closed branch, and light/dark themes.
- **WIDGETS-TREE-11:** The viewport has Tree semantics, one focus stop and a validated
  active descendant only when the selected row is mounted. Mounted wrappers have TreeItem
  semantics: source label (also for custom content), selected state, one-based level,
  source sibling position/count, expanded Some(bool) only for branches, Click and
  ScrollIntoView; closed branches offer Expand and open branches Collapse. Click selects
  the row through an explicit action handler rather than a position-dependent disclosure
  hit test; Expand/Collapse change only the target
  branch and are idempotent. Those explicit actions do not activate or change selection
  except the normal collapse fallback; no action operates on a removed row. Collapsed or
  unmounted rows have no accessibility nodes. The flat virtualized semantics use level and
  sibling metadata to represent hierarchy; they do not materialize hidden semantic children.
- **VIEW-SEM-04:** Tree/TreeItem roles and the four optional tree properties can be set,
  queried and bound like existing semantics. New properties default to None, and expand/
  collapse support defaults false. `perform_action` validates declared support, then routes
  Expand/Collapse through `on_action` handlers from target toward ancestors, stopping at the
  first true, without synthesizing global-focus keyboard input. TreeItem Click first offers
  the action to those handlers; if unhandled it uses the existing synthesized click fallback.
  Other roles keep their existing Click behavior. Unsupported actions and removed IDs return
  false. Existing remaining actions keep their behavior.
- **A11Y-TREE-05:** Tree/TreeItem map to the corresponding AccessKit roles. Expanded,
  level, position-in-set and size-of-set map without losing one-based values; None omits the
  property. Supported Expand/Collapse actions and incoming requests map to their backend-
  neutral counterparts, independent of current keyboard focus. Existing active-descendant
  validation still excludes absent/non-descendant rows. No AccessKit types enter widgets.
- **GALLERY-20:** Add a "Tree view" section immediately before Kitchen sink (ten sections,
  Kitchen sink stays last). A keyed `tree` shows Workspace (initially open), its src branch
  with main.rs/lib.rs and README.md, a closed Dependencies branch, and a closed Generated
  branch with 5,000 generated leaves. `tree-selection`, `tree-expanded`, `tree-activated`
  echo IDs/open count/last activation. Instructions describe arrows, Space and Enter.
  Pointer/keyboard interactions update echoes; opening Generated and scrolling constructs
  only viewport content. `tests/goldens/gallery-tree.png` at 960 x 640 records an open src
  and selected main.rs. The section follows the global theme switch.
- **GALLERY-21:** Insert a "tree" card immediately after Lists (thirteen SINK_CARDS entries),
  inventory `tree_view`, key `sink-tree_view`, with the same small Workspace hierarchy in a
  240 px viewport and initially open Workspace. `sink-tree-state` echoes selection and
  expansion count; Enter updates `sink-tree-activated`. Tests select a leaf, collapse its
  parent and verify the ancestor fallback. `gallery-sink-tree.png` records the card at
  960 x 640. Existing kitchen-sink inventory/scroll tests and affected goldens are updated.

## Performance and allocation

Let v be the number of nodes displayed by expansion, e the expansion-set size, and k the
number of mounted viewport/buffer rows. Rebuild the index only at initial build, expansion
change or revision: O(v log(v + e + 1)) or better time, O(v + e) storage; enumerate no
descendants of closed nodes and read no labels outside k. Metadata includes identity, depth,
displayed parent, sibling position/count and child count, with ID-to-index lookup for keys.
Traversal depth uses heap storage. A one-root source with one million collapsed children
must query zero child IDs and build one row; a 10,000-row open tree mounts O(k) widgets.

Scrolling performs O(k) row reconciliation and label/content work without O(v) source
enumeration or cloning the full index. Navigation uses indexed neighbors/parents and ID
lookups, not label scans. Expansion/selection updates may clone their application-owned
BTreeSet; they do not happen during steady layout/paint. Reuse metadata buffers where
practical; per-frame row lists retain the existing virtual-list allowance. Bounded tests
check call/live-row counts, not wall-clock thresholds. Performance trials remain local.

## Open questions

Resolved: the user accepted all six proposals by requesting implementation and the kitchen-sink example.

1. **Source trait + u64 IDs**, rather than a recursively owned node struct: apps can wrap
   existing models or generate children without copying them into widgets. A flattened
   expanded index gives simple navigation and bounded mounted widgets; expansion of a huge
   branch still costs O(v) metadata work and is not claimed to be fully lazy indexing.
2. **Single selection and BTreeSet expansion signals**, with useful private defaults. IDs
   survive reorder, and the application can observe/persist state without a controller API.
3. **32 px fixed rows, 20 px indentation, vertical scrolling** for the first feature.
   Custom builders replace label content while TreeView keeps interaction and theme chrome.
4. **Arrow/Space disclosure, Enter-only activation**, with ancestor selection on collapse;
   external hidden selections normalize rather than opening an unknown ancestor path.
5. **Proper tree roles/properties and explicit Expand/Collapse actions** in the existing
   semantics pipeline, including flat virtualized hierarchy metadata. No new dependency edge.
6. **Dedicated gallery section plus kitchen-sink card**, with deterministic correctness
   tests and software goldens; no GitHub benchmark or native stress run.
