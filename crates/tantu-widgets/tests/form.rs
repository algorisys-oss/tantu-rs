//! Tests for `docs/specs/widgets/form.md`, rules WIDGETS-FORM-01..06.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::Rect;
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::scene::Command;
use tantu_view::{ElementId, LogicalKey, Modifiers, NamedKey};
use tantu_widgets::{
    AutovalidateMode, Button, Column, Form, FormController, SizedBox, TextEditingController,
    TextFormField, validators,
};

const W: f32 = 400.0;
const H: f32 = 400.0;

struct Fx {
    t: WidgetTester,
    form: FormController,
    name: TextEditingController,
    email: TextEditingController,
}

/// Name (required, with helper) and Email (required, must contain "@", initially "x") in a
/// form, field mode `mode`.
fn fixture(mode: AutovalidateMode) -> Fx {
    type Out = Rc<RefCell<Option<(FormController, TextEditingController, TextEditingController)>>>;
    let out: Out = Rc::default();
    let t = {
        let out = out.clone();
        WidgetTester::with_size(W, H, move || {
            let form = FormController::new();
            let (name, email) = (
                TextEditingController::new(""),
                TextEditingController::new("x"),
            );
            *out.borrow_mut() = Some((form, name, email));
            Form::new(
                form,
                SizedBox::width(300.0).child(
                    Column::new()
                        .cross_axis_alignment(CrossAxisAlignment::Stretch)
                        .spacing(8.0)
                        .child(
                            TextFormField::new(name)
                                .label("Name")
                                .helper("Your full name")
                                .validator(validators::required("Enter a name")),
                        )
                        .child(
                            TextFormField::new(email)
                                .label("Email")
                                .validator(validators::required("Enter an email"))
                                .validator(validators::custom(|t| t.contains('@'), "Not an email")),
                        )
                        .child(Button::new("Other")),
                ),
            )
            .autovalidate(mode)
        })
    };
    let (form, name, email) = out.borrow_mut().take().expect("built");
    let mut fx = Fx {
        t,
        form,
        name,
        email,
    };
    fx.pump();
    fx
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
}

impl Fx {
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
    }

    fn validate(&mut self) -> bool {
        let form = self.form;
        let valid = self.t.enter(|| form.validate());
        self.pump();
        valid
    }

    fn input(&mut self, label: &str) -> (ElementId, accesskit::Node) {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .find(|(_, n)| n.role() == accesskit::Role::TextInput && n.label() == Some(label))
            .map(|(id, n)| (element(id), n))
            .unwrap_or_else(|| panic!("no field {label}"))
    }

    fn shows(&self, text: &str) -> bool {
        !self.t.find_all(&Finder::text(text)).is_empty()
    }

    fn text_rect(&self, text: &str) -> Rect {
        self.t.rect(self.t.find(&Finder::text(text)))
    }

    fn focused(&mut self) -> Option<ElementId> {
        ElementId::from_raw(self.t.a11y_tree().focus.0)
    }

    fn set(&mut self, c: TextEditingController, text: &str) {
        self.t.enter(|| c.set_text(text));
        self.pump();
    }

    /// Types a character into the field labelled `label`, then deletes it.
    fn touch(&mut self, label: &str) {
        let (id, _) = self.input(label);
        let at = self.t.rect(id).center();
        for kind in [
            tantu_view::PointerKind::Down(tantu_view::PointerButton::Primary),
            tantu_view::PointerKind::Up(tantu_view::PointerButton::Primary),
        ] {
            self.t.pointer(tantu_view::PointerEvent {
                kind,
                position: at,
                modifiers: Modifiers::default(),
            });
        }
        self.pump();
        self.t
            .press_key(LogicalKey::Character("a".into()), Modifiers::default());
        self.t
            .press_key(LogicalKey::Named(NamedKey::Backspace), Modifiers::default());
        self.pump();
    }
}

#[test]
fn widgets_form_01_layout() {
    let mut fx = fixture(AutovalidateMode::Disabled);
    let (id, _) = fx.input("Name");
    let field = fx.t.rect(id);
    let label = fx.text_rect("Name");
    assert_eq!(label.bottom + 4.0, field.top);
    let helper = fx.text_rect("Your full name");
    assert_eq!(helper.top, field.bottom + 4.0);
    // An error replaces the helper; the outline turns to the error color.
    assert!(!fx.validate());
    assert!(!fx.shows("Your full name"));
    let error = fx.text_rect("Enter a name");
    assert_eq!(error.top, field.bottom + 4.0);
    let colors = fx.t.enter(|| fx.t.theme().get().colors);
    let strokes: Vec<_> =
        fx.t.frame()
            .scene
            .entries()
            .iter()
            .filter(|e| e.element == Some(id))
            .filter_map(|e| match &e.command {
                Command::Stroke { color, .. } => Some(*color),
                _ => None,
            })
            .collect();
    assert!(strokes.contains(&colors.error), "{strokes:?}");
}

#[test]
fn widgets_form_02_validate() {
    let mut fx = fixture(AutovalidateMode::Disabled);
    // Name is empty and Email lacks "@": both fail, Name takes the focus.
    assert!(!fx.validate());
    assert!(fx.shows("Enter a name"));
    assert!(fx.shows("Not an email"));
    let name = fx.input("Name").0;
    assert_eq!(fx.focused(), Some(name));
    // With Name fixed, Email fails and takes the focus.
    let c = fx.name;
    fx.set(c, "Ada");
    assert!(!fx.validate());
    assert!(!fx.shows("Enter a name"));
    let email = fx.input("Email").0;
    assert_eq!(fx.focused(), Some(email));
    let c = fx.email;
    fx.set(c, "ada@example.com");
    assert!(fx.validate());
    assert!(!fx.shows("Not an email"));
    // A form without fields is valid.
    let out: Rc<RefCell<Option<FormController>>> = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(W, H, move || {
            let form = FormController::new();
            *out.borrow_mut() = Some(form);
            Form::new(form, SizedBox::new(10.0, 10.0))
        })
    };
    t.pump();
    let form = out.borrow_mut().take().expect("built");
    assert!(t.enter(|| form.validate()));
}

#[test]
fn widgets_form_03_autovalidate() {
    // Disabled: an error stays until the next validate.
    let mut fx = fixture(AutovalidateMode::Disabled);
    fx.validate();
    let c = fx.name;
    fx.set(c, "Ada");
    assert!(fx.shows("Enter a name"));
    // OnUserInteraction: nothing until the user edits; then every change.
    let mut fx = fixture(AutovalidateMode::OnUserInteraction);
    assert!(!fx.shows("Enter a name"));
    fx.touch("Name");
    assert!(fx.shows("Enter a name"));
    let c = fx.name;
    fx.set(c, "Ada");
    assert!(!fx.shows("Enter a name"));
    // Always: from the start.
    let fx = fixture(AutovalidateMode::Always);
    assert!(fx.shows("Enter a name"));
    assert!(fx.shows("Not an email"));
}

#[test]
fn widgets_form_04_reset_and_is_valid() {
    let mut fx = fixture(AutovalidateMode::OnUserInteraction);
    let form = fx.form;
    assert!(!fx.t.enter(|| form.is_valid()));
    let (name, email) = (fx.name, fx.email);
    fx.set(name, "Ada");
    fx.set(email, "ada@example.com");
    assert!(fx.t.enter(|| form.is_valid()));
    // is_valid shows nothing.
    fx.set(email, "nope");
    assert!(!fx.t.enter(|| form.is_valid()));
    assert!(!fx.shows("Not an email"));
    fx.validate();
    assert!(fx.shows("Not an email"));
    // Reset restores the initial texts and clears the errors.
    fx.t.enter(|| form.reset());
    fx.pump();
    assert_eq!(fx.t.enter(|| name.text()), "");
    assert_eq!(fx.t.enter(|| email.text()), "x");
    assert!(!fx.shows("Not an email"));
    // ...and OnUserInteraction fields wait for an edit again.
    fx.set(email, "still bad");
    assert!(!fx.shows("Not an email"));
}

#[test]
fn widgets_form_05_field_alone() {
    let out: Rc<RefCell<Option<TextEditingController>>> = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(W, H, move || {
            let c = TextEditingController::new("");
            *out.borrow_mut() = Some(c);
            SizedBox::width(300.0).child(
                TextFormField::new(c)
                    .label("Code")
                    .validator(validators::min_length(3, "Too short"))
                    .autovalidate(AutovalidateMode::Always),
            )
        })
    };
    t.pump();
    let c = out.borrow_mut().take().expect("built");
    assert!(!t.find_all(&Finder::text("Too short")).is_empty());
    t.enter(|| c.set_text("abcd"));
    t.pump();
    t.pump();
    assert!(t.find_all(&Finder::text("Too short")).is_empty());
    // The other validators.
    let max = validators::max_length(2, "Too long");
    assert_eq!(max("abc").as_deref(), Some("Too long"));
    assert_eq!(max("ab"), None);
    let required = validators::required("Needed");
    assert_eq!(required("  ").as_deref(), Some("Needed"));
    assert_eq!(required("x"), None);
}

#[test]
fn widgets_form_06_semantics() {
    let mut fx = fixture(AutovalidateMode::Disabled);
    let (_, name) = fx.input("Name");
    assert_eq!(name.invalid(), None);
    assert_eq!(name.description(), Some("Your full name"));
    fx.validate();
    let (_, name) = fx.input("Name");
    assert_eq!(name.invalid(), Some(accesskit::Invalid::True));
    assert_eq!(name.description(), Some("Enter a name"));
}
