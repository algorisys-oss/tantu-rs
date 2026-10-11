//! [`ListView`]. Spec: `docs/specs/widgets/list-view.md`.

use std::rc::Rc;

use tantu_view::layout::{ListIndex, RenderListContent, ScrollAxis};
use tantu_view::{AnyView, BuildCx, ElementId, For, IntoProp, Prop, View};

use crate::util::dynamic;
use crate::{ScrollController, ScrollView};

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
        let controller = self.controller.unwrap_or_default();
        let axis = if self.horizontal {
            ScrollAxis::Horizontal
        } else {
            ScrollAxis::Vertical
        };
        let content = ListContent {
            axis,
            count: self.count,
            item_extent: self.item_extent,
            build: self.build,
            controller,
        };
        let mut scroll = ScrollView::new(content).controller(controller);
        if self.horizontal {
            scroll = scroll.horizontal();
        }
        scroll.build(cx)
    }
}

/// The list's content: as long as every item, holding the visible ones (WIDGETS-LIST-02).
struct ListContent {
    axis: ScrollAxis,
    count: Prop<usize>,
    item_extent: f32,
    build: Rc<dyn Fn(usize) -> AnyView>,
    controller: ScrollController,
}

impl View for ListContent {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let count = dynamic(self.count);
        let (extent, controller, build) = (self.item_extent, self.controller, self.build);
        let visible = {
            let count = Rc::clone(&count);
            move || {
                let n = count();
                let (offset, viewport) = (controller.offset(), controller.viewport_extent());
                if n == 0 || extent.is_nan() || extent <= 0.0 {
                    return Vec::new();
                }
                let n_f = n as f32;
                let first = ((offset / extent).floor().clamp(0.0, n_f) as usize).saturating_sub(1);
                let last = (((offset + viewport) / extent).ceil() + 1.0).clamp(0.0, n_f) as usize;
                (first..last.max(first)).collect()
            }
        };
        let items = For::new(
            visible,
            |index: &usize| *index,
            move |index| ListItem {
                index,
                child: build(index),
            },
        );
        let id = cx.render(
            RenderListContent::new(self.axis, 0, extent),
            [AnyView::new(items)],
        );
        cx.bind(id, Prop::Dynamic(Box::new(move || count())), |e, count| {
            e.update_render::<RenderListContent>(|r| r.count = count);
        });
        id
    }
}

/// One built item, carrying its index as parent data.
struct ListItem {
    index: usize,
    child: AnyView,
}

impl View for ListItem {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.child.build(cx);
        if !cx.set_parent_data(id, ListIndex(self.index)) {
            tracing::warn!("a ListView item built a dynamic region; it is placed at its position");
        }
        id
    }
}
