//! Tests for `docs/specs/view/offstage.md`, rule A11Y-TREE-06.

use std::cell::Cell;
use std::rc::Rc;

use tantu_a11y::AccessibilityTree;
use tantu_a11y::accesskit::NodeId;
use tantu_view::core::Size;
use tantu_view::layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_view::scene::Scene;
use tantu_view::{AnyView, BuildCx, ElementId, Role, Semantics, View, ViewTree};

type Slot = Rc<Cell<Option<ElementId>>>;

/// A labelled list item, recorded in `slot`.
struct Item(&'static str, Slot);

impl View for Item {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(100.0), Some(20.0)), []);
        cx.set_semantics(
            id,
            Semantics {
                role: Role::ListItem,
                name: Some(self.0.to_owned()),
                ..Default::default()
            },
        );
        self.1.set(Some(id));
        id
    }
}

/// An offstage region around one child.
struct Hidden(AnyView);

impl View for Hidden {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let child = self.0;
        let id = cx.region(|cx| {
            child.build(cx);
        });
        cx.set_offstage(id, true);
        id
    }
}

/// A list of "shown" and an offstage "hidden" whose active descendant is the hidden item.
struct List(Slot, Slot);

impl View for List {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let hidden = self.1.clone();
        let list = cx.render(
            RenderFlex::column(),
            [
                AnyView::new(Item("shown", self.0)),
                AnyView::new(Hidden(AnyView::new(Item("hidden", self.1)))),
            ],
        );
        cx.set_semantics(
            list,
            Semantics {
                role: Role::List,
                active_descendant: hidden.get(),
                ..Default::default()
            },
        );
        list
    }
}

#[test]
fn a11y_tree_06_offstage_subtrees_have_no_nodes() {
    let (shown, hidden) = (Slot::default(), Slot::default());
    let (s, h) = (shown.clone(), hidden.clone());
    let mut tree = ViewTree::new(move || List(s, h));
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(400.0, 300.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    let (shown, hidden) = (shown.get().expect("built"), hidden.get().expect("built"));
    let update = AccessibilityTree::new("test").update(&tree);
    let id = |e: ElementId| NodeId(e.to_raw());
    assert!(update.nodes.iter().any(|(n, _)| *n == id(shown)));
    assert!(
        !update.nodes.iter().any(|(n, _)| *n == id(hidden)),
        "no node for the offstage item"
    );
    let (_, list) = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == tantu_a11y::accesskit::Role::List)
        .expect("the list");
    assert_eq!(
        list.children(),
        [id(shown)],
        "not counted among the children"
    );
    assert_eq!(
        list.active_descendant(),
        None,
        "an offstage active descendant is dropped"
    );
}
