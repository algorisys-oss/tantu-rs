# Menus, part 1: popup menus and context menus

- **Status:** Implemented (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (menus), `tantu-view` (reactive anchor, menu roles),
  `tantu-a11y` (role mapping), `examples/gallery`
- **Plan item:** Phase 4, "Menus (native + custom), context menus, toolbars, status bar",
  part 1 (popup and context menus)
- **Related:** [overlay](../view/overlay.md), [focus](../view/focus.md),
  [focus scopes](../view/focus-scope.md), [semantics](../a11y/semantics.md),
  [icon](icon.md), [tooltip](tooltip.md), [gallery](../examples/gallery.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Desktop apps put commands in menus: a "⋯" button opening a list of actions, a right-click
context menu on a grid row or a tree node, submenus for "Open recent" or "Move to". This part
provides Flutter's model in Tantu's terms: `MenuItem`s (with shortcut hints, icons, check
marks, submenus and dividers), a `MenuController` the app opens and closes, a `MenuAnchor`
that shows a menu below a widget, and a `ContextMenuRegion` that shows one at the pointer.
Menus are overlays (`OverlayPortal`), so they draw above everything and inherit the theme.

## Scope

In scope: menu items, dividers, check marks, leading icons, shortcut hint text, disabled
items, nested submenus; opening, closing and selecting with pointer and keyboard; focus
moving into an open menu and back on close; context menus from a secondary press or
Shift+F10; semantics; a reactive `OverlayPortal::anchor` (view amendment); gallery samples.

Out of scope (and where it goes): the in-window `MenuBar`, toolbars and status bars (part
2); native OS menus (macOS app menu, Windows/Linux native menu bars and context menus):
they need a platform-crate dependency such as `muda` and an ADR (part 3, deferred for the
user); typeahead (first-letter jumps), menu scrolling for menus taller than the window, and
open/close animations (later); radio groups as a distinct item kind (use check marks).

## Public API

```rust
// tantu-view
impl<V: View> OverlayPortal<V> {
    /// The anchor, a value, closure or signal; changes re-place the overlay at the next
    /// frame (VIEW-OVERLAY-11).
    pub fn anchor(self, anchor: impl IntoProp<Anchor>) -> Self;
}
pub enum Role { /* existing, */ Menu, MenuItem, MenuItemCheckBox }

// tantu-widgets
/// One entry of a menu: an action, a submenu or a divider.
#[derive(Clone)]
pub struct MenuItem { /* private */ }
impl MenuItem {
    /// An action labelled `label`.
    pub fn new(label: impl Into<String>) -> Self;
    /// A horizontal divider between groups of items.
    pub fn divider() -> Self;
    /// A submenu labelled `label` holding `items`.
    pub fn submenu(label: impl Into<String>, items: impl IntoIterator<Item = MenuItem>) -> Self;
    /// Called when the item is chosen (the menu closes first).
    pub fn on_select(self, f: impl Fn() + 'static) -> Self;
    /// Shortcut hint text shown at the trailing edge ("Ctrl+S"); display only.
    pub fn shortcut(self, text: impl Into<String>) -> Self;
    /// An icon at the leading edge.
    pub fn icon(self, icon: IconData) -> Self;
    /// A check mark at the leading edge while true (the item becomes a checkbox item).
    pub fn checked(self, checked: impl IntoProp<bool>) -> Self;
    /// Disabled items are dimmed and can't be chosen or opened (default enabled).
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
}

/// Opens and closes a menu; `Copy`, owned by the scope that created it.
#[derive(Clone, Copy)]
pub struct MenuController { /* a Signal<bool> */ }
impl MenuController {
    pub fn new() -> Self;
    pub fn open(&self);
    pub fn close(&self);
    pub fn toggle(&self);
    pub fn is_open(&self) -> bool; // tracked
}
impl Default for MenuController;

/// Shows a menu below `child` while its controller is open (Flutter's `MenuAnchor`).
pub struct MenuAnchor { /* private */ }
impl MenuAnchor {
    pub fn new(controller: MenuController, child: impl View) -> Self;
    pub fn item(self, item: MenuItem) -> Self;
    pub fn items(self, items: impl IntoIterator<Item = MenuItem>) -> Self;
}

/// Shows a menu at the pointer on a secondary press inside `child` (or on Shift+F10 while
/// the focus is inside it, at the child's top-left).
pub struct ContextMenuRegion { /* private */ }
impl ContextMenuRegion {
    pub fn new(child: impl View) -> Self;
    pub fn item(self, item: MenuItem) -> Self;
    pub fn items(self, items: impl IntoIterator<Item = MenuItem>) -> Self;
    /// Shares the open state (e.g. to close the menu from code).
    pub fn controller(self, controller: MenuController) -> Self;
}
```

## Behavior

- **VIEW-OVERLAY-11:** `OverlayPortal::anchor` accepts a value, closure or signal; a new
  anchor value places the overlay again at the next frame with the same rules
  (VIEW-OVERLAY-01, -03). `Anchor` is an `IntoProp` value.
- **VIEW-OVERLAY-12:** (found while implementing, review) A press inside a nested overlay
  (one whose target lies inside another overlay's content, at any depth) counts as inside
  the outer overlay for `on_outside_press`, so clicking a submenu doesn't close its parent
  menu (VIEW-OVERLAY-05 otherwise applies unchanged).
- **WIDGETS-MENU-01:** While its controller is open, `MenuAnchor` shows its items in a menu
  below the child (`Anchor::below()`, flipping above and clamping to the window). The menu
  is a column of 40 px items between 8 px top and bottom padding, at least 112 px and at
  most 280 px wide (as wide as its widest item); items are 12 px from the edges, with a
  24 px leading slot when any item has an icon or check mark, the label in the theme's
  `label` style, the shortcut text right-aligned in `on_surface_variant`, and a chevron
  (`Icons::CHEVRON_RIGHT`) on submenu items. A divider is a 1 px `outline_variant` line with
  8 px above and below. The surface is the theme's `surface` with a 4 px radius and a
  Material level-2 shadow. A menu with no items shows nothing.
- **WIDGETS-MENU-02:** Choosing an enabled action (primary click, or Enter/Space on the
  highlighted item) closes every open menu of that anchor or region, then calls its
  `on_select`. Disabled items are drawn at 38 % and ignore pointer and keys; dividers can't
  be highlighted.
- **WIDGETS-MENU-03:** Closing: Escape closes the innermost open menu; a primary press
  outside all open menus (and outside the anchor's child) closes them all; so do
  `MenuController::close` and choosing an item. Closing the root menu returns the focus to
  the element focused before it opened, if it still exists.
- **WIDGETS-MENU-04:** Keyboard: an opened menu takes the focus (it is a focus scope) with
  no item highlighted; Down/Up highlight the next/previous enabled item, wrapping (from
  none: the first/last); Home/End the first/last; Right on a submenu item, or Enter/Space,
  opens its submenu with its first item highlighted; Left (or Escape) in a submenu closes
  it and highlights its item in the parent. The highlighted item gets an `on_surface` 8 %
  layer (12 % while pressed). Modified keys pass through.
- **WIDGETS-MENU-05:** Pointer: hovering an enabled item highlights it; hovering a submenu
  item opens its submenu (closing any other open submenu at that level), and hovering
  another item of the parent closes it.
- **WIDGETS-MENU-06:** A submenu opens beside its item, top edges aligned (the item's
  TOP_RIGHT to the submenu's TOP_LEFT, moved up by the 8 px padding), flipping to the left
  side, or upward with bottom edges aligned (8 px below the item's bottom), when it would
  leave the window (VIEW-OVERLAY-03). Submenus nest to any depth.
- **WIDGETS-MENU-07:** A `checked` item shows a check mark (`Icons::CHECK`) in its leading
  slot while true and nothing while false; choosing it calls `on_select` (the app flips the
  value).
- **WIDGETS-MENU-08:** `ContextMenuRegion` opens its menu on a secondary press inside its
  child, with the menu's top-left at the pointer (flip and clamp on); another secondary
  press moves it. Shift+F10 with the focus inside opens it at the child's top-left. The
  region's child receives the press as usual (the region doesn't stop it).
- **WIDGETS-MENU-09:** Semantics: an open menu is a `Menu`; items are `MenuItem`s (or
  `MenuItemCheckBox` with `toggled` when `checked` is set) named by their label, with
  `disabled`, `expanded` on submenu items (true while open), `Click` and `Focus`; dividers
  have no node. The menu's `active_descendant` is the highlighted item.
- **A11Y-TREE-09:** `Role::Menu`, `MenuItem` and `MenuItemCheckBox` map to AccessKit's roles
  of the same names.
- **GALLERY-29:** The Buttons section gains a "⋯ More" `MenuAnchor` (Cut, Copy, Paste with
  shortcut hints, a divider, a checked "Word wrap", an "Open recent" submenu of three
  files, a disabled "Print") and a `ContextMenuRegion` box ("Right-click here"), with a
  "Menu: …" echo (`menu-echo`) naming the last chosen item. Tests: choosing Copy through the
  anchor; toggling Word wrap; a submenu item through the keyboard; a context-menu item.
  Golden `gallery-menu.png` with the menu and its submenu open.
- **GALLERY-30:** A "menus" kitchen-sink card after "dock" (seventeen `SINK_CARDS`
  entries, samples `menu_anchor` and `context_menu`), echoed by `sink-menus-state`.

## Performance and allocation

Menus build when opened and are disposed when closed; nothing runs per frame while closed.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **Flutter's `MenuAnchor` + controller model** with `MenuItem` values, rather than a menu
   builder closure; a `ContextMenuRegion` for right-click menus.
2. **Native menus deferred** to a part 3 that needs an ADR (a `muda`-style dependency in
   `tantu-platform-winit`, and `Platform` trait additions); custom menus work everywhere.
3. **40 px desktop items** (Material's 48 at density −2), 112–280 px wide, theme `surface`
   with a shadow.
4. **Focus moves into an open menu and back on close**; no item is highlighted until a key
   or the pointer picks one (Windows and GTK behavior when opened by pointer).
5. **Submenus open on hover immediately** (no delay) and close when a sibling is hovered;
   a hover delay can come with menu animations.
6. **A reactive `OverlayPortal::anchor`** (VIEW-OVERLAY-11) for context menus at the
   pointer, instead of a separate positioning mechanism.
7. **Gallery samples in the Buttons section** (the sidebar is full) plus a kitchen-sink card.
