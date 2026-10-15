//! [`TextField`]. Spec: `docs/specs/widgets/text-field.md`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use tantu_view::core::{Affine, EdgeInsets, Point, Rect, Vec2};
use tantu_view::layout::{
    BoxConstraints, RenderConstrainedBox, RenderLimitedBox, RenderParagraph, TextGeometry,
    TextStyleKey,
};
use tantu_view::reactive::{Scope, Signal, Timer, effect, signal, timer};
use tantu_view::scene::{BorderRadius, Clip, RoundedRect};
use tantu_view::text::TextStyle;
use tantu_view::{
    AnyView, BuildCx, CursorIcon, ElementId, FocusOptions, Handled, ImeEvent, IntoProp, LogicalKey,
    NamedKey, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics,
    SemanticsAction, SemanticsActions, SingleActivator, View,
};

use tantu_theme::ColorScheme;

use crate::formatter::TextInputFormatter;
use crate::text_editing::{
    Direction, EditingValue, Movement, TextRange, TextSelection, segment_at,
};
use crate::theme::{ALPHA_12, ALPHA_38, ThemeHandle, at};
use crate::util::dynamic;
use crate::{Padding, TextEditingController};

/// Select the whole text (bound to primary + A in a field).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectAllIntent;

/// Copy the selection to the clipboard (primary + C; WIDGETS-TEXTFIELD-12).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CopyIntent;

/// Cut the selection to the clipboard (primary + X).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CutIntent;

/// Paste the clipboard's text over the selection (primary + V).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PasteIntent;

/// Undo the last edit (primary + Z).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UndoIntent;

/// Redo the last undone edit (primary + Shift + Z, primary + Y).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RedoIntent;

/// The field's width in an unbounded width.
const MAX_WIDTH: f32 = 280.0;
/// The field's minimum height at density 0.
const MIN_HEIGHT: f32 = 40.0;
/// The content padding (horizontal, vertical).
const PADDING: EdgeInsets = EdgeInsets::symmetric(16.0, 8.0);
/// The caret's width.
const CARET_WIDTH: f32 = 2.0;
/// Half a blink period (WIDGETS-TEXTFIELD-10).
const BLINK_HALF_PERIOD: Duration = Duration::from_millis(500);
/// A second press within this interval of the first is a double-click (WIDGETS-TEXTFIELD-11).
const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(500);
/// ... and within this distance.
const DOUBLE_CLICK_SLOP: f32 = 4.0;

/// A text input (Flutter's `TextField`, Material 3 outlined look), editing a
/// [`TextEditingController`].
pub struct TextField {
    controller: TextEditingController,
    placeholder: Option<String>,
    max_lines: Option<u32>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(String)>>,
    on_submit: Option<Rc<dyn Fn(String)>>,
    style: Option<TextStyle>,
    decoration: Option<FieldDecoration>,
    pub(crate) formatters: Vec<Rc<dyn TextInputFormatter>>,
}

/// What a form field adds to its text field: its name, its error and helper text
/// (WIDGETS-FORM-01, -06).
#[derive(Clone)]
pub(crate) struct FieldDecoration {
    pub(crate) name: Option<String>,
    pub(crate) error: Rc<dyn Fn() -> Option<String>>,
    pub(crate) helper: Option<String>,
    /// A combo box's role, popup state and highlighted option (WIDGETS-COMBO-07).
    pub(crate) role: Option<Role>,
    pub(crate) expanded: Option<Rc<dyn Fn() -> bool>>,
    pub(crate) active: Option<Rc<dyn Fn() -> Option<ElementId>>>,
}

impl FieldDecoration {
    /// A decoration naming the field, with nothing else.
    pub(crate) fn named(name: Option<String>) -> Self {
        FieldDecoration {
            name,
            error: Rc::new(|| None),
            helper: None,
            role: None,
            expanded: None,
            active: None,
        }
    }
}

impl TextField {
    /// A single-line field editing `controller`.
    pub fn new(controller: TextEditingController) -> Self {
        TextField {
            controller,
            placeholder: None,
            max_lines: Some(1),
            enabled: Prop::Value(true),
            on_change: None,
            on_submit: None,
            style: None,
            decoration: None,
            formatters: Vec::new(),
        }
    }

    /// Adds a formatter; formatters run on the user's edits in the order added
    /// (WIDGETS-MASK-01).
    pub fn input_formatter(mut self, formatter: impl TextInputFormatter + 'static) -> Self {
        self.formatters.push(Rc::new(formatter));
        self
    }

    /// A form field's name, error and helper (WIDGETS-FORM-01, -06).
    pub(crate) fn decoration(mut self, decoration: FieldDecoration) -> Self {
        self.decoration = Some(decoration);
        self
    }

    /// Shown in the field (dimmed) while the text is empty.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// `None` for any number of lines (Enter inserts a newline); default `Some(1)`.
    pub fn max_lines(mut self, max_lines: Option<u32>) -> Self {
        self.max_lines = max_lines;
        self
    }

    /// A disabled field draws dimmed and ignores input.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with the text after every edit that changes it.
    pub fn on_change(mut self, f: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// Called with the text on Enter in a single-line field.
    pub fn on_submit(mut self, f: impl Fn(String) + 'static) -> Self {
        self.on_submit = Some(Rc::new(f));
        self
    }

    /// The text style (default: the theme's `text.body` at 16 px).
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = Some(style);
        self
    }
}

/// What the handlers share with the editor's paint.
type SharedGeometry = Rc<RefCell<TextGeometry>>;
/// The content's horizontal shift (WIDGETS-TEXTFIELD-09), also shared.
type SharedShift = Rc<Cell<f32>>;

/// What happens to the user's edits: the formatters rewrite them (WIDGETS-MASK-01) and
/// `on_change` hears about them (WIDGETS-TEXTFIELD-04).
#[derive(Clone, Default)]
struct Edits {
    on_change: Option<Rc<dyn Fn(String)>>,
    formatters: Rc<[Rc<dyn TextInputFormatter>]>,
}

/// Applies the user's edit `f` to the controller, formatted, and reports a text change.
fn apply(
    controller: TextEditingController,
    edits: &Edits,
    f: impl FnOnce(&EditingValue) -> EditingValue,
) {
    let before = controller.value();
    controller.edit(|old| {
        let new = f(old);
        // Selection-only edits and compositions in progress aren't formatted; the commit
        // ending a composition is, even when it keeps the preedit's text.
        let committed = old.composing.is_some() && new.composing.is_none();
        if (new.text == old.text && !committed) || new.composing.is_some() {
            return new;
        }
        let formatted = edits
            .formatters
            .iter()
            .fold(new, |value, formatter| formatter.format(old, value));
        let selection = formatted.selection;
        formatted.with_selection(selection)
    });
    report_if_changed(controller, edits, &before.text);
}

/// Reports a text change (also one made by undo or redo).
fn report_if_changed(controller: TextEditingController, edits: &Edits, before: &str) {
    let after = controller.value();
    if after.text != before {
        if let Some(on_change) = &edits.on_change {
            on_change(after.text);
        }
    }
}

impl View for TextField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let controller = self.controller;
        let enabled = dynamic(self.enabled);
        let single_line = self.max_lines == Some(1);
        let focused = signal(false);
        let caret_visible = signal(true);
        let dragging = Cell::new(false);
        let anchor = Cell::new(0usize);
        let geometry: SharedGeometry = Rc::default();
        let shift: SharedShift = Rc::default();
        let theme = ThemeHandle::of(cx);
        let style = self.style.unwrap_or_else(|| TextStyle {
            size: 16.0,
            ..theme.snapshot().text.body
        });
        let style = cx.text_style(style);
        let on_change = Edits {
            on_change: self.on_change,
            formatters: self.formatters.into(),
        };
        let on_submit = self.on_submit;

        let editor = Editor {
            controller,
            style,
            max_lines: self.max_lines,
            single_line,
            placeholder: self.placeholder.clone(),
            enabled: Rc::clone(&enabled),
            focused,
            geometry: Rc::clone(&geometry),
            shift: Rc::clone(&shift),
            theme,
            caret_visible,
        };
        let placeholder = self.placeholder;
        let content = Padding::new(PADDING).child(editor);
        FieldBox {
            content: AnyView::new(Sizer {
                theme,
                content: AnyView::new(content),
            }),
            theme,
            enabled: Rc::clone(&enabled),
            focused,
            caret_visible,
            controller,
            single_line,
            on_change,
            on_submit,
            geometry,
            shift,
            dragging: Rc::new(dragging),
            anchor: Rc::new(anchor),
            placeholder,
            decoration: self.decoration,
        }
        .build(cx)
    }
}

/// The element that fills the width and is at least `MIN_HEIGHT` tall, adjusted by the
/// theme's density (WIDGETS-THEME-03).
struct Sizer {
    theme: ThemeHandle,
    content: AnyView,
}

impl View for Sizer {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let minimum =
            |height: f32| BoxConstraints::new(f32::INFINITY, f32::INFINITY, height, f32::INFINITY);
        let id = cx.render(
            RenderConstrainedBox::new(minimum(MIN_HEIGHT)),
            [self.content],
        );
        cx.bind(id, self.theme.density_prop(), move |e, density| {
            e.update_render::<RenderConstrainedBox>(|r| {
                r.additional = minimum(density.apply(MIN_HEIGHT));
            });
        });
        id
    }
}

/// The field's box: the outline, focus, handlers and chords (WIDGETS-TEXTFIELD-02..06).
struct FieldBox {
    content: AnyView,
    theme: ThemeHandle,
    enabled: Rc<dyn Fn() -> bool>,
    focused: Signal<bool>,
    caret_visible: Signal<bool>,
    controller: TextEditingController,
    single_line: bool,
    on_change: Edits,
    on_submit: Option<Rc<dyn Fn(String)>>,
    geometry: SharedGeometry,
    shift: SharedShift,
    dragging: Rc<Cell<bool>>,
    anchor: Rc<Cell<usize>>,
    /// The accessible name (WIDGETS-SEM-02).
    placeholder: Option<String>,
    decoration: Option<FieldDecoration>,
}

impl View for FieldBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (enabled, focused, controller) = (self.enabled, self.focused, self.controller);
        let id = cx.render(
            RenderLimitedBox::new(MAX_WIDTH, f32::INFINITY),
            [self.content],
        );
        let theme = self.theme;
        // Semantics: a text input named by the placeholder with the text as its value, and
        // `SetValue` replacing the text (WIDGETS-SEM-02).
        cx.set_semantics(
            id,
            Semantics {
                role: match self.decoration.as_ref().and_then(|d| d.role) {
                    Some(role) => role,
                    None if self.single_line => Role::TextInput,
                    None => Role::MultilineTextInput,
                },
                name: self
                    .decoration
                    .as_ref()
                    .and_then(|d| d.name.clone())
                    .or_else(|| self.placeholder.clone()),
                focusable: true,
                actions: SemanticsActions {
                    set_value: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || controller.text())),
            |e, text: String| {
                e.update_semantics(|s| s.value = Some(text));
            },
        );
        {
            let enabled = Rc::clone(&enabled);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || !enabled())),
                |e, disabled| {
                    e.update_semantics(|s| s.disabled = disabled);
                },
            );
        }
        {
            let (enabled, on_change) = (Rc::clone(&enabled), self.on_change.clone());
            cx.on_action(id, move |a| match a.action {
                SemanticsAction::SetValue(text) if enabled() => {
                    apply(controller, &on_change, |v| v.select_all().insert(text));
                    true
                }
                _ => false,
            });
        }
        cx.set_paint(
            id,
            FieldPaint {
                enabled: true,
                focused: false,
                error: false,
                colors: theme.colors(),
                radius: theme.shape().extra_small,
            },
        );
        // A combo box's popup state and highlighted option (WIDGETS-COMBO-07).
        if let Some(expanded) = self.decoration.as_ref().and_then(|d| d.expanded.clone()) {
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || expanded())),
                |e, expanded| {
                    e.update_semantics(|s| s.expanded = Some(expanded));
                },
            );
        }
        if let Some(active) = self.decoration.as_ref().and_then(|d| d.active.clone()) {
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || active())),
                |e, active| {
                    e.update_semantics(|s| s.active_descendant = active);
                },
            );
        }
        // A form field's error: the outline, `invalid` and the description (WIDGETS-FORM-01, -06).
        if let Some(decoration) = self.decoration.clone() {
            let helper = decoration.helper.clone();
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || (decoration.error)())),
                move |e, error: Option<String>| {
                    let invalid = error.is_some();
                    let description = error.or_else(|| helper.clone());
                    e.update_semantics(|s| {
                        s.invalid = invalid;
                        s.description = description;
                    });
                    e.update_paint::<FieldPaint>(|p| p.error = invalid);
                },
            );
        }
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (theme.colors(), theme.shape().extra_small)
            })),
            |e, (colors, radius)| {
                e.update_paint::<FieldPaint>(|p| {
                    p.colors = colors;
                    p.radius = radius;
                });
            },
        );
        {
            let enabled = Rc::clone(&enabled);
            cx.bind(id, Prop::Dynamic(Box::new(move || enabled())), |e, on| {
                e.update_paint::<FieldPaint>(|p| p.enabled = on);
                e.set_focusable(on.then_some(FocusOptions {
                    traversable: true,
                    focus_on_press: true,
                }));
            });
        }
        cx.on_focus_change(id, move |is_focused| focused.set(is_focused));
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || focused.get())),
            |e, on| {
                e.update_paint::<FieldPaint>(|p| p.focused = on);
            },
        );
        // The field's scope owns its timers and the blink effect, so they die with it.
        let scope = cx.tree().scope(id);
        // The caret blinks while focused, restarting on every edit or caret move
        // (WIDGETS-TEXTFIELD-10).
        if let Some(scope) = scope {
            let blink = Rc::new(Blink {
                scope,
                visible: self.caret_visible,
                pending: Cell::new(None),
            });
            let enabled = Rc::clone(&enabled);
            scope.run(|| {
                effect(move || {
                    let on = focused.get() && enabled();
                    // Tracks the text and the selection.
                    let _ = controller.value();
                    if on {
                        blink.restart();
                    } else {
                        blink.stop();
                    }
                });
            });
        }
        // Chords (WIDGETS-TEXTFIELD-05).
        cx.shortcut(
            id,
            SingleActivator::character("a").primary(),
            SelectAllIntent,
        );
        cx.shortcut(id, SingleActivator::character("z").primary(), UndoIntent);
        cx.shortcut(
            id,
            SingleActivator::character("z").primary().shift(),
            RedoIntent,
        );
        cx.shortcut(id, SingleActivator::character("y").primary(), RedoIntent);
        cx.shortcut(id, SingleActivator::character("c").primary(), CopyIntent);
        cx.shortcut(id, SingleActivator::character("x").primary(), CutIntent);
        cx.shortcut(id, SingleActivator::character("v").primary(), PasteIntent);
        // Copy, cut and paste (WIDGETS-TEXTFIELD-12).
        let clipboard = cx.clipboard();
        {
            let (enabled, clipboard) = (Rc::clone(&enabled), Rc::clone(&clipboard));
            cx.action(id, move |_: &CopyIntent| {
                let value = controller.value();
                if enabled() && !value.selection.is_collapsed() {
                    clipboard.write_text(value.selected_text());
                }
            });
        }
        {
            let (enabled, clipboard, on_change) = (
                Rc::clone(&enabled),
                Rc::clone(&clipboard),
                self.on_change.clone(),
            );
            cx.action(id, move |_: &CutIntent| {
                let value = controller.value();
                if enabled() && !value.selection.is_collapsed() {
                    clipboard.write_text(value.selected_text());
                    apply(controller, &on_change, |v| v.insert(""));
                }
            });
        }
        {
            let (enabled, on_change) = (Rc::clone(&enabled), self.on_change.clone());
            let single_line = self.single_line;
            cx.action(id, move |_: &PasteIntent| {
                if !enabled() {
                    return;
                }
                let Some(text) = clipboard.read_text() else {
                    return;
                };
                let text = if single_line {
                    text.replace("\r\n", " ").replace(['\n', '\r'], " ")
                } else {
                    text
                };
                apply(controller, &on_change, |v| v.insert(&text));
            });
        }
        {
            let enabled = Rc::clone(&enabled);
            cx.action(id, move |_: &SelectAllIntent| {
                if enabled() {
                    controller.edit(|v| v.select_all());
                }
            });
        }
        {
            let (enabled, on_change) = (Rc::clone(&enabled), self.on_change.clone());
            cx.action(id, move |_: &UndoIntent| {
                if enabled() {
                    let before = controller.text();
                    controller.undo();
                    report_if_changed(controller, &on_change, &before);
                }
            });
        }
        {
            let (enabled, on_change) = (Rc::clone(&enabled), self.on_change.clone());
            cx.action(id, move |_: &RedoIntent| {
                if enabled() {
                    let before = controller.text();
                    controller.redo();
                    report_if_changed(controller, &on_change, &before);
                }
            });
        }
        // Keys (WIDGETS-TEXTFIELD-04).
        {
            let (enabled, on_change, on_submit) = (
                Rc::clone(&enabled),
                self.on_change.clone(),
                self.on_submit.clone(),
            );
            let (geometry, single_line) = (Rc::clone(&self.geometry), self.single_line);
            cx.on_key(id, Phase::Bubble, move |k| {
                let event = k.event;
                if !event.pressed || !enabled() {
                    return Handled::Continue;
                }
                let modifiers = event.modifiers;
                let select = modifiers.shift;
                let unit = if modifiers.control {
                    Movement::Word
                } else {
                    Movement::Grapheme
                };
                match &event.key {
                    LogicalKey::Named(NamedKey::Backspace) => {
                        apply(controller, &on_change, |v| {
                            v.delete(Direction::Backward, unit)
                        });
                    }
                    LogicalKey::Named(NamedKey::Delete) => {
                        apply(controller, &on_change, |v| {
                            v.delete(Direction::Forward, unit)
                        });
                    }
                    LogicalKey::Named(NamedKey::ArrowLeft) => {
                        controller.edit(|v| v.move_caret(Direction::Backward, unit, select));
                    }
                    LogicalKey::Named(NamedKey::ArrowRight) => {
                        controller.edit(|v| v.move_caret(Direction::Forward, unit, select));
                    }
                    LogicalKey::Named(NamedKey::Home) => {
                        controller.edit(|v| {
                            v.move_caret(Direction::Backward, Movement::LineBoundary, select)
                        });
                    }
                    LogicalKey::Named(NamedKey::End) => {
                        controller.edit(|v| {
                            v.move_caret(Direction::Forward, Movement::LineBoundary, select)
                        });
                    }
                    LogicalKey::Named(NamedKey::ArrowUp) => {
                        move_vertically(
                            controller,
                            &geometry.borrow(),
                            Direction::Backward,
                            select,
                        );
                    }
                    LogicalKey::Named(NamedKey::ArrowDown) => {
                        move_vertically(controller, &geometry.borrow(), Direction::Forward, select);
                    }
                    LogicalKey::Named(NamedKey::Enter) => {
                        if single_line {
                            if let Some(on_submit) = &on_submit {
                                on_submit(controller.text());
                            }
                        } else {
                            apply(controller, &on_change, |v| v.insert("\n"));
                        }
                    }
                    LogicalKey::Character(character) => {
                        if modifiers.control || modifiers.super_key || modifiers.alt {
                            return Handled::Continue;
                        }
                        let text: Arc<str> =
                            event.text.clone().unwrap_or_else(|| character.clone());
                        apply(controller, &on_change, |v| v.insert(&text));
                    }
                    _ => return Handled::Continue,
                }
                Handled::Stop
            });
        }
        // The pointer (WIDGETS-TEXTFIELD-06), with double-click word selection
        // (WIDGETS-TEXTFIELD-11): a pending 500 ms timer is the click interval.
        {
            let (geometry, dragging, anchor) =
                (Rc::clone(&self.geometry), self.dragging, self.anchor);
            let (enabled, shift) = (Rc::clone(&enabled), Rc::clone(&self.shift));
            let clicks = Cell::new(0u8);
            let last_press = Cell::new(Point::ZERO);
            let interval: Cell<Option<Timer>> = Cell::new(None);
            let word_anchor: Cell<Option<TextRange>> = Cell::new(None);
            cx.on_pointer(id, Phase::Bubble, move |p| {
                let local = Point::new(
                    p.local.x - PADDING.left + shift.get(),
                    p.local.y - PADDING.top,
                );
                match p.event.kind {
                    PointerKind::Down(PointerButton::Primary) if enabled() => {
                        let near = (p.local.x - last_press.get().x)
                            .hypot(p.local.y - last_press.get().y)
                            <= DOUBLE_CLICK_SLOP;
                        let within = interval.get().is_some_and(Timer::is_pending);
                        let count = if within && near { clicks.get() + 1 } else { 1 };
                        // A third press starts over (triple-click is later).
                        let count = if count > 2 { 1 } else { count };
                        clicks.set(count);
                        last_press.set(p.local);
                        if let Some(timer) = interval.take() {
                            timer.cancel();
                        }
                        if let Some(scope) = scope.filter(|s| !s.is_disposed()) {
                            interval.set(Some(scope.run(|| timer(DOUBLE_CLICK_INTERVAL, || {}))));
                        }
                        let offset = geometry.borrow().hit_test(local);
                        if count == 2 {
                            let word = segment_at(&controller.text(), offset);
                            word_anchor.set(Some(word));
                            anchor.set(word.start);
                            controller.select(TextSelection::new(word.start, word.end));
                        } else {
                            word_anchor.set(None);
                            anchor.set(offset);
                            controller.select(TextSelection::collapsed(offset));
                        }
                        dragging.set(true);
                        Handled::Stop
                    }
                    PointerKind::Move if dragging.get() => {
                        let offset = geometry.borrow().hit_test(local);
                        let selection = match word_anchor.get() {
                            // Extend by whole words after a double-click.
                            Some(word) => {
                                let here = segment_at(&controller.text(), offset);
                                if offset >= word.start {
                                    TextSelection::new(word.start, here.end)
                                } else {
                                    TextSelection::new(word.end, here.start)
                                }
                            }
                            None => TextSelection::new(anchor.get(), offset),
                        };
                        controller.select(selection);
                        Handled::Stop
                    }
                    PointerKind::Up(PointerButton::Primary) if dragging.get() => {
                        dragging.set(false);
                        Handled::Stop
                    }
                    _ => Handled::Continue,
                }
            });
        }
        // The IME (WIDGETS-TEXTFIELD-07).
        {
            let (enabled, on_change) = (Rc::clone(&enabled), self.on_change.clone());
            cx.on_ime(id, move |i| {
                if !enabled() {
                    return;
                }
                match i.event {
                    ImeEvent::Preedit { text, cursor } => {
                        apply(controller, &on_change, |v| v.compose(text, *cursor));
                    }
                    ImeEvent::Commit(text) => {
                        apply(controller, &on_change, |v| v.commit(text));
                    }
                    ImeEvent::Disabled => {
                        let value = controller.value();
                        if let Some(range) = value.composing {
                            let preedit = value.text[range.start..range.end].to_string();
                            apply(controller, &on_change, |v| v.commit(&preedit));
                        }
                    }
                    ImeEvent::Enabled => {}
                }
            });
        }
        {
            let (enabled, geometry, shift) = (
                Rc::clone(&enabled),
                Rc::clone(&self.geometry),
                Rc::clone(&self.shift),
            );
            cx.ime_area(id, move || {
                if !(focused.get_untracked() && enabled()) {
                    return None;
                }
                let caret = geometry.borrow().caret(controller.value().selection.extent);
                let left = caret.left + PADDING.left - shift.get();
                Some(Rect::from_ltrb(
                    left,
                    caret.top + PADDING.top,
                    left + CARET_WIDTH,
                    caret.bottom + PADDING.top,
                ))
            });
        }
        cx.set_cursor(id, CursorIcon::Text);
        id
    }
}

/// The caret blink: a chain of half-period timers in the field's scope
/// (WIDGETS-TEXTFIELD-10).
struct Blink {
    scope: Scope,
    visible: Signal<bool>,
    pending: Cell<Option<Timer>>,
}

impl Blink {
    /// Shows the caret and starts the cycle from the beginning.
    fn restart(self: &Rc<Self>) {
        self.stop();
        if !self.visible.get_untracked() {
            self.visible.set(true);
        }
        self.arm();
    }

    /// Cancels the next toggle (the caret stays as it is; the paint hides it when unfocused).
    fn stop(&self) {
        if let Some(timer) = self.pending.take() {
            timer.cancel();
        }
    }

    /// Toggles after half a period, then arms again.
    fn arm(self: &Rc<Self>) {
        if self.scope.is_disposed() {
            return;
        }
        let me = Rc::clone(self);
        let timer = self.scope.run(|| {
            timer(BLINK_HALF_PERIOD, move || {
                me.visible.update(|v| *v = !*v);
                me.arm();
            })
        });
        self.pending.set(Some(timer));
    }
}

/// Moves the caret to the previous or next line at its x (WIDGETS-TEXTFIELD-04).
fn move_vertically(
    controller: TextEditingController,
    geometry: &TextGeometry,
    direction: Direction,
    select: bool,
) {
    let value = controller.value();
    let caret = value.selection.extent;
    let Some(line) = geometry.line_of(caret) else {
        return;
    };
    let x = geometry.caret(caret).left;
    let target = match direction {
        Direction::Backward if line == 0 => 0,
        Direction::Backward => geometry.offset_at(line - 1, x),
        Direction::Forward if line + 1 >= geometry.lines.len() => value.text.len(),
        Direction::Forward => geometry.offset_at(line + 1, x),
    };
    let selection = if select {
        TextSelection::new(value.selection.base, target)
    } else {
        TextSelection::collapsed(target)
    };
    controller.select(selection);
}

/// Paints the outline and clips the content (WIDGETS-TEXTFIELD-02).
struct FieldPaint {
    enabled: bool,
    focused: bool,
    /// Showing a form error (WIDGETS-FORM-01).
    error: bool,
    colors: ColorScheme,
    radius: f32,
}

impl Paint for FieldPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let rect = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        let shape = RoundedRect::new(rect, BorderRadius::circular(self.radius));
        let colors = &self.colors;
        let (width, color) = match (self.enabled, self.focused) {
            (false, _) => (1.0, at(colors.on_surface, ALPHA_12)),
            (true, true) if self.error => (2.0, colors.error),
            (true, false) if self.error => (1.0, colors.error),
            (true, true) => (2.0, colors.primary),
            (true, false) => (1.0, colors.outline),
        };
        cx.scene().stroke(shape, width, color);
        cx.scene().push_clip(Clip::Rect(rect));
        cx.paint_children();
        cx.scene().pop();
    }
}

/// The editor element: the paragraph showing the controller's text (or the placeholder).
struct Editor {
    controller: TextEditingController,
    style: TextStyleKey,
    max_lines: Option<u32>,
    single_line: bool,
    placeholder: Option<String>,
    enabled: Rc<dyn Fn() -> bool>,
    focused: Signal<bool>,
    geometry: SharedGeometry,
    shift: SharedShift,
    theme: ThemeHandle,
    caret_visible: Signal<bool>,
}

impl View for Editor {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let mut paragraph = RenderParagraph::new("", self.style);
        paragraph.max_lines = self.max_lines;
        paragraph.soft_wrap = !self.single_line;
        let id = cx.render(paragraph, []);
        cx.set_paint(
            id,
            EditorPaint {
                geometry: self.geometry,
                shift: self.shift,
                empty: true,
                enabled: true,
                focused: false,
                selection: TextSelection::default(),
                composing: None,
                colors: self.theme.colors(),
                caret_visible: true,
            },
        );
        let caret_visible = self.caret_visible;
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || caret_visible.get())),
            |e, on| {
                e.update_paint::<EditorPaint>(|p| p.caret_visible = on);
            },
        );
        cx.bind(id, self.theme.colors_prop(), |e, colors| {
            e.update_paint::<EditorPaint>(|p| p.colors = colors);
        });
        let (controller, placeholder) = (self.controller, self.placeholder);
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || controller.text())),
            move |e, text: String| {
                let empty = text.is_empty();
                let shown = if empty {
                    placeholder.clone().unwrap_or_default()
                } else {
                    text
                };
                e.update_render::<RenderParagraph>(|p| p.text = Arc::from(shown));
                e.update_paint::<EditorPaint>(|p| p.empty = empty);
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || controller.value().selection)),
            |e, selection| {
                e.update_paint::<EditorPaint>(|p| p.selection = selection);
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || controller.value().composing)),
            |e, composing| {
                e.update_paint::<EditorPaint>(|p| p.composing = composing);
            },
        );
        let focused = self.focused;
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || focused.get())),
            |e, on| {
                e.update_paint::<EditorPaint>(|p| p.focused = on);
            },
        );
        let enabled = self.enabled;
        cx.bind(id, Prop::Dynamic(Box::new(move || enabled())), |e, on| {
            e.update_paint::<EditorPaint>(|p| p.enabled = on);
        });
        id
    }
}

/// Paints the selection, the text and the caret, and records the geometry
/// (WIDGETS-TEXTFIELD-02).
struct EditorPaint {
    geometry: SharedGeometry,
    shift: SharedShift,
    empty: bool,
    enabled: bool,
    focused: bool,
    selection: TextSelection,
    composing: Option<TextRange>,
    colors: ColorScheme,
    /// The blink's phase (WIDGETS-TEXTFIELD-10).
    caret_visible: bool,
}

impl Paint for EditorPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let Some(paragraph) = cx.render::<RenderParagraph>() else {
            return;
        };
        let (text, style, max_lines, soft_wrap) = (
            paragraph.text.clone(),
            paragraph.style,
            paragraph.max_lines,
            paragraph.soft_wrap,
        );
        let width = if soft_wrap {
            cx.constraints().max_width
        } else {
            f32::INFINITY
        };
        // The geometry of the text, or of a space for an empty field (the caret's height).
        let measured: &str = if self.empty { " " } else { &text };
        let mut geometry = cx
            .text_geometry(measured, style, &[], width, max_lines)
            .unwrap_or_default();
        if self.empty {
            for line in &mut geometry.lines {
                line.clusters.clear();
                line.range = 0..0;
            }
        }
        let active = self.focused && self.enabled;
        // Caret following in a single-line field (WIDGETS-TEXTFIELD-09).
        let mut shift = self.shift.get();
        if soft_wrap || self.empty {
            shift = 0.0;
        } else {
            let caret = geometry.caret(self.selection.extent);
            let text_width = geometry
                .lines
                .first()
                .and_then(|line| line.clusters.last())
                .map_or(0.0, |cluster| cluster.right);
            let width = cx.size().width;
            let max_shift = (text_width + CARET_WIDTH - width).max(0.0);
            if caret.left - shift > width - CARET_WIDTH {
                shift = caret.left - (width - CARET_WIDTH);
            }
            if caret.left - shift < 0.0 {
                shift = caret.left;
            }
            shift = shift.clamp(0.0, max_shift);
        }
        self.shift.set(shift);
        cx.scene()
            .push_transform(Affine::translate(Vec2::new(-shift, 0.0)));
        let colors = &self.colors;
        if active && !self.empty {
            for rect in geometry.selection_rects(self.selection.start(), self.selection.end()) {
                cx.scene().fill_rect(rect, colors.primary.with_alpha(0.4));
            }
        }
        let color = if !self.enabled {
            at(colors.on_surface, ALPHA_38)
        } else if self.empty {
            colors.on_surface_variant
        } else {
            colors.on_surface
        };
        cx.paint_text(&text, style, width, max_lines, color, Point::ZERO);
        // The composing range is underlined (WIDGETS-TEXTFIELD-08).
        if let Some(range) = self.composing.filter(|_| active && !self.empty) {
            for rect in geometry.selection_rects(range.start, range.end) {
                cx.scene().fill_rect(
                    Rect::from_ltrb(rect.left, rect.bottom - 1.0, rect.right, rect.bottom),
                    colors.primary,
                );
            }
        }
        if active && self.caret_visible {
            let caret = geometry.caret(self.selection.extent);
            cx.scene().fill_rect(
                Rect::from_ltrb(
                    caret.left,
                    caret.top,
                    caret.left + CARET_WIDTH,
                    caret.bottom,
                ),
                colors.primary,
            );
        }
        cx.scene().pop();
        *self.geometry.borrow_mut() = geometry;
    }
}
