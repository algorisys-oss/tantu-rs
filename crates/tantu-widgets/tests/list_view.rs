//! Tests for `docs/specs/widgets/list-view.md`, rules WIDGETS-LIST-01..03.

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Size, Vec2};
use tantu_view::layout::RenderParagraph;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{Keyed, View};
use tantu_widgets::{ListView, ScrollController, Text};

type Out = Rc<Cell<Option<ScrollController>>>;

/// A 300 × 200 window with a list keyed "l" built by `make`, pumped twice (the items appear
/// at the second frame), exposing the list's controller and the builder's call count.
fn list(
    make: impl FnOnce(Rc<Cell<usize>>) -> ListView + 'static,
) -> (WidgetTester, ScrollController, Rc<Cell<usize>>) {
    let out: Out = Rc::default();
    let built: Rc<Cell<usize>> = Rc::default();
    let make = Cell::new(Some(make));
    let mut tester = {
        let (out, built) = (out.clone(), built.clone());
        WidgetTester::with_size(300.0, 200.0, move || {
            let controller = ScrollController::new();
            out.set(Some(controller));
            let make = make.take().expect("built once");
            Keyed::new("l", make(built).controller(controller))
        })
    };
    tester.pump();
    (tester, out.get().expect("built"), built)
}

fn items(count: usize, built: Rc<Cell<usize>>) -> ListView {
    ListView::builder(count, 40.0, move |i| {
        built.set(built.get() + 1);
        Text::new(format!("Item {i}"))
    })
}

fn item_elements(tester: &WidgetTester) -> usize {
    tester.find_all(&Finder::render::<RenderParagraph>()).len()
}

#[test]
fn widgets_list_01_structure_and_placement() {
    let (mut tester, c, _) = list(|built| items(50, built));
    assert_eq!(
        tester.rect(tester.find(&Finder::key("l"))).size(),
        Size::new(300.0, 200.0)
    );
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 1800.0);
    for i in 0..5 {
        let rect = tester.rect(tester.find(&Finder::text(format!("Item {i}"))));
        assert_eq!(
            (rect.top, rect.size()),
            (i as f32 * 40.0, Size::new(300.0, 40.0)),
            "{i}"
        );
    }
    tester.enter(|| c.jump_to(100.0));
    tester.pump();
    tester.pump();
    assert_eq!(tester.rect(tester.find(&Finder::text("Item 3"))).top, 20.0);
    assert!(tester.find_all(&Finder::text("Item 0")).is_empty());
}

#[test]
fn widgets_list_02_virtualization() {
    let (mut tester, c, built) = list(|built| items(100_000, built));
    assert_eq!(item_elements(&tester), 6);
    assert_eq!(built.get(), 6);
    let third = tester.find(&Finder::text("Item 3"));
    // Wheel steps animate (WIDGETS-SCROLL-07): each is waited out.
    let step = std::time::Duration::from_millis(150);
    tester.scroll(&Finder::key("l"), Vec2::new(0.0, 10.0));
    tester.advance(step);
    assert_eq!(tester.find(&Finder::text("Item 3")), third);
    assert!(item_elements(&tester) <= 8);
    tester.scroll(&Finder::key("l"), Vec2::new(0.0, 400.0));
    tester.advance(step);
    assert!(tester.find_all(&Finder::text("Item 3")).is_empty());
    assert!(item_elements(&tester) <= 8);
    tester.find(&Finder::text("Item 10"));
    // Items are built once per appearance: 6 at first, then the ones scrolled in.
    assert!(built.get() <= 6 + 1 + 11, "{}", built.get());
    tester.enter(|| c.jump_to(f32::MAX));
    tester.pump();
    tester.pump();
    tester.find(&Finder::text("Item 99999"));
    assert!(item_elements(&tester) <= 8);
    // Scrolling across the list keeps the element count bounded.
    for _ in 0..200 {
        tester.scroll(&Finder::key("l"), Vec2::new(0.0, -25.0));
        tester.advance(step);
        assert!(item_elements(&tester) <= 8);
    }
}

#[test]
fn widgets_list_03_reactive_count_and_horizontal() {
    let out: Rc<Cell<Option<Signal<usize>>>> = Rc::default();
    let (mut tester, c, _) = {
        let out = out.clone();
        list(move |built| {
            let count = signal(5usize);
            out.set(Some(count));
            ListView::builder(count, 40.0, move |i| {
                built.set(built.get() + 1);
                Text::new(format!("Item {i}"))
            })
        })
    };
    let count = out.get().expect("built");
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 0.0);
    assert_eq!(item_elements(&tester), 5);
    tester.enter(|| count.set(10));
    tester.pump();
    tester.pump();
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 200.0);
    assert_eq!(item_elements(&tester), 6);
    tester.enter(|| count.set(0));
    tester.pump();
    tester.pump();
    assert_eq!(
        (
            item_elements(&tester),
            tester.enter(|| c.max_scroll_extent())
        ),
        (0, 0.0)
    );
    let (tester, c, _) = list(|built| items(50, built).horizontal());
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 1700.0);
    assert_eq!(tester.rect(tester.find(&Finder::text("Item 2"))).left, 80.0);
    assert_eq!(
        tester.rect(tester.find(&Finder::text("Item 2"))).size(),
        Size::new(40.0, 200.0)
    );
}

fn _is_view<V: View>(_: V) {}
