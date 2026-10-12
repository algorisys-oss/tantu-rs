//! Backend-neutral table properties and validated active descendants.
use tantu_a11y::AccessibilityTree;
use tantu_view::layout::RenderFlex;
use tantu_view::{BuildCx, ElementId, Semantics, SortDirection, View, ViewTree};

struct Grid;
struct Child(std::rc::Rc<std::cell::Cell<Option<ElementId>>>);
impl View for Child {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderFlex::column(), []);
        cx.set_semantics(
            id,
            Semantics {
                row_index: Some(0),
                column_index: Some(2),
                sort_direction: Some(SortDirection::Ascending),
                ..Default::default()
            },
        );
        self.0.set(Some(id));
        id
    }
}
impl View for Grid {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let out = std::rc::Rc::new(std::cell::Cell::new(None));
        let id = cx.render(
            RenderFlex::column(),
            [tantu_view::AnyView::new(Child(out.clone()))],
        );
        let child = out.get().expect("child built");
        cx.set_semantics(
            id,
            Semantics {
                row_count: Some(100),
                column_count: Some(3),
                multiselectable: true,
                active_descendant: Some(child),
                ..Default::default()
            },
        );
        id
    }
}
#[test]
fn view_sem_04_table_properties() {
    let tree = ViewTree::new(|| Grid);
    let grid = tree.children(tree.root())[0];
    let sem = tree.semantics(grid).expect("semantics");
    assert_eq!(sem.row_count, Some(100));
    assert!(sem.multiselectable);
    assert_eq!(sem.active_descendant, Some(tree.children(grid)[0]));
    assert_eq!(Semantics::default().row_count, None);
}
#[test]
fn a11y_tree_05_table_mapping() {
    let tree = ViewTree::new(|| Grid);
    let grid = tree.children(tree.root())[0];
    let child = tree.children(grid)[0];
    let update = AccessibilityTree::new("grid").update(&tree);
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == AccessibilityTree::node_id(grid))
        .expect("grid")
        .1;
    assert_eq!(node.row_count(), Some(100));
    assert_eq!(node.column_count(), Some(3));
    assert!(node.is_multiselectable());
    assert_eq!(
        node.active_descendant(),
        Some(AccessibilityTree::node_id(child))
    );
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == AccessibilityTree::node_id(child))
        .expect("cell")
        .1;
    assert_eq!(node.row_index(), Some(0));
    assert_eq!(node.column_index(), Some(2));
    assert_eq!(
        node.sort_direction(),
        Some(tantu_a11y::accesskit::SortDirection::Ascending)
    );
}
