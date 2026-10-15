//! [`SplitView`] and [`SplitPane`]: resizable panes. Spec: `docs/specs/widgets/split-view.md`.

use tantu_view::layout::Axis;
use tantu_view::reactive::Signal;
use tantu_view::{AnyView, BuildCx, ElementId, View};

/// One pane of a [`SplitView`]: its content and how it may be resized.
pub struct SplitPane {
    content: AnyView,
    min_size: f32,
    collapsible: bool,
    label: Option<String>,
}

impl SplitPane {
    /// A pane showing `content`, with no minimum size, not collapsible and unlabelled.
    pub fn new(content: impl View) -> Self {
        SplitPane {
            content: AnyView::new(content),
            min_size: 0.0,
            collapsible: false,
            label: None,
        }
    }

    /// The smallest size on the split's axis, in logical pixels (default 0).
    pub fn min_size(mut self, min: f32) -> Self {
        self.min_size = min;
        self
    }

    /// Lets the user collapse the pane to nothing (default false).
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Names the pane for assistive technology; its divider is then "Resize <label>".
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Panes along one axis with draggable dividers between them.
pub struct SplitView {
    axis: Axis,
    panes: Vec<SplitPane>,
    fractions: Option<Signal<Vec<f32>>>,
    thickness: f32,
    grab: f32,
}

impl SplitView {
    /// Panes side by side (`Axis::Horizontal`) or stacked (`Axis::Vertical`).
    pub fn new(axis: Axis) -> Self {
        SplitView {
            axis,
            panes: Vec::new(),
            fractions: None,
            thickness: 1.0,
            grab: 8.0,
        }
    }

    /// Panes side by side.
    pub fn horizontal() -> Self {
        SplitView::new(Axis::Horizontal)
    }

    /// Panes stacked top to bottom.
    pub fn vertical() -> Self {
        SplitView::new(Axis::Vertical)
    }

    /// Adds the next pane.
    pub fn pane(mut self, pane: SplitPane) -> Self {
        self.panes.push(pane);
        self
    }

    /// Shares the fractions (one per pane, summing to 1) with the app, which may set and
    /// persist them. Without it the split owns a signal starting at equal shares.
    pub fn fractions(mut self, fractions: Signal<Vec<f32>>) -> Self {
        self.fractions = Some(fractions);
        self
    }

    /// The visible line's thickness (default 1 px) and the grab area's (default 8 px).
    pub fn divider(mut self, thickness: f32, grab: f32) -> Self {
        self.thickness = thickness;
        self.grab = grab;
        self
    }
}

impl View for SplitView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        for pane in self.panes {
            let _ = (pane.content, pane.min_size, pane.collapsible, pane.label);
        }
        let _ = (cx, self.axis, self.fractions, self.thickness, self.grab);
        todo!("WIDGETS-SPLIT-01..08")
    }
}
