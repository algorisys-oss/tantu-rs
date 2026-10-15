//! Tests for `docs/specs/widgets/directionality.md`, rules WIDGETS-DIR-01..03.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::Rect;
use tantu_view::layout::{AlignmentDirectional, EdgeInsetsDirectional, TextDirection};
use tantu_view::reactive::{Signal, signal};
use tantu_view::{Anchor, Builder, Keyed, OverlayPortal};
use tantu_widgets::{
    Align, Column, Directionality, Padding, PositionedDirectional, Row, SizedBox, Stack,
};

const W: f32 = 400.0;
const H: f32 = 300.0;

type Seen = Rc<RefCell<Vec<(&'static str, TextDirection)>>>;

/// Records the direction a builder sees under `name`.
fn probe(seen: Seen, name: &'static str) -> impl tantu_view::View {
    Builder::new(move |cx| {
        seen.borrow_mut().push((name, Directionality::of(cx)));
        SizedBox::new(1.0, 1.0)
    })
}

#[test]
fn widgets_dir_01_nearest_direction() {
    let seen: Seen = Rc::default();
    let mut t = {
        let seen = seen.clone();
        WidgetTester::with_size(W, H, move || {
            Column::new()
                .child(probe(seen.clone(), "none"))
                .child(Directionality::new(
                    TextDirection::Rtl,
                    Column::new()
                        .child(probe(seen.clone(), "rtl"))
                        .child(Directionality::new(
                            TextDirection::Ltr,
                            probe(seen.clone(), "nested"),
                        ))
                        .child(
                            OverlayPortal::new(SizedBox::new(10.0, 10.0))
                                .visible(true)
                                .anchor(Anchor::below())
                                .overlay({
                                    let seen = seen.clone();
                                    move || probe(seen.clone(), "overlay")
                                }),
                        ),
                ))
        })
    };
    t.pump();
    t.pump();
    let seen = seen.borrow();
    let of = |name| {
        seen.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, d)| *d)
            .unwrap_or_else(|| panic!("{name} not built"))
    };
    assert_eq!(of("none"), TextDirection::Ltr);
    assert_eq!(of("rtl"), TextDirection::Rtl);
    assert_eq!(of("nested"), TextDirection::Ltr);
    assert_eq!(of("overlay"), TextDirection::Rtl);
}

struct Fx {
    t: WidgetTester,
    direction: Signal<TextDirection>,
}

/// `make()` inside a `Directionality` following a signal, starting `Rtl`.
fn fixture<V: tantu_view::View>(make: impl Fn() -> V + 'static) -> Fx {
    let out: Rc<RefCell<Option<Signal<TextDirection>>>> = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(W, H, move || {
            let direction = signal(TextDirection::Rtl);
            *out.borrow_mut() = Some(direction);
            Directionality::new(direction, make())
        })
    };
    t.pump();
    let direction = out.borrow_mut().take().expect("built");
    Fx { t, direction }
}

impl Fx {
    fn rect(&self, key: &str) -> Rect {
        self.t.rect(self.t.find(&Finder::key(key)))
    }

    fn flip(&mut self, to: TextDirection) {
        let direction = self.direction;
        self.t.enter(|| direction.set(to));
        self.t.pump();
        self.t.pump();
    }
}

#[test]
fn widgets_dir_02_row_and_column_follow() {
    let mut fx = fixture(|| {
        Column::new()
            .child(
                Row::new()
                    .child(Keyed::new("a", SizedBox::new(50.0, 20.0)))
                    .child(Keyed::new("b", SizedBox::new(30.0, 20.0))),
            )
            .child(Keyed::new("c", SizedBox::new(40.0, 20.0)))
    });
    // A row runs right to left; a column's children are centered either way, but its
    // cross start is the right edge.
    assert_eq!(fx.rect("a").right, W);
    assert_eq!(fx.rect("b").right, W - 50.0);
    fx.flip(TextDirection::Ltr);
    assert_eq!(fx.rect("a").left, 0.0);
    assert_eq!(fx.rect("b").left, 50.0);
    let fx = fixture(|| {
        Column::new()
            .cross_axis_alignment(tantu_view::layout::CrossAxisAlignment::Start)
            .child(SizedBox::new(W, 1.0))
            .child(Keyed::new("c", SizedBox::new(40.0, 20.0)))
    });
    assert_eq!(fx.rect("c").right, W);
}

#[test]
fn widgets_dir_03_directional_padding_align_positioned() {
    let mut fx = fixture(|| {
        Column::new()
            .child(
                SizedBox::new(W, 40.0).child(
                    Padding::directional(EdgeInsetsDirectional::new(10.0, 0.0, 0.0, 0.0)).child(
                        Align::directional(AlignmentDirectional::CENTER_START)
                            .child(Keyed::new("aligned", SizedBox::new(20.0, 20.0))),
                    ),
                ),
            )
            .child(
                SizedBox::new(W, 100.0).child(
                    Stack::new().child(
                        PositionedDirectional::new(Keyed::new(
                            "positioned",
                            SizedBox::new(30.0, 30.0),
                        ))
                        .start(5.0)
                        .top(0.0),
                    ),
                ),
            )
    });
    assert_eq!(fx.rect("aligned").right, W - 10.0);
    assert_eq!(fx.rect("positioned").right, W - 5.0);
    fx.flip(TextDirection::Ltr);
    assert_eq!(fx.rect("aligned").left, 10.0);
    assert_eq!(fx.rect("positioned").left, 5.0);
}
