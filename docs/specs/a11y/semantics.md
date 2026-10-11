# Semantics and the AccessKit tree

- **Status:** Agreed (the user said "continue" on the draft, 2026-10-12, taking its seven
  proposals)
- **Crates:** `tantu-view` (`Semantics` on elements, `bounds`, `perform_action`),
  `tantu-a11y` (the AccessKit tree and action mapping), `tantu-widgets` (every widget
  annotated), `tantu-test` (`a11y_tree`, `perform_action`), `examples/gallery`
- **Plan item:** Phase 3, "`tantu-a11y`", sub-item "semantics on elements, the AccessKit tree
  built from the view tree, actions routed back, every widget annotated"
- **Related:** [ADR 0013](../../adr/0013-accessibility-pipeline.md), [a11y platform](platform.md)
  (the other sub-item), [view tree](../view/tree.md), [focus](../view/focus.md),
  [events](../view/events.md), AccessKit 0.24 (`Role`, `Node`, `TreeUpdate`, `Action`),
  Flutter's `Semantics`

## Purpose

Assistive technology needs to know what is on screen: a button named "Save", a checkbox that
is checked, a slider at 40 of 100, a text field holding "hello", where each one is and which
has focus. This spec gives every element an optional `Semantics` value, set by its widget and
updated reactively, and a crate that turns the element tree into an AccessKit `TreeUpdate`.
Requests coming back from assistive technology (focus this, click that, increment) become the
same events a user would produce, so widgets need no second code path. The tester exposes the
tree, so every widget's semantics are checked like its paint.

## Scope

In scope:

- `tantu-view`: `Role`, `Semantics`, `SemanticsAction`, `BuildCx::set_semantics`,
  `ElementMut::update_semantics`, `ViewTree::semantics`, `ViewTree::bounds`,
  `ViewTree::perform_action`.
- `tantu-a11y`: `AccessibilityTree::update(&ViewTree) -> accesskit::TreeUpdate`,
  `map_action(&ActionRequest) -> Option<(ElementId, SemanticsAction)>`.
- Every widget sets semantics (table below); `IconButton`, `Icon` and `Image` take a
  `semantics_label`.
- `WidgetTester::a11y_tree()` and `WidgetTester::perform_action(finder, action)`.
- A gallery test that every sample has a node.

Out of scope (and where it goes): the platform side (adapters, events, the runner; the
[platform spec](platform.md)); text editing semantics (caret and selection ranges, text
positions: with the text field's next pass); merging descendants into one node (Flutter's
`MergeSemantics`), live regions, custom actions, hidden/offscreen culling of nodes,
localized role names: later.

## Public API

```rust
// tantu-view
/// What an element is, for assistive technology (a backend-neutral subset of AccessKit's roles).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Window, Group, Label, Button, CheckBox, RadioButton, Switch, Slider, TextInput,
    MultilineTextInput, Image, List, ListItem, ScrollView, Tooltip,
}

/// A checkable element's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggled { False, True, Mixed }

/// What assistive technology is told about an element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantics {
    pub role: Role,                      // default Group
    /// The accessible name (a label, a button's text).
    pub name: Option<String>,
    /// The current value as text (a field's text, a slider's number).
    pub value: Option<String>,
    /// A numeric value with its range and step, for sliders.
    pub numeric: Option<NumericValue>,
    pub toggled: Option<Toggled>,
    pub selected: Option<bool>,
    pub disabled: bool,
    /// The element takes keyboard focus (focusable elements set this themselves).
    pub focusable: bool,
    /// The actions the element supports, as assistive technology may request them.
    pub actions: SemanticsActions,      // bitflags-like struct of bools: click, focus, increment, decrement, set_value, scroll_into_view
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericValue { pub value: f64, pub min: f64, pub max: f64, pub step: f64 }

/// What assistive technology asked for.
#[derive(Clone, Debug, PartialEq)]
pub enum SemanticsAction { Focus, Click, Increment, Decrement, SetValue(String), ScrollIntoView }

impl BuildCx<'_> {
    /// Sets the element's semantics (a render element only; false for a region).
    pub fn set_semantics(&mut self, element: ElementId, semantics: Semantics) -> bool;
}
impl ElementMut<'_> {
    /// Changes the semantics in place (for reactive props); false without semantics.
    pub fn update_semantics(&mut self, f: impl FnOnce(&mut Semantics)) -> bool;
}
impl ViewTree {
    /// The element's semantics, if set.
    pub fn semantics(&self, element: ElementId) -> Option<&Semantics>;
    /// The element's bounds in window coordinates after the last layout (a region: the union
    /// of its children; `None` for an unknown element).
    pub fn bounds(&self, element: ElementId) -> Option<Rect>;
    /// Performs `action` on `element` as a user would (VIEW-SEM-03); false if nothing happened.
    pub fn perform_action(&mut self, element: ElementId, action: SemanticsAction) -> bool;
}

// tantu-a11y
/// Builds AccessKit tree updates from a view tree.
pub struct AccessibilityTree { /* app name, last sent ids */ }
impl AccessibilityTree {
    pub fn new(app_name: impl Into<String>) -> Self;
    /// The whole tree as one update (the root is the window node), with the focus.
    pub fn update(&mut self, tree: &ViewTree) -> accesskit::TreeUpdate;
    /// The element and view action an AccessKit request names, if the node is an element.
    pub fn map_action(request: &accesskit::ActionRequest) -> Option<(ElementId, SemanticsAction)>;
    /// The node id of an element (its id's bits).
    pub fn node_id(element: ElementId) -> accesskit::NodeId;
}

// tantu-widgets (new builders)
impl IconButton { pub fn semantics_label(self, label: impl Into<String>) -> Self; }
impl Icon { pub fn semantics_label(self, label: impl Into<String>) -> Self; }
impl Image { pub fn semantics_label(self, label: impl Into<String>) -> Self; }

// tantu-test
impl WidgetTester {
    /// The AccessKit tree of the last frame (TEST-WT-14).
    pub fn a11y_tree(&mut self) -> accesskit::TreeUpdate;
    /// Performs an accessibility action on the element `finder` matches, then pumps.
    pub fn perform_action(&mut self, finder: &Finder, action: SemanticsAction) -> bool;
}
```

Widget semantics:

| Widget | Role | name | value / state | actions |
|---|---|---|---|---|
| `Text`, `RichText` | Label | the text | | |
| `Button` | Button | the label | `disabled` | Click, Focus |
| `IconButton` | Button | `semantics_label` (else none) | `disabled` | Click, Focus |
| `Checkbox` | CheckBox | | `toggled` (Mixed when indeterminate), `disabled` | Click, Focus |
| `Radio` | RadioButton | | `toggled` True when selected, `disabled` | Click, Focus |
| `Switch` | Switch | | `toggled`, `disabled` | Click, Focus |
| `Slider` | Slider | | `numeric` (value, min, max, step = 5 % of the range or one division), `value` as text, `disabled` | Increment, Decrement, Focus |
| `TextField` | TextInput / MultilineTextInput | the placeholder | `value` = the text, `disabled` | Focus, SetValue |
| `Icon`, `Image` | Image | `semantics_label` (else none) | | |
| `ScrollView` | ScrollView | | | ScrollIntoView on children |
| `ListView` | List; a built item is wrapped in a ListItem (`selected` when it is the selection) | | | |
| `Tooltip` (shown) | Tooltip | the message | | |
| layout widgets, `Divider`, `Opacity`, `AnimatedSwitcher` | none (skipped) | | | |
| the tree's root | Window | the app name | | |

## Behavior

### `tantu-view`

- **VIEW-SEM-01:** `set_semantics` on a render element stores the value (`semantics` returns
  it; `None` before, and for regions, where `set_semantics` returns false);
  `update_semantics` changes it in place and is usable from a `bind` closure, so a reactive
  prop updates the name or value at the next frame; removing the element drops it.
- **VIEW-SEM-02:** `bounds` is the element's rect in window coordinates (its layout size at
  the sum of the offsets up to the root), the union of the children's bounds for a region,
  `Rect::ZERO`-sized for an element without layout, `None` for an unknown id. For a child of
  a scroll view it reflects the scroll offset.
- **VIEW-SEM-03:** `perform_action`: `Focus` focuses the element (as `ViewTree::focus`);
  `Click` dispatches a primary press and release at the element's center (so a button's
  `on_press` runs and a checkbox toggles) and returns whether a handler ran; `Increment` and
  `Decrement` focus the element and dispatch ArrowRight/ArrowUp or ArrowLeft/ArrowDown key
  presses as the element's role suggests (a slider steps, a scroll view scrolls); `SetValue`
  dispatches an IME-like replace: for an element with `set_value` in its actions, it focuses
  it, selects all and inserts the text (through the key path: select-all intent and a
  character event with `text`); `ScrollIntoView` scrolls every enclosing scroll view the least
  amount that shows the element. An element without semantics, or whose `actions` lack the
  action, returns false. Actions never panic on removed ids.

### `tantu-a11y`

- **A11Y-TREE-01:** `update` returns a `TreeUpdate` whose root is a `Window` node named after
  the app, holding one node per element with semantics, with `node_id(element)` as its id; an
  element without semantics (layout boxes, regions) contributes no node, and its descendants
  with semantics become children of the nearest semantic ancestor (or the window), in tree
  order. The update's `tree` names the root and `focus` is the focused element's node when it
  has semantics, else the root.
- **A11Y-TREE-02:** Each node carries the semantics as AccessKit properties: `role` mapped
  one to one, `label` from `name`, `value`, `numeric_value`/`min`/`max`/`step`, `toggled`,
  `selected`, the `disabled` flag, and `supported_actions` from `actions` (`Focus` when
  `focusable`). A text input's `value` is its text.
- **A11Y-TREE-03:** Each node's `bounds` is `ViewTree::bounds` as an AccessKit rect in the
  window's logical coordinates (no `transform`, since bounds are absolute), rounded to
  nothing; a zero-sized element still has a node.
- **A11Y-TREE-04:** `map_action` turns `Action::Focus`/`Click`/`Increment`/`Decrement`/
  `SetValue` (with `ActionData::Value`)/`ScrollIntoView` on a node whose id is an element into
  `(ElementId, SemanticsAction)`; other actions, or the window's id, give `None`.

### Widgets, tester, gallery

- **WIDGETS-SEM-01:** `Text` and `RichText` are `Label`s named by their text (reactively);
  `Button` and `IconButton` are `Button`s, named by the label or `semantics_label`, `disabled`
  while not enabled, with Click and Focus; `Icon` and `Image` are `Image`s named by
  `semantics_label`.
- **WIDGETS-SEM-02:** `Checkbox` (`toggled` Mixed while indeterminate), `Radio` and `Switch`
  follow the table, reactively; `Slider` exposes `numeric` and the value as text (the number
  with up to two decimals) and steps on Increment/Decrement as the keys do; `TextField` is a
  `TextInput` (`MultilineTextInput` when `max_lines` is not 1) named by its placeholder with
  the text as `value`, and `SetValue` replaces the text; `ScrollView` is a `ScrollView`;
  `ListView` is a `List` whose built items are `ListItem`s (`selected` for the selection);
  a shown `Tooltip` is a `Tooltip` named by its message.
- **TEST-WT-14:** `a11y_tree` builds the tree of the last frame with an `AccessibilityTree`
  (app name "test"); `perform_action` finds the element, calls `ViewTree::perform_action` and
  pumps, returning its result.
- **GALLERY-09:** In every section, each keyed sample has a node in the tree with the role of
  the table (the press button a `Button` named "Press me", the checkbox a `CheckBox`, the
  slider a `Slider`, the field a `TextInput`, the list a `List` with `ListItem` children);
  `perform_action(press, Click)` increments the press count.

## Performance and allocation

`update` walks the whole element tree once per call and allocates the node list (a few
hundred nodes for the gallery); it runs only while assistive technology is active (platform
spec). Semantics are stored per element (an `Option<Box<Semantics>>`, so elements without
them pay a pointer).

## Open questions

Resolved (2026-10-12; the user said "continue" on the draft, taking these proposals):

1. **Semantics are plain data on elements in `tantu-view`** (ADR 0013), with a `Role` enum
   that mirrors the AccessKit roles the widgets need, so `tantu-view` and the widgets never
   see AccessKit.
2. **Actions are synthesized as user input** (press/release, keys, the select-all intent),
   not routed to per-widget handlers: one behavior path, no new widget API; a widget that
   needs something smarter can get an `on_action` hook later.
3. **Full-tree updates**; AccessKit diffs against its own copy. Incremental updates can
   replace the body of `update` later.
4. **Unsemantic elements are skipped, not grouped**: AccessKit trees stay shallow (window →
   widgets), which is what screen readers read best; `Group` is available for widgets that
   want it.
5. **`semantics_label` builders** on `IconButton`, `Icon` and `Image` (Flutter's
   `semanticLabel`), rather than a general `Semantics` wrapper widget, which can come when a
   custom-paint view needs one.
6. **`tantu-test` depends on `tantu-a11y`** so tests read the real `TreeUpdate`; AGENTS.md
   row `tantu-test | view, theme, a11y, render-soft, render-headless`.
7. **AccessKit 0.24 and `accesskit_winit` 0.32** (the pair matching winit 0.30), as plain
   dependencies, not features.
