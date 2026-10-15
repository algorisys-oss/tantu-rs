//! Tests for `docs/specs/widgets/menu-bar.md`, rules WIDGETS-MENUBAR-01..04,
//! WIDGETS-TOOLBAR-01..03 and WIDGETS-STATUSBAR-01..02.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Point, Rect};
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::scene::Command;
use tantu_view::{ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerEvent, PointerKind};
use tantu_widgets::{
    Column, IconButton, Icons, MenuBar, MenuItem, StatusBar, Text, TextEditingController,
    TextField, Toolbar,
};

const W: f32 = 400.0;
const H: f32 = 300.0;

type Log = Rc<RefCell<Vec<String>>>;

fn record(log: &Log, name: &str) -> impl Fn() + 'static {
    let (log, name) = (log.clone(), name.to_owned());
    move || log.borrow_mut().push(name.clone())
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
}

fn nodes(t: &mut WidgetTester, role: accesskit::Role) -> Vec<(ElementId, accesskit::Node)> {
    t.a11y_tree()
        .nodes
        .into_iter()
        .filter(|(_, n)| n.role() == role)
        .map(|(id, n)| (element(id), n))
        .collect()
}

fn named(t: &mut WidgetTester, role: accesskit::Role, name: &str) -> (ElementId, accesskit::Node) {
    nodes(t, role)
        .into_iter()
        .find(|(_, n)| n.label() == Some(name))
        .unwrap_or_else(|| panic!("no {role:?} {name}"))
}

fn focused(t: &mut WidgetTester) -> Option<ElementId> {
    ElementId::from_raw(t.a11y_tree().focus.0)
}

fn key(t: &mut WidgetTester, key: NamedKey) {
    t.press_key(LogicalKey::Named(key), Modifiers::default());
    t.pump();
    t.pump();
}

/// The own fills of `element`, in window coordinates.
fn fills(t: &WidgetTester, element: ElementId) -> Vec<(Rect, tantu_view::core::Color)> {
    let origin = t.rect(element);
    t.frame()
        .scene
        .entries()
        .iter()
        .filter(|e| e.element == Some(element))
        .filter_map(|e| match &e.command {
            Command::Fill { shape, color } => Some((
                Rect::from_ltrb(
                    shape.rect.left + origin.left,
                    shape.rect.top + origin.top,
                    shape.rect.right + origin.left,
                    shape.rect.bottom + origin.top,
                ),
                *color,
            )),
            _ => None,
        })
        .collect()
}

/// File (New, Open, divider, Quit), Edit (Undo, Redo), View (Word wrap); a field below.
fn bar_fixture() -> (WidgetTester, Log) {
    let log: Log = Rc::default();
    let t = {
        let log = log.clone();
        WidgetTester::with_size(W, H, move || {
            let field = TextEditingController::new("");
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                .child(Keyed::new(
                    "bar",
                    MenuBar::new()
                        .menu(
                            "File",
                            [
                                MenuItem::new("New").on_select(record(&log, "new")),
                                MenuItem::new("Open").on_select(record(&log, "open")),
                                MenuItem::divider(),
                                MenuItem::new("Quit").on_select(record(&log, "quit")),
                            ],
                        )
                        .menu(
                            "Edit",
                            [
                                MenuItem::new("Undo").on_select(record(&log, "undo")),
                                MenuItem::new("Redo").on_select(record(&log, "redo")),
                            ],
                        )
                        .menu(
                            "View",
                            [MenuItem::new("Word wrap").on_select(record(&log, "wrap"))],
                        ),
                ))
                .child(Keyed::new("field", TextField::new(field)))
        })
    };
    (t, log)
}

fn top(t: &mut WidgetTester, name: &str) -> Rect {
    let id = named(t, accesskit::Role::MenuItem, name).0;
    t.rect(id)
}

fn open_menus(t: &mut WidgetTester) -> usize {
    nodes(t, accesskit::Role::Menu).len()
}

/// The highlighted item of the open menu, if any.
fn highlighted(t: &mut WidgetTester) -> Option<String> {
    let tree = t.a11y_tree();
    let menu = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == accesskit::Role::Menu)?
        .1
        .clone();
    let active = menu.active_descendant()?;
    tree.nodes
        .iter()
        .find(|(id, _)| *id == active)
        .and_then(|(_, n)| n.label().map(str::to_owned))
}

fn click(t: &mut WidgetTester, at: Point) {
    for kind in [
        PointerKind::Down(tantu_view::PointerButton::Primary),
        PointerKind::Up(tantu_view::PointerButton::Primary),
    ] {
        t.pointer(PointerEvent {
            kind,
            position: at,
            modifiers: Modifiers::default(),
        });
    }
    t.pump();
    t.pump();
}

#[test]
fn widgets_menubar_01_look() {
    let (mut t, _) = bar_fixture();
    let bar_id = t.find(&Finder::key("bar"));
    let bar = t.rect(bar_id);
    assert_eq!((bar.width(), bar.height()), (W, 40.0));
    let colors = t.enter(|| t.theme().get().colors);
    let own = fills(&t, bar_id);
    assert!(own.contains(&(bar, colors.surface)), "{own:?}");
    assert!(own.contains(&(
        Rect::from_ltrb(0.0, bar.bottom - 1.0, W, bar.bottom),
        colors.outline_variant
    )));
    let file = top(&mut t, "File");
    let edit = top(&mut t, "Edit");
    assert_eq!((file.left, file.top, file.height()), (0.0, 0.0, 40.0));
    assert_eq!(edit.left, file.right);
}

#[test]
fn widgets_menubar_02_pointer() {
    let (mut t, log) = bar_fixture();
    t.focus(&Finder::key("field"));
    let field = focused(&mut t);
    let file = top(&mut t, "File");
    click(&mut t, file.center());
    assert_eq!(open_menus(&mut t), 1);
    let menu = nodes(&mut t, accesskit::Role::Menu)[0].0;
    let menu = t.rect(menu);
    assert_eq!((menu.left, menu.top), (file.left, file.bottom));
    // Pressing the open item closes it.
    click(&mut t, file.center());
    assert_eq!(open_menus(&mut t), 0);
    // While a menu is open, hovering another top item switches to it.
    click(&mut t, file.center());
    let edit = top(&mut t, "Edit");
    t.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: edit.center(),
        modifiers: Modifiers::default(),
    });
    t.pump();
    t.pump();
    assert_eq!(open_menus(&mut t), 1);
    assert!(
        named(&mut t, accesskit::Role::MenuItem, "Edit")
            .1
            .is_expanded()
            == Some(true)
    );
    // Choosing closes; the focus goes back to the field.
    let undo = named(&mut t, accesskit::Role::MenuItem, "Undo").0;
    let at = t.rect(undo).center();
    click(&mut t, at);
    assert_eq!(*log.borrow(), ["undo"]);
    assert_eq!(open_menus(&mut t), 0);
    assert_eq!(focused(&mut t), field);
}

#[test]
fn widgets_menubar_03_keyboard() {
    let (mut t, log) = bar_fixture();
    t.focus(&Finder::key("bar"));
    let bar = focused(&mut t);
    let current = |t: &mut WidgetTester| {
        let tree = t.a11y_tree();
        let bar = tree
            .nodes
            .iter()
            .find(|(_, n)| n.role() == accesskit::Role::MenuBar)
            .expect("bar")
            .1
            .clone();
        let active = bar.active_descendant().expect("a current item");
        tree.nodes
            .iter()
            .find(|(id, _)| *id == active)
            .and_then(|(_, n)| n.label().map(str::to_owned))
    };
    assert_eq!(current(&mut t).as_deref(), Some("File"));
    key(&mut t, NamedKey::ArrowLeft);
    assert_eq!(current(&mut t).as_deref(), Some("View"), "wrapping");
    key(&mut t, NamedKey::ArrowRight);
    key(&mut t, NamedKey::ArrowRight);
    assert_eq!(current(&mut t).as_deref(), Some("Edit"));
    // Down opens the current menu with its first item highlighted.
    key(&mut t, NamedKey::ArrowDown);
    assert_eq!(open_menus(&mut t), 1);
    assert_eq!(highlighted(&mut t).as_deref(), Some("Undo"));
    // Right moves to the next menu, Left back.
    key(&mut t, NamedKey::ArrowRight);
    assert_eq!(open_menus(&mut t), 1);
    assert_eq!(highlighted(&mut t).as_deref(), Some("Word wrap"));
    key(&mut t, NamedKey::ArrowLeft);
    assert_eq!(highlighted(&mut t).as_deref(), Some("Undo"));
    // Escape closes; the bar has the focus again, on Edit.
    key(&mut t, NamedKey::Escape);
    assert_eq!(open_menus(&mut t), 0);
    assert_eq!(focused(&mut t), bar);
    assert_eq!(current(&mut t).as_deref(), Some("Edit"));
    // Enter opens; Enter chooses.
    key(&mut t, NamedKey::Enter);
    key(&mut t, NamedKey::ArrowDown);
    key(&mut t, NamedKey::Enter);
    assert_eq!(*log.borrow(), ["redo"]);
}

#[test]
fn widgets_menubar_04_semantics() {
    let (mut t, _) = bar_fixture();
    assert_eq!(nodes(&mut t, accesskit::Role::MenuBar).len(), 1);
    let mut names: Vec<String> = nodes(&mut t, accesskit::Role::MenuItem)
        .iter()
        .map(|(_, n)| n.label().unwrap_or_default().to_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["Edit", "File", "View"]);
    assert_eq!(
        named(&mut t, accesskit::Role::MenuItem, "File")
            .1
            .is_expanded(),
        Some(false)
    );
    let file = top(&mut t, "File");
    click(&mut t, file.center());
    assert_eq!(
        named(&mut t, accesskit::Role::MenuItem, "File")
            .1
            .is_expanded(),
        Some(true)
    );
}

/// Bold, Italic, Underline | Undo, keyed b1..b4.
fn toolbar_fixture() -> WidgetTester {
    let mut t = WidgetTester::with_size(W, H, || {
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .child(Keyed::new(
                "toolbar",
                Toolbar::new()
                    .child(Keyed::new(
                        "b1",
                        IconButton::new(Icons::FORMAT_BOLD).on_press(|| {}),
                    ))
                    .child(Keyed::new(
                        "b2",
                        IconButton::new(Icons::FORMAT_ITALIC).on_press(|| {}),
                    ))
                    .child(Keyed::new(
                        "b3",
                        IconButton::new(Icons::FORMAT_UNDERLINED).on_press(|| {}),
                    ))
                    .separator()
                    .child(Keyed::new(
                        "b4",
                        IconButton::new(Icons::UNDO).on_press(|| {}),
                    )),
            ))
    });
    t.pump();
    t
}

fn rect(t: &WidgetTester, key: &str) -> Rect {
    t.rect(t.find(&Finder::key(key)))
}

#[test]
fn widgets_toolbar_01_layout() {
    let t = toolbar_fixture();
    let bar = rect(&t, "toolbar");
    assert_eq!(bar.width(), W);
    assert!(bar.height() >= 48.0);
    let (b1, b2, b3, b4) = (
        rect(&t, "b1"),
        rect(&t, "b2"),
        rect(&t, "b3"),
        rect(&t, "b4"),
    );
    assert_eq!(b1.left, 8.0);
    assert_eq!(b2.left, b1.right + 4.0);
    assert_eq!(b3.left, b2.right + 4.0);
    // 4 px, the 9 px separator (4 + 1 + 4), 4 px.
    assert_eq!(b4.left, b3.right + 17.0);
    assert!((b1.center().y - bar.center().y).abs() < 0.5);
}

#[test]
fn widgets_toolbar_02_arrow_keys() {
    let mut t = toolbar_fixture();
    let id = |t: &WidgetTester, key: &str| t.find(&Finder::key(key));
    t.focus(&Finder::key("b1"));
    let check = |t: &mut WidgetTester, key: &str| {
        let expected = id(t, key);
        assert_eq!(focused(t), Some(expected), "{key}");
    };
    key(&mut t, NamedKey::ArrowRight);
    check(&mut t, "b2");
    key(&mut t, NamedKey::ArrowRight);
    key(&mut t, NamedKey::ArrowRight);
    check(&mut t, "b4");
    key(&mut t, NamedKey::ArrowRight);
    check(&mut t, "b1");
    key(&mut t, NamedKey::ArrowLeft);
    check(&mut t, "b4");
    key(&mut t, NamedKey::Home);
    check(&mut t, "b1");
    key(&mut t, NamedKey::End);
    check(&mut t, "b4");
}

#[test]
fn widgets_toolbar_03_semantics() {
    let mut t = toolbar_fixture();
    assert_eq!(nodes(&mut t, accesskit::Role::Toolbar).len(), 1);
}

fn status_fixture() -> WidgetTester {
    let mut t = WidgetTester::with_size(W, H, || {
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .child(Keyed::new(
                "status",
                StatusBar::new()
                    .leading(Keyed::new("ready", Text::new("Ready")))
                    .leading(Keyed::new("saved", Text::new("Saved")))
                    .trailing(Keyed::new("pos", Text::new("Ln 1, Col 1"))),
            ))
    });
    t.pump();
    t
}

#[test]
fn widgets_statusbar_01_layout() {
    let t = status_fixture();
    let bar = rect(&t, "status");
    assert_eq!((bar.width(), bar.height()), (W, 28.0));
    let colors = t.enter(|| t.theme().get().colors);
    let own = fills(&t, t.find(&Finder::key("status")));
    assert!(own.contains(&(bar, colors.surface_container_highest)));
    assert!(own.contains(&(
        Rect::from_ltrb(0.0, bar.top, W, bar.top + 1.0),
        colors.outline_variant
    )));
    let (ready, saved, pos) = (rect(&t, "ready"), rect(&t, "saved"), rect(&t, "pos"));
    assert_eq!(ready.left, 12.0);
    assert_eq!(saved.left, ready.right + 16.0);
    assert_eq!(pos.right, W - 12.0);
    assert!((ready.center().y - bar.center().y).abs() < 0.5);
}

#[test]
fn widgets_statusbar_02_semantics() {
    let mut t = status_fixture();
    assert_eq!(nodes(&mut t, accesskit::Role::Status).len(), 1);
}
