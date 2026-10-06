//! The [`Renderer`] trait every backend implements, with [`RenderReport`] and [`RenderError`].

use std::fmt;

use tantu_core::{Color, Rect};

use crate::command::{Clip, Command, RoundedRect};
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
        let mut report = RenderReport::default();
        // Depth inside a scope hidden by a non-finite transform or clip; 0 when not hidden.
        let mut hidden = 0usize;
        for entry in scene.entries() {
            let command = &entry.command;
            if hidden > 0 {
                match command {
                    Command::PushClip(_) | Command::PushTransform(_) | Command::PushLayer(_) => {
                        hidden += 1;
                    }
                    Command::PopClip | Command::PopTransform | Command::PopLayer => hidden -= 1,
                    _ => {}
                }
                continue;
            }
            let valid = match command {
                Command::PushTransform(transform) => transform.is_finite(),
                Command::PushClip(clip) => match clip {
                    Clip::Rect(rect) => rect.is_finite(),
                    Clip::RoundedRect(shape) => rounded_rect_is_finite(shape),
                },
                Command::PushLayer(_)
                | Command::PopClip
                | Command::PopTransform
                | Command::PopLayer => true,
                Command::Fill { shape, color } => {
                    rounded_rect_is_finite(shape) && color_is_finite(*color)
                }
                Command::Stroke {
                    shape,
                    width,
                    color,
                } => rounded_rect_is_finite(shape) && width.is_finite() && color_is_finite(*color),
                Command::BoxShadow(shadow) => {
                    rounded_rect_is_finite(&shadow.shape)
                        && color_is_finite(shadow.color)
                        && shadow.offset.is_finite()
                        && shadow.blur_radius.is_finite()
                        && shadow.spread_radius.is_finite()
                }
                Command::Image(image) => {
                    image.dest.is_finite()
                        && image.src.is_none_or(Rect::is_finite)
                        && image.opacity.is_finite()
                }
                Command::GlyphRun(run) => {
                    run.font_size.is_finite()
                        && color_is_finite(run.color)
                        && run.origin.is_finite()
                        && scene
                            .glyphs(run)
                            .iter()
                            .all(|g| g.x.is_finite() && g.y.is_finite())
                }
                Command::Custom(custom) => custom.bounds.is_finite(),
            };
            if !valid {
                bump(&mut report.invalid_commands);
                if matches!(command, Command::PushClip(_) | Command::PushTransform(_)) {
                    hidden = 1;
                }
                continue;
            }
            match command {
                Command::Image(image) if resources.image(image.image).is_none() => {
                    bump(&mut report.missing_images);
                }
                Command::GlyphRun(run) if resources.font(run.font).is_none() => {
                    bump(&mut report.missing_fonts);
                }
                Command::Custom(custom) if !handles_custom(custom.kind) => {
                    bump(&mut report.unhandled_custom);
                }
                _ => {}
            }
        }
        report
    }
}

/// Adds 1, saturating at `u32::MAX` (SCENE-RENDER-08).
fn bump(count: &mut u32) {
    *count = count.saturating_add(1);
}

fn color_is_finite(color: Color) -> bool {
    color.r.is_finite() && color.g.is_finite() && color.b.is_finite() && color.a.is_finite()
}

fn rounded_rect_is_finite(shape: &RoundedRect) -> bool {
    let radii = shape.radii;
    shape.rect.is_finite()
        && radii.top_left.is_finite()
        && radii.top_right.is_finite()
        && radii.bottom_right.is_finite()
        && radii.bottom_left.is_finite()
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
