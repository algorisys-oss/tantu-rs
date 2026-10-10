//! Tests for `docs/specs/view/focus.md`, rules VIEW-FOCUS-01..07.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::{Point, Size};
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_view::{
    AnyView, BuildCx, Dyn, ElementId, FocusOptions, Handled, Key, KeyEvent, Keyed, LogicalKey,
    Modifiers, NamedKey, Phase, PointerButton, PointerEvent, PointerKind, View, ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;

/// A box (100 × 20, or sized by its children) that can be focusable and logs focus changes,
/// key events and pointer presses.
struct Node {
    name: &'static str,
    focus: Option<FocusOptions>,
    stop_tab: bool,
    column: bool,
    root_keys: bool,
    children: Vec<AnyView>,
    log: Log,
}

fn node(name: &'static str, log: &Log) -> Node {
    Node {
        name,
        focus: None,
        stop_tab: false,
        column: false,
        root_keys: false,
        children: Vec::new(),
        log: log.clone(),
    }
}

impl Node {
    fn focusable(mut self, options: FocusOptions) -> Self {
        self.focus = Some(options);
        self
    }

    fn child(mut self, child: impl View) -> Self {
        self.children.push(AnyView::new(child));
        self
    }
}

fn key_name(key: &LogicalKey) -> String {
    match key {
        LogicalKey::Named(named) => format!("{named:?}"),
        LogicalKey::Character(c) => c.to_string(),
    }
}

impl View for Node {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = if self.column {
            cx.render(RenderFlex::column(), self.children)
        } else if self.children.is_empty() {
            cx.render(RenderConstrainedBox::sized(Some(100.0), Some(20.0)), [])
        } else {
            cx.render(
                RenderConstrainedBox::sized(Some(100.0), None),
                self.children,
            )
        };
        let name = self.name;
        if let Some(options) = self.focus {
            cx.focusable(id, options);
            let log = self.log.clone();
            cx.on_focus_change(id, move |focused| {
                log.borrow_mut().push(format!("{name} {focused}"));
            });
        }
        let targets = if self.root_keys {
            vec![(cx.parent(), "root")]
        } else {
            vec![(id, name)]
        };
        for (target, label) in targets {
            for phase in [Phase::Capture, Phase::Bubble] {
                let log = self.log.clone();
                let stop_tab = self.stop_tab;
                cx.on_key(target, phase, move |k| {
                    if !k.event.pressed {
                        return Handled::Continue;
                    }
                    let tag = if k.phase == Phase::Capture {
                        "capture"
                    } else {
                        "bubble"
                    };
                    log.borrow_mut()
                        .push(format!("{label} {tag} {}", key_name(&k.event.key)));
                    if stop_tab && k.event.key == LogicalKey::Named(NamedKey::Tab) {
                        Handled::Stop
                    } else {
                        Handled::Continue
                    }
                });
            }
        }
        let log = self.log.clone();
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if matches!(p.event.kind, PointerKind::Down(_)) {
                log.borrow_mut().push(format!("{name} pointer down"));
            }
            Handled::Continue
        });
        id
    }
}

const TRAVERSABLE: FocusOptions = FocusOptions {
    traversable: true,
    focus_on_press: false,
};

/// The test tree, in a column (each box 100 × 20 unless noted):
///
/// ```text
/// col
/// ├─ root  registers key handlers on the root element   y 0
/// ├─ a     focusable                                    y 20
/// ├─ b     not focusable                                y 40
/// ├─ c     focusable, not traversable                   y 60
/// ├─ (Dyn) d   focusable                                y 80
/// └─ e     focusable, focus on press, with child f      y 100..120
/// ```
struct Fixture {
    tree: ViewTree,
    log: Log,
}

impl Fixture {
    fn new(stop_tab_on: Option<&'static str>) -> Fixture {
        let log = Log::default();
        let l = log.clone();
        let mut tree = ViewTree::new(move || {
            let mut col = node("col", &l);
            col.column = true;
            col.root_keys = false;
            let mut a = node("a", &l).focusable(TRAVERSABLE);
            a.stop_tab = stop_tab_on == Some("a");
            let mut root = node("root", &l);
            root.root_keys = true;
            let l2 = l.clone();
            col.child(Keyed::new("root-keys", root))
                .child(Keyed::new("a", a))
                .child(Keyed::new("b", node("b", &l)))
                .child(Keyed::new(
                    "c",
                    node("c", &l).focusable(FocusOptions {
                        traversable: false,
                        focus_on_press: false,
                    }),
                ))
                .child(Dyn::new(move || {
                    Keyed::new("d", node("d", &l2).focusable(TRAVERSABLE))
                }))
                .child(Keyed::new(
                    "e",
                    node("e", &l)
                        .focusable(FocusOptions {
                            traversable: true,
                            focus_on_press: true,
                        })
                        .child(Keyed::new("f", node("f", &l))),
                ))
        });
        tree.layout(
            BoxConstraints::loose(Size::new(400.0, 400.0)),
            &mut NoTextMeasure,
        );
        Fixture { tree, log }
    }

    fn id(&self, name: &str) -> ElementId {
        self.tree.find_key(&Key::from(name))[0]
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.borrow_mut())
    }

    fn press(&mut self, key: LogicalKey, modifiers: Modifiers) -> bool {
        let handled = self
            .tree
            .dispatch_key(KeyEvent::press(key.clone(), modifiers));
        self.tree.dispatch_key(KeyEvent::release(key, modifiers));
        handled
    }
}

fn tab() -> LogicalKey {
    LogicalKey::Named(NamedKey::Tab)
}

const SHIFT: Modifiers = Modifiers {
    shift: true,
    control: false,
    alt: false,
    super_key: false,
};

#[test]
fn view_focus_01_focus_and_unfocus() {
    let mut f = Fixture::new(None);
    assert_eq!(f.tree.focused(), None);
    let (a, b) = (f.id("a"), f.id("b"));
    assert!(f.tree.focus(a));
    assert_eq!(f.tree.focused(), Some(a));
    assert!(!f.tree.focus(b));
    assert_eq!(f.tree.focused(), Some(a));
    f.tree.remove(b);
    assert!(!f.tree.focus(b));
    f.tree.unfocus();
    assert_eq!(f.tree.focused(), None);
}

#[test]
fn view_focus_02_focus_changes_are_reported() {
    let mut f = Fixture::new(None);
    let (a, d) = (f.id("a"), f.id("d"));
    f.tree.focus(a);
    assert_eq!(f.take(), ["a true"]);
    f.tree.focus(d);
    assert_eq!(f.take(), ["a false", "d true"]);
    f.tree.focus(d);
    assert!(f.take().is_empty());
    f.tree.unfocus();
    assert_eq!(f.take(), ["d false"]);
}

#[test]
fn view_focus_03_traversal() {
    let mut f = Fixture::new(None);
    let (a, c, d, e) = (f.id("a"), f.id("c"), f.id("d"), f.id("e"));
    assert_eq!(f.tree.focus_next(), Some(a));
    assert_eq!(f.tree.focus_next(), Some(d));
    assert_eq!(f.tree.focus_next(), Some(e));
    assert_eq!(f.tree.focus_next(), Some(a));
    assert_eq!(f.tree.focus_previous(), Some(e));
    f.tree.unfocus();
    assert_eq!(f.tree.focus_previous(), Some(e));
    // Not traversable, but focusable directly; traversal from it goes on in tree order.
    assert!(f.tree.focus(c));
    assert_eq!(f.tree.focus_next(), Some(d));

    // No traversable element: the focus stays.
    let log = Log::default();
    let l = log.clone();
    let mut tree = ViewTree::new(move || {
        node("only", &l).focusable(FocusOptions {
            traversable: false,
            focus_on_press: false,
        })
    });
    assert_eq!(tree.focus_next(), None);
    let only = tree.children(tree.root())[0];
    tree.focus(only);
    assert_eq!(tree.focus_next(), Some(only));
}

#[test]
fn view_focus_04_key_dispatch_along_the_focused_path() {
    let mut f = Fixture::new(None);
    let d = f.id("d");
    f.tree.focus(d);
    f.take();
    let enter = LogicalKey::Named(NamedKey::Enter);
    assert!(f.press(enter.clone(), Modifiers::default()));
    assert_eq!(
        f.take(),
        [
            "root capture Enter",
            "col capture Enter",
            "d capture Enter",
            "d bubble Enter",
            "col bubble Enter",
            "root bubble Enter",
        ]
    );
    // Nothing focused: only the root's handlers.
    f.tree.unfocus();
    f.take();
    f.press(LogicalKey::Character("x".into()), Modifiers::default());
    assert_eq!(f.take(), ["root capture x", "root bubble x"]);
}

#[test]
fn view_focus_05_tab_moves_focus() {
    let mut f = Fixture::new(None);
    let (a, d, e) = (f.id("a"), f.id("d"), f.id("e"));
    f.tree.focus(a);
    assert!(f.press(tab(), Modifiers::default()));
    assert_eq!(f.tree.focused(), Some(d));
    f.press(tab(), Modifiers::default());
    assert_eq!(f.tree.focused(), Some(e));
    f.press(tab(), SHIFT);
    assert_eq!(f.tree.focused(), Some(d));
    // A release alone moves nothing.
    f.tree
        .dispatch_key(KeyEvent::release(tab(), Modifiers::default()));
    assert_eq!(f.tree.focused(), Some(d));

    // A handled Tab doesn't move focus.
    let mut f = Fixture::new(Some("a"));
    let a = f.id("a");
    f.tree.focus(a);
    f.press(tab(), Modifiers::default());
    assert_eq!(f.tree.focused(), Some(a));
}

fn pointer(tree: &mut ViewTree, kind: PointerKind, x: f32, y: f32) {
    tree.dispatch_pointer(PointerEvent {
        kind,
        position: Point::new(x, y),
    });
}

#[test]
fn view_focus_06_focus_on_press() {
    let mut f = Fixture::new(None);
    let (a, e) = (f.id("a"), f.id("e"));
    f.tree.focus(a);
    f.take();
    // A press on f, inside e: e is focused before the pointer handlers run.
    pointer(
        &mut f.tree,
        PointerKind::Down(PointerButton::Primary),
        10.0,
        110.0,
    );
    pointer(
        &mut f.tree,
        PointerKind::Up(PointerButton::Primary),
        10.0,
        110.0,
    );
    assert_eq!(f.tree.focused(), Some(e));
    let log = f.take();
    assert_eq!(
        &log[..3],
        ["a false", "e true", "f pointer down"],
        "{log:?}"
    );
    // A press on a (no focus on press) leaves the focus.
    pointer(
        &mut f.tree,
        PointerKind::Down(PointerButton::Primary),
        10.0,
        30.0,
    );
    pointer(
        &mut f.tree,
        PointerKind::Up(PointerButton::Primary),
        10.0,
        30.0,
    );
    assert_eq!(f.tree.focused(), Some(e));
    // Secondary presses don't focus.
    f.tree.unfocus();
    pointer(
        &mut f.tree,
        PointerKind::Down(PointerButton::Secondary),
        10.0,
        110.0,
    );
    assert_eq!(f.tree.focused(), None);
}

#[test]
fn view_focus_07_removing_the_focus() {
    let mut f = Fixture::new(None);
    let d = f.id("d");
    f.tree.focus(d);
    f.take();
    f.tree.remove(d);
    assert_eq!(f.tree.focused(), None);
    assert_eq!(f.take(), ["d false"]);
    // Removing an ancestor of the focused element.
    let mut f = Fixture::new(None);
    let e = f.id("e");
    f.tree.focus(e);
    f.take();
    let col = f.tree.children(f.tree.root())[0];
    f.tree.remove(col);
    assert_eq!(f.tree.focused(), None);
    assert_eq!(f.take(), ["e false"]);
    // Their key handlers are gone: only the root's would run, and they were in col's subtree.
    assert!(!f.press(LogicalKey::Named(NamedKey::Enter), Modifiers::default()));
    assert!(f.take().is_empty());
}
