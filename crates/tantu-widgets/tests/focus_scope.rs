//! Tests for `docs/specs/view/focus-scope.md`, rule WIDGETS-FOCUSSCOPE-01.

use tantu_test::accesskit::NodeId;
use tantu_test::{Finder, WidgetTester};
use tantu_view::{Keyed, LogicalKey, Modifiers, NamedKey};
use tantu_widgets::{Button, Column, FocusScope};

/// The key of the focused button, read from the accessibility tree.
fn focused(tester: &mut WidgetTester) -> Option<&'static str> {
    let focus = tester.a11y_tree().focus;
    ["a", "b", "c", "d"]
        .into_iter()
        .find(|key| NodeId(tester.find(&Finder::key(*key)).to_raw()) == focus)
}

fn tab(tester: &mut WidgetTester, shift: bool) {
    tester.press_key(
        LogicalKey::Named(NamedKey::Tab),
        Modifiers {
            shift,
            ..Modifiers::default()
        },
    );
}

#[test]
fn widgets_focusscope_01_tab_cycles_inside_the_scope() {
    let mut tester = WidgetTester::new(|| {
        Column::new()
            .child(Keyed::new("a", Button::new("a")))
            .child(FocusScope::new(
                Column::new()
                    .child(Keyed::new("b", Button::new("b")))
                    .child(Keyed::new("c", Button::new("c"))),
            ))
            .child(Keyed::new("d", Button::new("d")))
    });
    tester.focus(&Finder::key("b"));
    tab(&mut tester, false);
    assert_eq!(focused(&mut tester), Some("c"));
    tab(&mut tester, false);
    assert_eq!(focused(&mut tester), Some("b"));
    tab(&mut tester, true);
    assert_eq!(focused(&mut tester), Some("c"));
    // Outside the scope, focus and traversal work as before, and Tab enters the scope.
    tester.focus(&Finder::key("d"));
    assert_eq!(focused(&mut tester), Some("d"));
    tab(&mut tester, false);
    assert_eq!(focused(&mut tester), Some("a"));
    tab(&mut tester, false);
    assert_eq!(focused(&mut tester), Some("b"));
    // Nothing is painted for the scope: the frame has the same commands as without it.
    let with_scope = tester.frame().scene.entries().len();
    let mut plain = WidgetTester::new(|| {
        Column::new()
            .child(Keyed::new("a", Button::new("a")))
            .child(
                Column::new()
                    .child(Keyed::new("b", Button::new("b")))
                    .child(Keyed::new("c", Button::new("c"))),
            )
            .child(Keyed::new("d", Button::new("d")))
    });
    plain.focus(&Finder::key("b"));
    assert_eq!(with_scope, plain.frame().scene.entries().len());
}
