//! Tests for `docs/specs/widgets/combo-box.md`, rules WIDGETS-COMBO-01..07.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{WidgetTester, accesskit};
use tantu_view::core::{Point, Rect};
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{
    ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_widgets::{Button, Column, ComboBox};

const W: f32 = 400.0;
const H: f32 = 500.0;
const FRUITS: [&str; 8] = [
    "Apple", "Banana", "Cherry", "Mango", "Orange", "Pear", "Plum", "Kiwi",
];

struct Fx {
    t: WidgetTester,
    selected: Signal<Option<usize>>,
    changes: Rc<RefCell<Vec<usize>>>,
}

fn fixture() -> Fx {
    let out: Rc<RefCell<Option<Signal<Option<usize>>>>> = Rc::default();
    let changes: Rc<RefCell<Vec<usize>>> = Rc::default();
    let t = {
        let (out, changes) = (out.clone(), changes.clone());
        WidgetTester::with_size(W, H, move || {
            let selected = signal(None);
            *out.borrow_mut() = Some(selected);
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .spacing(8.0)
                .child(Keyed::new(
                    "combo",
                    ComboBox::new(selected, FRUITS)
                        .placeholder("Fruit")
                        .on_change(move |i| changes.borrow_mut().push(i)),
                ))
                .child(Keyed::new("other", Button::new("Other")))
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

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
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
            .find(|(_, n)| n.role() == accesskit::Role::ComboBox)
            .map(|(id, n)| (element(id), n))
            .expect("a combo box")
    }

    fn list(&mut self) -> Option<(ElementId, accesskit::Node)> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .find(|(_, n)| n.role() == accesskit::Role::ListBox)
            .map(|(id, n)| (element(id), n))
    }

    fn options(&mut self) -> Vec<String> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .filter(|(_, n)| n.role() == accesskit::Role::ListBoxOption)
            .map(|(_, n)| n.label().unwrap_or_default().to_owned())
            .collect()
    }

    fn option(&mut self, name: &str) -> (ElementId, accesskit::Node) {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .find(|(_, n)| n.role() == accesskit::Role::ListBoxOption && n.label() == Some(name))
            .map(|(id, n)| (element(id), n))
            .unwrap_or_else(|| panic!("no option {name}"))
    }

    fn highlighted(&mut self) -> Option<String> {
        let tree = self.t.a11y_tree();
        let field = tree
            .nodes
            .iter()
            .find(|(_, n)| n.role() == accesskit::Role::ComboBox)?
            .1
            .clone();
        let active = field.active_descendant()?;
        tree.nodes
            .iter()
            .find(|(id, _)| *id == active)
            .and_then(|(_, n)| n.label().map(str::to_owned))
    }

    fn text(&mut self) -> String {
        self.field().1.value().unwrap_or_default().to_owned()
    }

    fn press(&mut self, at: Point) {
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

    fn open_by_press(&mut self) {
        let at = {
            let id = self.field().0;
            self.t.rect(id).center()
        };
        self.press(at);
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

    fn focused(&mut self) -> Option<ElementId> {
        ElementId::from_raw(self.t.a11y_tree().focus.0)
    }

    fn rect(&mut self, id: ElementId) -> Rect {
        self.t.rect(id)
    }
}

#[test]
fn widgets_combo_01_field_shows_the_selection() {
    let mut fx = fixture();
    assert_eq!(fx.text(), "");
    let field = fx.field().0;
    assert_eq!(fx.rect(field).width(), 240.0);
    let selected = fx.selected;
    fx.t.enter(|| selected.set(Some(1)));
    fx.pump();
    assert_eq!(fx.text(), "Banana");
    fx.t.enter(|| selected.set(Some(99)));
    fx.pump();
    assert_eq!(fx.text(), "");
}

#[test]
fn widgets_combo_02_popup() {
    let mut fx = fixture();
    assert!(fx.list().is_none());
    fx.open_by_press();
    let (list, _) = fx.list().expect("open");
    let field = fx.field().0;
    let (list, field) = (fx.rect(list), fx.rect(field));
    assert_eq!((list.left, list.top), (field.left, field.bottom));
    assert_eq!(
        (list.width(), list.height()),
        (240.0, 240.0),
        "six 40 px rows"
    );
    // Down opens it too.
    fx.key(NamedKey::Escape);
    assert!(fx.list().is_none());
    fx.key(NamedKey::ArrowDown);
    assert!(fx.list().is_some());
    // No match: one row that can't be chosen.
    fx.type_text("zz");
    assert_eq!(fx.options(), ["No matches"]);
    fx.key(NamedKey::Enter);
    assert_eq!(fx.t.enter(|| fx.selected.get_untracked()), None);
}

#[test]
fn widgets_combo_03_filtering() {
    let mut fx = fixture();
    fx.open_by_press();
    assert_eq!(fx.options().len().min(6), 6, "everything, six visible");
    fx.type_text("AN");
    assert_eq!(fx.options(), ["Banana", "Mango", "Orange"]);
}

#[test]
fn widgets_combo_04_highlight() {
    let mut fx = fixture();
    fx.open_by_press();
    assert_eq!(fx.highlighted().as_deref(), Some("Apple"));
    fx.key(NamedKey::ArrowDown);
    assert_eq!(fx.highlighted().as_deref(), Some("Banana"));
    fx.key(NamedKey::Home);
    fx.key(NamedKey::ArrowUp);
    assert_eq!(
        fx.highlighted().as_deref(),
        Some("Kiwi"),
        "wrapping to the last"
    );
    fx.key(NamedKey::Home);
    assert_eq!(fx.highlighted().as_deref(), Some("Apple"));
    let cherry = {
        let id = fx.option("Cherry").0;
        fx.rect(id).center()
    };
    fx.t.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: cherry,
        modifiers: Modifiers::default(),
    });
    fx.pump();
    assert_eq!(fx.highlighted().as_deref(), Some("Cherry"));
    // Opening highlights the selected option.
    fx.key(NamedKey::Enter);
    fx.open_by_press();
    assert_eq!(fx.highlighted().as_deref(), Some("Cherry"));
}

#[test]
fn widgets_combo_05_choosing() {
    let mut fx = fixture();
    fx.open_by_press();
    let field = fx.field().0;
    fx.type_text("an");
    fx.key(NamedKey::ArrowDown);
    fx.key(NamedKey::Enter);
    assert_eq!(fx.t.enter(|| fx.selected.get_untracked()), Some(3));
    assert_eq!(fx.text(), "Mango");
    assert!(fx.list().is_none());
    assert_eq!(*fx.changes.borrow(), [3]);
    assert_eq!(fx.focused(), Some(field));
    // A click picks too; choosing the same again reports nothing.
    fx.open_by_press();
    let pear = {
        let id = fx.option("Pear").0;
        fx.rect(id).center()
    };
    fx.press(pear);
    assert_eq!(fx.t.enter(|| fx.selected.get_untracked()), Some(5));
    assert_eq!(fx.text(), "Pear");
    fx.open_by_press();
    let pear = {
        let id = fx.option("Pear").0;
        fx.rect(id).center()
    };
    fx.press(pear);
    assert_eq!(*fx.changes.borrow(), [3, 5]);
}

#[test]
fn widgets_combo_06_closing_restores_the_text() {
    let mut fx = fixture();
    let selected = fx.selected;
    fx.t.enter(|| selected.set(Some(0)));
    fx.pump();
    fx.open_by_press();
    fx.type_text("zz");
    fx.key(NamedKey::Escape);
    assert!(fx.list().is_none());
    assert_eq!(fx.text(), "Apple");
    // A press outside, too.
    fx.open_by_press();
    fx.type_text("q");
    fx.press(Point::new(W - 5.0, H - 5.0));
    assert!(fx.list().is_none());
    assert_eq!(fx.text(), "Apple");
}

#[test]
fn widgets_combo_07_semantics() {
    let mut fx = fixture();
    let (_, field) = fx.field();
    assert_eq!(field.label(), Some("Fruit"));
    assert_eq!(field.is_expanded(), Some(false));
    let selected = fx.selected;
    fx.t.enter(|| selected.set(Some(2)));
    fx.pump();
    fx.open_by_press();
    assert_eq!(fx.field().1.is_expanded(), Some(true));
    assert_eq!(fx.option("Cherry").1.is_selected(), Some(true));
    assert_eq!(fx.option("Apple").1.is_selected(), Some(false));
}
