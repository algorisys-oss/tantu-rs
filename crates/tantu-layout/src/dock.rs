//! [`RenderDock`]: lays out a dock plan, groups sized as split panes with a tab strip on
//! top. Spec: `docs/specs/widgets/dock-area.md` (rules LAYOUT-DOCK-01..05).

use std::sync::Arc;

use tantu_core::{Rect, Size};

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

impl RenderBox for RenderDock {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!("LAYOUT-DOCK-01..05")
    }
}
