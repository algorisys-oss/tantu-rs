//! Tests for `docs/specs/facade/windows.md`, rules FACADE-WIN-01..07.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use tantu::core::Point;
use tantu::layout::RenderConstrainedBox;
use tantu::platform::{
    ButtonState, FakeLog, FakePlatform, FakeStep, PlatformContext, PointerButton, PointerId,
    WindowEvent, WindowId,
};
use tantu::prelude::*;
use tantu::reactive::{ticker, timer};
use tantu::scene::{Command, RenderError, RenderReport, Renderer, Resources, Scene};
use tantu::view::{BuildCx, ElementId, Handled, Phase, PointerKind};
use tantu::{AppHandler, WindowHandle, Windows};
use tantu_render_headless::HeadlessRenderer;

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
    fn factory(
        &self,
    ) -> impl FnMut(&mut dyn PlatformContext, WindowId) -> tantu::Result<Box<dyn Renderer>> + 'static
    {
        let list = self.0.clone();
        move |_, _| {
            let shared = Rc::new(RefCell::new(HeadlessRenderer::new(1, 1)));
            list.borrow_mut().push(shared.clone());
            Ok(Box::new(Recorder(shared)) as Box<dyn Renderer>)
        }
    }

    /// Glyph counts of the glyph runs in window `i`'s last frame.
    fn glyphs(&self, i: usize) -> Vec<usize> {
        let renderer = self.0.borrow()[i].clone();
        let renderer = renderer.borrow();
        let frame = renderer.last_frame().expect("a frame");
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
}

fn run(app: App, platform: FakePlatform, recorders: &Recorders) -> (FakeLog, AppHandler) {
    let mut handler = app
        .without_system_fonts()
        .font(FONT.to_vec())
        .handler(recorders.factory());
    let log = platform.run_logged(&mut handler);
    (log, handler)
}

fn press(x: f32, y: f32) -> [WindowEvent; 2] {
    [ButtonState::Pressed, ButtonState::Released].map(|state| WindowEvent::PointerButton {
        pointer: PointerId::Mouse,
        button: PointerButton::Primary,
        state,
        position: Point::new(x, y),
    })
}

/// A box filling the window that calls `on_press` on a primary press, over `label`.
struct Pad {
    on_press: Rc<dyn Fn()>,
    label: Rc<dyn Fn() -> String>,
}

impl View for Pad {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let label = self.label;
        let id = cx.render(
            RenderConstrainedBox::expand(),
            [AnyView::new(Text::new(move || label()))],
        );
        let on_press = self.on_press;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if matches!(p.event.kind, PointerKind::Down(_)) {
                on_press();
            }
            Handled::Stop
        });
        id
    }
}

fn pad(on_press: impl Fn() + 'static, label: impl Fn() -> String + 'static) -> Pad {
    Pad {
        on_press: Rc::new(on_press),
        label: Rc::new(label),
    }
}

#[test]
fn facade_win_01_windows_share_one_runtime() {
    let recorders = Recorders::default();
    let made: Rc<Cell<Option<Signal<u32>>>> = Rc::default();
    let app = {
        let (m1, m2) = (made.clone(), made.clone());
        App::new()
            .window(Window::new("A").size(200.0, 100.0), move || {
                let s = signal(1u32);
                m1.set(Some(s));
                pad(
                    move || s.update(|n| *n += 1),
                    move || "a".repeat(s.get() as usize),
                )
            })
            .window(Window::new("B").size(200.0, 100.0), move || {
                let s = m2.get().expect("A built first");
                Text::new(move || "b".repeat(s.get() as usize))
            })
    };
    let [down, up] = press(10.0, 10.0);
    let platform = FakePlatform::new().event(0, down).event(0, up);
    let (log, _) = run(app, platform, &recorders);
    assert_eq!(log.windows.len(), 2);
    // A's press wrote A's signal; B, reading it, redrew with 2 glyphs.
    assert_eq!(recorders.glyphs(0), [2]);
    assert_eq!(recorders.glyphs(1), [2]);
}

#[derive(Clone)]
struct Shared2 {
    count: Signal<u32>,
}

#[test]
fn facade_win_02_app_provided_state() {
    let recorders = Recorders::default();
    let inits = Rc::new(Cell::new(0));
    let reads: Rc<RefCell<Vec<Option<u32>>>> = Rc::default();
    let reader = |reads: Rc<RefCell<Vec<Option<u32>>>>| {
        move || {
            Builder::new(move |cx: &mut BuildCx<'_>| {
                let state = cx.inherit::<Shared2>();
                reads.borrow_mut().push(cx.inherit::<u32>());
                let count = state.expect("provided").count;
                pad(
                    move || count.update(|n| *n += 1),
                    move || "x".repeat(count.get() as usize),
                )
            })
        }
    };
    let app = {
        let inits = inits.clone();
        App::new()
            .provide(move || {
                inits.set(inits.get() + 1);
                Shared2 { count: signal(1) }
            })
            .provide(|| 1u32)
            .provide(|| 2u32)
            .window(Window::new("A").size(200.0, 100.0), reader(reads.clone()))
            .window(Window::new("B").size(200.0, 100.0), reader(reads.clone()))
    };
    let [down, up] = press(10.0, 10.0);
    let platform = FakePlatform::new()
        // Close A; the app's state outlives it.
        .event(0, WindowEvent::CloseRequested)
        .event(1, down)
        .event(1, up);
    let (log, _) = run(app, platform, &recorders);
    assert_eq!(inits.get(), 1, "init runs once");
    assert_eq!(
        *reads.borrow(),
        [Some(2), Some(2)],
        "the later provide wins"
    );
    assert!(log.windows[0].closed);
    assert_eq!(
        recorders.glyphs(1),
        [2],
        "B wrote the app's signal after A closed"
    );
}

/// The window opened by `opener`'s content, recorded for the test.
type Slot = Rc<Cell<Option<WindowHandle>>>;

#[test]
fn facade_win_03_open_from_a_view() {
    let recorders = Recorders::default();
    let opened: Slot = Rc::default();
    let counts: Rc<RefCell<Vec<usize>>> = Rc::default();
    let app = {
        let (opened, counts) = (opened.clone(), counts.clone());
        App::new().window(Window::new("A").size(200.0, 100.0), move || {
            Builder::new(move |cx: &mut BuildCx<'_>| {
                let windows = Windows::of(cx).expect("opened by the runner");
                let (opened, counts) = (opened.clone(), counts.clone());
                effect(move || counts.borrow_mut().push(windows.count()));
                timer(Duration::from_millis(10), move || {
                    let handle = windows.open(Window::new("B").size(200.0, 100.0), || {
                        // The new window's content sees the app's handle too.
                        Builder::new(|cx: &mut BuildCx<'_>| {
                            let n = Windows::of(cx).map_or(0, |w| w.count());
                            Text::new(format!("hello{n}"))
                        })
                    });
                    opened.set(Some(handle));
                });
                Text::new("a")
            })
        })
    };
    let platform = FakePlatform::new()
        .step(FakeStep::Idle)
        .step(FakeStep::Advance(Duration::from_millis(15)));
    let (log, _) = run(app, platform, &recorders);
    let titles: Vec<&str> = log
        .windows
        .iter()
        .map(|w| w.attributes.title.as_str())
        .collect();
    assert_eq!(titles, ["A", "B"]);
    assert!(opened.get().is_some());
    assert_eq!(*counts.borrow(), [1, 2]);
    assert_eq!(recorders.glyphs(1), [6], "B drew \"hello2\"");
}

#[test]
fn facade_win_04_closing() {
    let recorders = Recorders::default();
    let asked = Rc::new(Cell::new(0));
    let states: Rc<RefCell<Vec<bool>>> = Rc::default();
    let app = {
        let (asked, states) = (asked.clone(), states.clone());
        App::new()
            .window(Window::new("A").size(200.0, 100.0), move || {
                Builder::new(move |cx: &mut BuildCx<'_>| {
                    let windows = Windows::of(cx).expect("opened by the runner");
                    let me = windows.current();
                    let states = states.clone();
                    effect(move || states.borrow_mut().push(me.is_open()));
                    // A closes itself later, ending the run.
                    timer(Duration::from_millis(50), move || me.close());
                    Text::new("a")
                })
            })
            .window(
                Window::new("B").size(200.0, 100.0).on_close_request({
                    let asked = asked.clone();
                    move || {
                        asked.set(asked.get() + 1);
                        asked.get() > 1
                    }
                }),
                || Text::new("b"),
            )
    };
    let platform = FakePlatform::new()
        .event(1, WindowEvent::CloseRequested)
        .step(FakeStep::Idle)
        .event(1, WindowEvent::CloseRequested)
        .step(FakeStep::Advance(Duration::from_millis(60)))
        .step(FakeStep::Idle);
    let (log, handler) = run(app, platform, &recorders);
    assert_eq!(asked.get(), 2, "B asked twice");
    assert!(log.windows[1].closed, "the second request was allowed");
    assert!(log.windows[0].closed, "A closed itself");
    assert!(log.exited, "the last window closing ends the run");
    assert!(handler.finish().is_ok());
    // A's effect saw itself open each time it ran (`is_open` reads the open list, so it
    // re-runs when B closes); it is disposed with A, so no `false` ever arrives.
    let states = states.borrow();
    assert!(
        !states.is_empty() && states.iter().all(|open| *open),
        "{states:?}"
    );
}

#[test]
fn facade_win_05_set_title() {
    let recorders = Recorders::default();
    let app = App::new()
        .window(Window::new("A").size(200.0, 100.0), || {
            Builder::new(|cx: &mut BuildCx<'_>| {
                let windows = Windows::of(cx).expect("opened by the runner");
                let me = windows.current();
                timer(Duration::from_millis(10), move || me.set_title("Renamed"));
                Text::new("a")
            })
        })
        .window(Window::new("B").size(200.0, 100.0), || {
            Builder::new(|cx: &mut BuildCx<'_>| {
                let windows = Windows::of(cx).expect("opened by the runner");
                let me = windows.current();
                timer(Duration::from_millis(10), move || {
                    me.close();
                    // A closed handle ignores the call.
                    me.set_title("Ignored");
                });
                Text::new("b")
            })
        });
    let platform = FakePlatform::new()
        .step(FakeStep::Idle)
        .step(FakeStep::Advance(Duration::from_millis(15)))
        .step(FakeStep::Idle);
    let (log, _) = run(app, platform, &recorders);
    assert_eq!(log.windows[0].attributes.title, "Renamed");
    assert!(log.windows[1].closed);
    assert_eq!(log.windows[1].attributes.title, "B");
}

#[test]
fn facade_win_06_one_tick_per_round() {
    let recorders = Recorders::default();
    let ticks = Rc::new(Cell::new(0usize));
    let made: Rc<Cell<Option<Signal<usize>>>> = Rc::default();
    let app = {
        let (ticks, m1, m2) = (ticks.clone(), made.clone(), made.clone());
        App::new()
            .window(Window::new("A").size(200.0, 100.0), move || {
                let count = signal(0usize);
                m1.set(Some(count));
                let ticks = ticks.clone();
                ticker(move |_| {
                    ticks.set(ticks.get() + 1);
                    count.update(|n| *n += 1);
                })
                .start();
                Text::new(move || "x".repeat(count.get()))
            })
            // B shows A's count, so it redraws with A; C shows nothing that changes.
            .window(Window::new("B").size(200.0, 100.0), move || {
                let count = m2.get().expect("A built first");
                Text::new(move || "y".repeat(count.get()))
            })
            .window(Window::new("C").size(200.0, 100.0), || Text::new("c"))
    };
    let platform = FakePlatform::new()
        .step(FakeStep::Idle)
        .step(FakeStep::Idle)
        .step(FakeStep::Idle);
    let (log, _) = run(app, platform, &recorders);
    // As FACADE-APP-14's single window: one tick before the first frames, one per idle,
    // however many windows redraw.
    assert_eq!(
        ticks.get(),
        4,
        "the ticker ran once per round, not once per window"
    );
    assert_eq!(log.windows[0].redraws, 4);
    assert_eq!(log.windows[1].redraws, 4, "B follows A's count");
    assert_eq!(log.windows[2].redraws, 1, "C never changed");
    assert_eq!(recorders.glyphs(0), [4]);
    assert_eq!(recorders.glyphs(1), [4]);
    assert!(log.wakes.is_empty());
}

#[test]
fn facade_win_07_events_and_shared_redraws() {
    let recorders = Recorders::default();
    let made: Rc<Cell<Option<Signal<u32>>>> = Rc::default();
    let presses: Rc<RefCell<Vec<&'static str>>> = Rc::default();
    let app = {
        let (m1, m2) = (made.clone(), made.clone());
        let (p1, p2) = (presses.clone(), presses.clone());
        App::new()
            .window(Window::new("A").size(200.0, 100.0), move || {
                let s = signal(1u32);
                m1.set(Some(s));
                pad(
                    move || p1.borrow_mut().push("A"),
                    move || "a".repeat(s.get() as usize),
                )
            })
            .window(Window::new("B").size(200.0, 100.0), move || {
                let s = m2.get().expect("A built first");
                pad(
                    move || {
                        p2.borrow_mut().push("B");
                        s.update(|n| *n += 1);
                    },
                    || "b".to_owned(),
                )
            })
            .window(Window::new("C").size(200.0, 100.0), || Text::new("c"))
    };
    let [down, up] = press(10.0, 10.0);
    let platform = FakePlatform::new().event(1, down).event(1, up);
    let (log, _) = run(app, platform, &recorders);
    assert_eq!(*presses.borrow(), ["B"], "only B's tree got the press");
    assert_eq!(recorders.glyphs(0), [2], "A shows B's write");
    assert_eq!(log.windows[0].redraws, 2);
    assert_eq!(log.windows[2].redraws, 1, "C doesn't read it");
}
