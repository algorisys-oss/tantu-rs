//! Tab bars and kept tab panels. Spec: `docs/specs/widgets/tab-view.md`.
use crate::theme::{ALPHA_38, ThemeHandle, at};
use crate::{Column, Expanded, Icon, IconData, ScrollController, ScrollView, Text};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use tantu_view::core::{Rect, Size, Vec2};
use tantu_view::layout::{
    BoxConstraints, CrossAxisAlignment, LayoutChildren, MainAxisSize, RenderBox,
    RenderConstrainedBox,
};
use tantu_view::reactive::{Memo, Signal, effect, memo, signal, untrack};
use tantu_view::scene::{BorderRadius, RoundedRect};
use tantu_view::{
    AnimationController, AnyView, BuildCx, Curve, Dyn, ElementId, FocusOptions, Handled, IntoProp,
    LogicalKey, NamedKey, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics,
    SemanticsActions, SingleActivator, View,
};

/// One tab's header: a label, an optional leading icon, enabled by default.
#[derive(Clone)]
pub struct Tab {
    label: String,
    icon: Option<IconData>,
    enabled: Rc<dyn Fn() -> bool>,
}
impl Tab {
    /// A tab labelled `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            enabled: Rc::new(|| true),
        }
    }
    /// An icon above the label (the bar becomes 64 px tall instead of 48 px).
    pub fn icon(mut self, icon: IconData) -> Self {
        self.icon = Some(icon);
        self
    }
    /// A disabled tab is shown dimmed, can't be selected by the user and is skipped by keys.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = match enabled.into_prop() {
            Prop::Value(value) => Rc::new(move || value),
            Prop::Dynamic(f) => Rc::from(f),
        };
        self
    }
}

/// The row of tab headers. The selected index is a shared signal.
pub struct TabBar {
    selected: Signal<usize>,
    tabs: Vec<Tab>,
    scrollable: bool,
}
impl TabBar {
    /// A bar showing `selected` (clamped to the tabs).
    pub fn new(selected: Signal<usize>) -> Self {
        Self {
            selected,
            tabs: Vec::new(),
            scrollable: false,
        }
    }
    /// Adds the next tab.
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }
    /// Adds several tabs.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = Tab>) -> Self {
        self.tabs.extend(tabs);
        self
    }
    /// Natural-width tabs in a horizontal scroll view instead of equal widths; default false.
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
}
impl View for TabBar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.build_with(cx, Rc::default())
    }
}

type PageBuilder = Rc<dyn Fn() -> AnyView>;

/// The pages; only the selected one is shown. Pages build on first selection.
pub struct TabBarView {
    selected: Signal<usize>,
    pages: Vec<PageBuilder>,
    keep_alive: bool,
}
impl TabBarView {
    /// Pages following `selected` (clamped to the pages).
    pub fn new(selected: Signal<usize>) -> Self {
        Self {
            selected,
            pages: Vec::new(),
            keep_alive: true,
        }
    }
    /// Adds the next page; `build` runs when the page is first selected.
    pub fn page<V: View>(mut self, build: impl Fn() -> V + 'static) -> Self {
        self.pages.push(Rc::new(move || AnyView::new(build())));
        self
    }
    /// Keep visited pages alive offstage (default true); false disposes a page on leaving.
    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }
}
impl View for TabBarView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        build_pages(self.selected, self.pages, Vec::new(), self.keep_alive, cx)
    }
}

/// A `TabBar` above an expanded `TabBarView`, with Control+Tab cycling across both.
pub struct TabView {
    selected: Option<Signal<usize>>,
    tabs: Vec<(Tab, PageBuilder)>,
    scrollable: bool,
    keep_alive: bool,
}
impl Default for TabView {
    fn default() -> Self {
        Self::new()
    }
}
impl TabView {
    /// Uses an owner-scoped selected signal starting at 0 unless `selected` shares one.
    pub fn new() -> Self {
        Self {
            selected: None,
            tabs: Vec::new(),
            scrollable: false,
            keep_alive: true,
        }
    }
    /// Shares the selected index.
    pub fn selected(mut self, selected: Signal<usize>) -> Self {
        self.selected = Some(selected);
        self
    }
    /// Adds a tab and its page builder.
    pub fn tab<V: View>(mut self, tab: Tab, build: impl Fn() -> V + 'static) -> Self {
        self.tabs
            .push((tab, Rc::new(move || AnyView::new(build()))));
        self
    }
    /// As [`TabBar::scrollable`].
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
    /// As [`TabBarView::keep_alive`].
    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }
}

impl View for TabView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let selected = self.selected.unwrap_or_else(|| signal(0));
        let (tabs, pages): (Vec<Tab>, Vec<PageBuilder>) = self.tabs.into_iter().unzip();
        let names: Vec<String> = tabs.iter().map(|t| t.label.clone()).collect();
        let state = BarState::new(selected, &tabs);
        let bar = TabBar {
            selected,
            tabs,
            scrollable: self.scrollable,
        };
        let target: Rc<Cell<Option<ElementId>>> = Rc::default();
        let pages_id: Rc<Cell<Option<ElementId>>> = Rc::default();
        let pending = Rc::new(Cell::new(false));
        let moves = signal(0u32);
        let keep_alive = self.keep_alive;
        let mover = (target.clone(), pending.clone());
        let outer = Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .child(BarView {
                bar,
                target: target.clone(),
            })
            .child(Expanded::new(PagesView {
                selected,
                pages,
                names,
                keep_alive,
                id: pages_id.clone(),
            }))
            .child(Dyn::new(move || {
                moves.get();
                AnyView::new(FocusMove {
                    target: mover.0.clone(),
                    pending: mover.1.clone(),
                })
            }))
            .build(cx);
        // Focus inside the leaving page moves to the bar (WIDGETS-TAB-09).
        if let Some(pages) = pages_id.get() {
            let in_pages = signal(false);
            cx.on_focus_within(pages, move |inside| in_pages.set(inside));
            let shown = state.shown;
            let first = Cell::new(true);
            effect(move || {
                shown.get();
                if !first.replace(false) && untrack(|| in_pages.get()) {
                    pending.set(true);
                    untrack(|| moves.update(|m| *m = m.wrapping_add(1)));
                }
            });
        }
        bind_cycling(cx, outer, state);
        outer
    }
}

// ---------------------------------------------------------------------------------------
// The bar.

/// What the bar's parts share: the selection and which tabs are enabled.
#[derive(Clone)]
struct BarState {
    selected: Signal<usize>,
    shown: Memo<usize>,
    enabled: Rc<[Rc<dyn Fn() -> bool>]>,
}
impl BarState {
    fn new(selected: Signal<usize>, tabs: &[Tab]) -> Self {
        let n = tabs.len();
        Self {
            selected,
            // The shown index is the signal clamped to the tabs (WIDGETS-TAB-01).
            shown: memo(move || selected.get().min(n.saturating_sub(1))),
            enabled: tabs.iter().map(|t| t.enabled.clone()).collect(),
        }
    }
    fn len(&self) -> usize {
        self.enabled.len()
    }
    fn is_enabled(&self, i: usize) -> bool {
        untrack(|| self.enabled.get(i).is_some_and(|e| e()))
    }
    fn select(&self, i: usize) {
        if self.selected.get_untracked() != i {
            self.selected.set(i);
        }
    }
    /// The next (or previous) enabled tab after the shown one, wrapping.
    fn step(&self, forward: bool) -> Option<usize> {
        let n = self.len();
        let current = self.shown.get_untracked();
        (1..=n)
            .map(|k| {
                if forward {
                    (current + k) % n
                } else {
                    (current + n - k % n) % n
                }
            })
            .find(|i| self.is_enabled(*i))
    }
    fn first(&self) -> Option<usize> {
        (0..self.len()).find(|i| self.is_enabled(*i))
    }
    fn last(&self) -> Option<usize> {
        (0..self.len()).rev().find(|i| self.is_enabled(*i))
    }
}

/// Control+Tab, Control+Shift+Tab, Control+PageDown and Control+PageUp (WIDGETS-TAB-07).
#[derive(Clone)]
struct CycleTab(bool);

fn bind_cycling(cx: &mut BuildCx<'_>, element: ElementId, state: BarState) {
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
        cx.shortcut(element, activator, CycleTab(forward));
    }
    cx.action::<CycleTab>(element, move |intent| {
        if let Some(i) = state.step(intent.0) {
            state.select(i);
        }
    });
}

/// A tab's place in the bar after the last layout.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Slot {
    x: f32,
    width: f32,
    content: f32,
}
type Slots = Rc<RefCell<Vec<Slot>>>;

/// Where the indicator moves from and to.
#[derive(Clone, Copy)]
struct Motion {
    from: Option<(f32, f32)>,
    to: usize,
}

/// The indicator's resting place under tab `i`: its content's width (at least 24 px).
fn rest(slots: &[Slot], i: usize) -> Option<(f32, f32)> {
    slots.get(i).map(|s| {
        let width = s.content.max(24.0).min(s.width);
        (s.x + (s.width - width) / 2.0, width)
    })
}

/// The indicator at controller value `t` (WIDGETS-TAB-04).
fn indicator(slots: &[Slot], motion: Motion, t: f64) -> Option<(f32, f32)> {
    let to = rest(slots, motion.to)?;
    Some(match motion.from {
        Some(from) => {
            let e = Curve::Ease.transform(t) as f32;
            (from.0 + (to.0 - from.0) * e, from.1 + (to.1 - from.1) * e)
        }
        None => to,
    })
}

impl TabBar {
    /// Builds the bar, storing its focusable element in `target`.
    fn build_with(self, cx: &mut BuildCx<'_>, target: Rc<Cell<Option<ElementId>>>) -> ElementId {
        let state = BarState::new(self.selected, &self.tabs);
        let controller = self.scrollable.then(ScrollController::new);
        let content = BarContent {
            tabs: self.tabs,
            state,
            controller,
            target,
        };
        match controller {
            Some(controller) => ScrollView::new(content)
                .horizontal()
                .controller(controller)
                .without_scrollbar()
                .build(cx),
            None => content.build(cx),
        }
    }
}

/// A `TabBar` that reports its focusable element (for `TabView`).
struct BarView {
    bar: TabBar,
    target: Rc<Cell<Option<ElementId>>>,
}
impl View for BarView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.bar.build_with(cx, self.target)
    }
}

/// The row of tabs: the `TabList` element, focusable, painting the indicator.
struct BarContent {
    tabs: Vec<Tab>,
    state: BarState,
    controller: Option<ScrollController>,
    target: Rc<Cell<Option<ElementId>>>,
}
impl View for BarContent {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let state = self.state;
        let shown = state.shown;
        let n = self.tabs.len();
        let height = if self.tabs.iter().any(|t| t.icon.is_some()) {
            64.0
        } else {
            48.0
        };
        let slots: Slots = Rc::default();
        let ids: Rc<RefCell<Vec<ElementId>>> = Rc::default();
        let children: Vec<AnyView> = self
            .tabs
            .into_iter()
            .enumerate()
            .map(|(index, tab)| {
                AnyView::new(TabHeader {
                    index,
                    tab,
                    state: state.clone(),
                    theme,
                    ids: ids.clone(),
                })
            })
            .collect();
        let id = cx.render(
            BarRender {
                scrollable: self.controller.is_some(),
                height,
                slots: slots.clone(),
            },
            children,
        );
        // The indicator slides between tabs (WIDGETS-TAB-04); the first build shows it in place.
        let anim = AnimationController::new(Duration::from_millis(300));
        anim.set_value(1.0);
        let motion = Rc::new(Cell::new(Motion {
            from: None,
            to: shown.get_untracked(),
        }));
        {
            let (slots, motion) = (slots.clone(), motion.clone());
            effect(move || {
                let to = shown.get();
                untrack(|| {
                    let m = motion.get();
                    if m.to != to {
                        let from = indicator(&slots.borrow(), m, anim.value());
                        motion.set(Motion { from, to });
                        anim.reset();
                        anim.forward();
                    }
                });
            });
        }
        let focused = signal(false);
        cx.set_paint(
            id,
            BarPaint {
                slots: slots.clone(),
                motion,
                anim,
                focused,
                shown,
                theme,
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (shown.get(), anim.value(), focused.get(), theme.colors())
            })),
            |e, _| {
                e.update_paint::<BarPaint>(|_| {});
            },
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::TabList,
                focusable: n > 0,
                actions: SemanticsActions {
                    focus: n > 0,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        if n == 0 {
            // An empty bar has no focus stop (WIDGETS-TAB-01).
            return id;
        }
        self.target.set(Some(id));
        cx.focusable(
            id,
            FocusOptions {
                traversable: true,
                focus_on_press: true,
            },
        );
        cx.on_focus_within(id, move |inside| focused.set(inside));
        {
            let ids = ids.clone();
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || (focused.get(), shown.get()))),
                move |e, (focused, shown)| {
                    let active = focused.then(|| ids.borrow().get(shown).copied()).flatten();
                    e.update_semantics(|s| s.active_descendant = active);
                },
            );
        }
        // Keys on the bar (WIDGETS-TAB-06): automatic activation, wrapping, disabled skipped.
        {
            let state = state.clone();
            cx.on_key(id, Phase::Capture, move |k| {
                let event = k.event;
                if !event.pressed || event.modifiers != Default::default() {
                    return Handled::Continue;
                }
                let target = match &event.key {
                    LogicalKey::Named(NamedKey::ArrowLeft) => state.step(false),
                    LogicalKey::Named(NamedKey::ArrowRight) => state.step(true),
                    LogicalKey::Named(NamedKey::Home) => state.first(),
                    LogicalKey::Named(NamedKey::End) => state.last(),
                    _ => return Handled::Continue,
                };
                if let Some(i) = target {
                    state.select(i);
                }
                Handled::Stop
            });
        }
        bind_cycling(cx, id, state);
        if let Some(controller) = self.controller {
            // A vertical wheel scrolls the bar horizontally (WIDGETS-TAB-02).
            cx.on_pointer(id, Phase::Capture, move |p| match p.event.kind {
                PointerKind::Scroll(delta) if delta.y.abs() > delta.x.abs() => {
                    if controller.wheel(delta.y, Duration::from_millis(150)) {
                        Handled::Stop
                    } else {
                        Handled::Continue
                    }
                }
                _ => Handled::Continue,
            });
            // Selecting a tab reveals it.
            effect(move || {
                let i = shown.get();
                untrack(|| {
                    let Some(slot) = slots.borrow().get(i).copied() else {
                        return;
                    };
                    let viewport = controller.viewport_extent_untracked();
                    let offset = controller.offset_untracked();
                    if viewport <= 0.0 {
                        return;
                    }
                    if slot.x < offset || slot.width > viewport {
                        controller.jump_to(slot.x);
                    } else if slot.x + slot.width > offset + viewport {
                        controller.jump_to(slot.x + slot.width - viewport);
                    }
                });
            });
        }
        id
    }
}

/// Lays the tabs out in a row: equal shares of a bounded width (at least each tab's natural
/// width), or natural widths (WIDGETS-TAB-02). Records each tab's place in `slots`.
#[derive(Clone)]
struct BarRender {
    scrollable: bool,
    height: f32,
    slots: Slots,
}
impl PartialEq for BarRender {
    fn eq(&self, other: &Self) -> bool {
        self.scrollable == other.scrollable
            && self.height == other.height
            && Rc::ptr_eq(&self.slots, &other.slots)
    }
}
impl RenderBox for BarRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let n = children.len();
        let mut slots = self.slots.borrow_mut();
        slots.clear();
        if n == 0 {
            return c.constrain(Size::ZERO);
        }
        let h = self.height;
        let fixed = !self.scrollable && c.has_bounded_width();
        let share = if fixed { c.max_width / n as f32 } else { 0.0 };
        let mut x = 0.0;
        for i in 0..n {
            let natural = children
                .layout(i, BoxConstraints::new(0.0, f32::INFINITY, h, h))
                .width;
            let width = natural.max(share);
            children.layout(i, BoxConstraints::tight(Size::new(width, h)));
            children.set_offset(i, Vec2::new(x, 0.0));
            slots.push(Slot {
                x,
                width,
                content: (natural - 2.0 * PADDING).max(0.0),
            });
            x += width;
        }
        if fixed && x > c.max_width + 0.5 {
            tracing::debug!(overflow = x - c.max_width, "TabBar overflows its width");
        }
        c.constrain(Size::new(if fixed { c.max_width } else { x }, h))
    }
}

/// A tab's horizontal padding on each side.
const PADDING: f32 = 16.0;

/// One tab: its content centered, the natural width being the content's plus the padding.
#[derive(Clone, PartialEq)]
struct TabRender;
impl RenderBox for TabRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let inner = if c.has_bounded_width() {
            (c.max_width - 2.0 * PADDING).max(0.0)
        } else {
            f32::INFINITY
        };
        let mut content = Size::ZERO;
        for i in 0..children.len() {
            content =
                content.max(children.layout(i, BoxConstraints::new(0.0, inner, 0.0, c.max_height)));
        }
        let size = c.constrain(Size::new(content.width + 2.0 * PADDING, c.max_height));
        for i in 0..children.len() {
            children.set_offset(
                i,
                Vec2::new(
                    ((size.width - content.width) / 2.0).max(0.0),
                    ((size.height - content.height) / 2.0).max(0.0),
                ),
            );
        }
        size
    }
}

struct TabHeader {
    index: usize,
    tab: Tab,
    state: BarState,
    theme: ThemeHandle,
    ids: Rc<RefCell<Vec<ElementId>>>,
}
impl View for TabHeader {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (i, theme, state) = (self.index, self.theme, self.state);
        let shown = state.shown;
        let enabled = self.tab.enabled.clone();
        let color = {
            let enabled = enabled.clone();
            move || {
                let colors = theme.colors();
                if !enabled() {
                    at(colors.on_surface, ALPHA_38)
                } else if shown.get() == i {
                    colors.primary
                } else {
                    colors.on_surface_variant
                }
            }
        };
        let label = Text::new(self.tab.label.clone())
            .style(theme.snapshot().text.label)
            .color(color.clone());
        let content = match self.tab.icon {
            Some(icon) => AnyView::new(
                Column::new()
                    .main_axis_size(MainAxisSize::Min)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .spacing(2.0)
                    .child(Icon::new(icon).size(24.0).color(color))
                    .child(label),
            ),
            None => AnyView::new(label),
        };
        let id = cx.render(TabRender, [content]);
        self.ids.borrow_mut().push(id);
        let hover = signal(false);
        cx.set_paint(
            id,
            TabPaint {
                hover,
                enabled: enabled.clone(),
                theme,
            },
        );
        {
            let enabled = enabled.clone();
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || (hover.get(), enabled(), theme.colors()))),
                |e, _| {
                    e.update_paint::<TabPaint>(|_| {});
                },
            );
        }
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Tab,
                name: Some(self.tab.label),
                position_in_set: Some(i + 1),
                size_of_set: Some(state.len()),
                actions: SemanticsActions {
                    click: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        {
            let enabled = enabled.clone();
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || (shown.get() == i, enabled()))),
                |e, (selected, enabled)| {
                    e.update_semantics(|s| {
                        s.selected = Some(selected);
                        s.disabled = !enabled;
                    });
                },
            );
        }
        // A tab takes focus only when asked (the Focus action); traversal stops at the bar.
        cx.focusable(
            id,
            FocusOptions {
                traversable: false,
                focus_on_press: false,
            },
        );
        // A primary press and release inside selects (WIDGETS-TAB-05).
        let pressed = Cell::new(false);
        cx.on_pointer(id, Phase::Bubble, move |p| {
            match p.event.kind {
                PointerKind::Down(PointerButton::Primary) => {
                    pressed.set(state.is_enabled(i));
                }
                PointerKind::Up(PointerButton::Primary) => {
                    let inside = p.local.x >= 0.0
                        && p.local.y >= 0.0
                        && p.local.x <= p.size.width
                        && p.local.y <= p.size.height;
                    if pressed.replace(false) && inside && state.is_enabled(i) {
                        state.select(i);
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

/// A tab's hover layer (WIDGETS-TAB-03).
struct TabPaint {
    hover: Signal<bool>,
    enabled: Rc<dyn Fn() -> bool>,
    theme: ThemeHandle,
}
impl Paint for TabPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        if self.hover.get() && (self.enabled)() {
            let size = cx.size();
            let color = self.theme.colors().on_surface.with_alpha(0.08);
            cx.scene()
                .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), color);
        }
    }
}

/// The divider, the indicator and the focus ring, over the tabs (WIDGETS-TAB-03, -06).
struct BarPaint {
    slots: Slots,
    motion: Rc<Cell<Motion>>,
    anim: AnimationController,
    focused: Signal<bool>,
    shown: Memo<usize>,
    theme: ThemeHandle,
}
impl Paint for BarPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        cx.paint_children();
        let size = cx.size();
        let colors = self.theme.colors();
        cx.scene().fill_rect(
            Rect::from_ltrb(0.0, size.height - 1.0, size.width, size.height),
            colors.outline_variant,
        );
        let slots = self.slots.borrow();
        if let Some((x, width)) = indicator(&slots, self.motion.get(), self.anim.value()) {
            cx.scene().fill(
                RoundedRect::new(
                    Rect::from_ltrb(x, size.height - 3.0, x + width, size.height),
                    BorderRadius {
                        top_left: 3.0,
                        top_right: 3.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                ),
                colors.primary,
            );
        }
        if self.focused.get() {
            if let Some(slot) = slots.get(self.shown.get()) {
                cx.scene().stroke(
                    RoundedRect::from_rect(Rect::from_ltrb(
                        slot.x,
                        0.0,
                        slot.x + slot.width,
                        size.height,
                    )),
                    2.0,
                    colors.primary,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// The pages.

/// `TabBarView`'s build, with the panels' names when known (`TabView`).
fn build_pages(
    selected: Signal<usize>,
    pages: Vec<PageBuilder>,
    names: Vec<String>,
    keep_alive: bool,
    cx: &mut BuildCx<'_>,
) -> ElementId {
    let n = pages.len();
    let shown = memo(move || selected.get().min(n.saturating_sub(1)));
    let visited: Rc<[Signal<bool>]> = (0..n).map(|_| signal(false)).collect();
    {
        let visited = visited.clone();
        effect(move || {
            let i = shown.get();
            if let Some(v) = visited.get(i) {
                if !v.get_untracked() {
                    v.set(true);
                }
            }
        });
    }
    let children: Vec<AnyView> = pages
        .into_iter()
        .enumerate()
        .map(|(i, build)| {
            // Built on first selection and kept (or only while shown) (WIDGETS-TAB-08).
            let built: Memo<bool> = if keep_alive {
                let v = visited[i];
                memo(move || v.get())
            } else {
                memo(move || shown.get() == i)
            };
            AnyView::new(Page {
                index: i,
                build,
                built,
                shown,
                name: names.get(i).cloned(),
            })
        })
        .collect();
    cx.render(PagesRender, children)
}

/// A `TabBarView` that reports its element (for `TabView`).
struct PagesView {
    selected: Signal<usize>,
    pages: Vec<PageBuilder>,
    names: Vec<String>,
    keep_alive: bool,
    id: Rc<Cell<Option<ElementId>>>,
}
impl View for PagesView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = build_pages(self.selected, self.pages, self.names, self.keep_alive, cx);
        self.id.set(Some(id));
        id
    }
}

struct Page {
    index: usize,
    build: PageBuilder,
    built: Memo<bool>,
    shown: Memo<usize>,
    name: Option<String>,
}
impl View for Page {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (build, built) = (self.build, self.built);
        let content = Dyn::new(move || {
            if built.get() {
                untrack(|| build())
            } else {
                AnyView::new(Nothing)
            }
        });
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [AnyView::new(content)],
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::TabPanel,
                name: self.name,
                ..Default::default()
            },
        );
        // Only the shown page is onstage (WIDGETS-TAB-08, VIEW-OFFSTAGE).
        let (i, shown) = (self.index, self.shown);
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || shown.get() != i)),
            |e, offstage| e.set_offstage(offstage),
        );
        id
    }
}

/// Pages fill the constraints on bounded axes, each at the origin.
#[derive(Clone, PartialEq)]
struct PagesRender;
impl RenderBox for PagesRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let page = BoxConstraints::new(
            if c.has_bounded_width() {
                c.max_width
            } else {
                0.0
            },
            c.max_width,
            if c.has_bounded_height() {
                c.max_height
            } else {
                0.0
            },
            c.max_height,
        );
        let mut size = Size::ZERO;
        for i in 0..children.len() {
            size = size.max(children.layout(i, page));
            children.set_offset(i, Vec2::ZERO);
        }
        c.constrain(Size::new(
            if c.has_bounded_width() {
                c.max_width
            } else {
                size.width
            },
            if c.has_bounded_height() {
                c.max_height
            } else {
                size.height
            },
        ))
    }
}

/// Nothing: an empty region.
struct Nothing;
impl View for Nothing {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}

/// Moves the focus to the bar once when asked, at a frame (WIDGETS-TAB-09).
struct FocusMove {
    target: Rc<Cell<Option<ElementId>>>,
    pending: Rc<Cell<bool>>,
}
impl View for FocusMove {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if self.pending.replace(false) {
            if let Some(target) = self.target.get() {
                cx.focus(target);
            }
        }
        cx.region(|_| {})
    }
}
