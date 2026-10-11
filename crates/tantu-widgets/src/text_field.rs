//! [`TextField`]. Spec: `docs/specs/widgets/text-field.md`.

use std::rc::Rc;

use tantu_view::text::TextStyle;
use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

use crate::TextEditingController;

/// Select the whole text (bound to primary + A in a field).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectAllIntent;

/// Undo the last edit (primary + Z).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UndoIntent;

/// Redo the last undone edit (primary + Shift + Z, primary + Y).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RedoIntent;

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

impl View for TextField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.controller,
            self.placeholder,
            self.max_lines,
            self.enabled,
            self.on_change,
            self.on_submit,
            self.style,
        );
        todo!("WIDGETS-TEXTFIELD-01")
    }
}
