//! Runner snapshots modifiers separately for each window.
use std::cell::RefCell;
use std::rc::Rc;
use tantu::core::Point;
use tantu::layout::RenderConstrainedBox;
use tantu::platform::{ButtonState, Modifiers, PointerButton, PointerId, WindowEvent};
use tantu::view::{BuildCx, ElementId, Handled, Phase, PointerKind, View};
use tantu::{App, Window};
use tantu_platform::FakePlatform;
use tantu_render_headless::HeadlessRenderer;
struct Probe(Rc<RefCell<Vec<bool>>>);
impl View for Probe {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::expand(), []);
        let log = self.0;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if matches!(p.event.kind, PointerKind::Down(_)) {
                log.borrow_mut().push(p.event.modifiers.shift);
            }
            Handled::Continue
        });
        id
    }
}
fn press() -> WindowEvent {
    WindowEvent::PointerButton {
        pointer: PointerId::Mouse,
        button: PointerButton::Primary,
        state: ButtonState::Pressed,
        position: Point::new(5.0, 5.0),
    }
}
#[test]
fn facade_app_16_pointer_modifiers_per_window() {
    let a = Rc::new(RefCell::new(Vec::new()));
    let b = Rc::new(RefCell::new(Vec::new()));
    let (aa, bb) = (a.clone(), b.clone());
    let app = App::new()
        .window(Window::new("A"), move || Probe(aa))
        .window(Window::new("B"), move || Probe(bb));
    let platform = FakePlatform::new()
        .event(
            0,
            WindowEvent::ModifiersChanged(Modifiers {
                shift: true,
                ..Default::default()
            }),
        )
        .event(0, press())
        .event(1, press())
        .event(1, WindowEvent::ModifiersChanged(Modifiers::default()))
        .event(0, press());
    app.run_with(platform, |_, _| {
        Ok(Box::new(HeadlessRenderer::new(800, 600)))
    })
    .expect("run");
    assert_eq!(*a.borrow(), vec![true, true]);
    assert_eq!(*b.borrow(), vec![false]);
}
