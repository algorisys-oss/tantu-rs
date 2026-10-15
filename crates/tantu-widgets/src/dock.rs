//! The dock layout model: [`DockLayout`], a normalized tree of splits and tab groups of
//! app-named panels, with operations and a text format. Spec:
//! `docs/specs/widgets/dock-layout.md`.

mod area;
mod drag;

use std::collections::HashSet;
use std::sync::Arc;

use tantu_view::layout::Axis;

pub use area::DockArea;

/// A panel's identity, chosen by the app (`"explorer"`, `"editor:src/main.rs"`). Stable
/// across runs: it is what the text format stores.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PanelId(Arc<str>);

impl PanelId {
    /// A panel id.
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        PanelId(id.into())
    }

    /// The id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PanelId {
    fn from(id: &str) -> Self {
        PanelId::new(id)
    }
}

impl From<String> for PanelId {
    fn from(id: String) -> Self {
        PanelId::new(id)
    }
}

/// A tab group's identity within one layout. Not persisted; layout equality ignores it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DockGroupId(pub(crate) u64);

/// A split's identity within one layout. Not persisted; layout equality ignores it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DockSplitId(pub(crate) u64);

/// A description of a layout, for building one in code.
#[derive(Clone, Debug, PartialEq)]
pub enum DockNode {
    /// A tab group: its panels in order and the index of the active one.
    Tabs {
        /// The panels, in tab order.
        panels: Vec<PanelId>,
        /// The active tab's index.
        active: usize,
    },
    /// Children side by side (`Axis::Horizontal`) or stacked, with their fractions.
    Split {
        /// The axis the children are laid out along.
        axis: Axis,
        /// Each child with its fraction.
        children: Vec<(f32, DockNode)>,
    },
}

impl DockNode {
    /// A group of `panels`, the first active.
    pub fn tabs<P: Into<PanelId>>(panels: impl IntoIterator<Item = P>) -> Self {
        DockNode::Tabs {
            panels: panels.into_iter().map(Into::into).collect(),
            active: 0,
        }
    }

    /// Children side by side, each with its fraction.
    pub fn row(children: impl IntoIterator<Item = (f32, DockNode)>) -> Self {
        DockNode::Split {
            axis: Axis::Horizontal,
            children: children.into_iter().collect(),
        }
    }

    /// Children stacked top to bottom, each with its fraction.
    pub fn column(children: impl IntoIterator<Item = (f32, DockNode)>) -> Self {
        DockNode::Split {
            axis: Axis::Vertical,
            children: children.into_iter().collect(),
        }
    }
}

/// An edge of a group or of the whole layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockSide {
    /// The left edge.
    Left,
    /// The right edge.
    Right,
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
}

/// Where a panel goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockTarget {
    /// Into a group's tabs at `index` (clamped to the end).
    Tabs {
        /// The group.
        group: DockGroupId,
        /// The tab position, counting the group's tabs without the moving panel.
        index: usize,
    },
    /// A new group beside `group`, on `side`.
    Beside {
        /// The group.
        group: DockGroupId,
        /// Which side of it.
        side: DockSide,
    },
    /// A new group along one edge of the whole layout.
    Edge(DockSide),
}

/// Why an operation changed nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DockError {
    /// The panel isn't in the layout.
    UnknownPanel(PanelId),
    /// No group has this id.
    UnknownGroup(DockGroupId),
    /// No split has this id.
    UnknownSplit(DockSplitId),
    /// Moving a group's only panel beside its own group.
    NoOp,
}

/// Why a saved layout couldn't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DockLoadError {
    /// The first line isn't `tantu-dock 1`.
    Version,
    /// Malformed text at this byte offset.
    Syntax {
        /// The byte offset of the first unexpected character.
        offset: usize,
        /// What was expected there.
        expected: &'static str,
    },
}

/// One node of the normalized tree, for reading.
#[derive(Clone, Debug, PartialEq)]
pub enum DockView<'a> {
    /// A tab group.
    Tabs {
        /// Its id.
        id: DockGroupId,
        /// Its panels, in tab order.
        panels: &'a [PanelId],
        /// The active tab's index.
        active: usize,
    },
    /// A split.
    Split {
        /// Its id.
        id: DockSplitId,
        /// The axis its children are laid out along.
        axis: Axis,
        /// Its children's fractions (summing to 1).
        fractions: &'a [f32],
        /// Its children.
        children: Vec<DockView<'a>>,
    },
}

/// The text format's first line (WIDGETS-DOCK-11).
const HEADER: &str = "tantu-dock 1";
/// How deeply `load` nests before giving up (WIDGETS-DOCK-12).
const MAX_DEPTH: usize = 256;
/// Fractions closer than this compare equal (WIDGETS-DOCK-03).
const FRACTION_TOLERANCE: f32 = 1e-5;
/// The share an edge target takes (WIDGETS-DOCK-09).
const EDGE_SHARE: f32 = 0.25;

/// A node of the normalized tree.
#[derive(Clone, Debug)]
enum Node {
    Tabs {
        id: DockGroupId,
        panels: Vec<PanelId>,
        active: usize,
    },
    Split {
        id: DockSplitId,
        axis: Axis,
        fractions: Vec<f32>,
        children: Vec<Node>,
    },
}

impl Node {
    /// Panels in tree order.
    fn collect_panels(&self, out: &mut Vec<PanelId>) {
        match self {
            Node::Tabs { panels, .. } => out.extend(panels.iter().cloned()),
            Node::Split { children, .. } => {
                for child in children {
                    child.collect_panels(out);
                }
            }
        }
    }

    /// Group ids in tree order.
    fn collect_groups(&self, out: &mut Vec<DockGroupId>) {
        match self {
            Node::Tabs { id, .. } => out.push(*id),
            Node::Split { children, .. } => {
                for child in children {
                    child.collect_groups(out);
                }
            }
        }
    }

    fn find_group(&self, group: DockGroupId) -> Option<&Node> {
        match self {
            Node::Tabs { id, .. } => (*id == group).then_some(self),
            Node::Split { children, .. } => children.iter().find_map(|c| c.find_group(group)),
        }
    }

    fn find_group_mut(&mut self, group: DockGroupId) -> Option<&mut Node> {
        match self {
            Node::Tabs { id, .. } => {
                if *id == group {
                    Some(self)
                } else {
                    None
                }
            }
            Node::Split { children, .. } => {
                children.iter_mut().find_map(|c| c.find_group_mut(group))
            }
        }
    }

    /// The group holding `panel`.
    fn group_of(&self, panel: &PanelId) -> Option<DockGroupId> {
        match self {
            Node::Tabs { id, panels, .. } => panels.contains(panel).then_some(*id),
            Node::Split { children, .. } => children.iter().find_map(|c| c.group_of(panel)),
        }
    }

    fn find_split_mut(&mut self, split: DockSplitId) -> Option<&mut Node> {
        if matches!(self, Node::Split { id, .. } if *id == split) {
            return Some(self);
        }
        match self {
            Node::Split { children, .. } => {
                children.iter_mut().find_map(|c| c.find_split_mut(split))
            }
            Node::Tabs { .. } => None,
        }
    }

    fn view(&self) -> DockView<'_> {
        match self {
            Node::Tabs { id, panels, active } => DockView::Tabs {
                id: *id,
                panels,
                active: *active,
            },
            Node::Split {
                id,
                axis,
                fractions,
                children,
            } => DockView::Split {
                id: *id,
                axis: *axis,
                fractions,
                children: children.iter().map(Node::view).collect(),
            },
        }
    }

    /// Structural equality, ignoring ids (WIDGETS-DOCK-03).
    fn same(&self, other: &Node) -> bool {
        match (self, other) {
            (
                Node::Tabs {
                    panels: a,
                    active: x,
                    ..
                },
                Node::Tabs {
                    panels: b,
                    active: y,
                    ..
                },
            ) => a == b && x == y,
            (
                Node::Split {
                    axis: a,
                    fractions: f,
                    children: c,
                    ..
                },
                Node::Split {
                    axis: b,
                    fractions: g,
                    children: d,
                    ..
                },
            ) => {
                a == b
                    && f.len() == g.len()
                    && f.iter()
                        .zip(g)
                        .all(|(x, y)| (x - y).abs() <= FRACTION_TOLERANCE)
                    && c.len() == d.len()
                    && c.iter().zip(d).all(|(x, y)| x.same(y))
            }
            _ => false,
        }
    }

    fn save(&self, out: &mut String) {
        match self {
            Node::Tabs { panels, active, .. } => {
                out.push_str(&format!("tabs({active}"));
                for panel in panels {
                    out.push_str(" \"");
                    for c in panel.as_str().chars() {
                        if c == '"' || c == '\\' {
                            out.push('\\');
                        }
                        out.push(c);
                    }
                    out.push('"');
                }
                out.push(')');
            }
            Node::Split {
                axis,
                fractions,
                children,
                ..
            } => {
                out.push_str(match axis {
                    Axis::Horizontal => "row(",
                    Axis::Vertical => "column(",
                });
                for (i, (fraction, child)) in fractions.iter().zip(children).enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    out.push_str(&format!("{fraction} "));
                    child.save(out);
                }
                out.push(')');
            }
        }
    }
}

/// `fractions` cleaned for `n` children as LAYOUT-SPLIT-01: a missing entry counts as an
/// equal share, a negative or non-finite one as 0, normalized to sum to 1 (all zero: equal).
fn clean(fractions: &[f32], n: usize) -> Vec<f32> {
    let share = 1.0 / n.max(1) as f32;
    let mut out: Vec<f32> = (0..n)
        .map(|i| match fractions.get(i) {
            None => share,
            Some(f) if f.is_finite() && *f > 0.0 => *f,
            Some(_) => 0.0,
        })
        .collect();
    let sum: f32 = out.iter().sum();
    for f in &mut out {
        *f = if sum > 0.0 { *f / sum } else { share };
    }
    out
}

/// The axis a side lies along, and whether it comes first on that axis.
fn side_axis(side: DockSide) -> (Axis, bool) {
    match side {
        DockSide::Left => (Axis::Horizontal, true),
        DockSide::Right => (Axis::Horizontal, false),
        DockSide::Top => (Axis::Vertical, true),
        DockSide::Bottom => (Axis::Vertical, false),
    }
}

/// Puts `node` into normal form (WIDGETS-DOCK-02); `None` when nothing is left.
fn normalize(node: Node) -> Option<Node> {
    match node {
        Node::Tabs { id, panels, active } => {
            if panels.is_empty() {
                return None;
            }
            let active = active.min(panels.len() - 1);
            Some(Node::Tabs { id, panels, active })
        }
        Node::Split {
            id,
            axis,
            fractions,
            children,
        } => {
            let n = children.len();
            let fractions = clean(&fractions, n);
            let kept: Vec<(f32, Node)> = fractions
                .into_iter()
                .zip(children)
                .filter_map(|(f, child)| normalize(child).map(|c| (f, c)))
                .collect();
            match kept.len() {
                0 => None,
                1 => kept.into_iter().next().map(|(_, c)| c),
                _ => {
                    let raw: Vec<f32> = kept.iter().map(|(f, _)| *f).collect();
                    let shares = clean(&raw, raw.len());
                    let (mut fractions, mut children) = (Vec::new(), Vec::new());
                    for (share, (_, child)) in shares.into_iter().zip(kept) {
                        match child {
                            // A same-axis child is flattened into this split.
                            Node::Split {
                                axis: inner,
                                fractions: inner_fractions,
                                children: inner_children,
                                ..
                            } if inner == axis => {
                                for (f, c) in inner_fractions.into_iter().zip(inner_children) {
                                    fractions.push(share * f);
                                    children.push(c);
                                }
                            }
                            other => {
                                fractions.push(share);
                                children.push(other);
                            }
                        }
                    }
                    let fractions = clean(&fractions, children.len());
                    Some(Node::Split {
                        id,
                        axis,
                        fractions,
                        children,
                    })
                }
            }
        }
    }
}

/// The arrangement of docked panels: a normalized tree of splits and tab groups of
/// app-named panels, possibly empty (WIDGETS-DOCK-01..12).
///
/// ```
/// use tantu_widgets::{DockLayout, DockNode, DockSide, DockTarget, PanelId};
///
/// let mut layout = DockLayout::new(DockNode::row([
///     (0.25, DockNode::tabs(["explorer"])),
///     (0.75, DockNode::tabs(["editor"])),
/// ]));
/// layout.add("terminal", DockTarget::Edge(DockSide::Bottom)).unwrap();
/// let saved = layout.save();
/// assert_eq!(DockLayout::load(&saved, |_| true), Ok(layout));
/// ```
#[derive(Clone, Debug, Default)]
pub struct DockLayout {
    root: Option<Node>,
    /// The next id to hand out (WIDGETS-DOCK-03).
    next: u64,
}

impl DockLayout {
    /// The empty layout.
    pub fn empty() -> Self {
        DockLayout::default()
    }

    /// A layout from a description, normalized (WIDGETS-DOCK-01).
    pub fn new(root: DockNode) -> Self {
        let mut layout = DockLayout::empty();
        let mut seen = HashSet::new();
        let node = layout.build(root, &mut seen);
        layout.root = normalize(node);
        layout
    }

    fn fresh_group(&mut self) -> DockGroupId {
        self.next += 1;
        DockGroupId(self.next)
    }

    fn fresh_split(&mut self) -> DockSplitId {
        self.next += 1;
        DockSplitId(self.next)
    }

    /// A description as a node with fresh ids, dropping repeated panels.
    fn build(&mut self, node: DockNode, seen: &mut HashSet<PanelId>) -> Node {
        match node {
            DockNode::Tabs { panels, active } => {
                let panels = panels
                    .into_iter()
                    .filter(|panel| seen.insert(panel.clone()))
                    .collect();
                Node::Tabs {
                    id: self.fresh_group(),
                    panels,
                    active,
                }
            }
            DockNode::Split { axis, children } => {
                let id = self.fresh_split();
                let (fractions, children) = children
                    .into_iter()
                    .map(|(f, child)| (f, self.build(child, seen)))
                    .unzip();
                Node::Split {
                    id,
                    axis,
                    fractions,
                    children,
                }
            }
        }
    }

    /// Whether the layout has no panels.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// The tree, for reading; `None` when empty.
    pub fn root(&self) -> Option<DockView<'_>> {
        self.root.as_ref().map(Node::view)
    }

    /// Every panel, in tree order.
    pub fn panels(&self) -> Vec<PanelId> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.collect_panels(&mut out);
        }
        out
    }

    /// Whether `panel` is in the layout.
    pub fn contains(&self, panel: &PanelId) -> bool {
        self.group_of(panel).is_some()
    }

    /// The group holding `panel`.
    pub fn group_of(&self, panel: &PanelId) -> Option<DockGroupId> {
        self.root.as_ref()?.group_of(panel)
    }

    /// A group's panels and active index.
    pub fn group(&self, group: DockGroupId) -> Option<(&[PanelId], usize)> {
        match self.root.as_ref()?.find_group(group)? {
            Node::Tabs { panels, active, .. } => Some((panels, *active)),
            Node::Split { .. } => None,
        }
    }

    /// A group's active panel.
    pub fn active(&self, group: DockGroupId) -> Option<&PanelId> {
        let (panels, active) = self.group(group)?;
        panels.get(active)
    }

    /// The groups in tree order.
    pub fn groups(&self) -> Vec<DockGroupId> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.collect_groups(&mut out);
        }
        out
    }

    /// Makes `panel` its group's active tab (WIDGETS-DOCK-05).
    pub fn activate(&mut self, panel: &PanelId) -> Result<(), DockError> {
        let group = self
            .group_of(panel)
            .ok_or_else(|| DockError::UnknownPanel(panel.clone()))?;
        if let Some(Node::Tabs { panels, active, .. }) =
            self.root.as_mut().and_then(|r| r.find_group_mut(group))
        {
            if let Some(index) = panels.iter().position(|p| p == panel) {
                *active = index;
            }
        }
        Ok(())
    }

    /// Takes `panel` out of its group, keeping the active tab per WIDGETS-DOCK-06, then
    /// normalizes. False when it isn't in the layout.
    fn remove(&mut self, panel: &PanelId) -> bool {
        let Some(group) = self.group_of(panel) else {
            return false;
        };
        if let Some(Node::Tabs { panels, active, .. }) =
            self.root.as_mut().and_then(|r| r.find_group_mut(group))
        {
            if let Some(index) = panels.iter().position(|p| p == panel) {
                panels.remove(index);
                if index < *active {
                    *active -= 1;
                } else if index == *active && index >= panels.len() {
                    *active = index.saturating_sub(1);
                }
            }
        }
        self.root = self.root.take().and_then(normalize);
        true
    }

    /// Removes `panel` from the layout (WIDGETS-DOCK-06).
    pub fn close(&mut self, panel: &PanelId) -> Result<(), DockError> {
        if self.remove(panel) {
            Ok(())
        } else {
            Err(DockError::UnknownPanel(panel.clone()))
        }
    }

    /// Puts a new panel at `target`; a panel already in the layout is moved instead
    /// (WIDGETS-DOCK-07..09).
    pub fn add(&mut self, panel: impl Into<PanelId>, target: DockTarget) -> Result<(), DockError> {
        let panel = panel.into();
        if self.contains(&panel) {
            return self.move_panel(&panel, target);
        }
        self.check_target(target)?;
        let mut work = self.clone();
        work.insert(panel, target)?;
        *self = work;
        Ok(())
    }

    /// Moves a panel that is in the layout to `target` (WIDGETS-DOCK-07..09).
    pub fn move_panel(&mut self, panel: &PanelId, target: DockTarget) -> Result<(), DockError> {
        let source = self
            .group_of(panel)
            .ok_or_else(|| DockError::UnknownPanel(panel.clone()))?;
        self.check_target(target)?;
        let mut work = self.clone();
        work.remove(panel);
        let target_group = match target {
            DockTarget::Tabs { group, .. } | DockTarget::Beside { group, .. } => Some(group),
            DockTarget::Edge(_) => None,
        };
        if let Some(group) = target_group {
            if work.group(group).is_none() {
                // The target was the panel's own single-panel group.
                debug_assert_eq!(group, source);
                return match target {
                    // Reordering a group's only tab changes nothing.
                    DockTarget::Tabs { .. } => Ok(()),
                    _ => Err(DockError::NoOp),
                };
            }
        }
        work.insert(panel.clone(), target)?;
        *self = work;
        Ok(())
    }

    /// `UnknownGroup` when `target` names a group that isn't in the layout.
    fn check_target(&self, target: DockTarget) -> Result<(), DockError> {
        match target {
            DockTarget::Tabs { group, .. } | DockTarget::Beside { group, .. }
                if self.group(group).is_none() =>
            {
                Err(DockError::UnknownGroup(group))
            }
            _ => Ok(()),
        }
    }

    /// Inserts a panel that isn't in the layout at `target`, then normalizes.
    fn insert(&mut self, panel: PanelId, target: DockTarget) -> Result<(), DockError> {
        match target {
            DockTarget::Tabs { group, index } => {
                let Some(Node::Tabs { panels, active, .. }) =
                    self.root.as_mut().and_then(|r| r.find_group_mut(group))
                else {
                    return Err(DockError::UnknownGroup(group));
                };
                let index = index.min(panels.len());
                panels.insert(index, panel);
                *active = index;
            }
            DockTarget::Beside { group, side } => {
                let new = Node::Tabs {
                    id: self.fresh_group(),
                    panels: vec![panel],
                    active: 0,
                };
                let split = self.fresh_split();
                let Some(root) = self.root.as_mut() else {
                    return Err(DockError::UnknownGroup(group));
                };
                if !insert_beside(root, group, side, new, split) {
                    return Err(DockError::UnknownGroup(group));
                }
            }
            DockTarget::Edge(side) => {
                let new = Node::Tabs {
                    id: self.fresh_group(),
                    panels: vec![panel],
                    active: 0,
                };
                let (axis, first) = side_axis(side);
                self.root = Some(match self.root.take() {
                    None => new,
                    Some(Node::Split {
                        id,
                        axis: root_axis,
                        mut fractions,
                        mut children,
                    }) if root_axis == axis => {
                        for f in &mut fractions {
                            *f *= 1.0 - EDGE_SHARE;
                        }
                        let at = if first { 0 } else { children.len() };
                        fractions.insert(at, EDGE_SHARE);
                        children.insert(at, new);
                        Node::Split {
                            id,
                            axis,
                            fractions,
                            children,
                        }
                    }
                    Some(root) => {
                        let id = self.fresh_split();
                        let (fractions, children) = if first {
                            (vec![EDGE_SHARE, 1.0 - EDGE_SHARE], vec![new, root])
                        } else {
                            (vec![1.0 - EDGE_SHARE, EDGE_SHARE], vec![root, new])
                        };
                        Node::Split {
                            id,
                            axis,
                            fractions,
                            children,
                        }
                    }
                });
            }
        }
        self.root = self.root.take().and_then(normalize);
        Ok(())
    }

    /// Sets a split's fractions, cleaned as LAYOUT-SPLIT-01 (WIDGETS-DOCK-10).
    pub fn set_fractions(
        &mut self,
        split: DockSplitId,
        fractions: &[f32],
    ) -> Result<(), DockError> {
        match self.root.as_mut().and_then(|r| r.find_split_mut(split)) {
            Some(Node::Split {
                fractions: stored,
                children,
                ..
            }) => {
                *stored = clean(fractions, children.len());
                Ok(())
            }
            _ => Err(DockError::UnknownSplit(split)),
        }
    }

    /// The layout as text (WIDGETS-DOCK-11).
    pub fn save(&self) -> String {
        let mut out = format!("{HEADER}\n");
        match &self.root {
            None => out.push_str("empty"),
            Some(root) => root.save(&mut out),
        }
        out.push('\n');
        out
    }

    /// Reads a saved layout, keeping only the panels `keep` accepts, then normalizes
    /// (WIDGETS-DOCK-12).
    pub fn load(text: &str, keep: impl Fn(&PanelId) -> bool) -> Result<DockLayout, DockLoadError> {
        let (first, body) = match text.find('\n') {
            Some(end) => (&text[..end], end + 1),
            None => (text, text.len()),
        };
        if first.trim_end() != HEADER {
            return Err(DockLoadError::Version);
        }
        let mut parser = Parser {
            text,
            pos: body,
            keep: &keep,
        };
        parser.skip_space();
        let root = if parser.rest().starts_with("empty") {
            parser.pos += "empty".len();
            None
        } else {
            Some(parser.node(0)?)
        };
        parser.skip_space();
        if parser.pos < text.len() {
            return Err(parser.error("end of input"));
        }
        Ok(root.map_or_else(DockLayout::empty, DockLayout::new))
    }
}

/// Puts `new` beside group `group` inside `node` (WIDGETS-DOCK-08); `split` is the id for a
/// split it may need. False when the group isn't there.
fn insert_beside(
    node: &mut Node,
    group: DockGroupId,
    side: DockSide,
    new: Node,
    split: DockSplitId,
) -> bool {
    let (axis, first) = side_axis(side);
    if let Node::Tabs { id, .. } = node {
        if *id != group {
            return false;
        }
        // The group is the root: it becomes a split of the two.
        let old = std::mem::replace(
            node,
            Node::Split {
                id: split,
                axis,
                fractions: vec![0.5, 0.5],
                children: Vec::new(),
            },
        );
        if let Node::Split { children, .. } = node {
            *children = if first {
                vec![new, old]
            } else {
                vec![old, new]
            };
        }
        return true;
    }
    let Node::Split {
        axis: split_axis,
        fractions,
        children,
        ..
    } = node
    else {
        return false;
    };
    let index = children
        .iter()
        .position(|c| matches!(c, Node::Tabs { id, .. } if *id == group));
    match index {
        Some(i) if *split_axis == axis => {
            let half = fractions[i] / 2.0;
            fractions[i] = half;
            let at = if first { i } else { i + 1 };
            fractions.insert(at, half);
            children.insert(at, new);
            true
        }
        Some(i) => insert_beside(&mut children[i], group, side, new, split),
        None => {
            let mut new = Some(new);
            for child in children.iter_mut() {
                if child.find_group(group).is_some() {
                    return new
                        .take()
                        .is_some_and(|n| insert_beside(child, group, side, n, split));
                }
            }
            false
        }
    }
}

/// Reads the text format (WIDGETS-DOCK-12).
struct Parser<'t, 'k> {
    text: &'t str,
    pos: usize,
    keep: &'k dyn Fn(&PanelId) -> bool,
}

impl Parser<'_, '_> {
    fn rest(&self) -> &str {
        &self.text[self.pos..]
    }

    fn error(&self, expected: &'static str) -> DockLoadError {
        DockLoadError::Syntax {
            offset: self.pos,
            expected,
        }
    }

    fn skip_space(&mut self) {
        let rest = self.rest();
        self.pos += rest.len() - rest.trim_start().len();
    }

    /// Consumes `prefix` if the text continues with it.
    fn eat(&mut self, prefix: &str) -> bool {
        if self.rest().starts_with(prefix) {
            self.pos += prefix.len();
            true
        } else {
            false
        }
    }

    /// A run of characters up to whitespace, a parenthesis or a quote.
    fn token(&mut self) -> &str {
        let rest = &self.text[self.pos..];
        let len = rest
            .find(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '"')
            .unwrap_or(rest.len());
        self.pos += len;
        &rest[..len]
    }

    fn node(&mut self, depth: usize) -> Result<DockNode, DockLoadError> {
        if depth >= MAX_DEPTH {
            return Err(self.error("node"));
        }
        if self.eat("tabs(") {
            self.skip_space();
            let start = self.pos;
            let digits = self.token();
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                self.pos = start;
                return Err(self.error("node"));
            }
            let active = digits.bytes().fold(0usize, |n, b| {
                n.saturating_mul(10).saturating_add(usize::from(b - b'0'))
            });
            let mut panels = Vec::new();
            loop {
                self.skip_space();
                if self.eat(")") {
                    break;
                }
                if self.pos >= self.text.len() {
                    return Err(self.error(")"));
                }
                if !self.eat("\"") {
                    return Err(self.error("panel"));
                }
                let panel = PanelId::new(self.string()?);
                if (self.keep)(&panel) {
                    panels.push(panel);
                }
            }
            return Ok(DockNode::Tabs { panels, active });
        }
        let axis = if self.eat("row(") {
            Axis::Horizontal
        } else if self.eat("column(") {
            Axis::Vertical
        } else {
            return Err(self.error("node"));
        };
        let mut children = Vec::new();
        loop {
            self.skip_space();
            if self.eat(")") {
                break;
            }
            if self.pos >= self.text.len() {
                return Err(self.error(")"));
            }
            let start = self.pos;
            let fraction = self
                .token()
                .parse::<f32>()
                .map_err(|_| DockLoadError::Syntax {
                    offset: start,
                    expected: "fraction",
                })?;
            self.skip_space();
            children.push((fraction, self.node(depth + 1)?));
        }
        Ok(DockNode::Split { axis, children })
    }

    /// The rest of a quoted panel id, after its opening quote.
    fn string(&mut self) -> Result<String, DockLoadError> {
        let mut out = String::new();
        let mut chars = self.text[self.pos..].char_indices();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => {
                    self.pos += i + 1;
                    return Ok(out);
                }
                '\\' => match chars.next() {
                    Some((_, escaped)) => out.push(escaped),
                    None => break,
                },
                c => out.push(c),
            }
        }
        self.pos = self.text.len();
        Err(self.error("panel"))
    }
}

/// Compares the trees (axes, fractions within 1e-5, panels, active tabs), ignoring ids.
impl PartialEq for DockLayout {
    fn eq(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (None, None) => true,
            (Some(a), Some(b)) => a.same(b),
            _ => false,
        }
    }
}
