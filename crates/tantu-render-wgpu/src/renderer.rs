//! [`WgpuRenderer`] and the custom-command handler API.

use std::fmt;

use tantu_core::Rect;
use tantu_scene::{CustomKind, ImageData, RenderError, RenderReport, Renderer, Resources, Scene};

/// Why a renderer couldn't be created.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateError {
    /// No GPU adapter fits (no driver, or none supports what Tantu needs).
    NoAdapter,
    /// Creating the device or the surface failed.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for CreateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        todo!()
    }
}

/// Draws one kind of custom command with wgpu.
pub trait CustomHandler {
    /// Record drawing for `canvas.data` into `canvas.target`, inside `canvas.bounds`.
    fn draw(&mut self, canvas: CustomCanvas<'_>);
}

/// What a custom handler draws into.
pub struct CustomCanvas<'a> {
    /// The renderer's device.
    pub device: &'a wgpu::Device,
    /// The renderer's queue.
    pub queue: &'a wgpu::Queue,
    /// Commands recorded here run in paint order with the renderer's own.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// The texture to draw into (the target, or the open layer's offscreen texture).
    pub target: &'a wgpu::TextureView,
    /// Its format (premultiplied alpha, sRGB-encoded values, no hardware sRGB conversion).
    pub format: wgpu::TextureFormat,
    /// Its size in physical pixels.
    pub target_size: (u32, u32),
    /// Maps the command's local coordinates to target pixels, as `[a, b, c, d, e, f]` (as
    /// `tantu_core::Affine`, times the scale factor).
    pub transform: [f32; 6],
    /// The bounding box of the current clip in target pixels (`[x, y, width, height]`), for a
    /// scissor rect; `None` with no open clip. Rounded and rotated clips are approximated by
    /// their bounding box here.
    pub clip: Option<[u32; 4]>,
    /// The command's bounds, in local coordinates.
    pub bounds: Rect,
    /// The command's bytes (`Scene::custom_data`).
    pub data: &'a [u8],
}

/// A renderer that draws Scenes with wgpu.
pub struct WgpuRenderer {
    width: u32,
    height: u32,
    scale_factor: f32,
}

impl WgpuRenderer {
    /// An offscreen `width × height` target (RGBA8, premultiplied alpha), scale factor 1, fully
    /// transparent. Blocks while wgpu finds an adapter and creates a device.
    pub fn new_offscreen(width: u32, height: u32) -> Result<WgpuRenderer, CreateError> {
        todo!()
    }

    /// A renderer drawing into `window`'s surface, `width × height` physical pixels.
    pub fn for_window<W>(window: W, width: u32, height: u32) -> Result<WgpuRenderer, CreateError>
    where
        W: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + Send
            + Sync
            + 'static,
    {
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

    /// An offscreen target's pixels as straight-alpha RGBA8, read back from the GPU. `None` for
    /// a window renderer or a 0-width or 0-height target.
    pub fn snapshot(&self) -> Option<ImageData> {
        todo!()
    }

    /// Information about the adapter in use (name, backend), for logs and test output.
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        todo!()
    }
}

impl Renderer for WgpuRenderer {
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

impl fmt::Debug for WgpuRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WgpuRenderer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale_factor", &self.scale_factor)
            .finish_non_exhaustive()
    }
}
