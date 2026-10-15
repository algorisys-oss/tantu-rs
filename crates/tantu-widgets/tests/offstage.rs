//! Tests for `docs/specs/view/offstage.md`, rule WIDGETS-OFFSTAGE-01.

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::WidgetTester;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{BuildCx, Builder};
use tantu_widgets::{Column, Offstage, Text};

fn labels(t: &mut WidgetTester) -> Vec<String> {
    t.a11y_tree()
        .nodes
        .iter()
        .filter_map(|(_, n)| n.label().map(str::to_owned))
        .collect()
}

#[test]
fn widgets_offstage_01_builds_once_and_follows_the_prop() {
    let builds = Rc::new(Cell::new(0));
    let hidden: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let (b, h) = (builds.clone(), hidden.clone());
    let mut t = WidgetTester::new(move || {
        let flag = signal(true);
        h.set(Some(flag));
        Column::new()
            .child(Text::new("always"))
            .child(
                Offstage::new(flag).child(Builder::new(move |_: &mut BuildCx<'_>| {
                    b.set(b.get() + 1);
                    Text::new("kept")
                })),
            )
    });
    t.pump();
    let hidden = hidden.get().expect("the app ran");
    assert_eq!(builds.get(), 1, "built while starting offstage");
    assert!(!labels(&mut t).contains(&"kept".to_owned()));
    t.enter(|| hidden.set(false));
    t.pump();
    assert!(labels(&mut t).contains(&"kept".to_owned()));
    t.enter(|| hidden.set(true));
    t.pump();
    assert!(!labels(&mut t).contains(&"kept".to_owned()));
    t.enter(|| hidden.set(false));
    t.pump();
    assert!(labels(&mut t).contains(&"kept".to_owned()));
    assert_eq!(builds.get(), 1, "never rebuilt");
}
