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

use crate::{Align, Padding};

/// `Text`'s default color (Material 3's on-surface).
const ON_SURFACE: Color = Color::from_argb32(0xFF1C_1B1F);
/// The filled button's fill (Material 3's primary).
const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
/// A disabled button's fill: on-surface at 12 %.
const DISABLED_FILL: Color = Color::from_argb32(0x1F1C_1B1F);
/// A disabled button's label: on-surface at 38 %.
const DISABLED_LABEL: Color = Color::from_argb32(0x611C_1B1F);

/// Shows a string (a value, a closure or a signal) in a style and color (Flutter's `Text`).
pub struct Text {
    text: Prop<String>,
    style: TextStyle,
    color: Prop<Color>,
    max_lines: Option<u32>,
    soft_wrap: bool,
}

impl Text {
    /// `text` in `TextStyle::body()` and near-black, wrapping at the available width.
    pub fn new(text: impl IntoProp<String>) -> Self {
        Text {
            text: text.into_prop(),
            style: TextStyle::body(),
            color: Prop::Value(ON_SURFACE),
            max_lines: None,
            soft_wrap: true,
        }
    }

    /// The text style.
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// The text color.
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.color = color.into_prop();
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
        let mut paragraph = RenderParagraph::new("", cx.text_style(self.style));
        paragraph.max_lines = self.max_lines;
        paragraph.soft_wrap = self.soft_wrap;
        let id = cx.render(paragraph, []);
        cx.set_paint(id, ParagraphPaint { color: ON_SURFACE });
        cx.bind(id, self.text, |e, text: String| {
            e.update_render::<RenderParagraph>(|p| p.text = Arc::from(text));
        });
        cx.bind(id, self.color, |e, color| {
            e.update_paint::<ParagraphPaint>(|p| p.color = color);
        });
        id
    }
}

/// A pressable label (Material 3's filled button, with a fixed look until `tantu-theme`).
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
        let label_color = {
            let enabled = Rc::clone(&enabled);
            move || {
                if enabled() {
                    Color::WHITE
                } else {
                    DISABLED_LABEL
                }
            }
        };
        let label = Text::new(self.label)
            .style(TextStyle::label())
            .color(label_color);
        let content = Padding::new(EdgeInsets::symmetric(24.0, 10.0)).child(
            Align::new(Alignment::CENTER)
                .width_factor(1.0)
                .height_factor(1.0)
                .child(label),
        );
        let minimum = BoxConstraints::new(64.0, f32::INFINITY, 40.0, f32::INFINITY);
        let id = cx.render(RenderConstrainedBox::new(minimum), [AnyView::new(content)]);
        cx.set_paint(
            id,
            ButtonPaint {
                state: ButtonState::Idle,
                focused: false,
            },
        );
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
}

/// `color` with `alpha` white over it (source-over in sRGB).
fn over_white(color: Color, alpha: f32) -> Color {
    let mix = |c: f32| c * (1.0 - alpha) + alpha;
    Color {
        r: mix(color.r),
        g: mix(color.g),
        b: mix(color.b),
        a: color.a,
    }
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
        let color = match layer {
            // The strongest state layer wins; focus is 10 % (WIDGETS-BUTTON-07).
            Some(layer) => over_white(PRIMARY, layer.max(if self.focused { 0.10 } else { 0.0 })),
            None => DISABLED_FILL,
        };
        cx.scene()
            .fill(RoundedRect::new(rect, BorderRadius::circular(20.0)), color);
        if self.focused && layer.is_some() {
            // A 3 px outline 2 to 5 px outside the bounds (a stroke lies inside its shape).
            cx.scene().stroke(
                RoundedRect::new(rect.inflate(5.0), BorderRadius::circular(25.0)),
                3.0,
                PRIMARY,
            );
        }
    }

    fn overflow(&self) -> f32 {
        5.0
    }
}
