//! Pointer modifier propagation and modified taps for DataGrid selection.
use std::cell::RefCell;
use std::rc::Rc;
use tantu_test::{Finder, WidgetTester};
use tantu_view::core::Point;
use tantu_view::layout::RenderConstrainedBox;
use tantu_view::reactive::signal;
use tantu_view::{
    AnyView, BuildCx, ElementId, Handled, Keyed, Modifiers, Phase, PointerButton, PointerEvent,
    PointerKind, View,
};
use tantu_widgets::Text;

type Log = Rc<RefCell<Vec<(Phase, PointerKind, Modifiers)>>>;
struct Probe(Log);
impl View for Probe {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let count = signal(0);
        let id = cx.render(
            RenderConstrainedBox::expand(),
            [AnyView::new(Text::new(move || {
                format!("Downs: {}", count.get())
            }))],
        );
        for phase in [Phase::Capture, Phase::Bubble] {
            let log = self.0.clone();
            cx.on_pointer(id, phase, move |p| {
                log.borrow_mut()
                    .push((phase, p.event.kind, p.event.modifiers));
                if phase == Phase::Bubble
                    && p.event.kind == PointerKind::Down(PointerButton::Primary)
                {
                    count.update(|c| *c += 1);
                }
                Handled::Continue
            });
        }
        id
    }
}
#[test]
fn view_event_08_modifier_snapshot() {
    let log: Log = Rc::default();
    let capture = log.clone();
    let mut t = WidgetTester::new(move || Probe(capture));
    let modifiers = Modifiers {
        shift: true,
        alt: true,
        ..Default::default()
    };
    for (kind, pos) in [
        (
            PointerKind::Down(PointerButton::Primary),
            Point::new(50.0, 50.0),
        ),
        (PointerKind::Move, Point::new(900.0, 900.0)),
    ] {
        t.pointer(PointerEvent {
            kind,
            position: pos,
            modifiers,
        });
    }
    let log = log.borrow();
    assert!(log.iter().any(|(phase, kind, m)| *phase == Phase::Capture
        && *kind == PointerKind::Move
        && *m == modifiers));
    assert!(log.iter().any(|(phase, kind, m)| *phase == Phase::Bubble
        && *kind == PointerKind::Down(PointerButton::Primary)
        && *m == modifiers));
}
#[test]
fn test_wt_16_modified_tap_pumps() {
    let log: Log = Rc::default();
    let capture = log.clone();
    let mut t = WidgetTester::new(move || Keyed::new("p", Probe(capture)));
    let modifiers = Modifiers {
        shift: true,
        ..Default::default()
    };
    t.tap_with_modifiers(&Finder::key("p"), modifiers);
    t.find(&Finder::text("Downs: 1"));
    for kind in [
        PointerKind::Down(PointerButton::Primary),
        PointerKind::Up(PointerButton::Primary),
    ] {
        assert!(
            log.borrow()
                .iter()
                .any(|(_, k, m)| *k == kind && *m == modifiers)
        );
    }
    log.borrow_mut().clear();
    t.tap(&Finder::key("p"));
    assert!(
        log.borrow()
            .iter()
            .all(|(_, _, m)| *m == Modifiers::default())
    );
}
