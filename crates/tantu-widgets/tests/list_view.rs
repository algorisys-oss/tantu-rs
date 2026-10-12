//! Tests for `docs/specs/widgets/list-view.md`, rules WIDGETS-LIST-01..06 (04..06 in list-view-advanced.md).

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Point, Size, Vec2};
use tantu_view::layout::RenderParagraph;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{
    Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind, View,
};
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

fn top_of(tester: &WidgetTester, text: &str) -> f32 {
    tester.rect(tester.find(&Finder::text(text))).top
}

#[test]
fn widgets_list_04_variable_extents() {
    let (tester, c, _) = list(|built| {
        ListView::builder_with_extents(
            3,
            |i| [10.0, 20.0, 30.0][i],
            move |i| {
                built.set(built.get() + 1);
                Text::new(format!("Item {i}"))
            },
        )
    });
    assert_eq!(
        tester.enter(|| c.max_scroll_extent()),
        0.0,
        "60 px fit in 200"
    );
    let item1 = tester.rect(tester.find(&Finder::text("Item 1")));
    assert_eq!((item1.top, item1.height()), (10.0, 20.0));
    assert_eq!(top_of(&tester, "Item 2"), 30.0);
    // A long list with extents alternating 20 and 60 keeps a viewport's worth plus buffer.
    let calls: Rc<Cell<usize>> = Rc::default();
    let count_out: Rc<Cell<Option<Signal<usize>>>> = Rc::default();
    let (mut tester, c, _) = {
        let (calls, count_out) = (calls.clone(), count_out.clone());
        list(move |built| {
            let count = signal(100_000usize);
            count_out.set(Some(count));
            ListView::builder_with_extents(
                count,
                move |i| {
                    calls.set(calls.get() + 1);
                    if i % 2 == 0 { 20.0 } else { 60.0 }
                },
                move |i| {
                    built.set(built.get() + 1);
                    Text::new(format!("Item {i}"))
                },
            )
        })
    };
    assert_eq!(calls.get(), 100_000, "extent_of once per index");
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 4_000_000.0 - 200.0);
    assert!(item_elements(&tester) <= 14);
    tester.enter(|| c.jump_to(2_000_010.0));
    tester.pump();
    tester.pump();
    assert!(item_elements(&tester) <= 14);
    // Item 50 000 starts at 50 000 · 40 = 2 000 000, 10 px above the view.
    assert_eq!(top_of(&tester, "Item 50000"), -10.0);
    let count = count_out.get().expect("built");
    tester.enter(|| count.set(10));
    tester.pump();
    assert_eq!(
        calls.get(),
        100_010,
        "re-run for every index of the new count"
    );
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 200.0);
}

#[test]
fn widgets_list_05_separated() {
    let (tester, c, _) = list(|built| {
        ListView::separated(
            3,
            40.0,
            move |i| {
                built.set(built.get() + 1);
                Text::new(format!("Item {i}"))
            },
            1.0,
            |i| Text::new(format!("Sep {i}")),
        )
    });
    assert_eq!(top_of(&tester, "Item 0"), 0.0);
    let sep = tester.rect(tester.find(&Finder::text("Sep 0")));
    assert_eq!((sep.top, sep.height()), (40.0, 1.0));
    assert_eq!(top_of(&tester, "Item 1"), 41.0);
    assert_eq!(top_of(&tester, "Sep 1"), 81.0);
    assert_eq!(top_of(&tester, "Item 2"), 82.0);
    assert!(
        tester.find_all(&Finder::text("Sep 2")).is_empty(),
        "none after the last"
    );
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 0.0, "122 px fit");
    // A taller list: the content is 3 · 40 + 2 = 122 per three; 300 items → 12 299 px.
    let (tester, c, _) = list(|_| {
        ListView::separated(
            300,
            40.0,
            |i| Text::new(format!("Item {i}")),
            1.0,
            |_| Text::new("-"),
        )
    });
    assert_eq!(
        tester.enter(|| c.max_scroll_extent()),
        300.0 * 41.0 - 1.0 - 200.0
    );
    let (tester, _, _) = list(|_| {
        ListView::separated(
            1,
            40.0,
            |i| Text::new(format!("Item {i}")),
            1.0,
            |i| Text::new(format!("Sep {i}")),
        )
    });
    assert!(tester.find_all(&Finder::text("Sep 0")).is_empty());
    let (tester, _, _) = list(|_| {
        ListView::separated(
            0,
            40.0,
            |i| Text::new(format!("Item {i}")),
            1.0,
            |i| Text::new(format!("Sep {i}")),
        )
    });
    assert_eq!(item_elements(&tester), 0);
}

#[test]
fn widgets_list_06_selection_and_keys() {
    let sel_out: Rc<Cell<Option<Signal<Option<usize>>>>> = Rc::default();
    let activated: Rc<Cell<Option<usize>>> = Rc::default();
    let (mut tester, c, _) = {
        let (sel_out, activated) = (sel_out.clone(), activated.clone());
        list(move |_| {
            let selection = signal(None);
            sel_out.set(Some(selection));
            ListView::builder(50, 40.0, |i| Text::new(format!("Item {i}")))
                .selection(selection)
                .on_activate(move |i| activated.set(Some(i)))
        })
    };
    let selection = sel_out.get().expect("built");
    let selected = |tester: &WidgetTester| tester.enter(|| selection.get());
    let offset = |tester: &WidgetTester| tester.enter(|| c.offset());
    let press = |tester: &mut WidgetTester, key: NamedKey| {
        tester.press_key(LogicalKey::Named(key), Modifiers::default());
    };
    tester.tap(&Finder::text("Item 3"));
    assert_eq!(selected(&tester), Some(3));
    press(&mut tester, NamedKey::ArrowDown);
    assert_eq!(selected(&tester), Some(4));
    press(&mut tester, NamedKey::ArrowUp);
    assert_eq!(selected(&tester), Some(3));
    assert_eq!(offset(&tester), 0.0, "still in view");
    press(&mut tester, NamedKey::PageDown);
    assert_eq!(selected(&tester), Some(8));
    assert_eq!(
        offset(&tester),
        9.0 * 40.0 - 200.0,
        "scrolled the least to show item 8"
    );
    press(&mut tester, NamedKey::End);
    assert_eq!(selected(&tester), Some(49));
    assert_eq!(offset(&tester), 1800.0);
    press(&mut tester, NamedKey::ArrowDown);
    assert_eq!(selected(&tester), Some(49), "clamped");
    press(&mut tester, NamedKey::Home);
    assert_eq!(selected(&tester), Some(0));
    assert_eq!(offset(&tester), 0.0);
    press(&mut tester, NamedKey::PageUp);
    assert_eq!(selected(&tester), Some(0));
    press(&mut tester, NamedKey::Enter);
    assert_eq!(activated.get(), Some(0));
    // A double-click activates the item.
    activated.set(None);
    let center = tester.rect(tester.find(&Finder::text("Item 2"))).center();
    for _ in 0..2 {
        for kind in [
            PointerKind::Down(PointerButton::Primary),
            PointerKind::Up(PointerButton::Primary),
        ] {
            tester.pointer(PointerEvent {
                kind,
                position: center,
                modifiers: Default::default(),
            });
        }
        tester.advance(std::time::Duration::from_millis(50));
    }
    assert_eq!(selected(&tester), Some(2));
    assert_eq!(activated.get(), Some(2));
    // From None, ArrowDown selects the first item; ArrowUp the last.
    tester.enter(|| selection.set(None));
    tester.pump();
    press(&mut tester, NamedKey::ArrowDown);
    assert_eq!(selected(&tester), Some(0));
    tester.enter(|| selection.set(None));
    tester.pump();
    press(&mut tester, NamedKey::ArrowUp);
    assert_eq!(selected(&tester), Some(49));
    assert_eq!(offset(&tester), 1800.0);
    // Without a selection, keys scroll as a scroll view.
    let (mut tester, c, _) = list(|built| items(50, built));
    tester.tap(&Finder::text("Item 1"));
    tester.press_key(LogicalKey::Named(NamedKey::ArrowDown), Modifiers::default());
    assert_eq!(tester.enter(|| c.offset()), 40.0);
    let _ = Point::ZERO;
}
