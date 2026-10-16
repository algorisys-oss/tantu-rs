//! Rules PLATFORM-WINIT-01..04 and -14..17 and -19 against a real window (spec `docs/specs/platform-winit/shell.md`,
//! "Testing"). winit needs the main thread, so this target runs without the test harness. It runs
//! only with `TANTU_WINDOW_TESTS=1`; otherwise it reports that it was skipped.

// Without the harness this `main` is the test runner, so it prints its own results.
#![allow(clippy::print_stdout)]

use std::process::ExitCode;
use std::time::{Duration, Instant};

use tantu_platform::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tantu_platform::{
    PhysicalSize, Platform, PlatformContext, PlatformHandler, WindowAttributes, WindowEvent,
    WindowId,
};
use tantu_platform_winit::WinitPlatform;

/// What happened during the run.
#[derive(Debug, Default)]
struct Run {
    started: u32,
    idle: u32,
    window: Option<WindowId>,
    first_event: Option<WindowEvent>,
    initial_size: Option<PhysicalSize>,
    initial_scale: Option<f32>,
    handles_ok: bool,
    redraws: u32,
    after_close: Option<(Option<PhysicalSize>, Option<f32>, bool)>,
    /// When the wake-up was requested (for 50 ms later) and when `idle` saw it due.
    wake_requested: Option<Instant>,
    woke: Option<Instant>,
    /// Whether an accessibility update could be sent (PLATFORM-WINIT-15).
    a11y_update_sent: bool,
    /// Whether cursors could be set, on the window and on an unknown one (PLATFORM-WINIT-16).
    cursor_set: bool,
    /// The clipboard round trip, if a clipboard opened (PLATFORM-WINIT-17).
    clipboard: Option<(bool, Option<String>)>,
    /// The image, HTML and primary round trips, if a clipboard opened (PLATFORM-WINIT-19).
    rich: Option<RichRoundTrip>,
}

/// What the rich clipboard round trip wrote and read back.
#[derive(Debug, Default)]
struct RichRoundTrip {
    image: (bool, Option<tantu_platform::ClipboardImage>),
    html: (bool, Option<String>, Option<String>),
    primary: (bool, Option<String>),
}

impl PlatformHandler for Run {
    fn started(&mut self, cx: &mut dyn PlatformContext) {
        self.started += 1;
        let w = cx
            .create_window(
                &WindowAttributes::new("Tantu window test")
                    .size(320.0, 200.0)
                    .min_size(100.0, 100.0),
            )
            .expect("window creation");
        self.window = Some(w);
        self.initial_size = cx.inner_size(w);
        self.initial_scale = cx.scale_factor(w);
        self.handles_ok = cx
            .surface_target(w)
            .is_some_and(|t| t.window_handle().is_ok() && t.display_handle().is_ok());
        // The adapter exists from creation; an update while inactive is a no-op.
        use tantu_platform::accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate};
        let mut window = Node::new(Role::Window);
        window.set_label("Tantu window test");
        cx.update_accessibility(
            w,
            TreeUpdate {
                nodes: vec![(NodeId(0), window)],
                tree: Some(Tree::new(NodeId(0))),
                tree_id: TreeId::ROOT,
                focus: NodeId(0),
            },
        );
        self.a11y_update_sent = true;
        cx.set_cursor(w, tantu_platform::CursorIcon::ResizeHorizontal);
        cx.set_cursor(w, tantu_platform::CursorIcon::ResizeHorizontal);
        cx.set_cursor(WindowId::from_raw(99), tantu_platform::CursorIcon::Text);
        self.cursor_set = true;
        if let Some(clipboard) = cx.clipboard() {
            let written = clipboard.write_text("tantu clipboard test");
            self.clipboard = Some((written, clipboard.read_text()));
            let image = tantu_platform::ClipboardImage {
                width: 2,
                height: 1,
                rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
            };
            let image = (clipboard.write_image(&image), clipboard.read_image());
            let html = (
                clipboard.write_html("<b>tantu</b>", "tantu"),
                clipboard.read_html(),
                clipboard.read_text(),
            );
            let primary = (
                clipboard.write_primary("tantu primary"),
                clipboard.read_primary(),
            );
            self.rich = Some(RichRoundTrip {
                image,
                html,
                primary,
            });
        }
    }

    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent) {
        if self.first_event.is_none() {
            self.first_event = Some(event.clone());
        }
        if event == WindowEvent::RedrawRequested {
            self.redraws += 1;
            match self.redraws {
                1 => {
                    cx.set_title(window, "Tantu window test (renamed)");
                    // The second redraw comes from the wake-up (PLATFORM-WINIT-14).
                    let now = cx.now();
                    self.wake_requested = Some(now);
                    cx.wake_at(Some(now + Duration::from_millis(50)));
                }
                2 => {
                    cx.close_window(window);
                    self.after_close = Some((
                        cx.inner_size(window),
                        cx.scale_factor(window),
                        cx.surface_target(window).is_some(),
                    ));
                    cx.exit();
                }
                _ => {}
            }
        }
    }

    fn idle(&mut self, cx: &mut dyn PlatformContext) {
        self.idle += 1;
        if let (Some(requested), None, Some(window)) = (self.wake_requested, self.woke, self.window)
        {
            let now = cx.now();
            if now >= requested + Duration::from_millis(50) {
                self.woke = Some(now);
                cx.wake_at(None);
                cx.request_redraw(window);
            }
        }
    }
}

fn platform_winit_17_clipboard(run: &Run) {
    // A display may have no clipboard owner available; when writing worked, reading
    // returns the text.
    if let Some((true, read)) = &run.clipboard {
        assert_eq!(read.as_deref(), Some("tantu clipboard test"));
    }
}

fn platform_winit_19_rich_clipboard(run: &Run) {
    let Some(rich) = &run.rich else {
        return;
    };
    // What was written reads back (a display may refuse a kind; then nothing is checked).
    if let (true, read) = &rich.image {
        let read = read.as_ref().expect("the image reads back");
        assert_eq!((read.width, read.height), (2, 1));
        assert_eq!(read.rgba, [255, 0, 0, 255, 0, 0, 255, 255]);
    }
    if let (true, html, text) = &rich.html {
        assert!(
            html.as_deref().is_some_and(|h| h.contains("<b>tantu</b>")),
            "{html:?}"
        );
        assert_eq!(text.as_deref(), Some("tantu"));
    }
    match &rich.primary {
        (true, read) => assert_eq!(read.as_deref(), Some("tantu primary")),
        (false, read) => {
            assert!(!cfg!(target_os = "linux") || read.is_none());
            assert!(cfg!(target_os = "linux") || read.is_none());
        }
    }
}

fn platform_winit_16_set_cursor(run: &Run) {
    assert!(run.cursor_set, "set_cursor ran on a real window");
}

fn platform_winit_15_adapter(run: &Run) {
    assert!(
        run.a11y_update_sent,
        "update_accessibility ran without a screen reader"
    );
}

fn platform_winit_14_wake_at(run: &Run) {
    let (requested, woke) = (
        run.wake_requested
            .expect("the first redraw asked for a wake-up"),
        run.woke.expect("idle ran once the deadline passed"),
    );
    assert!(woke - requested >= Duration::from_millis(50));
    assert!(
        woke - requested < Duration::from_secs(2),
        "the loop woke without an event"
    );
}

fn platform_winit_01_run_lifecycle(run: &Run, result: &Result<(), tantu_platform::PlatformError>) {
    assert!(result.is_ok(), "run failed: {result:?}");
    assert_eq!(run.started, 1);
    assert!(run.idle >= 1);
}

fn platform_winit_02_create_window(run: &Run) {
    assert_eq!(run.window.map(WindowId::to_raw), Some(1));
    assert!(
        run.initial_size
            .is_some_and(|s| s.width > 0 && s.height > 0),
        "{:?}",
        run.initial_size
    );
    assert!(run.initial_scale.is_some_and(|s| s > 0.0));
    assert!(
        run.redraws >= 2,
        "a new window gets a redraw, and request_redraw another"
    );
}

fn platform_winit_03_close_window(run: &Run) {
    assert_eq!(run.after_close, Some((None, None, false)));
}

fn platform_winit_04_surface_target(run: &Run) {
    assert!(
        run.handles_ok,
        "surface target with valid window and display handles"
    );
}

fn main() -> ExitCode {
    if std::env::var_os("TANTU_WINDOW_TESTS").is_none_or(|v| v != "1") {
        println!("window tests skipped (set TANTU_WINDOW_TESTS=1 to open a real window)");
        return ExitCode::SUCCESS;
    }
    let platform = match WinitPlatform::new() {
        Ok(platform) => platform,
        Err(e) => {
            println!("window tests failed: no event loop: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut run = Run::default();
    let result = platform.run(&mut run);
    println!("{run:?}");
    platform_winit_01_run_lifecycle(&run, &result);
    platform_winit_02_create_window(&run);
    platform_winit_03_close_window(&run);
    platform_winit_04_surface_target(&run);
    platform_winit_14_wake_at(&run);
    platform_winit_15_adapter(&run);
    platform_winit_16_set_cursor(&run);
    platform_winit_17_clipboard(&run);
    platform_winit_19_rich_clipboard(&run);
    println!("window tests passed");
    ExitCode::SUCCESS
}
