//! [`ScrollView`] and [`ScrollController`]. Spec: `docs/specs/widgets/scroll.md`.

use tantu_view::reactive::{Signal, signal};
use tantu_view::{AnyView, BuildCx, ElementId, View};

/// A scroll view's offset and extents, shared between the app and the view (Flutter's
/// `ScrollController`). `Copy`; create it inside a runtime.
#[derive(Clone, Copy)]
pub struct ScrollController {
    offset: Signal<f32>,
    /// (viewport extent, max scroll extent) from the last frame.
    extents: Signal<(f32, f32)>,
}

impl ScrollController {
    /// A controller at offset 0 with no extents yet.
    pub fn new() -> Self {
        ScrollController {
            offset: signal(0.0),
            extents: signal((0.0, 0.0)),
        }
    }

    /// The offset in logical pixels (a tracked read).
    pub fn offset(&self) -> f32 {
        self.offset.get()
    }

    /// Sets the offset; clamped to `0..=max_scroll_extent` at the next frame.
    pub fn jump_to(&self, offset: f32) {
        self.offset.set(offset);
    }

    /// The largest offset: content minus viewport along the axis, at least 0 (0 before the
    /// first frame). A tracked read.
    pub fn max_scroll_extent(&self) -> f32 {
        self.extents.get().1
    }

    /// The viewport's extent along the axis, from the last frame.
    pub fn viewport_extent(&self) -> f32 {
        self.extents.get().0
    }
}

impl Default for ScrollController {
    fn default() -> Self {
        ScrollController::new()
    }
}

/// A scrollable box around one child (Flutter's `SingleChildScrollView` with a `Scrollbar`).
pub struct ScrollView {
    child: AnyView,
    horizontal: bool,
    controller: Option<ScrollController>,
}

impl ScrollView {
    /// A vertical scroll view around `child`, with its own controller.
    pub fn new(child: impl View) -> Self {
        ScrollView {
            child: AnyView::new(child),
            horizontal: false,
            controller: None,
        }
    }

    /// Scrolls left and right instead.
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Shares `controller` with the app.
    pub fn controller(mut self, controller: ScrollController) -> Self {
        self.controller = Some(controller);
        self
    }
}

impl View for ScrollView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.child, self.horizontal, self.controller);
        todo!("WIDGETS-SCROLL-01")
    }
}
