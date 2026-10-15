//! The dock layout model: [`DockLayout`], a normalized tree of splits and tab groups of
//! app-named panels, with operations and a text format. Spec:
//! `docs/specs/widgets/dock-layout.md`.

use std::sync::Arc;

use tantu_view::layout::Axis;

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
pub struct DockGroupId(u64);

/// A split's identity within one layout. Not persisted; layout equality ignores it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DockSplitId(u64);

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

/// The arrangement of docked panels: a normalized tree of splits and tab groups, possibly
/// empty.
#[derive(Clone, Debug, Default)]
pub struct DockLayout {}

impl DockLayout {
    /// The empty layout.
    pub fn empty() -> Self {
        todo!("WIDGETS-DOCK-01")
    }

    /// A layout from a description, normalized (WIDGETS-DOCK-01).
    pub fn new(root: DockNode) -> Self {
        let _ = root;
        todo!("WIDGETS-DOCK-01")
    }

    /// Whether the layout has no panels.
    pub fn is_empty(&self) -> bool {
        todo!("WIDGETS-DOCK-04")
    }

    /// The tree, for reading; `None` when empty.
    pub fn root(&self) -> Option<DockView<'_>> {
        todo!("WIDGETS-DOCK-04")
    }

    /// Every panel, in tree order.
    pub fn panels(&self) -> Vec<PanelId> {
        todo!("WIDGETS-DOCK-04")
    }

    /// Whether `panel` is in the layout.
    pub fn contains(&self, panel: &PanelId) -> bool {
        let _ = panel;
        todo!("WIDGETS-DOCK-04")
    }

    /// The group holding `panel`.
    pub fn group_of(&self, panel: &PanelId) -> Option<DockGroupId> {
        let _ = panel;
        todo!("WIDGETS-DOCK-04")
    }

    /// A group's panels and active index.
    pub fn group(&self, group: DockGroupId) -> Option<(&[PanelId], usize)> {
        let _ = group;
        todo!("WIDGETS-DOCK-04")
    }

    /// A group's active panel.
    pub fn active(&self, group: DockGroupId) -> Option<&PanelId> {
        let _ = group;
        todo!("WIDGETS-DOCK-04")
    }

    /// The groups in tree order.
    pub fn groups(&self) -> Vec<DockGroupId> {
        todo!("WIDGETS-DOCK-04")
    }

    /// Makes `panel` its group's active tab (WIDGETS-DOCK-05).
    pub fn activate(&mut self, panel: &PanelId) -> Result<(), DockError> {
        let _ = panel;
        todo!("WIDGETS-DOCK-05")
    }

    /// Removes `panel` from the layout (WIDGETS-DOCK-06).
    pub fn close(&mut self, panel: &PanelId) -> Result<(), DockError> {
        let _ = panel;
        todo!("WIDGETS-DOCK-06")
    }

    /// Puts a new panel at `target`; a panel already in the layout is moved instead
    /// (WIDGETS-DOCK-07..09).
    pub fn add(&mut self, panel: impl Into<PanelId>, target: DockTarget) -> Result<(), DockError> {
        let _ = (panel.into(), target);
        todo!("WIDGETS-DOCK-07")
    }

    /// Moves a panel that is in the layout to `target` (WIDGETS-DOCK-07..09).
    pub fn move_panel(&mut self, panel: &PanelId, target: DockTarget) -> Result<(), DockError> {
        let _ = (panel, target);
        todo!("WIDGETS-DOCK-07")
    }

    /// Sets a split's fractions, cleaned as LAYOUT-SPLIT-01 (WIDGETS-DOCK-10).
    pub fn set_fractions(
        &mut self,
        split: DockSplitId,
        fractions: &[f32],
    ) -> Result<(), DockError> {
        let _ = (split, fractions);
        todo!("WIDGETS-DOCK-10")
    }

    /// The layout as text (WIDGETS-DOCK-11).
    pub fn save(&self) -> String {
        todo!("WIDGETS-DOCK-11")
    }

    /// Reads a saved layout, keeping only the panels `keep` accepts, then normalizes
    /// (WIDGETS-DOCK-12).
    pub fn load(text: &str, keep: impl Fn(&PanelId) -> bool) -> Result<DockLayout, DockLoadError> {
        let _ = (text, keep);
        todo!("WIDGETS-DOCK-12")
    }
}

/// Compares the trees (axes, fractions, panels, active tabs), ignoring ids.
impl PartialEq for DockLayout {
    fn eq(&self, other: &Self) -> bool {
        let _ = other;
        todo!("WIDGETS-DOCK-03")
    }
}
