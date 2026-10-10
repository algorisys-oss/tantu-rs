//! [`LayoutTree`]: the layout half of the render tree (ADR 0009). Nodes hold a [`RenderBox`],
//! their children and parent data; the tree runs the layout pass with caching and relayout
//! boundaries.

use std::any::Any;
use std::fmt;

use tantu_core::{Arena, Id, Size, Vec2};

use crate::BoxConstraints;

/// A node in a [`LayoutTree`]. `Copy`, 8 bytes; stale after the node is removed (never reused).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayoutId(Id);

impl LayoutId {
    /// The underlying arena id.
    pub fn id(self) -> Id {
        self.0
    }

    /// Stable 64-bit form, never 0 (`Id::to_bits`).
    pub fn to_bits(self) -> u64 {
        todo!()
    }
}

/// Why a tree change was refused. Nothing changes when an error is returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreeError {
    /// The id is stale (or from another tree, when that isn't mistaken for a node here).
    UnknownNode(LayoutId),
    /// The child already has a different parent.
    HasParent {
        /// The child that was offered.
        child: LayoutId,
        /// Its current parent.
        parent: LayoutId,
    },
    /// The child is the parent itself or one of its ancestors.
    Cycle(LayoutId),
    /// The same child appears twice in the list.
    Duplicate(LayoutId),
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeError::UnknownNode(id) => write!(f, "unknown layout node {id:?}"),
            TreeError::HasParent { child, parent } => {
                write!(f, "layout node {child:?} already has parent {parent:?}")
            }
            TreeError::Cycle(id) => write!(f, "layout node {id:?} would become its own ancestor"),
            TreeError::Duplicate(id) => write!(f, "layout node {id:?} appears twice"),
        }
    }
}

impl std::error::Error for TreeError {}

/// A layout algorithm: what a node does in a layout pass. Built-in layouts and app-defined
/// custom layouts implement it the same way.
pub trait RenderBox: Any {
    /// Lays out the children through `children` (each gets constraints and returns its size,
    /// then gets an offset) and returns this node's size. The tree constrains the returned size
    /// to `constraints`.
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size;

    /// True when this node's size depends only on its constraints (not on its children). Such
    /// a node is a relayout boundary. Default `false`.
    fn sized_by_parent(&self) -> bool {
        false
    }

    /// The smallest width this node can have without its content overflowing, at the given
    /// height (`f32::INFINITY` for "any"). Default 0.
    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        0.0
    }

    /// The width beyond which more width doesn't reduce the height, at the given height.
    /// Default 0.
    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        0.0
    }

    /// As [`min_intrinsic_width`](Self::min_intrinsic_width), for the height at a given width.
    /// Default 0.
    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        0.0
    }

    /// As [`max_intrinsic_width`](Self::max_intrinsic_width), for the height at a given width.
    /// Default 0.
    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        0.0
    }
}

/// A node's children during its [`RenderBox::perform_layout`], by index in child order.
pub struct LayoutChildren<'a> {
    tree: &'a mut LayoutTree,
    parent: LayoutId,
}

impl LayoutChildren<'_> {
    /// Number of children.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True with no children.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// The child's id; `None` out of range.
    pub fn id(&self, index: usize) -> Option<LayoutId> {
        let _ = index;
        todo!()
    }

    /// Lays out child `index` with `constraints` and returns its size; this node's layout
    /// depends on that size. `Size::ZERO` out of range.
    pub fn layout(&mut self, index: usize, constraints: BoxConstraints) -> Size {
        let _ = (index, constraints);
        todo!()
    }

    /// Lays out child `index` when this node doesn't use the child's size (the child becomes a
    /// relayout boundary).
    pub fn layout_ignoring_size(&mut self, index: usize, constraints: BoxConstraints) {
        let _ = (index, constraints);
        todo!()
    }

    /// Sets child `index`'s offset from this node's top-left corner. Ignored out of range.
    pub fn set_offset(&mut self, index: usize, offset: Vec2) {
        let _ = (index, offset);
        todo!()
    }

    /// The child's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, index: usize) -> Option<&T> {
        let _ = index;
        todo!()
    }

    /// The child's minimum intrinsic width at `height` (see [`LayoutTree::min_intrinsic_width`]).
    pub fn min_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        let _ = (index, height);
        todo!()
    }

    /// The child's maximum intrinsic width at `height`.
    pub fn max_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        let _ = (index, height);
        todo!()
    }

    /// The child's minimum intrinsic height at `width`.
    pub fn min_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        let _ = (index, width);
        todo!()
    }

    /// The child's maximum intrinsic height at `width`.
    pub fn max_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        let _ = (index, width);
        todo!()
    }
}

/// A node's children while it computes an intrinsic size: the same queries, no layout.
pub struct IntrinsicChildren<'a> {
    tree: &'a mut LayoutTree,
    parent: LayoutId,
}

impl IntrinsicChildren<'_> {
    /// Number of children.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True with no children.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// The child's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, index: usize) -> Option<&T> {
        let _ = index;
        todo!()
    }

    /// The child's minimum intrinsic width at `height`; 0 out of range.
    pub fn min_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        let _ = (index, height);
        todo!()
    }

    /// The child's maximum intrinsic width at `height`.
    pub fn max_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        let _ = (index, height);
        todo!()
    }

    /// The child's minimum intrinsic height at `width`.
    pub fn min_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        let _ = (index, width);
        todo!()
    }

    /// The child's maximum intrinsic height at `width`.
    pub fn max_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        let _ = (index, width);
        todo!()
    }
}

/// One node: its layout object, structure and last layout.
struct Node {
    render: Option<Box<dyn RenderBox>>,
}

/// An arena of layout nodes and the layout pass over them.
#[derive(Default)]
pub struct LayoutTree {
    nodes: Arena<Node>,
}

impl fmt::Debug for LayoutTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayoutTree")
            .field("len", &self.nodes.len())
            .finish_non_exhaustive()
    }
}

impl LayoutTree {
    /// An empty tree.
    pub fn new() -> Self {
        todo!()
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True with no nodes.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// True if `id` is a node of this tree.
    pub fn contains(&self, id: LayoutId) -> bool {
        let _ = id;
        todo!()
    }

    /// Adds a node with no parent, no children and no parent data. It needs layout.
    pub fn insert(&mut self, render: impl RenderBox) -> LayoutId {
        let _ = render;
        todo!()
    }

    /// Removes `id` and all its descendants; detaches it from its parent (which then needs
    /// layout). Returns the number of nodes removed (0 for an unknown id).
    pub fn remove(&mut self, id: LayoutId) -> usize {
        let _ = id;
        todo!()
    }

    /// Makes `children` the ordered children of `parent`. Each child must have no parent or
    /// already be a child of `parent`; previous children not in the list are detached (they
    /// become roots, not removed). The parent needs layout if the list changed.
    pub fn set_children(
        &mut self,
        parent: LayoutId,
        children: &[LayoutId],
    ) -> Result<(), TreeError> {
        let _ = (parent, children);
        todo!()
    }

    /// The ordered children (empty for an unknown id).
    pub fn children(&self, id: LayoutId) -> &[LayoutId] {
        let _ = id;
        todo!()
    }

    /// The parent, if any.
    pub fn parent(&self, id: LayoutId) -> Option<LayoutId> {
        let _ = id;
        todo!()
    }

    /// The node's layout object, if it is a `T`.
    pub fn get<T: RenderBox>(&self, id: LayoutId) -> Option<&T> {
        let _ = id;
        todo!()
    }

    /// The node's layout object for changing it, if it is a `T`. Marks the node as needing
    /// layout (the escape hatch for layouts that can't be compared; prefer
    /// [`set`](Self::set)).
    pub fn get_mut<T: RenderBox>(&mut self, id: LayoutId) -> Option<&mut T> {
        let _ = id;
        todo!()
    }

    /// Replaces the node's layout object with `render` and marks the node as needing layout
    /// only if it differs from the current one (a different type, or `!=`). Children and
    /// parent data are kept. Returns whether anything changed (false for an unknown id).
    pub fn set<T: RenderBox + PartialEq>(&mut self, id: LayoutId, render: T) -> bool {
        let _ = (id, render);
        todo!()
    }

    /// Replaces the node's layout object (children and parent data are kept). Marks it as
    /// needing layout. Returns false for an unknown id.
    pub fn replace(&mut self, id: LayoutId, render: impl RenderBox) -> bool {
        let _ = (id, render);
        todo!()
    }

    /// Sets the data the node's parent reads (`None` clears it). Marks the parent as needing
    /// layout. Returns false for an unknown id.
    pub fn set_parent_data<T: Any>(&mut self, id: LayoutId, data: Option<T>) -> bool {
        let _ = (id, data);
        todo!()
    }

    /// The node's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, id: LayoutId) -> Option<&T> {
        let _ = id;
        todo!()
    }

    /// Marks the node as needing layout, and its ancestors up to the nearest relayout boundary.
    pub fn mark_needs_layout(&mut self, id: LayoutId) {
        let _ = id;
        todo!()
    }

    /// True if the node will be laid out by the next pass that reaches it.
    pub fn needs_layout(&self, id: LayoutId) -> bool {
        let _ = id;
        todo!()
    }

    /// True if the node was a relayout boundary in its last layout.
    pub fn is_relayout_boundary(&self, id: LayoutId) -> bool {
        let _ = id;
        todo!()
    }

    /// Runs a layout pass over `root`'s subtree with `constraints` for `root`, and returns
    /// `root`'s size (`Size::ZERO` for an unknown id).
    pub fn layout(&mut self, root: LayoutId, constraints: BoxConstraints) -> Size {
        let _ = (root, constraints);
        todo!()
    }

    /// The node's size from its last layout; `None` if it was never laid out.
    pub fn size(&self, id: LayoutId) -> Option<Size> {
        let _ = id;
        todo!()
    }

    /// The node's offset from its parent's top-left corner (`Vec2::ZERO` until set); `None` for
    /// an unknown id.
    pub fn offset(&self, id: LayoutId) -> Option<Vec2> {
        let _ = id;
        todo!()
    }

    /// The constraints of the node's last layout.
    pub fn constraints(&self, id: LayoutId) -> Option<BoxConstraints> {
        let _ = id;
        todo!()
    }

    /// The node's minimum intrinsic width at `height`, computed on demand and cached until the
    /// node or one of its descendants needs layout. 0 for an unknown id.
    pub fn min_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32 {
        let _ = (id, height);
        todo!()
    }

    /// The node's maximum intrinsic width at `height` (cached like
    /// [`min_intrinsic_width`](Self::min_intrinsic_width)).
    pub fn max_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32 {
        let _ = (id, height);
        todo!()
    }

    /// The node's minimum intrinsic height at `width` (cached).
    pub fn min_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32 {
        let _ = (id, width);
        todo!()
    }

    /// The node's maximum intrinsic height at `width` (cached).
    pub fn max_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32 {
        let _ = (id, width);
        todo!()
    }
}
