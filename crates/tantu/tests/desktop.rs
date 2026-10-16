//! Tests for `docs/specs/platform/native-dialogs.md`, rules FACADE-DESK-01..03.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use tantu::core::Point;
use tantu::layout::RenderConstrainedBox;
use tantu::platform::{
    ButtonState, FakeLog, FakePlatform, FakeStep, PlatformContext, PointerButton, PointerId,
    WindowEvent, WindowId,
};
use tantu::prelude::*;
use tantu::scene::{RenderError, RenderReport, Renderer, Resources, Scene};
use tantu::view::{BuildCx, ElementId, Handled, Phase, PointerKind};
use tantu_render_headless::HeadlessRenderer;

const FONT: &[u8] = include_bytes!("../../tantu-text/tests/fonts/LiberationSans-Regular.ttf");

struct Recorder(Rc<RefCell<HeadlessRenderer>>);

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

fn run(app: App, platform: FakePlatform) -> FakeLog {
    let mut handler = app.without_system_fonts().font(FONT.to_vec()).handler(
        |_: &mut dyn PlatformContext, _: WindowId| {
            Ok(
                Box::new(Recorder(Rc::new(RefCell::new(HeadlessRenderer::new(1, 1)))))
                    as Box<dyn Renderer>,
            )
        },
    );
    platform.run_logged(&mut handler)
}

fn click(platform: FakePlatform) -> FakePlatform {
    [ButtonState::Pressed, ButtonState::Released]
        .into_iter()
        .fold(platform, |p, state| {
            p.event(
                0,
                WindowEvent::PointerButton {
                    pointer: PointerId::Mouse,
                    button: PointerButton::Primary,
                    state,
                    position: Point::new(5.0, 5.0),
                },
            )
        })
}

type Log = Rc<RefCell<Vec<String>>>;

/// A box filling the window: on a press it asks for a file, a yes/no answer and an
/// ok/cancel answer, and shows a notification, logging every outcome.
struct Asker(Log);

impl View for Asker {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let desktop = Desktop::of(cx).expect("opened by the runner");
        let log = self.0;
        log.borrow_mut()
            .push(format!("native {}", desktop.has_native_dialogs()));
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if matches!(p.event.kind, PointerKind::Down(_)) {
                let question = |buttons| MessageRequest {
                    level: MessageLevel::Warning,
                    title: "Save?".into(),
                    description: "Unsaved changes".into(),
                    buttons,
                };
                let l = log.clone();
                desktop.file_dialog(
                    FileDialogRequest::new(FileDialogKind::Open).filter("Text", &["txt"]),
                    move |paths| l.borrow_mut().push(format!("paths {paths:?}")),
                );
                let l = log.clone();
                desktop.message(question(MessageButtons::YesNo), move |r| {
                    l.borrow_mut().push(format!("yes/no {r:?}"))
                });
                let l = log.clone();
                desktop.message(question(MessageButtons::OkCancel), move |r| {
                    l.borrow_mut().push(format!("ok/cancel {r:?}"))
                });
                let notified = desktop.notify("Done", "All saved");
                log.borrow_mut().push(format!("notified {notified}"));
            }
            Handled::Stop
        });
        id
    }
}

fn app(log: &Log) -> App {
    let log = log.clone();
    App::new().window(Window::new("A").size(200.0, 100.0), move || Asker(log))
}

#[test]
fn facade_desk_01_dialogs_and_notifications() {
    let log: Log = Rc::default();
    let platform = click(
        FakePlatform::new()
            .file_dialog_answer(vec![PathBuf::from("/tmp/a.txt")])
            .message_answer(MessageResult::Yes)
            .message_answer(MessageResult::Ok),
    )
    .step(FakeStep::Idle);
    let fake = run(app(&log), platform);
    assert_eq!(
        *log.borrow(),
        [
            "native true",
            "notified true",
            r#"paths ["/tmp/a.txt"]"#,
            "yes/no Yes",
            "ok/cancel Ok",
        ]
    );
    assert_eq!(fake.file_dialogs.len(), 1);
    assert_eq!(
        fake.file_dialogs[0].1.filters,
        [("Text".into(), vec!["txt".into()])]
    );
    assert_eq!(fake.message_dialogs.len(), 2);
    assert_eq!(fake.notifications[0].body, "All saved");
    // A window closed before its answers drops the callbacks.
    let log: Log = Rc::default();
    let platform = click(FakePlatform::new()).event(0, WindowEvent::CloseRequested);
    run(app(&log), platform);
    assert_eq!(*log.borrow(), ["native true", "notified true"]);
}

#[test]
fn facade_desk_02_unsupported_answers_like_cancel() {
    let log: Log = Rc::default();
    let platform = click(FakePlatform::new().without_native_dialogs()).step(FakeStep::Idle);
    let fake = run(app(&log), platform);
    assert_eq!(
        *log.borrow(),
        [
            "native false",
            "notified false",
            "paths []",
            "yes/no No",
            "ok/cancel Cancel",
        ]
    );
    assert!(fake.file_dialogs.is_empty() && fake.notifications.is_empty());
}

#[test]
fn facade_desk_03_opt_in_features() {
    let manifest = include_str!("../Cargo.toml");
    let line = |name: &str| {
        manifest
            .lines()
            .find(|l| l.starts_with(&format!("{name} = ")))
            .unwrap_or_else(|| panic!("no {name} line"))
            .to_owned()
    };
    assert!(line("native-dialogs").contains("tantu-platform-winit/native-dialogs"));
    assert!(line("notifications").contains("tantu-platform-winit/notifications"));
    let default = line("default");
    assert!(!default.contains("native-dialogs") && !default.contains("notifications"));
}
