//! Tests for `docs/specs/examples/enterprise-dashboard.md`, rules DASH-01..06.

use enterprise_dashboard::{Region, Status, dashboard, orders};
use tantu::core::Point;
use tantu::view::{
    ElementId, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_test::{Finder, WidgetTester, accesskit};

const W: f32 = 1280.0;
const H: f32 = 800.0;

fn app() -> WidgetTester {
    let mut t = WidgetTester::with_size(W, H, dashboard);
    t.pump();
    t.pump();
    t
}

fn text_of(t: &WidgetTester, key: &str) -> String {
    let element = t.find(&Finder::key(key));
    t.text(element).expect("a text")
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
}

fn node(t: &mut WidgetTester, role: accesskit::Role, name: &str) -> Option<ElementId> {
    t.a11y_tree()
        .nodes
        .into_iter()
        .find(|(_, n)| n.role() == role && n.label() == Some(name))
        .map(|(id, _)| element(id))
}

fn click_at(t: &mut WidgetTester, at: Point) {
    for kind in [
        PointerKind::Down(PointerButton::Primary),
        PointerKind::Up(PointerButton::Primary),
    ] {
        t.pointer(PointerEvent {
            kind,
            position: at,
            modifiers: Modifiers::default(),
        });
        t.pump();
    }
    t.pump();
}

fn click(t: &mut WidgetTester, role: accesskit::Role, name: &str) {
    let id = node(t, role, name).unwrap_or_else(|| panic!("no {role:?} {name}"));
    let at = t.rect(id).center();
    click_at(t, at);
}

/// Chooses `menu` → `item` in the menu bar.
fn menu(t: &mut WidgetTester, menu: &str, item: &str) {
    click(t, accesskit::Role::MenuItem, menu);
    click(t, accesskit::Role::MenuItem, item);
}

fn key(t: &mut WidgetTester, key: NamedKey, modifiers: Modifiers) {
    t.press_key(LogicalKey::Named(key), modifiers);
    t.pump();
    t.pump();
}

fn type_text(t: &mut WidgetTester, text: &str) {
    for ch in text.chars() {
        t.press_key(
            LogicalKey::Character(ch.to_string().into()),
            Modifiers::default(),
        );
    }
    t.pump();
}

fn primary() -> Modifiers {
    Modifiers {
        control: !cfg!(target_os = "macos"),
        super_key: cfg!(target_os = "macos"),
        ..Modifiers::default()
    }
}

/// Selects the first order: focus the grid, then Down.
fn select_first(t: &mut WidgetTester) {
    t.focus(&Finder::key("orders-grid"));
    key(t, NamedKey::ArrowDown, Modifiers::default());
}

/// Replaces the text of the field named `label`.
fn set_field(t: &mut WidgetTester, label: &str, text: &str) {
    click(t, accesskit::Role::TextInput, label);
    t.press_key(LogicalKey::Character("a".into()), primary());
    key(t, NamedKey::Backspace, Modifiers::default());
    type_text(t, text);
}

fn field_value(t: &mut WidgetTester, label: &str) -> Option<String> {
    t.a11y_tree()
        .nodes
        .into_iter()
        .find(|(_, n)| n.role() == accesskit::Role::TextInput && n.label() == Some(label))
        .and_then(|(_, n)| n.value().map(str::to_owned))
}

#[test]
fn dash_01_window() {
    let mut t = app();
    for role in [
        accesskit::Role::MenuBar,
        accesskit::Role::Toolbar,
        accesskit::Role::Status,
    ] {
        assert!(
            t.a11y_tree().nodes.iter().any(|(_, n)| n.role() == role),
            "{role:?}"
        );
    }
    for tab in ["Orders", "Activity", "Details", "Revenue"] {
        assert!(node(&mut t, accesskit::Role::Tab, tab).is_some(), "{tab}");
    }
    let orders = t.rect(t.find(&Finder::key("orders")));
    let details = t.rect(t.find(&Finder::key("details")));
    let revenue = t.rect(t.find(&Finder::key("revenue")));
    assert!(orders.right < details.left, "orders on the left");
    assert!(details.bottom <= revenue.top, "details above revenue");
    assert!((orders.width() / W - 0.55).abs() < 0.05, "{orders:?}");
}

#[test]
fn dash_02_orders() {
    let all = orders(10_000);
    assert_eq!(all.len(), 10_000);
    assert_eq!(all, orders(10_000), "deterministic");
    assert!(all.iter().enumerate().all(|(i, o)| o.id as usize == i + 1));
    for o in &all {
        assert!((10.0..=5000.0).contains(&o.amount), "{o:?}");
        assert!(((o.amount * 100.0).round() - o.amount * 100.0).abs() < 1e-6);
        assert_eq!(o.date.year(), 2026);
    }
    for region in [Region::North, Region::South, Region::East, Region::West] {
        assert!(all.iter().any(|o| o.region == region));
    }
    for status in [Status::Open, Status::Shipped, Status::Paid] {
        assert!(all.iter().any(|o| o.status == status));
    }
    let mut t = app();
    for header in ["No", "Customer", "Region", "Amount", "Date", "Status"] {
        assert!(
            node(&mut t, accesskit::Role::ColumnHeader, header).is_some(),
            "{header}"
        );
    }
    // Amounts and dates in the localization's formats.
    let first = &all[0];
    assert!(
        !t.find_all(&Finder::text(english_amount(first.amount)))
            .is_empty()
    );
}

/// An amount as English writes it with two digits.
fn english_amount(value: f64) -> String {
    let cents = (value * 100.0).round() as i64;
    let (whole, cents) = (cents / 100, cents % 100);
    let digits = whole.to_string();
    let mut grouped = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    format!("{grouped}.{cents:02}")
}

#[test]
fn dash_03_details_form() {
    let mut t = app();
    assert!(!t.find_all(&Finder::text("Select an order")).is_empty());
    select_first(&mut t);
    let first = orders(1).remove(0);
    assert_eq!(
        field_value(&mut t, "Customer").as_deref(),
        Some(first.customer.as_str())
    );
    // A valid edit is saved: the grid and the activity log show it.
    set_field(&mut t, "Customer", "ACME Corp");
    click(&mut t, accesskit::Role::Button, "Save");
    assert!(!t.find_all(&Finder::text("ACME Corp")).is_empty());
    click(&mut t, accesskit::Role::Tab, "Activity");
    assert!(!t.find_all(&Finder::text("Order 1 updated")).is_empty());
    click(&mut t, accesskit::Role::Tab, "Orders");
    // An invalid one isn't.
    set_field(&mut t, "Customer", "");
    click(&mut t, accesskit::Role::Button, "Save");
    assert!(!t.find_all(&Finder::text("Enter a customer")).is_empty());
    set_field(&mut t, "Amount", "0");
    click(&mut t, accesskit::Role::Button, "Save");
    assert!(
        !t.find_all(&Finder::text("Enter an amount above 0"))
            .is_empty()
    );
}

#[test]
fn dash_04_revenue() {
    let mut t = app();
    let all = orders(10_000);
    let total: f64 = all.iter().map(|o| o.amount).sum();
    assert_eq!(text_of(&t, "kpi-orders"), "10,000");
    assert_eq!(text_of(&t, "kpi-revenue"), english_amount(total));
    assert_eq!(text_of(&t, "kpi-average"), english_amount(total / 10_000.0));
    let by_region = |region: Region| -> f64 {
        all.iter()
            .filter(|o| o.region == region)
            .map(|o| o.amount)
            .sum()
    };
    let regions = [Region::North, Region::South, Region::East, Region::West];
    let max = regions.iter().map(|r| by_region(*r)).fold(0.0, f64::max);
    let heights: Vec<f32> = ["bar-north", "bar-south", "bar-east", "bar-west"]
        .iter()
        .map(|k| t.rect(t.find(&Finder::key(*k))).height())
        .collect();
    let tallest = heights.iter().copied().fold(0.0, f32::max);
    for (region, height) in regions.iter().zip(&heights) {
        let expected = (by_region(*region) / max) as f32 * tallest;
        assert!(
            (height - expected).abs() <= 1.0,
            "{region:?}: {height} vs {expected}"
        );
    }
    // An edit moves the totals.
    select_first(&mut t);
    set_field(&mut t, "Amount", "10000");
    click(&mut t, accesskit::Role::Button, "Save");
    let first = orders(1).remove(0);
    assert_eq!(
        text_of(&t, "kpi-revenue"),
        english_amount(total - first.amount + 10_000.0)
    );
}

#[test]
fn dash_05_language_layout_and_resets() {
    let mut t = app();
    menu(&mut t, "View", "Deutsch");
    assert_eq!(text_of(&t, "kpi-orders"), "10.000");
    assert!(node(&mut t, accesskit::Role::Tab, "Aufträge").is_some());
    menu(&mut t, "Ansicht", "English");
    assert_eq!(text_of(&t, "kpi-orders"), "10,000");
    // Closing a panel, then resetting the layout.
    click(&mut t, accesskit::Role::Button, "Close Activity");
    assert!(node(&mut t, accesskit::Role::Tab, "Activity").is_none());
    menu(&mut t, "File", "Reset layout");
    assert!(node(&mut t, accesskit::Role::Tab, "Activity").is_some());
    // Resetting the edits.
    select_first(&mut t);
    set_field(&mut t, "Customer", "ACME Corp");
    click(&mut t, accesskit::Role::Button, "Save");
    click(&mut t, accesskit::Role::Button, "Reset edits");
    assert!(t.find_all(&Finder::text("ACME Corp")).is_empty());
}

#[test]
fn dash_06_status_and_golden() {
    let mut t = app();
    assert_eq!(text_of(&t, "status-orders"), "10,000 orders");
    select_first(&mut t);
    assert_eq!(text_of(&t, "status-selection"), "Order 1 selected");
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/dashboard.png"
    ));
}
