//! Rules PLATFORM-WINIT-01..04 against a real window (spec `docs/specs/platform-winit/shell.md`,
//! "Testing"). winit needs the main thread, so this target runs without the test harness. It runs
//! only with `TANTU_WINDOW_TESTS=1`; otherwise it reports that it was skipped.

// Without the harness this `main` is the test runner, so it prints its own results.
#![allow(clippy::print_stdout)]

use std::process::ExitCode;

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
                    cx.request_redraw(window);
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

    fn idle(&mut self, _cx: &mut dyn PlatformContext) {
        self.idle += 1;
    }
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
    println!("window tests passed");
    ExitCode::SUCCESS
}
