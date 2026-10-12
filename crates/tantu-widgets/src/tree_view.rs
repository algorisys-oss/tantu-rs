//! Virtualized trees. Spec: `docs/specs/widgets/tree-view.md`.
use crate::{ScrollController, SizedBox};
use std::collections::BTreeSet;
use std::rc::Rc;
use tantu_view::reactive::Signal;
use tantu_view::{BuildCx, ElementId, IntoProp, View};

/// Stable application-assigned identity, unique throughout one source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeNodeId(pub u64);
/// An ordered forest with synchronous queries, consistent until the next revision.
pub trait TreeDataSource: 'static {
    /// Number of roots.
    fn root_count(&self) -> usize;
    /// Root at this zero-based index, or None for a missing entry.
    fn root(&self, index: usize) -> Option<TreeNodeId>;
    /// Immediate child count; zero for leaves and unknown nodes.
    fn child_count(&self, node: TreeNodeId) -> usize;
    /// Immediate child at this zero-based index, or None for a missing entry.
    fn child(&self, node: TreeNodeId, index: usize) -> Option<TreeNodeId>;
    /// Display/accessibility label; empty for unknown nodes.
    fn label(&self, node: TreeNodeId) -> String;
}
/// Snapshot passed to a custom row-content builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeRowState {
    /// Stable node identity.
    pub node: TreeNodeId,
    /// Zero-based depth.
    pub depth: usize,
    /// Whether the source reports children.
    pub expandable: bool,
    /// Whether this branch is open.
    pub expanded: bool,
    /// Whether this row is selected.
    pub selected: bool,
}
/// A vertically scrolling virtualized tree with a single focus stop.
pub struct TreeView;
impl TreeView {
    /// Builds default text labels, with 32 px rows and 20 px indentation.
    pub fn new(_source: Rc<dyn TreeDataSource>) -> Self {
        Self
    }
    /// Replaces label content only; chrome remains the tree's responsibility.
    pub fn builder<V: View>(
        _source: Rc<dyn TreeDataSource>,
        _build: impl Fn(TreeRowState) -> V + 'static,
    ) -> Self {
        Self
    }
    /// Row height; invalid values fall back to 32 px.
    pub fn item_extent(self, _extent: f32) -> Self {
        self
    }
    /// Per-depth indentation; negative or non-finite values fall back to 20 px.
    pub fn indentation(self, _indentation: f32) -> Self {
        self
    }
    /// Shares expansion state; defaults to an owner-scoped empty set.
    pub fn expanded(self, _expanded: Signal<BTreeSet<TreeNodeId>>) -> Self {
        self
    }
    /// Shares selection; defaults to an owner-scoped None signal.
    pub fn selection(self, _selection: Signal<Option<TreeNodeId>>) -> Self {
        self
    }
    /// Refreshes structure and mounted content when the revision changes; default 0.
    pub fn revision(self, _revision: impl IntoProp<u64>) -> Self {
        self
    }
    /// Shares the vertical scroll controller.
    pub fn controller(self, _controller: ScrollController) -> Self {
        self
    }
    /// Activates the selected node on a non-repeated Enter press.
    pub fn on_activate(self, _activate: impl Fn(TreeNodeId) + 'static) -> Self {
        self
    }
}
impl View for TreeView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        SizedBox::expand().build(cx)
    }
}
