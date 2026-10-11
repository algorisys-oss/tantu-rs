//! [`ListView`]. Specs: `docs/specs/widgets/list-view.md` and
//! `docs/specs/widgets/list-view-advanced.md`.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use tantu_view::core::Point;
use tantu_view::layout::{
    BoxConstraints, ListExtents, ListIndex, RenderConstrainedBox, RenderListContent, ScrollAxis,
};
use tantu_view::reactive::{Memo, Signal, Timer, memo, timer};
use tantu_view::{
    AnyView, BuildCx, ElementId, For, Handled, IntoProp, LogicalKey, NamedKey, Phase,
    PointerButton, PointerKind, Prop, Role, Semantics, SemanticsActions, View,
};

use crate::util::dynamic;
use crate::{ScrollController, ScrollView};

/// Builds a view for an index.
type Builder = Rc<dyn Fn(usize) -> AnyView>;

/// A second press within this interval of the first is a double-click (WIDGETS-LIST-06).
const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(500);
/// ... and within this distance.
const DOUBLE_CLICK_SLOP: f32 = 4.0;

/// A scrolling list that builds only its visible items (Flutter's `ListView.builder` with
/// `itemExtent`, `itemExtentBuilder` or `ListView.separated`).
pub struct ListView {
    count: Prop<usize>,
    item_extent: f32,
    /// Per-item extents (WIDGETS-LIST-04); `None` for a fixed extent.
    extent_of: Option<Rc<dyn Fn(usize) -> f32>>,
    build: Builder,
    /// Separators between items (WIDGETS-LIST-05): extent and builder.
    separator: Option<(f32, Builder)>,
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
}

/// How the list maps slots (the virtual children) to items and separators, and their
/// extents.
#[derive(Clone)]
struct Layout {
    item_extent: f32,
    extent_of: Option<Rc<dyn Fn(usize) -> f32>>,
    separator_extent: Option<f32>,
}

impl Layout {
    /// Slots for `count` items: one per item, plus one between each pair with separators.
    fn slots(&self, count: usize) -> usize {
        match self.separator_extent {
            Some(_) if count > 0 => 2 * count - 1,
            _ => count,
        }
    }

    /// The item of slot `slot`, or `None` for a separator.
    fn item_of(&self, slot: usize) -> Option<usize> {
        match self.separator_extent {
            Some(_) if slot % 2 == 1 => None,
            Some(_) => Some(slot / 2),
            None => Some(slot),
        }
    }

    /// The slot of item `item`.
    fn slot_of(&self, item: usize) -> usize {
        match self.separator_extent {
            Some(_) => 2 * item,
            None => item,
        }
    }

    /// The extents of every slot for `count` items (a table unless every slot is the same).
    fn extents(&self, count: usize) -> ListExtents {
        match (&self.extent_of, self.separator_extent) {
            (None, None) => ListExtents::Fixed(self.item_extent),
            (Some(extent_of), None) => ListExtents::table((0..count).map(|i| extent_of(i))),
            (_, Some(separator)) => ListExtents::table((0..self.slots(count)).map(|slot| {
                match self.item_of(slot) {
                    Some(item) => self
                        .extent_of
                        .as_ref()
                        .map_or(self.item_extent, |f| f(item)),
                    None => separator,
                }
            })),
        }
    }
}

/// What the key and pointer handlers need (WIDGETS-LIST-06).
#[derive(Clone)]
struct Navigation {
    selection: Signal<Option<usize>>,
    on_activate: Option<Rc<dyn Fn(usize)>>,
    layout: Layout,
    count: Rc<dyn Fn() -> usize>,
    /// (slots, their extents), computed once per count change.
    table: Memo<(usize, ListExtents)>,
    controller: ScrollController,
    axis: ScrollAxis,
}

impl Navigation {
    fn count(&self) -> usize {
        (self.count)()
    }

    /// Selects `item` (clamped) and scrolls it fully into view.
    fn select(&self, item: usize) {
        let count = self.count();
        if count == 0 {
            return;
        }
        let item = item.min(count - 1);
        if self.selection.get_untracked() != Some(item) {
            self.selection.set(Some(item));
        }
        let extents = self.table.get_untracked().1;
        let slot = self.layout.slot_of(item);
        let (start, extent) = (extents.start(slot), extents.extent(slot));
        let viewport = self.controller.viewport_extent_untracked();
        let offset = self.controller.offset_untracked();
        let target = if start < offset {
            start
        } else if start + extent > offset + viewport {
            start + extent - viewport
        } else {
            return;
        };
        self.controller.jump_to(target.max(0.0));
    }

    /// How many items from `from` fit the viewport (at least 1).
    fn page(&self, from: usize, forward: bool) -> usize {
        let extents = self.table.get_untracked().1;
        let viewport = self.controller.viewport_extent_untracked();
        let count = self.count();
        let mut fit = 0usize;
        let mut used = 0.0;
        let mut item = from;
        loop {
            if item >= count {
                break;
            }
            let slot = self.layout.slot_of(item);
            let extent = extents.extent(slot)
                + match self.layout.separator_extent {
                    Some(separator) if fit > 0 => separator,
                    _ => 0.0,
                };
            if used + extent > viewport {
                break;
            }
            used += extent;
            fit += 1;
            if forward {
                item += 1;
            } else if item == 0 {
                break;
            } else {
                item -= 1;
            }
        }
        fit.max(1)
    }

    /// Handles a key (WIDGETS-LIST-06); true when it moved or activated.
    fn key(&self, key: &LogicalKey) -> bool {
        let count = self.count();
        if count == 0 {
            return false;
        }
        let LogicalKey::Named(key) = key else {
            return false;
        };
        let (forward, back) = match self.axis {
            ScrollAxis::Vertical => (NamedKey::ArrowDown, NamedKey::ArrowUp),
            ScrollAxis::Horizontal => (NamedKey::ArrowRight, NamedKey::ArrowLeft),
        };
        let current = self.selection.get_untracked();
        let last = count - 1;
        let target = match key {
            k if *k == forward => current.map_or(0, |i| (i + 1).min(last)),
            k if *k == back => current.map_or(last, |i| i.saturating_sub(1)),
            NamedKey::Home => 0,
            NamedKey::End => last,
            NamedKey::PageDown => {
                let from = current.unwrap_or(0);
                (from + self.page(from, true)).min(last)
            }
            NamedKey::PageUp => {
                let from = current.unwrap_or(last);
                from.saturating_sub(self.page(from, false))
            }
            NamedKey::Enter => {
                if let (Some(item), Some(activate)) = (current, &self.on_activate) {
                    activate(item);
                }
                return true;
            }
            _ => return false,
        };
        self.select(target);
        true
    }

    /// The item at `position` along the axis (in content coordinates), if it is one.
    fn item_at(&self, position: f32) -> Option<usize> {
        let count = self.count();
        let extents = self.table.get_untracked().1;
        let slot = extents.index_at(position, self.layout.slots(count))?;
        self.layout.item_of(slot)
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
        let layout = Layout {
            item_extent: self.item_extent,
            extent_of: self.extent_of,
            separator_extent: self.separator.as_ref().map(|(extent, _)| *extent),
        };
        let count = dynamic(self.count);
        // The slot count and extents, computed once per count change (WIDGETS-LIST-04).
        let table = {
            let (count, layout) = (Rc::clone(&count), layout.clone());
            memo(move || {
                let n = count();
                (layout.slots(n), layout.extents(n))
            })
        };
        let content = ListContent {
            axis,
            layout: layout.clone(),
            table,
            build: self.build,
            separator: self.separator.map(|(_, build)| build),
            controller,
            selection: self.selection,
        };
        let mut scroll = ScrollView::new(content).controller(controller);
        if self.horizontal {
            scroll = scroll.horizontal();
        }
        let id = scroll.build(cx);
        // The list's own role replaces the scroll view's (WIDGETS-SEM-02).
        cx.set_semantics(
            id,
            Semantics {
                role: Role::List,
                ..Default::default()
            },
        );
        let Some(selection) = self.selection else {
            return id;
        };
        // Selection and keys (WIDGETS-LIST-06), on the scroll view's element: a capture
        // handler runs before the view's own scrolling keys.
        let nav = Navigation {
            selection,
            on_activate: self.on_activate,
            layout,
            count,
            table,
            controller,
            axis,
        };
        {
            let nav = nav.clone();
            cx.on_key(id, Phase::Capture, move |k| {
                if k.event.pressed && nav.key(&k.event.key) {
                    Handled::Stop
                } else {
                    Handled::Continue
                }
            });
        }
        let scope = cx.tree().scope(id);
        let clicks = Cell::new(0u8);
        let last_press = Cell::new(Point::ZERO);
        let interval: Cell<Option<Timer>> = Cell::new(None);
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if p.event.kind != PointerKind::Down(PointerButton::Primary) {
                return Handled::Continue;
            }
            let along = match nav.axis {
                ScrollAxis::Vertical => p.local.y,
                ScrollAxis::Horizontal => p.local.x,
            };
            let Some(item) = nav.item_at(along + nav.controller.offset_untracked()) else {
                return Handled::Continue;
            };
            let near = (p.local.x - last_press.get().x).hypot(p.local.y - last_press.get().y)
                <= DOUBLE_CLICK_SLOP;
            let within = interval.get().is_some_and(Timer::is_pending);
            let count = if within && near { clicks.get() + 1 } else { 1 };
            clicks.set(if count > 2 { 1 } else { count });
            last_press.set(p.local);
            if let Some(timer) = interval.take() {
                timer.cancel();
            }
            if let Some(scope) = scope.filter(|s| !s.is_disposed()) {
                interval.set(Some(scope.run(|| timer(DOUBLE_CLICK_INTERVAL, || {}))));
            }
            nav.select(item);
            if clicks.get() == 2 {
                if let Some(activate) = &nav.on_activate {
                    activate(item);
                }
            }
            // The press still reaches the item's handlers (it bubbles on).
            Handled::Continue
        });
        id
    }
}

/// The list's content: as long as every slot, holding the visible ones (WIDGETS-LIST-02).
struct ListContent {
    axis: ScrollAxis,
    layout: Layout,
    table: Memo<(usize, ListExtents)>,
    build: Builder,
    separator: Option<Builder>,
    controller: ScrollController,
    selection: Option<Signal<Option<usize>>>,
}

impl View for ListContent {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (layout, table) = (self.layout, self.table);
        let (controller, build, separator) = (self.controller, self.build, self.separator);
        let visible = move || {
            let (slots, extents) = table.get();
            let (offset, viewport) = (controller.offset(), controller.viewport_extent());
            // The slot starting exactly at the viewport's end isn't visible.
            let end = (offset + viewport).next_down().max(offset);
            let (Some(first), Some(last)) = (
                extents.index_at(offset, slots),
                extents.index_at(end, slots),
            ) else {
                return Vec::new();
            };
            let first = first.saturating_sub(1);
            let last = (last + 2).min(slots);
            (first..last.max(first)).collect()
        };
        let selection = self.selection;
        let items = {
            let layout = layout.clone();
            For::new(
                visible,
                |slot: &usize| *slot,
                move |slot| match layout.item_of(slot) {
                    Some(item) => ListItem {
                        slot,
                        item: Some((item, selection)),
                        child: build(item),
                    },
                    None => ListItem {
                        slot,
                        item: None,
                        child: separator
                            .as_ref()
                            .map_or_else(|| AnyView::new(Nothing), |f| f(slot / 2)),
                    },
                },
            )
        };
        let id = cx.render(
            RenderListContent::new(self.axis, 0, layout.item_extent),
            [AnyView::new(items)],
        );
        // The count and the extents follow the count prop (WIDGETS-LIST-03, -04).
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || table.get())),
            move |e, (slots, extents)| {
                e.update_render::<RenderListContent>(|r| {
                    r.count = slots;
                    r.extents = extents;
                });
            },
        );
        id
    }
}

/// An empty region, for a separator slot without a separator builder.
struct Nothing;

impl View for Nothing {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}

/// One built slot: a pass-through box carrying the slot as parent data and, for an item,
/// `ListItem` semantics with its selection state (WIDGETS-SEM-02).
struct ListItem {
    slot: usize,
    /// The item index and the list's selection, for an item (a separator has none).
    item: Option<(usize, Option<Signal<Option<usize>>>)>,
    child: AnyView,
}

impl View for ListItem {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [self.child],
        );
        cx.set_parent_data(id, ListIndex(self.slot));
        if let Some((item, selection)) = self.item {
            cx.set_semantics(
                id,
                Semantics {
                    role: Role::ListItem,
                    selected: Some(false),
                    actions: SemanticsActions {
                        scroll_into_view: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            if let Some(selection) = selection {
                cx.bind(
                    id,
                    Prop::Dynamic(Box::new(move || selection.get() == Some(item))),
                    |e, selected| {
                        e.update_semantics(|s| s.selected = Some(selected));
                    },
                );
            }
        }
        id
    }
}
