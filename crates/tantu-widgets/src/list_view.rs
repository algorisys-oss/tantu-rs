//! [`ListView`]. Spec: `docs/specs/widgets/list-view.md`.

use std::rc::Rc;

use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Prop, View};

use crate::ScrollController;

/// A scrolling list that builds only its visible items (Flutter's `ListView.builder` with
/// `itemExtent`).
pub struct ListView {
    count: Prop<usize>,
    item_extent: f32,
    build: Rc<dyn Fn(usize) -> AnyView>,
    horizontal: bool,
    controller: Option<ScrollController>,
}

impl ListView {
    /// `count` items (a value or a signal) of `item_extent` logical pixels each, built by
    /// `build(index)` when they come into view.
    pub fn builder<V: View>(
        count: impl IntoProp<usize>,
        item_extent: f32,
        build: impl Fn(usize) -> V + 'static,
    ) -> Self {
        ListView {
            count: count.into_prop(),
            item_extent,
            build: Rc::new(move |index| AnyView::new(build(index))),
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

impl View for ListView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.count,
            self.item_extent,
            self.build,
            self.horizontal,
            self.controller,
        );
        todo!("WIDGETS-LIST-01")
    }
}
