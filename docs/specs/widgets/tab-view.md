# TabView: tab bars and kept tab panels

- **Status:** Implemented (2026-10-15; agreed the same day, the user said "continue" on the six proposals)
- **Crates:** `tantu-widgets` (widgets), `tantu-view` and `tantu-a11y` (tab roles),
  `examples/gallery`
- **Plan item:** Phase 4, "`TabView`", part 2 (the widgets)
- **Related:** [Offstage](../view/offstage.md), [scroll](scroll.md),
  [transitions](transitions.md), [focus](../view/focus.md),
  [shortcuts](../view/shortcuts.md), [semantics](../a11y/semantics.md),
  [gallery](../examples/gallery.md), [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Settings dialogs, record editors and dashboards split one screen into named pages. Apps
get Flutter's `TabBar` and `TabBarView`, which can be placed separately (a bar in a toolbar,
the pages below), sharing one selected-index signal, and a `TabView` that composes the two
for the common case. Visited pages are kept alive offstage, so their state survives
switching, as in desktop toolkits.

## Scope

In scope: static tab sets built once (label, optional icon, disabled tabs); fixed (equal
width) and scrollable (natural width, horizontally scrolling) bars; the Material 3 primary
tab look with a sliding indicator; pointer, keyboard and accessibility interaction; lazy,
kept pages; a gallery section and a kitchen-sink card.

Out of scope: adding, removing, reordering or closing tabs at run time and drag-to-detach
(document tabs belong to the docking item); secondary/vertical tab styles; swipe paging and
page transition animations; tab overflow menus; RTL mirroring (Phase 4 i18n).

## Public API

All items are exported from `tantu-widgets` and the facade prelude.

```rust
/// One tab's header: a label, an optional leading icon, enabled by default.
#[derive(Clone)]
pub struct Tab { /* private */ }
impl Tab {
    pub fn new(label: impl Into<String>) -> Self;
    /// An icon above the label (the bar becomes 64 px tall instead of 48 px).
    pub fn icon(self, icon: IconData) -> Self;
    /// A disabled tab is shown dimmed, can't be selected by the user and is skipped by keys.
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
}

/// The row of tab headers. The selected index is a shared signal.
pub struct TabBar { /* private */ }
impl TabBar {
    pub fn new(selected: Signal<usize>) -> Self;
    pub fn tab(self, tab: Tab) -> Self;
    pub fn tabs(self, tabs: impl IntoIterator<Item = Tab>) -> Self;
    /// Natural-width tabs in a horizontal scroll view instead of equal widths; default false.
    pub fn scrollable(self, scrollable: bool) -> Self;
}

/// The pages; only the selected one is shown. Pages build on first selection.
pub struct TabBarView { /* private */ }
impl TabBarView {
    pub fn new(selected: Signal<usize>) -> Self;
    /// Adds the next page; `build` runs when the page is first selected.
    pub fn page<V: View>(self, build: impl Fn() -> V + 'static) -> Self;
    /// Keep visited pages alive offstage (default true); false disposes a page on leaving.
    pub fn keep_alive(self, keep_alive: bool) -> Self;
}

/// A `TabBar` above an expanded `TabBarView`, with Control+Tab cycling across both.
pub struct TabView { /* private */ }
impl TabView {
    /// Uses an owner-scoped selected signal starting at 0 unless `selected` shares one.
    pub fn new() -> Self;
    pub fn selected(self, selected: Signal<usize>) -> Self;
    pub fn tab<V: View>(self, tab: Tab, build: impl Fn() -> V + 'static) -> Self;
    pub fn scrollable(self, scrollable: bool) -> Self;
    pub fn keep_alive(self, keep_alive: bool) -> Self;
}

// tantu-view additions
pub enum Role { /* existing, */ TabList, Tab, TabPanel }
```

`Tab`, `TabBar`, `TabBarView` and `TabView` implement `View` (`Tab` only through the bar).
`TabView` implements `Default`. No new feature flags.

## Behavior

- **WIDGETS-TAB-01:** The shown index is `selected` clamped to `0..len`; the signal itself
  is not rewritten. An empty bar or view shows nothing and has no focus stop. The bar and
  the view read only the signal, so a `TabBar` and `TabBarView` sharing it stay in step and
  the app may set it directly; disabled tabs can still be selected by the app.
- **WIDGETS-TAB-02:** Fixed bars split a bounded width equally between tabs (each at least
  its label's width plus 2 × 16 px; overflow is reported like a `Row`'s); an unbounded width
  sizes each tab to its content. Scrollable bars size tabs to content plus 2 × 16 px, start
  at the left and scroll horizontally with the wheel; selecting a tab (by any means)
  reveals it. Height 48 px, or 64 px when any tab has an icon (24 px icon above the label).
- **WIDGETS-TAB-03:** Look (Material 3 primary tabs, read from the theme): labels in the
  `label` text style, `primary` when selected and `on_surface_variant` otherwise; disabled
  tabs at 38 % `on_surface`; a 1 px `outline_variant` divider along the bar's bottom; a
  3 px `primary` indicator with 3 px top corner radii under the selected tab's content
  (label or icon, whichever is wider, at least 24 px); hover fills the tab with `on_surface`
  at 8 %. Theme changes repaint without changing selection or pages.
- **WIDGETS-TAB-04:** When the shown index changes, the indicator slides from the old tab's
  position and width to the new one over 300 ms with `Curve::Ease` (Flutter's
  `kTabScrollDuration`); a change while sliding continues from the current position. The
  first build shows it in place without animating.
- **WIDGETS-TAB-05:** A primary press and release on an enabled tab sets `selected` to its
  index (on release inside the tab, like `Button`) and focuses the bar; disabled tabs and
  secondary buttons do nothing. Pressing the selected tab changes nothing.
- **WIDGETS-TAB-06:** The bar is a single focus stop. While it has focus, its focus ring
  (2 px `primary`, inside the tab) surrounds the selected tab. Left/Right select the
  previous/next enabled tab, wrapping; Home/End the first/last enabled tab (selection follows
  focus: WAI-ARIA's automatic activation). Enter and Space do nothing (already selected).
  Repeats move again; modified keys pass through. With every tab disabled, keys do nothing.
- **WIDGETS-TAB-07:** `TabView` binds Control+Tab / Control+Shift+Tab and Control+PageDown /
  Control+PageUp (shortcuts on its outer element, on every platform) to select the
  next/previous enabled tab, wrapping, from anywhere inside it, including the pages; Tab
  traversal is unaffected otherwise. A standalone `TabBar` binds them only while focused.
- **WIDGETS-TAB-08:** Pages build lazily: a page's builder runs the first time its index is
  shown, exactly once while kept. With `keep_alive(true)` a page that is left stays built
  inside an `Offstage` (VIEW-OFFSTAGE-01..05), so its fields, scroll offsets, selections and
  controllers are the same when it returns. With `keep_alive(false)` a left page is disposed
  and rebuilt on return. Pages fill the view's constraints (tight on bounded axes).
- **WIDGETS-TAB-09:** When the shown page changes while the focus is inside the leaving
  page, the focus moves to the bar (in a `TabView`) or is cleared (standalone
  `TabBarView`); the IME of a leaving field is disabled. Focus outside the pages is
  unchanged.
- **WIDGETS-TAB-10:** Semantics: the bar has `TabList`; each tab `Tab` with its label,
  `selected`, `disabled`, one-based `position_in_set`/`size_of_set`, `Click` and `Focus`
  (`Click` selects an enabled tab); the shown page's wrapper has `TabPanel` named after its
  tab. The bar's `active_descendant` is the selected tab while focused. Kept pages that are
  offstage have no nodes (A11Y-TREE-06).
- **A11Y-TREE-07:** `TabList`, `Tab` and `TabPanel` map to AccessKit's roles of the same
  names; tab selection and set position/size use the existing mappings.
- **GALLERY-22:** A "Tabs" section before Kitchen sink (eleven sections): a `TabView` (key
  `tabs`) with General (a `TextField`), Advanced (a `Switch` and a `Slider`), Data (a
  200-row `ListView`) and a disabled Archived tab, and a scrollable `TabBar` (key
  `tabs-scrollable`) of twelve tabs; "Tab: …" (`tab-echo`) shows the selected tab. Tests:
  typed text survives switching away and back; Control+Tab from inside the field moves on;
  arrows skip Archived. Golden `gallery-tabs.png` at 960 × 640 with Advanced selected.
- **GALLERY-23:** A "tabs" kitchen-sink card after "tree" (fourteen `SINK_CARDS` entries,
  samples `tab_bar` and `tab_view`): a three-tab `TabView` 200 px tall whose pages are a
  label, a checkbox and a text field, echoed by `sink-tabs-state`. Goldens touched by the
  new section and card are regenerated.

## Performance and allocation

Switching tabs builds at most one page, and kept pages cost nothing per frame (Offstage).
The bar's layout is O(tabs); indicator animation repaints only. No allocation per frame
beyond the existing widgets'.

## Open questions

Accepted by the user ("continue", 2026-10-15):

1. **Flutter's split** (`TabBar` + `TabBarView` over one `Signal<usize>`, as TreeView shares
   signals rather than a controller object) plus a composed `TabView`, the name PLAN.md uses.
2. **Visited pages kept alive by default**, built lazily, through the new `Offstage`
   (`docs/specs/view/offstage.md`, part 1 of this item); `keep_alive(false)` for heavy pages.
3. **Static tab sets** in this item; run-time add/close/reorder waits for docking, which
   needs document tabs anyway.
4. **WAI-ARIA keyboard model**: one stop, arrows with automatic activation, wrapping,
   disabled tabs skipped; Control+Tab / Control+PageDown cycling as on every desktop.
5. **Material 3 primary-tab look** (48/64 px, 3 px sliding indicator, 300 ms `Ease`), fixed
   and scrollable bars; no secondary or vertical tabs yet.
6. **Gallery "Tabs" section and a kitchen-sink card**, with behavior tests and goldens.
