//! [`Slider`]. Spec: `docs/specs/widgets/slider.md`.

use std::cell::Cell;
use std::rc::Rc;

use tantu_view::core::Rect;
use tantu_view::layout::{BoxConstraints, RenderConstrainedBox, RenderLimitedBox};
use tantu_view::reactive::{effect, signal};
use tantu_view::scene::{BorderRadius, RoundedRect};
use tantu_view::{
    AnyView, BuildCx, ElementId, FocusOptions, Handled, IntoProp, LogicalKey, NamedKey, Paint,
    PaintCx, Phase, PointerButton, PointerKind, Prop, View,
};

use tantu_theme::ColorScheme;

use crate::theme::{ALPHA_12, ALPHA_38, ThemeHandle, at};
use crate::util::dynamic;

/// The slider's height (Material 3's touch target).
const HEIGHT: f32 = 48.0;
/// Its width in an unbounded width: a 144 px track plus the state layer.
const MIN_WIDTH: f32 = 192.0;
/// The thumb's radius; the track is inset by it on both ends.
const THUMB_RADIUS: f32 = 10.0;
/// Half the track's height.
const TRACK_HALF: f32 = 2.0;
/// The state layer's radius.
const LAYER_RADIUS: f32 = 24.0;
/// A tick mark's radius.
const TICK_RADIUS: f32 = 1.0;
/// A continuous slider's keyboard step, as a fraction of the range.
const KEY_STEP: f64 = 0.05;

/// Picks a number in a range by dragging a thumb (Flutter's `Slider`, Material 3 look).
///
/// The slider is controlled: it shows `value` and reports changes through `on_change`, which
/// the app usually writes back to the signal it passed in:
///
/// ```
/// use tantu_view::ViewTree;
/// use tantu_view::reactive::signal;
/// use tantu_widgets::Slider;
///
/// let tree = ViewTree::new(|| {
///     let volume = signal(0.5f64);
///     Slider::new(volume).on_change(move |v| volume.set(v))
/// });
/// ```
pub struct Slider {
    value: Prop<f64>,
    min: Prop<f64>,
    max: Prop<f64>,
    divisions: u32,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(f64)>>,
    on_change_start: Option<Rc<dyn Fn(f64)>>,
    on_change_end: Option<Rc<dyn Fn(f64)>>,
}

impl Slider {
    /// A continuous slider showing `value` in the range 0 to 1, enabled, reporting nothing
    /// (and so drawn disabled until `on_change` is set).
    pub fn new(value: impl IntoProp<f64>) -> Self {
        Slider {
            value: value.into_prop(),
            min: Prop::Value(0.0),
            max: Prop::Value(1.0),
            divisions: 0,
            enabled: Prop::Value(true),
            on_change: None,
            on_change_start: None,
            on_change_end: None,
        }
    }

    /// The range (default 0 to 1).
    pub fn range(mut self, min: impl IntoProp<f64>, max: impl IntoProp<f64>) -> Self {
        self.min = min.into_prop();
        self.max = max.into_prop();
        self
    }

    /// A discrete slider with `divisions` equal steps (0, the default, is continuous).
    pub fn divisions(mut self, divisions: u32) -> Self {
        self.divisions = divisions;
        self
    }

    /// A disabled slider ignores input and draws dimmed. A slider with no `on_change` is
    /// disabled too, as in Flutter.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with the new value while the user drags or presses a key.
    pub fn on_change(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// Called with the value before a drag starts.
    pub fn on_change_start(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_change_start = Some(Rc::new(f));
        self
    }

    /// Called with the value when a drag ends.
    pub fn on_change_end(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_change_end = Some(Rc::new(f));
        self
    }
}

/// The value model: the range, the divisions and the fraction (WIDGETS-SLIDER-02).
struct Model {
    value: Rc<dyn Fn() -> f64>,
    min: Rc<dyn Fn() -> f64>,
    max: Rc<dyn Fn() -> f64>,
    divisions: u32,
}

impl Model {
    /// The range's span, when it is finite and positive.
    fn span(&self) -> Option<f64> {
        let span = (self.max)() - (self.min)();
        (span.is_finite() && span > 0.0).then_some(span)
    }

    /// `t` clamped to 0..=1 (0 for NaN) and snapped to the divisions.
    fn snap(&self, t: f64) -> f64 {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        if self.divisions == 0 {
            t
        } else {
            let n = f64::from(self.divisions);
            (t * n).round() / n
        }
    }

    /// The current value's fraction along the track.
    fn fraction(&self) -> f64 {
        let value = (self.value)();
        match self.span() {
            Some(span) if value.is_finite() => self.snap((value - (self.min)()) / span),
            _ => 0.0,
        }
    }

    /// The value at fraction `t` (snapped).
    fn value_at(&self, t: f64) -> f64 {
        let min = (self.min)();
        match self.span() {
            Some(span) => min + self.snap(t) * span,
            None => min,
        }
    }

    /// The value under a pointer at `x` in a slider `width` wide (WIDGETS-SLIDER-05).
    fn value_under(&self, x: f32, width: f32) -> f64 {
        let length = (width - 2.0 * THUMB_RADIUS).max(0.0);
        let t = if length > 0.0 {
            f64::from((x - THUMB_RADIUS) / length)
        } else {
            0.0
        };
        self.value_at(t)
    }

    /// The value one keyboard step away (`sign` ±1; WIDGETS-SLIDER-06).
    fn stepped(&self, sign: f64) -> f64 {
        let min = (self.min)();
        let Some(span) = self.span() else {
            return min;
        };
        let value = (self.value)();
        let value = if value.is_finite() { value } else { min };
        if self.divisions == 0 {
            (value + sign * KEY_STEP * span).clamp(min, min + span)
        } else {
            let step = span / f64::from(self.divisions);
            self.value_at((value + sign * step - min) / span)
        }
    }
}

/// The inner element: full width, 48 px tall (WIDGETS-SLIDER-01).
struct Track;

impl View for Track {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let constraints = BoxConstraints::expand(None, Some(HEIGHT));
        cx.render(RenderConstrainedBox::new(constraints), [])
    }
}

impl View for Slider {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let model = Rc::new(Model {
            value: dynamic(self.value),
            min: dynamic(self.min),
            max: dynamic(self.max),
            divisions: self.divisions,
        });
        let enabled: Rc<dyn Fn() -> bool> = {
            let prop = dynamic(self.enabled);
            let has_on_change = self.on_change.is_some();
            Rc::new(move || has_on_change && prop())
        };
        let on_change = self.on_change.unwrap_or_else(|| Rc::new(|_| {}));
        let on_change_start = self.on_change_start.unwrap_or_else(|| Rc::new(|_| {}));
        let on_change_end = self.on_change_end.unwrap_or_else(|| Rc::new(|_| {}));
        let (hovered, dragging, focused) = (signal(false), signal(false), signal(false));
        // Becoming disabled clears hover and drag (WIDGETS-SLIDER-04).
        {
            let enabled = Rc::clone(&enabled);
            effect(move || {
                if !enabled() {
                    if hovered.get_untracked() {
                        hovered.set(false);
                    }
                    if dragging.get_untracked() {
                        dragging.set(false);
                    }
                }
            });
        }
        let theme = ThemeHandle::of(cx);
        let id = cx.render(
            RenderLimitedBox::new(MIN_WIDTH, f32::INFINITY),
            [AnyView::new(Track)],
        );
        cx.set_paint(
            id,
            SliderPaint {
                fraction: 0.0,
                divisions: self.divisions,
                enabled: false,
                hovered: false,
                focused: false,
                dragging: false,
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<SliderPaint>(|p| p.colors = colors);
        });
        {
            let model = Rc::clone(&model);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || model.fraction() as f32)),
                |e, fraction| {
                    e.update_paint::<SliderPaint>(|p| p.fraction = fraction);
                },
            );
        }
        {
            let enabled = Rc::clone(&enabled);
            cx.bind(id, Prop::Dynamic(Box::new(move || enabled())), |e, on| {
                e.update_paint::<SliderPaint>(|p| p.enabled = on);
                // Focusable and traversable while enabled; a press focuses it.
                e.set_focusable(on.then_some(FocusOptions {
                    traversable: true,
                    focus_on_press: true,
                }));
            });
        }
        cx.on_focus_change(id, move |is_focused| focused.set(is_focused));
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || hovered.get())),
            |e, on| {
                e.update_paint::<SliderPaint>(|p| p.hovered = on);
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || focused.get())),
            |e, on| {
                e.update_paint::<SliderPaint>(|p| p.focused = on);
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || dragging.get())),
            |e, on| {
                e.update_paint::<SliderPaint>(|p| p.dragging = on);
            },
        );
        // Keys (WIDGETS-SLIDER-06).
        {
            let (model, enabled, on_change) = (
                Rc::clone(&model),
                Rc::clone(&enabled),
                Rc::clone(&on_change),
            );
            cx.on_key(id, Phase::Bubble, move |k| {
                if !k.event.pressed || !enabled() {
                    return Handled::Continue;
                }
                let LogicalKey::Named(key) = &k.event.key else {
                    return Handled::Continue;
                };
                let new = match key {
                    NamedKey::ArrowRight | NamedKey::ArrowUp => model.stepped(1.0),
                    NamedKey::ArrowLeft | NamedKey::ArrowDown => model.stepped(-1.0),
                    NamedKey::Home => model.value_at(0.0),
                    NamedKey::End => model.value_at(1.0),
                    _ => return Handled::Continue,
                };
                if new != (model.value)() {
                    on_change(new);
                }
                Handled::Stop
            });
        }
        // The pointer (WIDGETS-SLIDER-05, -07).
        let last_reported: Rc<Cell<Option<f64>>> = Rc::default();
        let report = {
            let (model, on_change, last) = (
                Rc::clone(&model),
                Rc::clone(&on_change),
                Rc::clone(&last_reported),
            );
            move |x: f32, width: f32| {
                let value = model.value_under(x, width);
                let previous = last.get().unwrap_or_else(|| (model.value)());
                if value != previous {
                    on_change(value);
                }
                last.set(Some(value));
            }
        };
        cx.on_pointer(id, Phase::Bubble, move |p| match p.event.kind {
            PointerKind::Enter => {
                if enabled() {
                    hovered.set(true);
                }
                Handled::Continue
            }
            PointerKind::Leave => {
                hovered.set(false);
                Handled::Continue
            }
            PointerKind::Down(PointerButton::Primary) if enabled() => {
                dragging.set(true);
                last_reported.set(None);
                on_change_start((model.value)());
                report(p.local.x, p.size.width);
                Handled::Stop
            }
            PointerKind::Move if dragging.get_untracked() => {
                report(p.local.x, p.size.width);
                Handled::Stop
            }
            PointerKind::Up(PointerButton::Primary) if dragging.get_untracked() => {
                dragging.set(false);
                let value = last_reported.get().unwrap_or_else(|| (model.value)());
                on_change_end(value);
                Handled::Stop
            }
            _ => Handled::Continue,
        });
        id
    }
}

/// Paints the tracks, ticks, state layer and thumb (WIDGETS-SLIDER-03, -04, -07).
struct SliderPaint {
    fraction: f32,
    divisions: u32,
    enabled: bool,
    hovered: bool,
    focused: bool,
    dragging: bool,
    colors: ColorScheme,
}

fn circle(cx: f32, cy: f32, radius: f32) -> RoundedRect {
    RoundedRect::new(
        Rect::from_ltrb(cx - radius, cy - radius, cx + radius, cy + radius),
        BorderRadius::circular(radius),
    )
}

impl Paint for SliderPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let length = (size.width - 2.0 * THUMB_RADIUS).max(0.0);
        let (x, y) = (THUMB_RADIUS + self.fraction * length, size.height / 2.0);
        let colors = &self.colors;
        let (active, inactive) = if self.enabled {
            (colors.primary, colors.surface_container_highest)
        } else {
            (
                at(colors.on_surface, ALPHA_38),
                at(colors.on_surface, ALPHA_12),
            )
        };
        let track = |from: f32, to: f32| {
            RoundedRect::new(
                Rect::from_ltrb(from, y - TRACK_HALF, to, y + TRACK_HALF),
                BorderRadius::circular(TRACK_HALF),
            )
        };
        let scene = cx.scene();
        scene.fill(track(THUMB_RADIUS, x), active);
        scene.fill(track(x, size.width - THUMB_RADIUS), inactive);
        if self.divisions > 0 {
            for i in 0..=self.divisions {
                let tick = THUMB_RADIUS + (i as f32 / self.divisions as f32) * length;
                let color = if tick <= x {
                    at(colors.on_primary, ALPHA_38)
                } else {
                    at(colors.on_surface_variant, ALPHA_38)
                };
                scene.fill(circle(tick, y, TICK_RADIUS), color);
            }
        }
        if self.enabled {
            let layer = [
                (self.hovered, 0.08f32),
                (self.focused, 0.10),
                (self.dragging, 0.12),
            ]
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, alpha)| alpha)
            .fold(0.0, f32::max);
            if layer > 0.0 {
                scene.fill(circle(x, y, LAYER_RADIUS), colors.primary.with_alpha(layer));
            }
        }
        scene.fill(circle(x, y, THUMB_RADIUS), active);
    }

    fn overflow(&self) -> f32 {
        LAYER_RADIUS - THUMB_RADIUS
    }
}
