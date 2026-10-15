//! Forms: [`Form`], [`FormController`], [`TextFormField`] and [`validators`]. Spec:
//! `docs/specs/widgets/form.md`.

use std::rc::Rc;

use tantu_view::{AnyView, BuildCx, ElementId, View};

use crate::TextEditingController;

/// Checks a field's text: `None` when valid, else the message to show.
pub type Validator = Rc<dyn Fn(&str) -> Option<String>>;

/// Ready-made validators.
pub mod validators {
    use super::Validator;

    /// Fails on empty (or whitespace-only) text.
    pub fn required(message: impl Into<String>) -> Validator {
        let _ = message.into();
        todo!("WIDGETS-FORM-02")
    }

    /// Fails when the text has fewer than `length` characters.
    pub fn min_length(length: usize, message: impl Into<String>) -> Validator {
        let _ = (length, message.into());
        todo!("WIDGETS-FORM-02")
    }

    /// Fails when the text has more than `length` characters.
    pub fn max_length(length: usize, message: impl Into<String>) -> Validator {
        let _ = (length, message.into());
        todo!("WIDGETS-FORM-02")
    }

    /// Fails when `test` returns false for the text.
    pub fn custom(test: impl Fn(&str) -> bool + 'static, message: impl Into<String>) -> Validator {
        let _ = (Box::new(test) as Box<dyn Fn(&str) -> bool>, message.into());
        todo!("WIDGETS-FORM-02")
    }
}

/// When a field checks itself without [`FormController::validate`] (WIDGETS-FORM-03).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AutovalidateMode {
    /// Only `validate` and `reset` change the error.
    #[default]
    Disabled,
    /// Every change after the user's first edit.
    OnUserInteraction,
    /// Every change, and when the field builds.
    Always,
}

/// Validates and resets the fields of a [`Form`]; `Copy` (WIDGETS-FORM-02..04).
#[derive(Clone, Copy)]
pub struct FormController {
    _private: (),
}

impl Default for FormController {
    fn default() -> Self {
        FormController::new()
    }
}

impl FormController {
    /// A controller with no form yet.
    pub fn new() -> Self {
        todo!("WIDGETS-FORM-02")
    }

    /// Validates every field, shows their errors and focuses the first invalid one; true
    /// when all are valid.
    pub fn validate(&self) -> bool {
        todo!("WIDGETS-FORM-02")
    }

    /// Restores every field's initial text and clears the errors.
    pub fn reset(&self) {
        todo!("WIDGETS-FORM-04")
    }

    /// Whether every field is valid now (tracked; shows nothing).
    pub fn is_valid(&self) -> bool {
        todo!("WIDGETS-FORM-04")
    }
}

/// Groups the form fields inside `child` under a controller.
pub struct Form {
    controller: FormController,
    child: AnyView,
    autovalidate: AutovalidateMode,
}

impl Form {
    /// A form of the fields inside `child`.
    pub fn new(controller: FormController, child: impl View) -> Self {
        Form {
            controller,
            child: AnyView::new(child),
            autovalidate: AutovalidateMode::Disabled,
        }
    }

    /// The fields' default autovalidate mode (default `Disabled`).
    pub fn autovalidate(mut self, mode: AutovalidateMode) -> Self {
        self.autovalidate = mode;
        self
    }
}

impl View for Form {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.controller, self.child, self.autovalidate);
        todo!("WIDGETS-FORM-02")
    }
}

/// A labelled `TextField` with validation (Flutter's `TextFormField`).
pub struct TextFormField {
    controller: TextEditingController,
    label: Option<String>,
    helper: Option<String>,
    placeholder: Option<String>,
    validators: Vec<Validator>,
    autovalidate: Option<AutovalidateMode>,
    max_lines: Option<Option<u32>>,
    on_change: Option<Rc<dyn Fn(String)>>,
}

impl TextFormField {
    /// A field editing `controller`; its text when built is what `reset` restores.
    pub fn new(controller: TextEditingController) -> Self {
        TextFormField {
            controller,
            label: None,
            helper: None,
            placeholder: None,
            validators: Vec::new(),
            autovalidate: None,
            max_lines: None,
            on_change: None,
        }
    }

    /// The label above the field (also its accessible name).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Helper text under the field, shown when there is no error.
    pub fn helper(mut self, helper: impl Into<String>) -> Self {
        self.helper = Some(helper.into());
        self
    }

    /// Placeholder text shown while the field is empty.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Adds a validator; they run in order and the first message shows.
    pub fn validator(mut self, validator: Validator) -> Self {
        self.validators.push(validator);
        self
    }

    /// Overrides the form's autovalidate mode for this field.
    pub fn autovalidate(mut self, mode: AutovalidateMode) -> Self {
        self.autovalidate = Some(mode);
        self
    }

    /// As [`TextField::max_lines`](crate::TextField::max_lines).
    pub fn max_lines(mut self, max_lines: Option<u32>) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Called with the new text after each edit.
    pub fn on_change(mut self, f: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl View for TextFormField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.controller,
            self.label,
            self.helper,
            self.placeholder,
            self.validators,
            self.autovalidate,
            self.max_lines,
            self.on_change,
        );
        todo!("WIDGETS-FORM-01")
    }
}
