//! [`TextField`]. Spec: `docs/specs/widgets/text-field.md`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use tantu_view::core::{Color, EdgeInsets, Point, Rect};
use tantu_view::layout::{
    BoxConstraints, RenderConstrainedBox, RenderLimitedBox, RenderParagraph, TextGeometry,
    TextStyleKey,
};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{BorderRadius, Clip, RoundedRect};
use tantu_view::text::TextStyle;
use tantu_view::{
    AnyView, BuildCx, CursorIcon, ElementId, FocusOptions, Handled, ImeEvent, IntoProp, LogicalKey,
    NamedKey, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, SingleActivator, View,
};

use crate::text_editing::{Direction, EditingValue, Movement, TextRange, TextSelection};
use crate::util::dynamic;
use crate::{Padding, TextEditingController};

/// Select the whole text (bound to primary + A in a field).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectAllIntent;

/// Undo the last edit (primary + Z).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UndoIntent;

/// Redo the last undone edit (primary + Shift + Z, primary + Y).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RedoIntent;

/// The field's width in an unbounded width.
const MAX_WIDTH: f32 = 280.0;
/// The field's minimum height.
const MIN_HEIGHT: f32 = 40.0;
/// The content padding (horizontal, vertical).
const PADDING: EdgeInsets = EdgeInsets::symmetric(16.0, 8.0);
/// The outline's corner radius.
const RADIUS: f32 = 4.0;
/// The caret's width.
const CARET_WIDTH: f32 = 2.0;

const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
const OUTLINE: Color = Color::from_argb32(0xFF79_747E);
const ON_SURFACE: Color = Color::from_argb32(0xFF1C_1B1F);
const ON_SURFACE_VARIANT: Color = Color::from_argb32(0xFF49_454F);
const ON_SURFACE_38: Color = Color::from_argb32(0x611C_1B1F);
const ON_SURFACE_12: Color = Color::from_argb32(0x1F1C_1B1F);

/// A text input (Flutter's `TextField`, Material 3 outlined look), editing a
/// [`TextEditingController`].
pub struct TextField {
    controller: TextEditingController,
    placeholder: Option<String>,
    max_lines: Option<u32>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(String)>>,
    on_submit: Option<Rc<dyn Fn(String)>>,
    style: TextStyle,
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
            style: TextStyle {
                size: 16.0,
                ..TextStyle::body()
            },
        }
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

    /// The text style (default `TextStyle::body()` at 16 px).
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }
}

/// What the handlers share with the editor's paint.
type SharedGeometry = Rc<RefCell<TextGeometry>>;

/// Applies `f` to the controller and reports a text change (WIDGETS-TEXTFIELD-04).
fn apply(
    controller: TextEditingController,
    on_change: &Option<Rc<dyn Fn(String)>>,
    f: impl FnOnce(&EditingValue) -> EditingValue,
) {
    let before = controller.value();
    controller.edit(f);
    let after = controller.value();
    if after.text != before.text {
        if let Some(on_change) = on_change {
            on_change(after.text);
        }
    }
}

/// Reports a text change made by undo or redo.
fn report_if_changed(
    controller: TextEditingController,
    on_change: &Option<Rc<dyn Fn(String)>>,
    before: &str,
) {
    let after = controller.value();
    if after.text != before {
        if let Some(on_change) = on_change {
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
        let dragging = Cell::new(false);
        let anchor = Cell::new(0usize);
        let geometry: SharedGeometry = Rc::default();
        let style = cx.text_style(self.style);
        let on_change = self.on_change;
        let on_submit = self.on_submit;

        let editor = Editor {
            controller,
            style,
            max_lines: self.max_lines,
            single_line,
            placeholder: self.placeholder,
            enabled: Rc::clone(&enabled),
            focused,
            geometry: Rc::clone(&geometry),
        };
        let content = Padding::new(PADDING).child(editor);
        let constraints =
            BoxConstraints::new(f32::INFINITY, f32::INFINITY, MIN_HEIGHT, f32::INFINITY);
        FieldBox {
            content: AnyView::new(Sizer {
                constraints,
                content: AnyView::new(content),
            }),
            enabled: Rc::clone(&enabled),
            focused,
            controller,
            single_line,
            on_change,
            on_submit,
            geometry,
            dragging: Rc::new(dragging),
            anchor: Rc::new(anchor),
        }
        .build(cx)
    }
}

/// The element that fills the width and is at least `MIN_HEIGHT` tall.
struct Sizer {
    constraints: BoxConstraints,
    content: AnyView,
}

impl View for Sizer {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderConstrainedBox::new(self.constraints), [self.content])
    }
}

/// The field's box: the outline, focus, handlers and chords (WIDGETS-TEXTFIELD-02..06).
struct FieldBox {
    content: AnyView,
    enabled: Rc<dyn Fn() -> bool>,
    focused: Signal<bool>,
    controller: TextEditingController,
    single_line: bool,
    on_change: Option<Rc<dyn Fn(String)>>,
    on_submit: Option<Rc<dyn Fn(String)>>,
    geometry: SharedGeometry,
    dragging: Rc<Cell<bool>>,
    anchor: Rc<Cell<usize>>,
}

impl View for FieldBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (enabled, focused, controller) = (self.enabled, self.focused, self.controller);
        let id = cx.render(
            RenderLimitedBox::new(MAX_WIDTH, f32::INFINITY),
            [self.content],
        );
        cx.set_paint(
            id,
            FieldPaint {
                enabled: true,
                focused: false,
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
        // The pointer (WIDGETS-TEXTFIELD-06).
        {
            let (geometry, dragging, anchor) =
                (Rc::clone(&self.geometry), self.dragging, self.anchor);
            let enabled = Rc::clone(&enabled);
            cx.on_pointer(id, Phase::Bubble, move |p| {
                let local = Point::new(p.local.x - PADDING.left, p.local.y - PADDING.top);
                match p.event.kind {
                    PointerKind::Down(PointerButton::Primary) if enabled() => {
                        let offset = geometry.borrow().hit_test(local);
                        anchor.set(offset);
                        dragging.set(true);
                        controller.select(TextSelection::collapsed(offset));
                        Handled::Stop
                    }
                    PointerKind::Move if dragging.get() => {
                        let offset = geometry.borrow().hit_test(local);
                        controller.select(TextSelection::new(anchor.get(), offset));
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
            let (enabled, geometry) = (Rc::clone(&enabled), Rc::clone(&self.geometry));
            cx.ime_area(id, move || {
                if !(focused.get_untracked() && enabled()) {
                    return None;
                }
                let caret = geometry.borrow().caret(controller.value().selection.extent);
                Some(Rect::from_ltrb(
                    caret.left + PADDING.left,
                    caret.top + PADDING.top,
                    caret.left + PADDING.left + CARET_WIDTH,
                    caret.bottom + PADDING.top,
                ))
            });
        }
        cx.set_cursor(id, CursorIcon::Text);
        id
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
}

impl Paint for FieldPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let rect = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        let shape = RoundedRect::new(rect, BorderRadius::circular(RADIUS));
        let (width, color) = match (self.enabled, self.focused) {
            (false, _) => (1.0, ON_SURFACE_12),
            (true, true) => (2.0, PRIMARY),
            (true, false) => (1.0, OUTLINE),
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
                empty: true,
                enabled: true,
                focused: false,
                selection: TextSelection::default(),
                composing: None,
            },
        );
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
    empty: bool,
    enabled: bool,
    focused: bool,
    selection: TextSelection,
    composing: Option<TextRange>,
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
        if active && !self.empty {
            for rect in geometry.selection_rects(self.selection.start(), self.selection.end()) {
                cx.scene().fill_rect(rect, PRIMARY.with_alpha(0.4));
            }
        }
        let color = if !self.enabled {
            ON_SURFACE_38
        } else if self.empty {
            ON_SURFACE_VARIANT
        } else {
            ON_SURFACE
        };
        cx.paint_text(&text, style, width, max_lines, color, Point::ZERO);
        // The composing range is underlined (WIDGETS-TEXTFIELD-08).
        if let Some(range) = self.composing.filter(|_| active && !self.empty) {
            for rect in geometry.selection_rects(range.start, range.end) {
                cx.scene().fill_rect(
                    Rect::from_ltrb(rect.left, rect.bottom - 1.0, rect.right, rect.bottom),
                    PRIMARY,
                );
            }
        }
        if active {
            let caret = geometry.caret(self.selection.extent);
            cx.scene().fill_rect(
                Rect::from_ltrb(
                    caret.left,
                    caret.top,
                    caret.left + CARET_WIDTH,
                    caret.bottom,
                ),
                PRIMARY,
            );
        }
        *self.geometry.borrow_mut() = geometry;
    }
}
