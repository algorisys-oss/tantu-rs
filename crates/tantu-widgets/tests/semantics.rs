//! Tests for `docs/specs/a11y/semantics.md`, rules WIDGETS-SEM-01..02.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use tantu_test::accesskit::{Action, Node, NodeId, Role, Toggled, TreeUpdate};
use tantu_test::{Finder, WidgetTester};
use tantu_view::core::Vec2;
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{Keyed, SemanticsAction};
use tantu_widgets::{
    Button, Checkbox, Column, Icon, IconButton, Icons, Image, ListView, Radio, RichText,
    ScrollView, SizedBox, Slider, Switch, Text, TextEditingController, TextField, TextSpan,
    Tooltip, decode_png,
};

/// The node of the element keyed `key`.
fn node<'a>(tester: &WidgetTester, update: &'a TreeUpdate, key: &str) -> &'a Node {
    let id = tester.find(&Finder::key(key));
    let wanted = NodeId(id.to_raw());
    &update
        .nodes
        .iter()
        .find(|(n, _)| *n == wanted)
        .unwrap_or_else(|| panic!("no node for {key}"))
        .1
}

fn nodes_with_role(update: &TreeUpdate, role: Role) -> Vec<&Node> {
    update
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == role)
        .map(|(_, n)| n)
        .collect()
}

#[test]
fn widgets_sem_01_text_buttons_images() {
    let enabled_out: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let png = {
        // A 2 × 2 PNG made by the software renderer's encoder is overkill: a tiny hand-made one.
        include_bytes!("../../tantu-render-conformance/goldens/images.png")
    };
    let mut tester = {
        let enabled_out = enabled_out.clone();
        WidgetTester::new(move || {
            let enabled = signal(true);
            enabled_out.set(Some(enabled));
            let label = signal("Go".to_owned());
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(Keyed::new("text", Text::new(move || label.get())))
                .child(Keyed::new(
                    "rich",
                    RichText::new(
                        TextSpan::group()
                            .child(TextSpan::new("Rich "))
                            .child(TextSpan::new("text")),
                    ),
                ))
                .child(Keyed::new(
                    "button",
                    Button::new("Press").enabled(enabled).on_press(|| {}),
                ))
                .child(Keyed::new(
                    "icon-button",
                    IconButton::new(Icons::HOME)
                        .semantics_label("Home")
                        .on_press(|| {}),
                ))
                .child(Keyed::new(
                    "unnamed",
                    IconButton::new(Icons::HOME).on_press(|| {}),
                ))
                .child(Keyed::new(
                    "icon",
                    Icon::new(Icons::STAR).semantics_label("Star"),
                ))
                .child(Keyed::new(
                    "image",
                    Image::new(decode_png(png).expect("a png"))
                        .size(10.0, 10.0)
                        .semantics_label("Shapes"),
                ))
                .child(Keyed::new("label-signal", Text::new(move || label.get())))
                .child(Keyed::new(
                    "set",
                    Button::new("Rename").on_press(move || label.set("Gone".to_owned())),
                ))
        })
    };
    let update = tester.a11y_tree();
    let text = node(&tester, &update, "text");
    assert_eq!((text.role(), text.label()), (Role::Label, Some("Go")));
    let rich = node(&tester, &update, "rich");
    assert_eq!(
        (rich.role(), rich.label()),
        (Role::Label, Some("Rich text"))
    );
    let button = node(&tester, &update, "button");
    assert_eq!(
        (button.role(), button.label()),
        (Role::Button, Some("Press"))
    );
    assert!(!button.is_disabled());
    assert!(button.supports_action(Action::Click) && button.supports_action(Action::Focus));
    let icon_button = node(&tester, &update, "icon-button");
    assert_eq!(
        (icon_button.role(), icon_button.label()),
        (Role::Button, Some("Home"))
    );
    let unnamed = node(&tester, &update, "unnamed");
    assert_eq!((unnamed.role(), unnamed.label()), (Role::Button, None));
    let icon = node(&tester, &update, "icon");
    assert_eq!((icon.role(), icon.label()), (Role::Image, Some("Star")));
    let image = node(&tester, &update, "image");
    assert_eq!((image.role(), image.label()), (Role::Image, Some("Shapes")));
    // Reactive: a disabled button, a renamed text.
    let enabled = enabled_out.get().expect("built");
    tester.enter(|| enabled.set(false));
    tester.tap(&Finder::key("set"));
    let update = tester.a11y_tree();
    assert!(node(&tester, &update, "button").is_disabled());
    assert_eq!(node(&tester, &update, "text").label(), Some("Gone"));
}

#[test]
fn widgets_sem_02_controls_lists_tooltips() {
    type State = (
        Signal<bool>,
        Signal<bool>,
        Signal<f64>,
        Signal<Option<usize>>,
        TextEditingController,
    );
    let state: Rc<Cell<Option<State>>> = Rc::default();
    let mut tester = {
        let state = state.clone();
        WidgetTester::with_size(600.0, 600.0, move || {
            let (checked, indeterminate, value, selection) =
                (signal(false), signal(false), signal(40.0f64), signal(None));
            let field = TextEditingController::new("hello");
            state.set(Some((checked, indeterminate, value, selection, field)));
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(Keyed::new(
                    "checkbox",
                    Checkbox::new(checked)
                        .indeterminate(indeterminate)
                        .on_change(move |v| checked.set(v)),
                ))
                .child(Keyed::new("radio", Radio::new(1, 1).on_change(|_| {})))
                .child(Keyed::new("radio-off", Radio::new(2, 1).on_change(|_| {})))
                .child(Keyed::new("switch", Switch::new(true).on_change(|_| {})))
                .child(Keyed::new("disabled-switch", Switch::new(false)))
                .child(
                    SizedBox::width(300.0).child(Keyed::new(
                        "slider",
                        Slider::new(value)
                            .range(0.0, 100.0)
                            .on_change(move |v| value.set(v)),
                    )),
                )
                .child(SizedBox::width(300.0).child(Keyed::new(
                    "field",
                    TextField::new(field).placeholder("Name"),
                )))
                .child(SizedBox::width(300.0).child(Keyed::new(
                    "notes",
                    TextField::new(TextEditingController::new("")).max_lines(None),
                )))
                .child(SizedBox::new(300.0, 100.0).child(Keyed::new(
                    "scroll",
                    ScrollView::new(SizedBox::new(300.0, 400.0)),
                )))
                .child(
                    SizedBox::new(300.0, 100.0).child(Keyed::new(
                        "list",
                        ListView::builder(50, 40.0, |i| Text::new(format!("Item {i}")))
                            .selection(selection),
                    )),
                )
                .child(Keyed::new(
                    "tip-target",
                    Tooltip::new("Help", Button::new("?").on_press(|| {})),
                ))
        })
    };
    let (checked, indeterminate, value, selection, field) = state.get().expect("built");
    // The list's items appear at the second frame (WIDGETS-LIST-02).
    tester.pump();
    let update = tester.a11y_tree();
    let checkbox = node(&tester, &update, "checkbox");
    assert_eq!(
        (checkbox.role(), checkbox.toggled()),
        (Role::CheckBox, Some(Toggled::False))
    );
    assert!(checkbox.supports_action(Action::Click));
    let radio = node(&tester, &update, "radio");
    assert_eq!(
        (radio.role(), radio.toggled()),
        (Role::RadioButton, Some(Toggled::True))
    );
    assert_eq!(
        node(&tester, &update, "radio-off").toggled(),
        Some(Toggled::False)
    );
    let switch = node(&tester, &update, "switch");
    assert_eq!(
        (switch.role(), switch.toggled()),
        (Role::Switch, Some(Toggled::True))
    );
    assert!(node(&tester, &update, "disabled-switch").is_disabled());
    let slider = node(&tester, &update, "slider");
    assert_eq!(slider.role(), Role::Slider);
    assert_eq!(slider.numeric_value(), Some(40.0));
    assert_eq!(
        (slider.min_numeric_value(), slider.max_numeric_value()),
        (Some(0.0), Some(100.0))
    );
    assert_eq!(slider.numeric_value_step(), Some(5.0));
    assert_eq!(slider.value(), Some("40"));
    let field_node = node(&tester, &update, "field");
    assert_eq!(
        (field_node.role(), field_node.label(), field_node.value()),
        (Role::TextInput, Some("Name"), Some("hello"))
    );
    assert!(field_node.supports_action(Action::SetValue));
    assert_eq!(
        node(&tester, &update, "notes").role(),
        Role::MultilineTextInput
    );
    assert_eq!(node(&tester, &update, "scroll").role(), Role::ScrollView);
    let list = node(&tester, &update, "list");
    assert_eq!(list.role(), Role::List);
    let items = nodes_with_role(&update, Role::ListItem);
    assert!(items.len() >= 3, "{}", items.len());
    assert!(items.iter().all(|i| i.is_selected() == Some(false)));
    assert!(nodes_with_role(&update, Role::Tooltip).is_empty());
    // Reactive states.
    tester.enter(|| {
        checked.set(true);
        value.set(62.5);
        selection.set(Some(1));
    });
    tester.pump();
    tester.hover(&Finder::key("tip-target"));
    tester.advance(Duration::from_millis(500));
    let update = tester.a11y_tree();
    assert_eq!(
        node(&tester, &update, "checkbox").toggled(),
        Some(Toggled::True)
    );
    assert_eq!(node(&tester, &update, "slider").value(), Some("62.5"));
    let items = nodes_with_role(&update, Role::ListItem);
    assert_eq!(
        items
            .iter()
            .filter(|i| i.is_selected() == Some(true))
            .count(),
        1
    );
    let tips = nodes_with_role(&update, Role::Tooltip);
    assert_eq!(tips.len(), 1);
    assert_eq!(tips[0].label(), Some("Help"));
    tester.enter(|| indeterminate.set(true));
    tester.pump();
    let update = tester.a11y_tree();
    assert_eq!(
        node(&tester, &update, "checkbox").toggled(),
        Some(Toggled::Mixed)
    );
    // Actions through the tester: increment steps the slider, SetValue replaces the text.
    assert!(tester.perform_action(&Finder::key("slider"), SemanticsAction::Increment));
    assert_eq!(tester.enter(|| value.get()), 67.5);
    assert!(tester.perform_action(
        &Finder::key("field"),
        SemanticsAction::SetValue("bye".into())
    ));
    assert_eq!(tester.enter(|| field.text()), "bye");
    // ScrollIntoView on a scrolled-out list item.
    tester.scroll(&Finder::key("list"), Vec2::new(0.0, 400.0));
    tester.advance(Duration::from_millis(150));
    let first = tester.find_all(&Finder::text("Item 0"));
    assert!(first.is_empty());
    assert!(tester.perform_action(&Finder::text("Item 12"), SemanticsAction::ScrollIntoView));
    let rect = tester.rect(tester.find(&Finder::text("Item 12")));
    let list_rect = tester.rect(tester.find(&Finder::key("list")));
    assert!(
        rect.top >= list_rect.top && rect.bottom <= list_rect.bottom,
        "{rect:?} in {list_rect:?}"
    );
}
