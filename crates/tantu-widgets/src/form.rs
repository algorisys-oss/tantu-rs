//! Forms: [`Form`], [`FormController`], [`TextFormField`] and [`validators`]. Spec:
//! `docs/specs/widgets/form.md`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_view::layout::CrossAxisAlignment;
use tantu_view::reactive::{Signal, effect, on_cleanup, signal, untrack};
use tantu_view::{AnyView, BuildCx, Dyn, ElementId, Provide, View};

use crate::text_field::FieldDecoration;
use crate::theme::ThemeHandle;
use crate::{Column, Text, TextEditingController, TextField};

/// Space between a field and its label or message (WIDGETS-FORM-01).
const GAP: f32 = 4.0;

/// Checks a field's text: `None` when valid, else the message to show.
pub type Validator = Rc<dyn Fn(&str) -> Option<String>>;

/// Ready-made validators.
pub mod validators {
    use super::Validator;

    /// Fails on empty (or whitespace-only) text.
    pub fn required(message: impl Into<String>) -> Validator {
        custom(|text| !text.trim().is_empty(), message)
    }

    /// Fails when the text has fewer than `length` characters.
    pub fn min_length(length: usize, message: impl Into<String>) -> Validator {
        custom(move |text| text.chars().count() >= length, message)
    }

    /// Fails when the text has more than `length` characters.
    pub fn max_length(length: usize, message: impl Into<String>) -> Validator {
        custom(move |text| text.chars().count() <= length, message)
    }

    /// Fails when `test` returns false for the text.
    pub fn custom(test: impl Fn(&str) -> bool + 'static, message: impl Into<String>) -> Validator {
        let message = message.into();
        std::rc::Rc::new(move |text: &str| (!test(text)).then(|| message.clone()))
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
    inner: Signal<Rc<FormInner>>,
}

/// A field's validation: shows its error when asked; true when valid.
type ValidateFn = Rc<dyn Fn(bool) -> bool>;
/// Where a field's focus goes.
type Target = Rc<Cell<Option<ElementId>>>;

/// A registered field: validate (showing the error or not), reset, and where its focus goes.
struct FieldEntry {
    key: u64,
    validate: Rc<dyn Fn(bool) -> bool>,
    reset: Rc<dyn Fn()>,
    target: Rc<Cell<Option<ElementId>>>,
}

/// What a form's controller holds.
struct FormInner {
    fields: RefCell<Vec<FieldEntry>>,
    next: Cell<u64>,
    mode: Cell<AutovalidateMode>,
    /// The field to focus after `validate`, and the signal that moves the focus.
    target: Cell<Option<ElementId>>,
    moves: Signal<u32>,
    /// Bumped when fields register or leave, so `is_valid` readers follow.
    revision: Signal<u32>,
}

impl FormController {
    fn inner(&self) -> Rc<FormInner> {
        self.inner.get_untracked()
    }

    /// Registers a field; returns its key.
    fn register(&self, entry: impl FnOnce(u64) -> FieldEntry) -> u64 {
        let inner = self.inner();
        let key = inner.next.get();
        inner.next.set(key + 1);
        inner.fields.borrow_mut().push(entry(key));
        untrack(|| inner.revision.update(|r| *r = r.wrapping_add(1)));
        key
    }

    fn unregister(&self, key: u64) {
        let inner = self.inner();
        inner.fields.borrow_mut().retain(|f| f.key != key);
        untrack(|| inner.revision.update(|r| *r = r.wrapping_add(1)));
    }
}

impl Default for FormController {
    fn default() -> Self {
        FormController::new()
    }
}

impl FormController {
    /// A controller with no form yet.
    pub fn new() -> Self {
        FormController {
            inner: signal(Rc::new(FormInner {
                fields: RefCell::default(),
                next: Cell::new(0),
                mode: Cell::new(AutovalidateMode::Disabled),
                target: Cell::new(None),
                moves: signal(0),
                revision: signal(0),
            })),
        }
    }

    /// Validates every field, shows their errors and focuses the first invalid one; true
    /// when all are valid.
    pub fn validate(&self) -> bool {
        let inner = self.inner();
        let entries: Vec<(ValidateFn, Target)> = inner
            .fields
            .borrow()
            .iter()
            .map(|f| (Rc::clone(&f.validate), Rc::clone(&f.target)))
            .collect();
        let mut first = None;
        for (validate, target) in entries {
            if !validate(true) && first.is_none() {
                first = Some(target.get());
            }
        }
        match first {
            None => true,
            Some(target) => {
                // The first invalid field takes the focus (WIDGETS-FORM-02).
                inner.target.set(target);
                untrack(|| inner.moves.update(|m| *m = m.wrapping_add(1)));
                false
            }
        }
    }

    /// Restores every field's initial text and clears the errors.
    pub fn reset(&self) {
        let resets: Vec<Rc<dyn Fn()>> = self
            .inner()
            .fields
            .borrow()
            .iter()
            .map(|f| Rc::clone(&f.reset))
            .collect();
        for reset in resets {
            reset();
        }
    }

    /// Whether every field is valid now (tracked; shows nothing).
    pub fn is_valid(&self) -> bool {
        let inner = self.inner.get();
        inner.revision.get();
        let validates: Vec<Rc<dyn Fn(bool) -> bool>> = inner
            .fields
            .borrow()
            .iter()
            .map(|f| Rc::clone(&f.validate))
            .collect();
        validates.iter().all(|validate| validate(false))
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

/// Focuses the form's pending target once, at a frame.
struct FocusMove {
    controller: FormController,
}

impl View for FocusMove {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if let Some(target) = self.controller.inner().target.take() {
            cx.focus(target);
        }
        cx.region(|_| {})
    }
}

impl View for Form {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let controller = self.controller;
        controller.inner().mode.set(self.autovalidate);
        let moves = controller.inner().moves;
        let child = self.child;
        cx.region(move |cx| {
            Provide::new(controller, child).build(cx);
            Dyn::new(move || {
                moves.get();
                FocusMove { controller }
            })
            .build(cx);
        })
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

/// Builds a view and reports its element.
struct Capture {
    view: AnyView,
    out: Rc<Cell<Option<ElementId>>>,
}

impl View for Capture {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.view.build(cx);
        self.out.set(Some(id));
        id
    }
}

impl View for TextFormField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let form = cx.inherit::<FormController>();
        let mode = self
            .autovalidate
            .or_else(|| form.map(|f| f.inner().mode.get()))
            .unwrap_or_default();
        let controller = self.controller;
        let initial = untrack(|| controller.text());
        let validators: Rc<[Validator]> = self.validators.into();
        let check = {
            let validators = Rc::clone(&validators);
            Rc::new(move |text: &str| validators.iter().find_map(|v| v(text)))
        };
        let error: Signal<Option<String>> = signal(None);
        let interacted = signal(false);
        // Autovalidation as the text changes (WIDGETS-FORM-03).
        {
            let check = Rc::clone(&check);
            effect(move || {
                let text = controller.text();
                let validate = match mode {
                    AutovalidateMode::Always => true,
                    AutovalidateMode::OnUserInteraction => untrack(|| interacted.get()),
                    AutovalidateMode::Disabled => false,
                };
                if validate {
                    let found = check(&text);
                    if untrack(|| error.get()) != found {
                        error.set(found);
                    }
                }
            });
        }
        let target: Rc<Cell<Option<ElementId>>> = Rc::default();
        if let Some(form) = form {
            let check = Rc::clone(&check);
            let key = form.register(|key| FieldEntry {
                key,
                validate: Rc::new(move |show| {
                    let found = check(&controller.text());
                    let valid = found.is_none();
                    if show {
                        untrack(|| error.set(found));
                    }
                    valid
                }),
                reset: Rc::new(move || {
                    interacted.set(false);
                    controller.set_text(initial.clone());
                    error.set(None);
                }),
                target: Rc::clone(&target),
            });
            on_cleanup(move || form.unregister(key));
        }
        let helper = self.helper.clone();
        let mut field = TextField::new(controller).decoration(FieldDecoration {
            name: self.label.clone(),
            error: Rc::new(move || error.get()),
            helper: helper.clone(),
        });
        if let Some(placeholder) = self.placeholder {
            field = field.placeholder(placeholder);
        }
        if let Some(max_lines) = self.max_lines {
            field = field.max_lines(max_lines);
        }
        {
            let (check, on_change) = (Rc::clone(&check), self.on_change);
            field = field.on_change(move |text| {
                // The user's first edit starts OnUserInteraction validation.
                if !interacted.get_untracked() {
                    interacted.set(true);
                    if mode == AutovalidateMode::OnUserInteraction {
                        error.set(check(&text));
                    }
                }
                if let Some(f) = &on_change {
                    f(text);
                }
            });
        }
        let mut column = Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .spacing(GAP);
        if let Some(label) = self.label {
            column = column.child(
                Text::new(label)
                    .style(theme.snapshot().text.label)
                    .color(move || theme.colors().on_surface_variant),
            );
        }
        column = column.child(Capture {
            view: AnyView::new(field),
            out: target,
        });
        let message_style = theme.snapshot().text.label;
        column = column.child(Dyn::new(move || {
            let message = error
                .get()
                .map(|e| (e, true))
                .or_else(|| helper.clone().map(|h| (h, false)));
            match message {
                Some((text, is_error)) => {
                    AnyView::new(Text::new(text).style(message_style.clone()).color(move || {
                        let colors = theme.colors();
                        if is_error {
                            colors.error
                        } else {
                            colors.on_surface_variant
                        }
                    }))
                }
                None => AnyView::new(Empty),
            }
        }));
        column.build(cx)
    }
}

/// Nothing: an empty region.
struct Empty;

impl View for Empty {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}
