//! Tests for `docs/specs/examples/counter.md`, rules COUNTER-01..02.

use std::cell::RefCell;
use std::rc::Rc;

use tantu::core::{Point, Vec2};
use tantu::platform::{
    ButtonState, FakePlatform, PlatformContext, PointerButton, PointerId, WindowEvent, WindowId,
};
use tantu::prelude::*;
use tantu::scene::{Command, RenderError, RenderReport, Renderer, Resources, Scene};
use tantu_render_headless::{HeadlessRenderer, RecordedFrame};

/// The example's `main.rs`, in a module so its private `counter` can be reached.
#[allow(dead_code)]
mod app {
    include!("../src/main.rs");

    /// The example's `counter` view.
    pub fn view() -> impl tantu::view::View {
        counter()
    }
}

const FONT: &[u8] =
    include_bytes!("../../../crates/tantu-text/tests/fonts/LiberationSans-Regular.ttf");

/// The source with all whitespace removed.
fn normalized(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn counter_01_main_is_the_agents_snippet() {
    let agents = include_str!("../../../AGENTS.md");
    let section = agents
        .split("## Authoring style we are aiming for")
        .nth(1)
        .expect("AGENTS.md has the authoring section");
    let snippet = section
        .split("```rust")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .expect("the section starts with a Rust code block");
    let main = include_str!("../src/main.rs");
    assert_eq!(normalized(main), normalized(snippet));
}

type Shared = Rc<RefCell<HeadlessRenderer>>;

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

/// Runs `app` on `platform` and returns the frames of its first window.
fn frames(app: App, platform: FakePlatform) -> Vec<RecordedFrame> {
    let shared: Shared = Rc::new(RefCell::new(HeadlessRenderer::new(1, 1)));
    let renderer = shared.clone();
    let mut handler = app.handler(
        move |_: &mut dyn PlatformContext, _: WindowId| -> tantu::Result<Box<dyn Renderer>> {
            Ok(Box::new(Recorder(renderer.clone())))
        },
    );
    platform.run_logged(&mut handler);
    handler.finish().expect("the run succeeds");
    shared.borrow_mut().take_frames()
}

fn window() -> Window {
    Window::new("Counter").size(400.0, 300.0)
}

fn with_font(app: App) -> App {
    app.without_system_fonts().font(FONT.to_vec())
}

/// The glyph ids of the first glyph run in a frame.
fn first_run(frame: &RecordedFrame) -> Vec<u32> {
    frame
        .scene
        .entries()
        .iter()
        .find_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(frame.scene.glyphs(run).iter().map(|g| g.id).collect()),
            _ => None,
        })
        .unwrap_or_default()
}

/// The glyph ids of `text` in the title style.
fn title_glyphs(text: &'static str) -> Vec<u32> {
    let app =
        with_font(App::new()).window(window(), move || Text::new(text).style(TextStyle::title()));
    first_run(frames(app, FakePlatform::new()).last().expect("a frame"))
}

/// The center of the last fill (the button; the window background comes first) in window
/// coordinates.
fn button_center(frame: &RecordedFrame) -> Point {
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

fn click(platform: FakePlatform, at: Point) -> FakePlatform {
    let event = |state| WindowEvent::PointerButton {
        pointer: PointerId::Mouse,
        button: PointerButton::Primary,
        state,
        position: at,
    };
    platform
        .event(0, event(ButtonState::Pressed))
        .event(0, event(ButtonState::Released))
}

#[test]
fn counter_02_pressing_increments() {
    let first = frames(
        with_font(App::new()).window(window(), app::view),
        FakePlatform::new(),
    );
    let first = first.last().expect("a frame");
    assert_eq!(first_run(first), title_glyphs("Count: 0"));
    let center = button_center(first);

    let once = click(FakePlatform::new(), center);
    let frames1 = frames(with_font(App::new()).window(window(), app::view), once);
    assert_eq!(
        first_run(frames1.last().expect("a frame")),
        title_glyphs("Count: 1")
    );

    let thrice = click(click(click(FakePlatform::new(), center), center), center);
    let frames3 = frames(with_font(App::new()).window(window(), app::view), thrice);
    assert_eq!(
        first_run(frames3.last().expect("a frame")),
        title_glyphs("Count: 3")
    );
}
