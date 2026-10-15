//! Tests for `docs/specs/widgets/date-picker.md`, rules WIDGETS-DATE-01..08.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Point, Rect};
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{
    ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_widgets::{Button, CalendarDatePicker, Column, Date, DatePickerField, Weekday};

const W: f32 = 500.0;
const H: f32 = 600.0;

fn d(y: i32, m: u32, day: u32) -> Date {
    Date::new(y, m, day).expect("a real date")
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
}

struct Fx {
    t: WidgetTester,
    selected: Signal<Option<Date>>,
    changes: Rc<RefCell<Vec<Date>>>,
}

type Make = Box<dyn FnOnce(Signal<Option<Date>>, Rc<RefCell<Vec<Date>>>) -> tantu_view::AnyView>;

fn fixture(initial: Option<Date>, make: Make) -> Fx {
    let out: Rc<RefCell<Option<Signal<Option<Date>>>>> = Rc::default();
    let changes: Rc<RefCell<Vec<Date>>> = Rc::default();
    let make = RefCell::new(Some(make));
    let t = {
        let (out, changes) = (out.clone(), changes.clone());
        WidgetTester::with_size(W, H, move || {
            let selected = signal(initial);
            *out.borrow_mut() = Some(selected);
            let make = make.borrow_mut().take().expect("built once");
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .spacing(8.0)
                .child(make(selected, changes.clone()))
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

/// A calendar keyed "cal" over 2025-01-01..2027-12-31, today 2026-10-16.
fn calendar(initial: Option<Date>, first_day: Weekday) -> Fx {
    fixture(
        initial,
        Box::new(move |selected, changes| {
            tantu_view::AnyView::new(Keyed::new(
                "cal",
                CalendarDatePicker::new(selected)
                    .first_date(d(2025, 1, 1))
                    .last_date(d(2027, 12, 31))
                    .today(d(2026, 10, 16))
                    .first_day_of_week(first_day)
                    .on_change(move |date| changes.borrow_mut().push(date)),
            ))
        }),
    )
}

/// A date field keyed "field" over 2026-01-01..2027-12-31, today 2026-10-16.
fn date_field(initial: Option<Date>) -> Fx {
    fixture(
        initial,
        Box::new(move |selected, changes| {
            tantu_view::AnyView::new(Keyed::new(
                "field",
                DatePickerField::new(selected)
                    .label("Due")
                    .first_date(d(2026, 1, 1))
                    .last_date(d(2027, 12, 31))
                    .today(d(2026, 10, 16))
                    .on_change(move |date| changes.borrow_mut().push(date)),
            ))
        }),
    )
}

impl Fx {
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
    }

    fn nodes(&mut self, role: accesskit::Role) -> Vec<(ElementId, accesskit::Node)> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .filter(|(_, n)| n.role() == role)
            .map(|(id, n)| (element(id), n))
            .collect()
    }

    fn grid(&mut self) -> Option<(ElementId, accesskit::Node)> {
        self.nodes(accesskit::Role::Grid).into_iter().next()
    }

    fn cell(&mut self, name: &str) -> (ElementId, accesskit::Node) {
        self.nodes(accesskit::Role::Cell)
            .into_iter()
            .find(|(_, n)| n.label() == Some(name))
            .unwrap_or_else(|| panic!("no cell {name}"))
    }

    fn field(&mut self) -> (ElementId, accesskit::Node) {
        self.nodes(accesskit::Role::ComboBox)
            .into_iter()
            .next()
            .expect("the date field")
    }

    /// The name of the node `holder`'s active descendant is.
    fn active_name(&mut self, holder: accesskit::Role) -> Option<String> {
        let tree = self.t.a11y_tree();
        let active = tree
            .nodes
            .iter()
            .find(|(_, n)| n.role() == holder)?
            .1
            .active_descendant()?;
        tree.nodes
            .iter()
            .find(|(id, _)| *id == active)
            .and_then(|(_, n)| n.label().map(str::to_owned))
    }

    fn rect(&mut self, id: ElementId) -> Rect {
        self.t.rect(id)
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

    fn click_cell(&mut self, name: &str) {
        let at = {
            let id = self.cell(name).0;
            self.rect(id).center()
        };
        self.press(at);
    }

    fn key(&mut self, key: NamedKey, modifiers: Modifiers) {
        self.t.press_key(LogicalKey::Named(key), modifiers);
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

    fn selected(&mut self) -> Option<Date> {
        let selected = self.selected;
        self.t.enter(|| selected.get_untracked())
    }

    fn focused(&mut self) -> Option<ElementId> {
        ElementId::from_raw(self.t.a11y_tree().focus.0)
    }
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

fn alt() -> Modifiers {
    Modifiers {
        alt: true,
        ..Modifiers::default()
    }
}

/// The platform's primary modifier for chords: Command on macOS, Control elsewhere.
fn primary() -> Modifiers {
    Modifiers {
        control: !cfg!(target_os = "macos"),
        super_key: cfg!(target_os = "macos"),
        ..Modifiers::default()
    }
}

#[test]
fn widgets_date_01_dates() {
    assert!(Date::new(2026, 2, 29).is_none());
    assert!(Date::new(2024, 2, 29).is_some());
    assert!(Date::new(1900, 2, 29).is_none(), "a century");
    assert!(Date::new(2000, 2, 29).is_some(), "divisible by 400");
    assert!(Date::new(2026, 13, 1).is_none());
    assert!(Date::new(2026, 4, 31).is_none());
    assert!(Date::new(0, 1, 1).is_none());
    assert!(Date::new(10000, 1, 1).is_none());
    assert_eq!(Date::days_in_month(2026, 2), 28);
    assert_eq!(Date::days_in_month(2024, 2), 29);
    assert_eq!(Date::days_in_month(2026, 12), 31);
    assert_eq!(Date::days_in_month(2026, 13), 0);
    let date = d(2026, 10, 16);
    assert_eq!((date.year(), date.month(), date.day()), (2026, 10, 16));
    assert_eq!(date.weekday(), Weekday::Friday);
    assert_eq!(d(2000, 1, 1).weekday(), Weekday::Saturday);
    assert_eq!(d(1, 1, 1).weekday(), Weekday::Monday);
    assert_eq!(date.add_days(16), Some(d(2026, 11, 1)));
    assert_eq!(date.add_days(-365), Some(d(2025, 10, 16)));
    assert_eq!(d(9999, 12, 31).add_days(1), None);
    assert_eq!(d(1, 1, 1).add_days(-1), None);
    assert_eq!(d(2024, 1, 31).add_months(1), Some(d(2024, 2, 29)));
    assert_eq!(d(2026, 11, 30).add_months(-13), Some(d(2025, 10, 30)));
    assert_eq!(d(9999, 12, 1).add_months(1), None);
    assert!(d(2026, 1, 31) < d(2026, 2, 1));
    assert!(d(2025, 12, 31) < d(2026, 1, 1));
    assert_eq!(Date::parse_iso("2026-10-16"), Some(date));
    assert_eq!(Date::parse_iso("2026-02-30"), None);
    assert_eq!(Date::parse_iso("2026-1-16"), None);
    assert_eq!(Date::parse_iso("26-10-16"), None);
    assert_eq!(Date::parse_iso("2026/10/16"), None);
    assert_eq!(Date::parse_iso("2026-10-16x"), None);
    assert_eq!(date.to_string(), "2026-10-16");
    assert_eq!(d(7, 3, 4).to_string(), "0007-03-04");
}

#[test]
fn widgets_date_02_layout() {
    let mut fx = calendar(Some(d(2026, 10, 16)), Weekday::Monday);
    let cal = fx.t.rect(fx.t.find(&Finder::key("cal")));
    assert_eq!((cal.width(), cal.height()), (280.0, 320.0));
    assert_eq!(fx.grid().expect("grid").1.label(), Some("October 2026"));
    let headers: Vec<String> = fx
        .nodes(accesskit::Role::ColumnHeader)
        .into_iter()
        .map(|(_, n)| n.label().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(headers[0], "Monday");
    assert_eq!(headers[6], "Sunday");
    assert_eq!(fx.t.find_all(&Finder::text("Mo")).len(), 1);
    // October 1, 2026 is a Thursday: column 3 of the first week.
    let (first, node) = fx.cell("Thursday, October 1, 2026");
    assert_eq!((node.row_index(), node.column_index()), (Some(0), Some(3)));
    let first = fx.rect(first);
    assert_eq!((first.left, first.top), (cal.left + 120.0, cal.top + 80.0));
    assert_eq!((first.width(), first.height()), (40.0, 40.0));
    let (_, last) = fx.cell("Saturday, October 31, 2026");
    assert_eq!((last.row_index(), last.column_index()), (Some(4), Some(5)));
    assert_eq!(
        fx.nodes(accesskit::Role::Cell).len(),
        31,
        "other months blank"
    );
    // The month buttons, disabled where the range ends.
    let next = fx.t.find(&Finder::semantics_label("Next month"));
    let at = fx.rect(next).center();
    fx.press(at);
    assert_eq!(fx.grid().expect("grid").1.label(), Some("November 2026"));
    let previous = fx.t.find(&Finder::semantics_label("Previous month"));
    let at = fx.rect(previous).center();
    fx.press(at);
    fx.press(at);
    assert_eq!(fx.grid().expect("grid").1.label(), Some("September 2026"));
    // Sunday first moves Thursday to column 4.
    let mut fx = calendar(Some(d(2026, 10, 16)), Weekday::Sunday);
    let (_, node) = fx.cell("Thursday, October 1, 2026");
    assert_eq!(node.column_index(), Some(4));
    // With nothing selected the calendar shows today's month; at the range's end the next
    // button is disabled.
    let mut fx = calendar(None, Weekday::Monday);
    assert_eq!(fx.grid().expect("grid").1.label(), Some("October 2026"));
    let mut fx = calendar(Some(d(2027, 12, 5)), Weekday::Monday);
    let disabled =
        fx.t.a11y_tree()
            .nodes
            .into_iter()
            .find(|(_, n)| n.label() == Some("Next month"))
            .map(|(_, n)| n.is_disabled());
    assert_eq!(disabled, Some(true));
}

#[test]
fn widgets_date_03_appearance() {
    let mut fx = calendar(Some(d(2026, 10, 20)), Weekday::Monday);
    let colors = fx.t.enter(|| fx.t.theme().get().colors);
    let commands = |fx: &mut Fx, id: ElementId| -> Vec<Command> {
        fx.t.frame()
            .scene
            .entries()
            .iter()
            .filter(|e| e.element == Some(id))
            .map(|e| e.command.clone())
            .collect()
    };
    let selected = fx.cell("Tuesday, October 20, 2026").0;
    let filled = commands(&mut fx, selected).iter().any(|c| {
        matches!(c, Command::Fill { color, shape, .. }
            if *color == colors.primary && shape.rect.width() == 36.0)
    });
    assert!(filled, "the selected day is a primary circle");
    let today = fx.cell("Friday, October 16, 2026").0;
    let outlined = commands(&mut fx, today)
        .iter()
        .any(|c| matches!(c, Command::Stroke { color, .. } if *color == colors.primary));
    assert!(outlined, "today is outlined");
    // Out-of-range days can't be chosen: a range starting mid-month.
    let mut fx = fixture(
        Some(d(2026, 10, 16)),
        Box::new(|selected, _| {
            tantu_view::AnyView::new(CalendarDatePicker::new(selected).first_date(d(2026, 10, 10)))
        }),
    );
    let (_, node) = fx.cell("Monday, October 5, 2026");
    assert!(node.is_disabled());
    fx.click_cell("Monday, October 5, 2026");
    assert_eq!(fx.selected(), Some(d(2026, 10, 16)));
}

#[test]
fn widgets_date_04_choosing() {
    let mut fx = calendar(Some(d(2026, 10, 16)), Weekday::Monday);
    fx.click_cell("Tuesday, October 20, 2026");
    assert_eq!(fx.selected(), Some(d(2026, 10, 20)));
    assert_eq!(*fx.changes.borrow(), [d(2026, 10, 20)]);
    fx.click_cell("Tuesday, October 20, 2026");
    assert_eq!(fx.changes.borrow().len(), 1, "the same date again");
    let (_, node) = fx.cell("Tuesday, October 20, 2026");
    assert_eq!(node.is_selected(), Some(true));
}

#[test]
fn widgets_date_05_keyboard() {
    let mut fx = calendar(Some(d(2026, 10, 16)), Weekday::Monday);
    fx.t.focus(&Finder::key("cal"));
    let grid = accesskit::Role::Grid;
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Friday, October 16, 2026")
    );
    fx.key(NamedKey::ArrowRight, Modifiers::default());
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Saturday, October 17, 2026")
    );
    fx.key(NamedKey::ArrowDown, Modifiers::default());
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Saturday, October 24, 2026")
    );
    fx.key(NamedKey::PageDown, Modifiers::default());
    assert_eq!(fx.grid().expect("grid").1.label(), Some("November 2026"));
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Tuesday, November 24, 2026")
    );
    fx.key(NamedKey::PageUp, shift());
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Monday, November 24, 2025")
    );
    fx.key(NamedKey::End, Modifiers::default());
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Sunday, November 30, 2025")
    );
    fx.key(NamedKey::Enter, Modifiers::default());
    assert_eq!(fx.selected(), Some(d(2025, 11, 30)));
    fx.key(NamedKey::Home, Modifiers::default());
    fx.type_text(" ");
    assert_eq!(fx.selected(), Some(d(2025, 11, 1)));
    // Moves are clamped to the range.
    fx.key(NamedKey::PageUp, shift());
    fx.key(NamedKey::PageUp, shift());
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Wednesday, January 1, 2025")
    );
    fx.key(NamedKey::ArrowLeft, Modifiers::default());
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Wednesday, January 1, 2025")
    );
    // Without a displayed selection the cursor starts on today.
    let mut fx = calendar(None, Weekday::Monday);
    fx.t.focus(&Finder::key("cal"));
    assert_eq!(
        fx.active_name(grid).as_deref(),
        Some("Friday, October 16, 2026")
    );
}

#[test]
fn widgets_date_06_semantics() {
    let mut fx = calendar(Some(d(2026, 10, 16)), Weekday::Monday);
    let (_, grid) = fx.grid().expect("grid");
    assert_eq!((grid.row_count(), grid.column_count()), (Some(6), Some(7)));
    assert_eq!(fx.nodes(accesskit::Role::ColumnHeader).len(), 7);
    let (_, node) = fx.cell("Friday, October 16, 2026");
    assert_eq!(node.is_selected(), Some(true));
    let (_, node) = fx.cell("Saturday, October 17, 2026");
    assert_ne!(node.is_selected(), Some(true));
    assert!(!node.is_disabled());
    fx.t.find(&Finder::semantics_label("Previous month"));
    fx.t.find(&Finder::semantics_label("Next month"));
}

#[test]
fn widgets_date_07_field() {
    let mut fx = date_field(None);
    let field = fx.t.rect(fx.t.find(&Finder::key("field")));
    assert_eq!(field.width(), 240.0);
    let (id, node) = fx.field();
    assert_eq!(node.label(), Some("Due"));
    assert_eq!(node.value().unwrap_or_default(), "");
    let at = fx.rect(id).center();
    fx.press(at);
    fx.type_text("20261020");
    assert_eq!(fx.field().1.value(), Some("2026-10-20"));
    assert_eq!(fx.selected(), Some(d(2026, 10, 20)));
    assert_eq!(*fx.changes.borrow(), [d(2026, 10, 20)]);
    // A complete text that isn't a date in the range is an error.
    fx.t.press_key(LogicalKey::Character("a".into()), primary());
    fx.type_text("20261340");
    let (_, node) = fx.field();
    assert_eq!(node.invalid(), Some(accesskit::Invalid::True));
    assert_eq!(node.description(), Some("Invalid date"));
    assert_eq!(fx.selected(), Some(d(2026, 10, 20)));
    fx.t.press_key(LogicalKey::Character("a".into()), primary());
    fx.type_text("20300101");
    assert_eq!(fx.field().1.description(), Some("Invalid date"));
    // Leaving restores the selected date; the field then follows the app.
    fx.key(NamedKey::Tab, Modifiers::default());
    assert_ne!(fx.focused(), Some(id));
    assert_eq!(fx.field().1.value(), Some("2026-10-20"));
    assert_eq!(fx.field().1.invalid(), None);
    let selected = fx.selected;
    fx.t.enter(|| selected.set(Some(d(2027, 3, 4))));
    fx.pump();
    assert_eq!(fx.field().1.value(), Some("2027-03-04"));
}

#[test]
fn widgets_date_08_popup() {
    let mut fx = date_field(Some(d(2026, 10, 20)));
    assert!(fx.grid().is_none());
    let (id, _) = fx.field();
    let at = fx.rect(id).center();
    fx.press(at);
    fx.key(NamedKey::ArrowDown, alt());
    let (grid, node) = fx.grid().expect("open");
    assert_eq!(node.label(), Some("October 2026"));
    let (grid, field) = (fx.rect(grid), fx.rect(id));
    assert!(grid.top >= field.bottom, "below the field");
    assert_eq!(fx.field().1.is_expanded(), Some(true));
    let combo = accesskit::Role::ComboBox;
    assert_eq!(
        fx.active_name(combo).as_deref(),
        Some("Tuesday, October 20, 2026")
    );
    fx.key(NamedKey::ArrowRight, Modifiers::default());
    assert_eq!(
        fx.active_name(combo).as_deref(),
        Some("Wednesday, October 21, 2026")
    );
    fx.key(NamedKey::Enter, Modifiers::default());
    assert!(fx.grid().is_none());
    assert_eq!(fx.selected(), Some(d(2026, 10, 21)));
    assert_eq!(fx.field().1.value(), Some("2026-10-21"));
    assert_eq!(fx.focused(), Some(id));
    // Escape closes without choosing.
    fx.key(NamedKey::ArrowDown, alt());
    fx.key(NamedKey::ArrowRight, Modifiers::default());
    fx.key(NamedKey::Escape, Modifiers::default());
    assert!(fx.grid().is_none());
    assert_eq!(fx.selected(), Some(d(2026, 10, 21)));
    // The button opens it and focuses the field; a click on a day chooses and closes.
    fx.key(NamedKey::Tab, Modifiers::default());
    fx.key(NamedKey::Tab, Modifiers::default());
    assert_ne!(fx.focused(), Some(id));
    let button = fx.t.find(&Finder::semantics_label("Choose date"));
    let at = fx.rect(button).center();
    fx.press(at);
    assert!(fx.grid().is_some());
    assert_eq!(fx.focused(), Some(id));
    fx.click_cell("Friday, October 30, 2026");
    assert!(fx.grid().is_none());
    assert_eq!(fx.selected(), Some(d(2026, 10, 30)));
    assert_eq!(*fx.changes.borrow(), [d(2026, 10, 21), d(2026, 10, 30)]);
    // A press outside closes it.
    fx.key(NamedKey::ArrowDown, alt());
    assert!(fx.grid().is_some());
    fx.press(Point::new(W - 5.0, H - 5.0));
    assert!(fx.grid().is_none());
}

#[test]
fn widgets_date_09_button_toggles_the_popup() {
    let mut fx = date_field(Some(d(2026, 10, 20)));
    let button = fx.t.find(&Finder::semantics_label("Choose date"));
    let at = fx.rect(button).center();
    fx.press(at);
    assert!(fx.grid().is_some(), "opened");
    fx.press(at);
    assert!(fx.grid().is_none(), "closed by the same button");
    fx.press(at);
    assert!(fx.grid().is_some(), "opened again");
}
