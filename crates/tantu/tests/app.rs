//! Tests for `docs/specs/facade/app.md`, rules FACADE-APP-01..09.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu::core::{Point, Size, Vec2};
use tantu::layout::RenderConstrainedBox;
use tantu::platform::{
    ButtonState, FakeLog, FakePlatform, PhysicalSize, PlatformContext, PointerButton, PointerId,
    ScrollDelta, WindowEvent, WindowId,
};
use tantu::prelude::*;
use tantu::scene::{Command, RenderError, RenderReport, Renderer, Resources, Scene};
use tantu::view::{BuildCx, ElementId, Handled, Phase, PointerKind};
use tantu::{AppHandler, Error};
use tantu_render_headless::{HeadlessRenderer, RecordedFrame};

const FONT: &[u8] = include_bytes!("../../tantu-text/tests/fonts/LiberationSans-Regular.ttf");

type Shared = Rc<RefCell<HeadlessRenderer>>;

/// A renderer the test keeps a handle to.
struct Recorder(Shared);

impl Renderer for Recorder {
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.0.borrow_mut().resize(width, height, scale_factor);
    }

    fn render(
        &mut self,
        scene: &Scene,
        resources: &Resources,
    ) -> std::result::Result<RenderReport, RenderError> {
        self.0.borrow_mut().render(scene, resources)
    }
}

/// The renderers created so far, in window order.
#[derive(Clone, Default)]
struct Recorders(Rc<RefCell<Vec<Shared>>>);

impl Recorders {
    /// A factory that records each renderer; `setup` can prepare it (e.g. inject a failure).
    fn factory(
        &self,
        setup: impl Fn(&mut HeadlessRenderer) + 'static,
    ) -> impl FnMut(&mut dyn PlatformContext, WindowId) -> tantu::Result<Box<dyn Renderer>> + 'static
    {
        let list = self.0.clone();
        move |_, _| {
            let mut renderer = HeadlessRenderer::new(1, 1);
            setup(&mut renderer);
            let shared = Rc::new(RefCell::new(renderer));
            list.borrow_mut().push(shared.clone());
            Ok(Box::new(Recorder(shared)) as Box<dyn Renderer>)
        }
    }

    fn get(&self, i: usize) -> Shared {
        self.0.borrow()[i].clone()
    }

    fn len(&self) -> usize {
        self.0.borrow().len()
    }
}

fn run(app: App, platform: FakePlatform, recorders: &Recorders) -> (FakeLog, AppHandler) {
    let mut handler = app.handler(recorders.factory(|_| {}));
    let log = platform.run_logged(&mut handler);
    (log, handler)
}

fn with_font(app: App) -> App {
    app.without_system_fonts().font(FONT.to_vec())
}

/// Glyph counts of the glyph runs in a frame.
fn glyph_counts(frame: &RecordedFrame) -> Vec<usize> {
    frame
        .scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(frame.scene.glyphs(run).len()),
            _ => None,
        })
        .collect()
}

/// The glyph ids of all glyph runs in a frame.
fn glyph_ids(frame: &RecordedFrame) -> Vec<u32> {
    frame
        .scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(frame.scene.glyphs(run).iter().map(|g| g.id)),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn facade_app_01_windows_trees_and_renderers() {
    let recorders = Recorders::default();
    let app = App::new()
        .window(Window::new("A").size(200.0, 100.0), || SizedBox::shrink())
        .window(Window::new("B").size(300.0, 150.0), || SizedBox::shrink());
    let (log, handler) = run(app, FakePlatform::new().scale_factor(2.0), &recorders);
    let titles: Vec<&str> = log
        .windows
        .iter()
        .map(|w| w.attributes.title.as_str())
        .collect();
    assert_eq!(titles, ["A", "B"]);
    assert_eq!(recorders.len(), 2);
    assert_eq!(recorders.get(0).borrow().size(), (400, 200));
    assert_eq!(recorders.get(1).borrow().size(), (600, 300));
    assert_eq!(recorders.get(1).borrow().scale_factor(), 2.0);
    for i in 0..2 {
        assert_eq!(recorders.get(i).borrow().frame_count(), 1);
    }
    assert!(handler.finish().is_ok());

    // No windows: exit at once.
    let (log, handler) = run(App::new(), FakePlatform::new(), &Recorders::default());
    assert!(log.exited);
    assert!(handler.finish().is_ok());

    // A failing renderer ends the run with its error.
    let mut handler = App::new()
        .window(Window::new("A"), || SizedBox::shrink())
        .handler(|_, _| Err(Error::Renderer("no GPU".into())));
    let log = FakePlatform::new().run_logged(&mut handler);
    assert!(log.exited);
    assert!(matches!(handler.finish(), Err(Error::Renderer(_))));
}

#[test]
fn facade_app_02_first_frame_shows_the_content() {
    let recorders = Recorders::default();
    let app = with_font(App::new()).window(Window::new("A").size(200.0, 100.0), || Text::new("Hi"));
    run(app, FakePlatform::new().scale_factor(2.0), &recorders);
    let renderer = recorders.get(0);
    let renderer = renderer.borrow();
    let frame = renderer.last_frame().expect("a frame");
    assert_eq!(frame.scene.size(), Size::new(200.0, 100.0));
    assert_eq!(glyph_counts(frame), [2]);
}

/// A counter like the AGENTS.md snippet, in a window of its own.
fn counter_app() -> App {
    with_font(App::new()).window(Window::new("Counter").size(200.0, 100.0), || {
        let count = signal(0);
        Column::new()
            .child(Text::new(move || format!("Count: {}", count.get())))
            .child(Button::new("Increment").on_press(move || count.update(|c| *c += 1)))
    })
}

fn button(state: ButtonState, x: f32, y: f32) -> WindowEvent {
    WindowEvent::PointerButton {
        pointer: PointerId::Mouse,
        button: PointerButton::Primary,
        state,
        position: Point::new(x, y),
    }
}

/// Where the counter's button is: under the text, centered (found from the first frame).
fn button_center(frame: &RecordedFrame) -> Point {
    // The button fills a rounded rect; its element's transform holds the offset. Take the
    // first fill, in the button's own coordinates, plus the frame's translation for it.
    let mut offset = Vec2::ZERO;
    for entry in frame.scene.entries() {
        match &entry.command {
            Command::PushTransform(t) => {
                let [_, _, _, _, e, f] = t.coeffs();
                offset = offset + Vec2::new(e, f);
            }
            Command::Fill { shape, .. } => {
                let r = shape.rect;
                return Point::new(
                    offset.x + (r.left + r.right) / 2.0,
                    offset.y + (r.top + r.bottom) / 2.0,
                );
            }
            _ => {}
        }
    }
    panic!("no button fill in the frame");
}

#[test]
fn facade_app_03_changes_request_one_frame() {
    // Find the button from a first run, then script a click on it.
    let probe = Recorders::default();
    run(counter_app(), FakePlatform::new(), &probe);
    let center = button_center(probe.get(0).borrow().last_frame().expect("a frame"));

    let recorders = Recorders::default();
    let platform = FakePlatform::new()
        .event(0, button(ButtonState::Pressed, center.x, center.y))
        .event(0, button(ButtonState::Released, center.x, center.y))
        .step(tantu::platform::FakeStep::Idle);
    let (log, _) = run(counter_app(), platform, &recorders);
    // The first frame, one for the press (pressed look), one for the release (new count);
    // the idle step draws nothing.
    assert_eq!(log.windows[0].redraws, 3);
    let renderer = recorders.get(0);
    let frames = renderer.borrow_mut().take_frames();
    assert_eq!(frames.len(), 3);
    let (first, last) = (glyph_ids(&frames[0]), glyph_ids(&frames[2]));
    assert_ne!(first, last, "the count changed");
}

#[test]
fn facade_app_04_resize_and_scale() {
    let recorders = Recorders::default();
    let platform = FakePlatform::new()
        .event(0, WindowEvent::Resized(PhysicalSize::new(600, 300)))
        .event(0, WindowEvent::ScaleFactorChanged(2.0));
    let app = App::new().window(Window::new("A").size(200.0, 100.0), || SizedBox::shrink());
    run(app, platform, &recorders);
    let renderer = recorders.get(0);
    let renderer = renderer.borrow();
    let sizes: Vec<(u32, u32, f32, Size)> = renderer
        .frames()
        .iter()
        .map(|f| (f.width, f.height, f.scale_factor, f.scene.size()))
        .collect();
    assert_eq!(
        sizes,
        [
            (200, 100, 1.0, Size::new(200.0, 100.0)),
            (600, 300, 1.0, Size::new(600.0, 300.0)),
            (600, 300, 2.0, Size::new(300.0, 150.0)),
        ]
    );
}

/// A box filling the window that logs the pointer kinds it receives.
struct Logger(Rc<RefCell<Vec<PointerKind>>>);

impl View for Logger {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::expand(), []);
        let log = self.0;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            log.borrow_mut().push(p.event.kind);
            Handled::Continue
        });
        id
    }
}

#[test]
fn facade_app_05_pointer_events_are_converted() {
    let kinds: Rc<RefCell<Vec<PointerKind>>> = Rc::default();
    let k = kinds.clone();
    let app = App::new().window(Window::new("A").size(200.0, 100.0), move || Logger(k));
    let platform = FakePlatform::new()
        .event(
            0,
            WindowEvent::PointerMoved {
                pointer: PointerId::Mouse,
                position: Point::new(5.0, 5.0),
            },
        )
        .event(0, button(ButtonState::Pressed, 5.0, 5.0))
        .event(
            0,
            WindowEvent::PointerButton {
                pointer: PointerId::Mouse,
                button: PointerButton::Back,
                state: ButtonState::Released,
                position: Point::new(5.0, 5.0),
            },
        )
        .event(
            0,
            WindowEvent::Wheel {
                pointer: PointerId::Mouse,
                delta: ScrollDelta::Lines { x: 0.0, y: 2.0 },
                position: Point::new(5.0, 5.0),
            },
        )
        .event(
            0,
            WindowEvent::Wheel {
                pointer: PointerId::Mouse,
                delta: ScrollDelta::Pixels(Vec2::new(1.0, 2.0)),
                position: Point::new(5.0, 5.0),
            },
        )
        .event(0, button(ButtonState::Released, 5.0, 5.0))
        .event(
            0,
            WindowEvent::PointerLeft {
                pointer: PointerId::Mouse,
            },
        );
    run(app, platform, &Recorders::default());
    use tantu::view::PointerButton as B;
    assert_eq!(
        *kinds.borrow(),
        [
            PointerKind::Enter,
            PointerKind::Move,
            PointerKind::Down(B::Primary),
            PointerKind::Up(B::Other(3)),
            PointerKind::Scroll(Vec2::new(0.0, 80.0)),
            PointerKind::Scroll(Vec2::new(1.0, 2.0)),
            PointerKind::Up(B::Primary),
            PointerKind::Leave,
        ]
    );
}

#[test]
fn facade_app_06_closing() {
    let disposed = Rc::new(Cell::new(0));
    let d = disposed.clone();
    let d2 = disposed.clone();
    let app = App::new()
        .window(Window::new("A"), move || {
            tantu::reactive::on_cleanup(move || d.set(d.get() + 1));
            SizedBox::shrink()
        })
        .window(Window::new("B"), move || {
            tantu::reactive::on_cleanup(move || d2.set(d2.get() + 10));
            SizedBox::shrink()
        });
    let platform = FakePlatform::new()
        .event(0, WindowEvent::CloseRequested)
        .step(tantu::platform::FakeStep::Idle);
    let recorders = Recorders::default();
    let mut handler = app.handler(recorders.factory(|_| {}));
    let log = platform.run_logged(&mut handler);
    assert!(log.windows[0].closed && !log.windows[1].closed);
    assert!(!log.exited);
    assert_eq!(disposed.get(), 1);
    // Closing the last one exits.
    let disposed = Rc::new(Cell::new(0));
    let d = disposed.clone();
    let app = App::new().window(Window::new("A"), move || {
        tantu::reactive::on_cleanup(move || d.set(d.get() + 1));
        SizedBox::shrink()
    });
    let mut handler = app.handler(Recorders::default().factory(|_| {}));
    let log = FakePlatform::new()
        .event(0, WindowEvent::CloseRequested)
        .run_logged(&mut handler);
    assert!(log.windows[0].closed);
    assert!(log.exited);
    assert_eq!(disposed.get(), 1);
    assert!(handler.finish().is_ok());
}

#[test]
fn facade_app_07_render_errors() {
    // A lost target is retried.
    let recorders = Recorders::default();
    let app = App::new().window(Window::new("A"), || SizedBox::shrink());
    let mut handler =
        app.handler(recorders.factory(|r| r.fail_next_render(RenderError::TargetLost)));
    let log = FakePlatform::new().run_logged(&mut handler);
    assert_eq!(log.windows[0].redraws, 2);
    assert_eq!(recorders.get(0).borrow().frames().len(), 1);
    assert!(handler.finish().is_ok());

    // Any other error ends the run.
    let app = App::new().window(Window::new("A"), || SizedBox::shrink());
    let mut handler =
        app.handler(Recorders::default().factory(|r| r.fail_next_render(RenderError::OutOfMemory)));
    let log = FakePlatform::new().run_logged(&mut handler);
    assert!(log.exited);
    assert!(matches!(
        handler.finish(),
        Err(Error::Render(RenderError::OutOfMemory))
    ));
}

#[test]
#[cfg(not(all(feature = "winit", feature = "wgpu")))]
fn facade_app_08_run_needs_winit_and_wgpu() {
    let result = App::new()
        .window(Window::new("A"), || SizedBox::shrink())
        .run();
    assert!(matches!(result, Err(Error::Unsupported(_))));
}

#[test]
fn facade_app_09_first_font_is_the_default_family() {
    let recorders = Recorders::default();
    let app = with_font(App::new()).window(Window::new("A"), || Text::new("abc"));
    run(app, FakePlatform::new(), &recorders);
    let frame = recorders
        .get(0)
        .borrow()
        .last_frame()
        .cloned()
        .expect("a frame");
    assert_eq!(glyph_counts(&frame), [3]);
    // No fonts at all: nothing to draw text with.
    let recorders = Recorders::default();
    let app = App::new()
        .without_system_fonts()
        .window(Window::new("A"), || Text::new("abc"));
    run(app, FakePlatform::new(), &recorders);
    let frame = recorders
        .get(0)
        .borrow()
        .last_frame()
        .cloned()
        .expect("a frame");
    assert!(glyph_counts(&frame).iter().all(|n| *n == 0));
}
