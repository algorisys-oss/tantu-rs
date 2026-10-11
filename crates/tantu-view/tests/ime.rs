//! Tests for `docs/specs/widgets/text-field-ime.md`, rules VIEW-IME-01..02.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::{Rect, Size};
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_view::{AnyView, BuildCx, ElementId, FocusOptions, ImeEvent, Key, Keyed, View, ViewTree};

type Log = Rc<RefCell<Vec<String>>>;

/// A focusable 100 × 20 box logging IME events, with an IME area when `area` is set.
struct Box_ {
    name: &'static str,
    area: Option<Rect>,
    log: Log,
}

impl View for Box_ {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(100.0), Some(20.0)), []);
        cx.focusable(id, FocusOptions::default());
        let (name, log) = (self.name, self.log);
        cx.on_ime(id, move |i| {
            assert_eq!(i.element, id);
            log.borrow_mut().push(format!("{name} {:?}", i.event));
        });
        let area = self.area;
        cx.ime_area(id, move || area);
        id
    }
}

struct Column(Vec<AnyView>);

impl View for Column {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderFlex::column(), self.0)
    }
}

fn fixture(log: &Log) -> ViewTree {
    let log = log.clone();
    let mut tree = ViewTree::new(move || {
        Column(vec![
            AnyView::new(Keyed::new(
                "a",
                Box_ {
                    name: "a",
                    area: Some(Rect::from_ltwh(1.0, 2.0, 3.0, 4.0)),
                    log: log.clone(),
                },
            )),
            AnyView::new(Keyed::new(
                "b",
                Box_ {
                    name: "b",
                    area: None,
                    log: log.clone(),
                },
            )),
            AnyView::new(Keyed::new("c", RenderOnly)),
        ])
    });
    tree.layout(
        BoxConstraints::loose(Size::new(300.0, 300.0)),
        &mut NoTextMeasure,
    );
    tree
}

/// A focusable box with no IME handlers.
struct RenderOnly;

impl View for RenderOnly {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(100.0), Some(20.0)), []);
        cx.focusable(id, FocusOptions::default());
        id
    }
}

fn id(tree: &ViewTree, key: &str) -> ElementId {
    tree.find_key(&Key::from(key))[0]
}

#[test]
fn view_ime_01_dispatch_to_the_focused_element() {
    let log: Log = Rc::default();
    let mut tree = fixture(&log);
    let commit = ImeEvent::Commit("x".into());
    // Nothing focused: nothing runs.
    assert!(!tree.dispatch_ime(commit.clone()));
    assert!(log.borrow().is_empty());
    let (a, b, c) = (id(&tree, "a"), id(&tree, "b"), id(&tree, "c"));
    tree.focus(a);
    assert!(tree.dispatch_ime(commit.clone()));
    assert!(tree.dispatch_ime(ImeEvent::Preedit {
        text: "y".into(),
        cursor: None,
    }));
    assert_eq!(
        log.borrow().clone(),
        ["a Commit(\"x\")", "a Preedit { text: \"y\", cursor: None }"]
    );
    log.borrow_mut().clear();
    // Only the focused element, not its ancestors or siblings.
    tree.focus(b);
    assert!(tree.dispatch_ime(ImeEvent::Enabled));
    assert_eq!(log.borrow().clone(), ["b Enabled"]);
    tree.focus(c);
    assert!(!tree.dispatch_ime(commit.clone()));
    // Removing an element drops its handlers.
    tree.focus(a);
    tree.remove(a);
    assert!(!tree.dispatch_ime(commit));
}

#[test]
fn view_ime_02_ime_area() {
    let log: Log = Rc::default();
    let mut tree = fixture(&log);
    assert_eq!(tree.ime_area(), None);
    let (a, b, c) = (id(&tree, "a"), id(&tree, "b"), id(&tree, "c"));
    tree.focus(a);
    assert_eq!(tree.ime_area(), Some(Rect::from_ltwh(1.0, 2.0, 3.0, 4.0)));
    // In window coordinates: b sits 20 px down, but provides no area.
    tree.focus(b);
    assert_eq!(tree.ime_area(), None);
    tree.focus(c);
    assert_eq!(tree.ime_area(), None);
    tree.unfocus();
    assert_eq!(tree.ime_area(), None);
    // An element lower in the column reports its area offset by its position.
    let log: Log = Rc::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            Column(vec![
                AnyView::new(RenderOnly),
                AnyView::new(Keyed::new(
                    "a",
                    Box_ {
                        name: "a",
                        area: Some(Rect::from_ltwh(1.0, 2.0, 3.0, 4.0)),
                        log,
                    },
                )),
            ])
        })
    };
    tree.layout(
        BoxConstraints::loose(Size::new(300.0, 300.0)),
        &mut NoTextMeasure,
    );
    let a = id(&tree, "a");
    tree.focus(a);
    assert_eq!(tree.ime_area(), Some(Rect::from_ltwh(1.0, 22.0, 3.0, 4.0)));
}
