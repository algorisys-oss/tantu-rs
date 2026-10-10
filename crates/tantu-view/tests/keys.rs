//! Tests for VIEW-KEY-01 (`docs/specs/test/widget-tester.md`).

use tantu_layout::RenderConstrainedBox;
use tantu_view::{AnyView, BuildCx, Dyn, ElementId, ElementKind, Key, Keyed, View, ViewTree};

struct Boxed;

impl View for Boxed {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderConstrainedBox::sized(Some(1.0), Some(1.0)), [])
    }
}

struct Group(Vec<AnyView>);

impl View for Group {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderConstrainedBox::sized(None, None), self.0)
    }
}

#[test]
fn view_key_01_keyed_elements() {
    let mut tree = ViewTree::new(|| {
        Keyed::new(
            "outer",
            Group(vec![
                AnyView::new(Keyed::new("x", Boxed)),
                AnyView::new(Boxed),
                AnyView::new(Keyed::new("x", Boxed)),
                AnyView::new(Keyed::new("dyn", Dyn::new(|| Boxed))),
            ]),
        )
    });
    let outer = tree.find_key(&Key::from("outer"));
    assert_eq!(outer, [tree.children(tree.root())[0]]);
    let group = outer[0];
    let xs = tree.find_key(&Key::from("x"));
    assert_eq!(xs, [tree.children(group)[0], tree.children(group)[2]]);
    let region = tree.find_key(&"dyn".into());
    assert_eq!(region.len(), 1);
    assert_eq!(tree.kind(region[0]), Some(ElementKind::Region));
    assert!(tree.find_key(&Key::from("none")).is_empty());
    // Removing an element drops its key, and its descendants'.
    tree.remove(xs[0]);
    assert_eq!(tree.find_key(&Key::from("x")), [xs[1]]);
    tree.remove(group);
    assert!(tree.find_key(&Key::from("x")).is_empty());
    assert!(tree.find_key(&Key::from("outer")).is_empty());
}
