//! # Tantu: Accessibility
//!
//! AccessKit trees from a view tree's element semantics (ADR 0013, spec
//! `docs/specs/a11y/semantics.md`). [`AccessibilityTree::update`] walks the elements and emits
//! a node per element with [`Semantics`](tantu_view::Semantics); [`AccessibilityTree::map_action`]
//! turns an AccessKit request into a [`SemanticsAction`](tantu_view::SemanticsAction) the view
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

use tantu_view::{ElementId, SemanticsAction, ViewTree};

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
    pub fn update(&mut self, tree: &ViewTree) -> accesskit::TreeUpdate {
        let _ = (tree, &self.app_name);
        todo!()
    }

    /// The element and view action an AccessKit request names, if the node is an element
    /// (A11Y-TREE-04).
    pub fn map_action(request: &accesskit::ActionRequest) -> Option<(ElementId, SemanticsAction)> {
        let _ = request;
        todo!()
    }

    /// The node id of an element (its id's bits).
    pub fn node_id(element: ElementId) -> accesskit::NodeId {
        let _ = element;
        todo!()
    }
}
