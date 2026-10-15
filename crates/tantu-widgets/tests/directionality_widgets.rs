//! Tests for `docs/specs/widgets/directionality-widgets.md`, rules WIDGETS-RTL-01..06.

use std::cell::Cell;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Point, Rect};
use tantu_view::layout::{CrossAxisAlignment, TextDirection};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{
    AnyView, ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent,
    PointerKind, View,
};
use tantu_widgets::{
    CalendarDatePicker, Column, ComboBox, Date, Directionality, MenuAnchor, MenuBar,
    MenuController, MenuItem, Padding, ScrollController, ScrollView, SizedBox, Slider, Switch,
    TreeDataSource, TreeNodeId, TreeView,
};

/// `make()` inside a `Directionality` following a signal that starts `Rtl`.
fn window<V: View>(
    width: f32,
    height: f32,
    make: impl Fn() -> V + 'static,
) -> (WidgetTester, Signal<TextDirection>) {
    let out: Rc<Cell<Option<Signal<TextDirection>>>> = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(width, height, move || {
            let direction = signal(TextDirection::Rtl);
            out.set(Some(direction));
            Directionality::new(direction, make())
        })
    };
    t.pump();
    t.pump();
    (t, out.get().expect("built"))
}

fn set(t: &mut WidgetTester, direction: Signal<TextDirection>, to: TextDirection) {
    t.enter(|| direction.set(to));
    t.pump();
    t.pump();
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

fn named(t: &mut WidgetTester, role: accesskit::Role, name: &str) -> ElementId {
    nodes(t, role)
        .into_iter()
        .find(|(_, n)| n.label() == Some(name))
        .map(|(id, _)| id)
        .unwrap_or_else(|| panic!("no {role:?} {name}"))
}

fn key(t: &mut WidgetTester, key: NamedKey) {
    t.press_key(LogicalKey::Named(key), Modifiers::default());
    t.pump();
    t.pump();
}

fn pointer(t: &mut WidgetTester, kind: PointerKind, at: Point) {
    t.pointer(PointerEvent {
        kind,
        position: at,
        modifiers: Modifiers::default(),
    });
    t.pump();
}

fn click(t: &mut WidgetTester, at: Point) {
    pointer(t, PointerKind::Down(PointerButton::Primary), at);
    pointer(t, PointerKind::Up(PointerButton::Primary), at);
    t.pump();
}

/// The filled shapes `element` painted, in its own coordinates.
fn fills(t: &WidgetTester, element: ElementId) -> Vec<Rect> {
    t.frame()
        .entries_for(element)
        .filter_map(|e| match e.command {
            Command::Fill { shape, .. } => Some(shape.rect),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_rtl_01_switch_thumb() {
    let (mut t, direction) = window(200.0, 100.0, || {
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .child(Keyed::new("t", Switch::new(true).on_change(|_| {})))
    });
    let id = t.find(&Finder::key("t"));
    // The on thumb (radius 12) sits at the left under RTL.
    let thumb = |t: &WidgetTester| {
        fills(t, id)
            .into_iter()
            .find(|r| r.width() == 24.0)
            .map(|r| r.center().x)
    };
    assert_eq!(thumb(&t), Some(20.0));
    set(&mut t, direction, TextDirection::Ltr);
    assert_eq!(thumb(&t), Some(40.0));
}

#[test]
fn widgets_rtl_02_slider() {
    let out: Rc<Cell<Option<Signal<f64>>>> = Rc::default();
    let (mut t, direction) = {
        let out = out.clone();
        window(300.0, 200.0, move || {
            let value = signal(20.0f64);
            out.set(Some(value));
            Column::new().child(Keyed::new(
                "s",
                Slider::new(value)
                    .range(0.0, 100.0)
                    .on_change(move |v| value.set(v)),
            ))
        })
    };
    let value = out.get().expect("built");
    let id = t.find(&Finder::key("s"));
    let thumb = |t: &WidgetTester| {
        fills(t, id)
            .into_iter()
            .find(|r| r.width() == 20.0 && r.height() == 20.0)
            .map(|r| r.center().x)
            .expect("a thumb")
    };
    let rtl = thumb(&t);
    set(&mut t, direction, TextDirection::Ltr);
    let ltr = thumb(&t);
    assert_eq!(rtl, 300.0 - ltr, "mirrored");
    set(&mut t, direction, TextDirection::Rtl);
    // Left increases, Right decreases.
    t.focus(&Finder::key("s"));
    key(&mut t, NamedKey::ArrowLeft);
    assert_eq!(t.enter(|| value.get_untracked()), 25.0);
    key(&mut t, NamedKey::ArrowRight);
    key(&mut t, NamedKey::ArrowRight);
    assert_eq!(t.enter(|| value.get_untracked()), 15.0);
    // A press maps mirrored: the thumb follows to where it was pressed.
    let slider = t.rect(id);
    click(&mut t, Point::new(234.0, slider.center().y));
    assert!((thumb(&t) - 234.0).abs() < 0.5, "{}", thumb(&t));
    assert!(t.enter(|| value.get_untracked()) < 50.0);
}

#[test]
fn widgets_rtl_03_scrollbar_side() {
    let out: Rc<Cell<Option<ScrollController>>> = Rc::default();
    let (mut t, direction) = window(300.0, 200.0, {
        let out = out.clone();
        move || {
            let controller = ScrollController::new();
            out.set(Some(controller));
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(SizedBox::new(200.0, 100.0).child(Keyed::new(
                    "sv",
                    ScrollView::new(SizedBox::new(180.0, 1000.0)).controller(controller),
                )))
        }
    });
    let controller = out.get().expect("built");
    let id = t.find(&Finder::key("sv"));
    let bar = |t: &WidgetTester| {
        fills(t, id)
            .into_iter()
            .find(|r| r.width() <= 10.0 && r.height() >= 10.0)
            .expect("a thumb")
    };
    let thumb = bar(&t);
    assert!(thumb.left < 20.0, "at the left: {thumb:?}");
    // It drags from there.
    let view = t.rect(id);
    let at = Point::new(view.left + thumb.center().x, view.top + thumb.center().y);
    pointer(&mut t, PointerKind::Down(PointerButton::Primary), at);
    pointer(&mut t, PointerKind::Move, Point::new(at.x, at.y + 30.0));
    pointer(
        &mut t,
        PointerKind::Up(PointerButton::Primary),
        Point::new(at.x, at.y + 30.0),
    );
    assert!(t.enter(|| controller.offset()) > 0.0);
    set(&mut t, direction, TextDirection::Ltr);
    assert!(bar(&t).left > 180.0);
}

struct Model {
    children: HashMap<u64, Vec<u64>>,
}

impl TreeDataSource for Model {
    fn root_count(&self) -> usize {
        1
    }
    fn root(&self, _: usize) -> Option<TreeNodeId> {
        Some(TreeNodeId(0))
    }
    fn child_count(&self, id: TreeNodeId) -> usize {
        self.children.get(&id.0).map_or(0, Vec::len)
    }
    fn child(&self, id: TreeNodeId, i: usize) -> Option<TreeNodeId> {
        self.children.get(&id.0)?.get(i).map(|c| TreeNodeId(*c))
    }
    fn label(&self, id: TreeNodeId) -> String {
        format!("Node {}", id.0)
    }
}

#[test]
fn widgets_rtl_04_tree_view() {
    let model = Rc::new(Model {
        children: HashMap::from([(0, vec![1]), (1, vec![2])]),
    });
    let out: Rc<Cell<Option<Signal<Option<TreeNodeId>>>>> = Rc::default();
    let (mut t, _) = {
        let out = out.clone();
        window(240.0, 200.0, move || {
            let selected = signal(None);
            out.set(Some(selected));
            Keyed::new(
                "tree",
                TreeView::new(model.clone())
                    .expanded(signal(BTreeSet::from([TreeNodeId(0)])))
                    .selection(selected),
            )
        })
    };
    let tree = t.rect(t.find(&Finder::key("tree")));
    let label = |t: &WidgetTester, text: &str| t.rect(t.find(&Finder::text(text)));
    let (root, child) = (label(&t, "Node 0"), label(&t, "Node 1"));
    // Indented from the right: the expander's 20 px, then 20 px per level.
    assert_eq!(tree.right - root.right, 20.0);
    assert_eq!(root.right - child.right, 20.0);
    // Left expands the collapsed Node 1; Right collapses it.
    click(&mut t, child.center());
    t.focus(&Finder::key("tree"));
    key(&mut t, NamedKey::ArrowLeft);
    assert!(!t.find_all(&Finder::text("Node 2")).is_empty(), "expanded");
    key(&mut t, NamedKey::ArrowRight);
    assert!(t.find_all(&Finder::text("Node 2")).is_empty(), "collapsed");
}

#[test]
fn widgets_rtl_05_menus() {
    let out: Rc<Cell<Option<MenuController>>> = Rc::default();
    let items = || {
        vec![
            MenuItem::new("Cut"),
            MenuItem::submenu(
                "Open recent",
                [MenuItem::new("a.txt"), MenuItem::new("b.txt")],
            ),
        ]
    };
    let (mut t, _) = window(400.0, 300.0, {
        let out = out.clone();
        move || {
            let controller = MenuController::new();
            out.set(Some(controller));
            Padding::all(20.0).child(
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .child(MenuAnchor::new(controller, SizedBox::new(60.0, 30.0)).items(items())),
            )
        }
    });
    let controller = out.get().expect("built");
    t.enter(|| controller.open());
    t.pump();
    t.pump();
    key(&mut t, NamedKey::End);
    key(&mut t, NamedKey::ArrowLeft);
    let menus = nodes(&mut t, accesskit::Role::Menu);
    assert_eq!(menus.len(), 2, "Left opens the submenu");
    let item = named(&mut t, accesskit::Role::MenuItem, "Open recent");
    let item = t.rect(item);
    let sub = t.rect(menus[1].0);
    assert_eq!(sub.right, item.left, "to the left of its item");
    key(&mut t, NamedKey::ArrowRight);
    assert_eq!(
        nodes(&mut t, accesskit::Role::Menu).len(),
        1,
        "Right closes it"
    );

    // A menu bar's arrows run right to left.
    let (mut t, _) = window(400.0, 300.0, || {
        Keyed::new(
            "bar",
            MenuBar::new()
                .menu("File", [MenuItem::new("New")])
                .menu("Edit", [MenuItem::new("Undo")])
                .menu("View", [MenuItem::new("Zoom")]),
        )
    });
    t.focus(&Finder::key("bar"));
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
    assert_eq!(current(&mut t).as_deref(), Some("Edit"));
    key(&mut t, NamedKey::ArrowRight);
    assert_eq!(current(&mut t).as_deref(), Some("File"));
}

/// The glyph ids of every glyph run in the last frame, in paint order, with the element
/// that painted each.
fn runs(t: &WidgetTester) -> Vec<(Option<ElementId>, Vec<u32>)> {
    let frame = t.frame();
    frame
        .scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some((
                e.element,
                frame.scene.glyphs(run).iter().map(|g| g.id).collect(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_rtl_06_combo_arrow_and_calendar_chevrons() {
    let (mut t, direction) = window(400.0, 200.0, || {
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .child(Keyed::new(
                "combo",
                ComboBox::new(signal(None), ["One", "Two"]),
            ))
    });
    let field = t.rect(t.find(&Finder::key("combo")));
    let arrow = |t: &WidgetTester| {
        let (element, _) = runs(t)
            .into_iter()
            .find(|(_, glyphs)| glyphs.len() == 1)
            .expect("the arrow");
        t.rect(element.expect("an element"))
    };
    assert!(arrow(&t).right < field.left + 40.0, "{:?}", arrow(&t));
    set(&mut t, direction, TextDirection::Ltr);
    assert!(arrow(&t).left > field.right - 40.0);

    // The calendar's chevrons swap.
    let date = Date::new(2026, 10, 16).expect("exists");
    let (mut t, direction) = window(400.0, 400.0, move || {
        AnyView::new(CalendarDatePicker::new(signal(Some(date))))
    });
    let chevrons = |t: &WidgetTester| -> (Vec<u32>, Vec<u32>) {
        // The month label's run comes first, then the previous and next buttons' icons.
        let runs = runs(t);
        (runs[1].1.clone(), runs[2].1.clone())
    };
    let (rtl_previous, rtl_next) = chevrons(&t);
    set(&mut t, direction, TextDirection::Ltr);
    let (ltr_previous, ltr_next) = chevrons(&t);
    assert_ne!(ltr_previous, ltr_next);
    assert_eq!((rtl_previous, rtl_next), (ltr_next, ltr_previous));
}
