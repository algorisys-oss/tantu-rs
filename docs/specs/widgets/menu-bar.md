# Menus, part 2: menu bar, toolbars and status bar

- **Status:** Implemented (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (`MenuBar`, `Toolbar`, `StatusBar`), `tantu-view` (roles),
  `tantu-a11y` (role mapping), `examples/gallery`
- **Plan item:** Phase 4, "Menus (native + custom), context menus, toolbars, status bar",
  part 2
- **Related:** [menus part 1](menu.md), [focus](../view/focus.md),
  [semantics](../a11y/semantics.md), [divider](divider.md), [gallery](../examples/gallery.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

The frame of a desktop window: a menu bar along the top (File, Edit, View…), toolbars of
icon buttons under it, and a status bar along the bottom. `MenuBar` reuses part 1's menus
(items, submenus, keyboard, semantics) and adds the bar's own behavior: switching between
menus by hovering or with the arrow keys. `Toolbar` lays out tool controls with separators
and moves the focus between them with the arrow keys. `StatusBar` shows short state
("Ln 12, Col 4", "Saved") at the window's bottom edge. The gallery's own window uses all
three.

## Scope

In scope: the three widgets, their look, pointer and keyboard behavior and semantics; the
gallery window using a menu bar, a status bar and a scrolling sidebar; a kitchen-sink card
with a toolbar and a status bar.

Out of scope (and where it goes): native menu bars (part 3); mnemonics (Alt+F) and F10 to
reach the bar (they need app-wide key routing; later); toolbar overflow menus and
customization; making a toolbar a single Tab stop (WAI-ARIA's toolbar pattern) — controls
stay in Tab order and arrows are an extra (review); status bar progress indicators.

## Public API

```rust
/// A row of top-level menus (Flutter's `MenuBar`).
pub struct MenuBar { /* private */ }
impl MenuBar {
    pub fn new() -> Self;
    /// Adds a top-level menu labelled `label`.
    pub fn menu(self, label: impl Into<String>, items: impl IntoIterator<Item = MenuItem>) -> Self;
}
impl Default for MenuBar;

/// A row of tool controls with separators.
pub struct Toolbar { /* private */ }
impl Toolbar {
    pub fn new() -> Self;
    pub fn child(self, child: impl View) -> Self;
    /// A vertical separator between groups of controls.
    pub fn separator(self) -> Self;
}
impl Default for Toolbar;

/// A strip along the window's bottom: leading items on the left, trailing on the right.
pub struct StatusBar { /* private */ }
impl StatusBar {
    pub fn new() -> Self;
    pub fn leading(self, item: impl View) -> Self;
    pub fn trailing(self, item: impl View) -> Self;
}
impl Default for StatusBar;

// tantu-view
pub enum Role { /* existing, */ MenuBar, Toolbar, Status }
```

## Behavior

- **WIDGETS-MENUBAR-01:** The bar fills its width, 40 px tall, on the theme's `surface`
  with a 1 px `outline_variant` line along its bottom; top-level items sit from the left,
  each its label (theme `label` style) with 12 px on each side. An item whose menu is open,
  or the bar's current item while the bar has the focus, gets an `on_surface` 8 % layer; a
  focused bar also outlines its current item (2 px `primary`).
- **WIDGETS-MENUBAR-02:** Pointer: a primary press on a top-level item opens its menu
  below it (part 1's menu, `Anchor::below()`), or closes it if it is open; while a menu is
  open, hovering another top-level item opens that one instead. A press on the bar doesn't
  take the focus. Choosing an item, Escape at a top menu and a press outside close the
  open menu (part 1's rules), and the focus goes back where it was.
- **WIDGETS-MENUBAR-03:** Keyboard: the bar is one Tab stop with a current item (the
  first, initially). Left/Right move the current item, wrapping; Down, Enter or Space
  open its menu with the first item highlighted. In an open top menu, Left, or Right on an
  item without a submenu, close it and open the previous/next top menu (wrapping) with its
  first item highlighted, making that the current item.
- **WIDGETS-MENUBAR-04:** Semantics: the bar is a `MenuBar`, top-level items `MenuItem`s
  named by their labels with `expanded` while open; the focused bar's `active_descendant`
  is its current item.
- **WIDGETS-TOOLBAR-01:** A toolbar fills its width, at least 48 px tall, its children in a
  row centred vertically with 8 px side padding and 4 px between them; a separator is a
  1 × 24 px `outline_variant` line with 4 px more on each side.
- **WIDGETS-TOOLBAR-02:** While the focus is inside a toolbar child, Left/Right move it to
  the previous/next child that takes focus, wrapping, Home/End to the first/last; separators
  are skipped. Modified keys pass through; Tab is unchanged.
- **WIDGETS-TOOLBAR-03:** Semantics: a toolbar is a `Toolbar`.
- **WIDGETS-STATUSBAR-01:** A status bar fills its width, 28 px tall, on the theme's
  `surface_container_highest` with a 1 px `outline_variant` line along its top; leading
  items sit from the left and trailing items from the right, 16 px apart and 12 px from the
  edges, centred vertically.
- **WIDGETS-STATUSBAR-02:** Semantics: a status bar is a `Status` (a polite live region).
- **A11Y-TREE-10:** `Role::MenuBar`, `Toolbar` and `Status` map to AccessKit's roles of the
  same names.
- **GALLERY-31:** The gallery window gets a menu bar above its header (key `menubar`):
  View (a checked "Dark theme" mirroring the switch), Go (one item per section) and Help
  ("About Tantu" setting the status text), and a status bar below everything (key
  `statusbar`) showing "Section: <name>" leading and "Light"/"Dark" trailing (or the About
  text until the next navigation). The sidebar scrolls, so every section stays reachable
  however many there are; tests reveal a sidebar button before tapping it. Tests:
  navigating with Go → Tabs; View → Dark theme toggles the theme and the switch; the
  status bar follows. Every gallery golden is regenerated.
- **GALLERY-32:** A "bars" kitchen-sink card after "menus" (eighteen `SINK_CARDS` entries,
  samples `tool_bar` and `status_bar`): a toolbar of Bold, Italic and Underline icon
  buttons, a separator, Undo and Redo, echoing "Format: …" (`sink-bars-state`), and a status
  bar sample. Tests: Bold toggles the echo; Right from Bold focuses Italic.

## Performance and allocation

None beyond the general rules; menus build only while open.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **`MenuBar` reuses part 1's menus**; the bar adds switching by hover and arrows.
2. **The bar is one Tab stop** with a current item, and pressing it doesn't steal focus
   from the content (as native menu bars behave).
3. **Toolbar controls stay in Tab order**, with arrow keys as an extra, rather than
   WAI-ARIA's single-stop toolbar (which needs roving focus the view layer doesn't offer
   yet).
4. **Mnemonics and F10** are deferred until app-wide key routing exists.
5. **The gallery window uses a menu bar, a status bar and a scrolling sidebar**, which
   also solves the full sidebar (HANDOFF open question).
