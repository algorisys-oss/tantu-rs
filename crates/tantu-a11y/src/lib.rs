//! # Tantu: Accessibility
//!
//! AccessKit trees from a view tree's element semantics (ADR 0013, spec
//! `docs/specs/a11y/semantics.md`). [`AccessibilityTree::update`] walks the elements and emits
//! a node per element with `Semantics` (from `tantu-view`); [`AccessibilityTree::map_action`]
//! turns an AccessKit request into a `SemanticsAction` the view
//! tree performs. The `accesskit` data crate is re-exported for callers.
//!
//! ```
//! use tantu_a11y::AccessibilityTree;
//! use tantu_view::ViewTree;
//! use tantu_view::layout::{BoxConstraints, RenderConstrainedBox};
//!
//! struct Leaf;
//! impl tantu_view::View for Leaf {
//!     fn build(self, cx: &mut tantu_view::BuildCx<'_>) -> tantu_view::ElementId {
//!         cx.render(RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED), [])
//!     }
//! }
//! let tree = ViewTree::new(|| Leaf);
//! let update = AccessibilityTree::new("example").update(&tree);
//! assert_eq!(update.nodes.len(), 1, "the window node");
//! ```

#![forbid(unsafe_code)]

pub use accesskit;

use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Rect, Role, Toggled, Tree, TreeUpdate,
};
use tantu_view::{ElementId, Semantics, SemanticsAction, ViewTree};

/// The window node's id: 0, which no element has (element ids are never 0).
const WINDOW: NodeId = NodeId(0);

/// Builds AccessKit tree updates from a view tree (A11Y-TREE-01..04).
pub struct AccessibilityTree {
    app_name: String,
}

impl AccessibilityTree {
    /// A builder naming the window node `app_name`.
    pub fn new(app_name: impl Into<String>) -> Self {
        AccessibilityTree {
            app_name: app_name.into(),
        }
    }

    /// The whole tree as one update (the root is the window node), with the focus
    /// (A11Y-TREE-01..03).
    pub fn update(&mut self, tree: &ViewTree) -> TreeUpdate {
        let mut nodes = Vec::new();
        let mut children = Vec::new();
        visit(tree, tree.root(), &mut nodes, &mut children);
        let mut window = Node::new(Role::Window);
        window.set_label(self.app_name.as_str());
        if let Some(bounds) = tree.bounds(tree.root()) {
            window.set_bounds(rect(bounds));
        }
        window.set_children(children);
        nodes.push((WINDOW, window));
        let focus = tree
            .focused()
            .filter(|id| tree.semantics(*id).is_some())
            .map_or(WINDOW, Self::node_id);
        TreeUpdate {
            nodes,
            tree: Some(Tree::new(WINDOW)),
            tree_id: accesskit::TreeId::ROOT,
            focus,
        }
    }

    /// The element and view action an AccessKit request names, if the node is an element
    /// (A11Y-TREE-04).
    pub fn map_action(request: &ActionRequest) -> Option<(ElementId, SemanticsAction)> {
        let element = ElementId::from_raw(request.target_node.0)?;
        let action = match request.action {
            Action::Focus => SemanticsAction::Focus,
            Action::Expand => SemanticsAction::Expand,
            Action::Collapse => SemanticsAction::Collapse,
            Action::Click => SemanticsAction::Click,
            Action::Increment => SemanticsAction::Increment,
            Action::Decrement => SemanticsAction::Decrement,
            Action::SetValue => match &request.data {
                Some(ActionData::Value(value)) => SemanticsAction::SetValue(value.to_string()),
                _ => return None,
            },
            Action::ScrollIntoView => SemanticsAction::ScrollIntoView,
            _ => return None,
        };
        Some((element, action))
    }

    /// The node id of an element (its id's bits).
    pub fn node_id(element: ElementId) -> NodeId {
        NodeId(element.to_raw())
    }
}

/// Adds the nodes of `element`'s subtree to `nodes`, appending the ids of its semantic roots
/// to `siblings` (A11Y-TREE-01).
fn visit(
    tree: &ViewTree,
    element: ElementId,
    nodes: &mut Vec<(NodeId, Node)>,
    siblings: &mut Vec<NodeId>,
) {
    if tree.is_offstage(element) {
        return; // Offstage subtrees have no nodes (A11Y-TREE-06).
    }
    match tree.semantics(element) {
        Some(semantics) => {
            let mut children = Vec::new();
            for child in tree.children(element) {
                visit(tree, *child, nodes, &mut children);
            }
            let mut node = node(semantics);
            if let Some(active) = semantics.active_descendant.filter(|active| {
                if *active == element
                    || tree.semantics(*active).is_none()
                    || tree.is_offstage(*active)
                {
                    return false;
                }
                let mut parent = tree.parent(*active);
                while let Some(id) = parent {
                    if id == element {
                        return true;
                    }
                    parent = tree.parent(id);
                }
                false
            }) {
                node.set_active_descendant(AccessibilityTree::node_id(active));
            }
            if let Some(bounds) = tree.bounds(element) {
                node.set_bounds(rect(bounds));
            }
            node.set_children(children);
            let id = AccessibilityTree::node_id(element);
            nodes.push((id, node));
            siblings.push(id);
        }
        None => {
            for child in tree.children(element) {
                visit(tree, *child, nodes, siblings);
            }
        }
    }
}

/// An AccessKit node with the semantics' properties (A11Y-TREE-02).
fn node(semantics: &Semantics) -> Node {
    use tantu_view::Role as R;
    let role = match semantics.role {
        R::Window => Role::Window,
        R::Group => Role::Group,
        R::Label => Role::Label,
        R::Button => Role::Button,
        R::CheckBox => Role::CheckBox,
        R::RadioButton => Role::RadioButton,
        R::Switch => Role::Switch,
        R::Slider => Role::Slider,
        R::TextInput => Role::TextInput,
        R::MultilineTextInput => Role::MultilineTextInput,
        R::Image => Role::Image,
        R::List => Role::List,
        R::ListItem => Role::ListItem,
        R::ScrollView => Role::ScrollView,
        R::Tooltip => Role::Tooltip,
        R::Grid => Role::Grid,
        R::ColumnHeader => Role::ColumnHeader,
        R::Cell => Role::Cell,
        R::Tree => Role::Tree,
        R::TreeItem => Role::TreeItem,
        R::TabList => Role::TabList,
        R::Tab => Role::Tab,
        R::TabPanel => Role::TabPanel,
        R::Splitter => Role::Splitter,
        R::Menu => Role::Menu,
        R::MenuItem => Role::MenuItem,
        R::MenuItemCheckBox => Role::MenuItemCheckBox,
        R::MenuBar => Role::MenuBar,
        R::Toolbar => Role::Toolbar,
        R::Status => Role::Status,
    };
    let mut node = Node::new(role);
    if let Some(v) = semantics.expanded {
        node.set_expanded(v);
    }
    if let Some(v) = semantics.level {
        node.set_level(v);
    }
    if let Some(v) = semantics.position_in_set {
        node.set_position_in_set(v);
    }
    if let Some(v) = semantics.size_of_set {
        node.set_size_of_set(v);
    }
    if let Some(v) = semantics.row_count {
        node.set_row_count(v);
    }
    if let Some(v) = semantics.column_count {
        node.set_column_count(v);
    }
    if let Some(v) = semantics.row_index {
        node.set_row_index(v);
    }
    if let Some(v) = semantics.column_index {
        node.set_column_index(v);
    }
    if semantics.multiselectable {
        node.set_multiselectable();
    }
    if let Some(v) = semantics.sort_direction {
        node.set_sort_direction(match v {
            tantu_view::SortDirection::Ascending => accesskit::SortDirection::Ascending,
            tantu_view::SortDirection::Descending => accesskit::SortDirection::Descending,
        });
    }
    if let Some(name) = &semantics.name {
        node.set_label(name.as_str());
    }
    if let Some(value) = &semantics.value {
        node.set_value(value.as_str());
    }
    if let Some(numeric) = semantics.numeric {
        node.set_numeric_value(numeric.value);
        node.set_min_numeric_value(numeric.min);
        node.set_max_numeric_value(numeric.max);
        node.set_numeric_value_step(numeric.step);
    }
    if let Some(toggled) = semantics.toggled {
        node.set_toggled(match toggled {
            tantu_view::Toggled::False => Toggled::False,
            tantu_view::Toggled::True => Toggled::True,
            tantu_view::Toggled::Mixed => Toggled::Mixed,
        });
    }
    if let Some(selected) = semantics.selected {
        node.set_selected(selected);
    }
    if semantics.disabled {
        node.set_disabled();
    }
    let actions = semantics.actions;
    for (on, action) in [
        (actions.click, Action::Click),
        (actions.expand, Action::Expand),
        (actions.collapse, Action::Collapse),
        (actions.focus || semantics.focusable, Action::Focus),
        (actions.increment, Action::Increment),
        (actions.decrement, Action::Decrement),
        (actions.set_value, Action::SetValue),
        (actions.scroll_into_view, Action::ScrollIntoView),
    ] {
        if on {
            node.add_action(action);
        }
    }
    node
}

/// A Tantu rect as an AccessKit rect (A11Y-TREE-03).
fn rect(bounds: tantu_view::core::Rect) -> Rect {
    Rect::new(
        f64::from(bounds.left),
        f64::from(bounds.top),
        f64::from(bounds.right),
        f64::from(bounds.bottom),
    )
}
