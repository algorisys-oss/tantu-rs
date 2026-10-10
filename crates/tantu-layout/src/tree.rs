//! [`LayoutTree`]: the layout half of the render tree (ADR 0009). Nodes hold a [`RenderBox`],
//! their children and parent data; the tree runs the layout pass with caching and relayout
//! boundaries.

use std::any::Any;
use std::fmt;

use tantu_core::{Arena, Id, Size, Vec2};

use crate::{BoxConstraints, TextMeasure};

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
        self.0.to_bits()
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
        self.tree.children(self.parent).len()
    }

    /// True with no children.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The child's id; `None` out of range.
    pub fn id(&self, index: usize) -> Option<LayoutId> {
        self.tree.child(self.parent, index)
    }

    /// Lays out child `index` with `constraints` and returns its size; this node's layout
    /// depends on that size. `Size::ZERO` out of range.
    pub fn layout(&mut self, index: usize, constraints: BoxConstraints) -> Size {
        match self.tree.child(self.parent, index) {
            Some(child) => self.tree.layout_node(child, constraints, true, false),
            None => Size::ZERO,
        }
    }

    /// Lays out child `index` when this node doesn't use the child's size (the child becomes a
    /// relayout boundary).
    pub fn layout_ignoring_size(&mut self, index: usize, constraints: BoxConstraints) {
        if let Some(child) = self.tree.child(self.parent, index) {
            self.tree.layout_node(child, constraints, false, false);
        }
    }

    /// Sets child `index`'s offset from this node's top-left corner. Ignored out of range.
    pub fn set_offset(&mut self, index: usize, offset: Vec2) {
        if let Some(child) = self.tree.child(self.parent, index) {
            if let Some(node) = self.tree.nodes.get_mut(child.0) {
                node.offset = offset;
            }
        }
    }

    /// The pass's text measurer (ADR 0010).
    pub fn text(&mut self) -> &mut dyn TextMeasure {
        todo!()
    }

    /// The child's size from its last layout (`Size::ZERO` if it was never laid out or the
    /// index is out of range), so a layout can position children after sizing them all.
    pub fn size(&self, index: usize) -> Size {
        self.tree
            .child(self.parent, index)
            .and_then(|child| self.tree.size(child))
            .unwrap_or(Size::ZERO)
    }

    /// The child's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, index: usize) -> Option<&T> {
        self.tree
            .child(self.parent, index)
            .and_then(|child| self.tree.parent_data(child))
    }

    /// The child's minimum intrinsic width at `height` (see [`LayoutTree::min_intrinsic_width`]).
    pub fn min_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MinWidth, height)
    }

    /// The child's maximum intrinsic width at `height`.
    pub fn max_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MaxWidth, height)
    }

    /// The child's minimum intrinsic height at `width`.
    pub fn min_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MinHeight, width)
    }

    /// The child's maximum intrinsic height at `width`.
    pub fn max_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MaxHeight, width)
    }
}

/// A node's children while it computes an intrinsic size: the same queries, no layout.
pub struct IntrinsicChildren<'a> {
    tree: &'a mut LayoutTree,
    parent: LayoutId,
}

impl IntrinsicChildren<'_> {
    /// The query's text measurer (ADR 0010).
    pub fn text(&mut self) -> &mut dyn TextMeasure {
        todo!()
    }

    /// Number of children.
    pub fn len(&self) -> usize {
        self.tree.children(self.parent).len()
    }

    /// True with no children.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The child's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, index: usize) -> Option<&T> {
        self.tree
            .child(self.parent, index)
            .and_then(|child| self.tree.parent_data(child))
    }

    /// The child's minimum intrinsic width at `height`; 0 out of range.
    pub fn min_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MinWidth, height)
    }

    /// The child's maximum intrinsic width at `height`.
    pub fn max_intrinsic_width(&mut self, index: usize, height: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MaxWidth, height)
    }

    /// The child's minimum intrinsic height at `width`.
    pub fn min_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MinHeight, width)
    }

    /// The child's maximum intrinsic height at `width`.
    pub fn max_intrinsic_height(&mut self, index: usize, width: f32) -> f32 {
        self.tree
            .child_intrinsic(self.parent, index, Intrinsic::MaxHeight, width)
    }
}

/// The four intrinsic-size queries.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Intrinsic {
    MinWidth,
    MaxWidth,
    MinHeight,
    MaxHeight,
}

/// A layout tree borrowed together with a text measurer (ADR 0010), from
/// [`LayoutTree::with_text`].
pub struct LayoutSession<'a> {
    tree: &'a mut LayoutTree,
    text: &'a mut dyn TextMeasure,
}

impl LayoutSession<'_> {
    /// As [`LayoutTree::layout`], with the session's measurer.
    pub fn layout(&mut self, root: LayoutId, constraints: BoxConstraints) -> Size {
        let _ = (root, constraints, &self.tree, &self.text);
        todo!()
    }

    /// As [`LayoutTree::min_intrinsic_width`], with the session's measurer.
    pub fn min_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32 {
        let _ = (id, height);
        todo!()
    }

    /// As [`LayoutTree::max_intrinsic_width`], with the session's measurer.
    pub fn max_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32 {
        let _ = (id, height);
        todo!()
    }

    /// As [`LayoutTree::min_intrinsic_height`], with the session's measurer.
    pub fn min_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32 {
        let _ = (id, width);
        todo!()
    }

    /// As [`LayoutTree::max_intrinsic_height`], with the session's measurer.
    pub fn max_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32 {
        let _ = (id, width);
        todo!()
    }
}

/// One node: its layout object, structure and last layout.
struct Node {
    /// The layout object; `None` only while it is running (taken out of the arena so it can
    /// lay out its children through the tree).
    render: Option<Box<dyn RenderBox>>,
    parent: Option<LayoutId>,
    children: Vec<LayoutId>,
    parent_data: Option<Box<dyn Any>>,
    needs_layout: bool,
    /// In `LayoutTree::dirty`, waiting to be laid out from its own last constraints.
    queued: bool,
    /// Constraints and size of the last layout (both set together).
    constraints: Option<BoxConstraints>,
    size: Option<Size>,
    offset: Vec2,
    /// Whether the last layout made it a relayout boundary, and whether its parent ignored its
    /// size then (so a re-layout from the queue can recompute the same answer).
    boundary: bool,
    ignored_size: bool,
    /// Cached intrinsic sizes: (query, argument bits, value).
    intrinsics: Vec<(Intrinsic, u32, f32)>,
}

/// An arena of layout nodes and the layout pass over them.
#[derive(Default)]
pub struct LayoutTree {
    nodes: Arena<Node>,
    /// Nodes where marking stopped (relayout boundaries and roots); may hold stale entries,
    /// which the pass skips.
    dirty: Vec<LayoutId>,
    /// Scratch for the pass: queued boundaries in the pass's subtree, with their depths.
    scratch: Vec<(u32, LayoutId)>,
}

impl fmt::Debug for LayoutTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayoutTree")
            .field("len", &self.nodes.len())
            .finish_non_exhaustive()
    }
}

impl LayoutTree {
    /// A session that runs layout passes and intrinsic queries with `text` as the text
    /// context. [`LayoutTree::layout`] and the tree's intrinsic methods use
    /// [`NoTextMeasure`](crate::NoTextMeasure).
    pub fn with_text<'a>(&'a mut self, text: &'a mut dyn TextMeasure) -> LayoutSession<'a> {
        let _ = text;
        todo!()
    }

    /// An empty tree.
    pub fn new() -> Self {
        LayoutTree::default()
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True with no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// True if `id` is a node of this tree.
    pub fn contains(&self, id: LayoutId) -> bool {
        self.nodes.contains(id.0)
    }

    /// Adds a node with no parent, no children and no parent data. It needs layout.
    pub fn insert(&mut self, render: impl RenderBox) -> LayoutId {
        LayoutId(self.nodes.insert(Node {
            render: Some(Box::new(render)),
            parent: None,
            children: Vec::new(),
            parent_data: None,
            needs_layout: true,
            queued: false,
            constraints: None,
            size: None,
            offset: Vec2::ZERO,
            boundary: false,
            ignored_size: false,
            intrinsics: Vec::new(),
        }))
    }

    /// Removes `id` and all its descendants; detaches it from its parent (which then needs
    /// layout). Returns the number of nodes removed (0 for an unknown id).
    pub fn remove(&mut self, id: LayoutId) -> usize {
        let Some(node) = self.nodes.get(id.0) else {
            return 0;
        };
        if let Some(parent) = node.parent {
            if let Some(p) = self.nodes.get_mut(parent.0) {
                p.children.retain(|c| *c != id);
            }
            self.mark_needs_layout(parent);
        }
        let mut stack = vec![id];
        let mut removed = 0;
        while let Some(next) = stack.pop() {
            if let Some(node) = self.nodes.remove(next.0) {
                stack.extend(node.children);
                removed += 1;
            }
        }
        removed
    }

    /// Makes `children` the ordered children of `parent`. Each child must have no parent or
    /// already be a child of `parent`; previous children not in the list are detached (they
    /// become roots, not removed). The parent needs layout if the list changed.
    pub fn set_children(
        &mut self,
        parent: LayoutId,
        children: &[LayoutId],
    ) -> Result<(), TreeError> {
        if !self.contains(parent) {
            return Err(TreeError::UnknownNode(parent));
        }
        let ancestors = self.ancestors_sorted(parent);
        for (i, &child) in children.iter().enumerate() {
            let Some(node) = self.nodes.get(child.0) else {
                return Err(TreeError::UnknownNode(child));
            };
            if children[..i].contains(&child) {
                return Err(TreeError::Duplicate(child));
            }
            if ancestors
                .binary_search_by_key(&child.to_bits(), |a| a.to_bits())
                .is_ok()
            {
                return Err(TreeError::Cycle(child));
            }
            if let Some(current) = node.parent {
                if current != parent {
                    return Err(TreeError::HasParent {
                        child,
                        parent: current,
                    });
                }
            }
        }
        let old = match self.nodes.get_mut(parent.0) {
            Some(node) => std::mem::replace(&mut node.children, children.to_vec()),
            None => return Err(TreeError::UnknownNode(parent)),
        };
        for child in &old {
            if let Some(node) = self.nodes.get_mut(child.0) {
                node.parent = None;
            }
        }
        for child in children {
            if let Some(node) = self.nodes.get_mut(child.0) {
                node.parent = Some(parent);
            }
        }
        if old != children {
            self.mark_needs_layout(parent);
        }
        Ok(())
    }

    /// The ordered children (empty for an unknown id).
    pub fn children(&self, id: LayoutId) -> &[LayoutId] {
        self.nodes
            .get(id.0)
            .map_or(&[][..], |node| node.children.as_slice())
    }

    /// The parent, if any.
    pub fn parent(&self, id: LayoutId) -> Option<LayoutId> {
        self.nodes.get(id.0).and_then(|node| node.parent)
    }

    /// The node's layout object, if it is a `T`.
    pub fn get<T: RenderBox>(&self, id: LayoutId) -> Option<&T> {
        let render: &dyn RenderBox = self.nodes.get(id.0)?.render.as_deref()?;
        (render as &dyn Any).downcast_ref::<T>()
    }

    /// The node's layout object for changing it, if it is a `T`. Marks the node as needing
    /// layout (the escape hatch for layouts that can't be compared; prefer
    /// [`set`](Self::set)).
    pub fn get_mut<T: RenderBox>(&mut self, id: LayoutId) -> Option<&mut T> {
        let is_t = self.get::<T>(id).is_some();
        if !is_t {
            return None;
        }
        self.mark_needs_layout(id);
        let render: &mut dyn RenderBox = self.nodes.get_mut(id.0)?.render.as_deref_mut()?;
        (render as &mut dyn Any).downcast_mut::<T>()
    }

    /// Replaces the node's layout object with `render` and marks the node as needing layout
    /// only if it differs from the current one (a different type, or `!=`). Children and
    /// parent data are kept. Returns whether anything changed (false for an unknown id).
    pub fn set<T: RenderBox + PartialEq>(&mut self, id: LayoutId, render: T) -> bool {
        if !self.contains(id) {
            return false;
        }
        if self.get::<T>(id) == Some(&render) {
            return false;
        }
        self.replace(id, render)
    }

    /// Replaces the node's layout object (children and parent data are kept). Marks it as
    /// needing layout. Returns false for an unknown id.
    pub fn replace(&mut self, id: LayoutId, render: impl RenderBox) -> bool {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return false;
        };
        node.render = Some(Box::new(render));
        self.mark_needs_layout(id);
        true
    }

    /// Sets the data the node's parent reads (`None` clears it). Marks the parent as needing
    /// layout. Returns false for an unknown id.
    pub fn set_parent_data<T: Any>(&mut self, id: LayoutId, data: Option<T>) -> bool {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return false;
        };
        node.parent_data = data.map(|d| Box::new(d) as Box<dyn Any>);
        if let Some(parent) = node.parent {
            self.mark_needs_layout(parent);
        }
        true
    }

    /// The node's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, id: LayoutId) -> Option<&T> {
        self.nodes
            .get(id.0)?
            .parent_data
            .as_deref()?
            .downcast_ref::<T>()
    }

    /// Marks the node as needing layout, and its ancestors up to the nearest relayout boundary.
    pub fn mark_needs_layout(&mut self, id: LayoutId) {
        let mut current = id;
        loop {
            let Some(node) = self.nodes.get_mut(current.0) else {
                return;
            };
            let had_intrinsics = !node.intrinsics.is_empty();
            node.intrinsics.clear();
            node.needs_layout = true;
            // A node never laid out counts as not a boundary.
            let boundary = node.boundary && node.size.is_some();
            match node.parent {
                Some(parent) if !boundary || had_intrinsics => current = parent,
                _ => {
                    if !node.queued {
                        node.queued = true;
                        self.dirty.push(current);
                    }
                    return;
                }
            }
        }
    }

    /// True if the node will be laid out by the next pass that reaches it.
    pub fn needs_layout(&self, id: LayoutId) -> bool {
        self.nodes.get(id.0).is_some_and(|node| node.needs_layout)
    }

    /// True if the node was a relayout boundary in its last layout.
    pub fn is_relayout_boundary(&self, id: LayoutId) -> bool {
        self.nodes.get(id.0).is_some_and(|node| node.boundary)
    }

    /// Runs a layout pass over `root`'s subtree with `constraints` for `root`, and returns
    /// `root`'s size (`Size::ZERO` for an unknown id).
    pub fn layout(&mut self, root: LayoutId, constraints: BoxConstraints) -> Size {
        if !self.contains(root) {
            return Size::ZERO;
        }
        let size = self.layout_node(root, constraints, true, true);
        self.flush_boundaries(root);
        size
    }

    /// The node's size from its last layout; `None` if it was never laid out.
    pub fn size(&self, id: LayoutId) -> Option<Size> {
        self.nodes.get(id.0)?.size
    }

    /// The node's offset from its parent's top-left corner (`Vec2::ZERO` until set); `None` for
    /// an unknown id.
    pub fn offset(&self, id: LayoutId) -> Option<Vec2> {
        self.nodes.get(id.0).map(|node| node.offset)
    }

    /// The constraints of the node's last layout.
    pub fn constraints(&self, id: LayoutId) -> Option<BoxConstraints> {
        self.nodes.get(id.0)?.constraints
    }

    /// The node's minimum intrinsic width at `height`, computed on demand and cached until the
    /// node or one of its descendants needs layout. 0 for an unknown id.
    pub fn min_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32 {
        self.intrinsic(id, Intrinsic::MinWidth, height)
    }

    /// The node's maximum intrinsic width at `height` (cached like
    /// [`min_intrinsic_width`](Self::min_intrinsic_width)).
    pub fn max_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32 {
        self.intrinsic(id, Intrinsic::MaxWidth, height)
    }

    /// The node's minimum intrinsic height at `width` (cached).
    pub fn min_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32 {
        self.intrinsic(id, Intrinsic::MinHeight, width)
    }

    /// The node's maximum intrinsic height at `width` (cached).
    pub fn max_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32 {
        self.intrinsic(id, Intrinsic::MaxHeight, width)
    }
}

impl LayoutTree {
    /// The `index`-th child of `parent`.
    fn child(&self, parent: LayoutId, index: usize) -> Option<LayoutId> {
        self.children(parent).get(index).copied()
    }

    /// `id` and its ancestors, sorted by bits (for binary search).
    fn ancestors_sorted(&self, id: LayoutId) -> Vec<LayoutId> {
        let mut out = Vec::new();
        let mut current = Some(id);
        while let Some(c) = current {
            out.push(c);
            current = self.parent(c);
        }
        out.sort_unstable_by_key(|a| a.to_bits());
        out
    }

    /// Lays out one node, skipping it when it is clean and its constraints are unchanged.
    fn layout_node(
        &mut self,
        id: LayoutId,
        constraints: BoxConstraints,
        parent_uses_size: bool,
        is_root: bool,
    ) -> Size {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return Size::ZERO;
        };
        let Some(mut render) = node.render.take() else {
            // Already running: can't happen through the public API.
            return node.size.unwrap_or(Size::ZERO);
        };
        node.boundary =
            is_root || !parent_uses_size || constraints.is_tight() || render.sized_by_parent();
        node.ignored_size = !parent_uses_size;
        if !node.needs_layout && node.constraints == Some(constraints) {
            if let Some(size) = node.size {
                node.render = Some(render);
                return size;
            }
        }

        let raw = render.perform_layout(
            constraints,
            &mut LayoutChildren {
                tree: self,
                parent: id,
            },
        );
        let size = constraints.constrain(raw);
        if cfg!(debug_assertions)
            && constraints.is_normalized()
            && !constraints.is_satisfied_by(raw)
        {
            tracing::warn!(
                layout = std::any::type_name_of_val(&*render),
                ?constraints,
                ?raw,
                "layout returned a size outside its constraints; constrained to {size:?}"
            );
        }
        if let Some(node) = self.nodes.get_mut(id.0) {
            node.render = Some(render);
            node.constraints = Some(constraints);
            node.size = Some(size);
            node.needs_layout = false;
        }
        size
    }

    /// Lays out the queued dirty boundaries inside `root`'s subtree from their last
    /// constraints, top down. Entries outside the subtree stay queued; stale ones are dropped.
    fn flush_boundaries(&mut self, root: LayoutId) {
        if self.dirty.is_empty() {
            return;
        }
        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.clear();
        let mut dirty = std::mem::take(&mut self.dirty);
        dirty.retain(|&id| {
            let Some(node) = self.nodes.get(id.0) else {
                return false;
            };
            match self.depth_below(id, root) {
                Some(depth) => {
                    if node.needs_layout {
                        scratch.push((depth, id));
                    }
                    false
                }
                None => node.needs_layout,
            }
        });
        for &(_, id) in &scratch {
            if let Some(node) = self.nodes.get_mut(id.0) {
                node.queued = false;
            }
        }
        for &id in &dirty {
            if let Some(node) = self.nodes.get_mut(id.0) {
                node.queued = true;
            }
        }
        scratch.sort_unstable_by_key(|&(depth, _)| depth);
        for &(_, id) in &scratch {
            let Some(node) = self.nodes.get(id.0) else {
                continue;
            };
            if !node.needs_layout {
                continue; // laid out by an ancestor's re-layout
            }
            let Some(constraints) = node.constraints else {
                continue; // never laid out: its parent lays it out
            };
            let uses_size = !node.ignored_size;
            let is_root = id == root || node.parent.is_none();
            self.layout_node(id, constraints, uses_size, is_root);
        }
        // Entries queued again during the pass (none expected) are kept.
        dirty.append(&mut self.dirty);
        self.dirty = dirty;
        self.scratch = scratch;
    }

    /// How many levels `id` is below `root`; `None` if it isn't in `root`'s subtree.
    fn depth_below(&self, id: LayoutId, root: LayoutId) -> Option<u32> {
        let mut depth = 0;
        let mut current = id;
        loop {
            if current == root {
                return Some(depth);
            }
            current = self.parent(current)?;
            depth += 1;
        }
    }

    /// The `index`-th child's intrinsic size; 0 out of range.
    fn child_intrinsic(
        &mut self,
        parent: LayoutId,
        index: usize,
        kind: Intrinsic,
        arg: f32,
    ) -> f32 {
        match self.child(parent, index) {
            Some(child) => self.intrinsic(child, kind, arg),
            None => 0.0,
        }
    }

    /// A node's intrinsic size, from the cache or computed (NaN and negative reported as 0).
    fn intrinsic(&mut self, id: LayoutId, kind: Intrinsic, arg: f32) -> f32 {
        let bits = arg.to_bits();
        let Some(node) = self.nodes.get_mut(id.0) else {
            return 0.0;
        };
        if let Some(&(_, _, value)) = node
            .intrinsics
            .iter()
            .find(|(k, b, _)| *k == kind && *b == bits)
        {
            return value;
        }
        let Some(render) = node.render.take() else {
            return 0.0;
        };
        let mut children = IntrinsicChildren {
            tree: self,
            parent: id,
        };
        let value = match kind {
            Intrinsic::MinWidth => render.min_intrinsic_width(arg, &mut children),
            Intrinsic::MaxWidth => render.max_intrinsic_width(arg, &mut children),
            Intrinsic::MinHeight => render.min_intrinsic_height(arg, &mut children),
            Intrinsic::MaxHeight => render.max_intrinsic_height(arg, &mut children),
        };
        let value = if value >= 0.0 { value } else { 0.0 };
        if let Some(node) = self.nodes.get_mut(id.0) {
            node.render = Some(render);
            node.intrinsics.push((kind, bits, value));
        }
        value
    }
}
