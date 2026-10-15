//! [`Directionality`] and [`PositionedDirectional`]: right-to-left layout. Spec:
//! `docs/specs/widgets/directionality.md`.

use tantu_view::layout::TextDirection;
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Prop, View};

/// Gives its subtree a text direction (Flutter's `Directionality`, WIDGETS-DIR-01).
pub struct Directionality {
    direction: Prop<TextDirection>,
    child: AnyView,
}

impl Directionality {
    /// Lays `child` out in `direction` (a value, closure or signal).
    pub fn new(direction: impl IntoProp<TextDirection>, child: impl View) -> Self {
        Directionality {
            direction: direction.into_prop(),
            child: AnyView::new(child),
        }
    }

    /// The nearest direction above the element being built (`Ltr` without one); a tracked
    /// read.
    pub fn of(cx: &BuildCx<'_>) -> TextDirection {
        let _ = cx;
        todo!("WIDGETS-DIR-01")
    }
}

impl View for Directionality {
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.direction, self.child);
        todo!("WIDGETS-DIR-01")
    }
}

/// [`Positioned`](crate::Positioned) with start and end instead of left and right
/// (Flutter's `PositionedDirectional`, WIDGETS-DIR-03).
pub struct PositionedDirectional {
    child: AnyView,
    start: Option<f32>,
    top: Option<f32>,
    end: Option<f32>,
    bottom: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
}

impl PositionedDirectional {
    /// Wraps `child` with no edges set.
    pub fn new(child: impl View) -> Self {
        PositionedDirectional {
            child: AnyView::new(child),
            start: None,
            top: None,
            end: None,
            bottom: None,
            width: None,
            height: None,
        }
    }

    /// Distance from the stack's start edge.
    pub fn start(mut self, value: f32) -> Self {
        self.start = Some(value);
        self
    }

    /// Distance from the stack's top edge.
    pub fn top(mut self, value: f32) -> Self {
        self.top = Some(value);
        self
    }

    /// Distance from the stack's end edge.
    pub fn end(mut self, value: f32) -> Self {
        self.end = Some(value);
        self
    }

    /// Distance from the stack's bottom edge.
    pub fn bottom(mut self, value: f32) -> Self {
        self.bottom = Some(value);
        self
    }

    /// The child's width.
    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(value);
        self
    }

    /// The child's height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(value);
        self
    }
}

impl View for PositionedDirectional {
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.child,
            self.start,
            self.top,
            self.end,
            self.bottom,
            self.width,
            self.height,
        );
        todo!("WIDGETS-DIR-03")
    }
}
