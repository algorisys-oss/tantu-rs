//! [`SoftRenderer`] and the custom-command handler API.

use std::collections::HashMap;
use std::fmt;

use tantu_core::Rect;
use tantu_scene::{CustomKind, ImageData, RenderError, RenderReport, Renderer, Resources, Scene};

/// Draws one kind of custom command.
pub trait CustomHandler {
    /// Draw `canvas.data` into `canvas.pixmap`, inside `canvas.bounds`.
    fn draw(&mut self, canvas: CustomCanvas<'_>);
}

/// What a custom handler draws into.
pub struct CustomCanvas<'a> {
    /// The target (or the open layer's offscreen target), premultiplied RGBA8.
    pub pixmap: tiny_skia::PixmapMut<'a>,
    /// Maps the command's local coordinates to target pixels (current transform × scale factor).
    pub transform: tiny_skia::Transform,
    /// The current clip as a mask over the pixmap, if any clip is open.
    pub clip: Option<&'a tiny_skia::Mask>,
    /// The command's bounds, in local coordinates.
    pub bounds: Rect,
    /// The command's bytes (`Scene::custom_data`).
    pub data: &'a [u8],
}

/// A renderer that draws Scenes into an in-memory RGBA8 target on the CPU.
pub struct SoftRenderer {
    width: u32,
    height: u32,
    scale_factor: f32,
    handlers: HashMap<CustomKind, Box<dyn CustomHandler>>,
}

impl SoftRenderer {
    /// A `width × height` physical-pixel target, scale factor 1, fully transparent.
    pub fn new(width: u32, height: u32) -> SoftRenderer {
        todo!()
    }

    /// Current target width and height in physical pixels.
    pub fn size(&self) -> (u32, u32) {
        todo!()
    }

    /// Current scale factor.
    pub fn scale_factor(&self) -> f32 {
        todo!()
    }

    /// Draw custom commands of `kind` with `handler`, replacing any earlier handler for it.
    pub fn register_custom(&mut self, kind: CustomKind, handler: impl CustomHandler + 'static) {
        todo!()
    }

    /// The target's pixels as straight-alpha RGBA8. `None` for a 0-width or 0-height target.
    pub fn snapshot(&self) -> Option<ImageData> {
        todo!()
    }
}

impl Renderer for SoftRenderer {
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        todo!()
    }

    fn render(
        &mut self,
        scene: &Scene,
        resources: &Resources,
    ) -> Result<RenderReport, RenderError> {
        todo!()
    }
}

impl fmt::Debug for SoftRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftRenderer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale_factor", &self.scale_factor)
            .field("custom_kinds", &self.handlers.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}
