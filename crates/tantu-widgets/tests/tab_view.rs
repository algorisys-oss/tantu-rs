//! Tests for `docs/specs/widgets/tab-view.md`, rules WIDGETS-TAB-01..10.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_theme::Theme;
use tantu_view::core::{Point, Rect, Vec2};
use tantu_view::reactive::{Signal, on_cleanup, signal};
use tantu_view::scene::Command;
use tantu_view::{
    ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
    SemanticsAction,
};
use tantu_widgets::{
    Button, Column, Expanded, Icons, SizedBox, Tab, TabBar, TabBarView, TabView, Text,
    TextEditingController, TextField,
};

const W: f32 = 480.0;
const H: f32 = 320.0;

/// A tab as the accessibility tree reports it.
#[derive(Debug, PartialEq)]
struct TabNode {
    id: ElementId,
    label: String,
    selected: bool,
    disabled: bool,
    position: Option<usize>,
    size: Option<usize>,
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("node ids are element ids")
}

fn tab_nodes(t: &mut WidgetTester) -> Vec<TabNode> {
    t.a11y_tree()
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == accesskit::Role::Tab)
        .map(|(id, n)| TabNode {
            id: element(*id),
            label: n.label().unwrap_or_default().to_owned(),
            selected: n.is_selected() == Some(true),
            disabled: n.is_disabled(),
            position: n.position_in_set(),
            size: n.size_of_set(),
        })
        .collect()
}

fn tab(t: &mut WidgetTester, label: &str) -> ElementId {
    tab_nodes(t)
        .into_iter()
        .find(|n| n.label == label)
        .unwrap_or_else(|| panic!("no tab {label}"))
        .id
}

fn tab_rect(t: &mut WidgetTester, label: &str) -> Rect {
    let id = tab(t, label);
    t.rect(id)
}

fn shown(t: &mut WidgetTester) -> String {
    tab_nodes(t)
        .into_iter()
        .find(|n| n.selected)
        .map(|n| n.label)
        .unwrap_or_default()
}

fn role_ids(t: &mut WidgetTester, role: accesskit::Role) -> Vec<ElementId> {
    t.a11y_tree()
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == role)
        .map(|(id, _)| element(*id))
        .collect()
}

fn focused(t: &mut WidgetTester) -> Option<ElementId> {
    ElementId::from_raw(t.a11y_tree().focus.0)
}

/// Focuses the bar by pressing the selected tab (which changes nothing else).
fn focus_bar(t: &mut WidgetTester) {
    let label = shown(t);
    let center = tab_rect(t, &label).center();
    press(t, center, PointerButton::Primary);
}

fn key(t: &mut WidgetTester, key: NamedKey, modifiers: Modifiers) {
    t.press_key(LogicalKey::Named(key), modifiers);
    t.pump();
}

fn control() -> Modifiers {
    Modifiers {
        control: true,
        ..Default::default()
    }
}

fn press(t: &mut WidgetTester, at: Point, button: PointerButton) {
    for kind in [PointerKind::Down(button), PointerKind::Up(button)] {
        t.pointer(PointerEvent {
            kind,
            position: at,
            modifiers: Default::default(),
        });
    }
    t.pump();
}

/// The `TabView` fixture: General (a text field), Advanced (counts its builds and live
/// copies), Archived (disabled) and Data, inside a column with a button after it.
struct Fx {
    t: WidgetTester,
    selected: Signal<usize>,
    controller: TextEditingController,
    builds: Rc<Cell<usize>>,
    live: Rc<Cell<usize>>,
}

fn fixture(keep_alive: bool) -> Fx {
    let out = Rc::new(RefCell::new(None));
    let store = out.clone();
    let builds = Rc::new(Cell::new(0));
    let live = Rc::new(Cell::new(0));
    let (b, l) = (builds.clone(), live.clone());
    let mut t = WidgetTester::with_size(W, H, move || {
        let selected = signal(0usize);
        let controller = TextEditingController::new("");
        *store.borrow_mut() = Some((selected, controller.clone()));
        let (b, l) = (b.clone(), l.clone());
        Column::new()
            .child(Expanded::new(Keyed::new(
                "tabs",
                TabView::new()
                    .selected(selected)
                    .keep_alive(keep_alive)
                    .tab(Tab::new("General"), move || {
                        Keyed::new("field", TextField::new(controller.clone()))
                    })
                    .tab(Tab::new("Advanced"), move || {
                        b.set(b.get() + 1);
                        l.set(l.get() + 1);
                        let l = l.clone();
                        on_cleanup(move || l.set(l.get() - 1));
                        Keyed::new("advanced", Text::new("Advanced page"))
                    })
                    .tab(Tab::new("Archived").enabled(false), || {
                        Text::new("Archived page")
                    })
                    .tab(Tab::new("Data"), || Text::new("Data page")),
            )))
            .child(Keyed::new("after", Button::new("After")))
    });
    t.pump();
    t.pump();
    let (selected, controller) = out.borrow_mut().take().expect("built");
    Fx {
        t,
        selected,
        controller,
        builds,
        live,
    }
}

impl Fx {
    fn select(&mut self, index: usize) {
        self.t.enter(|| self.selected.set(index));
        self.t.pump();
        self.t.pump();
    }
    fn selected(&self) -> usize {
        self.t.enter(|| self.selected.get())
    }
    fn bar(&mut self) -> ElementId {
        role_ids(&mut self.t, accesskit::Role::TabList)[0]
    }
}

/// The indicator: the bar's 3 px tall `primary` fill, in window coordinates.
fn indicator(t: &mut WidgetTester, bar: ElementId) -> Option<Rect> {
    let primary = t.enter(|| t.theme().get().colors.primary);
    let origin = t.rect(bar);
    t.frame()
        .scene
        .entries()
        .iter()
        .filter(|e| e.element == Some(bar))
        .find_map(|e| match &e.command {
            Command::Fill { shape, color }
                if *color == primary && (shape.rect.height() - 3.0).abs() < 0.01 =>
            {
                Some(Rect::from_ltrb(
                    shape.rect.left + origin.left,
                    shape.rect.top + origin.top,
                    shape.rect.right + origin.left,
                    shape.rect.bottom + origin.top,
                ))
            }
            _ => None,
        })
}

#[test]
fn widgets_tab_01_clamped_index_shared_signal_and_empty_bar() {
    let mut f = fixture(true);
    assert_eq!(shown(&mut f.t), "General");
    f.select(9);
    assert_eq!(shown(&mut f.t), "Data", "the shown index is clamped");
    assert_eq!(f.selected(), 9, "the signal is not rewritten");
    f.select(2);
    assert_eq!(
        shown(&mut f.t),
        "Archived",
        "the app may select a disabled tab"
    );
    f.t.find(&Finder::text("Archived page"));

    // A bar and a view sharing one signal stay in step.
    let out = Rc::new(RefCell::new(None));
    let store = out.clone();
    let mut t = WidgetTester::with_size(W, H, move || {
        let selected = signal(1usize);
        *store.borrow_mut() = Some(selected);
        Column::new()
            .child(TabBar::new(selected).tab(Tab::new("A")).tab(Tab::new("B")))
            .child(Expanded::new(
                TabBarView::new(selected)
                    .page(|| Text::new("Page A"))
                    .page(|| Text::new("Page B")),
            ))
    });
    t.pump();
    t.pump();
    assert_eq!(shown(&mut t), "B");
    t.find(&Finder::text("Page B"));
    let selected = out.borrow_mut().take().expect("built");
    t.enter(|| selected.set(0));
    t.pump();
    t.pump();
    assert_eq!(shown(&mut t), "A");
    t.find(&Finder::text("Page A"));

    // An empty bar shows nothing and is no focus stop.
    let mut t = WidgetTester::with_size(W, H, || {
        Column::new()
            .child(Keyed::new("empty", TabBar::new(signal(0))))
            .child(Keyed::new("next", Button::new("Next")))
    });
    t.pump();
    assert!(role_ids(&mut t, accesskit::Role::Tab).is_empty());
    let empty = t.find(&Finder::key("empty"));
    let next = t.find(&Finder::key("next"));
    key(&mut t, NamedKey::Tab, Modifiers::default());
    assert_eq!(focused(&mut t), Some(next));
    key(&mut t, NamedKey::Tab, Modifiers::default());
    assert_eq!(focused(&mut t), Some(next), "the empty bar is no stop");
    assert_ne!(focused(&mut t), Some(empty));
}

#[test]
fn widgets_tab_02_fixed_and_scrollable_sizes() {
    let mut f = fixture(true);
    for label in ["General", "Advanced", "Archived", "Data"] {
        let r = tab_rect(&mut f.t, label);
        assert_eq!(r.width(), W / 4.0, "{label} gets an equal share");
        assert_eq!(r.height(), 48.0);
    }
    let bar = f.bar();
    assert_eq!(f.t.rect(bar).height(), 48.0);

    // An icon makes the bar 64 px tall.
    let mut t = WidgetTester::with_size(W, H, || {
        TabBar::new(signal(0))
            .tab(Tab::new("Home").icon(Icons::HOME))
            .tab(Tab::new("Info"))
    });
    t.pump();
    assert_eq!(tab_rect(&mut t, "Info").height(), 64.0);

    // Scrollable: natural widths from the left, wheel scrolling, selection reveals.
    let out = Rc::new(RefCell::new(None));
    let store = out.clone();
    let mut t = WidgetTester::with_size(300.0, H, move || {
        let selected = signal(0usize);
        *store.borrow_mut() = Some(selected);
        Keyed::new(
            "bar",
            TabBar::new(selected)
                .scrollable(true)
                .tabs((0..12).map(|i| Tab::new(format!("Tab {i}")))),
        )
    });
    t.pump();
    t.pump();
    let first = tab_rect(&mut t, "Tab 0");
    assert_eq!(first.left, 0.0);
    let label = t.rect(t.find(&Finder::text("Tab 0")));
    assert!((first.width() - (label.width() + 32.0)).abs() < 0.5);
    t.scroll(&Finder::key("bar"), Vec2::new(0.0, 120.0));
    t.advance(Duration::from_millis(400));
    assert!(tab_rect(&mut t, "Tab 0").left < 0.0, "the wheel scrolls");
    let selected = out.borrow_mut().take().expect("built");
    t.enter(|| selected.set(11));
    t.pump();
    t.pump();
    let last = tab_rect(&mut t, "Tab 11");
    assert!(
        last.left >= -0.5 && last.right <= 300.5,
        "the selected tab is revealed: {last:?}"
    );
}

#[test]
fn widgets_tab_03_material_look_and_theme_changes() {
    let mut f = fixture(true);
    let bar = f.bar();
    let general = tab_rect(&mut f.t, "General");
    let ind = indicator(&mut f.t, bar).expect("an indicator");
    assert_eq!(ind.bottom, general.bottom);
    let label = f.t.rect(f.t.find(&Finder::text("General")));
    assert!((ind.width() - label.width().max(24.0)).abs() < 0.5);
    assert!((ind.center().x - general.center().x).abs() < 0.5);
    f.t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/tabs-light.png"
    ));
    f.select(1);
    f.t.advance(Duration::from_millis(400));
    f.t.enter(|| f.t.theme().set(Theme::dark()));
    f.t.pump();
    assert_eq!(shown(&mut f.t), "Advanced");
    assert_eq!(f.builds.get(), 1, "a theme change keeps the pages");
    f.t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/tabs-dark.png"
    ));
}

#[test]
fn widgets_tab_04_indicator_slides() {
    let mut f = fixture(true);
    let bar = f.bar();
    let general = tab_rect(&mut f.t, "General");
    let data = tab_rect(&mut f.t, "Data");
    assert!(
        (indicator(&mut f.t, bar).unwrap().center().x - general.center().x).abs() < 0.5,
        "the first build shows it in place"
    );
    f.select(3);
    f.t.advance(Duration::from_millis(150));
    let mid = indicator(&mut f.t, bar).unwrap().center().x;
    assert!(
        mid > general.center().x + 1.0 && mid < data.center().x - 1.0,
        "sliding: {mid}"
    );
    // A change while sliding continues from the current position.
    f.select(0);
    f.t.advance(Duration::from_millis(1));
    let restart = indicator(&mut f.t, bar).unwrap().center().x;
    assert!(
        (restart - mid).abs() < 20.0,
        "continues near {mid}: {restart}"
    );
    f.t.advance(Duration::from_millis(400));
    let end = indicator(&mut f.t, bar).unwrap().center().x;
    assert!((end - general.center().x).abs() < 0.5);
}

#[test]
fn widgets_tab_05_pointer_selection() {
    let mut f = fixture(true);
    let advanced = tab(&mut f.t, "Advanced");
    let center = f.t.rect(advanced).center();
    press(&mut f.t, center, PointerButton::Primary);
    assert_eq!(f.selected(), 1);
    let bar = f.bar();
    assert_eq!(focused(&mut f.t), Some(bar), "a press focuses the bar");
    let archived = tab_rect(&mut f.t, "Archived").center();
    press(&mut f.t, archived, PointerButton::Primary);
    assert_eq!(f.selected(), 1, "disabled tabs do nothing");
    let data = tab_rect(&mut f.t, "Data").center();
    press(&mut f.t, data, PointerButton::Secondary);
    assert_eq!(f.selected(), 1, "secondary presses do nothing");
    // A release outside the tab selects nothing.
    f.t.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: data,
        modifiers: Default::default(),
    });
    f.t.pointer(PointerEvent {
        kind: PointerKind::Up(PointerButton::Primary),
        position: Point::new(data.x, 200.0),
        modifiers: Default::default(),
    });
    f.t.pump();
    assert_eq!(f.selected(), 1);
}

#[test]
fn widgets_tab_06_keyboard_on_the_bar() {
    let mut f = fixture(true);
    let bar = f.bar();
    focus_bar(&mut f.t);
    let general = tab_rect(&mut f.t, "General");
    let primary = f.t.enter(|| f.t.theme().get().colors.primary);
    let origin = f.t.rect(bar);
    let ring = f.t.frame().scene.entries().iter().any(|e| {
        e.element == Some(bar)
            && matches!(&e.command, Command::Stroke { shape, width, color }
                if *color == primary && *width == 2.0
                    && (shape.rect.left + origin.left - general.left).abs() < 0.5
                    && (shape.rect.right + origin.left - general.right).abs() < 0.5)
    });
    assert!(ring, "the focus ring surrounds the selected tab");
    key(&mut f.t, NamedKey::ArrowRight, Modifiers::default());
    assert_eq!(f.selected(), 1);
    key(&mut f.t, NamedKey::ArrowRight, Modifiers::default());
    assert_eq!(f.selected(), 3, "Archived is skipped");
    key(&mut f.t, NamedKey::ArrowRight, Modifiers::default());
    assert_eq!(f.selected(), 0, "wraps");
    key(&mut f.t, NamedKey::ArrowLeft, Modifiers::default());
    assert_eq!(f.selected(), 3, "wraps back");
    key(&mut f.t, NamedKey::Home, Modifiers::default());
    assert_eq!(f.selected(), 0);
    key(&mut f.t, NamedKey::End, Modifiers::default());
    assert_eq!(f.selected(), 3);
    key(&mut f.t, NamedKey::Enter, Modifiers::default());
    assert_eq!(f.selected(), 3);
    f.t.press_key(
        LogicalKey::Named(NamedKey::ArrowLeft),
        Modifiers {
            alt: true,
            ..Default::default()
        },
    );
    f.t.pump();
    assert_eq!(f.selected(), 3, "modified keys pass through");

    // Every tab disabled: keys do nothing.
    let out = Rc::new(RefCell::new(None));
    let store = out.clone();
    let mut t = WidgetTester::with_size(W, H, move || {
        let selected = signal(0usize);
        *store.borrow_mut() = Some(selected);
        Keyed::new(
            "bar",
            TabBar::new(selected)
                .tab(Tab::new("A").enabled(false))
                .tab(Tab::new("B").enabled(false)),
        )
    });
    t.pump();
    focus_bar(&mut t);
    key(&mut t, NamedKey::ArrowRight, Modifiers::default());
    let selected = out.borrow_mut().take().expect("built");
    assert_eq!(t.enter(|| selected.get()), 0);
}

#[test]
fn widgets_tab_07_control_tab_cycles_from_inside_pages() {
    let mut f = fixture(true);
    f.t.tap(&Finder::key("field"));
    f.t.pump();
    key(&mut f.t, NamedKey::Tab, control());
    assert_eq!(f.selected(), 1, "Control+Tab from inside the field");
    key(
        &mut f.t,
        NamedKey::Tab,
        Modifiers {
            control: true,
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(f.selected(), 0);
    key(&mut f.t, NamedKey::PageUp, control());
    assert_eq!(f.selected(), 3, "wraps, skipping Archived");
    key(&mut f.t, NamedKey::PageDown, control());
    assert_eq!(f.selected(), 0);
    // Plain Tab traverses as usual.
    focus_bar(&mut f.t);
    key(&mut f.t, NamedKey::Tab, Modifiers::default());
    assert_eq!(f.selected(), 0);
    let field = f.t.find(&Finder::key("field"));
    assert_eq!(focused(&mut f.t), Some(field));

    // A standalone bar binds them only while focused.
    let out = Rc::new(RefCell::new(None));
    let store = out.clone();
    let mut t = WidgetTester::with_size(W, H, move || {
        let selected = signal(0usize);
        *store.borrow_mut() = Some(selected);
        Column::new()
            .child(Keyed::new(
                "bar",
                TabBar::new(selected).tab(Tab::new("A")).tab(Tab::new("B")),
            ))
            .child(Keyed::new("other", Button::new("Other")))
    });
    t.pump();
    let selected = out.borrow_mut().take().expect("built");
    t.focus(&Finder::key("other"));
    key(&mut t, NamedKey::Tab, control());
    assert_eq!(t.enter(|| selected.get()), 0);
    focus_bar(&mut t);
    key(&mut t, NamedKey::PageDown, control());
    assert_eq!(t.enter(|| selected.get()), 1);
}

#[test]
fn widgets_tab_08_lazy_kept_pages() {
    let mut f = fixture(true);
    assert_eq!(f.builds.get(), 0, "unvisited pages are not built");
    f.t.enter_text(&Finder::key("field"), "draft");
    f.t.pump();
    let field = f.t.find(&Finder::key("field"));
    f.select(1);
    assert_eq!(f.builds.get(), 1);
    f.t.find(&Finder::text("Advanced page"));
    f.select(0);
    f.select(1);
    f.select(0);
    assert_eq!(f.builds.get(), 1, "built exactly once while kept");
    assert_eq!(f.live.get(), 1);
    assert_eq!(f.t.find(&Finder::key("field")), field, "the same element");
    assert_eq!(f.t.enter(|| f.controller.text()), "draft");
    // Pages fill the view.
    let panel = role_ids(&mut f.t, accesskit::Role::TabPanel)[0];
    let tabs = f.t.rect(f.t.find(&Finder::key("tabs")));
    let panel = f.t.rect(panel);
    assert_eq!(
        (panel.width(), panel.bottom),
        (tabs.width(), tabs.bottom),
        "the page fills the view below the bar"
    );

    let mut f = fixture(false);
    f.select(1);
    f.select(0);
    assert_eq!(f.live.get(), 0, "a left page is disposed");
    f.select(1);
    assert_eq!(f.builds.get(), 2, "and rebuilt on return");
}

#[test]
fn widgets_tab_09_focus_leaving_page() {
    let mut f = fixture(true);
    f.t.tap(&Finder::key("field"));
    f.t.pump();
    let bar = f.bar();
    f.select(1);
    assert_eq!(focused(&mut f.t), Some(bar), "focus moves to the bar");
    // Focus outside the pages is unchanged.
    f.t.focus(&Finder::key("after"));
    f.select(0);
    let after = f.t.find(&Finder::key("after"));
    assert_eq!(focused(&mut f.t), Some(after));

    // A standalone view clears it.
    let out = Rc::new(RefCell::new(None));
    let store = out.clone();
    let mut t = WidgetTester::with_size(W, H, move || {
        let selected = signal(0usize);
        *store.borrow_mut() = Some(selected);
        let controller = TextEditingController::new("");
        SizedBox::new(W, H).child(
            TabBarView::new(selected)
                .page(move || Keyed::new("field", TextField::new(controller.clone())))
                .page(|| Text::new("Other")),
        )
    });
    t.pump();
    t.tap(&Finder::key("field"));
    t.pump();
    let selected = out.borrow_mut().take().expect("built");
    t.enter(|| selected.set(1));
    t.pump();
    t.pump();
    assert_eq!(focused(&mut t), None, "the focus is cleared");
}

#[test]
fn widgets_tab_10_semantics() {
    let mut f = fixture(true);
    let tabs = tab_nodes(&mut f.t);
    assert_eq!(tabs.len(), 4);
    assert_eq!(
        tabs.iter()
            .map(|n| (n.label.as_str(), n.selected, n.disabled, n.position, n.size))
            .collect::<Vec<_>>(),
        [
            ("General", true, false, Some(1), Some(4)),
            ("Advanced", false, false, Some(2), Some(4)),
            ("Archived", false, true, Some(3), Some(4)),
            ("Data", false, false, Some(4), Some(4)),
        ]
    );
    let update = f.t.a11y_tree();
    let panels: Vec<_> = update
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == accesskit::Role::TabPanel)
        .map(|(_, n)| n.label().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(panels, ["General"]);
    let update = f.t.a11y_tree();
    let bar = f.bar();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| element(*id) == bar)
        .expect("the bar")
        .1;
    assert_eq!(node.active_descendant(), None, "only while focused");
    let advanced = tabs[1].id;
    let center = f.t.rect(advanced).center();
    press(&mut f.t, center, PointerButton::Primary);
    f.t.pump();
    let update = f.t.a11y_tree();
    let panels: Vec<_> = update
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == accesskit::Role::TabPanel)
        .map(|(_, n)| n.label().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(panels, ["Advanced"], "kept pages offstage have no nodes");
    let update = f.t.a11y_tree();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| element(*id) == bar)
        .expect("the bar")
        .1;
    assert_eq!(
        node.active_descendant(),
        Some(accesskit::NodeId(advanced.to_raw()))
    );
}

/// Tabs' `Click` and `Focus` through `ViewTree::perform_action` (VIEW-SEM-03).
#[test]
fn widgets_tab_10_click_and_focus_actions() {
    use tantu_view::core::Size;
    use tantu_view::layout::{BoxConstraints, NoTextMeasure};
    use tantu_view::scene::Scene;
    use tantu_view::{Role, ViewTree};
    let selected = Rc::new(Cell::new(None));
    let store = selected.clone();
    let mut tree = ViewTree::new(move || {
        let s = signal(0usize);
        store.set(Some(s));
        TabBar::new(s)
            .tab(Tab::new("A"))
            .tab(Tab::new("B"))
            .tab(Tab::new("C").enabled(false))
    });
    let frame = |tree: &mut ViewTree| {
        for _ in 0..2 {
            tree.frame(
                BoxConstraints::tight(Size::new(W, H)),
                &mut NoTextMeasure,
                &mut Scene::new(),
            );
        }
    };
    frame(&mut tree);
    let selected = selected.get().expect("built");
    let tab = |tree: &ViewTree, name: &str| {
        let mut stack = vec![tree.root()];
        while let Some(id) = stack.pop() {
            if tree
                .semantics(id)
                .is_some_and(|s| s.role == Role::Tab && s.name.as_deref() == Some(name))
            {
                return id;
            }
            stack.extend(tree.children(id));
        }
        panic!("no tab {name}")
    };
    let b = tab(&tree, "B");
    assert!(tree.perform_action(b, SemanticsAction::Click));
    frame(&mut tree);
    assert_eq!(tree.enter(|| selected.get()), 1);
    let c = tab(&tree, "C");
    tree.perform_action(c, SemanticsAction::Click);
    frame(&mut tree);
    assert_eq!(
        tree.enter(|| selected.get()),
        1,
        "disabled tabs aren't selected"
    );
    let a = tab(&tree, "A");
    assert!(tree.perform_action(a, SemanticsAction::Focus));
    assert!(tree.focused().is_some());
}
