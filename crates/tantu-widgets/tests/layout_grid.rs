//! Tests for `docs/specs/widgets/layout-grid.md`, rule WIDGETS-LGRID-01.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::Keyed;
use tantu_view::layout::{TextDirection, TrackSize};
use tantu_view::reactive::{Signal, signal};
use tantu_widgets::{Directionality, GridPlacement, LayoutGrid, SizedBox};

#[test]
fn widgets_lgrid_01_layout_grid() {
    let out: Rc<RefCell<Option<Signal<TextDirection>>>> = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let direction = signal(TextDirection::Ltr);
            *out.borrow_mut() = Some(direction);
            Directionality::new(
                direction,
                SizedBox::width(300.0).child(
                    LayoutGrid::new([TrackSize::AUTO, TrackSize::fr(1.0)])
                        .column_gap(10.0)
                        .row_gap(4.0)
                        .child(Keyed::new("label", SizedBox::new(60.0, 20.0)))
                        .child(Keyed::new("value", SizedBox::new(1.0, 20.0)))
                        .child(
                            GridPlacement::new(Keyed::new("wide", SizedBox::new(1.0, 30.0)))
                                .column(0)
                                .row(1)
                                .column_span(2),
                        ),
                ),
            )
        })
    };
    t.pump();
    let rect = |t: &WidgetTester, key: &str| t.rect(t.find(&Finder::key(key)));
    let (label, value, wide) = (rect(&t, "label"), rect(&t, "value"), rect(&t, "wide"));
    assert_eq!((label.left, label.width()), (0.0, 60.0));
    assert_eq!((value.left, value.width()), (70.0, 230.0));
    assert_eq!((wide.left, wide.top, wide.width()), (0.0, 24.0, 300.0));
    let direction = out.borrow_mut().take().expect("built");
    t.enter(|| direction.set(TextDirection::Rtl));
    t.pump();
    t.pump();
    let (label, value) = (rect(&t, "label"), rect(&t, "value"));
    assert_eq!(label.right, 300.0);
    assert_eq!(value.left, 0.0);
}
