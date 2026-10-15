# Forms, part 1: validation and field binding

- **Status:** Implemented (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (`Form`, `FormController`, `TextFormField`, `Validator`),
  `tantu-view` (`Semantics::invalid`, `description`), `tantu-a11y` (mapping),
  `examples/gallery`
- **Plan item:** Phase 4, "Forms: validation, field binding, masked input, date/time
  pickers, combo box with search", part 1 (validation and binding)
- **Related:** [TextField](text-field.md), [inherited values](../view/inherited.md),
  [focus](../view/focus.md), [semantics](../a11y/semantics.md),
  [gallery](../examples/gallery.md), [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Data-entry screens (customer records, settings, sign-up) need labelled fields, rules on
their values, error messages under the fields that break them, and a submit that refuses
to go ahead until everything is valid. Flutter's `Form`, `FormField` and `TextFormField`
are the model: a `Form` groups fields, a `FormController` validates or resets them all, and
each `TextFormField` binds a `TextEditingController` to a label, helper text and
validators.

## Scope

In scope: `Form` and `FormController`; `TextFormField` (label, helper, validators,
autovalidate modes, error display, initial value for reset); a few `Validator`
constructors; focusing the first invalid field; error semantics; gallery samples.

Out of scope (later parts of this item or later): a combo box with search (part 2), masked
input (part 3), date and time pickers (part 4); form fields other than text (checkboxes,
sliders and pickers can join through the same registration later); asynchronous
validation; saving to a model object (apps read their controllers).

## Public API

```rust
// tantu-view
pub struct Semantics { /* existing, */ pub invalid: bool, pub description: Option<String> }

// tantu-widgets
/// Checks a field's text: `None` when valid, else the message to show.
pub type Validator = Rc<dyn Fn(&str) -> Option<String>>;
/// Ready-made validators.
pub mod validators {
    pub fn required(message: impl Into<String>) -> Validator;
    pub fn min_length(length: usize, message: impl Into<String>) -> Validator;
    pub fn max_length(length: usize, message: impl Into<String>) -> Validator;
    /// Valid when `test` returns true for the text.
    pub fn custom(test: impl Fn(&str) -> bool + 'static, message: impl Into<String>) -> Validator;
}

/// When a field checks itself without `FormController::validate`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AutovalidateMode { #[default] Disabled, OnUserInteraction, Always }

/// Validates and resets the fields of a `Form`; `Copy`.
#[derive(Clone, Copy)]
pub struct FormController { /* private */ }
impl FormController {
    pub fn new() -> Self;
    /// Validates every field, shows their errors, focuses the first invalid one; true when
    /// all are valid.
    pub fn validate(&self) -> bool;
    /// Restores every field's initial text and clears the errors.
    pub fn reset(&self);
    /// Whether every field is valid now (tracked; doesn't show errors).
    pub fn is_valid(&self) -> bool;
}
impl Default for FormController;

/// Groups the form fields inside `child` under `controller`.
pub struct Form { /* private */ }
impl Form {
    pub fn new(controller: FormController, child: impl View) -> Self;
    /// The fields' default autovalidate mode (default `Disabled`).
    pub fn autovalidate(self, mode: AutovalidateMode) -> Self;
}

/// A labelled `TextField` with validation (Flutter's `TextFormField`).
pub struct TextFormField { /* private */ }
impl TextFormField {
    /// Its initial text, restored by `reset`, is the controller's text when built.
    pub fn new(controller: TextEditingController) -> Self;
    pub fn label(self, label: impl Into<String>) -> Self;
    pub fn helper(self, helper: impl Into<String>) -> Self;
    pub fn placeholder(self, text: impl Into<String>) -> Self;
    /// Adds a validator; they run in order and the first message shows.
    pub fn validator(self, validator: Validator) -> Self;
    /// Overrides the form's autovalidate mode for this field.
    pub fn autovalidate(self, mode: AutovalidateMode) -> Self;
    pub fn max_lines(self, max_lines: Option<u32>) -> Self;
    pub fn on_change(self, f: impl Fn(String) + 'static) -> Self;
}
```

## Behavior

- **WIDGETS-FORM-01:** A `TextFormField` is a column: its label (theme `label` style,
  `on_surface_variant`) 4 px above the text field, and under the field, 4 px below, either
  its error (theme `label` style, `error` color) or its helper text (`on_surface_variant`),
  or nothing. While it shows an error, the field's outline uses the `error` color.
- **WIDGETS-FORM-02:** `FormController::validate` runs every field of its form (fields
  inside the form's child, at any depth, registered when they build and forgotten when
  removed): each shows its first validator's message (or clears it), the first invalid
  field in tree order takes the focus, and the result is true when none failed. A form
  with no fields is valid.
- **WIDGETS-FORM-03:** Autovalidate: `Disabled` fields change their error only through
  `validate`/`reset` (an error shown by `validate` stays until then); `OnUserInteraction`
  fields validate on every change after the user's first edit; `Always` fields validate on
  every change and when they build. A field's own mode overrides the form's.
- **WIDGETS-FORM-04:** `reset` restores every field's initial text (its controller's text
  when it built), clears every error and makes `OnUserInteraction` fields wait for an edit
  again. `is_valid` runs the validators silently and is reactive to the fields' text.
- **WIDGETS-FORM-05:** A `TextFormField` outside a `Form` works alone: its autovalidate mode
  applies, and nothing else validates it.
- **WIDGETS-FORM-06:** Semantics: the text field is named by the label; a field showing an
  error has `invalid` set and the error as its `description`; otherwise the helper is the
  description.
- **A11Y-TREE-12:** `Semantics::invalid` maps to AccessKit's `Invalid::True` and
  `description` to its description.
- **GALLERY-36:** The Text field section gains a small form (key `form`): Name (required),
  Email (required, must contain "@" and a "."), a Submit button that validates and echoes
  "Form: …" (`form-echo`: "Form: fix 2 fields", "Form: submitted Ada <ada@example.com>"),
  and Reset. Tests: submitting empty shows both errors and focuses Name; fixing them
  submits; Reset clears. Golden `gallery-form.png` with the errors showing.
- **GALLERY-37:** A "form" kitchen-sink card after "dialogs" (twenty `SINK_CARDS` entries,
  sample `text_form_field`): one field validating on user interaction (at least 3
  characters), echoed by `sink-form-state` ("Valid" / "Invalid").

## Performance and allocation

Validators run on `validate`, on changes of autovalidating fields, and for `is_valid`
readers; nothing per frame.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **Flutter's model** (`Form`, a controller instead of a `GlobalKey<FormState>`,
   `TextFormField`), with validators as plain closures plus a few constructors.
2. **Fields register through an inherited value** the `Form` provides, so they can sit
   anywhere inside it.
3. **The first invalid field takes the focus** on `validate`, as on the web and in Flutter.
4. **Error semantics** through two new `Semantics` fields (`invalid`, `description`),
   mapped to AccessKit.
5. **Only text fields in part 1**; other field kinds join the same registration later.
