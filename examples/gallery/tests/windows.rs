//! Tests for `docs/specs/facade/windows.md`, rule GALLERY-53: the gallery's second window,
//! driven through the app runner and the fake platform.

use std::cell::RefCell;
use std::rc::Rc;

use tantu::platform::{
    ButtonState, FakePlatform, Key, KeyEvent, KeyLocation, NamedKey, PlatformContext, WindowEvent,
    WindowId,
};
use tantu::prelude::*;
use tantu::scene::{Command, RenderError, RenderReport, Renderer, Resources, Scene};
use tantu_render_headless::HeadlessRenderer;

const FONT: &[u8] =
    include_bytes!("../../../crates/tantu-text/tests/fonts/LiberationSans-Regular.ttf");

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

fn key(key: Key) -> [WindowEvent; 2] {
    [ButtonState::Pressed, ButtonState::Released].map(|state| {
        WindowEvent::Keyboard(KeyEvent {
            key: key.clone(),
            location: KeyLocation::Standard,
            state,
            repeat: false,
            text: None,
        })
    })
}

/// The window background's color in a renderer's last frame (its first fill).
fn background(renderer: &Shared) -> Color {
    let renderer = renderer.borrow();
    let frame = renderer.last_frame().expect("a frame");
    frame
        .scene
        .entries()
        .iter()
        .find_map(|e| match &e.command {
            Command::Fill { color, .. } => Some(*color),
            _ => None,
        })
        .expect("a background fill")
}

#[test]
fn gallery_53_second_window_shares_dark_mode() {
    let renderers: Rc<RefCell<Vec<Shared>>> = Rc::default();
    let mut handler = gallery::app()
        .without_system_fonts()
        .font(FONT.to_vec())
        .handler({
            let renderers = renderers.clone();
            move |_: &mut dyn PlatformContext, _: WindowId| {
                let shared = Rc::new(RefCell::new(HeadlessRenderer::new(1, 1)));
                renderers.borrow_mut().push(shared.clone());
                Ok(Box::new(Recorder(shared)) as Box<dyn Renderer>)
            }
        });
    let mut platform = FakePlatform::new();
    // View → New window, by keyboard: Tab to the menu bar, Down opens View, Down, Enter.
    for k in [
        Key::Named(NamedKey::Tab),
        Key::Named(NamedKey::ArrowDown),
        Key::Named(NamedKey::ArrowDown),
        Key::Named(NamedKey::Enter),
    ] {
        for event in key(k) {
            platform = platform.event(0, event);
        }
    }
    // In the second window: Tab to its switch, Space toggles it.
    for k in [Key::Named(NamedKey::Tab), Key::Character(" ".into())] {
        for event in key(k) {
            platform = platform.event(1, event);
        }
    }
    let log = platform.run_logged(&mut handler);
    assert_eq!(log.windows.len(), 2, "View → New window opened a window");
    let second = &log.windows[1];
    assert_eq!(second.attributes.title, "Gallery · second window");
    assert_eq!(second.attributes.size, Size::new(360.0, 200.0));
    let dark = Theme::dark().colors.surface;
    let renderers = renderers.borrow();
    assert_eq!(
        background(&renderers[1]),
        dark,
        "the second window turned dark"
    );
    assert_eq!(
        background(&renderers[0]),
        dark,
        "the main window follows the second window's switch"
    );
}
