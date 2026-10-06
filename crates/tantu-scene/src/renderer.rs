//! The [`Renderer`] trait every backend implements, with [`RenderReport`] and [`RenderError`].

use std::fmt;

use crate::handles::CustomKind;
use crate::resources::Resources;
use crate::scene::Scene;

/// What a renderer couldn't draw in one frame. All zero means everything was drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderReport {
    /// `Image` commands whose handle isn't in the resources (or that the backend couldn't
    /// upload).
    pub missing_images: u32,
    /// `GlyphRun` commands whose font isn't in the resources (or that the backend couldn't
    /// load).
    pub missing_fonts: u32,
    /// `Custom` commands with no handler for their kind.
    pub unhandled_custom: u32,
    /// Draw commands skipped because of non-finite values.
    pub invalid_commands: u32,
}

impl RenderReport {
    /// True if every count is 0.
    pub fn is_clean(&self) -> bool {
        *self == RenderReport::default()
    }

    /// The report every renderer gives for `scene` with `resources`, before any
    /// backend-specific failures (an image that couldn't be uploaded, a font that couldn't be
    /// parsed), which the backend adds on top. `handles_custom` says whether the renderer has a
    /// handler for a kind.
    ///
    /// Non-finite transforms and clips hide their scope; draw commands with non-finite values
    /// are invalid; then missing images and fonts and unhandled custom kinds are counted. Does
    /// not allocate.
    pub fn for_scene(
        scene: &Scene,
        resources: &Resources,
        handles_custom: &dyn Fn(CustomKind) -> bool,
    ) -> RenderReport {
        todo!()
    }
}

/// Why a frame couldn't be rendered at all.
#[derive(Debug)]
#[non_exhaustive]
pub enum RenderError {
    /// The target (e.g. a window surface) was lost or is outdated. Call
    /// [`Renderer::resize`] and try again.
    TargetLost,
    /// The backend ran out of memory.
    OutOfMemory,
    /// Any other backend failure.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::TargetLost => f.write_str("render target lost or outdated"),
            RenderError::OutOfMemory => f.write_str("renderer out of memory"),
            RenderError::Backend(e) => write!(f, "renderer backend error: {e}"),
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderError::Backend(e) => Some(e.as_ref()),
            RenderError::TargetLost | RenderError::OutOfMemory => None,
        }
    }
}

/// Draws Scenes into a target. Implemented by each backend; object-safe.
///
/// A renderer is created by its backend crate, already bound to its target. See
/// `docs/specs/scene/renderer.md` for the contract every implementation follows: each frame
/// starts transparent, missing resources are counted in the [`RenderReport`] rather than
/// failing, and `render` never panics.
pub trait Renderer {
    /// Sets the target size in physical pixels and the scale from logical to physical pixels.
    /// A non-finite or non-positive `scale_factor` is treated as 1.
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32);

    /// Draws `scene`, looking up its handles in `resources`.
    fn render(&mut self, scene: &Scene, resources: &Resources)
    -> Result<RenderReport, RenderError>;
}
