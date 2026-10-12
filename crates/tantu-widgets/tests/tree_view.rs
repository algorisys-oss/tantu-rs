//! Rules in the TreeView specification, exercised through the real widget/a11y pipeline.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;
use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_theme::Theme;
use tantu_view::core::Point;
use tantu_view::reactive::{Signal, on_cleanup, signal};
use tantu_view::{
    ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
    Role, SemanticsAction, Show,
};
use tantu_widgets::{ScrollController, Text, TreeDataSource, TreeNodeId as Id, TreeView};

#[derive(Default)]
struct Model {
    roots: RefCell<Vec<Option<Id>>>,
    children: RefCell<HashMap<Id, Vec<Option<Id>>>>,
    suffix: RefCell<String>,
    queries: Cell<usize>,
    labels: Cell<usize>,
}
impl TreeDataSource for Model {
    fn root_count(&self) -> usize {
        self.queries.set(self.queries.get() + 1);
        self.roots.borrow().len()
    }
    fn root(&self, i: usize) -> Option<Id> {
        self.queries.set(self.queries.get() + 1);
        self.roots.borrow().get(i).copied().flatten()
    }
    fn child_count(&self, id: Id) -> usize {
        self.queries.set(self.queries.get() + 1);
        self.children.borrow().get(&id).map_or(0, Vec::len)
    }
    fn child(&self, id: Id, i: usize) -> Option<Id> {
        self.queries.set(self.queries.get() + 1);
        self.children.borrow().get(&id)?.get(i).copied().flatten()
    }
    fn label(&self, id: Id) -> String {
        self.labels.set(self.labels.get() + 1);
        format!("Node {}{}", id.0, self.suffix.borrow())
    }
}
fn model() -> Rc<Model> {
    Rc::new(Model {
        roots: RefCell::new(vec![Some(Id(0)), Some(Id(9))]),
        children: RefCell::new(HashMap::from([
            (Id(0), vec![Some(Id(1)), Some(Id(4))]),
            (Id(1), vec![Some(Id(2)), Some(Id(3))]),
        ])),
        ..Default::default()
    })
}
#[derive(Clone, Copy)]
struct Handles {
    expanded: Signal<BTreeSet<Id>>,
    selected: Signal<Option<Id>>,
    revision: Signal<u64>,
    controller: ScrollController,
    shown: Signal<bool>,
}
struct Fixture {
    t: WidgetTester,
    h: Handles,
    activations: Rc<RefCell<Vec<Id>>>,
}
impl Fixture {
    fn with(m: Rc<Model>, expanded: &[u64]) -> Self {
        Self::custom(m, expanded, 32.0, 20.0, None)
    }
    fn custom(
        m: Rc<Model>,
        expanded: &[u64],
        extent: f32,
        indent: f32,
        live: Option<Rc<Cell<usize>>>,
    ) -> Self {
        let out = Rc::new(RefCell::new(None));
        let capture = out.clone();
        let initial: BTreeSet<_> = expanded.iter().map(|n| Id(*n)).collect();
        let activations = Rc::new(RefCell::new(Vec::new()));
        let calls = activations.clone();
        let t = WidgetTester::with_size(240.0, 128.0, move || {
            let h = Handles {
                expanded: signal(initial.clone()),
                selected: signal(None),
                revision: signal(0),
                controller: ScrollController::new(),
                shown: signal(true),
            };
            *capture.borrow_mut() = Some(h);
            Show::new(
                move || h.shown.get(),
                move || {
                    let tree = if let Some(live) = &live {
                        let live = live.clone();
                        TreeView::builder(m.clone(), move |state| {
                            live.set(live.get() + 1);
                            let drop = live.clone();
                            on_cleanup(move || drop.set(drop.get() - 1));
                            Text::new(format!("Custom {} {}", state.node.0, state.selected))
                        })
                    } else {
                        TreeView::new(m.clone())
                    };
                    let calls = calls.clone();
                    Keyed::new(
                        "tree",
                        tree.expanded(h.expanded)
                            .selection(h.selected)
                            .revision(h.revision)
                            .controller(h.controller)
                            .item_extent(extent)
                            .indentation(indent)
                            .on_activate(move |id| calls.borrow_mut().push(id)),
                    )
                },
            )
        });
        let h = out.borrow_mut().take().unwrap();
        Self { t, h, activations }
    }
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
    }
    fn select(&mut self, id: Option<u64>) {
        self.t.enter(|| self.h.selected.set(id.map(Id)));
        self.pump();
    }
    fn selected(&self) -> Option<Id> {
        self.t.enter(|| self.h.selected.get())
    }
    fn expand(&mut self, ids: &[u64]) {
        self.t
            .enter(|| self.h.expanded.set(ids.iter().map(|n| Id(*n)).collect()));
        self.pump();
    }
    fn key(&mut self, key: NamedKey) {
        self.t
            .press_key(LogicalKey::Named(key), Modifiers::default());
        self.pump();
    }
    fn item(&mut self, n: u64) -> ElementId {
        self.t
            .a11y_tree()
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == accesskit::Role::TreeItem
                    && node.label() == Some(format!("Node {n}").as_str())
            })
            .map(|(id, _)| ElementId::from_raw(id.0).unwrap())
            .expect("mounted row")
    }
    fn ids(&mut self) -> Vec<String> {
        self.t
            .a11y_tree()
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == accesskit::Role::TreeItem)
            .map(|(_, n)| n.label().unwrap().to_owned())
            .collect()
    }
    fn press(&mut self, n: u64, x: f32) {
        let id = self.item(n);
        let r = self.t.rect(id);
        self.t.pointer(PointerEvent {
            kind: PointerKind::Down(PointerButton::Primary),
            position: Point::new(x, r.center().y),
            modifiers: Default::default(),
        });
        self.pump();
        self.t.pointer(PointerEvent {
            kind: PointerKind::Up(PointerButton::Primary),
            position: Point::new(x, r.center().y),
            modifiers: Default::default(),
        });
        self.pump();
    }
}
#[test]
fn widgets_tree_01_preorder_closed_and_empty() {
    let mut f = Fixture::with(model(), &[]);
    assert_eq!(f.ids(), ["Node 0", "Node 9"]);
    f.expand(&[0, 1]);
    assert_eq!(f.ids(), ["Node 0", "Node 1", "Node 2", "Node 3", "Node 4"]);
    f.expand(&[2]);
    assert_eq!(f.ids(), ["Node 0", "Node 9"]);
    let mut empty = Fixture::with(Rc::new(Model::default()), &[]);
    assert!(empty.ids().is_empty());
    empty.t.focus(&Finder::key("tree"));
    empty.key(NamedKey::Enter);
    assert!(empty.activations.borrow().is_empty());
}
#[test]
fn widgets_tree_02_extents_virtualization_and_bounded_work() {
    let m = model();
    *m.roots.borrow_mut() = (0..10000).map(|n| Some(Id(n))).collect();
    m.children.borrow_mut().clear();
    let mut f = Fixture::custom(m.clone(), &[], f32::NAN, f32::INFINITY, None);
    assert!(f.ids().len() <= 6);
    let row = f.item(0);
    assert_eq!(
        f.t.rect(row).size(),
        tantu_view::core::Size::new(240.0, 32.0)
    );
    let label = f.t.find(&Finder::text("Node 0"));
    assert_eq!(f.t.rect(label).left, 20.0);
    let queries = m.queries.get();
    f.t.enter(|| f.h.controller.jump_to(200000.0));
    f.pump();
    assert!(f.ids().len() <= 6);
    assert_eq!(queries, m.queries.get());
    assert!(m.labels.get() < 40);
    let mut f = Fixture::custom(model(), &[0, 1], -1.0, -1.0, None);
    let row = f.item(2);
    assert_eq!(f.t.rect(row).height(), 32.0);
    assert_eq!(f.t.rect(f.t.find(&Finder::text("Node 2"))).left, 60.0);
    let f = Fixture::custom(model(), &[0, 1], 40.0, f32::MAX, None);
    let r = f.t.rect(f.t.find(&Finder::text("Node 2")));
    assert!(r.left.is_finite() && r.left <= 240.0 && r.width() >= 0.0);
}
#[test]
fn widgets_tree_03_disclosure_selection_focus_and_empty_space() {
    let mut f = Fixture::with(model(), &[]);
    f.press(0, 10.0);
    assert_eq!(f.selected(), Some(Id(0)));
    assert!(f.t.enter(|| f.h.expanded.get()).contains(&Id(0)));
    assert_eq!(
        f.t.a11y_tree().focus,
        accesskit::NodeId(f.t.find(&Finder::key("tree")).to_raw())
    );
    f.press(1, 100.0);
    assert_eq!(f.selected(), Some(Id(1)));
    assert!(!f.t.enter(|| f.h.expanded.get()).contains(&Id(1)));
    f.press(1, 30.0);
    assert!(f.t.enter(|| f.h.expanded.get()).contains(&Id(1)));
    assert!(f.activations.borrow().is_empty());
    f.expand(&[]);
    f.press(9, 10.0);
    assert_eq!(f.selected(), Some(Id(9)));
    assert!(!f.t.enter(|| f.h.expanded.get()).contains(&Id(9)));
    f.t.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Secondary),
        position: Point::new(100.0, 10.0),
        modifiers: Default::default(),
    });
    f.pump();
    assert_eq!(f.selected(), Some(Id(9)));
    f.t.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: Point::new(100.0, 100.0),
        modifiers: Default::default(),
    });
    f.pump();
    assert_eq!(f.selected(), Some(Id(9)));
}
#[test]
fn widgets_tree_04_navigation_pages_modifiers_and_tab() {
    let mut f = Fixture::with(model(), &[0, 1]);
    f.t.focus(&Finder::key("tree"));
    f.key(NamedKey::ArrowDown);
    assert_eq!(f.selected(), Some(Id(0)));
    f.key(NamedKey::PageDown);
    assert_eq!(f.selected(), Some(Id(4)));
    assert_eq!(f.t.enter(|| f.h.controller.offset()), 32.0);
    f.key(NamedKey::End);
    assert_eq!(f.selected(), Some(Id(9)));
    f.key(NamedKey::PageUp);
    assert_eq!(f.selected(), Some(Id(1)));
    f.key(NamedKey::Home);
    assert_eq!(f.selected(), Some(Id(0)));
    f.select(None);
    f.key(NamedKey::ArrowUp);
    assert_eq!(f.selected(), Some(Id(9)));
    let offset = f.t.enter(|| f.h.controller.offset());
    f.t.press_key(
        LogicalKey::Named(NamedKey::Home),
        Modifiers {
            control: true,
            ..Default::default()
        },
    );
    f.pump();
    assert_eq!(f.selected(), Some(Id(9)));
    assert_eq!(f.t.enter(|| f.h.controller.offset()), offset);
    f.key(NamedKey::Tab);
    assert_eq!(
        f.t.a11y_tree().focus,
        accesskit::NodeId(f.t.find(&Finder::key("tree")).to_raw())
    );
}
#[test]
fn widgets_tree_05_branch_keys_and_activation_repeats() {
    let mut f = Fixture::with(model(), &[]);
    f.t.focus(&Finder::key("tree"));
    f.key(NamedKey::ArrowRight);
    assert_eq!(f.selected(), Some(Id(0)));
    f.key(NamedKey::ArrowRight);
    assert!(f.t.enter(|| f.h.expanded.get()).contains(&Id(0)));
    f.key(NamedKey::ArrowRight);
    assert_eq!(f.selected(), Some(Id(1)));
    f.t.press_key(LogicalKey::Character(" ".into()), Modifiers::default());
    f.pump();
    f.key(NamedKey::ArrowRight);
    assert_eq!(f.selected(), Some(Id(2)));
    f.key(NamedKey::ArrowLeft);
    assert_eq!(f.selected(), Some(Id(1)));
    f.key(NamedKey::ArrowLeft);
    assert!(!f.t.enter(|| f.h.expanded.get()).contains(&Id(1)));
    f.key(NamedKey::Enter);
    assert_eq!(*f.activations.borrow(), vec![Id(1)]);
}
#[test]
fn widgets_tree_06_collapse_fallback_external_and_restore() {
    let mut f = Fixture::with(model(), &[0, 1]);
    f.select(Some(2));
    f.expand(&[0]);
    assert_eq!(f.selected(), Some(Id(1)));
    f.expand(&[0, 1]);
    f.select(Some(3));
    f.expand(&[1]);
    assert_eq!(f.selected(), Some(Id(0)));
    f.expand(&[0, 1]);
    assert!(f.ids().contains(&"Node 3".to_owned()));
    f.select(Some(800));
    assert_eq!(f.selected(), None);
    f.expand(&[]);
    f.select(Some(2));
    assert_eq!(f.selected(), None);
    assert!(f.activations.borrow().is_empty());
}
#[test]
fn widgets_tree_07_revision_reorder_refresh_and_removal() {
    let m = model();
    let mut f = Fixture::with(m.clone(), &[0, 1]);
    f.select(Some(2));
    m.children.borrow_mut().get_mut(&Id(1)).unwrap().reverse();
    *m.suffix.borrow_mut() = " changed".into();
    f.t.enter(|| f.h.revision.update(|r| *r += 1));
    f.pump();
    assert_eq!(f.selected(), Some(Id(2)));
    f.t.find(&Finder::text("Node 2 changed"));
    m.children.borrow_mut().insert(Id(1), vec![]);
    f.t.enter(|| f.h.revision.update(|r| *r += 1));
    f.pump();
    assert_eq!(f.selected(), None);
    let q = m.queries.get();
    f.select(Some(4));
    f.t.hover(&Finder::text("Node 4 changed"));
    assert_eq!(m.queries.get(), q);
    f.t.enter(|| f.h.shown.set(false));
    f.pump();
    assert!(f.t.find_all(&Finder::key("tree")).is_empty());
    assert!(f.t.enter(|| f.h.selected.try_get()).is_some());
}
#[test]
fn widgets_tree_08_identity_reposition_and_scope_disposal() {
    let m = model();
    let mut f = Fixture::with(m.clone(), &[0]);
    let id = f.item(4);
    let top = f.t.rect(id).top;
    f.expand(&[0, 1]);
    assert_eq!(id, f.item(4));
    assert!(f.t.rect(id).top > top);
    let live = Rc::new(Cell::new(0));
    *m.roots.borrow_mut() = (0..100).map(|n| Some(Id(n))).collect();
    m.children.borrow_mut().clear();
    let mut f = Fixture::custom(m, &[], 32.0, 20.0, Some(live.clone()));
    assert!(live.get() > 0 && live.get() <= 6);
    f.t.enter(|| f.h.controller.jump_to(1000.0));
    f.pump();
    assert!(live.get() <= 6);
    f.t.enter(|| f.h.shown.set(false));
    f.pump();
    assert_eq!(live.get(), 0);
}
#[test]
fn widgets_tree_09_missing_duplicates_cycles_and_deep_chain() {
    let m = model();
    *m.roots.borrow_mut() = vec![Some(Id(0)), None, Some(Id(0))];
    m.children
        .borrow_mut()
        .insert(Id(0), vec![Some(Id(0)), None, Some(Id(9)), Some(Id(9))]);
    let mut f = Fixture::with(m, &[0]);
    assert_eq!(f.ids(), ["Node 0", "Node 9"]);
    let m = Rc::new(Model {
        roots: RefCell::new(vec![Some(Id(0))]),
        children: RefCell::new((0..20000).map(|n| (Id(n), vec![Some(Id(n + 1))])).collect()),
        ..Default::default()
    });
    let mut f = Fixture::with(m, &(0..20000).collect::<Vec<_>>());
    assert!(f.ids().len() <= 6);
    f.t.enter(|| f.h.controller.jump_to(640000.0));
    f.pump();
    assert!(f.ids().iter().any(|s| s == "Node 20000"));
}
#[test]
fn widgets_tree_10_themed_chrome_and_goldens() {
    let mut f = Fixture::with(model(), &[0, 1]);
    f.select(Some(2));
    f.t.focus(&Finder::key("tree"));
    f.pump();
    let tree = f.t.find(&Finder::key("tree"));
    assert!(
        f.t.frame()
            .scene
            .entries()
            .iter()
            .any(|e| e.element == Some(tree))
    );
    f.t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/tree-light.png"
    ));
    f.t.enter(|| f.t.theme().set(Theme::dark()));
    f.pump();
    assert_eq!(f.selected(), Some(Id(2)));
    assert_eq!(tree, f.t.find(&Finder::key("tree")));
    f.t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/tree-dark.png"
    ));
}
fn raw() -> tantu_view::ViewTree {
    tantu_view::ViewTree::new(|| TreeView::new(model()))
}
fn raw_frame(tree: &mut tantu_view::ViewTree) {
    tree.frame(
        tantu_view::layout::BoxConstraints::tight(tantu_view::core::Size::new(240.0, 128.0)),
        &mut tantu_view::layout::NoTextMeasure,
        &mut tantu_view::scene::Scene::new(),
    );
    tree.frame(
        tantu_view::layout::BoxConstraints::tight(tantu_view::core::Size::new(240.0, 128.0)),
        &mut tantu_view::layout::NoTextMeasure,
        &mut tantu_view::scene::Scene::new(),
    );
}
fn raw_item(tree: &tantu_view::ViewTree, name: &str) -> ElementId {
    let mut stack = vec![tree.root()];
    while let Some(id) = stack.pop() {
        if tree
            .semantics(id)
            .is_some_and(|s| s.role == Role::TreeItem && s.name.as_deref() == Some(name))
        {
            return id;
        }
        stack.extend(tree.children(id));
    }
    panic!("missing tree row {name}")
}
#[test]
fn widgets_tree_11_semantics_and_actions_independent_of_focus() {
    let mut t = raw();
    raw_frame(&mut t);
    let root = raw_item(&t, "Node 0");
    let s = t.semantics(root).unwrap();
    assert_eq!(
        (s.level, s.position_in_set, s.size_of_set, s.expanded),
        (Some(1), Some(1), Some(2), Some(false))
    );
    assert!(s.actions.expand);
    assert!(t.perform_action(root, SemanticsAction::Expand));
    raw_frame(&mut t);
    let row = raw_item(&t, "Node 1");
    assert!(t.perform_action(row, SemanticsAction::Click));
    raw_frame(&mut t);
    assert!(!t.semantics(row).unwrap().expanded.unwrap());
    let leaf = raw_item(&t, "Node 4");
    assert_eq!(t.semantics(leaf).unwrap().expanded, None);
    assert!(!t.perform_action(leaf, SemanticsAction::Expand));
    assert!(t.perform_action(root, SemanticsAction::Collapse));
    raw_frame(&mut t);
    assert!(!t.perform_action(row, SemanticsAction::Expand));
}
#[test]
fn view_sem_04_tree_fields_defaults_and_action_support() {
    let s = tantu_view::Semantics::default();
    assert_eq!(
        (s.expanded, s.level, s.position_in_set, s.size_of_set),
        (None, None, None, None)
    );
    assert!(!s.actions.expand && !s.actions.collapse);
    let mut t = raw();
    raw_frame(&mut t);
    let id = raw_item(&t, "Node 0");
    assert!(t.perform_action(id, SemanticsAction::Expand));
    raw_frame(&mut t);
    assert!(t.semantics(id).unwrap().actions.collapse);
}
#[test]
fn a11y_tree_05_tree_mapping_properties() {
    let mut f = Fixture::with(model(), &[0]);
    let id = f.item(1);
    let update = f.t.a11y_tree();
    let n = &update
        .nodes
        .iter()
        .find(|(n, _)| n.0 == id.to_raw())
        .unwrap()
        .1;
    assert_eq!(n.role(), accesskit::Role::TreeItem);
    assert_eq!(
        (
            n.level(),
            n.position_in_set(),
            n.size_of_set(),
            n.is_expanded()
        ),
        (Some(2), Some(1), Some(2), Some(false))
    );
    assert!(n.supports_action(accesskit::Action::Expand));
}

#[test]
fn widgets_tree_01_million_closed_children_are_not_queried() {
    struct Generated(Rc<Cell<usize>>);
    impl TreeDataSource for Generated {
        fn root_count(&self) -> usize {
            1
        }
        fn root(&self, i: usize) -> Option<Id> {
            (i == 0).then_some(Id(0))
        }
        fn child_count(&self, n: Id) -> usize {
            if n == Id(0) { 1_000_000 } else { 0 }
        }
        fn child(&self, _: Id, i: usize) -> Option<Id> {
            self.0.set(self.0.get() + 1);
            Some(Id(i as u64 + 1))
        }
        fn label(&self, n: Id) -> String {
            format!("Node {}", n.0)
        }
    }
    let calls = Rc::new(Cell::new(0));
    let src = calls.clone();
    let mut t =
        WidgetTester::with_size(240.0, 128.0, move || TreeView::new(Rc::new(Generated(src))));
    t.pump();
    t.find(&Finder::text("Node 0"));
    assert_eq!(calls.get(), 0);
    assert_eq!(
        t.a11y_tree()
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == accesskit::Role::TreeItem)
            .count(),
        1
    );
}

#[test]
fn widgets_tree_05_repeat_and_key_release_do_not_activate_or_toggle() {
    use tantu_view::{KeyEvent, ViewTree};
    let calls = Rc::new(Cell::new(0));
    let capture = calls.clone();
    let mut t = ViewTree::new(move || {
        TreeView::new(model()).on_activate(move |_| capture.set(capture.get() + 1))
    });
    raw_frame(&mut t);
    let tree = t.children(t.root())[0];
    assert!(t.focus(tree));
    t.dispatch_key(KeyEvent::press(
        LogicalKey::Named(NamedKey::ArrowDown),
        Modifiers::default(),
    ));
    t.dispatch_key(KeyEvent::press(
        LogicalKey::Named(NamedKey::Enter),
        Modifiers::default(),
    ));
    assert_eq!(calls.get(), 1);
    let mut repeat = KeyEvent::press(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    repeat.repeat = true;
    t.dispatch_key(repeat);
    t.dispatch_key(KeyEvent::release(
        LogicalKey::Named(NamedKey::Enter),
        Modifiers::default(),
    ));
    assert_eq!(calls.get(), 1);
    let mut repeat = KeyEvent::press(LogicalKey::Character(" ".into()), Modifiers::default());
    repeat.repeat = true;
    t.dispatch_key(repeat);
    raw_frame(&mut t);
    let root = raw_item(&t, "Node 0");
    assert_eq!(t.semantics(root).unwrap().expanded, Some(false));
}
