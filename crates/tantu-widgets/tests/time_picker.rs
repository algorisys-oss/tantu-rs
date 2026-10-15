//! Tests for `docs/specs/widgets/time-picker.md`, rules WIDGETS-TIME-01..04 and VIEW-SEM-05.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::layout::{BoxConstraints, CrossAxisAlignment, RenderConstrainedBox};
use tantu_view::reactive::{Signal, signal};
use tantu_view::{
    BuildCx, ElementId, FocusOptions, Handled, Keyed, LogicalKey, Modifiers, NamedKey, Phase,
    PointerButton, PointerEvent, PointerKind, Role, Semantics, SemanticsAction, SemanticsActions,
    View,
};
use tantu_widgets::{Button, Column, Time, TimePickerField};

fn t(hour: u32, minute: u32) -> Time {
    Time::new(hour, minute).expect("a real time")
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
}

struct Fx {
    t: WidgetTester,
    selected: Signal<Option<Time>>,
    changes: Rc<RefCell<Vec<Time>>>,
}

/// A field keyed "time" with a 15-minute step, then a button.
fn fixture(initial: Option<Time>) -> Fx {
    let out: Rc<RefCell<Option<Signal<Option<Time>>>>> = Rc::default();
    let changes: Rc<RefCell<Vec<Time>>> = Rc::default();
    let t = {
        let (out, changes) = (out.clone(), changes.clone());
        WidgetTester::with_size(400.0, 300.0, move || {
            let selected = signal(initial);
            *out.borrow_mut() = Some(selected);
            let changes = changes.clone();
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .spacing(8.0)
                .child(Keyed::new(
                    "time",
                    TimePickerField::new(selected)
                        .label("Start")
                        .minute_step(15)
                        .on_change(move |time| changes.borrow_mut().push(time)),
                ))
                .child(Button::new("Other"))
        })
    };
    let selected = out.borrow_mut().take().expect("built");
    let mut fx = Fx {
        t,
        selected,
        changes,
    };
    fx.pump();
    fx
}

impl Fx {
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
    }

    fn field(&mut self) -> (ElementId, accesskit::Node) {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .find(|(_, n)| n.role() == accesskit::Role::SpinButton)
            .map(|(id, n)| (element(id), n))
            .expect("a spin button")
    }

    fn text(&mut self) -> String {
        self.field().1.value().unwrap_or_default().to_owned()
    }

    fn click_field(&mut self) {
        let at = {
            let id = self.field().0;
            self.t.rect(id).center()
        };
        for kind in [
            PointerKind::Down(PointerButton::Primary),
            PointerKind::Up(PointerButton::Primary),
        ] {
            self.t.pointer(PointerEvent {
                kind,
                position: at,
                modifiers: Modifiers::default(),
            });
        }
        self.pump();
    }

    fn key(&mut self, key: NamedKey) {
        self.t
            .press_key(LogicalKey::Named(key), Modifiers::default());
        self.pump();
    }

    fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.t.press_key(
                LogicalKey::Character(ch.to_string().into()),
                Modifiers::default(),
            );
        }
        self.pump();
    }

    fn selected(&mut self) -> Option<Time> {
        let selected = self.selected;
        self.t.enter(|| selected.get_untracked())
    }
}

#[test]
fn widgets_time_01_times() {
    assert!(Time::new(24, 0).is_none());
    assert!(Time::new(0, 60).is_none());
    let time = t(9, 5);
    assert_eq!((time.hour(), time.minute()), (9, 5));
    assert_eq!(time.to_string(), "09:05");
    assert_eq!(t(23, 50).add_minutes(15), t(0, 5));
    assert_eq!(t(0, 5).add_minutes(-15), t(23, 50));
    assert_eq!(t(12, 0).add_minutes(-24 * 60 * 3), t(12, 0));
    assert!(t(9, 59) < t(10, 0));
    assert_eq!(Time::parse("09:05"), Some(time));
    assert_eq!(Time::parse("9:05"), None);
    assert_eq!(Time::parse("24:00"), None);
    assert_eq!(Time::parse("09-05"), None);
    assert_eq!(Time::parse("09:055"), None);
}

#[test]
fn widgets_time_02_field() {
    let mut fx = fixture(None);
    let field = fx.t.rect(fx.t.find(&Finder::key("time")));
    assert_eq!(field.width(), 120.0);
    assert_eq!(fx.text(), "");
    assert_eq!(fx.field().1.label(), Some("Start"));
    fx.click_field();
    fx.type_text("0930");
    assert_eq!(fx.text(), "09:30");
    assert_eq!(fx.selected(), Some(t(9, 30)));
    assert_eq!(*fx.changes.borrow(), [t(9, 30)]);
    // An impossible time is an error and changes nothing.
    fx.key(NamedKey::Backspace);
    fx.key(NamedKey::Backspace);
    fx.type_text("75");
    assert_eq!(fx.text(), "09:75");
    let (_, node) = fx.field();
    assert_eq!(node.invalid(), Some(accesskit::Invalid::True));
    assert_eq!(node.description(), Some("Invalid time"));
    assert_eq!(fx.selected(), Some(t(9, 30)));
    // Leaving restores; the field then follows the app.
    fx.key(NamedKey::Tab);
    assert_eq!(fx.text(), "09:30");
    assert_eq!(fx.field().1.invalid(), None);
    let selected = fx.selected;
    fx.t.enter(|| selected.set(Some(t(18, 0))));
    fx.pump();
    assert_eq!(fx.text(), "18:00");
}

#[test]
fn widgets_time_03_stepping() {
    let mut fx = fixture(Some(t(10, 7)));
    fx.click_field();
    fx.key(NamedKey::End);
    fx.key(NamedKey::ArrowUp);
    assert_eq!(fx.selected(), Some(t(10, 15)), "snapped up to the step");
    fx.key(NamedKey::ArrowUp);
    assert_eq!(fx.selected(), Some(t(10, 30)));
    assert_eq!(fx.text(), "10:30");
    fx.t.enter(|| fx.selected.set(Some(t(10, 7))));
    fx.pump();
    // The field has the focus, so the text stays; stepping reads the selection.
    fx.key(NamedKey::ArrowDown);
    assert_eq!(fx.selected(), Some(t(10, 0)), "snapped down to the step");
    // The caret before the colon steps hours, wrapping at midnight.
    fx.key(NamedKey::Home);
    fx.key(NamedKey::ArrowDown);
    assert_eq!(fx.selected(), Some(t(9, 0)));
    fx.t.enter(|| fx.selected.set(Some(t(23, 50))));
    fx.pump();
    fx.key(NamedKey::ArrowUp);
    assert_eq!(fx.selected(), Some(t(0, 50)));
    assert_eq!(fx.text(), "00:50");
    assert_eq!(fx.changes.borrow().len(), 5);
    // From nothing, Up selects midnight.
    let mut fx = fixture(None);
    fx.click_field();
    fx.key(NamedKey::ArrowUp);
    assert_eq!(fx.selected(), Some(t(0, 0)));
    assert_eq!(fx.text(), "00:00");
}

#[test]
fn widgets_time_04_semantics() {
    let mut fx = fixture(Some(t(9, 0)));
    let (_, node) = fx.field();
    assert_eq!(node.label(), Some("Start"));
    assert_eq!(node.value(), Some("09:00"));
    assert!(node.supports_action(accesskit::Action::Increment));
    assert!(node.supports_action(accesskit::Action::Decrement));
    assert!(fx.t.perform_action(&Finder::key("time"), SemanticsAction::Increment));
    fx.pump();
    assert_eq!(fx.selected(), Some(t(9, 15)));
    assert!(fx.t.perform_action(&Finder::key("time"), SemanticsAction::Decrement));
    fx.pump();
    assert_eq!(fx.selected(), Some(t(9, 0)));
}

/// A focusable spin button recording the keys it receives.
struct Spin {
    keys: Rc<RefCell<Vec<LogicalKey>>>,
}

impl View for Spin {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight_for(Some(50.0), Some(20.0))),
            [],
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::SpinButton,
                actions: SemanticsActions {
                    increment: true,
                    decrement: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        cx.focusable(id, FocusOptions::default());
        let keys = self.keys;
        cx.on_key(id, Phase::Bubble, move |k| {
            if k.event.pressed {
                keys.borrow_mut().push(k.event.key.clone());
            }
            Handled::Stop
        });
        id
    }
}

#[test]
fn view_sem_05_spin_button_increments_up() {
    let keys: Rc<RefCell<Vec<LogicalKey>>> = Rc::default();
    let mut t = {
        let keys = keys.clone();
        WidgetTester::with_size(200.0, 100.0, move || {
            Keyed::new("spin", Spin { keys: keys.clone() })
        })
    };
    t.pump();
    assert!(t.perform_action(&Finder::key("spin"), SemanticsAction::Increment));
    assert!(t.perform_action(&Finder::key("spin"), SemanticsAction::Decrement));
    assert_eq!(
        *keys.borrow(),
        [
            LogicalKey::Named(NamedKey::ArrowUp),
            LogicalKey::Named(NamedKey::ArrowDown)
        ]
    );
}
