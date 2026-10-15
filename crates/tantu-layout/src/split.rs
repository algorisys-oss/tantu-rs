//! [`RenderSplit`]: panes along one axis with gaps for dividers. Spec:
//! `docs/specs/widgets/split-view.md` (rules LAYOUT-SPLIT-01..05).

use std::sync::Arc;

use tantu_core::Size;

use crate::{Axis, BoxConstraints, LayoutChildren, RenderBox};

/// Lays out `n` panes along `axis`, separated by a fixed `gap`, sized by fractions of the
/// space left after the gaps. Children are the `n` panes followed by the `n - 1` divider
/// handles, each `handle` thick and centred on its gap (so the handles overlap the panes'
/// edges and, coming last, are hit first).
#[derive(Clone, Debug)]
pub struct RenderSplit {
    /// The axis the panes are laid out along.
    pub axis: Axis,
    /// Requested share of the space left after the gaps, per pane (cleaned at layout).
    pub fractions: Arc<[f32]>,
    /// Smallest size of each pane on the axis, in logical pixels (missing entries are 0).
    pub min_sizes: Arc<[f32]>,
    /// Panes that may shrink to 0 when their fraction is 0 (missing entries are false).
    pub collapsible: Arc<[bool]>,
    /// The gap between neighbouring panes.
    pub gap: f32,
    /// The divider handles' thickness along the axis, centred on each gap (at least `gap`).
    pub handle: f32,
    /// The number of panes.
    panes: usize,
    /// The pane sizes from the last layout (not part of equality).
    sizes: Vec<f32>,
}

impl RenderSplit {
    /// `panes` panes along `axis` at equal shares, no minimums, a 1 px gap and 8 px handles.
    pub fn new(axis: Axis, panes: usize) -> Self {
        RenderSplit {
            axis,
            fractions: Arc::from(Vec::new()),
            min_sizes: Arc::from(Vec::new()),
            collapsible: Arc::from(Vec::new()),
            gap: 1.0,
            handle: 8.0,
            panes,
            sizes: Vec::new(),
        }
    }

    /// The number of panes.
    pub fn panes(&self) -> usize {
        self.panes
    }

    /// The pane sizes on the axis from the last layout, in order (empty before the first).
    pub fn sizes(&self) -> &[f32] {
        &self.sizes
    }
}

/// Compares everything but the last layout's sizes.
impl PartialEq for RenderSplit {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.fractions == other.fractions
            && self.min_sizes == other.min_sizes
            && self.collapsible == other.collapsible
            && self.gap == other.gap
            && self.handle == other.handle
            && self.panes == other.panes
    }
}

impl RenderBox for RenderSplit {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!("LAYOUT-SPLIT-01..05")
    }
}
