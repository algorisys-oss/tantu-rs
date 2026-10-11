//! [`Text`] and [`Button`]. Spec: `docs/specs/widgets/basic.md`.

use std::rc::Rc;
use std::sync::Arc;

use tantu_view::core::{Color, EdgeInsets, Rect};
use tantu_view::layout::{Alignment, BoxConstraints, RenderConstrainedBox, RenderParagraph};
use tantu_view::reactive::{effect, signal};
use tantu_view::scene::{BorderRadius, RoundedRect};
use tantu_view::text::TextStyle;
use tantu_view::{
    AnyView, BuildCx, CursorIcon, ElementId, FocusOptions, Handled, IntoProp, LogicalKey, NamedKey,
    Paint, PaintCx, ParagraphPaint, Phase, PointerButton, PointerKind, Prop, View,
};

use tantu_theme::ColorScheme;

use crate::theme::{ALPHA_12, ALPHA_38, ThemeHandle, at, blend};
use crate::{Align, Padding};

/// The button's minimum size at density 0 (Material 3).
const BUTTON_MIN: (f32, f32) = (64.0, 40.0);

/// Shows a string (a value, a closure or a signal) in a style and color (Flutter's `Text`).
pub struct Text {
    text: Prop<String>,
    style: Option<TextStyle>,
    color: Option<Prop<Color>>,
    max_lines: Option<u32>,
    soft_wrap: bool,
}

impl Text {
    /// `text` in the theme's body style and on-surface color, wrapping at the available
    /// width.
    pub fn new(text: impl IntoProp<String>) -> Self {
        Text {
            text: text.into_prop(),
            style: None,
            color: None,
            max_lines: None,
            soft_wrap: true,
        }
    }

    /// The text style (default: the theme's `text.body`).
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// The text color (default: the theme's `on_surface`).
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.color = Some(color.into_prop());
        self
    }

    /// Keep at most `max_lines` lines.
    pub fn max_lines(mut self, max_lines: u32) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Wrap at the available width (`true`, the default) or only at hard line breaks.
    pub fn soft_wrap(mut self, soft_wrap: bool) -> Self {
        self.soft_wrap = soft_wrap;
        self
    }
}

impl View for Text {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let style = self.style.unwrap_or_else(|| theme.snapshot().text.body);
        let color = self
            .color
            .unwrap_or_else(|| theme.color_prop(|c| c.on_surface));
        let mut paragraph = RenderParagraph::new("", cx.text_style(style));
        paragraph.max_lines = self.max_lines;
        paragraph.soft_wrap = self.soft_wrap;
        let id = cx.render(paragraph, []);
        cx.set_paint(
            id,
            ParagraphPaint {
                color: Color::TRANSPARENT,
            },
        );
        cx.bind(id, self.text, |e, text: String| {
            e.update_render::<RenderParagraph>(|p| p.text = Arc::from(text));
        });
        cx.bind(id, color, |e, color| {
            e.update_paint::<ParagraphPaint>(|p| p.color = color);
        });
        id
    }
}

/// A pressable label (Material 3's filled button, in the theme's colors).
pub struct Button {
    label: Prop<String>,
    on_press: Option<Rc<dyn Fn()>>,
    enabled: Prop<bool>,
}

impl Button {
    /// A button labelled `label`, enabled, doing nothing when pressed.
    pub fn new(label: impl IntoProp<String>) -> Self {
        Button {
            label: label.into_prop(),
            on_press: None,
            enabled: Prop::Value(true),
        }
    }

    /// Called when a primary-button press that started on the button is released over it.
    pub fn on_press(mut self, f: impl Fn() + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }

    /// A disabled button ignores the pointer and draws dimmed.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }
}

impl View for Button {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let enabled: Rc<dyn Fn() -> bool> = match self.enabled {
            Prop::Value(value) => Rc::new(move || value),
            Prop::Dynamic(f) => Rc::from(f),
        };
        let (hovered, pressed, focused) = (signal(false), signal(false), signal(false));
        // Becoming disabled clears hover and press (WIDGETS-BUTTON-03).
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
        let theme = ThemeHandle::of(cx);
        let label_color = {
            let enabled = Rc::clone(&enabled);
            move || {
                let colors = theme.colors();
                if enabled() {
                    colors.on_primary
                } else {
                    at(colors.on_surface, ALPHA_38)
                }
            }
        };
        let label = Text::new(self.label)
            .style(theme.snapshot().text.label)
            .color(label_color);
        let content = Padding::new(EdgeInsets::symmetric(24.0, 10.0)).child(
            Align::new(Alignment::CENTER)
                .width_factor(1.0)
                .height_factor(1.0)
                .child(label),
        );
        let minimum = BoxConstraints::new(BUTTON_MIN.0, f32::INFINITY, BUTTON_MIN.1, f32::INFINITY);
        let id = cx.render(RenderConstrainedBox::new(minimum), [AnyView::new(content)]);
        cx.set_paint(
            id,
            ButtonPaint {
                state: ButtonState::Idle,
                focused: false,
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<ButtonPaint>(|p| p.colors = colors);
        });
        // The minimum height follows the density (WIDGETS-THEME-03).
        cx.bind(id, theme.density_prop(), |e, density| {
            e.update_render::<RenderConstrainedBox>(|r| {
                r.additional = BoxConstraints::new(
                    BUTTON_MIN.0,
                    f32::INFINITY,
                    density.apply(BUTTON_MIN.1),
                    f32::INFINITY,
                );
            });
        });
        // Focusable while enabled (WIDGETS-BUTTON-05); a click doesn't focus it.
        {
            let enabled = Rc::clone(&enabled);
            cx.bind(id, Prop::Dynamic(Box::new(move || enabled())), |e, on| {
                e.set_focusable(on.then_some(FocusOptions::default()));
            });
        }
        cx.on_focus_change(id, move |is_focused| focused.set(is_focused));
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || focused.get())),
            |e, focused| {
                e.update_paint::<ButtonPaint>(|p| p.focused = focused);
            },
        );
        // Space and Enter press a focused, enabled button (WIDGETS-BUTTON-06).
        {
            let (enabled, on_press) = (Rc::clone(&enabled), self.on_press.clone());
            cx.on_key(id, Phase::Bubble, move |k| {
                let activates = matches!(&k.event.key, LogicalKey::Named(NamedKey::Enter))
                    || k.event.key == LogicalKey::Character(" ".into());
                if !(k.event.pressed && !k.event.repeat && activates && enabled()) {
                    return Handled::Continue;
                }
                if let Some(on_press) = &on_press {
                    on_press();
                }
                Handled::Stop
            });
        }
        let state = {
            let enabled = Rc::clone(&enabled);
            move || {
                if !enabled() {
                    ButtonState::Disabled
                } else if pressed.get() {
                    ButtonState::Pressed
                } else if hovered.get() {
                    ButtonState::Hovered
                } else {
                    ButtonState::Idle
                }
            }
        };
        cx.bind(id, Prop::Dynamic(Box::new(state)), |e, state| {
            e.update_paint::<ButtonPaint>(|p| p.state = state);
        });
        let on_press = self.on_press;
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
                    if let Some(on_press) = &on_press {
                        on_press();
                    }
                }
                Handled::Stop
            }
            _ => Handled::Continue,
        });
        cx.set_cursor(id, CursorIcon::Pointer);
        id
    }
}

/// What a button shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ButtonState {
    Idle,
    Hovered,
    Pressed,
    Disabled,
}

/// Fills the button's rounded background for its state.
struct ButtonPaint {
    state: ButtonState,
    focused: bool,
    colors: ColorScheme,
}

impl Paint for ButtonPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let rect = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        let layer = match self.state {
            ButtonState::Disabled => None,
            ButtonState::Idle => Some(0.0f32),
            ButtonState::Hovered => Some(0.08),
            ButtonState::Pressed => Some(0.12),
        };
        let colors = self.colors;
        let color = match layer {
            // The strongest state layer wins; focus is 10 % (WIDGETS-BUTTON-07).
            Some(layer) => blend(
                colors.primary,
                colors.on_primary,
                layer.max(if self.focused { 0.10 } else { 0.0 }),
            ),
            None => at(colors.on_surface, ALPHA_12),
        };
        // A pill: the radius is half the height.
        let radius = size.height / 2.0;
        cx.scene().fill(
            RoundedRect::new(rect, BorderRadius::circular(radius)),
            color,
        );
        if self.focused && layer.is_some() {
            // A 3 px outline 2 to 5 px outside the bounds (a stroke lies inside its shape).
            cx.scene().stroke(
                RoundedRect::new(rect.inflate(5.0), BorderRadius::circular(radius + 5.0)),
                3.0,
                colors.primary,
            );
        }
    }

    fn overflow(&self) -> f32 {
        5.0
    }
}
