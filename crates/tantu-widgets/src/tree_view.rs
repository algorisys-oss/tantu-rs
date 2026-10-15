//! Virtualized trees. Spec: `docs/specs/widgets/tree-view.md`.
use crate::{ScrollController, ScrollView, Text};
use std::collections::BTreeSet;
use std::rc::Rc;
use tantu_view::layout::TextDirection;
use tantu_view::reactive::Signal;
use tantu_view::{BuildCx, ElementId, IntoProp, View};

/// Stable application-assigned identity, unique throughout one source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeNodeId(pub u64);
/// An ordered forest with synchronous queries, consistent until the next revision.
pub trait TreeDataSource: 'static {
    /// Number of roots.
    fn root_count(&self) -> usize;
    /// Root at this zero-based index, or None for a missing entry.
    fn root(&self, index: usize) -> Option<TreeNodeId>;
    /// Immediate child count; zero for leaves and unknown nodes.
    fn child_count(&self, node: TreeNodeId) -> usize;
    /// Immediate child at this zero-based index, or None for a missing entry.
    fn child(&self, node: TreeNodeId, index: usize) -> Option<TreeNodeId>;
    /// Display/accessibility label; empty for unknown nodes.
    fn label(&self, node: TreeNodeId) -> String;
}
/// Snapshot passed to a custom row-content builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeRowState {
    /// Stable node identity.
    pub node: TreeNodeId,
    /// Zero-based depth.
    pub depth: usize,
    /// Whether the source reports children.
    pub expandable: bool,
    /// Whether this branch is open.
    pub expanded: bool,
    /// Whether this row is selected.
    pub selected: bool,
}
type RowBuilder = Rc<dyn Fn(TreeRowState) -> tantu_view::AnyView>;

/// A vertically scrolling virtualized tree with a single focus stop.
pub struct TreeView {
    source: Rc<dyn TreeDataSource>,
    build: Option<RowBuilder>,
    extent: f32,
    indent: f32,
    expanded: Option<Signal<BTreeSet<TreeNodeId>>>,
    selection: Option<Signal<Option<TreeNodeId>>>,
    revision: tantu_view::Prop<u64>,
    controller: Option<ScrollController>,
    activate: Option<Rc<dyn Fn(TreeNodeId)>>,
}
impl TreeView {
    /// Builds default text labels, with 32 px rows and 20 px indentation.
    pub fn new(source: Rc<dyn TreeDataSource>) -> Self {
        Self {
            source,
            build: None,
            extent: 32.0,
            indent: 20.0,
            expanded: None,
            selection: None,
            revision: tantu_view::Prop::Value(0),
            controller: None,
            activate: None,
        }
    }
    /// Replaces label content only; chrome remains the tree's responsibility.
    pub fn builder<V: View>(
        source: Rc<dyn TreeDataSource>,
        build: impl Fn(TreeRowState) -> V + 'static,
    ) -> Self {
        let mut tree = Self::new(source);
        tree.build = Some(Rc::new(move |state| tantu_view::AnyView::new(build(state))));
        tree
    }
    /// Row height; invalid values fall back to 32 px.
    pub fn item_extent(mut self, extent: f32) -> Self {
        self.extent = if extent.is_finite() && extent > 0.0 {
            extent
        } else {
            32.0
        };
        self
    }
    /// Per-depth indentation; negative or non-finite values fall back to 20 px.
    pub fn indentation(mut self, indentation: f32) -> Self {
        self.indent = if indentation.is_finite() && indentation >= 0.0 {
            indentation
        } else {
            20.0
        };
        self
    }
    /// Shares expansion state; defaults to an owner-scoped empty set.
    pub fn expanded(mut self, expanded: Signal<BTreeSet<TreeNodeId>>) -> Self {
        self.expanded = Some(expanded);
        self
    }
    /// Shares selection; defaults to an owner-scoped None signal.
    pub fn selection(mut self, selection: Signal<Option<TreeNodeId>>) -> Self {
        self.selection = Some(selection);
        self
    }
    /// Refreshes structure and mounted content when the revision changes; default 0.
    pub fn revision(mut self, revision: impl IntoProp<u64>) -> Self {
        self.revision = revision.into_prop();
        self
    }
    /// Shares the vertical scroll controller.
    pub fn controller(mut self, controller: ScrollController) -> Self {
        self.controller = Some(controller);
        self
    }
    /// Activates the selected node on a non-repeated Enter press.
    pub fn on_activate(mut self, activate: impl Fn(TreeNodeId) + 'static) -> Self {
        self.activate = Some(Rc::new(activate));
        self
    }
}

use crate::theme::ThemeHandle;
use crate::util::dynamic;
use std::cell::RefCell;
use std::collections::HashMap;
use tantu_view::core::{Affine, Color, Point, Rect, Size, Vec2};
use tantu_view::layout::{BoxConstraints, LayoutChildren, RenderBox, RenderConstrainedBox};
use tantu_view::reactive::{effect, memo, on_cleanup, signal, untrack};
use tantu_view::{
    AnyView, Dyn, FocusOptions, For, Handled, KeyEvent, LogicalKey, NamedKey, Paint, PaintCx,
    Phase, PointerButton, PointerKind, Prop, Role, Semantics, SemanticsAction, SemanticsActions,
};

/// One displayed node; no label is stored in the index.
#[derive(Clone, Copy)]
struct Meta {
    node: TreeNodeId,
    depth: usize,
    parent: Option<TreeNodeId>,
    position: usize,
    siblings: usize,
    children: usize,
    expanded: bool,
}
#[derive(Default)]
struct Index {
    rows: Vec<Meta>,
    by_id: HashMap<TreeNodeId, usize>,
    revision: u64,
}
/// Traversal cursor: enumerate siblings one at a time, without allocating closed children.
struct Cursor {
    parent: Option<TreeNodeId>,
    depth: usize,
    next: usize,
    count: usize,
}
impl Index {
    fn build(source: &dyn TreeDataSource, expanded: &BTreeSet<TreeNodeId>, revision: u64) -> Self {
        let mut index = Self {
            revision,
            ..Default::default()
        };
        let mut stack = vec![Cursor {
            parent: None,
            depth: 0,
            next: 0,
            count: source.root_count(),
        }];
        while let Some(cursor) = stack.last_mut() {
            if cursor.next == cursor.count {
                stack.pop();
                continue;
            }
            let position = cursor.next;
            cursor.next += 1;
            let node = match cursor.parent {
                Some(parent) => source.child(parent, position),
                None => source.root(position),
            };
            let Some(node) = node else {
                continue;
            };
            if index.by_id.contains_key(&node) {
                tracing::warn!(node = node.0, "duplicate or cyclic TreeView node skipped");
                continue;
            }
            let children = source.child_count(node);
            let meta = Meta {
                node,
                depth: cursor.depth,
                parent: cursor.parent,
                position: position + 1,
                siblings: cursor.count,
                children,
                expanded: children > 0 && expanded.contains(&node),
            };
            index.by_id.insert(node, index.rows.len());
            index.rows.push(meta);
            if meta.expanded {
                stack.push(Cursor {
                    parent: Some(node),
                    depth: meta.depth + 1,
                    next: 0,
                    count: children,
                });
            }
        }
        index
    }
    fn meta(&self, node: TreeNodeId) -> Option<Meta> {
        self.by_id.get(&node).map(|i| self.rows[*i])
    }
}

/// Mounted IDs plus a cheap invalidation signal; updating one row never clones the map.
#[derive(Clone)]
struct Mounted {
    rows: Rc<RefCell<HashMap<TreeNodeId, ElementId>>>,
    version: Signal<usize>,
}
impl Mounted {
    fn changed(&self) {
        self.version.update(|v| *v = v.wrapping_add(1));
    }
}

#[derive(Clone)]
struct Navigation {
    index: Signal<Rc<Index>>,
    expanded: Signal<BTreeSet<TreeNodeId>>,
    selected: Signal<Option<TreeNodeId>>,
    controller: ScrollController,
    extent: f32,
    activate: Option<Rc<dyn Fn(TreeNodeId)>>,
}
fn along(index: usize, extent: f32) -> f32 {
    (index as f32 * extent).min(f32::MAX / 4.0)
}
impl Navigation {
    fn reveal(&self, node: TreeNodeId) {
        let index = self.index.get_untracked();
        let Some(i) = index.by_id.get(&node) else {
            return;
        };
        let start = along(*i, self.extent);
        let viewport = self.controller.viewport_extent_untracked();
        let offset = self.controller.offset_untracked();
        let target = if self.extent > viewport || start < offset {
            start
        } else if start + self.extent > offset + viewport {
            start + self.extent - viewport
        } else {
            return;
        };
        self.controller.jump_to(target);
    }
    fn select(&self, node: TreeNodeId) {
        if self.selected.get_untracked() != Some(node) {
            self.selected.set(Some(node));
        }
        self.reveal(node);
    }
    fn set_open(&self, node: TreeNodeId, open: bool) {
        let Some(meta) = self.index.get_untracked().meta(node) else {
            return;
        };
        if meta.children == 0 || meta.expanded == open {
            return;
        }
        self.expanded.update(|set| {
            if open {
                set.insert(node);
            } else {
                set.remove(&node);
            }
        });
    }
    fn key(&self, event: &KeyEvent) -> bool {
        let index = self.index.get_untracked();
        if index.rows.is_empty() {
            return false;
        }
        let current = self
            .selected
            .get_untracked()
            .and_then(|id| index.by_id.get(&id).copied());
        let last = index.rows.len() - 1;
        let page = (self.controller.viewport_extent_untracked() / self.extent)
            .floor()
            .max(1.0) as usize;
        let target = match &event.key {
            LogicalKey::Named(NamedKey::ArrowDown) => {
                Some(current.map_or(0, |i| (i + 1).min(last)))
            }
            LogicalKey::Named(NamedKey::ArrowUp) => {
                Some(current.map_or(last, |i| i.saturating_sub(1)))
            }
            LogicalKey::Named(NamedKey::Home) => Some(0),
            LogicalKey::Named(NamedKey::End) => Some(last),
            LogicalKey::Named(NamedKey::PageDown) => {
                Some(current.map_or(0, |i| i.saturating_add(page).min(last)))
            }
            LogicalKey::Named(NamedKey::PageUp) => {
                Some(current.map_or(last, |i| i.saturating_sub(page)))
            }
            LogicalKey::Named(NamedKey::ArrowRight | NamedKey::ArrowLeft) => {
                let Some(i) = current else {
                    self.select(index.rows[0].node);
                    return true;
                };
                let meta = index.rows[i];
                if event.key == LogicalKey::Named(NamedKey::ArrowRight) {
                    if !meta.expanded && meta.children > 0 {
                        self.set_open(meta.node, true);
                    } else if meta.expanded
                        && index
                            .rows
                            .get(i + 1)
                            .is_some_and(|m| m.parent == Some(meta.node))
                    {
                        self.select(index.rows[i + 1].node);
                    }
                } else if meta.expanded {
                    self.set_open(meta.node, false);
                } else if let Some(parent) = meta.parent {
                    self.select(parent);
                }
                return true;
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if !event.repeat {
                    if let (Some(i), Some(activate)) = (current, &self.activate) {
                        activate(index.rows[i].node);
                    }
                }
                return true;
            }
            LogicalKey::Character(key) if key.as_ref() == " " => {
                if !event.repeat {
                    if let Some(i) = current {
                        let m = index.rows[i];
                        self.set_open(m.node, !m.expanded);
                    }
                }
                return true;
            }
            _ => return false,
        };
        if let Some(i) = target {
            self.select(index.rows[i].node);
        }
        true
    }
    fn window(&self) -> (usize, usize) {
        let count = self.index.get().rows.len();
        if count == 0 {
            return (0, 0);
        }
        let offset = self
            .controller
            .offset()
            .min(along(count.saturating_sub(1), self.extent));
        let viewport = self.controller.viewport_extent();
        let first = (offset / self.extent).floor() as usize;
        let end = ((offset + viewport).next_down().max(offset) / self.extent).floor() as usize;
        (first.saturating_sub(1), end.saturating_add(2).min(count))
    }
}

impl View for TreeView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        // A private owner region outlives the viewport and all of its row scopes.
        let mut viewport = None;
        cx.region(|cx| {
            let expanded = self.expanded.unwrap_or_else(|| signal(BTreeSet::new()));
            let selected = self.selection.unwrap_or_else(|| signal(None));
            let controller = self.controller.unwrap_or_default();
            let index = signal(Rc::new(Index::default()));
            let revision = dynamic(self.revision);
            let request = memo(move || (revision(), expanded.get()));
            {
                let source = self.source.clone();
                effect(move || {
                    let (revision, expanded) = request.get();
                    let next = Rc::new(untrack(|| {
                        Index::build(source.as_ref(), &expanded, revision)
                    }));
                    let old = index.get_untracked();
                    if let Some(id) = selected
                        .get_untracked()
                        .filter(|id| !next.by_id.contains_key(id))
                    {
                        let mut fallback = None;
                        if old.revision == revision {
                            let mut parent = old.meta(id).and_then(|m| m.parent);
                            while let Some(id) = parent {
                                if next.by_id.contains_key(&id) {
                                    fallback = Some(id);
                                    break;
                                }
                                parent = old.meta(id).and_then(|m| m.parent);
                            }
                        }
                        selected.set(fallback);
                    }
                    index.set(next);
                });
            }
            let nav = Navigation {
                index,
                expanded,
                selected,
                controller,
                extent: self.extent,
                activate: self.activate,
            };
            let mounted = Mounted {
                rows: Rc::default(),
                version: signal(0),
            };
            let focused = signal(false);
            let theme = ThemeHandle::of(cx);
            let wrapper = cx.render(
                RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
                [AnyView::new(
                    ScrollView::new(TreeContent {
                        nav: nav.clone(),
                        source: self.source,
                        build: self.build,
                        indent: self.indent,
                        mounted: mounted.clone(),
                    })
                    .controller(controller),
                )],
            );
            let Some(id) = cx.tree().children(wrapper).first().copied() else {
                return;
            };
            viewport = Some(id);
            cx.set_paint(wrapper, FocusPaint { focused, theme });
            cx.bind(
                wrapper,
                Prop::Dynamic(Box::new(move || (focused.get(), theme.colors()))),
                |e, _| {
                    e.update_paint::<FocusPaint>(|_| {});
                },
            );
            cx.focusable(
                id,
                FocusOptions {
                    traversable: true,
                    focus_on_press: true,
                },
            );
            cx.on_focus_change(id, move |value| focused.set(value));
            cx.set_semantics(
                id,
                Semantics {
                    role: Role::Tree,
                    focusable: true,
                    actions: SemanticsActions {
                        focus: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    mounted.version.get();
                    selected
                        .get()
                        .and_then(|id| mounted.rows.borrow().get(&id).copied())
                })),
                |e, active| {
                    e.update_semantics(|s| s.active_descendant = active);
                },
            );
            {
                let nav = nav.clone();
                // Left and Right swap under RTL (WIDGETS-RTL-04).
                let direction = dynamic(crate::directionality::direction_prop(cx));
                cx.on_key(id, Phase::Capture, move |k| {
                    let mut event = k.event.clone();
                    if direction() == TextDirection::Rtl {
                        event.key = match event.key {
                            LogicalKey::Named(NamedKey::ArrowLeft) => {
                                LogicalKey::Named(NamedKey::ArrowRight)
                            }
                            LogicalKey::Named(NamedKey::ArrowRight) => {
                                LogicalKey::Named(NamedKey::ArrowLeft)
                            }
                            other => other,
                        };
                    }
                    if focused.get_untracked()
                        && event.pressed
                        && event.modifiers == Default::default()
                        && nav.key(&event)
                    {
                        Handled::Stop
                    } else {
                        Handled::Continue
                    }
                });
            }
            // Offset is intentionally untracked: manual scrolling may unmount the selection.
            effect(move || {
                let index = nav.index.get();
                let selection = selected.get();
                controller.viewport_extent();
                if let Some(node) = selection {
                    if index.by_id.contains_key(&node) {
                        nav.reveal(node);
                    } else {
                        selected.set(None);
                    }
                }
            });
        });
        viewport.unwrap_or_else(|| cx.region(|_| {}))
    }
}
struct TreeContent {
    nav: Navigation,
    source: Rc<dyn TreeDataSource>,
    build: Option<RowBuilder>,
    indent: f32,
    mounted: Mounted,
}
impl View for TreeContent {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let nav = self.nav;
        let each = nav.clone();
        let rows = nav.clone();
        let layout = nav.clone();
        let items = For::new(
            move || {
                let (first, end) = each.window();
                let index = each.index.get();
                index.rows[first..end].iter().map(|m| m.node).collect()
            },
            |node: &TreeNodeId| *node,
            move |node| TreeRow {
                node,
                nav: rows.clone(),
                source: self.source.clone(),
                build: self.build.clone(),
                indent: self.indent,
                mounted: self.mounted.clone(),
            },
        );
        let id = cx.render(
            ContentRender {
                count: 0,
                first: 0,
                extent: nav.extent,
            },
            [AnyView::new(items)],
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (layout.index.get().rows.len(), layout.window().0)
            })),
            |e, (count, first)| {
                e.update_render::<ContentRender>(|r| {
                    r.count = count;
                    r.first = first;
                });
            },
        );
        id
    }
}
/// Children are the reconciled buffered slice in order; no stale absolute parent-data index.
#[derive(Clone, PartialEq)]
struct ContentRender {
    count: usize,
    first: usize,
    extent: f32,
}
impl RenderBox for ContentRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let bounded = c.has_bounded_width();
        let mut width = if bounded { c.max_width } else { 0.0 };
        for i in 0..children.len() {
            let size = children.layout(
                i,
                BoxConstraints::new(
                    if bounded { width } else { 0.0 },
                    c.max_width,
                    self.extent,
                    self.extent,
                ),
            );
            width = width.max(size.width);
            children.set_offset(i, Vec2::new(0.0, along(self.first + i, self.extent)));
        }
        c.constrain(Size::new(width, along(self.count, self.extent)))
    }
}
struct TreeRow {
    node: TreeNodeId,
    nav: Navigation,
    source: Rc<dyn TreeDataSource>,
    build: Option<RowBuilder>,
    indent: f32,
    mounted: Mounted,
}
impl View for TreeRow {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let node = self.node;
        let nav = self.nav;
        let index = nav.index;
        let selected = nav.selected;
        let theme = ThemeHandle::of(cx);
        let hover = signal(false);
        let source = self.source;
        let label_source = source.clone();
        let content = Dyn::new(move || {
            let index = index.get();
            let meta = index.meta(node);
            let state = TreeRowState {
                node,
                depth: meta.map_or(0, |m| m.depth),
                expandable: meta.is_some_and(|m| m.children > 0),
                expanded: meta.is_some_and(|m| m.expanded),
                selected: selected.get() == Some(node),
            };
            if let Some(build) = &self.build {
                build(state)
            } else {
                AnyView::new(
                    Text::new(source.label(node))
                        .style(theme.snapshot().text.body)
                        .color(move || theme.colors().on_surface),
                )
            }
        });
        let id = cx.render(
            RowRender {
                indent: 0.0,
                rtl: false,
            },
            [AnyView::new(content)],
        );
        // Indented from the right under RTL (WIDGETS-RTL-04).
        let direction = dynamic(crate::directionality::direction_prop(cx));
        {
            let direction = Rc::clone(&direction);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || direction() == TextDirection::Rtl)),
                |e, rtl| {
                    e.update_render::<RowRender>(|r| r.rtl = rtl);
                    e.update_paint::<RowPaint>(|p| p.rtl = rtl);
                },
            );
        }
        let mounted = self.mounted;
        mounted.rows.borrow_mut().insert(node, id);
        mounted.changed();
        on_cleanup(move || {
            mounted.rows.borrow_mut().remove(&node);
            if mounted.version.try_get().is_some() {
                mounted.changed();
            }
        });
        cx.set_semantics(
            id,
            Semantics {
                role: Role::TreeItem,
                actions: SemanticsActions {
                    click: true,
                    scroll_into_view: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        cx.set_paint(
            id,
            RowPaint {
                indent: 0.0,
                expanded: None,
                selected: false,
                hover,
                theme,
                rtl: false,
            },
        );
        let indent = self.indent;
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || (index.get(), selected.get()))),
            move |e, (index, selection)| {
                if let Some(m) = index.meta(node) {
                    let indent = along(m.depth, indent);
                    e.update_render::<RowRender>(|r| r.indent = indent);
                    e.update_paint::<RowPaint>(|p| {
                        p.indent = indent;
                        p.expanded = (m.children > 0).then_some(m.expanded);
                        p.selected = selection == Some(node);
                    });
                    e.update_semantics(|s| {
                        s.name = Some(label_source.label(node));
                        s.level = Some(m.depth + 1);
                        s.position_in_set = Some(m.position);
                        s.size_of_set = Some(m.siblings);
                        s.expanded = (m.children > 0).then_some(m.expanded);
                        s.selected = Some(selection == Some(node));
                        s.actions.expand = m.children > 0 && !m.expanded;
                        s.actions.collapse = m.expanded;
                    });
                }
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || (hover.get(), theme.colors()))),
            |e, _| {
                e.update_paint::<RowPaint>(|_| {});
            },
        );
        {
            let nav = nav.clone();
            cx.on_pointer(id, Phase::Capture, move |p| {
                if p.event.kind == PointerKind::Down(PointerButton::Primary) {
                    let index = nav.index.get_untracked();
                    let Some(m) = index.meta(node) else {
                        return Handled::Continue;
                    };
                    nav.select(node);
                    let start = along(m.depth, indent).min(p.size.width);
                    let x = if direction() == TextDirection::Rtl {
                        p.size.width - p.local.x
                    } else {
                        p.local.x
                    };
                    if m.children > 0 && x >= start && x < (start + 20.0).min(p.size.width) {
                        nav.set_open(node, !m.expanded);
                        return Handled::Stop;
                    }
                }
                Handled::Continue
            });
        }
        cx.on_pointer(id, Phase::Bubble, move |p| {
            match p.event.kind {
                PointerKind::Enter => hover.set(true),
                PointerKind::Leave => hover.set(false),
                _ => {}
            }
            Handled::Continue
        });
        cx.on_action(id, move |a| match a.action {
            SemanticsAction::Click => {
                nav.select(node);
                true
            }
            SemanticsAction::Expand => {
                nav.set_open(node, true);
                true
            }
            SemanticsAction::Collapse => {
                nav.set_open(node, false);
                true
            }
            _ => false,
        });
        id
    }
}
#[derive(Clone, PartialEq)]
struct RowRender {
    indent: f32,
    rtl: bool,
}
impl RenderBox for RowRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let start = (self.indent + 20.0).min(c.max_width);
        let width = (c.max_width - start).max(0.0);
        let mut size = Size::ZERO;
        for i in 0..children.len() {
            let child = children.layout(i, BoxConstraints::new(0.0, width, 0.0, c.max_height));
            size = size.max(child);
        }
        let row = c.constrain(Size::new(
            if c.has_bounded_width() {
                c.max_width
            } else {
                start + size.width
            },
            size.height,
        ));
        for i in 0..children.len() {
            let child = children.size(i);
            let x = if self.rtl {
                row.width - start - child.width
            } else {
                start
            };
            children.set_offset(
                i,
                Vec2::new(x, ((c.max_height - child.height) / 2.0).max(0.0)),
            );
        }
        row
    }
}
struct RowPaint {
    indent: f32,
    expanded: Option<bool>,
    selected: bool,
    hover: Signal<bool>,
    theme: ThemeHandle,
    rtl: bool,
}
fn bar(cx: &mut PaintCx<'_, '_>, from: Point, to: Point, color: Color) {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    cx.scene().push_transform(
        Affine::translate(Vec2::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0))
            * Affine::rotate(dy.atan2(dx)),
    );
    cx.scene().fill_rect(
        Rect::from_ltrb(-dx.hypot(dy) / 2.0, -1.0, dx.hypot(dy) / 2.0, 1.0),
        color,
    );
    cx.scene().pop();
}
impl Paint for RowPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let colors = self.theme.colors();
        let bounds = Rect::from_origin_size(Point::ZERO, size);
        if self.selected {
            cx.scene()
                .fill_rect(bounds, colors.primary.with_alpha(0.12));
        }
        if self.hover.get() {
            cx.scene()
                .fill_rect(bounds, colors.on_surface.with_alpha(0.08));
        }
        if let Some(open) = self.expanded {
            let start = self.indent.min(size.width) + 10.0;
            // Under RTL the expander sits at the right and a closed one points left.
            let (x, toward) = if self.rtl {
                (size.width - start, -1.0)
            } else {
                (start, 1.0)
            };
            let y = size.height / 2.0;
            let (a, b, c) = if open {
                (
                    Point::new(x - 4.0, y - 2.0),
                    Point::new(x, y + 2.0),
                    Point::new(x + 4.0, y - 2.0),
                )
            } else {
                (
                    Point::new(x - 2.0 * toward, y - 4.0),
                    Point::new(x + 2.0 * toward, y),
                    Point::new(x - 2.0 * toward, y + 4.0),
                )
            };
            bar(cx, a, b, colors.on_surface_variant);
            bar(cx, b, c, colors.on_surface_variant);
        }
    }
}
struct FocusPaint {
    focused: Signal<bool>,
    theme: ThemeHandle,
}
impl Paint for FocusPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        cx.paint_children();
        if self.focused.get() {
            let size = cx.size();
            let color = self.theme.colors().primary;
            for rect in [
                Rect::from_ltrb(0.0, 0.0, size.width, 2.0),
                Rect::from_ltrb(0.0, size.height - 2.0, size.width, size.height),
                Rect::from_ltrb(0.0, 0.0, 2.0, size.height),
                Rect::from_ltrb(size.width - 2.0, 0.0, size.width, size.height),
            ] {
                cx.scene().fill_rect(rect, color);
            }
        }
    }
}
