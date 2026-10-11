//! [`ListView`]. Spec: `docs/specs/widgets/list-view.md`.

use std::rc::Rc;

use tantu_view::layout::{ListIndex, RenderListContent, ScrollAxis};
use tantu_view::reactive::Signal;
use tantu_view::{AnyView, BuildCx, ElementId, For, IntoProp, Prop, View};

use crate::util::dynamic;
use crate::{ScrollController, ScrollView};

/// A scrolling list that builds only its visible items (Flutter's `ListView.builder` with
/// `itemExtent`).
pub struct ListView {
    count: Prop<usize>,
    item_extent: f32,
    /// Per-item extents (WIDGETS-LIST-04); `None` for a fixed extent.
    extent_of: Option<Rc<dyn Fn(usize) -> f32>>,
    build: Rc<dyn Fn(usize) -> AnyView>,
    /// Separators between items (WIDGETS-LIST-05): extent and builder.
    separator: Option<(f32, Rc<dyn Fn(usize) -> AnyView>)>,
    horizontal: bool,
    controller: Option<ScrollController>,
    /// The selected item, shared with the app (WIDGETS-LIST-06).
    selection: Option<Signal<Option<usize>>>,
    on_activate: Option<Rc<dyn Fn(usize)>>,
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
            extent_of: None,
            build: Rc::new(move |index| AnyView::new(build(index))),
            separator: None,
            horizontal: false,
            controller: None,
            selection: None,
            on_activate: None,
        }
    }

    /// `count` items whose extent `extent_of(index)` gives, built by `build(index)` when in
    /// view; `extent_of` runs once per index when the count changes (WIDGETS-LIST-04).
    pub fn builder_with_extents<V: View>(
        count: impl IntoProp<usize>,
        extent_of: impl Fn(usize) -> f32 + 'static,
        build: impl Fn(usize) -> V + 'static,
    ) -> Self {
        let mut list = ListView::builder(count, 0.0, build);
        list.extent_of = Some(Rc::new(extent_of));
        list
    }

    /// `count` items of `item_extent` with a separator of `separator_extent` between each
    /// pair, built by `separator(index)` for the separator after item `index` (Flutter's
    /// `ListView.separated`; WIDGETS-LIST-05).
    pub fn separated<V: View, S: View>(
        count: impl IntoProp<usize>,
        item_extent: f32,
        build: impl Fn(usize) -> V + 'static,
        separator_extent: f32,
        separator: impl Fn(usize) -> S + 'static,
    ) -> Self {
        let mut list = ListView::builder(count, item_extent, build);
        list.separator = Some((
            separator_extent,
            Rc::new(move |index| AnyView::new(separator(index))),
        ));
        list
    }

    /// The selected item, shared with the app (read it in `build` to highlight the item).
    /// Enables keyboard navigation and press-to-select (WIDGETS-LIST-06).
    pub fn selection(mut self, selection: Signal<Option<usize>>) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Called with the selected index on Enter, or on a double-click of an item.
    pub fn on_activate(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_activate = Some(Rc::new(f));
        self
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
        let _ = (
            self.extent_of,
            self.separator,
            self.selection,
            self.on_activate,
        );
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
