//! Tests for `docs/specs/view/focus-scope.md`, rules VIEW-REVEAL-01..03 and VIEW-SCOPE-01..04.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use tantu_core::Size;
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_view::{
    AnyView, BuildCx, ElementId, ElementKind, FocusOptions, Key, Keyed, Role, Semantics,
    SemanticsAction, View, ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;
type Names = Rc<RefCell<HashMap<ElementId, &'static str>>>;

/// A box (100 × 20, or a column of its children) with optional focus, scope, semantics and a
/// "scroller" action handler that logs what it is asked to reveal.
struct Node {
    name: &'static str,
    focus: Option<FocusOptions>,
    scope: bool,
    region: bool,
    scroller: bool,
    semantics: bool,
    children: Vec<AnyView>,
    log: Log,
    names: Names,
    refuse: Rc<Cell<bool>>,
}

impl Node {
    fn child(mut self, child: impl View) -> Self {
        self.children.push(AnyView::new(child));
        self
    }
}

impl View for Node {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = if self.region {
            cx.region(|cx| {
                for child in self.children {
                    child.build(cx);
                }
            })
        } else if self.children.is_empty() {
            cx.render(RenderConstrainedBox::sized(Some(100.0), Some(20.0)), [])
        } else {
            cx.render(RenderFlex::column(), self.children)
        };
        let name = self.name;
        self.names.borrow_mut().insert(id, name);
        if let Some(options) = self.focus {
            cx.focusable(id, options);
        }
        if self.scope {
            cx.focus_scope(id);
        }
        if self.semantics {
            cx.set_semantics(
                id,
                Semantics {
                    role: Role::Button,
                    ..Default::default()
                },
            );
        }
        if self.scroller {
            let (log, names, refuse) = (self.log, self.names, self.refuse);
            cx.on_action(id, move |a| {
                if *a.action != SemanticsAction::ScrollIntoView {
                    return false;
                }
                let target = names.borrow().get(&a.target).copied().unwrap_or("?");
                log.borrow_mut().push(format!("{name} reveals {target}"));
                !(name == "inner" && refuse.get())
            });
        }
        id
    }
}

const TRAVERSABLE: FocusOptions = FocusOptions {
    traversable: true,
    focus_on_press: false,
};

const NOT_TRAVERSABLE: FocusOptions = FocusOptions {
    traversable: false,
    focus_on_press: false,
};

/// The test tree, under a column:
///
/// ```text
/// col
/// ├─ a          focusable
/// ├─ outer      scroller (a container that reveals descendants)
/// │  └─ inner   scroller
/// │     ├─ leaf     focusable, with semantics
/// │     └─ plain    nothing
/// ├─ S          scope (a region)
/// │  ├─ b       focusable
/// │  ├─ c       focusable
/// │  └─ T       scope (a render element)
/// │     └─ t    focusable
/// ├─ d          focusable
/// └─ E          scope (a region)
///    └─ e       focusable, not traversable
/// ```
struct Fixture {
    tree: ViewTree,
    log: Log,
    refuse: Rc<Cell<bool>>,
}

impl Fixture {
    fn new() -> Fixture {
        let log = Log::default();
        let names = Names::default();
        let refuse = Rc::new(Cell::new(false));
        let node = {
            let (log, names, refuse) = (log.clone(), names.clone(), refuse.clone());
            move |name: &'static str| Node {
                name,
                focus: None,
                scope: false,
                region: false,
                scroller: false,
                semantics: false,
                children: Vec::new(),
                log: log.clone(),
                names: names.clone(),
                refuse: refuse.clone(),
            }
        };
        let mut tree = ViewTree::new(move || {
            let focusable = |name, options| Node {
                focus: Some(options),
                ..node(name)
            };
            let scroller = |name| Node {
                scroller: true,
                ..node(name)
            };
            let scope = |name, region| Node {
                scope: true,
                region,
                ..node(name)
            };
            let leaf = Node {
                semantics: true,
                ..focusable("leaf", TRAVERSABLE)
            };
            node("col")
                .child(Keyed::new("a", focusable("a", TRAVERSABLE)))
                .child(Keyed::new(
                    "outer",
                    scroller("outer").child(Keyed::new(
                        "inner",
                        scroller("inner")
                            .child(Keyed::new("leaf", leaf))
                            .child(Keyed::new("plain", node("plain"))),
                    )),
                ))
                .child(Keyed::new(
                    "S",
                    scope("S", true)
                        .child(Keyed::new("b", focusable("b", TRAVERSABLE)))
                        .child(Keyed::new("c", focusable("c", TRAVERSABLE)))
                        .child(Keyed::new(
                            "T",
                            scope("T", false).child(Keyed::new("t", focusable("t", TRAVERSABLE))),
                        )),
                ))
                .child(Keyed::new("d", focusable("d", TRAVERSABLE)))
                .child(Keyed::new(
                    "E",
                    scope("E", true).child(Keyed::new("e", focusable("e", NOT_TRAVERSABLE))),
                ))
        });
        tree.layout(
            BoxConstraints::loose(Size::new(400.0, 400.0)),
            &mut NoTextMeasure,
        );
        Fixture { tree, log, refuse }
    }

    fn id(&self, name: &str) -> ElementId {
        self.tree.find_key(&Key::from(name))[0]
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.borrow_mut())
    }

    fn focused(&self) -> Option<&'static str> {
        let focused = self.tree.focused()?;
        ["a", "leaf", "b", "c", "t", "d", "e"]
            .into_iter()
            .find(|name| self.id(name) == focused)
    }
}

#[test]
fn view_reveal_01_every_container_reveals_the_one_below() {
    let mut f = Fixture::new();
    let (leaf, plain, inner, a) = (f.id("leaf"), f.id("plain"), f.id("inner"), f.id("a"));
    // The inner container reveals the element, the outer one reveals the inner container.
    assert!(f.tree.ensure_visible(leaf));
    assert_eq!(f.take(), ["inner reveals leaf", "outer reveals inner"]);
    // Semantics are not needed.
    assert!(f.tree.ensure_visible(plain));
    assert_eq!(f.take(), ["inner reveals plain", "outer reveals inner"]);
    // A container that declines leaves the target as it was; the walk still reaches the root.
    f.refuse.set(true);
    assert!(f.tree.ensure_visible(leaf));
    assert_eq!(f.take(), ["inner reveals leaf", "outer reveals leaf"]);
    f.refuse.set(false);
    // The element's own handlers are not asked: the walk starts at its parent.
    assert!(f.tree.ensure_visible(inner));
    assert_eq!(f.take(), ["outer reveals inner"]);
    // No container above: false, nothing called.
    assert!(!f.tree.ensure_visible(a));
    assert!(f.take().is_empty());
    // An unknown element.
    f.tree.remove(a);
    assert!(!f.tree.ensure_visible(a));
    assert!(f.take().is_empty());
}

#[test]
fn view_reveal_02_traversal_reveals_the_new_focus() {
    let mut f = Fixture::new();
    let (a, leaf, b) = (f.id("a"), f.id("leaf"), f.id("b"));
    // `focus` doesn't scroll.
    assert!(f.tree.focus(leaf));
    assert!(f.take().is_empty());
    // Tab forward from `a` lands on `leaf` and reveals it.
    f.tree.focus(a);
    assert_eq!(f.tree.focus_next(), Some(leaf));
    assert_eq!(f.take(), ["inner reveals leaf", "outer reveals inner"]);
    // Backward too.
    f.tree.focus(b);
    f.take();
    assert_eq!(f.tree.focus_previous(), Some(leaf));
    assert_eq!(f.take(), ["inner reveals leaf", "outer reveals inner"]);
    // Moving to an element with no container above calls nothing.
    assert_eq!(f.tree.focus_next(), Some(b));
    assert!(f.take().is_empty());
    // `focus_into` doesn't scroll either.
    let s = f.id("S");
    f.tree.unfocus();
    assert_eq!(f.tree.focus_into(s), Some(b));
    assert!(f.take().is_empty());
}

#[test]
fn view_reveal_03_scroll_into_view_action_is_ensure_visible() {
    let mut f = Fixture::new();
    let (leaf, plain) = (f.id("leaf"), f.id("plain"));
    // Every enclosing container, not only the first.
    assert!(f.tree.perform_action(leaf, SemanticsAction::ScrollIntoView));
    assert_eq!(f.take(), ["inner reveals leaf", "outer reveals inner"]);
    // Without semantics the action is refused.
    assert!(
        !f.tree
            .perform_action(plain, SemanticsAction::ScrollIntoView)
    );
    assert!(f.take().is_empty());
}

#[test]
fn view_scope_01_innermost_scope() {
    let mut f = Fixture::new();
    let (s, t, e) = (f.id("S"), f.id("T"), f.id("E"));
    let root = f.tree.root();
    assert_eq!(f.tree.kind(s), Some(ElementKind::Region));
    assert_eq!(f.tree.kind(t), Some(ElementKind::Render));
    assert_eq!(f.tree.focus_scope_of(f.id("b")), Some(s));
    assert_eq!(f.tree.focus_scope_of(f.id("t")), Some(t));
    assert_eq!(f.tree.focus_scope_of(f.id("e")), Some(e));
    // A scope is its own innermost scope.
    assert_eq!(f.tree.focus_scope_of(s), Some(s));
    assert_eq!(f.tree.focus_scope_of(t), Some(t));
    // The root is the scope of everything outside a marked one, and of itself.
    assert_eq!(f.tree.focus_scope_of(f.id("a")), Some(root));
    assert_eq!(f.tree.focus_scope_of(f.id("leaf")), Some(root));
    assert_eq!(f.tree.focus_scope_of(root), Some(root));
    // An unknown element.
    let a = f.id("a");
    f.tree.remove(a);
    assert_eq!(f.tree.focus_scope_of(a), None);
}

#[test]
fn view_scope_02_traversal_stays_in_the_scope() {
    let mut f = Fixture::new();
    // Inside S: b → c → t (the nested scope's contents) → b (wrapping within S).
    f.tree.focus(f.id("b"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("c"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("t"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("b"));
    f.tree.focus_previous();
    assert_eq!(f.focused(), Some("t"));
    f.tree.focus_previous();
    assert_eq!(f.focused(), Some("c"));
    // Inside T, the only traversable element is t: Tab stays there.
    f.tree.focus(f.id("t"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("t"));
    // The root scope covers everything: a → leaf → b … d → a.
    f.tree.focus(f.id("a"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("leaf"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("b"));
    f.tree.focus(f.id("d"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("a"));
    f.tree.focus_previous();
    assert_eq!(f.focused(), Some("d"));
    // A scope with no traversable element leaves the focus unchanged.
    f.tree.focus(f.id("e"));
    assert_eq!(f.tree.focus_next(), Some(f.id("e")));
    assert_eq!(f.tree.focus_previous(), Some(f.id("e")));
    assert_eq!(f.focused(), Some("e"));
    // Nothing focused: the root scope, from either end.
    f.tree.unfocus();
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("a"));
    f.tree.unfocus();
    f.tree.focus_previous();
    assert_eq!(f.focused(), Some("d"));
}

#[test]
fn view_scope_03_focus_into_restores_or_starts() {
    let mut f = Fixture::new();
    let (s, t, e, a) = (f.id("S"), f.id("T"), f.id("E"), f.id("a"));
    let root = f.tree.root();
    // Nothing remembered yet: the first traversable descendant.
    assert_eq!(f.tree.focus_into(s), Some(f.id("b")));
    assert_eq!(f.focused(), Some("b"));
    assert_eq!(f.tree.focus_into(t), Some(f.id("t")));
    // Every enclosing scope remembers the focus.
    f.tree.focus(f.id("c"));
    f.tree.unfocus();
    assert_eq!(f.tree.focus_into(s), Some(f.id("c")));
    assert_eq!(f.tree.focus_into(t), Some(f.id("t")));
    f.tree.focus(f.id("d"));
    assert_eq!(f.tree.focus_into(root), Some(f.id("d")));
    f.tree.unfocus();
    assert_eq!(f.tree.focus_into(root), Some(f.id("d")));
    assert_eq!(f.tree.focus_into(s), Some(f.id("c")));
    // Neither remembered nor traversable: nothing changes.
    f.tree.focus(a);
    assert_eq!(f.tree.focus_into(e), None);
    assert_eq!(f.focused(), Some("a"));
    // Not a scope.
    assert_eq!(f.tree.focus_into(f.id("b")), None);
    assert_eq!(f.focused(), Some("a"));
}

#[test]
fn view_scope_04_removal_forgets() {
    let mut f = Fixture::new();
    let s = f.id("S");
    // The remembered element is removed: back to the first traversable descendant.
    let c = f.id("c");
    f.tree.focus(c);
    f.tree.unfocus();
    f.tree.remove(c);
    assert_eq!(f.tree.focus_into(s), Some(f.id("b")));
    // The scope itself is removed.
    f.tree.unfocus();
    f.tree.remove(s);
    assert_eq!(f.tree.focus_into(s), None);
    assert_eq!(f.tree.focus_scope_of(s), None);
    assert_eq!(f.tree.focused(), None);
    // Traversal no longer sees the scope's contents.
    f.tree.focus(f.id("a"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("leaf"));
    f.tree.focus_next();
    assert_eq!(f.focused(), Some("d"));
}
