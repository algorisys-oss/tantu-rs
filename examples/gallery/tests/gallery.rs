//! Tests for `docs/specs/examples/gallery.md`, rules GALLERY-01..11, and
//! `docs/specs/examples/kitchen-sink.md`, GALLERY-12..14.

use gallery::{SECTIONS, SINK_CARDS, gallery};
use tantu::core::{Point, Rect, Vec2};
use tantu::scene::Command;
use tantu::theme::Theme;
use tantu::view::{PointerButton, PointerEvent, PointerKind};
use tantu_test::{Finder, WidgetTester};

fn rect(tester: &WidgetTester, key: &str) -> Rect {
    tester.rect(tester.find(&Finder::key(key)))
}

fn text_of(tester: &WidgetTester, key: &str) -> String {
    let element = tester.find(&Finder::key(key));
    tester.text(element).expect("a text")
}

fn nav(section: &str) -> String {
    format!("nav-{}", section.to_lowercase().replace(' ', "-"))
}

#[test]
fn gallery_17_grid_editors() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    let mut t = WidgetTester::with_size(960.0, 640.0, gallery);
    t.tap(&Finder::key(nav("Data grid")));
    t.pump();
    assert_eq!(text_of(&t, "grid-edit-error"), "Edit error: none");
    t.tap(&Finder::text("Person 2"));
    t.press_key(LogicalKey::Named(NamedKey::F(2)), Modifiers::default());
    t.pump();
    assert!(
        t.a11y_tree()
            .nodes
            .iter()
            .any(|(_, n)| n.role() == tantu_test::accesskit::Role::TextInput)
    );
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-grid-editor.png"
    ));
    for c in "Edited".chars() {
        t.press_key(
            LogicalKey::Character(c.to_string().into()),
            Modifiers::default(),
        );
    }
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    t.pump();
    t.find(&Finder::text("Edited"));
    t.tap(&Finder::text("Person 3"));
    t.tap(&Finder::text("Person 3"));
    t.pump();
    t.press_key(LogicalKey::Named(NamedKey::Escape), Modifiers::default());
    t.pump();
    t.find(&Finder::text("Person 3"));
    t.press_key(LogicalKey::Named(NamedKey::End), Modifiers::default());
    t.pump();
    t.press_key(LogicalKey::Named(NamedKey::F(2)), Modifiers::default());
    t.pump();
    for c in "101".chars() {
        t.press_key(
            LogicalKey::Character(c.to_string().into()),
            Modifiers::default(),
        );
    }
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    t.pump();
    assert_eq!(
        text_of(&t, "grid-edit-error"),
        "Edit error: Score must be between 0 and 100"
    );
    assert!(
        t.a11y_tree()
            .nodes
            .iter()
            .any(|(_, n)| n.role() == tantu_test::accesskit::Role::TextInput)
    );
}

#[test]
fn gallery_16_grid_selection() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Data grid")));
    tester.pump();
    assert_eq!(text_of(&tester, "grid-selection"), "Selection: none");
    tester.tap(&Finder::text("Person 2"));
    assert_eq!(text_of(&tester, "grid-selection"), "Selection: rows 2..2");
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-grid-row-selection.png"
    ));
    tester.tap(&Finder::key("grid-mode-range"));
    tester.pump();
    tester.tap(&Finder::text("Person 2"));
    tester.press_key(
        tantu::view::LogicalKey::Named(tantu::view::NamedKey::ArrowDown),
        tantu::view::Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    tester.press_key(
        tantu::view::LogicalKey::Named(tantu::view::NamedKey::ArrowRight),
        tantu::view::Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(
        text_of(&tester, "grid-selection"),
        "Selection: (2, 1)..(3, 2)"
    );
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-grid-range-selection.png"
    ));
}

#[test]
fn gallery_01_structure_and_sections() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    assert_eq!(rect(&tester, "header").width(), 960.0);
    assert_eq!(rect(&tester, "sidebar").width(), 200.0);
    let content = rect(&tester, "content");
    assert_eq!((content.left, content.width()), (200.0, 760.0));
    assert_eq!(text_of(&tester, "heading"), "Buttons");
    assert_eq!(text_of(&tester, "current"), "Buttons");
    tester.find(&Finder::key("press"));
    for section in SECTIONS {
        tester.tap(&Finder::key(nav(section)));
        assert_eq!(text_of(&tester, "heading"), section);
        assert_eq!(text_of(&tester, "current"), section);
        if section != "Buttons" {
            assert!(
                tester.find_all(&Finder::key("press")).is_empty(),
                "{section}"
            );
        }
    }
}

#[test]
fn gallery_02_samples_react() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    assert_eq!(text_of(&tester, "press-count"), "Pressed 0 times");
    tester.tap(&Finder::key("press"));
    tester.tap(&Finder::key("press"));
    assert_eq!(text_of(&tester, "press-count"), "Pressed 2 times");
    tester.tap(&Finder::key(nav("Selection")));
    assert_eq!(text_of(&tester, "checkbox-state"), "Checkbox: off");
    tester.tap(&Finder::key("checkbox"));
    assert_eq!(text_of(&tester, "checkbox-state"), "Checkbox: on");
    tester.tap(&Finder::key("switch"));
    assert_eq!(text_of(&tester, "switch-state"), "Switch: on");
    assert_eq!(text_of(&tester, "radio-state"), "Radio: a");
    tester.tap(&Finder::key("radio-c"));
    assert_eq!(text_of(&tester, "radio-state"), "Radio: c");
    assert_eq!(text_of(&tester, "slider-state"), "Slider: 50");
    tester.drag(&Finder::key("slider"), Vec2::new(1000.0, 0.0));
    assert_eq!(text_of(&tester, "slider-state"), "Slider: 100");
}

#[test]
fn gallery_03_text_field_and_list() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Text field")));
    assert_eq!(text_of(&tester, "echo"), "You typed: ");
    tester.enter_text(&Finder::key("field"), "hi");
    assert_eq!(text_of(&tester, "echo"), "You typed: hi");
    tester.tap(&Finder::key(nav("Lists")));
    tester.pump();
    tester.find(&Finder::text("Row 0"));
    tester.scroll(&Finder::key("list"), Vec2::new(0.0, 3200.0));
    // Wheel steps animate (WIDGETS-SCROLL-07): wait the step out.
    tester.advance(std::time::Duration::from_millis(150));
    tester.pump();
    assert!(tester.find_all(&Finder::text("Row 0")).is_empty());
    tester.find(&Finder::text("Row 100"));
}

#[test]
fn gallery_04_goldens() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-buttons.png"
    ));
    tester.tap(&Finder::key(nav("Selection")));
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-selection.png"
    ));
}

#[test]
fn gallery_05_dark_mode() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    let fill_of = |tester: &WidgetTester, key: &str| {
        let element = tester.find(&Finder::key(key));
        tester
            .frame()
            .entries_for(element)
            .find_map(|e| match e.command {
                Command::Fill { color, .. } => Some(color),
                _ => None,
            })
            .expect("a fill")
    };
    assert_eq!(fill_of(&tester, "press"), Theme::light().colors.primary);
    tester.tap(&Finder::key("dark"));
    assert_eq!(fill_of(&tester, "press"), Theme::dark().colors.primary);
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-dark.png"
    ));
    tester.tap(&Finder::key("dark"));
    assert_eq!(fill_of(&tester, "press"), Theme::light().colors.primary);
}

#[test]
fn gallery_06_animation_section() {
    use std::time::Duration;
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Animation")));
    assert_eq!(rect(&tester, "box").width(), 100.0);
    let dot_before = rect(&tester, "dot");
    tester.tap(&Finder::key("grow"));
    tester.advance(Duration::from_millis(150));
    assert_eq!(rect(&tester, "box").width(), 150.0);
    assert_ne!(rect(&tester, "dot"), dot_before, "the dot keeps moving");
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-animation.png"
    ));
    tester.advance(Duration::from_millis(150));
    assert_eq!(rect(&tester, "box").width(), 200.0);
    tester.tap(&Finder::key("grow"));
    tester.advance(Duration::from_millis(300));
    assert_eq!(rect(&tester, "box").width(), 100.0);
}

#[test]
fn gallery_07_switcher() {
    use std::time::Duration;
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Animation")));
    tester.find(&Finder::key("switcher"));
    assert!(!tester.find_all(&Finder::text("One")).is_empty());
    assert!(tester.find_all(&Finder::text("Two")).is_empty());
    tester.tap(&Finder::key("next"));
    tester.advance(Duration::from_millis(150));
    assert!(!tester.find_all(&Finder::text("One")).is_empty());
    assert!(!tester.find_all(&Finder::text("Two")).is_empty());
    let half = tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter(|e| matches!(e.command, tantu::scene::Command::PushLayer(l) if (l.opacity - 0.5).abs() < 1e-3))
        .count();
    assert_eq!(half, 2);
    tester.advance(Duration::from_millis(150));
    assert!(tester.find_all(&Finder::text("One")).is_empty());
    assert!(!tester.find_all(&Finder::text("Two")).is_empty());
}

#[test]
fn gallery_08_contacts_list() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Lists")));
    tester.pump();
    tester.find(&Finder::key("contacts"));
    assert_eq!(text_of(&tester, "selected"), "Selected: none");
    tester.find(&Finder::text("Group 0"));
    tester.tap(&Finder::text("Contact 1"));
    assert_eq!(text_of(&tester, "selected"), "Selected: 1");
    tester.press_key(LogicalKey::Named(NamedKey::ArrowDown), Modifiers::default());
    assert_eq!(text_of(&tester, "selected"), "Selected: 2");
    tester.press_key(LogicalKey::Named(NamedKey::End), Modifiers::default());
    assert_eq!(text_of(&tester, "selected"), "Selected: 199");
    tester.find(&Finder::text("Contact 199"));
    assert!(
        tester.find_all(&Finder::text("Contact 1")).is_empty(),
        "scrolled away"
    );
}

#[test]
fn gallery_09_every_sample_has_a_node() {
    use tantu::view::SemanticsAction;
    use tantu_test::accesskit::Role;
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    let role_of = |tester: &mut WidgetTester, key: &str| {
        let id = tester.find(&Finder::key(key));
        let update = tester.a11y_tree();
        update
            .nodes
            .iter()
            .find(|(n, _)| *n == tantu_test::accesskit::NodeId(id.to_raw()))
            .map(|(_, n)| (n.role(), n.label().map(str::to_owned)))
            .unwrap_or_else(|| panic!("no node for {key}"))
    };
    assert_eq!(
        role_of(&mut tester, "press"),
        (Role::Button, Some("Press me".to_owned()))
    );
    assert_eq!(role_of(&mut tester, "icon-button").0, Role::Button);
    tester.tap(&Finder::key(nav("Selection")));
    assert_eq!(role_of(&mut tester, "checkbox").0, Role::CheckBox);
    assert_eq!(role_of(&mut tester, "switch").0, Role::Switch);
    assert_eq!(role_of(&mut tester, "radio-a").0, Role::RadioButton);
    assert_eq!(role_of(&mut tester, "slider").0, Role::Slider);
    tester.tap(&Finder::key(nav("Text field")));
    assert_eq!(role_of(&mut tester, "field").0, Role::TextInput);
    assert_eq!(role_of(&mut tester, "notes").0, Role::MultilineTextInput);
    tester.tap(&Finder::key(nav("Lists")));
    assert_eq!(role_of(&mut tester, "list").0, Role::List);
    let update = tester.a11y_tree();
    assert!(update.nodes.iter().any(|(_, n)| n.role() == Role::ListItem));
    tester.tap(&Finder::key(nav("Media")));
    assert_eq!(role_of(&mut tester, "image").0, Role::Image);
    tester.tap(&Finder::key(nav("Buttons")));
    assert!(tester.perform_action(&Finder::key("press"), SemanticsAction::Click));
    assert_eq!(text_of(&tester, "press-count"), "Pressed 1 times");
}

#[test]
fn gallery_10_data_grid() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Data grid")));
    // The cells appear one frame after the grid's first paint.
    tester.pump();
    tester.find(&Finder::key("grid"));
    assert_eq!(text_of(&tester, "first-row"), "First row: 0");
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-grid.png"
    ));
    tester.scroll(&Finder::key("grid"), Vec2::new(0.0, 320.0));
    // The wheel step animates (WIDGETS-SCROLL-07); let it finish.
    tester.advance(std::time::Duration::from_millis(200));
    assert_eq!(text_of(&tester, "first-row"), "First row: 10");
    // The Id column is frozen: a horizontal scroll moves "Name" but not "Id".
    let id = tester.rect(tester.find(&Finder::text("Id")));
    let name = tester.rect(tester.find(&Finder::text("Name")));
    tester.scroll(&Finder::key("grid"), Vec2::new(200.0, 0.0));
    tester.advance(std::time::Duration::from_millis(200));
    assert_eq!(tester.rect(tester.find(&Finder::text("Id"))), id);
    let moved = tester.rect(tester.find(&Finder::text("Name")));
    assert!(moved.left < name.left, "{moved:?} vs {name:?}");
}

#[test]
fn gallery_11_sortable_headers() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Data grid")));
    tester.pump();
    assert_eq!(text_of(&tester, "sort"), "Sort: none");
    tester.tap(&Finder::text("City"));
    assert_eq!(text_of(&tester, "sort"), "Sort: City ascending");
    tester.tap(&Finder::text("City"));
    assert_eq!(text_of(&tester, "sort"), "Sort: City descending");
    tester.tap(&Finder::text("City"));
    assert_eq!(text_of(&tester, "sort"), "Sort: none");
}

#[test]
fn gallery_15_resize_and_reorder() {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Data grid")));
    tester.pump();
    assert_eq!(text_of(&tester, "name-width"), "Name width: 200");
    assert_eq!(
        text_of(&tester, "order"),
        "Order: Id, Name, City, Amount, Active, Score"
    );
    let grid = rect(&tester, "grid");
    let header_y = grid.top + 18.0;
    let drag = |tester: &mut WidgetTester, from: f32, to: f32| {
        for (kind, x) in [
            (PointerKind::Down(PointerButton::Primary), from),
            (PointerKind::Move, to),
            (PointerKind::Up(PointerButton::Primary), to),
        ] {
            tester.pointer(PointerEvent {
                kind,
                position: Point::new(grid.left + x, header_y),
                modifiers: Default::default(),
            });
            tester.pump();
        }
    };
    // City (180 px from 280) dragged 200 px left, past the middle of Name: its nearest
    // boundary is the one before Name.
    let city = rect(&tester, "grid").left;
    drag(&mut tester, 280.0 + 90.0, 280.0 + 90.0 - 200.0);
    assert_eq!(
        text_of(&tester, "order"),
        "Order: Id, City, Name, Amount, Active, Score"
    );
    let city_header = tester.rect(tester.find(&Finder::text("City")));
    assert!(
        city_header.left >= city + 80.0 && city_header.left < city + 260.0,
        "City where Name was: {city_header:?}"
    );
    // Name now spans 260..460: its right edge dragged 40 px.
    drag(&mut tester, 457.0, 497.0);
    assert_eq!(text_of(&tester, "name-width"), "Name width: 240");
    assert_eq!(text_of(&tester, "sort"), "Sort: none");
}

/// Opens the Kitchen sink section and lets its first frames settle.
fn open_sink() -> WidgetTester {
    let mut tester = WidgetTester::with_size(960.0, 640.0, gallery);
    tester.tap(&Finder::key(nav("Kitchen sink")));
    tester.pump();
    tester.pump();
    tester
}

fn sink_rect(tester: &WidgetTester, key: &str) -> Rect {
    rect(tester, &format!("sink-{key}"))
}

#[test]
fn gallery_12_kitchen_sink_cards() {
    assert_eq!(SECTIONS.last(), Some(&"Kitchen sink"));
    let mut tester = open_sink();
    tester.find(&Finder::key("kitchen-sink"));
    let page = rect(&tester, "sink");
    let first = sink_rect(&tester, SINK_CARDS[0].0);
    assert!(
        (first.top - page.top).abs() < 1.0,
        "first card at the top: {first:?} in {page:?}"
    );
    // Every card and every widget it lists, scrolled into view in turn.
    for (card, widgets) in SINK_CARDS {
        tester.ensure_visible(&Finder::key(format!("sink-{card}")));
        tester.pump();
        for widget in widgets.iter() {
            assert!(
                !tester
                    .find_all(&Finder::key(format!("sink-{widget}")))
                    .is_empty(),
                "card {card} lacks {widget}"
            );
        }
    }
    // Taller than the window: a wheel step moves the page.
    tester.scroll(&Finder::key("sink"), Vec2::new(-1e6, -1e6));
    tester.advance(std::time::Duration::from_millis(200));
    let before = sink_rect(&tester, "buttons").top;
    tester.scroll(&Finder::key("sink"), Vec2::new(0.0, 300.0));
    tester.advance(std::time::Duration::from_millis(200));
    assert_eq!(sink_rect(&tester, "buttons").top, before - 300.0);
}

#[test]
fn gallery_13_kitchen_sink_echoes() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    let mut tester = open_sink();
    let state = |t: &WidgetTester, card: &str| text_of(t, &format!("sink-{card}-state"));
    // Buttons.
    assert_eq!(state(&tester, "buttons"), "Pressed 0");
    tester.tap(&Finder::key("sink-button"));
    assert_eq!(state(&tester, "buttons"), "Pressed 1");
    // Toggles.
    tester.ensure_visible(&Finder::key("sink-toggles"));
    tester.pump();
    assert_eq!(
        state(&tester, "toggles"),
        "Checkbox on, Radio b, Switch off"
    );
    tester.tap(&Finder::key("sink-checkbox"));
    tester.tap(&Finder::key("sink-radio-c"));
    tester.tap(&Finder::key("sink-switch"));
    assert_eq!(
        state(&tester, "toggles"),
        "Checkbox off, Radio c, Switch on"
    );
    // Fields.
    tester.ensure_visible(&Finder::key("sink-fields"));
    tester.pump();
    assert_eq!(state(&tester, "fields"), "Typed: ");
    tester.enter_text(&Finder::key("sink-text_field"), "hi");
    assert_eq!(state(&tester, "fields"), "Typed: hi");
    // Grid: the filter switch and a header sort.
    tester.ensure_visible(&Finder::key("sink-grid"));
    tester.pump();
    assert_eq!(state(&tester, "grid"), "Sort: none, rows 1000");
    tester.tap(&Finder::key("sink-grid-filter"));
    assert_eq!(state(&tester, "grid"), "Sort: none, rows 500");
    tester.tap(&Finder::text("Id"));
    assert_eq!(state(&tester, "grid"), "Sort: Id ascending, rows 500");
    tester.tap(&Finder::text("Id"));
    assert_eq!(state(&tester, "grid"), "Sort: Id descending, rows 500");
    tester.find(&Finder::text("998"));
    // Focus: a tap focuses a button in the scope, Tab wraps inside it.
    tester.ensure_visible(&Finder::key("sink-focus"));
    tester.pump();
    tester.focus(&Finder::key("sink-focus-two"));
    assert_eq!(state(&tester, "focus"), "Focus: Two");
    tester.press_key(LogicalKey::Named(NamedKey::Tab), Modifiers::default());
    assert_eq!(state(&tester, "focus"), "Focus: Three");
    tester.press_key(LogicalKey::Named(NamedKey::Tab), Modifiers::default());
    assert_eq!(state(&tester, "focus"), "Focus: One");
}

#[test]
fn gallery_14_kitchen_sink_goldens() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    let mut tester = open_sink();
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-sink-top.png"
    ));
    // A tap on a card title near the top focuses the page's scroll view (a card further
    // down may sit below the window); End goes to the bottom.
    tester.tap(&Finder::text("Toggles"));
    tester.press_key(LogicalKey::Named(NamedKey::End), Modifiers::default());
    tester.pump();
    let last = sink_rect(&tester, "focus");
    assert!(
        last.bottom <= 640.0 && last.top > 0.0,
        "at the end: {last:?}"
    );
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-sink-end.png"
    ));
}

#[test]
fn gallery_18_kitchen_sink_grid_editing_and_selection() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    let mut t = open_sink();
    t.ensure_visible(&Finder::key("sink-grid"));
    t.pump();
    assert_eq!(text_of(&t, "sink-grid-edit-error"), "Edit error: none");
    t.find(&Finder::text("cargo run --release -p data-grid-stress"));
    t.tap(&Finder::text("Item 0"));
    assert_eq!(
        text_of(&t, "sink-grid-selection"),
        "Selection: (0, 1)..(0, 1)"
    );
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowDown),
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(
        text_of(&t, "sink-grid-selection"),
        "Selection: (0, 1)..(1, 1)"
    );
    t.press_key(LogicalKey::Named(NamedKey::F(2)), Modifiers::default());
    t.pump();
    for c in "Updated".chars() {
        t.press_key(
            LogicalKey::Character(c.to_string().into()),
            Modifiers::default(),
        );
    }
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    t.pump();
    t.find(&Finder::text("Updated"));
    // A client-sort grid clears selection when a successful edit recomputes its row map.
    t.tap(&Finder::text("Updated"));
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.press_key(LogicalKey::Named(NamedKey::F(2)), Modifiers::default());
    t.pump();
    for c in "101".chars() {
        t.press_key(
            LogicalKey::Character(c.to_string().into()),
            Modifiers::default(),
        );
    }
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    t.pump();
    assert_eq!(
        text_of(&t, "sink-grid-edit-error"),
        "Edit error: Score must be between 0 and 100"
    );
    assert!(
        t.a11y_tree()
            .nodes
            .iter()
            .any(|(_, n)| n.role() == tantu_test::accesskit::Role::TextInput)
    );
    t.press_key(LogicalKey::Named(NamedKey::Escape), Modifiers::default());
    t.pump();
    // A double click enters Name again; Escape preserves its committed value.
    t.tap(&Finder::text("Updated"));
    t.tap(&Finder::text("Updated"));
    t.pump();
    assert!(
        t.a11y_tree()
            .nodes
            .iter()
            .any(|(_, n)| n.role() == tantu_test::accesskit::Role::TextInput)
    );
    t.press_key(LogicalKey::Named(NamedKey::Escape), Modifiers::default());
    t.pump();
    t.find(&Finder::text("Updated"));
}

#[test]
fn gallery_18_kitchen_sink_grid_header_controls_and_golden() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    let mut t = open_sink();
    t.ensure_visible(&Finder::key("sink-grid"));
    t.pump();
    assert_eq!(text_of(&t, "sink-grid-name-width"), "Name width: 200");
    assert_eq!(text_of(&t, "sink-grid-order"), "Order: Id, Name, Score");
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-sink-grid.png"
    ));
    t.tap(&Finder::text("Item 0"));
    t.press_key(LogicalKey::Named(NamedKey::F(6)), Modifiers::default());
    let primary = Modifiers {
        control: !cfg!(target_os = "macos"),
        super_key: cfg!(target_os = "macos"),
        ..Default::default()
    };
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers {
            alt: true,
            ..Default::default()
        },
    );
    assert_eq!(text_of(&t, "sink-grid-name-width"), "Name width: 210");
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers {
            shift: true,
            ..primary
        },
    );
    assert_eq!(text_of(&t, "sink-grid-order"), "Order: Id, Score, Name");
}

#[test]
fn gallery_19_kitchen_sink_popup_inherits_local_theme() {
    let mut t = open_sink();
    assert_eq!(
        text_of(&t, "sink-popup-state"),
        "Popup: hidden, theme: dark"
    );
    t.tap(&Finder::key("sink-overlay_portal"));
    t.pump();
    let popup = t.find(&Finder::key("sink-popup-close"));
    let target = rect(&t, "sink-popup-target");
    let bounds = t.rect(popup);
    assert!(
        (bounds.left - target.right - 12.0).abs() < 0.01,
        "popup beside target: {target:?} {bounds:?}"
    );
    assert!((bounds.center().y - target.center().y).abs() < 0.01);
    let fills = |t: &WidgetTester, color| {
        t.frame()
            .entries_for(popup)
            .any(|e| matches!(e.command, Command::Fill { color: c, .. } if c == color))
    };
    assert!(fills(&t, Theme::dark().colors.primary));
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-sink-popup.png"
    ));
    t.tap(&Finder::key("sink-popup-dark"));
    t.pump();
    assert_eq!(t.find(&Finder::key("sink-popup-close")), popup);
    assert!(fills(&t, Theme::light().colors.primary));
    assert_eq!(
        text_of(&t, "sink-popup-state"),
        "Popup: visible, theme: light"
    );
    // The global gallery switch is outside the target and closes the popup on press.
    t.tap(&Finder::key("dark"));
    assert!(t.find_all(&Finder::key("sink-popup-close")).is_empty());
    t.tap(&Finder::key("sink-overlay_portal"));
    t.pump();
    let close = t.find(&Finder::key("sink-popup-close"));
    assert!(t.frame().entries_for(close).any(|e| matches!(e.command, Command::Fill { color, .. } if color == Theme::light().colors.primary)), "local light theme overrides global dark");
    t.tap(&Finder::key("sink-popup-close"));
    assert!(t.find_all(&Finder::key("sink-popup-close")).is_empty());
}

#[test]
fn gallery_20_tree_section_navigation_activation_and_golden() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    let mut t = WidgetTester::with_size(960.0, 640.0, gallery);
    assert!(SECTIONS.contains(&"Tree view"));
    t.tap(&Finder::key("nav-tree-view"));
    t.pump();
    t.tap(&Finder::text("src"));
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.pump();
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.pump();
    assert_eq!(text_of(&t, "tree-selection"), "Selected: 2");
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    assert_eq!(text_of(&t, "tree-activated"), "Activated: 2");
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-tree.png"
    ));
    t.tap(&Finder::text("Generated"));
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.press_key(LogicalKey::Named(NamedKey::End), Modifiers::default());
    t.pump();
    t.find(&Finder::text("Generated 4999"));
    assert!(
        t.a11y_tree()
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == tantu_test::accesskit::Role::TreeItem)
            .count()
            < 25
    );
}

#[test]
fn gallery_21_kitchen_sink_tree_selection_collapse_and_golden() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    assert!(
        SINK_CARDS
            .iter()
            .any(|(key, widgets)| *key == "tree" && widgets.contains(&"tree_view"))
    );
    let mut t = open_sink();
    t.ensure_visible(&Finder::key("sink-tree"));
    t.pump();
    t.tap(&Finder::text("src"));
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.pump();
    assert_eq!(text_of(&t, "sink-tree-state"), "Selected: 2, expanded: 2");
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    assert_eq!(text_of(&t, "sink-tree-activated"), "Activated: 2");
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-sink-tree.png"
    ));
    // Collapse Workspace through its disclosure control.
    let tree = rect(&t, "sink-tree_view");
    t.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: Point::new(tree.left + 10.0, tree.top + 16.0),
        modifiers: Default::default(),
    });
    t.pump();
    assert_eq!(text_of(&t, "sink-tree-state"), "Selected: 0, expanded: 1");
}

#[test]
fn gallery_22_tabs_section_keeps_pages_and_skips_archived() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    assert_eq!(SECTIONS.len(), 13);
    assert_eq!(
        SECTIONS.iter().position(|s| *s == "Tabs"),
        SECTIONS
            .iter()
            .position(|s| *s == "Kitchen sink")
            .map(|i| i - 1)
    );
    let mut t = WidgetTester::with_size(960.0, 640.0, gallery);
    t.tap(&Finder::key(nav("Tabs")));
    t.pump();
    t.pump();
    assert_eq!(text_of(&t, "tab-echo"), "Tab: General");
    t.find(&Finder::key("tabs-scrollable"));
    t.enter_text(&Finder::key("tabs-field"), "hello");
    t.pump();
    let control = Modifiers {
        control: true,
        ..Default::default()
    };
    t.press_key(LogicalKey::Named(NamedKey::Tab), control);
    t.pump();
    t.pump();
    assert_eq!(text_of(&t, "tab-echo"), "Tab: Advanced");
    t.tap(&Finder::text("General"));
    t.pump();
    t.pump();
    t.find(&Finder::text("hello"));
    // Arrows skip the disabled Archived tab.
    t.tap(&Finder::text("Data"));
    t.pump();
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.pump();
    assert_eq!(text_of(&t, "tab-echo"), "Tab: General");
    t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    t.pump();
    t.advance(std::time::Duration::from_millis(400));
    assert_eq!(text_of(&t, "tab-echo"), "Tab: Advanced");
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-tabs.png"
    ));
}

#[test]
fn gallery_23_kitchen_sink_tabs_card() {
    let index = |key: &str| SINK_CARDS.iter().position(|(k, _)| *k == key);
    assert_eq!(SINK_CARDS.len(), 16);
    assert_eq!(index("tabs"), index("tree").map(|i| i + 1));
    assert!(
        SINK_CARDS
            .iter()
            .any(|(key, widgets)| *key == "tabs" && widgets == &["tab_bar", "tab_view"])
    );
    let mut t = open_sink();
    t.ensure_visible(&Finder::key("sink-tabs"));
    t.pump();
    assert_eq!(sink_rect(&t, "tab_view").height(), 200.0);
    assert_eq!(text_of(&t, "sink-tabs-state"), "Tab: Note, range: Day");
    t.tap(&Finder::text("Option"));
    t.pump();
    t.tap(&Finder::text("Week"));
    t.pump();
    t.pump();
    t.advance(std::time::Duration::from_millis(400));
    assert_eq!(text_of(&t, "sink-tabs-state"), "Tab: Option, range: Week");
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-sink-tabs.png"
    ));
}

#[test]
fn gallery_24_split_view_section() {
    use tantu::view::{LogicalKey, Modifiers, NamedKey};
    assert_eq!(SECTIONS.len(), 13);
    let position = |name: &str| SECTIONS.iter().position(|s| *s == name);
    assert_eq!(position("Split view"), position("Tabs").map(|i| i - 1));
    let mut t = WidgetTester::with_size(960.0, 640.0, gallery);
    t.tap(&Finder::key(nav("Split view")));
    t.pump();
    t.pump();
    for key in [
        "split",
        "split-folders",
        "split-messages",
        "split-preview",
        "split-output",
    ] {
        t.find(&Finder::key(key));
    }
    let initial = "Sizes: 25 % · 35 % · 40 %";
    assert_eq!(text_of(&t, "split-echo"), initial);
    // Dragging the first divider changes the sizes.
    t.drag(
        &Finder::semantics_label("Resize Folders"),
        Vec2::new(60.0, 0.0),
    );
    t.pump();
    let dragged = text_of(&t, "split-echo");
    assert_ne!(dragged, initial);
    // Enter collapses Folders and restores it.
    t.focus(&Finder::semantics_label("Resize Folders"));
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    t.pump();
    t.pump();
    assert!(text_of(&t, "split-echo").starts_with("Sizes: 0 % · "));
    t.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    t.pump();
    t.pump();
    assert_eq!(text_of(&t, "split-echo"), dragged);
    // Reset restores the initial sizes.
    t.tap(&Finder::text("Reset"));
    t.pump();
    t.pump();
    assert_eq!(text_of(&t, "split-echo"), initial);
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-split.png"
    ));
}

#[test]
fn gallery_25_kitchen_sink_split_card() {
    let index = |key: &str| SINK_CARDS.iter().position(|(k, _)| *k == key);
    assert_eq!(SINK_CARDS.len(), 16);
    assert_eq!(index("split"), index("tabs").map(|i| i + 1));
    assert!(
        SINK_CARDS
            .iter()
            .any(|(key, widgets)| *key == "split" && widgets == &["split_view"])
    );
    let mut t = open_sink();
    t.ensure_visible(&Finder::key("sink-split"));
    t.pump();
    assert_eq!(sink_rect(&t, "split_view").height(), 160.0);
    assert_eq!(text_of(&t, "sink-split-state"), "Sizes: 50 % · 50 %");
    t.drag(
        &Finder::semantics_label("Resize Left"),
        Vec2::new(40.0, 0.0),
    );
    t.pump();
    assert_ne!(text_of(&t, "sink-split-state"), "Sizes: 50 % · 50 %");
}

#[test]
fn gallery_26_docking_section() {
    assert_eq!(SECTIONS.len(), 13);
    let position = |name: &str| SECTIONS.iter().position(|s| *s == name);
    assert_eq!(position("Docking"), position("Split view").map(|i| i - 1));
    let mut t = WidgetTester::with_size(960.0, 640.0, gallery);
    t.tap(&Finder::key(nav("Docking")));
    t.pump();
    t.pump();
    t.find(&Finder::key("dock"));
    assert_eq!(text_of(&t, "dock-echo"), "Active: Explorer · Panels: 6");
    t.tap(&Finder::semantics_label("Search"));
    t.pump();
    assert_eq!(text_of(&t, "dock-echo"), "Active: Search · Panels: 6");
    // Text typed in the editor survives switching to Readme and back.
    t.enter_text(&Finder::key("dock-editor"), "hello");
    t.tap(&Finder::semantics_label("Readme"));
    t.pump();
    t.tap(&Finder::semantics_label("Editor"));
    t.pump();
    t.find(&Finder::text("hello"));
    // Closing updates the echo; Save, close, Restore brings a panel back.
    t.tap(&Finder::semantics_label("Close Problems"));
    t.pump();
    assert!(text_of(&t, "dock-echo").ends_with("Panels: 5"));
    t.tap(&Finder::text("Save"));
    t.pump();
    t.tap(&Finder::semantics_label("Close Terminal"));
    t.pump();
    assert!(t.find_all(&Finder::semantics_label("Terminal")).is_empty());
    t.tap(&Finder::text("Restore"));
    t.pump();
    t.pump();
    t.find(&Finder::semantics_label("Terminal"));
    assert!(text_of(&t, "dock-echo").ends_with("Panels: 5"));
    t.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-docking.png"
    ));
}

#[test]
fn gallery_27_kitchen_sink_dock_card() {
    let index = |key: &str| SINK_CARDS.iter().position(|(k, _)| *k == key);
    assert_eq!(SINK_CARDS.len(), 16);
    assert_eq!(index("dock"), index("split").map(|i| i + 1));
    assert!(
        SINK_CARDS
            .iter()
            .any(|(key, widgets)| *key == "dock" && widgets == &["dock_area"])
    );
    let mut t = open_sink();
    t.ensure_visible(&Finder::key("sink-dock"));
    t.pump();
    let rect = sink_rect(&t, "dock_area");
    assert_eq!((rect.width(), rect.height()), (600.0, 220.0));
    assert_eq!(text_of(&t, "sink-dock-state"), "Active: Notes · Panels: 3");
    t.tap(&Finder::semantics_label("Log"));
    t.pump();
    t.pump();
    assert_eq!(text_of(&t, "sink-dock-state"), "Active: Log · Panels: 3");
}
