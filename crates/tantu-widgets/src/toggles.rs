//! [`Checkbox`], [`Radio`] and [`Switch`]. Spec: `docs/specs/widgets/toggles.md`.

use std::rc::Rc;

use tantu_view::core::{Affine, Color, Point, Rect, Size, Vec2};
use tantu_view::layout::{BoxConstraints, RenderConstrainedBox};
use tantu_view::reactive::{effect, signal};
use tantu_view::scene::{BorderRadius, RoundedRect, SceneBuilder};
use tantu_view::{
    AnyView, BuildCx, CursorIcon, ElementId, FocusOptions, Handled, IntoProp, LogicalKey, NamedKey,
    Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics, SemanticsActions,
    Toggled, View,
};

use tantu_theme::ColorScheme;

use crate::theme::{ALPHA_12, ALPHA_38, ThemeHandle, at};
use crate::util::{disabled_prop, dynamic};

/// The checkbox's and radio's element at density 0 (Flutter's desktop `shrinkWrap` tap
/// target); the density shrinks or grows both sides (WIDGETS-THEME-03).
const TOGGLE_SIZE: Size = Size::new(40.0, 40.0);
/// The switch's element at density 0.
const SWITCH_SIZE: Size = Size::new(60.0, 40.0);
/// The state layer's radius.
const LAYER_RADIUS: f32 = 20.0;
/// The outlines' width.
const STROKE: f32 = 2.0;
/// The checkbox box's side.
const BOX_SIZE: f32 = 18.0;
/// The box's corner radius.
const BOX_RADIUS: f32 = 2.0;
/// The radio ring's radius.
const RING_RADIUS: f32 = 8.0;
/// The radio dot's radius.
const DOT_RADIUS: f32 = 4.5;
/// The switch track.
const TRACK: Size = Size::new(52.0, 32.0);
/// The thumb's distance from the track's near edge.
const THUMB_INSET: f32 = 16.0;
/// Thumb radii: unselected, selected, pressed.
const THUMB_OFF: f32 = 8.0;
const THUMB_ON: f32 = 12.0;
const THUMB_PRESSED: f32 = 14.0;

/// A box that is checked or not (Flutter's `Checkbox`, Material 3 look).
///
/// The checkbox is controlled: it shows `value` and reports `!value` through `on_change`
/// when activated; the app writes its signal and the box follows at the next frame.
pub struct Checkbox {
    value: Prop<bool>,
    indeterminate: Prop<bool>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(bool)>>,
}

impl Checkbox {
    /// A checkbox showing `value`, enabled, reporting nothing (so drawn disabled until
    /// `on_change` is set).
    pub fn new(value: impl IntoProp<bool>) -> Self {
        Checkbox {
            value: value.into_prop(),
            indeterminate: Prop::Value(false),
            enabled: Prop::Value(true),
            on_change: None,
        }
    }

    /// Shows a dash instead of the check or the empty box while true (a partial selection).
    pub fn indeterminate(mut self, indeterminate: impl IntoProp<bool>) -> Self {
        self.indeterminate = indeterminate.into_prop();
        self
    }

    /// A disabled checkbox ignores input and draws dimmed. No `on_change` disables it too.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with `!value` when the user activates the checkbox.
    pub fn on_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// One choice of a group, selected while the group's value equals its own (Flutter's
/// `Radio<T>`, Material 3 look). Controlled: `on_change` reports this radio's value.
pub struct Radio<T> {
    value: T,
    group: Prop<T>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(T)>>,
}

impl<T: Copy + PartialEq + 'static> Radio<T> {
    /// A radio for `value`, selected while `group` (a closure or a signal) equals it.
    pub fn new(value: T, group: impl IntoProp<T>) -> Self {
        Radio {
            value,
            group: group.into_prop(),
            enabled: Prop::Value(true),
            on_change: None,
        }
    }

    /// A disabled radio ignores input and draws dimmed. No `on_change` disables it too.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with this radio's `value` when the user activates it.
    pub fn on_change(mut self, f: impl Fn(T) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// An on/off switch (Flutter's `Switch`, Material 3 look). Controlled: `on_change` reports
/// `!value`.
pub struct Switch {
    value: Prop<bool>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(bool)>>,
}

impl Switch {
    /// A switch showing `value`, enabled, reporting nothing (so drawn disabled until
    /// `on_change` is set).
    pub fn new(value: impl IntoProp<bool>) -> Self {
        Switch {
            value: value.into_prop(),
            enabled: Prop::Value(true),
            on_change: None,
        }
    }

    /// A disabled switch ignores input and draws dimmed. No `on_change` disables it too.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with `!value` when the user activates the switch.
    pub fn on_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// What every toggle's paint knows about its interaction state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct State {
    enabled: bool,
    hovered: bool,
    focused: bool,
    pressed: bool,
}

impl State {
    /// The state layer's alpha (the strongest state wins), 0 when none (WIDGETS-TOGGLE-08).
    fn layer_alpha(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        [
            (self.hovered, 0.08f32),
            (self.focused, 0.10),
            (self.pressed, 0.12),
        ]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, alpha)| alpha)
        .fold(0.0, f32::max)
    }

    /// Paints the state layer centered on `center`, in primary when `selected`, else in
    /// on-surface.
    pub(crate) fn paint_layer(
        &self,
        scene: &mut SceneBuilder<'_>,
        colors: &ColorScheme,
        center: Point,
        selected: bool,
    ) {
        let alpha = self.layer_alpha();
        if alpha > 0.0 {
            let base = if selected {
                colors.primary
            } else {
                colors.on_surface
            };
            scene.fill(circle(center, LAYER_RADIUS), base.with_alpha(alpha));
        }
    }
}

/// A paint with a toggle [`State`] and the theme's colors.
pub(crate) trait TogglePaint: Paint {
    fn state_mut(&mut self) -> &mut State;
    fn colors_mut(&mut self) -> &mut ColorScheme;
}

pub(crate) fn circle(center: Point, radius: f32) -> RoundedRect {
    RoundedRect::new(
        Rect::from_ltrb(
            center.x - radius,
            center.y - radius,
            center.x + radius,
            center.y + radius,
        ),
        BorderRadius::circular(radius),
    )
}

/// Builds a toggle's element: tight at `size` adjusted by the theme's density, with `paint`
/// (its colors kept in step with the theme), `children`, the focus and the pointer and key
/// handlers that call `activate` (WIDGETS-TOGGLE-01, -05, -06, -07, -08; WIDGETS-THEME-01,
/// -03). The icon button is built on it too.
pub(crate) fn build_toggle<P: TogglePaint>(
    cx: &mut BuildCx<'_>,
    size: Size,
    enabled: Rc<dyn Fn() -> bool>,
    activate: Rc<dyn Fn()>,
    paint: P,
    // The role and name; focus, click and `disabled` are filled in here.
    semantics: Semantics,
    children: impl IntoIterator<Item = AnyView>,
) -> ElementId {
    let theme = ThemeHandle::of(cx);
    let id = cx.render(
        RenderConstrainedBox::new(BoxConstraints::tight(size)),
        children,
    );
    cx.set_paint(id, paint);
    // Semantics: the role, the name, disabled with `enabled` (WIDGETS-SEM-01, -02).
    cx.set_semantics(
        id,
        Semantics {
            focusable: true,
            actions: SemanticsActions {
                click: true,
                focus: true,
                ..Default::default()
            },
            ..semantics
        },
    );
    cx.bind(id, disabled_prop(&enabled), |e, disabled| {
        e.update_semantics(|s| s.disabled = disabled);
    });
    cx.bind(id, theme.colors_prop(), |e, colors| {
        e.update_paint::<P>(|p| *p.colors_mut() = colors);
    });
    cx.bind(id, theme.density_prop(), move |e, density| {
        e.update_render::<RenderConstrainedBox>(|r| {
            r.additional = BoxConstraints::tight(Size::new(
                density.apply(size.width),
                density.apply(size.height),
            ));
        });
    });
    let (hovered, pressed, focused) = (signal(false), signal(false), signal(false));
    // Becoming disabled clears hover and press (WIDGETS-TOGGLE-05).
    {
        let enabled = Rc::clone(&enabled);
        effect(move || {
            if !enabled() {
                if hovered.get_untracked() {
                    hovered.set(false);
                }
                if pressed.get_untracked() {
                    pressed.set(false);
                }
            }
        });
    }
    {
        let enabled = Rc::clone(&enabled);
        cx.bind(id, Prop::Dynamic(Box::new(move || enabled())), |e, on| {
            e.update_paint::<P>(|p| p.state_mut().enabled = on);
            e.set_focusable(on.then_some(FocusOptions::default()));
        });
    }
    cx.on_focus_change(id, move |is_focused| focused.set(is_focused));
    cx.bind(
        id,
        Prop::Dynamic(Box::new(move || focused.get())),
        |e, on| {
            e.update_paint::<P>(|p| p.state_mut().focused = on);
        },
    );
    cx.bind(
        id,
        Prop::Dynamic(Box::new(move || hovered.get())),
        |e, on| {
            e.update_paint::<P>(|p| p.state_mut().hovered = on);
        },
    );
    cx.bind(
        id,
        Prop::Dynamic(Box::new(move || pressed.get())),
        |e, on| {
            e.update_paint::<P>(|p| p.state_mut().pressed = on);
        },
    );
    // Space and Enter activate (WIDGETS-TOGGLE-07).
    {
        let (enabled, activate) = (Rc::clone(&enabled), Rc::clone(&activate));
        cx.on_key(id, Phase::Bubble, move |k| {
            let activates = matches!(&k.event.key, LogicalKey::Named(NamedKey::Enter))
                || k.event.key == LogicalKey::Character(" ".into());
            if !(k.event.pressed && !k.event.repeat && activates && enabled()) {
                return Handled::Continue;
            }
            activate();
            Handled::Stop
        });
    }
    // Press and release inside (WIDGETS-TOGGLE-06).
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
            pressed.set(true);
            Handled::Stop
        }
        PointerKind::Up(PointerButton::Primary) if pressed.get_untracked() => {
            pressed.set(false);
            let inside = p.local.x >= 0.0
                && p.local.y >= 0.0
                && p.local.x < p.size.width
                && p.local.y < p.size.height;
            if inside && enabled() {
                activate();
            }
            Handled::Stop
        }
        _ => Handled::Continue,
    });
    cx.set_cursor(id, CursorIcon::Pointer);
    id
}

/// Semantics with just a role, for `build_toggle`.
pub(crate) fn role(role: Role) -> Semantics {
    Semantics {
        role,
        ..Default::default()
    }
}

/// Whether a toggle is enabled: its prop, and it has an `on_change` (WIDGETS-TOGGLE-05).
pub(crate) fn enabled_fn(prop: Prop<bool>, has_on_change: bool) -> Rc<dyn Fn() -> bool> {
    let prop = dynamic(prop);
    Rc::new(move || has_on_change && prop())
}

impl View for Checkbox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let enabled = enabled_fn(self.enabled, self.on_change.is_some());
        let value = dynamic(self.value);
        let activate: Rc<dyn Fn()> = {
            let (value, on_change) = (Rc::clone(&value), self.on_change);
            Rc::new(move || {
                if let Some(on_change) = &on_change {
                    on_change(!value());
                }
            })
        };
        let paint = CheckboxPaint {
            state: State::default(),
            colors: ThemeHandle::of(cx).colors(),
            value: false,
            indeterminate: false,
        };
        let id = build_toggle(
            cx,
            TOGGLE_SIZE,
            enabled,
            activate,
            paint,
            role(Role::CheckBox),
            [],
        );
        let indeterminate = dynamic(self.indeterminate);
        {
            let (value, indeterminate) = (Rc::clone(&value), Rc::clone(&indeterminate));
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || (value(), indeterminate()))),
                |e, (on, mixed)| {
                    e.update_paint::<CheckboxPaint>(|p| {
                        p.value = on;
                        p.indeterminate = mixed;
                    });
                    e.update_semantics(|s| {
                        s.toggled = Some(if mixed {
                            Toggled::Mixed
                        } else if on {
                            Toggled::True
                        } else {
                            Toggled::False
                        });
                    });
                },
            );
        }
        id
    }
}

impl<T: Copy + PartialEq + 'static> View for Radio<T> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let enabled = enabled_fn(self.enabled, self.on_change.is_some());
        let (value, group) = (self.value, dynamic(self.group));
        let activate: Rc<dyn Fn()> = {
            let on_change = self.on_change;
            Rc::new(move || {
                if let Some(on_change) = &on_change {
                    on_change(value);
                }
            })
        };
        let paint = RadioPaint {
            state: State::default(),
            colors: ThemeHandle::of(cx).colors(),
            selected: false,
        };
        let id = build_toggle(
            cx,
            TOGGLE_SIZE,
            enabled,
            activate,
            paint,
            role(Role::RadioButton),
            [],
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || group() == value)),
            |e, on| {
                e.update_paint::<RadioPaint>(|p| p.selected = on);
                e.update_semantics(|s| {
                    s.toggled = Some(if on { Toggled::True } else { Toggled::False })
                });
            },
        );
        id
    }
}

impl View for Switch {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let enabled = enabled_fn(self.enabled, self.on_change.is_some());
        let value = dynamic(self.value);
        let activate: Rc<dyn Fn()> = {
            let (value, on_change) = (Rc::clone(&value), self.on_change);
            Rc::new(move || {
                if let Some(on_change) = &on_change {
                    on_change(!value());
                }
            })
        };
        let paint = SwitchPaint {
            state: State::default(),
            colors: ThemeHandle::of(cx).colors(),
            value: false,
        };
        let id = build_toggle(
            cx,
            SWITCH_SIZE,
            enabled,
            activate,
            paint,
            role(Role::Switch),
            [],
        );
        cx.bind(id, Prop::Dynamic(Box::new(move || value())), |e, on| {
            e.update_paint::<SwitchPaint>(|p| p.value = on);
            e.update_semantics(|s| {
                s.toggled = Some(if on { Toggled::True } else { Toggled::False })
            });
        });
        id
    }
}

/// Paints the checkbox (WIDGETS-TOGGLE-02).
struct CheckboxPaint {
    state: State,
    colors: ColorScheme,
    value: bool,
    indeterminate: bool,
}

impl TogglePaint for CheckboxPaint {
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    fn colors_mut(&mut self) -> &mut ColorScheme {
        &mut self.colors
    }
}

impl Paint for CheckboxPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let half = BOX_SIZE / 2.0;
        let rect = Rect::from_ltrb(
            center.x - half,
            center.y - half,
            center.x + half,
            center.y + half,
        );
        let shape = RoundedRect::new(rect, BorderRadius::circular(BOX_RADIUS));
        let selected = self.value || self.indeterminate;
        let colors = &self.colors;
        let scene = cx.scene();
        self.state.paint_layer(scene, colors, center, selected);
        let fill = if self.state.enabled {
            colors.primary
        } else {
            at(colors.on_surface, ALPHA_38)
        };
        if self.indeterminate {
            scene.fill(shape, fill);
            let dash = Rect::from_ltrb(
                rect.left + 0.2 * BOX_SIZE,
                center.y - STROKE / 2.0,
                rect.left + 0.8 * BOX_SIZE,
                center.y + STROKE / 2.0,
            );
            scene.fill_rect(dash, colors.on_primary);
        } else if self.value {
            scene.fill(shape, fill);
            // Flutter's check: start, mid and end as fractions of the box.
            let point =
                |fx: f32, fy: f32| Point::new(rect.left + fx * BOX_SIZE, rect.top + fy * BOX_SIZE);
            let (start, mid, end) = (point(0.15, 0.45), point(0.40, 0.70), point(0.85, 0.25));
            for (from, to) in [(start, mid), (mid, end)] {
                check_bar(scene, from, to, colors.on_primary);
            }
        } else {
            let outline = if self.state.enabled {
                colors.on_surface_variant
            } else {
                at(colors.on_surface, ALPHA_38)
            };
            scene.stroke(shape, STROKE, outline);
        }
    }
}

/// One bar of the check mark: a rect of width `STROKE` from `from` to `to`, extended by half
/// the stroke at both ends, in a transform scope that rotates it into place.
fn check_bar(scene: &mut SceneBuilder<'_>, from: Point, to: Point, color: Color) {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let length = dx.hypot(dy);
    let angle = dy.atan2(dx);
    let mid = Vec2::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
    scene.push_transform(Affine::translate(mid) * Affine::rotate(angle));
    let half = length / 2.0 + STROKE / 2.0;
    scene.fill_rect(
        Rect::from_ltrb(-half, -STROKE / 2.0, half, STROKE / 2.0),
        color,
    );
    scene.pop();
}

/// Paints the radio (WIDGETS-TOGGLE-03).
struct RadioPaint {
    state: State,
    colors: ColorScheme,
    selected: bool,
}

impl TogglePaint for RadioPaint {
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    fn colors_mut(&mut self) -> &mut ColorScheme {
        &mut self.colors
    }
}

impl Paint for RadioPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let colors = &self.colors;
        let scene = cx.scene();
        self.state.paint_layer(scene, colors, center, self.selected);
        let color = match (self.state.enabled, self.selected) {
            (false, _) => at(colors.on_surface, ALPHA_38),
            (true, true) => colors.primary,
            (true, false) => colors.on_surface_variant,
        };
        scene.stroke(circle(center, RING_RADIUS), STROKE, color);
        if self.selected {
            scene.fill(circle(center, DOT_RADIUS), color);
        }
    }
}

/// Paints the switch (WIDGETS-TOGGLE-04).
struct SwitchPaint {
    state: State,
    colors: ColorScheme,
    value: bool,
}

impl TogglePaint for SwitchPaint {
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    fn colors_mut(&mut self) -> &mut ColorScheme {
        &mut self.colors
    }
}

impl Paint for SwitchPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let track = Rect::from_ltrb(
            center.x - TRACK.width / 2.0,
            center.y - TRACK.height / 2.0,
            center.x + TRACK.width / 2.0,
            center.y + TRACK.height / 2.0,
        );
        let track_shape = RoundedRect::new(track, BorderRadius::circular(TRACK.height / 2.0));
        let thumb_center = Point::new(
            if self.value {
                track.right - THUMB_INSET
            } else {
                track.left + THUMB_INSET
            },
            center.y,
        );
        let thumb_radius = if self.state.enabled && self.state.pressed {
            THUMB_PRESSED
        } else if self.value {
            THUMB_ON
        } else {
            THUMB_OFF
        };
        let colors = &self.colors;
        let scene = cx.scene();
        self.state
            .paint_layer(scene, colors, thumb_center, self.value);
        let (track_fill, outline, thumb) = match (self.state.enabled, self.value) {
            (true, true) => (colors.primary, colors.primary, colors.on_primary),
            (true, false) => (
                colors.surface_container_highest,
                colors.outline,
                colors.outline,
            ),
            (false, _) => (
                at(colors.on_surface, ALPHA_12),
                at(colors.on_surface, ALPHA_12),
                at(colors.on_surface, ALPHA_38),
            ),
        };
        scene.fill(track_shape, track_fill);
        if !self.value {
            scene.stroke(track_shape, STROKE, outline);
        }
        scene.fill(circle(thumb_center, thumb_radius), thumb);
    }
}
