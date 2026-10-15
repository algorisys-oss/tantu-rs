//! Drag-to-dock: the target and indicator under the pointer. Spec:
//! `docs/specs/widgets/dock-drag.md`.

use tantu_view::core::{Point, Rect};
use tantu_view::layout::{Axis, DockPlan, DockPlanNode};

use super::{DockGroupId, DockLayout, DockSide, DockTarget, PanelId};

/// How far the pointer moves before a press on a tab becomes a drag (WIDGETS-DOCKDRAG-01).
pub(super) const DRAG_SLOP: f32 = 4.0;
/// The dragged tab's opacity.
pub(super) const DRAGGED_OPACITY: f32 = 0.38;
/// A zone indicator's fill opacity.
pub(super) const ZONE_ALPHA: f32 = 0.16;
/// The edge bands of a group's content, as a fraction of its size (WIDGETS-DOCKDRAG-03).
const ZONE_BAND: f32 = 0.25;
/// The dock's edge bands (WIDGETS-DOCKDRAG-04).
const EDGE_BAND: f32 = 16.0;
/// The share an edge target shows (the share `DockLayout` gives it).
const EDGE_SHARE: f32 = 0.25;
/// Half a divider's grab area.
const DIVIDER_HALF: f32 = 4.0;
/// The drop line's width.
const LINE: f32 = 2.0;

/// What the dock draws while dragging (WIDGETS-DOCKDRAG-02..04).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Indicator {
    /// A drop line between tabs.
    Line(Rect),
    /// A zone fill with an outline.
    Zone(Rect),
}

/// A press on a tab, and the drag it may become.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DragState {
    pub(super) panel: PanelId,
    pub(super) target: Option<(DockTarget, Indicator)>,
}

/// A strip's tab places (in its row) and scroll offset.
pub(super) struct StripPlaces {
    pub(super) slots: Vec<(f32, f32)>,
    pub(super) offset: f32,
}

/// What the target search reads: the layout and the last frame's geometry.
pub(super) struct Geometry<'a> {
    pub(super) layout: &'a DockLayout,
    pub(super) plan: &'a DockPlan,
    pub(super) rects: &'a [Rect],
    pub(super) strip_height: f32,
    pub(super) gap: f32,
    pub(super) places: &'a dyn Fn(DockGroupId) -> Option<StripPlaces>,
}

/// `p`'s distance along `axis`.
fn main(p: Point, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => p.x,
        Axis::Vertical => p.y,
    }
}

fn contains(rect: Rect, p: Point) -> bool {
    p.x >= rect.left && p.x < rect.right && p.y >= rect.top && p.y < rect.bottom
}

/// The nearest side among (left, right, top, bottom) distances, ties in that order.
fn nearest(distances: [f32; 4]) -> (DockSide, f32) {
    let sides = [
        DockSide::Left,
        DockSide::Right,
        DockSide::Top,
        DockSide::Bottom,
    ];
    let mut best = (sides[0], distances[0]);
    for (side, d) in sides.into_iter().zip(distances).skip(1) {
        if d < best.1 {
            best = (side, d);
        }
    }
    best
}

/// The part of `rect` on `side`, `share` of it.
fn part(rect: Rect, side: DockSide, share: f32) -> Rect {
    let (w, h) = (rect.width() * share, rect.height() * share);
    match side {
        DockSide::Left => Rect::from_ltrb(rect.left, rect.top, rect.left + w, rect.bottom),
        DockSide::Right => Rect::from_ltrb(rect.right - w, rect.top, rect.right, rect.bottom),
        DockSide::Top => Rect::from_ltrb(rect.left, rect.top, rect.right, rect.top + h),
        DockSide::Bottom => Rect::from_ltrb(rect.left, rect.bottom - h, rect.right, rect.bottom),
    }
}

impl Geometry<'_> {
    /// Whether `p` is over a divider's grab area (WIDGETS-DOCKDRAG-05).
    fn over_divider(&self, p: Point) -> bool {
        self.plan.nodes.iter().enumerate().any(|(node, n)| {
            let DockPlanNode::Split { axis, children, .. } = n else {
                return false;
            };
            let Some(split) = self.rects.get(node) else {
                return false;
            };
            if !contains(*split, p) {
                return false;
            }
            let valid: Vec<usize> = children
                .iter()
                .copied()
                .filter(|c| *c > node && *c < self.rects.len())
                .collect();
            valid.windows(2).any(|pair| {
                let end = match axis {
                    Axis::Horizontal => self.rects[pair[0]].right,
                    Axis::Vertical => self.rects[pair[0]].bottom,
                };
                (main(p, *axis) - (end + self.gap / 2.0)).abs() <= DIVIDER_HALF
            })
        })
    }

    /// The group under `p` and its rect.
    fn group_at(&self, p: Point) -> Option<(DockGroupId, Rect)> {
        self.plan
            .nodes
            .iter()
            .enumerate()
            .find_map(|(node, n)| match n {
                DockPlanNode::Group { key, .. } => self
                    .rects
                    .get(node)
                    .filter(|r| contains(**r, p))
                    .map(|r| (DockGroupId(*key), *r)),
                DockPlanNode::Split { .. } => None,
            })
    }

    /// The insertion point over `group`'s strip (WIDGETS-DOCKDRAG-02).
    fn strip_target(
        &self,
        group: DockGroupId,
        rect: Rect,
        strip: Rect,
        dragged: &PanelId,
        p: Point,
    ) -> Option<(DockTarget, Indicator)> {
        let (panels, _) = self.layout.group(group)?;
        let places = (self.places)(group)?;
        let x = p.x - rect.left + places.offset;
        let others: Vec<(f32, f32)> = panels
            .iter()
            .zip(&places.slots)
            .filter(|(panel, _)| *panel != dragged)
            .map(|(_, slot)| *slot)
            .collect();
        let index = others.iter().filter(|(x0, w)| x0 + w / 2.0 < x).count();
        let boundary = match others.get(index) {
            Some((x0, _)) => *x0,
            None => others.last().map_or(0.0, |(x0, w)| x0 + w),
        };
        let line_x = (rect.left + boundary - places.offset).clamp(rect.left, rect.right);
        let line = Rect::from_ltrb(
            line_x - LINE / 2.0,
            strip.top,
            line_x + LINE / 2.0,
            strip.bottom,
        );
        Some((DockTarget::Tabs { group, index }, Indicator::Line(line)))
    }

    /// The dock edge band under `p` (WIDGETS-DOCKDRAG-04).
    fn edge_target(&self, dock: Rect, p: Point) -> Option<(DockTarget, Indicator)> {
        let (side, distance) = nearest([
            p.x - dock.left,
            dock.right - p.x,
            p.y - dock.top,
            dock.bottom - p.y,
        ]);
        (distance < EDGE_BAND).then(|| {
            (
                DockTarget::Edge(side),
                Indicator::Zone(part(dock, side, EDGE_SHARE)),
            )
        })
    }

    /// The zone of `group`'s content under `p` (WIDGETS-DOCKDRAG-03).
    fn zone_target(&self, group: DockGroupId, content: Rect, p: Point) -> (DockTarget, Indicator) {
        let (w, h) = (content.width().max(1e-3), content.height().max(1e-3));
        let (side, fraction) = nearest([
            (p.x - content.left) / w,
            (content.right - p.x) / w,
            (p.y - content.top) / h,
            (content.bottom - p.y) / h,
        ]);
        if fraction < ZONE_BAND {
            (
                DockTarget::Beside { group, side },
                Indicator::Zone(part(content, side, 0.5)),
            )
        } else {
            (
                DockTarget::Tabs {
                    group,
                    index: usize::MAX,
                },
                Indicator::Zone(content),
            )
        }
    }

    /// The target under `p` for dragging `panel`, if dropping there changes the layout
    /// (WIDGETS-DOCKDRAG-02..05).
    pub(super) fn target(&self, panel: &PanelId, p: Point) -> Option<(DockTarget, Indicator)> {
        let dock = *self.rects.first()?;
        if !contains(dock, p) || self.over_divider(p) {
            return None;
        }
        let found = match self.group_at(p) {
            Some((group, rect)) => {
                let strip_bottom = rect.top + self.strip_height.min(rect.height());
                let strip = Rect::from_ltrb(rect.left, rect.top, rect.right, strip_bottom);
                if contains(strip, p) {
                    self.strip_target(group, rect, strip, panel, p)
                } else {
                    self.edge_target(dock, p).or_else(|| {
                        let content =
                            Rect::from_ltrb(rect.left, strip_bottom, rect.right, rect.bottom);
                        Some(self.zone_target(group, content, p))
                    })
                }
            }
            None => self.edge_target(dock, p),
        }?;
        // Only targets that change the layout (WIDGETS-DOCKDRAG-05): not the tab's own place
        // in its own strip (where a drop would only activate it), not a no-op move.
        if let DockTarget::Tabs { group, index } = found.0 {
            if let Some((panels, _)) = self.layout.group(group) {
                let current = panels.iter().position(|p| p == panel);
                if current.is_some() && current == Some(index.min(panels.len() - 1)) {
                    return None;
                }
            }
        }
        let mut moved = self.layout.clone();
        match moved.move_panel(panel, found.0) {
            Ok(()) if moved != *self.layout => Some(found),
            _ => None,
        }
    }
}
