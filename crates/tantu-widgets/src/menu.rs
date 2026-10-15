//! Popup menus and context menus: [`MenuItem`], [`MenuController`], [`MenuAnchor`] and
//! [`ContextMenuRegion`]. Spec: `docs/specs/widgets/menu.md`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_theme::ColorScheme;
use tantu_view::core::{Color, Rect, Size, Vec2};
use tantu_view::layout::{
    Alignment, BoxConstraints, LayoutChildren, RenderBox, RenderConstrainedBox,
};
use tantu_view::reactive::{Memo, Signal, effect, memo, signal, untrack};
use tantu_view::scene::{BorderRadius, BoxShadow, RoundedRect};
use tantu_view::{
    Anchor, AnyView, BuildCx, Dyn, ElementId, FocusOptions, Handled, IntoProp, LogicalKey,
    Modifiers, NamedKey, OverlayPortal, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop,
    Role, Semantics, SemanticsActions, Toggled, View,
};

use crate::directionality::{directional_key, is_rtl};
use crate::theme::{ALPHA_38, ThemeHandle, at};
use crate::{Icon, IconData, Icons, SizedBox, Text};

/// What a menu item is.
#[derive(Clone)]
enum ItemKind {
    Action,
    Divider,
    Submenu(Vec<MenuItem>),
}

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
        build_menu(
            cx,
            self.child,
            self.items,
            self.controller,
            Prop::Value(Anchor::below()),
        )
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
        let controller = self.controller.unwrap_or_default();
        let at = signal(Anchor::new(Alignment::TOP_LEFT, Alignment::TOP_LEFT));
        let region = RegionBox {
            child: self.child,
            controller,
            at,
        };
        build_menu(
            cx,
            AnyView::new(region),
            self.items,
            controller,
            Prop::Dynamic(Box::new(move || at.get())),
        )
    }
}

/// The context menu's target: the child, opening the menu on a secondary press or
/// Shift+F10 (WIDGETS-MENU-08).
struct RegionBox {
    child: AnyView,
    controller: MenuController,
    at: Signal<Anchor>,
}

impl View for RegionBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (controller, at) = (self.controller, self.at);
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [self.child],
        );
        let place = move |x: f32, y: f32| {
            at.set(Anchor::new(Alignment::TOP_LEFT, Alignment::TOP_LEFT).offset(x, y));
            controller.open();
        };
        cx.on_pointer(id, Phase::Capture, move |p| {
            if p.event.kind == PointerKind::Down(PointerButton::Secondary) {
                place(p.local.x, p.local.y);
            }
            Handled::Continue
        });
        cx.on_key(id, Phase::Bubble, move |k| {
            let shift_only = Modifiers {
                shift: true,
                ..Default::default()
            };
            if k.event.pressed
                && k.event.modifiers == shift_only
                && k.event.key == LogicalKey::Named(NamedKey::F(10))
            {
                place(0.0, 0.0);
                return Handled::Stop;
            }
            Handled::Continue
        });
        id
    }
}

// ---------------------------------------------------------------------------------------
// Shared menu machinery.

/// Item height (WIDGETS-MENU-01).
const ITEM_HEIGHT: f32 = 40.0;
/// Padding above and below the items.
const PAD_V: f32 = 8.0;
/// Item side padding.
const PAD_H: f32 = 12.0;
/// The leading slot's size.
const LEAD: f32 = 24.0;
/// The space after the leading slot.
const LEAD_GAP: f32 = 12.0;
/// The space before the trailing shortcut or chevron.
const TRAIL_GAP: f32 = 24.0;
/// Menu widths.
const MIN_WIDTH: f32 = 112.0;
const MAX_WIDTH: f32 = 280.0;
/// A divider's height: 8 px, the line, 8 px.
const DIVIDER_HEIGHT: f32 = 17.0;
/// The highlight layer.
const HIGHLIGHT_ALPHA: f32 = 0.08;
/// The surface's corner radius.
const RADIUS: f32 = 4.0;

/// One menu chain (a root menu and its submenus): how to close it, and where the focus was.
struct Chain {
    /// Closes every menu of the chain.
    close: Rc<dyn Fn()>,
    previous: Cell<Option<ElementId>>,
}

/// Focuses the pending target once, at a frame, if nothing else took the focus.
struct Restore {
    target: Rc<Cell<Option<ElementId>>>,
    only_if_unfocused: bool,
}

impl View for Restore {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if let Some(target) = self.target.take() {
            if !self.only_if_unfocused || cx.tree().focused().is_none() {
                cx.focus(target);
            }
        }
        cx.region(|_| {})
    }
}

/// A portal around `child` showing `items` while `controller` is open, and restoring the
/// focus when the menu closes (WIDGETS-MENU-01, -03).
fn build_menu(
    cx: &mut BuildCx<'_>,
    child: AnyView,
    items: Vec<MenuItem>,
    controller: MenuController,
    anchor: Prop<Anchor>,
) -> ElementId {
    let chain = Rc::new(Chain {
        close: Rc::new(move || controller.close()),
        previous: Cell::new(None),
    });
    let items: Rc<[MenuItem]> = items.into();
    let has_items = !items.is_empty();
    let portal = {
        let chain = Rc::clone(&chain);
        OverlayPortal::new(child)
            .visible(move || has_items && controller.is_open())
            .anchor(anchor)
            .on_outside_press(move || controller.close())
            .overlay(move || MenuView {
                items: Rc::clone(&items),
                chain: Rc::clone(&chain),
                parent: None,
                sideways: None,
                take_focus: true,
                highlight_first: false,
            })
    };
    let target: Rc<Cell<Option<ElementId>>> = Rc::default();
    let moves = signal(0u32);
    {
        let (chain, target) = (Rc::clone(&chain), Rc::clone(&target));
        let was_open = Cell::new(false);
        effect(move || {
            let open = controller.is_open();
            if was_open.replace(open) && !open {
                if let Some(previous) = chain.previous.take() {
                    target.set(Some(previous));
                    untrack(|| moves.update(|m| *m = m.wrapping_add(1)));
                }
            }
        });
    }
    cx.region(|cx| {
        portal.build(cx);
        Dyn::new(move || {
            moves.get();
            Restore {
                target: Rc::clone(&target),
                only_if_unfocused: true,
            }
        })
        .build(cx);
    })
}

/// Whether an open menu was opened by the keyboard, keeping its last value while the menu
/// closes, so closing doesn't rebuild the menu (which would take the focus on its way out).
fn sticky(keyboard: Memo<Option<bool>>) -> Memo<bool> {
    let last = Cell::new(false);
    memo(move || {
        if let Some(flag) = keyboard.get() {
            last.set(flag);
        }
        last.get()
    })
}

/// How a submenu reaches its parent menu.
#[derive(Clone)]
struct ParentLink {
    /// Closes this submenu and focuses the parent with its item highlighted.
    close: Rc<dyn Fn()>,
}

/// One menu level (WIDGETS-MENU-01..07, -09).
struct MenuView {
    items: Rc<[MenuItem]>,
    chain: Rc<Chain>,
    parent: Option<ParentLink>,
    /// For a menu bar's menu: moves to the previous (false) or next (true) top menu.
    sideways: Option<Rc<dyn Fn(bool)>>,
    take_focus: bool,
    highlight_first: bool,
}

/// What a menu's items share.
#[derive(Clone)]
struct Level {
    items: Rc<[MenuItem]>,
    chain: Rc<Chain>,
    highlighted: Signal<Option<usize>>,
    /// The open submenu's index and whether the keyboard opened it.
    open_sub: Signal<Option<(usize, bool)>>,
    menu: Rc<Cell<Option<ElementId>>>,
    refocus: Rc<Cell<Option<ElementId>>>,
    moves: Signal<u32>,
}

impl Level {
    fn selectable(&self, i: usize) -> bool {
        self.items.get(i).is_some_and(|item| {
            !matches!(item.kind, ItemKind::Divider) && untrack(|| (item.enabled)())
        })
    }

    /// The next selectable item after `from` (or before, `forward` false), wrapping; from
    /// none, the first (or last).
    fn step(&self, from: Option<usize>, forward: bool) -> Option<usize> {
        let n = self.items.len();
        (1..=n)
            .map(|k| match (from, forward) {
                (Some(i), true) => (i + k) % n,
                (Some(i), false) => (i + n - k % n) % n,
                (None, true) => k - 1,
                (None, false) => n - k,
            })
            .find(|i| self.selectable(*i))
    }

    /// Chooses item `i`: an action closes the chain and runs; a submenu opens.
    fn activate(&self, i: usize, keyboard: bool) {
        if !self.selectable(i) {
            return;
        }
        match &self.items[i].kind {
            ItemKind::Submenu(_) => self.open_sub.set(Some((i, keyboard))),
            ItemKind::Action => {
                (self.chain.close)();
                if let Some(f) = &self.items[i].on_select {
                    f();
                }
            }
            ItemKind::Divider => {}
        }
    }

    /// Focuses this menu again at the next frame (after a submenu closed).
    fn refocus(&self) {
        self.refocus.set(self.menu.get());
        untrack(|| self.moves.update(|m| *m = m.wrapping_add(1)));
    }
}

impl View for MenuView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let rtl = is_rtl(cx);
        let items = self.items;
        let level = Level {
            items: Rc::clone(&items),
            chain: Rc::clone(&self.chain),
            highlighted: signal(None),
            open_sub: signal(None),
            menu: Rc::default(),
            refocus: Rc::default(),
            moves: signal(0),
        };
        if self.highlight_first {
            level.highlighted.set(level.step(None, true));
        }
        let has_leading = items
            .iter()
            .any(|i| i.icon.is_some() || i.checked.is_some());
        let ids: Rc<RefCell<Vec<Option<ElementId>>>> =
            Rc::new(RefCell::new(vec![None; items.len()]));
        let mut children: Vec<AnyView> = Vec::new();
        for (i, item) in items.iter().enumerate() {
            match &item.kind {
                ItemKind::Divider => children.push(AnyView::new(DividerRow)),
                ItemKind::Action => children.push(AnyView::new(ItemRow {
                    index: i,
                    item: item.clone(),
                    level: level.clone(),
                    has_leading,
                    ids: Rc::clone(&ids),
                })),
                ItemKind::Submenu(sub) => {
                    let sub: Rc<[MenuItem]> = sub.clone().into();
                    let open_sub = level.open_sub;
                    let keyboard = memo(move || open_sub.get().filter(|s| s.0 == i).map(|s| s.1));
                    let link = {
                        let level = level.clone();
                        ParentLink {
                            close: Rc::new(move || {
                                level.open_sub.set(None);
                                level.highlighted.set(Some(i));
                                level.refocus();
                            }),
                        }
                    };
                    let chain = Rc::clone(&self.chain);
                    let row = ItemRow {
                        index: i,
                        item: item.clone(),
                        level: level.clone(),
                        has_leading,
                        ids: Rc::clone(&ids),
                    };
                    // A submenu opens toward the end edge (WIDGETS-RTL-05).
                    let rtl = Rc::clone(&rtl);
                    children.push(AnyView::new(
                        OverlayPortal::new(row)
                            .visible(move || keyboard.get().is_some())
                            .anchor(move || {
                                let (target, follower) = if rtl() {
                                    (Alignment::TOP_LEFT, Alignment::TOP_RIGHT)
                                } else {
                                    (Alignment::TOP_RIGHT, Alignment::TOP_LEFT)
                                };
                                Anchor::new(target, follower).offset(0.0, -PAD_V)
                            })
                            .overlay(move || {
                                let (sub, chain, link) =
                                    (Rc::clone(&sub), Rc::clone(&chain), link.clone());
                                let by_keyboard = sticky(keyboard);
                                Dyn::new(move || {
                                    let by_keyboard = by_keyboard.get();
                                    MenuView {
                                        items: Rc::clone(&sub),
                                        chain: Rc::clone(&chain),
                                        parent: Some(link.clone()),
                                        sideways: None,
                                        take_focus: by_keyboard,
                                        highlight_first: by_keyboard,
                                    }
                                })
                            }),
                    ));
                }
            }
        }
        {
            let (target, moves) = (Rc::clone(&level.refocus), level.moves);
            children.push(AnyView::new(Dyn::new(move || {
                moves.get();
                Restore {
                    target: Rc::clone(&target),
                    only_if_unfocused: false,
                }
            })));
        }
        let id = cx.render(MenuRender, children);
        level.menu.set(Some(id));
        cx.set_paint(
            id,
            MenuPaint {
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<MenuPaint>(|p| p.colors = colors);
        });
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Menu,
                focusable: true,
                ..Default::default()
            },
        );
        {
            let (highlighted, ids) = (level.highlighted, Rc::clone(&ids));
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || highlighted.get())),
                move |e, highlighted| {
                    let active = highlighted.and_then(|i| ids.borrow().get(i).copied().flatten());
                    e.update_semantics(|s| s.active_descendant = active);
                },
            );
        }
        cx.focusable(
            id,
            FocusOptions {
                traversable: false,
                focus_on_press: false,
            },
        );
        cx.focus_scope(id);
        // Keys (WIDGETS-MENU-04).
        {
            let (level, parent, sideways) =
                (level.clone(), self.parent.clone(), self.sideways.clone());
            let rtl = Rc::clone(&rtl);
            cx.on_key(id, Phase::Bubble, move |k| {
                let event = &k.event;
                if !event.pressed || event.modifiers != Modifiers::default() {
                    return Handled::Continue;
                }
                // Left and Right swap under RTL (WIDGETS-RTL-05).
                let key = directional_key(&event.key, rtl());
                let current = level.highlighted.get_untracked();
                let move_to = |i: Option<usize>| {
                    if i.is_some() {
                        level.highlighted.set(i);
                    }
                };
                match &key {
                    LogicalKey::Named(NamedKey::ArrowDown) => move_to(level.step(current, true)),
                    LogicalKey::Named(NamedKey::ArrowUp) => move_to(level.step(current, false)),
                    LogicalKey::Named(NamedKey::Home) => move_to(level.step(None, true)),
                    LogicalKey::Named(NamedKey::End) => move_to(level.step(None, false)),
                    LogicalKey::Named(NamedKey::Enter) => {
                        if let Some(i) = current {
                            level.activate(i, true);
                        }
                    }
                    LogicalKey::Character(c) if &**c == " " => {
                        if let Some(i) = current {
                            level.activate(i, true);
                        }
                    }
                    LogicalKey::Named(NamedKey::ArrowRight) => match (current, &sideways) {
                        (Some(i), _) if matches!(level.items[i].kind, ItemKind::Submenu(_)) => {
                            level.activate(i, true);
                        }
                        // A menu bar's menu moves to the next top menu (WIDGETS-MENUBAR-03).
                        (_, Some(sideways)) => sideways(true),
                        _ => return Handled::Continue,
                    },
                    LogicalKey::Named(NamedKey::ArrowLeft) => match (&parent, &sideways) {
                        (Some(parent), _) => (parent.close)(),
                        (None, Some(sideways)) => sideways(false),
                        _ => return Handled::Continue,
                    },
                    LogicalKey::Named(NamedKey::Escape) => match &parent {
                        Some(parent) => (parent.close)(),
                        None => (level.chain.close)(),
                    },
                    _ => return Handled::Continue,
                }
                Handled::Stop
            });
        }
        if self.take_focus {
            // The first focus recorded is kept, so switching a bar's menus keeps the
            // focus to return to.
            if self.parent.is_none() && self.chain.previous.get().is_none() {
                self.chain.previous.set(cx.tree().focused());
            }
            cx.focus(id);
        }
        id
    }
}

/// The menu's items stacked, as wide as the widest (WIDGETS-MENU-01).
#[derive(Clone, PartialEq)]
struct MenuRender;

impl RenderBox for MenuRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let mut widest = 0.0f32;
        for i in 0..children.len() {
            let size = children.layout(i, BoxConstraints::new(0.0, MAX_WIDTH, 0.0, f32::INFINITY));
            widest = widest.max(size.width);
        }
        let width = widest.clamp(MIN_WIDTH, MAX_WIDTH);
        let mut y = PAD_V;
        for i in 0..children.len() {
            let size = children.layout(i, BoxConstraints::new(width, width, 0.0, f32::INFINITY));
            children.set_offset(i, Vec2::new(0.0, y));
            y += size.height;
        }
        c.constrain(Size::new(width, y + PAD_V))
    }
}

/// The menu's surface and shadow.
struct MenuPaint {
    colors: ColorScheme,
}

impl Paint for MenuPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let shape = RoundedRect::new(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            BorderRadius::circular(RADIUS),
        );
        let scene = cx.scene();
        // Material's level-2 elevation: a tight key shadow and a wider ambient one.
        for (offset, blur, spread, alpha) in [(1.0, 2.0, 0.0, 0.3), (2.0, 6.0, 2.0, 0.15)] {
            scene.box_shadow(BoxShadow {
                shape,
                color: Color::BLACK.with_alpha(alpha),
                offset: Vec2::new(0.0, offset),
                blur_radius: blur,
                spread_radius: spread,
            });
        }
        scene.fill(shape, self.colors.surface);
    }

    fn overflow(&self) -> f32 {
        10.0
    }
}

/// A divider row: a 1 px line between 8 px gaps.
struct DividerRow;

impl View for DividerRow {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::new(
                0.0,
                f32::INFINITY,
                DIVIDER_HEIGHT,
                DIVIDER_HEIGHT,
            )),
            [],
        );
        cx.set_paint(
            id,
            DividerPaint {
                color: theme.colors().outline_variant,
            },
        );
        cx.bind(id, theme.color_prop(|c| c.outline_variant), |e, color| {
            e.update_paint::<DividerPaint>(|p| p.color = color);
        });
        id
    }
}

struct DividerPaint {
    color: Color,
}

impl Paint for DividerPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let width = cx.size().width;
        cx.scene()
            .fill_rect(Rect::from_ltrb(0.0, PAD_V, width, PAD_V + 1.0), self.color);
    }
}

/// One item (WIDGETS-MENU-01, -02, -05, -07, -09).
struct ItemRow {
    index: usize,
    item: MenuItem,
    level: Level,
    has_leading: bool,
    ids: Rc<RefCell<Vec<Option<ElementId>>>>,
}

impl View for ItemRow {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (i, item, level) = (self.index, self.item, self.level);
        let theme = ThemeHandle::of(cx);
        let enabled = Rc::clone(&item.enabled);
        let ink = {
            let enabled = Rc::clone(&enabled);
            move |color: Color| {
                if enabled() {
                    color
                } else {
                    at(color, ALPHA_38)
                }
            }
        };
        let lead: AnyView =
            if !self.has_leading {
                AnyView::new(SizedBox::new(0.0, 0.0))
            } else if let Some(checked) = item.checked.clone() {
                let ink = ink.clone();
                AnyView::new(SizedBox::new(LEAD, LEAD).child(
                    Icon::new(Icons::CHECK).size(20.0).color(move || {
                        if checked() {
                            ink(theme.colors().on_surface)
                        } else {
                            Color::TRANSPARENT
                        }
                    }),
                ))
            } else if let Some(icon) = item.icon {
                let ink = ink.clone();
                AnyView::new(
                    SizedBox::new(LEAD, LEAD).child(
                        Icon::new(icon)
                            .size(20.0)
                            .color(move || ink(theme.colors().on_surface_variant)),
                    ),
                )
            } else {
                AnyView::new(SizedBox::new(LEAD, LEAD))
            };
        let label = {
            let ink = ink.clone();
            Text::new(item.label.clone())
                .style(theme.snapshot().text.label)
                .color(move || ink(theme.colors().on_surface))
        };
        let trail: AnyView = match (&item.kind, &item.shortcut) {
            (ItemKind::Submenu(_), _) => {
                let ink = ink.clone();
                let rtl = is_rtl(cx);
                AnyView::new(
                    Icon::new(move || {
                        if rtl() {
                            Icons::CHEVRON_LEFT
                        } else {
                            Icons::CHEVRON_RIGHT
                        }
                    })
                    .size(20.0)
                    .color(move || ink(theme.colors().on_surface_variant)),
                )
            }
            (_, Some(shortcut)) => {
                let ink = ink.clone();
                AnyView::new(
                    Text::new(shortcut.clone())
                        .style(theme.snapshot().text.label)
                        .color(move || ink(theme.colors().on_surface_variant)),
                )
            }
            _ => AnyView::new(SizedBox::new(0.0, 0.0)),
        };
        let id = cx.render(ItemRender, [lead, AnyView::new(label), trail]);
        if let Some(slot) = self.ids.borrow_mut().get_mut(i) {
            *slot = Some(id);
        }
        let highlighted = level.highlighted;
        cx.set_paint(
            id,
            ItemPaint {
                highlighted: false,
                colors: theme.colors(),
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (highlighted.get() == Some(i), theme.colors())
            })),
            |e, (on, colors)| {
                e.update_paint::<ItemPaint>(|p| {
                    p.highlighted = on;
                    p.colors = colors;
                });
            },
        );
        let submenu = matches!(item.kind, ItemKind::Submenu(_));
        cx.set_semantics(
            id,
            Semantics {
                role: if item.checked.is_some() {
                    Role::MenuItemCheckBox
                } else {
                    Role::MenuItem
                },
                name: Some(item.label.clone()),
                actions: SemanticsActions {
                    click: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        {
            let (enabled, checked, open_sub) =
                (Rc::clone(&enabled), item.checked.clone(), level.open_sub);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    (
                        enabled(),
                        checked.as_ref().map(|c| c()),
                        submenu.then(|| open_sub.get().is_some_and(|s| s.0 == i)),
                    )
                })),
                |e, (enabled, checked, expanded)| {
                    e.update_semantics(|s| {
                        s.disabled = !enabled;
                        s.toggled = checked.map(|c| if c { Toggled::True } else { Toggled::False });
                        s.expanded = expanded;
                    });
                },
            );
        }
        let pressed = Cell::new(false);
        cx.on_pointer(id, Phase::Bubble, move |p| {
            match p.event.kind {
                PointerKind::Enter => {
                    if level.selectable(i) {
                        level.highlighted.set(Some(i));
                        let open = level.open_sub.get_untracked();
                        if submenu {
                            if open.map(|s| s.0) != Some(i) {
                                level.open_sub.set(Some((i, false)));
                            }
                        } else if open.is_some() {
                            level.open_sub.set(None);
                        }
                    }
                }
                PointerKind::Down(PointerButton::Primary) => pressed.set(true),
                PointerKind::Up(PointerButton::Primary) => {
                    let inside = p.local.x >= 0.0
                        && p.local.y >= 0.0
                        && p.local.x <= p.size.width
                        && p.local.y <= p.size.height;
                    if pressed.replace(false) && inside {
                        level.activate(i, false);
                    }
                }
                _ => {}
            }
            Handled::Continue
        });
        id
    }
}

/// An item's lead, label and trailing parts in a 40 px row (WIDGETS-MENU-01).
#[derive(Clone, PartialEq)]
struct ItemRender;

impl RenderBox for ItemRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let loose = BoxConstraints::loose(Size::new(f32::INFINITY, ITEM_HEIGHT));
        let sizes: Vec<Size> = (0..children.len())
            .map(|i| children.layout(i, loose))
            .collect();
        let lead = sizes.first().copied().unwrap_or(Size::ZERO);
        let label = sizes.get(1).copied().unwrap_or(Size::ZERO);
        let trail = sizes.get(2).copied().unwrap_or(Size::ZERO);
        let lead_space = if lead.width > 0.0 {
            lead.width + LEAD_GAP
        } else {
            0.0
        };
        let trail_space = if trail.width > 0.0 {
            TRAIL_GAP + trail.width
        } else {
            0.0
        };
        let natural = PAD_H + lead_space + label.width + trail_space + PAD_H;
        let width = if c.min_width == c.max_width {
            c.max_width
        } else {
            natural.min(c.max_width)
        };
        let centre = |h: f32| (ITEM_HEIGHT - h) / 2.0;
        if !sizes.is_empty() {
            children.set_offset(0, Vec2::new(PAD_H, centre(lead.height)));
        }
        if sizes.len() > 1 {
            children.set_offset(1, Vec2::new(PAD_H + lead_space, centre(label.height)));
        }
        if sizes.len() > 2 {
            children.set_offset(
                2,
                Vec2::new(width - PAD_H - trail.width, centre(trail.height)),
            );
        }
        c.constrain(Size::new(width, ITEM_HEIGHT))
    }
}

/// The highlight layer (WIDGETS-MENU-04).
struct ItemPaint {
    highlighted: bool,
    colors: ColorScheme,
}

impl Paint for ItemPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        if self.highlighted {
            let size = cx.size();
            cx.scene().fill_rect(
                Rect::from_ltrb(0.0, 0.0, size.width, size.height),
                self.colors.on_surface.with_alpha(HIGHLIGHT_ALPHA),
            );
        }
    }
}

/// A row of top-level menus (Flutter's `MenuBar`; WIDGETS-MENUBAR-01..04).
pub struct MenuBar {
    menus: Vec<(String, Vec<MenuItem>)>,
}

impl Default for MenuBar {
    fn default() -> Self {
        MenuBar::new()
    }
}

impl MenuBar {
    /// An empty menu bar.
    pub fn new() -> Self {
        MenuBar { menus: Vec::new() }
    }

    /// Adds a top-level menu labelled `label`.
    pub fn menu(
        mut self,
        label: impl Into<String>,
        items: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        self.menus.push((label.into(), items.into_iter().collect()));
        self
    }
}

/// The menu bar's height (WIDGETS-MENUBAR-01).
const BAR_HEIGHT: f32 = 40.0;

/// What the bar's parts share.
#[derive(Clone)]
struct Bar {
    /// The open top menu and whether the keyboard opened it.
    open: Signal<Option<(usize, bool)>>,
    current: Signal<usize>,
    focused: Signal<bool>,
    count: usize,
}

impl View for MenuBar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let count = self.menus.len();
        let bar = Bar {
            open: signal(None),
            current: signal(0),
            focused: signal(false),
            count,
        };
        let open = bar.open;
        let chain = Rc::new(Chain {
            close: Rc::new(move || {
                if open.get_untracked().is_some() {
                    open.set(None);
                }
            }),
            previous: Cell::new(None),
        });
        let ids: Rc<RefCell<Vec<Option<ElementId>>>> = Rc::new(RefCell::new(vec![None; count]));
        let mut children: Vec<AnyView> = self
            .menus
            .into_iter()
            .enumerate()
            .map(|(i, (label, items))| {
                AnyView::new(top_menu(
                    i,
                    label,
                    items,
                    bar.clone(),
                    Rc::clone(&chain),
                    Rc::clone(&ids),
                ))
            })
            .collect();
        // Focus goes back when the bar's menus close (WIDGETS-MENUBAR-02).
        let target: Rc<Cell<Option<ElementId>>> = Rc::default();
        let moves = signal(0u32);
        {
            let (chain, target) = (Rc::clone(&chain), Rc::clone(&target));
            let was_open = Cell::new(false);
            effect(move || {
                let is_open = open.get().is_some();
                if was_open.replace(is_open) && !is_open {
                    if let Some(previous) = chain.previous.take() {
                        target.set(Some(previous));
                        untrack(|| moves.update(|m| *m = m.wrapping_add(1)));
                    }
                }
            });
        }
        children.push(AnyView::new(Dyn::new(move || {
            moves.get();
            Restore {
                target: Rc::clone(&target),
                only_if_unfocused: true,
            }
        })));
        let id = cx.render(BarRender, children);
        cx.set_paint(
            id,
            BarPaint {
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<BarPaint>(|p| p.colors = colors);
        });
        cx.set_semantics(
            id,
            Semantics {
                role: Role::MenuBar,
                focusable: count > 0,
                actions: SemanticsActions {
                    focus: count > 0,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        if count == 0 {
            return id;
        }
        // One Tab stop that a press doesn't take (WIDGETS-MENUBAR-02, -03).
        cx.focusable(
            id,
            FocusOptions {
                traversable: true,
                focus_on_press: false,
            },
        );
        let (current, focused) = (bar.current, bar.focused);
        cx.on_focus_change(id, move |is| focused.set(is));
        {
            let ids = Rc::clone(&ids);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || (focused.get(), current.get()))),
                move |e, (focused, current)| {
                    let active = focused
                        .then(|| ids.borrow().get(current).copied().flatten())
                        .flatten();
                    e.update_semantics(|s| s.active_descendant = active);
                },
            );
        }
        let rtl = is_rtl(cx);
        cx.on_key(id, Phase::Bubble, move |k| {
            let event = &k.event;
            if !event.pressed || event.modifiers != Modifiers::default() {
                return Handled::Continue;
            }
            let i = current.get_untracked();
            // The bar runs right to left under RTL (WIDGETS-RTL-05).
            match &directional_key(&event.key, rtl()) {
                LogicalKey::Named(NamedKey::ArrowLeft) => current.set((i + count - 1) % count),
                LogicalKey::Named(NamedKey::ArrowRight) => current.set((i + 1) % count),
                LogicalKey::Named(NamedKey::ArrowDown | NamedKey::Enter) => {
                    open.set(Some((i, true)));
                }
                LogicalKey::Character(c) if &**c == " " => open.set(Some((i, true))),
                _ => return Handled::Continue,
            }
            Handled::Stop
        });
        id
    }
}

/// Top-level menu `i`: its label in the bar and its menu below it.
fn top_menu(
    i: usize,
    label: String,
    items: Vec<MenuItem>,
    bar: Bar,
    chain: Rc<Chain>,
    ids: Rc<RefCell<Vec<Option<ElementId>>>>,
) -> impl View {
    let items: Rc<[MenuItem]> = items.into();
    let open = bar.open;
    let keyboard = memo(move || open.get().filter(|o| o.0 == i).map(|o| o.1));
    let count = bar.count;
    let (current, has_items) = (bar.current, !items.is_empty());
    let sideways: Rc<dyn Fn(bool)> = Rc::new(move |forward| {
        let j = if forward {
            (i + 1) % count
        } else {
            (i + count - 1) % count
        };
        current.set(j);
        open.set(Some((j, true)));
    });
    OverlayPortal::new(TopLabel {
        index: i,
        label,
        bar,
        ids,
    })
    .visible(move || has_items && keyboard.get().is_some())
    .anchor(Anchor::below())
    .on_outside_press(move || {
        if open.get_untracked().is_some_and(|o| o.0 == i) {
            open.set(None);
        }
    })
    .overlay(move || {
        let (items, chain, sideways) = (Rc::clone(&items), Rc::clone(&chain), Rc::clone(&sideways));
        let by_keyboard = sticky(keyboard);
        Dyn::new(move || {
            let by_keyboard = by_keyboard.get();
            MenuView {
                items: Rc::clone(&items),
                chain: Rc::clone(&chain),
                parent: None,
                sideways: Some(Rc::clone(&sideways)),
                take_focus: true,
                highlight_first: by_keyboard,
            }
        })
    })
}

/// A top-level item's label in the bar (WIDGETS-MENUBAR-01, -02, -04).
struct TopLabel {
    index: usize,
    label: String,
    bar: Bar,
    ids: Rc<RefCell<Vec<Option<ElementId>>>>,
}

impl View for TopLabel {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (i, bar) = (self.index, self.bar);
        let theme = ThemeHandle::of(cx);
        let text = Text::new(self.label.clone())
            .style(theme.snapshot().text.label)
            .color(move || theme.colors().on_surface);
        let id = cx.render(TopRender, [AnyView::new(text)]);
        if let Some(slot) = self.ids.borrow_mut().get_mut(i) {
            *slot = Some(id);
        }
        let (open, current, focused) = (bar.open, bar.current, bar.focused);
        cx.set_paint(
            id,
            TopPaint {
                layer: false,
                ring: false,
                colors: theme.colors(),
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                let is_open = open.get().is_some_and(|o| o.0 == i);
                let is_current = focused.get() && current.get() == i;
                (is_open || is_current, is_current, theme.colors())
            })),
            |e, (layer, ring, colors)| {
                e.update_paint::<TopPaint>(|p| {
                    p.layer = layer;
                    p.ring = ring;
                    p.colors = colors;
                });
            },
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::MenuItem,
                name: Some(self.label),
                actions: SemanticsActions {
                    click: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || open.get().is_some_and(|o| o.0 == i))),
            |e, expanded| {
                e.update_semantics(|s| s.expanded = Some(expanded));
            },
        );
        cx.on_pointer(id, Phase::Bubble, move |p| match p.event.kind {
            PointerKind::Down(PointerButton::Primary) => {
                current.set(i);
                if open.get_untracked().is_some_and(|o| o.0 == i) {
                    open.set(None);
                } else {
                    open.set(Some((i, false)));
                }
                Handled::Stop
            }
            PointerKind::Enter => {
                if open.get_untracked().is_some_and(|o| o.0 != i) {
                    current.set(i);
                    open.set(Some((i, false)));
                }
                Handled::Continue
            }
            _ => Handled::Continue,
        });
        id
    }
}

/// Top-level items side by side at their natural widths, the bar filling its width.
#[derive(Clone, PartialEq)]
struct BarRender;

impl RenderBox for BarRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let mut x = 0.0;
        for i in 0..children.len() {
            let size = children.layout(
                i,
                BoxConstraints::new(0.0, f32::INFINITY, BAR_HEIGHT, BAR_HEIGHT),
            );
            children.set_offset(i, Vec2::new(x, 0.0));
            x += size.width;
        }
        let width = if c.has_bounded_width() {
            c.max_width
        } else {
            x
        };
        c.constrain(Size::new(width, BAR_HEIGHT))
    }
}

/// A top-level item: its label centred with 12 px on each side.
#[derive(Clone, PartialEq)]
struct TopRender;

impl RenderBox for TopRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let mut label = Size::ZERO;
        for i in 0..children.len() {
            label = children.layout(
                i,
                BoxConstraints::loose(Size::new(f32::INFINITY, c.max_height)),
            );
        }
        let size = c.constrain(Size::new(label.width + 2.0 * PAD_H, c.max_height));
        for i in 0..children.len() {
            children.set_offset(
                i,
                Vec2::new(PAD_H, ((size.height - label.height) / 2.0).max(0.0)),
            );
        }
        size
    }
}

/// The bar's surface and bottom line.
struct BarPaint {
    colors: ColorScheme,
}

impl Paint for BarPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let scene = cx.scene();
        scene.fill_rect(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            self.colors.surface,
        );
        scene.fill_rect(
            Rect::from_ltrb(0.0, size.height - 1.0, size.width, size.height),
            self.colors.outline_variant,
        );
    }
}

/// A top-level item's layer and focus ring.
struct TopPaint {
    layer: bool,
    ring: bool,
    colors: ColorScheme,
}

impl Paint for TopPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let bounds = Rect::from_ltrb(0.0, 0.0, size.width, size.height);
        if self.layer {
            cx.scene()
                .fill_rect(bounds, self.colors.on_surface.with_alpha(HIGHLIGHT_ALPHA));
        }
        if self.ring {
            cx.scene()
                .stroke(RoundedRect::from_rect(bounds), 2.0, self.colors.primary);
        }
    }
}
