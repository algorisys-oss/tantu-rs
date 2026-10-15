//! [`DockArea`]: shows a [`DockLayout`] and lets the user change it. Spec:
//! `docs/specs/widgets/dock-area.md`.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use tantu_view::core::{EdgeInsets, Point, Rect, Size, Vec2};
use tantu_view::layout::{
    Axis, BoxConstraints, CrossAxisAlignment, DockPlan, DockPlanNode, DockSlot, LayoutChildren,
    MainAxisSize, RenderBox, RenderConstrainedBox, RenderDock,
};
use tantu_view::reactive::{Memo, Signal, effect, memo, on_cleanup, signal, untrack};
use tantu_view::{
    AnyView, BuildCx, CursorIcon, Dyn, ElementId, FocusOptions, For, Handled, LogicalKey, NamedKey,
    NumericValue, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics,
    SemanticsActions, Show, SingleActivator, View,
};

use super::DockTarget;
use super::drag::{
    DRAG_SLOP, DRAGGED_OPACITY, DragState, Geometry, Indicator, StripPlaces, ZONE_ALPHA,
};
use super::{DockGroupId, DockLayout, DockSplitId, DockView, PanelId};
use crate::split_view::{DividerPaint, KEY_STEP};
use crate::theme::ThemeHandle;
use crate::{Icon, Icons, Opacity, Padding, Row, ScrollController, ScrollView, Text};
use tantu_view::scene::RoundedRect;

type PanelFn<T> = Rc<dyn Fn(&PanelId) -> T>;

/// The hover and press layer's opacity.
const LAYER_ALPHA: f32 = 0.08;
/// A tab's horizontal padding.
const TAB_PADDING: f32 = 12.0;
/// The close icon's size.
const CLOSE_SIZE: f32 = 16.0;
/// The active tab's indicator thickness.
const INDICATOR: f32 = 2.0;
/// The visible gap between groups.
const GAP: f32 = 1.0;
/// The groups' minimum on every split axis.
const MIN_GROUP: f32 = 80.0;
/// The tab strips' height.
const STRIP_HEIGHT: f32 = 36.0;

/// Shows a dock layout and lets the user change it (WIDGETS-DOCKAREA-01..09).
///
/// ```
/// use tantu_view::ViewTree;
/// use tantu_view::reactive::signal;
/// use tantu_widgets::{DockArea, DockLayout, DockNode, PanelId, Text};
///
/// let tree = ViewTree::new(|| {
///     let layout = signal(DockLayout::new(DockNode::row([
///         (0.3, DockNode::tabs(["files"])),
///         (0.7, DockNode::tabs(["editor", "preview"])),
///     ])));
///     DockArea::new(layout, |id: &PanelId| Text::new(format!("Panel {}", id.as_str())))
/// });
/// ```
pub struct DockArea {
    layout: Signal<DockLayout>,
    content: PanelFn<AnyView>,
    title: PanelFn<String>,
    closable: PanelFn<bool>,
    on_close_request: PanelFn<bool>,
    active_group: Option<Signal<Option<DockGroupId>>>,
    placeholder: Option<Rc<dyn Fn() -> AnyView>>,
}

impl DockArea {
    /// A dock area showing `layout`; `content` builds a panel's view the first time it
    /// appears in the layout.
    pub fn new<V: View>(
        layout: Signal<DockLayout>,
        content: impl Fn(&PanelId) -> V + 'static,
    ) -> Self {
        DockArea {
            layout,
            content: Rc::new(move |id| AnyView::new(content(id))),
            title: Rc::new(|id| id.as_str().to_owned()),
            closable: Rc::new(|_| true),
            on_close_request: Rc::new(|_| true),
            active_group: None,
            placeholder: None,
        }
    }

    /// A tab's title (default: the panel id). Signals read inside are tracked.
    pub fn title(mut self, title: impl Fn(&PanelId) -> String + 'static) -> Self {
        self.title = Rc::new(title);
        self
    }

    /// Whether a panel shows a close button and can be closed (default: all).
    pub fn closable(mut self, closable: impl Fn(&PanelId) -> bool + 'static) -> Self {
        self.closable = Rc::new(closable);
        self
    }

    /// Asked before the user closes a panel; false keeps it. Default: close.
    pub fn on_close_request(mut self, f: impl Fn(&PanelId) -> bool + 'static) -> Self {
        self.on_close_request = Rc::new(f);
        self
    }

    /// Shares the active group (the one the user last worked in) with the app.
    pub fn active_group(mut self, group: Signal<Option<DockGroupId>>) -> Self {
        self.active_group = Some(group);
        self
    }

    /// What to show when the layout is empty (default: nothing).
    pub fn placeholder<V: View>(mut self, view: impl Fn() -> V + 'static) -> Self {
        self.placeholder = Some(Rc::new(move || AnyView::new(view())));
        self
    }
}

/// What every part of the area shares.
struct Dock {
    layout: Signal<DockLayout>,
    content: PanelFn<AnyView>,
    title: PanelFn<String>,
    closable: PanelFn<bool>,
    on_close_request: PanelFn<bool>,
    active_group: Signal<Option<DockGroupId>>,
    /// Stable layout keys for panels (DockSlot::Panel).
    keys: Keys,
    /// The plan built from the layout.
    plan: Memo<Arc<DockPlan>>,
    /// Each plan node's rect from the last layout, recorded at paint.
    rects: Signal<Vec<Rect>>,
    /// Each group's strip element, for moving focus to it.
    strips: RefCell<HashMap<DockGroupId, ElementId>>,
    /// Each group's tab places and scroll controller, for drop targets.
    places: RefCell<HashMap<DockGroupId, StripPlaceSource>>,
    /// A strip to focus as soon as it is built (after a drop into a new group).
    focus_request: Cell<Option<DockGroupId>>,
    /// Where the last press landed (dock coordinates), seen by the root first.
    down_at: Cell<Point>,
    /// The tab pressed with the primary button, and where (dock coordinates).
    pressed: RefCell<Option<(PanelId, Point)>>,
    /// The drag in progress (WIDGETS-DOCKDRAG-01).
    drag: Signal<Option<DragState>>,
}

/// A strip's published tab places and its scroll controller.
type StripPlaceSource = (Signal<Vec<(f32, f32)>>, ScrollController);

/// Stable layout keys for panels, shared by the plan and the panel hosts.
type Keys = Rc<RefCell<HashMap<PanelId, u64>>>;

/// `panel`'s key, assigned on first sight.
fn panel_key(keys: &Keys, panel: &PanelId) -> u64 {
    let mut keys = keys.borrow_mut();
    let next = keys.len() as u64 + 1;
    *keys.entry(panel.clone()).or_insert(next)
}

/// The plan for `layout`: its tree with group, split and panel keys.
fn build_plan(keys: &Keys, layout: &DockLayout) -> DockPlan {
    fn walk(keys: &Keys, view: &DockView<'_>, nodes: &mut Vec<DockPlanNode>) -> usize {
        let index = nodes.len();
        match view {
            DockView::Tabs { id, panels, .. } => nodes.push(DockPlanNode::Group {
                key: id.0,
                panels: panels.iter().map(|p| panel_key(keys, p)).collect(),
            }),
            DockView::Split {
                id,
                axis,
                fractions,
                children,
            } => {
                nodes.push(DockPlanNode::Split {
                    key: id.0,
                    axis: *axis,
                    fractions: fractions.to_vec(),
                    children: Vec::new(),
                });
                let mut indices = Vec::new();
                for child in children {
                    indices.push(walk(keys, child, nodes));
                }
                if let DockPlanNode::Split { children, .. } = &mut nodes[index] {
                    *children = indices;
                }
            }
        }
        index
    }
    let mut nodes = Vec::new();
    if let Some(root) = layout.root() {
        walk(keys, &root, &mut nodes);
    }
    DockPlan { nodes }
}

impl Dock {
    fn key(&self, panel: &PanelId) -> u64 {
        panel_key(&self.keys, panel)
    }

    /// Recomputes the drag's target for the pointer at `at` (dock coordinates).
    fn update_drag(&self, at: Point) {
        let Some(panel) = self
            .drag
            .with_untracked(|d| d.as_ref().map(|d| d.panel.clone()))
        else {
            return;
        };
        let target = untrack(|| {
            let places = |group: DockGroupId| {
                let places = self.places.borrow();
                let (slots, controller) = places.get(&group)?;
                Some(StripPlaces {
                    slots: slots.get_untracked(),
                    offset: controller.offset_untracked(),
                })
            };
            let rects = self.rects.get_untracked();
            let plan = self.plan.get_untracked();
            self.layout.with_untracked(|layout| {
                Geometry {
                    layout,
                    plan: &plan,
                    rects: &rects,
                    strip_height: STRIP_HEIGHT,
                    gap: GAP,
                    places: &places,
                }
                .target(&panel, at)
            })
        });
        let state = Some(DragState { panel, target });
        if self.drag.get_untracked() != state {
            self.drag.set(state);
        }
    }

    /// Moves `panel` to `target`, then activates its group and focuses its strip
    /// (WIDGETS-DOCKDRAG-06).
    fn drop_panel(
        &self,
        panel: &PanelId,
        target: DockTarget,
        focus: &FocusTracker,
        moves: Signal<u32>,
    ) {
        self.layout.update(|l| {
            let _ = l.move_panel(panel, target);
        });
        let group = untrack(|| self.layout.with(|l| l.group_of(panel)));
        self.set_active_group(group);
        let Some(group) = group else {
            return;
        };
        let strip = self.strips.borrow().get(&group).copied();
        match strip {
            Some(strip) => {
                focus.target.set(Some(strip));
                moves.update(|m| *m = m.wrapping_add(1));
            }
            None => self.focus_request.set(Some(group)),
        }
    }

    /// Activates `panel` in the layout.
    fn activate(&self, panel: &PanelId) {
        let active = untrack(|| {
            self.layout
                .with(|l| l.group_of(panel).and_then(|g| l.active(g)) == Some(panel))
        });
        if !active {
            self.layout.update(|l| {
                let _ = l.activate(panel);
            });
        }
    }

    /// Asks the app, then closes `panel` (WIDGETS-DOCKAREA-03).
    fn request_close(&self, panel: &PanelId) {
        if (self.closable)(panel) && (self.on_close_request)(panel) {
            self.layout.update(|l| {
                let _ = l.close(panel);
            });
        }
    }

    fn set_active_group(&self, group: Option<DockGroupId>) {
        if group.is_some() && self.active_group.get_untracked() != group {
            self.active_group.set(group);
        }
    }

    /// The active group's panels and active index.
    fn active_tabs(&self) -> Option<(Vec<PanelId>, usize)> {
        let group = self.active_group.get_untracked()?;
        untrack(|| {
            self.layout.with(|l| {
                l.group(group)
                    .map(|(panels, active)| (panels.to_vec(), active))
            })
        })
    }

    /// Activates the active group's next (or previous) tab, wrapping (WIDGETS-DOCKAREA-05).
    fn cycle(&self, forward: bool) {
        if let Some((panels, active)) = self.active_tabs() {
            let n = panels.len();
            let next = if forward {
                (active + 1) % n
            } else {
                (active + n - 1) % n
            };
            self.activate(&panels[next]);
        }
    }
}

impl View for DockArea {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let layout = self.layout;
        let active_group = self.active_group.unwrap_or_else(|| signal(None));
        let keys: Keys = Rc::default();
        let plan = {
            let keys = Rc::clone(&keys);
            memo(move || Arc::new(layout.with(|l| build_plan(&keys, l))))
        };
        let dock = Rc::new(Dock {
            layout,
            content: self.content,
            title: self.title,
            closable: self.closable,
            on_close_request: self.on_close_request,
            active_group,
            keys,
            plan,
            rects: signal(Vec::new()),
            strips: RefCell::default(),
            places: RefCell::default(),
            focus_request: Cell::new(None),
            down_at: Cell::new(Point::new(0.0, 0.0)),
            pressed: RefCell::new(None),
            drag: signal(None),
        });
        // The active group falls back to the first group (WIDGETS-DOCKAREA-04).
        effect(move || {
            let groups = layout.with(|l| l.groups());
            let current = active_group.get_untracked();
            if current.is_none_or(|g| !groups.contains(&g)) {
                let first = groups.first().copied();
                if first != current {
                    active_group.set(first);
                }
            }
        });
        // Focus leaving hidden or closed panels (WIDGETS-DOCKAREA-07).
        let focus = FocusTracker::default();
        let moves = signal(0u32);
        {
            let (dock, focus) = (Rc::clone(&dock), focus.clone());
            effect(move || {
                let still_shown = layout.with(|l| {
                    let panel = focus.panel.borrow().clone()?;
                    let group = l.group_of(&panel);
                    Some((group, group.and_then(|g| l.active(g)) == Some(&panel)))
                });
                let Some((group, shown)) = still_shown else {
                    return;
                };
                if shown {
                    focus.group.set(group);
                    return;
                }
                // Hidden or closed: the strip of its group (if the group remains) takes it.
                let target = group.or(focus.group.get());
                let alive =
                    untrack(|| layout.with(|l| target.is_some_and(|g| l.group(g).is_some())));
                *focus.panel.borrow_mut() = None;
                if alive {
                    let strip = target.and_then(|g| dock.strips.borrow().get(&g).copied());
                    focus.target.set(strip);
                    untrack(|| moves.update(|m| *m = m.wrapping_add(1)));
                }
            });
        }
        let mut children: Vec<AnyView> = Vec::new();
        {
            let dock = Rc::clone(&dock);
            children.push(AnyView::new(For::new(
                move || layout.with(|l| l.groups()),
                |g: &DockGroupId| *g,
                move |group| StripView {
                    dock: Rc::clone(&dock),
                    group,
                },
            )));
        }
        {
            let (dock, focus) = (Rc::clone(&dock), focus.clone());
            children.push(AnyView::new(For::new(
                move || layout.with(|l| l.panels()),
                |p: &PanelId| p.clone(),
                move |panel| PanelHost {
                    dock: Rc::clone(&dock),
                    panel,
                    focus: focus.clone(),
                },
            )));
        }
        if let Some(placeholder) = self.placeholder {
            children.push(AnyView::new(PlaceholderHost {
                layout,
                placeholder,
            }));
        }
        {
            let dock = Rc::clone(&dock);
            children.push(AnyView::new(For::new(
                move || {
                    plan.with(|plan| {
                        let mut out = Vec::new();
                        for node in &plan.nodes {
                            if let DockPlanNode::Split { key, children, .. } = node {
                                for index in 0..children.len().saturating_sub(1) {
                                    out.push((*key, index));
                                }
                            }
                        }
                        out
                    })
                },
                |d: &(u64, usize)| *d,
                move |(split, index)| DividerView {
                    dock: Rc::clone(&dock),
                    split,
                    index,
                },
            )));
        }
        {
            let focus = focus.clone();
            children.push(AnyView::new(Dyn::new(move || {
                moves.get();
                AnyView::new(FocusMove {
                    target: focus.target.clone(),
                })
            })));
        }
        let id = cx.render(RenderDock::new(plan.get_untracked()), children);
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || plan.get())),
            |e, plan| {
                e.update_render::<RenderDock>(|r| r.plan = plan);
            },
        );
        let theme = ThemeHandle::of(cx);
        cx.set_paint(
            id,
            DockPaint {
                rects: dock.rects,
                indicator: None,
                colors: theme.colors(),
            },
        );
        {
            let drag = dock.drag;
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    let indicator = drag.with(|d| d.as_ref().and_then(|d| d.target.map(|t| t.1)));
                    (indicator, theme.colors())
                })),
                |e, (indicator, colors)| {
                    e.update_paint::<DockPaint>(|p| {
                        p.indicator = indicator;
                        p.colors = colors;
                    });
                },
            );
        }
        // Dragging tabs (WIDGETS-DOCKDRAG-01..06): the root sees the captured path's events
        // first, in its own coordinates.
        {
            let (dock, focus) = (Rc::clone(&dock), focus.clone());
            cx.on_pointer(id, Phase::Capture, move |p| match p.event.kind {
                PointerKind::Down(_) => {
                    *dock.pressed.borrow_mut() = None;
                    dock.down_at.set(p.local);
                    Handled::Continue
                }
                PointerKind::Move => {
                    if dock.drag.with_untracked(|d| d.is_some()) {
                        dock.update_drag(p.local);
                        return Handled::Stop;
                    }
                    let start = dock.pressed.borrow().as_ref().map(|(_, start)| *start);
                    match start {
                        Some(start)
                            if (p.local.x - start.x).hypot(p.local.y - start.y) > DRAG_SLOP =>
                        {
                            let panel = dock.pressed.borrow().as_ref().map(|(p, _)| p.clone());
                            if let Some(panel) = panel {
                                dock.drag.set(Some(DragState {
                                    panel,
                                    target: None,
                                }));
                                dock.update_drag(p.local);
                            }
                            Handled::Stop
                        }
                        _ => Handled::Continue,
                    }
                }
                PointerKind::Up(PointerButton::Primary) => {
                    *dock.pressed.borrow_mut() = None;
                    let Some(drag) = dock.drag.get_untracked() else {
                        return Handled::Continue;
                    };
                    dock.drag.set(None);
                    if let Some((target, _)) = drag.target {
                        dock.drop_panel(&drag.panel, target, &focus, moves);
                    }
                    Handled::Stop
                }
                _ => Handled::Continue,
            });
        }
        {
            let dock = Rc::clone(&dock);
            cx.on_key(id, Phase::Capture, move |k| {
                if k.event.pressed
                    && k.event.key == LogicalKey::Named(NamedKey::Escape)
                    && dock.drag.with_untracked(|d| d.is_some())
                {
                    dock.drag.set(None);
                    *dock.pressed.borrow_mut() = None;
                    return Handled::Stop;
                }
                Handled::Continue
            });
        }
        // A dragged panel leaving the layout cancels the drag (WIDGETS-DOCKDRAG-06).
        {
            let drag = dock.drag;
            effect(move || {
                let gone = layout
                    .with(|l| drag.with(|d| d.as_ref().is_some_and(|d| !l.contains(&d.panel))));
                if gone {
                    drag.set(None);
                }
            });
        }
        // Shortcuts from anywhere inside (WIDGETS-DOCKAREA-05).
        for (key, shift, forward) in [
            (NamedKey::Tab, false, true),
            (NamedKey::Tab, true, false),
            (NamedKey::PageDown, false, true),
            (NamedKey::PageUp, false, false),
        ] {
            let mut activator = SingleActivator::new(LogicalKey::Named(key)).control();
            if shift {
                activator = activator.shift();
            }
            cx.shortcut(id, activator, CycleDock(forward));
        }
        cx.shortcut(
            id,
            SingleActivator::new(LogicalKey::Character("w".into())).control(),
            CloseDock,
        );
        cx.shortcut(
            id,
            SingleActivator::new(LogicalKey::Named(NamedKey::F(4))).control(),
            CloseDock,
        );
        {
            let dock = Rc::clone(&dock);
            cx.action::<CycleDock>(id, move |intent| dock.cycle(intent.0));
        }
        {
            let dock = Rc::clone(&dock);
            cx.action::<CloseDock>(id, move |_| {
                if let Some((panels, active)) = dock.active_tabs() {
                    dock.request_close(&panels[active]);
                }
            });
        }
        id
    }
}

/// Control+Tab and friends: the active group's next or previous tab.
#[derive(Clone)]
struct CycleDock(bool);

/// Control+W, Control+F4: close the active group's active panel.
#[derive(Clone)]
struct CloseDock;

/// Which panel holds the focus, and where focus should go when it hides.
#[derive(Clone, Default)]
struct FocusTracker {
    panel: Rc<RefCell<Option<PanelId>>>,
    group: Rc<Cell<Option<DockGroupId>>>,
    target: Rc<Cell<Option<ElementId>>>,
}

/// Focuses the pending target once, at a frame (WIDGETS-DOCKAREA-07).
struct FocusMove {
    target: Rc<Cell<Option<ElementId>>>,
}

impl View for FocusMove {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if let Some(target) = self.target.take() {
            cx.focus(target);
        }
        cx.region(|_| {})
    }
}

/// Records the node rects after layout, and draws the drop indicator over the children
/// (WIDGETS-DOCKDRAG-02..04).
struct DockPaint {
    rects: Signal<Vec<Rect>>,
    indicator: Option<Indicator>,
    colors: tantu_theme::ColorScheme,
}

impl Paint for DockPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        if let Some(dock) = cx.render::<RenderDock>() {
            if self.rects.get_untracked() != dock.node_rects() {
                self.rects.set(dock.node_rects().to_vec());
            }
        }
        cx.paint_children();
        let primary = self.colors.primary;
        match self.indicator {
            Some(Indicator::Line(rect)) => cx.scene().fill_rect(rect, primary),
            Some(Indicator::Zone(rect)) => {
                let scene = cx.scene();
                scene.fill_rect(rect, primary.with_alpha(ZONE_ALPHA));
                scene.stroke(RoundedRect::from_rect(rect), 2.0, primary);
            }
            None => {}
        }
    }
}

/// The empty layout's placeholder: offstage while there are panels (WIDGETS-DOCKAREA-09).
struct PlaceholderHost {
    layout: Signal<DockLayout>,
    placeholder: Rc<dyn Fn() -> AnyView>,
}

impl View for PlaceholderHost {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (layout, placeholder) = (self.layout, self.placeholder);
        let empty = memo(move || layout.with(|l| l.is_empty()));
        let content = Show::new(move || empty.get(), move || placeholder());
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [AnyView::new(content)],
        );
        cx.set_parent_data(id, DockSlot::Placeholder);
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || !empty.get())),
            |e, hide| {
                e.set_offstage(hide);
            },
        );
        id
    }
}

// ---------------------------------------------------------------------------------------
// Panels.

/// A panel's content, kept while the panel is in the layout (WIDGETS-DOCKAREA-01).
struct PanelHost {
    dock: Rc<Dock>,
    panel: PanelId,
    focus: FocusTracker,
}

impl View for PanelHost {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (dock, panel) = (self.dock, self.panel);
        let content = untrack(|| (dock.content)(&panel));
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [content],
        );
        cx.set_parent_data(id, DockSlot::Panel(dock.key(&panel)));
        let layout = dock.layout;
        let shown = {
            let panel = panel.clone();
            memo(move || {
                layout.with(|l| l.group_of(&panel).and_then(|g| l.active(g)) == Some(&panel))
            })
        };
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || !shown.get())),
            |e, hide| {
                e.set_offstage(hide);
            },
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::TabPanel,
                ..Default::default()
            },
        );
        {
            let (title, panel) = (Rc::clone(&dock.title), panel.clone());
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || title(&panel))),
                |e, name| {
                    e.update_semantics(|s| s.name = Some(name));
                },
            );
        }
        // A press inside makes the group active (WIDGETS-DOCKAREA-03, -04).
        {
            let (dock, panel) = (Rc::clone(&dock), panel.clone());
            cx.on_pointer(id, Phase::Capture, move |p| {
                if let PointerKind::Down(_) = p.event.kind {
                    let group = untrack(|| layout.with(|l| l.group_of(&panel)));
                    dock.set_active_group(group);
                }
                Handled::Continue
            });
        }
        {
            let (dock, focus) = (Rc::clone(&dock), self.focus);
            cx.on_focus_within(id, move |inside| {
                if inside {
                    let group = untrack(|| layout.with(|l| l.group_of(&panel)));
                    *focus.panel.borrow_mut() = Some(panel.clone());
                    focus.group.set(group);
                    dock.set_active_group(group);
                } else if focus.panel.borrow().as_ref() == Some(&panel) {
                    *focus.panel.borrow_mut() = None;
                }
            });
        }
        id
    }
}

// ---------------------------------------------------------------------------------------
// Strips and tabs.

/// A tab's place in its strip's row after the last layout.
type Slots = Rc<RefCell<Vec<(f32, f32)>>>;

/// Publishes the row's tab places after each layout, at paint, for revealing the active tab.
struct RowPaint {
    slots: Slots,
    published: Signal<Vec<(f32, f32)>>,
}

impl Paint for RowPaint {
    fn paint(&self, _cx: &mut PaintCx<'_, '_>) {
        let slots = self.slots.borrow();
        if self.published.get_untracked() != *slots {
            self.published.set(slots.clone());
        }
    }
}

/// A group's tab strip: the `TabList` focus stop, scrolling its tabs (WIDGETS-DOCKAREA-02).
struct StripView {
    dock: Rc<Dock>,
    group: DockGroupId,
}

impl View for StripView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (dock, group) = (self.dock, self.group);
        let layout = dock.layout;
        let theme = ThemeHandle::of(cx);
        let controller = ScrollController::new();
        let slots: Slots = Rc::default();
        let published = signal(Vec::new());
        let row = StripRow {
            dock: Rc::clone(&dock),
            group,
            slots,
            published,
        };
        let scroller = ScrollView::new(row)
            .horizontal()
            .controller(controller)
            .without_scrollbar()
            .unfocusable();
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [AnyView::new(scroller)],
        );
        cx.set_parent_data(id, DockSlot::Strip(group.0));
        dock.strips.borrow_mut().insert(group, id);
        dock.places
            .borrow_mut()
            .insert(group, (published, controller));
        {
            let dock = Rc::clone(&dock);
            on_cleanup(move || {
                let mut strips = dock.strips.borrow_mut();
                if strips.get(&group) == Some(&id) {
                    strips.remove(&group);
                    dock.places.borrow_mut().remove(&group);
                }
            });
        }
        cx.set_paint(
            id,
            StripPaint {
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<StripPaint>(|p| p.colors = colors);
        });
        cx.set_semantics(
            id,
            Semantics {
                role: Role::TabList,
                focusable: true,
                actions: SemanticsActions {
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        cx.focusable(
            id,
            FocusOptions {
                traversable: true,
                focus_on_press: true,
            },
        );
        // A drop into a new group focuses its strip once it exists (WIDGETS-DOCKDRAG-06).
        if dock.focus_request.get() == Some(group) {
            dock.focus_request.set(None);
            cx.focus(id);
        }
        {
            let dock = Rc::clone(&dock);
            cx.on_focus_within(id, move |inside| {
                if inside {
                    dock.set_active_group(Some(group));
                }
            });
        }
        // A press anywhere in the strip makes the group active (WIDGETS-DOCKAREA-04).
        {
            let dock = Rc::clone(&dock);
            cx.on_pointer(id, Phase::Capture, move |p| match p.event.kind {
                PointerKind::Down(_) => {
                    dock.set_active_group(Some(group));
                    Handled::Continue
                }
                // A vertical wheel scrolls the strip horizontally.
                PointerKind::Scroll(delta) if delta.y.abs() > delta.x.abs() => {
                    if controller.wheel(delta.y, Duration::from_millis(150)) {
                        Handled::Stop
                    } else {
                        Handled::Continue
                    }
                }
                _ => Handled::Continue,
            });
        }
        // Arrows, Home and End on the focused strip (WIDGETS-DOCKAREA-05).
        {
            let dock = Rc::clone(&dock);
            cx.on_key(id, Phase::Capture, move |k| {
                let event = &k.event;
                let Some((panels, active)) =
                    untrack(|| layout.with(|l| l.group(group).map(|(p, a)| (p.to_vec(), a))))
                else {
                    return Handled::Continue;
                };
                // Control+Shift+Left/Right move the active tab (WIDGETS-DOCKDRAG-07).
                let chord = tantu_view::Modifiers {
                    control: true,
                    shift: true,
                    ..Default::default()
                };
                if event.pressed && event.modifiers == chord {
                    let index = match &event.key {
                        LogicalKey::Named(NamedKey::ArrowLeft) if active > 0 => active - 1,
                        LogicalKey::Named(NamedKey::ArrowRight) if active + 1 < panels.len() => {
                            active + 1
                        }
                        _ => return Handled::Continue,
                    };
                    let panel = panels[active].clone();
                    layout.update(|l| {
                        let _ = l.move_panel(&panel, DockTarget::Tabs { group, index });
                    });
                    return Handled::Stop;
                }
                if !event.pressed || event.modifiers != Default::default() {
                    return Handled::Continue;
                }
                let n = panels.len();
                let next = match &event.key {
                    LogicalKey::Named(NamedKey::ArrowLeft) => (active + n - 1) % n,
                    LogicalKey::Named(NamedKey::ArrowRight) => (active + 1) % n,
                    LogicalKey::Named(NamedKey::Home) => 0,
                    LogicalKey::Named(NamedKey::End) => n - 1,
                    _ => return Handled::Continue,
                };
                dock.activate(&panels[next]);
                Handled::Stop
            });
        }
        // The active tab is revealed whenever it changes.
        effect(move || {
            let active = layout.with(|l| l.group(group).map(|(_, a)| a));
            let places = published.get();
            untrack(|| {
                let Some((x, width)) = active.and_then(|a| places.get(a).copied()) else {
                    return;
                };
                let viewport = controller.viewport_extent_untracked();
                let offset = controller.offset_untracked();
                if viewport <= 0.0 {
                    return;
                }
                if x < offset || width > viewport {
                    controller.jump_to(x);
                } else if x + width > offset + viewport {
                    controller.jump_to(x + width - viewport);
                }
            });
        });
        id
    }
}

/// The strip's background (WIDGETS-DOCKAREA-02).
struct StripPaint {
    colors: tantu_theme::ColorScheme,
}

impl Paint for StripPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().fill_rect(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            self.colors.surface_container_highest,
        );
    }
}

/// The row of tabs inside a strip's scroll view.
struct StripRow {
    dock: Rc<Dock>,
    group: DockGroupId,
    slots: Slots,
    published: Signal<Vec<(f32, f32)>>,
}

impl View for StripRow {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (dock, group) = (self.dock, self.group);
        let layout = dock.layout;
        let tabs = For::new(
            move || layout.with(|l| l.group(group).map(|(p, _)| p.to_vec()).unwrap_or_default()),
            |p: &PanelId| p.clone(),
            move |panel| DockTab {
                dock: Rc::clone(&dock),
                group,
                panel,
            },
        );
        let id = cx.render(
            RowRender {
                slots: Rc::clone(&self.slots),
            },
            [AnyView::new(tabs)],
        );
        cx.set_paint(
            id,
            RowPaint {
                slots: self.slots,
                published: self.published,
            },
        );
        id
    }
}

/// Tabs side by side at their natural widths, full height; records their places.
#[derive(Clone)]
struct RowRender {
    slots: Slots,
}

impl PartialEq for RowRender {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.slots, &other.slots)
    }
}

impl RenderBox for RowRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let height = if c.has_bounded_height() {
            c.max_height
        } else {
            0.0
        };
        let mut slots = self.slots.borrow_mut();
        slots.clear();
        let mut x = 0.0;
        for i in 0..children.len() {
            let width = children
                .layout(i, BoxConstraints::new(0.0, f32::INFINITY, height, height))
                .width;
            children.set_offset(i, Vec2::new(x, 0.0));
            slots.push((x, width));
            x += width;
        }
        c.constrain(Size::new(x, height))
    }
}

/// One tab: title and close button (WIDGETS-DOCKAREA-02, -03, -08).
struct DockTab {
    dock: Rc<Dock>,
    group: DockGroupId,
    panel: PanelId,
}

impl View for DockTab {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (dock, group, panel) = (self.dock, self.group, self.panel);
        let layout = dock.layout;
        let theme = ThemeHandle::of(cx);
        let active = {
            let panel = panel.clone();
            memo(move || layout.with(|l| l.active(group) == Some(&panel)))
        };
        let hover = signal(false);
        let title = {
            let (title, panel) = (Rc::clone(&dock.title), panel.clone());
            move || title(&panel)
        };
        let closable = (dock.closable)(&panel);
        let mut row = Row::new()
            .main_axis_size(MainAxisSize::Min)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(4.0)
            .child(
                Text::new(title.clone())
                    .style(theme.snapshot().text.label)
                    .color(move || {
                        let colors = theme.colors();
                        if active.get() {
                            colors.on_surface
                        } else {
                            colors.on_surface_variant
                        }
                    }),
            );
        if closable {
            row = row.child(CloseButton {
                dock: Rc::clone(&dock),
                panel: panel.clone(),
                visible: Rc::new(move || active.get() || hover.get()),
            });
        }
        // The dragged tab is dimmed (WIDGETS-DOCKDRAG-01).
        let dragged = {
            let (drag, panel) = (dock.drag, panel.clone());
            memo(move || drag.with(|d| d.as_ref().is_some_and(|d| d.panel == panel)))
        };
        let content = Opacity::new(
            move || if dragged.get() { DRAGGED_OPACITY } else { 1.0 },
            Padding::new(EdgeInsets::symmetric(TAB_PADDING, 0.0)).child(row),
        );
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [AnyView::new(content)],
        );
        let primary_here = {
            let active_group = dock.active_group;
            memo(move || active_group.get() == Some(group))
        };
        cx.set_paint(
            id,
            TabPaint {
                active: false,
                hover: false,
                current_group: false,
                colors: theme.colors(),
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (
                    active.get(),
                    hover.get(),
                    primary_here.get(),
                    theme.colors(),
                )
            })),
            |e, (active, hover, current_group, colors)| {
                e.update_paint::<TabPaint>(|p| {
                    p.active = active;
                    p.hover = hover;
                    p.current_group = current_group;
                    p.colors = colors;
                });
            },
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Tab,
                actions: SemanticsActions {
                    click: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        {
            let panel = panel.clone();
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    let position = layout.with(|l| {
                        l.group(group).map(|(panels, _)| {
                            (
                                panels.iter().position(|p| *p == panel).map_or(0, |i| i + 1),
                                panels.len(),
                            )
                        })
                    });
                    (title(), active.get(), position)
                })),
                |e, (name, selected, position)| {
                    e.update_semantics(|s| {
                        s.name = Some(name);
                        s.selected = Some(selected);
                        s.position_in_set = position.map(|p| p.0);
                        s.size_of_set = position.map(|p| p.1);
                    });
                },
            );
        }
        cx.focusable(
            id,
            FocusOptions {
                traversable: false,
                focus_on_press: false,
            },
        );
        // Presses (WIDGETS-DOCKAREA-03).
        let pressed = Cell::new(None);
        cx.on_pointer(id, Phase::Bubble, move |p| {
            let inside = p.local.x >= 0.0
                && p.local.y >= 0.0
                && p.local.x <= p.size.width
                && p.local.y <= p.size.height;
            match p.event.kind {
                PointerKind::Down(button) => {
                    pressed.set(Some(button));
                    // A primary press may become a drag (WIDGETS-DOCKDRAG-01).
                    if button == PointerButton::Primary {
                        *dock.pressed.borrow_mut() = Some((panel.clone(), dock.down_at.get()));
                    }
                }
                PointerKind::Up(button) => {
                    if pressed.replace(None) == Some(button) && inside {
                        match button {
                            PointerButton::Primary => dock.activate(&panel),
                            PointerButton::Middle if closable => dock.request_close(&panel),
                            _ => {}
                        }
                    }
                }
                PointerKind::Enter => hover.set(true),
                PointerKind::Leave => hover.set(false),
                _ => {}
            }
            Handled::Continue
        });
        id
    }
}

/// A tab's background, indicator and hover layer (WIDGETS-DOCKAREA-02).
struct TabPaint {
    active: bool,
    hover: bool,
    current_group: bool,
    colors: tantu_theme::ColorScheme,
}

impl Paint for TabPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let bounds = Rect::from_ltrb(0.0, 0.0, size.width, size.height);
        let scene = cx.scene();
        if self.active {
            scene.fill_rect(bounds, self.colors.surface);
        }
        if self.hover {
            scene.fill_rect(bounds, self.colors.on_surface.with_alpha(LAYER_ALPHA));
        }
        if self.active {
            let color = if self.current_group {
                self.colors.primary
            } else {
                self.colors.outline
            };
            scene.fill_rect(
                Rect::from_ltrb(0.0, size.height - INDICATOR, size.width, size.height),
                color,
            );
        }
    }
}

/// A tab's close button (WIDGETS-DOCKAREA-02, -03, -08).
struct CloseButton {
    dock: Rc<Dock>,
    panel: PanelId,
    visible: Rc<dyn Fn() -> bool>,
}

impl View for CloseButton {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (dock, panel, visible) = (self.dock, self.panel, self.visible);
        let theme = ThemeHandle::of(cx);
        let icon = {
            let visible = Rc::clone(&visible);
            Icon::new(Icons::CLOSE).size(CLOSE_SIZE).color(move || {
                if visible() {
                    theme.colors().on_surface_variant
                } else {
                    tantu_view::core::Color::TRANSPARENT
                }
            })
        };
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(CLOSE_SIZE, CLOSE_SIZE))),
            [AnyView::new(icon)],
        );
        let hover = signal(false);
        cx.set_paint(
            id,
            CloseLayer {
                hover: false,
                colors: theme.colors(),
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || (hover.get(), theme.colors()))),
            |e, (hover, colors)| {
                e.update_paint::<CloseLayer>(|p| {
                    p.hover = hover;
                    p.colors = colors;
                });
            },
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Button,
                actions: SemanticsActions {
                    click: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        {
            let (title, panel) = (Rc::clone(&dock.title), panel.clone());
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || format!("Close {}", title(&panel)))),
                |e, name| {
                    e.update_semantics(|s| s.name = Some(name));
                },
            );
        }
        let pressed = Cell::new(false);
        cx.on_pointer(id, Phase::Bubble, move |p| match p.event.kind {
            PointerKind::Down(PointerButton::Primary) => {
                pressed.set(true);
                Handled::Stop
            }
            PointerKind::Up(PointerButton::Primary) => {
                let inside = p.local.x >= 0.0
                    && p.local.y >= 0.0
                    && p.local.x <= p.size.width
                    && p.local.y <= p.size.height;
                if pressed.replace(false) && inside {
                    dock.request_close(&panel);
                }
                Handled::Stop
            }
            PointerKind::Enter => {
                hover.set(true);
                Handled::Continue
            }
            PointerKind::Leave => {
                hover.set(false);
                Handled::Continue
            }
            _ => Handled::Continue,
        });
        id
    }
}

/// The close button's hover layer.
struct CloseLayer {
    hover: bool,
    colors: tantu_theme::ColorScheme,
}

impl Paint for CloseLayer {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        if self.hover {
            let size = cx.size();
            cx.scene().fill_rect(
                Rect::from_ltrb(0.0, 0.0, size.width, size.height),
                self.colors.on_surface.with_alpha(LAYER_ALPHA),
            );
        }
    }
}

// ---------------------------------------------------------------------------------------
// Dividers.

/// A drag in progress: the pointer's position on the axis and the children's sizes.
type Press = (f32, Vec<f32>);

/// The divider between children `index` and `index + 1` of a split (WIDGETS-DOCKAREA-06).
struct DividerView {
    dock: Rc<Dock>,
    split: u64,
    index: usize,
}

impl DividerView {
    /// The split's axis and its children's current sizes along it.
    fn sizes(dock: &Dock, split: u64) -> Option<(Axis, Vec<f32>)> {
        let rects = dock.rects.get();
        dock.plan.with(|plan| {
            plan.nodes.iter().enumerate().find_map(|(node, n)| match n {
                DockPlanNode::Split {
                    key,
                    axis,
                    children,
                    ..
                } if *key == split => {
                    let sizes = children
                        .iter()
                        .filter(|c| **c > node)
                        .map(|c| {
                            rects.get(*c).map_or(0.0, |r| match axis {
                                Axis::Horizontal => r.width(),
                                Axis::Vertical => r.height(),
                            })
                        })
                        .collect();
                    Some((*axis, sizes))
                }
                _ => None,
            })
        })
    }

    /// Moves boundary `index` so child `index` becomes `candidate`, clamped at the group
    /// minimum, and writes the split's fractions. True if anything changed.
    fn move_to(dock: &Dock, split: u64, index: usize, start: &[f32], candidate: f32) -> bool {
        if index + 1 >= start.len() {
            return false;
        }
        let sum = start[index] + start[index + 1];
        let (lo, hi) = (MIN_GROUP, sum - MIN_GROUP);
        if hi < lo {
            return false;
        }
        let a = candidate.clamp(lo, hi);
        if (a - start[index]).abs() < 1e-3 {
            return false;
        }
        let mut sizes = start.to_vec();
        sizes[index] = a;
        sizes[index + 1] = sum - a;
        let total: f32 = sizes.iter().sum();
        if total <= 0.0 {
            return false;
        }
        let fractions: Vec<f32> = sizes.iter().map(|s| s / total).collect();
        dock.layout.update(|l| {
            let _ = l.set_fractions(DockSplitId(split), &fractions);
        });
        true
    }

    /// "Resize <title>", after the active panel of the first group before the boundary.
    fn label(dock: &Dock, split: u64, index: usize) -> String {
        fn first_group<'a>(view: &'a DockView<'a>) -> Option<(&'a [PanelId], usize)> {
            match view {
                DockView::Tabs { panels, active, .. } => Some((panels, *active)),
                DockView::Split { children, .. } => children.first().and_then(first_group),
            }
        }
        fn find<'a>(view: &'a DockView<'a>, split: u64) -> Option<&'a DockView<'a>> {
            match view {
                DockView::Split { id, .. } if id.0 == split => Some(view),
                DockView::Split { children, .. } => children.iter().find_map(|c| find(c, split)),
                DockView::Tabs { .. } => None,
            }
        }
        dock.layout
            .with(|l| {
                let root = l.root()?;
                let DockView::Split { children, .. } = find(&root, split)? else {
                    return None;
                };
                let (panels, active) = first_group(children.get(index)?)?;
                panels
                    .get(active)
                    .map(|p| format!("Resize {}", (dock.title)(p)))
            })
            .unwrap_or_else(|| "Resize".to_owned())
    }
}

impl View for DividerView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (dock, split, index) = (self.dock, self.split, self.index);
        let theme = ThemeHandle::of(cx);
        let axis = untrack(|| DividerView::sizes(&dock, split)).map_or(Axis::Horizontal, |s| s.0);
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.set_parent_data(id, DockSlot::Divider { split, index });
        let (hovered, dragging, focused) = (signal(false), signal(false), signal(false));
        cx.set_paint(
            id,
            DividerPaint {
                axis,
                thickness: GAP,
                hovered: false,
                focused: false,
                colors: theme.colors(),
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (
                    hovered.get() || dragging.get(),
                    focused.get(),
                    theme.colors(),
                )
            })),
            |e, (hovered, focused, colors)| {
                e.update_paint::<DividerPaint>(|p| {
                    p.hovered = hovered;
                    p.focused = focused;
                    p.colors = colors;
                });
            },
        );
        cx.on_focus_change(id, move |is| focused.set(is));
        cx.set_cursor(
            id,
            match axis {
                Axis::Horizontal => CursorIcon::ResizeHorizontal,
                Axis::Vertical => CursorIcon::ResizeVertical,
            },
        );
        cx.focusable(
            id,
            FocusOptions {
                traversable: true,
                focus_on_press: false,
            },
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Splitter,
                focusable: true,
                actions: SemanticsActions {
                    focus: true,
                    increment: true,
                    decrement: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        {
            let dock = Rc::clone(&dock);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    let numeric = DividerView::sizes(&dock, split).map(|(_, sizes)| {
                        let total: f32 = sizes.iter().sum();
                        if total > 0.0 {
                            let before: f32 = sizes.iter().take(index + 1).sum();
                            (
                                f64::from(before / total * 100.0),
                                f64::from(KEY_STEP / total * 100.0),
                            )
                        } else {
                            (0.0, 0.0)
                        }
                    });
                    (DividerView::label(&dock, split, index), numeric)
                })),
                |e, (name, numeric)| {
                    e.update_semantics(|s| {
                        s.name = Some(name);
                        s.numeric = numeric.map(|(value, step)| NumericValue {
                            value,
                            min: 0.0,
                            max: 100.0,
                            step,
                        });
                        s.value = numeric.map(|(value, _)| format!("{value:.0} %"));
                    });
                },
            );
        }
        let press: Rc<RefCell<Option<Press>>> = Rc::default();
        {
            let dock = Rc::clone(&dock);
            let main = move |p: Point| match axis {
                Axis::Horizontal => p.x,
                Axis::Vertical => p.y,
            };
            cx.on_pointer(id, Phase::Bubble, move |p| match p.event.kind {
                PointerKind::Enter => {
                    hovered.set(true);
                    Handled::Continue
                }
                PointerKind::Leave => {
                    hovered.set(false);
                    Handled::Continue
                }
                PointerKind::Down(PointerButton::Primary) => {
                    let Some((_, sizes)) = untrack(|| DividerView::sizes(&dock, split)) else {
                        return Handled::Continue;
                    };
                    if sizes.len() <= index + 1 {
                        return Handled::Continue;
                    }
                    *press.borrow_mut() = Some((main(p.event.position), sizes));
                    dragging.set(true);
                    Handled::Stop
                }
                PointerKind::Move => {
                    let press = press.borrow();
                    let Some((start, sizes)) = press.as_ref() else {
                        return Handled::Continue;
                    };
                    let delta = main(p.event.position) - start;
                    DividerView::move_to(&dock, split, index, sizes, sizes[index] + delta);
                    Handled::Stop
                }
                PointerKind::Up(PointerButton::Primary) => {
                    if press.borrow_mut().take().is_none() {
                        return Handled::Continue;
                    }
                    dragging.set(false);
                    Handled::Stop
                }
                _ => Handled::Continue,
            });
        }
        cx.on_key(id, Phase::Bubble, move |k| {
            let event = &k.event;
            if !event.pressed || event.modifiers != Default::default() {
                return Handled::Continue;
            }
            let Some((_, sizes)) = untrack(|| DividerView::sizes(&dock, split)) else {
                return Handled::Continue;
            };
            if sizes.len() <= index + 1 {
                return Handled::Continue;
            }
            let sum = sizes[index] + sizes[index + 1];
            let candidate = match &event.key {
                LogicalKey::Named(NamedKey::ArrowLeft | NamedKey::ArrowUp) => {
                    sizes[index] - KEY_STEP
                }
                LogicalKey::Named(NamedKey::ArrowRight | NamedKey::ArrowDown) => {
                    sizes[index] + KEY_STEP
                }
                LogicalKey::Named(NamedKey::Home) => MIN_GROUP,
                LogicalKey::Named(NamedKey::End) => sum - MIN_GROUP,
                _ => return Handled::Continue,
            };
            if DividerView::move_to(&dock, split, index, &sizes, candidate) {
                Handled::Stop
            } else {
                Handled::Continue
            }
        });
        id
    }
}
