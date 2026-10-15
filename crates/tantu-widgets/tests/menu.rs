//! Tests for `docs/specs/widgets/menu.md`, rules WIDGETS-MENU-01..09.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Point, Rect};
use tantu_view::layout::RenderConstrainedBox;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{
    BuildCx, ElementId, FocusOptions, Handled, Keyed, LogicalKey, Modifiers, NamedKey, Phase,
    PointerButton, PointerEvent, PointerKind, View,
};
use tantu_widgets::{
    Align, Button, Column, ContextMenuRegion, MenuAnchor, MenuController, MenuItem, Padding,
    SizedBox, Stack,
};

const W: f32 = 400.0;
const H: f32 = 300.0;

type Log = Rc<RefCell<Vec<String>>>;

fn record(log: &Log, name: &str) -> impl Fn() + 'static {
    let (log, name) = (log.clone(), name.to_owned());
    move || log.borrow_mut().push(name.clone())
}

/// Cut, Copy, divider, Word wrap (checked), Open recent ▸ (a.txt, b.txt disabled, c.txt),
/// Print (disabled).
fn items(log: &Log, wrap: Signal<bool>) -> Vec<MenuItem> {
    let toggle = {
        let log = log.clone();
        move || {
            log.borrow_mut().push("wrap".to_owned());
            wrap.update(|w| *w = !*w);
        }
    };
    vec![
        MenuItem::new("Cut")
            .shortcut("Ctrl+X")
            .on_select(record(log, "cut")),
        MenuItem::new("Copy")
            .shortcut("Ctrl+C")
            .on_select(record(log, "copy")),
        MenuItem::divider(),
        MenuItem::new("Word wrap").checked(wrap).on_select(toggle),
        MenuItem::submenu(
            "Open recent",
            [
                MenuItem::new("a.txt").on_select(record(log, "a.txt")),
                MenuItem::new("b.txt")
                    .enabled(false)
                    .on_select(record(log, "b.txt")),
                MenuItem::new("c.txt").on_select(record(log, "c.txt")),
            ],
        ),
        MenuItem::new("Print")
            .enabled(false)
            .on_select(record(log, "print")),
    ]
}

struct Fx {
    t: WidgetTester,
    controller: MenuController,
    wrap: Signal<bool>,
    log: Log,
}

/// An "Open" button at `(x, 20)` toggling a menu, and an "Other" button below.
fn fixture_at(x: f32) -> Fx {
    let log: Log = Rc::default();
    type Out = Rc<RefCell<Option<(MenuController, Signal<bool>)>>>;
    let out: Out = Rc::default();
    let t = {
        let (log, out) = (log.clone(), out.clone());
        WidgetTester::with_size(W, H, move || {
            let controller = MenuController::new();
            let wrap = signal(true);
            *out.borrow_mut() = Some((controller, wrap));
            Padding::new(tantu_view::core::EdgeInsets::from_ltrb(x, 20.0, 0.0, 0.0)).child(
                Column::new()
                    .child(Keyed::new(
                        "anchor",
                        MenuAnchor::new(
                            controller,
                            Keyed::new(
                                "open",
                                Button::new("Open").on_press(move || controller.toggle()),
                            ),
                        )
                        .items(items(&log, wrap)),
                    ))
                    .child(SizedBox::height(150.0))
                    .child(Keyed::new("other", Button::new("Other"))),
            )
        })
    };
    let (controller, wrap) = out.borrow_mut().take().expect("built");
    let mut fx = Fx {
        t,
        controller,
        wrap,
        log,
    };
    fx.pump();
    fx
}

fn fixture() -> Fx {
    fixture_at(20.0)
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
}

fn is_item(role: accesskit::Role) -> bool {
    matches!(
        role,
        accesskit::Role::MenuItem | accesskit::Role::MenuItemCheckBox
    )
}

impl Fx {
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
    }

    fn open(&mut self) {
        let c = self.controller;
        self.t.enter(|| c.open());
        self.pump();
    }

    fn menus(&mut self) -> Vec<(ElementId, accesskit::Node)> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .filter(|(_, n)| n.role() == accesskit::Role::Menu)
            .map(|(id, n)| (element(id), n))
            .collect()
    }

    fn item(&mut self, name: &str) -> (ElementId, accesskit::Node) {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .find(|(_, n)| is_item(n.role()) && n.label() == Some(name))
            .map(|(id, n)| (element(id), n))
            .unwrap_or_else(|| panic!("no item {name}"))
    }

    fn item_rect(&mut self, name: &str) -> Rect {
        let id = self.item(name).0;
        self.t.rect(id)
    }

    fn item_names(&mut self) -> Vec<String> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .filter(|(_, n)| is_item(n.role()))
            .map(|(_, n)| n.label().unwrap_or_default().to_owned())
            .collect()
    }

    /// The label of the innermost (last) menu's highlighted item.
    fn highlighted(&mut self) -> Option<String> {
        let tree = self.t.a11y_tree();
        let menu = tree
            .nodes
            .iter()
            .rfind(|(_, n)| n.role() == accesskit::Role::Menu)?
            .1
            .clone();
        let active = menu.active_descendant()?;
        tree.nodes
            .iter()
            .find(|(id, _)| *id == active)
            .and_then(|(_, n)| n.label().map(str::to_owned))
    }

    fn pointer(&mut self, kind: PointerKind, at: Point) {
        self.t.pointer(PointerEvent {
            kind,
            position: at,
            modifiers: Modifiers::default(),
        });
        self.pump();
    }

    fn click(&mut self, at: Point) {
        self.pointer(PointerKind::Down(PointerButton::Primary), at);
        self.pointer(PointerKind::Up(PointerButton::Primary), at);
    }

    fn key(&mut self, key: NamedKey) {
        self.t
            .press_key(LogicalKey::Named(key), Modifiers::default());
        self.pump();
    }

    fn log(&self) -> Vec<String> {
        self.log.borrow().clone()
    }

    fn focused(&mut self) -> Option<ElementId> {
        ElementId::from_raw(self.t.a11y_tree().focus.0)
    }
}

#[test]
fn widgets_menu_01_layout() {
    let mut fx = fixture();
    assert!(fx.menus().is_empty());
    fx.open();
    let menus = fx.menus();
    assert_eq!(menus.len(), 1);
    let menu = fx.t.rect(menus[0].0);
    let anchor = fx.t.rect(fx.t.find(&Finder::key("open")));
    // Below the child, start edges aligned.
    assert_eq!((menu.left, menu.top), (anchor.left, anchor.bottom));
    assert!((112.0..=280.0).contains(&menu.width()), "{menu:?}");
    // Items: 40 px tall from 8 px below the top; dividers have no node.
    assert_eq!(
        fx.item_names(),
        ["Cut", "Copy", "Word wrap", "Open recent", "Print"]
    );
    let cut = fx.item_rect("Cut");
    assert_eq!((cut.top, cut.height()), (menu.top + 8.0, 40.0));
    assert_eq!((cut.left, cut.right), (menu.left, menu.right));
    let copy = fx.item_rect("Copy");
    let wrap = fx.item_rect("Word wrap");
    assert_eq!(copy.top, cut.bottom);
    assert_eq!(wrap.top, copy.bottom + 17.0, "8 + 1 + 8 px of divider");
    // The shortcut hint shows.
    fx.t.find(&Finder::text("Ctrl+X"));
    // A menu with no items shows nothing.
    let mut t = WidgetTester::with_size(W, H, || {
        let controller = MenuController::new();
        controller.open();
        MenuAnchor::new(controller, Button::new("Empty"))
    });
    t.pump();
    assert!(
        !t.a11y_tree()
            .nodes
            .iter()
            .any(|(_, n)| n.role() == accesskit::Role::Menu)
    );
}

#[test]
fn widgets_menu_02_choosing() {
    let mut fx = fixture();
    fx.open();
    let copy = fx.item_rect("Copy").center();
    fx.click(copy);
    assert_eq!(fx.log(), ["copy"]);
    assert!(fx.menus().is_empty());
    assert!(!fx.t.enter(|| fx.controller.is_open()));
    // A disabled item does nothing and the menu stays open.
    fx.open();
    let print = fx.item_rect("Print").center();
    fx.click(print);
    assert_eq!(fx.log(), ["copy"]);
    assert_eq!(fx.menus().len(), 1);
    // Enter on the highlighted item.
    fx.key(NamedKey::ArrowDown);
    fx.key(NamedKey::Enter);
    assert_eq!(fx.log(), ["copy", "cut"]);
    assert!(fx.menus().is_empty());
}

#[test]
fn widgets_menu_03_closing_and_focus() {
    let mut fx = fixture();
    fx.t.focus(&Finder::key("other"));
    let other = fx.focused();
    fx.open();
    assert_ne!(fx.focused(), other, "the menu takes the focus");
    // Escape closes the innermost menu, then the root; the focus comes back.
    fx.key(NamedKey::End);
    fx.key(NamedKey::ArrowRight);
    assert_eq!(fx.menus().len(), 2);
    fx.key(NamedKey::Escape);
    assert_eq!(fx.menus().len(), 1);
    assert_eq!(fx.highlighted().as_deref(), Some("Open recent"));
    fx.key(NamedKey::Escape);
    assert!(fx.menus().is_empty());
    assert!(!fx.t.enter(|| fx.controller.is_open()));
    assert_eq!(fx.focused(), other);
    // A press outside closes everything.
    fx.open();
    fx.key(NamedKey::End);
    fx.key(NamedKey::ArrowRight);
    fx.click(Point::new(W - 5.0, H - 5.0));
    assert!(fx.menus().is_empty());
    // So does the controller.
    fx.open();
    let c = fx.controller;
    fx.t.enter(|| c.close());
    fx.pump();
    assert!(fx.menus().is_empty());
}

#[test]
fn widgets_menu_04_keyboard() {
    let mut fx = fixture();
    fx.open();
    assert_eq!(fx.highlighted(), None);
    fx.key(NamedKey::ArrowDown);
    assert_eq!(fx.highlighted().as_deref(), Some("Cut"));
    fx.key(NamedKey::ArrowDown);
    fx.key(NamedKey::ArrowDown);
    assert_eq!(
        fx.highlighted().as_deref(),
        Some("Word wrap"),
        "the divider is skipped"
    );
    fx.key(NamedKey::ArrowDown);
    fx.key(NamedKey::ArrowDown);
    assert_eq!(
        fx.highlighted().as_deref(),
        Some("Cut"),
        "Print is skipped, wrapping"
    );
    fx.key(NamedKey::ArrowUp);
    assert_eq!(fx.highlighted().as_deref(), Some("Open recent"));
    fx.key(NamedKey::Home);
    assert_eq!(fx.highlighted().as_deref(), Some("Cut"));
    fx.key(NamedKey::End);
    assert_eq!(fx.highlighted().as_deref(), Some("Open recent"));
    // Right opens the submenu with its first item highlighted.
    fx.key(NamedKey::ArrowRight);
    assert_eq!(fx.menus().len(), 2);
    assert_eq!(fx.highlighted().as_deref(), Some("a.txt"));
    fx.key(NamedKey::ArrowDown);
    assert_eq!(fx.highlighted().as_deref(), Some("c.txt"));
    // Left closes it; Enter on the submenu item opens it again.
    fx.key(NamedKey::ArrowLeft);
    assert_eq!(fx.menus().len(), 1);
    fx.key(NamedKey::Enter);
    assert_eq!(fx.menus().len(), 2);
    assert_eq!(fx.highlighted().as_deref(), Some("a.txt"));
    fx.key(NamedKey::ArrowDown);
    fx.key(NamedKey::Enter);
    assert_eq!(fx.log(), ["c.txt"]);
    assert!(fx.menus().is_empty());
}

#[test]
fn widgets_menu_05_hover() {
    let mut fx = fixture();
    fx.open();
    let copy = fx.item_rect("Copy").center();
    fx.pointer(PointerKind::Move, copy);
    assert_eq!(fx.highlighted().as_deref(), Some("Copy"));
    let recent = fx.item_rect("Open recent").center();
    fx.pointer(PointerKind::Move, recent);
    assert_eq!(fx.menus().len(), 2);
    let cut = fx.item_rect("Cut").center();
    fx.pointer(PointerKind::Move, cut);
    assert_eq!(fx.menus().len(), 1);
}

#[test]
fn widgets_menu_06_submenu_position() {
    let mut fx = fixture();
    fx.open();
    fx.key(NamedKey::End);
    fx.key(NamedKey::ArrowRight);
    let item = fx.item_rect("Open recent");
    let menus = fx.menus();
    let sub = fx.t.rect(menus[1].0);
    assert_eq!((sub.left, sub.top), (item.right, item.top - 8.0));
    // Near the window's right edge it opens to the left.
    let mut fx = fixture_at(250.0);
    fx.open();
    fx.key(NamedKey::End);
    fx.key(NamedKey::ArrowRight);
    let item = fx.item_rect("Open recent");
    let menus = fx.menus();
    let sub = fx.t.rect(menus[1].0);
    assert_eq!(sub.right, item.left);
}

#[test]
fn widgets_menu_07_check_marks() {
    let mut fx = fixture();
    fx.open();
    assert_eq!(
        fx.item("Word wrap").1.toggled(),
        Some(accesskit::Toggled::True)
    );
    let at = fx.item_rect("Word wrap").center();
    fx.click(at);
    assert_eq!(fx.log(), ["wrap"]);
    assert!(!fx.t.enter(|| fx.wrap.get_untracked()));
    fx.open();
    assert_eq!(
        fx.item("Word wrap").1.toggled(),
        Some(accesskit::Toggled::False)
    );
}

/// A focusable box that logs its secondary presses.
struct Target(Log);

impl View for Target {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.focusable(id, FocusOptions::default());
        let log = self.0;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if p.event.kind == PointerKind::Down(PointerButton::Secondary) {
                log.borrow_mut().push("secondary".to_owned());
            }
            Handled::Continue
        });
        id
    }
}

#[test]
fn widgets_menu_08_context_menu() {
    let log: Log = Rc::default();
    let mut fx = {
        let log2 = log.clone();
        let t = WidgetTester::with_size(W, H, move || {
            Stack::new().child(Align::new(tantu_view::layout::Alignment::TOP_LEFT).child(
                Padding::all(20.0).child(SizedBox::new(300.0, 200.0).child(Keyed::new(
                    "region",
                    ContextMenuRegion::new(Keyed::new("target", Target(log2.clone()))).items([
                        MenuItem::new("Rename").on_select(record(&log2, "rename")),
                        MenuItem::new("Delete").on_select(record(&log2, "delete")),
                    ]),
                ))),
            ))
        });
        Fx {
            t,
            controller: MenuController::new(),
            wrap: signal(false),
            log: log.clone(),
        }
    };
    fx.pump();
    // A secondary press opens the menu at the pointer; the child still gets the press.
    fx.pointer(
        PointerKind::Down(PointerButton::Secondary),
        Point::new(50.0, 60.0),
    );
    fx.pointer(
        PointerKind::Up(PointerButton::Secondary),
        Point::new(50.0, 60.0),
    );
    let menus = fx.menus();
    assert_eq!(menus.len(), 1);
    let menu = fx.t.rect(menus[0].0);
    assert_eq!((menu.left, menu.top), (50.0, 60.0));
    assert_eq!(fx.log(), ["secondary"]);
    // Another secondary press moves it.
    fx.pointer(
        PointerKind::Down(PointerButton::Secondary),
        Point::new(80.0, 90.0),
    );
    fx.pointer(
        PointerKind::Up(PointerButton::Secondary),
        Point::new(80.0, 90.0),
    );
    let menus = fx.menus();
    assert_eq!(menus.len(), 1);
    assert_eq!(fx.t.rect(menus[0].0).left, 80.0);
    let delete = fx.item_rect("Delete").center();
    fx.click(delete);
    assert_eq!(fx.log(), ["secondary", "secondary", "delete"]);
    assert!(fx.menus().is_empty());
    // Shift+F10 with the focus inside: at the child's top-left.
    fx.t.focus(&Finder::key("target"));
    fx.t.press_key(
        LogicalKey::Named(NamedKey::F(10)),
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    fx.pump();
    let menus = fx.menus();
    assert_eq!(menus.len(), 1);
    let menu = fx.t.rect(menus[0].0);
    assert_eq!((menu.left, menu.top), (20.0, 20.0));
}

#[test]
fn widgets_menu_09_semantics() {
    let mut fx = fixture();
    fx.open();
    let (_, cut) = fx.item("Cut");
    assert_eq!(cut.role(), accesskit::Role::MenuItem);
    assert!(cut.supports_action(accesskit::Action::Click));
    let (_, wrap) = fx.item("Word wrap");
    assert_eq!(wrap.role(), accesskit::Role::MenuItemCheckBox);
    assert!(fx.item("Print").1.is_disabled());
    assert_eq!(fx.item("Open recent").1.is_expanded(), Some(false));
    fx.key(NamedKey::End);
    fx.key(NamedKey::ArrowRight);
    assert_eq!(fx.item("Open recent").1.is_expanded(), Some(true));
    // Escape closes the submenu; the parent item reports it collapsed again.
    fx.key(NamedKey::Escape);
    assert_eq!(fx.item("Open recent").1.is_expanded(), Some(false));
}
