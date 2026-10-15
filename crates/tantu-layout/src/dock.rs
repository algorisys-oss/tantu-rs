//! [`RenderDock`]: lays out a dock plan, groups sized as split panes with a tab strip on
//! top. Spec: `docs/specs/widgets/dock-area.md` (rules LAYOUT-DOCK-01..05).

use std::sync::Arc;

use tantu_core::{Rect, Size, Vec2};

use crate::split::{clean_fractions, length, resolve_sizes};
use crate::{Axis, BoxConstraints, LayoutChildren, RenderBox};

/// The geometry of a dock layout: node 0 is the root; groups are leaves. Nodes and panels
/// carry stable keys, so children find their place even after the tree changes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DockPlan {
    /// The nodes; node 0 is the root.
    pub nodes: Vec<DockPlanNode>,
}

/// One node of a [`DockPlan`].
#[derive(Clone, Debug, PartialEq)]
pub enum DockPlanNode {
    /// A tab group and the keys of its panels.
    Group {
        /// The group's key.
        key: u64,
        /// Its panels' keys.
        panels: Vec<u64>,
    },
    /// A split and its children's node indices (each greater than this node's).
    Split {
        /// The split's key.
        key: u64,
        /// The axis its children are laid out along.
        axis: Axis,
        /// The children's fractions.
        fractions: Vec<f32>,
        /// The children's node indices.
        children: Vec<usize>,
    },
}

/// Which part of the plan a child of a [`RenderDock`] is (parent data, set once when the
/// child is built).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockSlot {
    /// The tab strip of the group with this key.
    Strip(u64),
    /// The content of the panel with this key, in whichever group lists it.
    Panel(u64),
    /// The divider between children `index` and `index + 1` of the split with this key.
    Divider {
        /// The split's key.
        split: u64,
        /// The boundary.
        index: usize,
    },
    /// Fills the whole dock (the empty layout's placeholder).
    Placeholder,
}

/// Lays out a dock plan: groups sized as split panes, each with a strip on top.
#[derive(Clone, Debug)]
pub struct RenderDock {
    /// The plan.
    pub plan: Arc<DockPlan>,
    /// The tab strips' height.
    pub strip_height: f32,
    /// Every group's minimum on every split axis.
    pub min_group: f32,
    /// The gap between neighbouring groups.
    pub gap: f32,
    /// The dividers' thickness, centred on the gaps.
    pub handle: f32,
    /// Each node's rect from the last layout (not part of equality).
    rects: Vec<Rect>,
}

impl RenderDock {
    /// A dock for `plan` with a 36 px strip, an 80 px group minimum, a 1 px gap and 8 px
    /// dividers.
    pub fn new(plan: Arc<DockPlan>) -> Self {
        RenderDock {
            plan,
            strip_height: 36.0,
            min_group: 80.0,
            gap: 1.0,
            handle: 8.0,
            rects: Vec::new(),
        }
    }

    /// Each plan node's rect from the last layout, indexed like `plan.nodes`.
    pub fn node_rects(&self) -> &[Rect] {
        &self.rects
    }
}

/// Compares everything but the last layout's rects.
impl PartialEq for RenderDock {
    fn eq(&self, other: &Self) -> bool {
        self.plan == other.plan
            && self.strip_height == other.strip_height
            && self.min_group == other.min_group
            && self.gap == other.gap
            && self.handle == other.handle
    }
}

/// `rect`'s extent and start along `axis`.
fn span(rect: Rect, axis: Axis) -> (f32, f32) {
    match axis {
        Axis::Horizontal => (rect.left, rect.right),
        Axis::Vertical => (rect.top, rect.bottom),
    }
}

/// A rect running from `start` to `end` along `axis`, with `cross`'s extent across it.
fn along(axis: Axis, start: f32, end: f32, cross: Rect) -> Rect {
    match axis {
        Axis::Horizontal => Rect::from_ltrb(start, cross.top, end, cross.bottom),
        Axis::Vertical => Rect::from_ltrb(cross.left, start, cross.right, end),
    }
}

impl RenderDock {
    /// A split node's valid children: later nodes, in range (LAYOUT-DOCK-04).
    fn valid_children(&self, node: usize) -> Vec<usize> {
        match self.plan.nodes.get(node) {
            Some(DockPlanNode::Split { children, .. }) => children
                .iter()
                .copied()
                .filter(|c| *c > node && *c < self.plan.nodes.len())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Fills `rects` for a dock of `size` (LAYOUT-DOCK-01, -05).
    fn place_nodes(&mut self, size: Size) {
        let plan = Arc::clone(&self.plan);
        self.rects.clear();
        self.rects
            .resize(plan.nodes.len(), Rect::from_ltrb(0.0, 0.0, 0.0, 0.0));
        if plan.nodes.is_empty() {
            return;
        }
        self.rects[0] = Rect::from_ltrb(0.0, 0.0, size.width, size.height);
        let gap = length(self.gap);
        let min_group = length(self.min_group);
        let mut sizes = Vec::new();
        for (node, entry) in plan.nodes.iter().enumerate() {
            let DockPlanNode::Split {
                axis, fractions, ..
            } = entry
            else {
                continue;
            };
            let children = self.valid_children(node);
            if children.is_empty() {
                continue;
            }
            let rect = self.rects[node];
            let (start, end) = span(rect, *axis);
            let space = (end - start - gap * (children.len() - 1) as f32).max(0.0);
            clean_fractions(fractions, children.len(), &mut sizes);
            resolve_sizes(&mut sizes, Some(space), |_, _| min_group);
            let mut position = start;
            for (child, size) in children.iter().zip(&sizes) {
                self.rects[*child] = along(*axis, position, position + size, rect);
                position += size + gap;
            }
        }
    }

    /// Where a child with `slot` goes, or `None` (LAYOUT-DOCK-02..04).
    fn slot_rect(&self, slot: Option<DockSlot>, size: Size) -> Option<Rect> {
        let nodes = &self.plan.nodes;
        let group = |key: u64| {
            nodes
                .iter()
                .position(|n| matches!(n, DockPlanNode::Group { key: k, .. } if *k == key))
        };
        let strip = |rect: Rect| rect.top + length(self.strip_height).min(rect.height());
        match slot? {
            DockSlot::Placeholder => Some(Rect::from_ltrb(0.0, 0.0, size.width, size.height)),
            DockSlot::Strip(key) => {
                let rect = self.rects[group(key)?];
                Some(Rect::from_ltrb(
                    rect.left,
                    rect.top,
                    rect.right,
                    strip(rect),
                ))
            }
            DockSlot::Panel(key) => {
                let node = nodes.iter().position(
                    |n| matches!(n, DockPlanNode::Group { panels, .. } if panels.contains(&key)),
                )?;
                let rect = self.rects[node];
                Some(Rect::from_ltrb(
                    rect.left,
                    strip(rect),
                    rect.right,
                    rect.bottom,
                ))
            }
            DockSlot::Divider { split, index } => {
                let node = nodes
                    .iter()
                    .position(|n| matches!(n, DockPlanNode::Split { key, .. } if *key == split))?;
                let DockPlanNode::Split { axis, .. } = &nodes[node] else {
                    return None;
                };
                let children = self.valid_children(node);
                if index + 1 >= children.len() {
                    return None;
                }
                let gap = length(self.gap);
                let handle = length(self.handle).max(gap);
                let (_, end) = span(self.rects[children[index]], *axis);
                let centre = end + gap / 2.0;
                Some(along(
                    *axis,
                    centre - handle / 2.0,
                    centre + handle / 2.0,
                    self.rects[node],
                ))
            }
        }
    }
}

impl RenderBox for RenderDock {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let bounded = |max: f32| if max.is_finite() { max } else { 0.0 };
        if !constraints.max_width.is_finite() || !constraints.max_height.is_finite() {
            tracing::debug!("a dock in an unbounded axis takes no space along it");
        }
        let size = constraints.constrain(Size::new(
            bounded(constraints.max_width),
            bounded(constraints.max_height),
        ));
        self.place_nodes(size);
        for i in 0..children.len() {
            let slot = children.parent_data::<DockSlot>(i).copied();
            match self.slot_rect(slot, size) {
                Some(rect) => {
                    children.layout(i, BoxConstraints::tight(rect.size()));
                    children.set_offset(i, Vec2::new(rect.left, rect.top));
                }
                None => {
                    children.layout(i, BoxConstraints::tight(Size::ZERO));
                    children.set_offset(i, Vec2::ZERO);
                }
            }
        }
        size
    }
}
