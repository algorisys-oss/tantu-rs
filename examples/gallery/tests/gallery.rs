//! Tests for `docs/specs/examples/gallery.md`, rules GALLERY-01..11, and
//! `docs/specs/examples/kitchen-sink.md`, GALLERY-12..14.

use gallery::{SECTIONS, SINK_CARDS, gallery};
use tantu::core::{Rect, Vec2};
use tantu::scene::Command;
use tantu::theme::Theme;
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
    tester.tap(&Finder::key("sink-focus-two"));
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
    // A tap on the page's heading area focuses the scroll view; End goes to the bottom.
    tester.tap(&Finder::key("sink-text"));
    tester.press_key(LogicalKey::Named(NamedKey::End), Modifiers::default());
    tester.pump();
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/gallery-sink-end.png"
    ));
}
