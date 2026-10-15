//! [`SplitView`] and [`SplitPane`]: resizable panes. Spec: `docs/specs/widgets/split-view.md`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use tantu_theme::ColorScheme;
use tantu_view::core::{EdgeInsets, Point, Rect};
use tantu_view::layout::{Axis, RenderConstrainedBox, RenderPadding, RenderSplit};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::RoundedRect;
use tantu_view::{
    AnyView, BuildCx, CursorIcon, ElementId, FocusOptions, Handled, LogicalKey, NamedKey,
    NumericValue, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics,
    SemanticsActions, View,
};

use crate::theme::ThemeHandle;

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

    /// Names the pane for assistive technology; its divider is then "Resize `label`".
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

/// How far one arrow key moves a divider (WIDGETS-SPLIT-06).
const KEY_STEP: f32 = 10.0;
/// The focus outline's width (WIDGETS-SPLIT-02).
const FOCUS_WIDTH: f32 = 2.0;
/// A hovered or dragged divider's line thickness.
const ACTIVE_THICKNESS: f32 = 3.0;
/// Size comparisons tolerate this much rounding.
const EPSILON: f32 = 1e-3;

/// What the dividers share: the panes' rules, the fractions and the last laid-out sizes.
struct Model {
    axis: Axis,
    min_sizes: Vec<f32>,
    collapsible: Vec<bool>,
    fractions: Signal<Vec<f32>>,
    /// The pane sizes from the last layout, recorded at paint.
    sizes: Signal<Vec<f32>>,
    /// Each pane's size before it was last collapsed, for restoring with Enter.
    restore: RefCell<Vec<Option<f32>>>,
}

impl Model {
    fn min_size(&self, pane: usize) -> f32 {
        let min = self.min_sizes[pane];
        if min.is_finite() && min > 0.0 {
            min
        } else {
            0.0
        }
    }

    /// Whether pane `pane` may have size `size` (WIDGETS-SPLIT-03, -04).
    fn allowed(&self, pane: usize, size: f32) -> bool {
        size >= self.min_size(pane) - EPSILON || (self.collapsible[pane] && size.abs() < EPSILON)
    }

    /// The size pane `pane` takes when dragged to `candidate`: collapsed below half its
    /// minimum when collapsible, else at least its minimum (WIDGETS-SPLIT-04).
    fn fit(&self, pane: usize, candidate: f32) -> f32 {
        let min = self.min_size(pane);
        if self.collapsible[pane] && candidate < min / 2.0 {
            0.0
        } else {
            candidate.max(min)
        }
    }

    /// The sizes after moving boundary `i` (between panes `i` and `i + 1`) so pane `i`
    /// becomes `candidate`, from `start`; `None` when the neighbours can't move.
    fn moved(&self, start: &[f32], i: usize, candidate: f32) -> Option<Vec<f32>> {
        if i + 1 >= start.len() || !candidate.is_finite() {
            return None;
        }
        let sum = start[i] + start[i + 1];
        let mut a = self.fit(i, candidate.clamp(0.0, sum));
        let mut b = sum - a;
        if !self.allowed(i + 1, b) {
            b = self.fit(i + 1, b.max(0.0));
            if b > sum {
                b = if self.collapsible[i + 1] { 0.0 } else { sum };
            }
            a = sum - b;
        }
        if a < -EPSILON || b < -EPSILON || !self.allowed(i, a) || !self.allowed(i + 1, b) {
            return None;
        }
        let mut sizes = start.to_vec();
        sizes[i] = a.max(0.0);
        sizes[i + 1] = b.max(0.0);
        // Remember what a pane had before it collapsed (WIDGETS-SPLIT-06's Enter).
        let mut restore = self.restore.borrow_mut();
        for pane in [i, i + 1] {
            if sizes[pane] == 0.0 && start[pane] > 0.0 {
                restore[pane] = Some(start[pane]);
            }
        }
        Some(sizes)
    }

    /// Writes `sizes` as fractions of their sum, in place (WIDGETS-SPLIT-03). True when
    /// they differ from the current layout.
    fn write(&self, sizes: &[f32]) -> bool {
        let current = self.sizes.get_untracked();
        if current.len() == sizes.len()
            && current
                .iter()
                .zip(sizes)
                .all(|(a, b)| (a - b).abs() < EPSILON)
        {
            return false;
        }
        let total: f32 = sizes.iter().sum();
        if total <= 0.0 {
            return false;
        }
        self.fractions.update(|fractions| {
            fractions.clear();
            fractions.extend(sizes.iter().map(|size| size / total));
        });
        true
    }

    /// The boundary's position as a percentage, for assistive technology (WIDGETS-SPLIT-08).
    fn numeric(&self, i: usize) -> NumericValue {
        let sizes = self.sizes.get();
        let total: f32 = sizes.iter().sum();
        let (value, step) = if total > 0.0 {
            let before: f32 = sizes.iter().take(i + 1).sum();
            (
                f64::from(before / total * 100.0),
                f64::from(KEY_STEP / total * 100.0),
            )
        } else {
            (0.0, 0.0)
        };
        NumericValue {
            value,
            min: 0.0,
            max: 100.0,
            step,
        }
    }

    /// The new sizes for a key on divider `i`, if it moves anything (WIDGETS-SPLIT-06).
    fn key(&self, i: usize, key: NamedKey) -> Option<Vec<f32>> {
        let start = self.sizes.get_untracked();
        if i + 1 >= start.len() {
            return None;
        }
        let sum = start[i] + start[i + 1];
        let candidate = match key {
            NamedKey::ArrowLeft | NamedKey::ArrowUp => start[i] - KEY_STEP,
            NamedKey::ArrowRight | NamedKey::ArrowDown => start[i] + KEY_STEP,
            NamedKey::Home => self.min_size(i),
            NamedKey::End => sum - self.min_size(i + 1),
            NamedKey::Enter => {
                // The nearest collapsible neighbour, the preceding pane first.
                let pane = [i, i + 1].into_iter().find(|p| self.collapsible[*p])?;
                let target = if start[pane] > 0.0 {
                    0.0
                } else {
                    let saved = self.restore.borrow()[pane];
                    saved.unwrap_or(self.min_size(pane)).min(sum)
                };
                let candidate = if pane == i { target } else { sum - target };
                // Collapsing and restoring bypass the drag's half-minimum rule.
                let mut sizes = start.clone();
                sizes[i] = candidate;
                sizes[i + 1] = sum - candidate;
                if !self.allowed(i, sizes[i]) || !self.allowed(i + 1, sizes[i + 1]) {
                    return None;
                }
                if target == 0.0 {
                    self.restore.borrow_mut()[pane] = Some(start[pane]);
                }
                return Some(sizes);
            }
            _ => return None,
        };
        // Arrows, Home and End stop at minimums rather than collapsing.
        let min_a = self.min_size(i);
        let max_a = sum - self.min_size(i + 1);
        if max_a < min_a {
            return None;
        }
        let a = candidate.clamp(min_a, max_a);
        let mut sizes = start;
        sizes[i] = a;
        sizes[i + 1] = sum - a;
        Some(sizes)
    }
}

/// Records the layout's pane sizes for the dividers (WIDGETS-SPLIT-03, -08).
struct SplitPaint {
    sizes: Signal<Vec<f32>>,
}

impl Paint for SplitPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        if let Some(split) = cx.render::<RenderSplit>() {
            if self.sizes.get_untracked() != split.sizes() {
                self.sizes.set(split.sizes().to_vec());
            }
        }
    }
}

/// A divider's line and focus outline (WIDGETS-SPLIT-02).
struct DividerPaint {
    axis: Axis,
    thickness: f32,
    hovered: bool,
    focused: bool,
    colors: ColorScheme,
}

impl Paint for DividerPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let (thickness, color) = if self.hovered {
            (ACTIVE_THICKNESS, self.colors.primary)
        } else {
            (self.thickness, self.colors.outline_variant)
        };
        let line = match self.axis {
            Axis::Horizontal => {
                let left = (size.width - thickness) / 2.0;
                Rect::from_ltrb(left, 0.0, left + thickness, size.height)
            }
            Axis::Vertical => {
                let top = (size.height - thickness) / 2.0;
                Rect::from_ltrb(0.0, top, size.width, top + thickness)
            }
        };
        let scene = cx.scene();
        scene.fill_rect(line, color);
        if self.focused {
            let bounds = Rect::from_ltrb(0.0, 0.0, size.width, size.height);
            scene.stroke(
                RoundedRect::from_rect(bounds),
                FOCUS_WIDTH,
                self.colors.primary,
            );
        }
    }
}

/// A drag in progress: the pointer's position on the axis and the sizes at the press.
type Press = (f32, Vec<f32>);

/// The divider between panes `index` and `index + 1`.
struct Handle {
    model: Rc<Model>,
    index: usize,
    label: String,
    thickness: f32,
}

impl View for Handle {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let Handle {
            model,
            index,
            label,
            thickness,
        } = self;
        let theme = ThemeHandle::of(cx);
        let id = cx.render(RenderConstrainedBox::expand(), []);
        let (hovered, dragging, focused) = (signal(false), signal(false), signal(false));
        cx.set_paint(
            id,
            DividerPaint {
                axis: model.axis,
                thickness,
                hovered: false,
                focused: false,
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<DividerPaint>(|p| p.colors = colors);
        });
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || hovered.get() || dragging.get())),
            |e, on| {
                e.update_paint::<DividerPaint>(|p| p.hovered = on);
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || focused.get())),
            |e, on| {
                e.update_paint::<DividerPaint>(|p| p.focused = on);
            },
        );
        cx.on_focus_change(id, move |is_focused| focused.set(is_focused));
        // WIDGETS-SPLIT-05.
        cx.set_cursor(
            id,
            match model.axis {
                Axis::Horizontal => CursorIcon::ResizeHorizontal,
                Axis::Vertical => CursorIcon::ResizeVertical,
            },
        );
        // A focus stop that a press doesn't focus (WIDGETS-SPLIT-06, -07).
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
                name: Some(label),
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
            let model = Rc::clone(&model);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || model.numeric(index))),
                |e, numeric| {
                    e.update_semantics(|s| {
                        s.numeric = Some(numeric);
                        s.value = Some(format!("{:.0} %", numeric.value));
                    });
                },
            );
        }
        // Dragging (WIDGETS-SPLIT-03, -04): the sizes at the press and the pointer's position.
        let press: Rc<RefCell<Option<Press>>> = Rc::default();
        {
            let model = Rc::clone(&model);
            let axis = model.axis;
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
                    let sizes = model.sizes.get_untracked();
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
                    if let Some(new) = model.moved(sizes, index, sizes[index] + delta) {
                        model.write(&new);
                    }
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
        // Keys (WIDGETS-SPLIT-06).
        cx.on_key(id, Phase::Bubble, move |k| {
            let event = &k.event;
            let modified = event.modifiers.shift
                || event.modifiers.control
                || event.modifiers.alt
                || event.modifiers.super_key;
            if !event.pressed || modified {
                return Handled::Continue;
            }
            let LogicalKey::Named(key) = &event.key else {
                return Handled::Continue;
            };
            match model.key(index, *key) {
                Some(sizes) if model.write(&sizes) => Handled::Stop,
                _ => Handled::Continue,
            }
        });
        id
    }
}

/// A labelled pane: a group named after the pane around its content (WIDGETS-SPLIT-08).
struct Labelled {
    label: String,
    content: AnyView,
}

impl View for Labelled {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderPadding::new(EdgeInsets::ZERO), [self.content]);
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Group,
                name: Some(self.label),
                ..Default::default()
            },
        );
        id
    }
}

impl View for SplitView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let n = self.panes.len();
        let fractions = self
            .fractions
            .unwrap_or_else(|| signal(vec![1.0 / n.max(1) as f32; n]));
        let model = Rc::new(Model {
            axis: self.axis,
            min_sizes: self.panes.iter().map(|p| p.min_size).collect(),
            collapsible: self.panes.iter().map(|p| p.collapsible).collect(),
            fractions,
            sizes: signal(Vec::new()),
            restore: RefCell::new(vec![None; n]),
        });
        let thickness = if self.thickness.is_finite() && self.thickness > 0.0 {
            self.thickness
        } else {
            0.0
        };
        let mut render = RenderSplit::new(self.axis, n);
        render.min_sizes = Arc::from(model.min_sizes.clone());
        render.collapsible = Arc::from(model.collapsible.clone());
        render.gap = thickness;
        render.handle = self.grab;
        render.fractions = Arc::from(fractions.get_untracked());
        let labels: Vec<Option<String>> = self.panes.iter().map(|p| p.label.clone()).collect();
        let mut children: Vec<AnyView> = self
            .panes
            .into_iter()
            .map(|pane| match pane.label {
                Some(label) => AnyView::new(Labelled {
                    label,
                    content: pane.content,
                }),
                None => pane.content,
            })
            .collect();
        for (index, label) in labels.iter().enumerate().take(n.saturating_sub(1)) {
            let label = match label {
                Some(label) => format!("Resize {label}"),
                None => format!("Resize pane {}", index + 1),
            };
            children.push(AnyView::new(Handle {
                model: Rc::clone(&model),
                index,
                label,
                thickness,
            }));
        }
        let id = cx.render(render, children);
        cx.set_paint(id, SplitPaint { sizes: model.sizes });
        // WIDGETS-SPLIT-01: the fractions relayout the split without rebuilding a pane.
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || fractions.get())),
            |e, fractions| {
                e.update_render::<RenderSplit>(|r| r.fractions = Arc::from(fractions));
            },
        );
        id
    }
}
