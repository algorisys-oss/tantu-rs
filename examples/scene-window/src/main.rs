//! `cargo run -p scene-window`: opens a window and draws the animated demo Scene with wgpu.

use std::error::Error;
use std::time::Instant;

use scene_window::{demo_image, demo_scene};
use tantu_platform::{
    Platform, PlatformContext, PlatformHandler, WindowAttributes, WindowEvent, WindowId,
};
use tantu_platform_winit::WinitPlatform;
use tantu_render_wgpu::WgpuRenderer;
use tantu_scene::{ImageId, RenderError, Renderer, Resources, Scene};

/// The demo app: one window, one renderer, a Scene rebuilt every frame.
struct Demo {
    resources: Resources,
    image: ImageId,
    scene: Scene,
    start: Instant,
    window: Option<WindowId>,
    renderer: Option<WgpuRenderer>,
    error: Option<String>,
}

impl Demo {
    fn fail(&mut self, cx: &mut dyn PlatformContext, error: String) {
        self.error = Some(error);
        cx.exit();
    }

    /// Resizes the renderer to the window's current size and scale factor.
    fn fit(&mut self, cx: &mut dyn PlatformContext, window: WindowId) {
        let (Some(size), Some(scale)) = (cx.inner_size(window), cx.scale_factor(window)) else {
            return;
        };
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(size.width, size.height, scale);
        }
        cx.request_redraw(window);
    }
}

impl PlatformHandler for Demo {
    fn started(&mut self, cx: &mut dyn PlatformContext) {
        let attributes = WindowAttributes::new("Tantu · Phase 1 demo")
            .size(900.0, 600.0)
            .min_size(200.0, 150.0);
        let window = match cx.create_window(&attributes) {
            Ok(window) => window,
            Err(e) => return self.fail(cx, format!("opening a window failed: {e}")),
        };
        let (Some(target), Some(size)) = (cx.surface_target(window), cx.inner_size(window)) else {
            return self.fail(cx, "the window has no surface".to_owned());
        };
        match WgpuRenderer::for_window(target, size.width, size.height) {
            Ok(renderer) => self.renderer = Some(renderer),
            Err(e) => return self.fail(cx, format!("creating the wgpu renderer failed: {e}")),
        }
        self.window = Some(window);
        self.fit(cx, window);
    }

    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                cx.close_window(window);
                cx.exit();
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged(_) => self.fit(cx, window),
            WindowEvent::RedrawRequested => {
                let (Some(size), Some(scale), Some(renderer)) = (
                    cx.inner_size(window),
                    cx.scale_factor(window),
                    &mut self.renderer,
                ) else {
                    return;
                };
                let time = self.start.elapsed().as_secs_f32();
                demo_scene(&mut self.scene, size.to_logical(scale), time, self.image);
                match renderer.render(&self.scene, &self.resources) {
                    Ok(_) | Err(RenderError::TargetLost) => {}
                    Err(e) => return self.fail(cx, format!("rendering failed: {e}")),
                }
                // Keep animating; the surface's vsync paces the frames.
                cx.request_redraw(window);
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut resources = Resources::new();
    let image = resources.add_image(demo_image());
    let mut demo = Demo {
        resources,
        image,
        scene: Scene::new(),
        start: Instant::now(),
        window: None,
        renderer: None,
        error: None,
    };
    WinitPlatform::new()?.run(&mut demo)?;
    match demo.error {
        Some(error) => Err(error.into()),
        None => Ok(()),
    }
}
