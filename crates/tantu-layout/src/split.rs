//! [`RenderSplit`]: panes along one axis with gaps for dividers. Spec:
//! `docs/specs/widgets/split-view.md` (rules LAYOUT-SPLIT-01..05).

use std::sync::Arc;

use tantu_core::{Size, Vec2};

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

/// A value as a length: negative and non-finite values count as 0.
fn length(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

/// A size with `main` along `axis` and `cross` across it.
fn size_on(axis: Axis, main: f32, cross: f32) -> Size {
    match axis {
        Axis::Horizontal => Size::new(main, cross),
        Axis::Vertical => Size::new(cross, main),
    }
}

/// An offset of `main` along `axis`.
fn offset_on(axis: Axis, main: f32) -> Vec2 {
    match axis {
        Axis::Horizontal => Vec2::new(main, 0.0),
        Axis::Vertical => Vec2::new(0.0, main),
    }
}

impl RenderSplit {
    /// The smallest size of pane `i` (LAYOUT-SPLIT-02).
    fn min_size(&self, i: usize) -> f32 {
        length(self.min_sizes.get(i).copied().unwrap_or(0.0))
    }

    /// Fills `sizes` with the cleaned, normalized fractions (LAYOUT-SPLIT-01).
    fn clean_fractions(&mut self) {
        let n = self.panes;
        self.sizes.clear();
        for i in 0..n {
            let fraction = self.fractions.get(i).map_or(1.0 / n as f32, |f| length(*f));
            self.sizes.push(fraction);
        }
        let sum: f32 = self.sizes.iter().sum();
        for fraction in &mut self.sizes {
            *fraction = if sum > 0.0 {
                *fraction / sum
            } else {
                1.0 / n as f32
            };
        }
    }

    /// Whether pane `i` is collapsed: collapsible with a cleaned fraction of 0.
    fn collapsed(&self, i: usize, fractions: &[f32]) -> bool {
        self.collapsible.get(i) == Some(&true) && fractions[i] == 0.0
    }

    /// Turns the cleaned fractions in `sizes` into pane sizes for `space` (LAYOUT-SPLIT-02),
    /// or into the minimums when `space` is unbounded (LAYOUT-SPLIT-05). Collapsed panes
    /// keep a fraction of 0 throughout, so they can be told apart while sizes are written.
    fn resolve(&mut self, space: Option<f32>) {
        let mut sizes = std::mem::take(&mut self.sizes);
        let n = sizes.len();
        // The effective minimum: 0 for a collapsed pane.
        let min = |this: &Self, sizes: &[f32], i: usize| {
            if this.collapsed(i, sizes) {
                0.0
            } else {
                this.min_size(i)
            }
        };
        match space {
            None => {
                for i in 0..n {
                    sizes[i] = min(self, &sizes, i);
                }
            }
            Some(space) => {
                let total_min: f32 = (0..n).map(|i| min(self, &sizes, i)).sum();
                if total_min > space {
                    // The minimums don't fit: scale them down together.
                    let scale = space / total_min;
                    for i in 0..n {
                        sizes[i] = min(self, &sizes, i) * scale;
                    }
                } else {
                    let (mut deficit, mut excess) = (0.0f32, 0.0f32);
                    for i in 0..n {
                        let (share, m) = (sizes[i] * space, min(self, &sizes, i));
                        deficit += (m - share).max(0.0);
                        excess += (share - m).max(0.0);
                    }
                    for i in 0..n {
                        let (share, m) = (sizes[i] * space, min(self, &sizes, i));
                        sizes[i] = if share < m {
                            m
                        } else if excess > 0.0 {
                            share - deficit * (share - m) / excess
                        } else {
                            share
                        };
                    }
                }
            }
        }
        self.sizes = sizes;
    }
}

impl RenderBox for RenderSplit {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let n = self.panes;
        let count = children.len();
        if n == 0 {
            self.sizes.clear();
            for i in 0..count {
                children.layout(i, BoxConstraints::tight(Size::ZERO));
            }
            return constraints.smallest();
        }
        let axis = self.axis;
        let (min_main, max_main, min_cross, max_cross) = match axis {
            Axis::Horizontal => (
                constraints.min_width,
                constraints.max_width,
                constraints.min_height,
                constraints.max_height,
            ),
            Axis::Vertical => (
                constraints.min_height,
                constraints.max_height,
                constraints.min_width,
                constraints.max_width,
            ),
        };
        let gap = length(self.gap);
        let handle = length(self.handle).max(gap);
        let gaps = gap * (n - 1) as f32;
        self.clean_fractions();
        let main = if max_main.is_finite() {
            self.resolve(Some((max_main - gaps).max(0.0)));
            max_main
        } else {
            tracing::debug!("a split view in an unbounded axis shows its panes at their minimums");
            self.resolve(None);
            (self.sizes.iter().sum::<f32>() + gaps).max(min_main)
        };
        let panes = n.min(count);
        let cross = if max_cross.is_finite() {
            max_cross
        } else {
            // Loose on the cross axis first; the split takes the largest pane.
            let mut largest = 0.0f32;
            for i in 0..panes {
                let loose = match axis {
                    Axis::Horizontal => {
                        BoxConstraints::new(self.sizes[i], self.sizes[i], 0.0, f32::INFINITY)
                    }
                    Axis::Vertical => {
                        BoxConstraints::new(0.0, f32::INFINITY, self.sizes[i], self.sizes[i])
                    }
                };
                let size = children.layout(i, loose);
                largest = largest.max(match axis {
                    Axis::Horizontal => size.height,
                    Axis::Vertical => size.width,
                });
            }
            largest.max(min_cross)
        };
        let mut position = 0.0f32;
        for i in 0..panes {
            let size = self.sizes[i];
            children.layout(i, BoxConstraints::tight(size_on(axis, size, cross)));
            children.set_offset(i, offset_on(axis, position));
            position += size;
            // The handle centred on the gap after this pane.
            let index = n + i;
            if i + 1 < n && index < count {
                children.layout(index, BoxConstraints::tight(size_on(axis, handle, cross)));
                children.set_offset(index, offset_on(axis, position + gap / 2.0 - handle / 2.0));
            }
            position += gap;
        }
        for i in (2 * n - 1).min(count)..count {
            children.layout(i, BoxConstraints::tight(Size::ZERO));
        }
        constraints.constrain(size_on(axis, main, cross))
    }
}
