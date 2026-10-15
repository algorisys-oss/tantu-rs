//! Popup menus and context menus: [`MenuItem`], [`MenuController`], [`MenuAnchor`] and
//! [`ContextMenuRegion`]. Spec: `docs/specs/widgets/menu.md`.

use std::rc::Rc;

use tantu_view::reactive::{Signal, signal};
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Prop, View};

use crate::IconData;

#[allow(dead_code)] // used once menus are implemented
/// What a menu item is.
#[derive(Clone)]
enum ItemKind {
    Action,
    Divider,
    Submenu(Vec<MenuItem>),
}

#[allow(dead_code)] // used once menus are implemented
/// One entry of a menu: an action, a submenu or a divider.
#[derive(Clone)]
pub struct MenuItem {
    kind: ItemKind,
    label: String,
    on_select: Option<Rc<dyn Fn()>>,
    shortcut: Option<String>,
    icon: Option<IconData>,
    checked: Option<Rc<dyn Fn() -> bool>>,
    enabled: Rc<dyn Fn() -> bool>,
}

/// A prop as a shared closure.
fn shared<T: Clone + 'static>(prop: Prop<T>) -> Rc<dyn Fn() -> T> {
    match prop {
        Prop::Value(value) => Rc::new(move || value.clone()),
        Prop::Dynamic(f) => Rc::from(f),
    }
}

impl MenuItem {
    fn with_kind(kind: ItemKind, label: String) -> Self {
        MenuItem {
            kind,
            label,
            on_select: None,
            shortcut: None,
            icon: None,
            checked: None,
            enabled: Rc::new(|| true),
        }
    }

    /// An action labelled `label`.
    pub fn new(label: impl Into<String>) -> Self {
        MenuItem::with_kind(ItemKind::Action, label.into())
    }

    /// A horizontal divider between groups of items.
    pub fn divider() -> Self {
        MenuItem::with_kind(ItemKind::Divider, String::new())
    }

    /// A submenu labelled `label` holding `items`.
    pub fn submenu(label: impl Into<String>, items: impl IntoIterator<Item = MenuItem>) -> Self {
        MenuItem::with_kind(ItemKind::Submenu(items.into_iter().collect()), label.into())
    }

    /// Called when the item is chosen (the menu closes first).
    pub fn on_select(mut self, f: impl Fn() + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Shortcut hint text shown at the trailing edge ("Ctrl+S"); display only.
    pub fn shortcut(mut self, text: impl Into<String>) -> Self {
        self.shortcut = Some(text.into());
        self
    }

    /// An icon at the leading edge.
    pub fn icon(mut self, icon: IconData) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A check mark at the leading edge while true (the item becomes a checkbox item).
    pub fn checked(mut self, checked: impl IntoProp<bool>) -> Self {
        self.checked = Some(shared(checked.into_prop()));
        self
    }

    /// Disabled items are dimmed and can't be chosen or opened (default enabled).
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = shared(enabled.into_prop());
        self
    }
}

/// Opens and closes a menu; `Copy`, owned by the scope that created it.
#[derive(Clone, Copy)]
pub struct MenuController {
    open: Signal<bool>,
}

impl Default for MenuController {
    fn default() -> Self {
        MenuController::new()
    }
}

impl MenuController {
    /// A closed menu.
    pub fn new() -> Self {
        MenuController {
            open: signal(false),
        }
    }

    /// Opens the menu.
    pub fn open(&self) {
        if !self.open.get_untracked() {
            self.open.set(true);
        }
    }

    /// Closes the menu (and its submenus).
    pub fn close(&self) {
        if self.open.get_untracked() {
            self.open.set(false);
        }
    }

    /// Opens a closed menu, closes an open one.
    pub fn toggle(&self) {
        self.open.set(!self.open.get_untracked());
    }

    /// Whether the menu is open (tracked).
    pub fn is_open(&self) -> bool {
        self.open.get()
    }
}

/// Shows a menu below `child` while its controller is open (Flutter's `MenuAnchor`).
pub struct MenuAnchor {
    controller: MenuController,
    child: AnyView,
    items: Vec<MenuItem>,
}

impl MenuAnchor {
    /// A menu anchored below `child`, shown while `controller` is open.
    pub fn new(controller: MenuController, child: impl View) -> Self {
        MenuAnchor {
            controller,
            child: AnyView::new(child),
            items: Vec::new(),
        }
    }

    /// Adds an item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Adds several items.
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl View for MenuAnchor {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.controller, self.child, self.items);
        todo!("WIDGETS-MENU-01..07")
    }
}

/// Shows a menu at the pointer on a secondary press inside `child` (or on Shift+F10 while
/// the focus is inside it, at the child's top-left).
pub struct ContextMenuRegion {
    child: AnyView,
    items: Vec<MenuItem>,
    controller: Option<MenuController>,
}

impl ContextMenuRegion {
    /// A context menu region around `child`.
    pub fn new(child: impl View) -> Self {
        ContextMenuRegion {
            child: AnyView::new(child),
            items: Vec::new(),
            controller: None,
        }
    }

    /// Adds an item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Adds several items.
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Shares the open state (e.g. to close the menu from code).
    pub fn controller(mut self, controller: MenuController) -> Self {
        self.controller = Some(controller);
        self
    }
}

impl View for ContextMenuRegion {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.child, self.items, self.controller);
        todo!("WIDGETS-MENU-08")
    }
}
