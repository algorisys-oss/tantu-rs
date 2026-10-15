//! Tests for `docs/specs/facade/app.md`, rules FACADE-APP-01..15, -17 and -18.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu::core::{Point, Rect, Size, Vec2};
use tantu::layout::RenderConstrainedBox;
use tantu::platform::{
    ButtonState, FakeLog, FakePlatform, FakeStep, PhysicalSize, PlatformContext, PointerButton,
    PointerId, ScrollDelta, WindowEvent, WindowId,
};
use tantu::prelude::*;
use tantu::reactive::{ticker, timer};
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
        .window(Window::new("A").size(200.0, 100.0), SizedBox::shrink)
        .window(Window::new("B").size(300.0, 150.0), SizedBox::shrink);
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
        .window(Window::new("A"), SizedBox::shrink)
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
    // The button's fill is the last one (the window background comes first), in its own
    // coordinates; add the translations of the transform scopes open around it.
    let mut stack = vec![Vec2::ZERO];
    let mut last = None;
    for entry in frame.scene.entries() {
        let offset = *stack.last().expect("the base offset stays");
        match &entry.command {
            Command::PushTransform(t) => {
                let [_, _, _, _, e, f] = t.coeffs();
                stack.push(offset + Vec2::new(e, f));
            }
            Command::PopTransform => {
                stack.pop();
            }
            Command::Fill { shape, .. } => {
                let r = shape.rect;
                last = Some(Point::new(
                    offset.x + (r.left + r.right) / 2.0,
                    offset.y + (r.top + r.bottom) / 2.0,
                ));
            }
            _ => {}
        }
    }
    last.expect("the button fills its background")
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
    let app = App::new().window(Window::new("A").size(200.0, 100.0), SizedBox::shrink);
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
    let app = App::new().window(Window::new("A"), SizedBox::shrink);
    let mut handler =
        app.handler(recorders.factory(|r| r.fail_next_render(RenderError::TargetLost)));
    let log = FakePlatform::new().run_logged(&mut handler);
    assert_eq!(log.windows[0].redraws, 2);
    assert_eq!(recorders.get(0).borrow().frames().len(), 1);
    assert!(handler.finish().is_ok());

    // Any other error ends the run.
    let app = App::new().window(Window::new("A"), SizedBox::shrink);
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
    let result = App::new().window(Window::new("A"), SizedBox::shrink).run();
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

/// The first fill of a frame: (rect, color), and whether any draw came before it.
fn first_fill(frame: &RecordedFrame) -> (tantu::core::Rect, Color, bool) {
    let mut drew = false;
    for entry in frame.scene.entries() {
        match &entry.command {
            Command::Fill { shape, color } => return (shape.rect, *color, drew),
            Command::GlyphRun(_) | Command::Stroke { .. } | Command::Image(_) => drew = true,
            _ => {}
        }
    }
    panic!("no fill in the frame");
}

#[test]
fn facade_app_10_windows_paint_a_background() {
    let recorders = Recorders::default();
    let red = Color::from_argb32(0xFFFF_0000);
    let app = with_font(App::new())
        .window(Window::new("A").size(200.0, 100.0), || Text::new("Hi"))
        .window(
            Window::new("B").size(300.0, 150.0).background(red),
            SizedBox::shrink,
        );
    run(app, FakePlatform::new().scale_factor(2.0), &recorders);
    for (i, size, color) in [
        (0, Size::new(200.0, 100.0), Color::WHITE),
        (1, Size::new(300.0, 150.0), red),
    ] {
        let renderer = recorders.get(i);
        let renderer = renderer.borrow();
        let frame = renderer.last_frame().expect("a frame");
        let (rect, fill, drew_before) = first_fill(frame);
        assert_eq!(
            rect,
            tantu::core::Rect::from_ltwh(0.0, 0.0, size.width, size.height)
        );
        assert_eq!(fill, color);
        assert!(!drew_before);
    }
}

/// A box filling the window, focused on press, logging key presses as "key shift".
struct KeyLogger(Rc<RefCell<Vec<String>>>);

impl View for KeyLogger {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        use tantu::view::{FocusOptions, LogicalKey};
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.focusable(
            id,
            FocusOptions {
                traversable: true,
                focus_on_press: true,
            },
        );
        let log = self.0;
        cx.on_key(id, Phase::Bubble, move |k| {
            if k.event.pressed {
                let key = match &k.event.key {
                    LogicalKey::Named(named) => format!("{named:?}"),
                    LogicalKey::Character(c) => c.to_string(),
                };
                log.borrow_mut()
                    .push(format!("{key} {}", k.event.modifiers.shift));
            }
            Handled::Continue
        });
        id
    }
}

fn key(key: tantu::platform::Key, state: ButtonState) -> WindowEvent {
    WindowEvent::Keyboard(tantu::platform::KeyEvent {
        key,
        location: tantu::platform::KeyLocation::Standard,
        state,
        repeat: false,
        text: None,
    })
}

#[test]
fn facade_app_11_keyboard_events_are_forwarded() {
    use tantu::platform::{Key, Modifiers as PlatformModifiers, NamedKey as PlatformNamed};
    let keys: Rc<RefCell<Vec<String>>> = Rc::default();
    let k = keys.clone();
    let app = App::new().window(Window::new("A").size(200.0, 100.0), move || KeyLogger(k));
    let shift = PlatformModifiers {
        shift: true,
        ..PlatformModifiers::default()
    };
    let platform = FakePlatform::new()
        // Focus the logger with a click.
        .event(0, button(ButtonState::Pressed, 5.0, 5.0))
        .event(0, button(ButtonState::Released, 5.0, 5.0))
        .event(
            0,
            key(Key::Named(PlatformNamed::Enter), ButtonState::Pressed),
        )
        .event(0, WindowEvent::ModifiersChanged(shift))
        .event(
            0,
            key(Key::Named(PlatformNamed::Shift), ButtonState::Pressed),
        )
        .event(0, key(Key::Character("A".into()), ButtonState::Pressed))
        .event(0, key(Key::Named(PlatformNamed::F5), ButtonState::Pressed))
        .event(0, key(Key::Unidentified, ButtonState::Pressed))
        .event(
            0,
            WindowEvent::ModifiersChanged(PlatformModifiers::default()),
        )
        .event(
            0,
            key(Key::Named(PlatformNamed::ArrowLeft), ButtonState::Pressed),
        );
    run(app, platform, &Recorders::default());
    assert_eq!(
        *keys.borrow(),
        ["Enter false", "A true", "F(5) true", "ArrowLeft false"]
    );
}

/// A focusable view logging IME events, providing an IME area while `area` holds one.
struct ImeLogger {
    log: Rc<RefCell<Vec<String>>>,
    area: Rc<Cell<Option<Rect>>>,
}

impl View for ImeLogger {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        use tantu::view::{FocusOptions, ImeEvent};
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.focusable(
            id,
            FocusOptions {
                traversable: true,
                focus_on_press: true,
            },
        );
        let (log, area) = (self.log, self.area.clone());
        let tick = tantu::reactive::signal(0u32);
        // Focus changes ask for a frame (a real widget repaints its focus indicator).
        cx.on_focus_change(id, move |_| tick.update(|t| *t += 1));
        cx.on_ime(id, move |i| {
            log.borrow_mut().push(format!("{:?}", i.event));
            if matches!(i.event, ImeEvent::Commit(_)) {
                // After a commit the view stops wanting IME input (and asks for a frame).
                area.set(None);
                tick.update(|t| *t += 1);
            }
        });
        cx.bind(
            id,
            tantu::view::Prop::Dynamic(Box::new(move || tick.get())),
            |_, _| {},
        );
        let area = self.area;
        cx.ime_area(id, move || area.get());
        id
    }
}

#[test]
fn facade_app_12_ime_events_and_area() {
    use tantu::platform::ImeEvent;
    let log: Rc<RefCell<Vec<String>>> = Rc::default();
    let area: Rc<Cell<Option<Rect>>> =
        Rc::new(Cell::new(Some(Rect::from_ltwh(1.0, 2.0, 3.0, 4.0))));
    let (l, a) = (log.clone(), area.clone());
    let app = App::new().window(Window::new("A").size(200.0, 100.0), move || ImeLogger {
        log: l,
        area: a,
    });
    let platform = FakePlatform::new()
        .event(0, button(ButtonState::Pressed, 5.0, 5.0))
        .event(0, button(ButtonState::Released, 5.0, 5.0))
        .event(
            0,
            WindowEvent::Ime(ImeEvent::Preedit {
                text: "ab".into(),
                cursor: Some((0, 2)),
            }),
        )
        .event(0, WindowEvent::Ime(ImeEvent::Commit("ab".into())));
    let (fake, _) = run(app, platform, &Recorders::default());
    assert_eq!(
        *log.borrow(),
        [
            "Preedit { text: \"ab\", cursor: Some((0, 2)) }",
            "Commit(\"ab\")"
        ]
    );
    // The IME was allowed with the area once the view was focused, and disallowed after the
    // commit (the frame after it saw no area).
    assert!(!fake.windows[0].ime_allowed);
    assert_eq!(
        fake.windows[0].ime_area,
        Some((Point::new(1.0, 2.0), Size::new(3.0, 4.0)))
    );
    // Without a focused provider, the IME is never enabled.
    let area: Rc<Cell<Option<Rect>>> = Rc::default();
    let log: Rc<RefCell<Vec<String>>> = Rc::default();
    let (l, a) = (log.clone(), area.clone());
    let app = App::new().window(Window::new("A").size(200.0, 100.0), move || ImeLogger {
        log: l,
        area: a,
    });
    let platform = FakePlatform::new()
        .event(0, button(ButtonState::Pressed, 5.0, 5.0))
        .event(0, button(ButtonState::Released, 5.0, 5.0));
    let (fake, _) = run(app, platform, &Recorders::default());
    assert!(!fake.windows[0].ime_allowed);
    assert_eq!(fake.windows[0].ime_area, None);
}

/// A view that records the provided theme signal (if any) and runs `then` with it.
fn theme_probe(
    seen: Rc<Cell<Option<Signal<Theme>>>>,
    then: impl Fn(Option<Signal<Theme>>) + 'static,
) -> impl View {
    Builder::new(move |cx| {
        let theme = Theme::signal(cx);
        seen.set(theme);
        then(theme);
        SizedBox::shrink()
    })
}

#[cfg(feature = "default-theme")]
#[test]
fn facade_app_13_runner_provides_the_theme() {
    let recorders = Recorders::default();
    let red = Color::from_argb32(0xFFFF_0000);
    let seen: Rc<Cell<Option<Signal<Theme>>>> = Rc::default();
    let app = with_font(App::new())
        .window(Window::new("A").size(200.0, 100.0), {
            let seen = seen.clone();
            move || theme_probe(seen, |_| {})
        })
        .window(Window::new("B").size(200.0, 100.0), || {
            theme_probe(Rc::default(), |theme| {
                theme.expect("provided").set(Theme::dark());
            })
        })
        .window(Window::new("C").size(200.0, 100.0).background(red), || {
            theme_probe(Rc::default(), |theme| {
                theme.expect("provided").set(Theme::dark());
            })
        });
    run(app, FakePlatform::new(), &recorders);
    assert!(seen.get().is_some());
    for (i, color) in [
        (0, Color::WHITE),
        (1, Theme::dark().colors.surface),
        (2, red),
    ] {
        let renderer = recorders.get(i);
        let renderer = renderer.borrow();
        let frame = renderer.last_frame().expect("a frame");
        assert_eq!(first_fill(frame).1, color, "window {i}");
    }
}

#[cfg(not(feature = "default-theme"))]
#[test]
fn facade_app_13_without_the_feature() {
    let recorders = Recorders::default();
    let seen: Rc<Cell<Option<Signal<Theme>>>> = Rc::default();
    let app = with_font(App::new()).window(Window::new("A").size(200.0, 100.0), {
        let seen = seen.clone();
        move || theme_probe(seen, |_| {})
    });
    run(app, FakePlatform::new(), &recorders);
    assert!(seen.get().is_none());
    let renderer = recorders.get(0);
    let renderer = renderer.borrow();
    let frame = renderer.last_frame().expect("a frame");
    assert_eq!(first_fill(frame).1, Color::WHITE);
}

#[test]
fn facade_app_14_ticks_and_wake_ups() {
    use std::time::Duration;
    // A timer: the runner asks for one wake-up, then redraws once the deadline passed.
    let recorders = Recorders::default();
    let app = with_font(App::new()).window(Window::new("A").size(200.0, 100.0), || {
        let text = signal("a".to_owned());
        timer(Duration::from_millis(20), move || {
            text.set("abc".to_owned())
        });
        Text::new(text)
    });
    let platform = FakePlatform::new()
        .step(FakeStep::Idle)
        .step(FakeStep::Advance(Duration::from_millis(10)))
        .step(FakeStep::Advance(Duration::from_millis(15)));
    let (log, _) = run(app, platform, &recorders);
    // One wake-up for the deadline, cleared once it passed (a passed `WaitUntil` would fire
    // again at once).
    assert_eq!(log.wakes, [Some(Duration::from_millis(20)), None]);
    assert_eq!(
        log.windows[0].redraws, 2,
        "the initial frame and the one after the deadline"
    );
    let renderer = recorders.get(0);
    let renderer = renderer.borrow();
    assert_eq!(glyph_counts(renderer.last_frame().expect("a frame")), [3]);

    // A ticker: every idle requests the next frame, no wake-ups.
    let recorders = Recorders::default();
    let app = with_font(App::new()).window(Window::new("A").size(200.0, 100.0), || {
        let count = signal(0usize);
        ticker(move |_| count.update(|n| *n += 1)).start();
        Text::new(move || "x".repeat(count.get()))
    });
    let platform = FakePlatform::new()
        .step(FakeStep::Idle)
        .step(FakeStep::Idle)
        .step(FakeStep::Idle);
    let (log, _) = run(app, platform, &recorders);
    assert!(log.wakes.is_empty());
    assert_eq!(log.windows[0].redraws, 4);
    let renderer = recorders.get(0);
    let renderer = renderer.borrow();
    assert_eq!(glyph_counts(renderer.last_frame().expect("a frame")), [4]);
}

#[test]
fn facade_app_15_accessibility_trees_and_actions() {
    use tantu::platform::AccessibilityEvent;
    use tantu::platform::accesskit::{Action, ActionRequest, Role, TreeId};
    let presses = Rc::new(Cell::new(0u32));
    let app = with_font(App::new()).window(Window::new("Hello app").size(300.0, 200.0), {
        let presses = presses.clone();
        move || {
            let presses = presses.clone();
            Column::new().child(Button::new("Go").on_press(move || presses.set(presses.get() + 1)))
        }
    });
    let updates_after_step = |log: &FakeLog| log.windows[0].accessibility_updates.len();
    // Nothing is sent before activation, even though frames happen.
    let platform = FakePlatform::new()
        .event(0, WindowEvent::Resized(PhysicalSize::new(320, 200)))
        .event(0, WindowEvent::Accessibility(AccessibilityEvent::Activated));
    let (log, _) = run(app, platform, &Recorders::default());
    assert_eq!(updates_after_step(&log), 1, "one tree on activation");
    let first = &log.windows[0].accessibility_updates[0];
    let root = first.tree.as_ref().expect("a full tree").root;
    let window = &first
        .nodes
        .iter()
        .find(|(id, _)| *id == root)
        .expect("root")
        .1;
    assert_eq!(window.label(), Some("Hello app"));
    let (button_id, button) = first
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Button)
        .expect("the button has a node");
    assert_eq!(button.label(), Some("Go"));

    // Active: every frame sends a tree; an action reaches the view; deactivation stops it.
    let presses = Rc::new(Cell::new(0u32));
    let app = with_font(App::new()).window(Window::new("Hello app").size(300.0, 200.0), {
        let presses = presses.clone();
        move || {
            let presses = presses.clone();
            Column::new().child(Button::new("Go").on_press(move || presses.set(presses.get() + 1)))
        }
    });
    let click = ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: *button_id,
        data: None,
    };
    let platform = FakePlatform::new()
        .event(0, WindowEvent::Accessibility(AccessibilityEvent::Activated))
        .step(FakeStep::Idle)
        .event(0, WindowEvent::Resized(PhysicalSize::new(320, 200)))
        .event(
            0,
            WindowEvent::Accessibility(AccessibilityEvent::Action(click)),
        )
        .event(
            0,
            WindowEvent::Accessibility(AccessibilityEvent::Deactivated),
        )
        .event(0, WindowEvent::Resized(PhysicalSize::new(330, 200)));
    let (log, _) = run(app, platform, &Recorders::default());
    assert_eq!(presses.get(), 1, "the click reached the button");
    // Activation: 1; idle: nothing; resize frame: 1; the click's frame: 1; after
    // deactivation: nothing.
    assert_eq!(updates_after_step(&log), 3);
}

/// A box filling its constraints that asks for `cursor`.
struct CursorZone(tantu::view::CursorIcon);

impl View for CursorZone {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.set_cursor(id, self.0);
        id
    }
}

#[test]
fn facade_app_17_cursor_follows_the_tree() {
    use std::time::Duration;
    use tantu::platform::CursorIcon;
    use tantu::view::CursorIcon as ViewCursor;
    // A text zone above a resize zone; a timer shrinks the text zone under a still pointer.
    let app = App::new().window(Window::new("A").size(200.0, 100.0), || {
        let height = signal(50.0f32);
        timer(Duration::from_millis(20), move || height.set(20.0));
        Column::new()
            .child(SizedBox::height(height).child(CursorZone(ViewCursor::Text)))
            .child(Expanded::new(CursorZone(ViewCursor::ResizeVertical)))
    });
    let moved = |y: f32| WindowEvent::PointerMoved {
        pointer: PointerId::Mouse,
        position: Point::new(5.0, y),
    };
    let platform = FakePlatform::new()
        .step(FakeStep::Idle)
        .event(0, moved(30.0))
        // Still over the text zone: nothing new is sent.
        .event(0, moved(35.0))
        // The frame after the deadline moves the boundary above the pointer.
        .step(FakeStep::Advance(Duration::from_millis(25)))
        .event(0, moved(36.0))
        .event(
            0,
            WindowEvent::PointerLeft {
                pointer: PointerId::Mouse,
            },
        );
    let (log, _) = run(app, platform, &Recorders::default());
    assert_eq!(
        log.windows[0].cursors,
        [CursorIcon::Text, CursorIcon::ResizeVertical],
        "only changes are sent, after pointer events and after frames; leaving sends nothing"
    );
}

/// Writes "from the app" to the clipboard on a press, after recording what it held.
struct ClipboardUser(Rc<RefCell<Vec<Option<String>>>>);

impl View for ClipboardUser {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::expand(), []);
        let clipboard = cx.clipboard();
        let seen = self.0;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if let PointerKind::Down(_) = p.event.kind {
                seen.borrow_mut().push(clipboard.read_text());
                clipboard.write_text("from the app");
            }
            Handled::Continue
        });
        id
    }
}

#[test]
fn facade_app_18_runner_installs_the_platform_clipboard() {
    let seen: Rc<RefCell<Vec<Option<String>>>> = Rc::default();
    let s = seen.clone();
    let app = App::new().window(Window::new("A").size(200.0, 100.0), move || {
        ClipboardUser(s.clone())
    });
    let platform = FakePlatform::new()
        .clipboard_text("from the platform")
        .event(0, button(ButtonState::Pressed, 5.0, 5.0))
        .event(0, button(ButtonState::Released, 5.0, 5.0));
    let (log, _) = run(app, platform, &Recorders::default());
    assert_eq!(*seen.borrow(), [Some("from the platform".to_owned())]);
    assert_eq!(log.clipboard_text.as_deref(), Some("from the app"));
}
