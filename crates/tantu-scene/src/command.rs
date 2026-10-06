//! The command set: shapes, layers, images, glyph runs, custom commands, and [`Entry`].

use tantu_core::{Affine, Color, Point, Rect, Vec2};

use crate::handles::{CustomKind, ElementId, FontId, ImageId};

/// Per-corner circular radii, Flutter's `BorderRadius` (circular corners only).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderRadius {
    /// Top-left corner radius.
    pub top_left: f32,
    /// Top-right corner radius.
    pub top_right: f32,
    /// Bottom-right corner radius.
    pub bottom_right: f32,
    /// Bottom-left corner radius.
    pub bottom_left: f32,
}

impl BorderRadius {
    /// All four corners zero.
    pub const ZERO: BorderRadius = BorderRadius {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    /// The same radius on every corner.
    #[inline]
    pub const fn circular(radius: f32) -> BorderRadius {
        BorderRadius {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

/// A rect with rounded corners. Zero radii make it a plain rect.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RoundedRect {
    /// The outer bounds.
    pub rect: Rect,
    /// The corner radii.
    pub radii: BorderRadius,
}

impl RoundedRect {
    /// A plain rect (zero radii).
    #[inline]
    pub const fn from_rect(rect: Rect) -> RoundedRect {
        RoundedRect {
            rect,
            radii: BorderRadius::ZERO,
        }
    }

    /// A rect with radii.
    #[inline]
    pub const fn new(rect: Rect, radii: BorderRadius) -> RoundedRect {
        RoundedRect { rect, radii }
    }
}

/// A clip shape, in the local coordinates in effect when it is pushed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Clip {
    /// Clip to a rect.
    Rect(Rect),
    /// Clip to a rounded rect.
    RoundedRect(RoundedRect),
}

/// A group of commands composited together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    /// Group opacity, nominally 0..=1. Default 1.
    pub opacity: f32,
    /// A color laid over the group's content (Clay's overlay color). Default `None`.
    pub overlay_color: Option<Color>,
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            opacity: 1.0,
            overlay_color: None,
        }
    }
}

/// Flutter's `BoxShadow`, cast by a rounded rect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    /// The shape casting the shadow.
    pub shape: RoundedRect,
    /// Shadow color.
    pub color: Color,
    /// How far the shadow is moved from the shape.
    pub offset: Vec2,
    /// Blur radius, as in Flutter (sigma = `blur_radius · 0.57735 + 0.5`).
    pub blur_radius: f32,
    /// How far the shape is inflated before blurring.
    pub spread_radius: f32,
}

/// How an image is sampled when scaled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageSampling {
    /// Nearest neighbor: crisp pixels.
    Nearest,
    /// Bilinear filtering.
    #[default]
    Linear,
}

/// An image drawn into a rect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageDraw {
    /// The image to draw.
    pub image: ImageId,
    /// Part of the image to draw, in image pixels. `None` draws the whole image.
    pub src: Option<Rect>,
    /// Where to draw it, in local coordinates.
    pub dest: Rect,
    /// How to sample when scaling.
    pub sampling: ImageSampling,
    /// Multiplies the image's alpha, nominally 0..=1.
    pub opacity: f32,
}

/// One positioned glyph, relative to its run's origin (baseline).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    /// Glyph id in the run's font.
    pub id: u32,
    /// Horizontal offset from the run's origin.
    pub x: f32,
    /// Vertical offset from the run's origin.
    pub y: f32,
}

/// A run of glyphs in one font, size and color.
///
/// The glyphs live in the Scene's glyph buffer: read them with [`Scene::glyphs`](crate::Scene::glyphs).
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRun {
    /// The font face.
    pub font: FontId,
    /// Font size in logical pixels (em size).
    pub font_size: f32,
    /// Fill color of the glyphs.
    pub color: Color,
    /// Baseline origin of the run, in local coordinates.
    pub origin: Point,
    /// First glyph in the Scene's glyph buffer.
    pub(crate) start: u32,
    /// Number of glyphs.
    pub(crate) len: u32,
}

/// A command a renderer may or may not know how to draw.
///
/// The bytes live in the Scene's data buffer: read them with
/// [`Scene::custom_data`](crate::Scene::custom_data).
#[derive(Clone, Debug, PartialEq)]
pub struct CustomDraw {
    /// What it draws; renderers look up their handler by kind.
    pub kind: CustomKind,
    /// The area it draws into, in local coordinates. Used for culling and damage.
    pub bounds: Rect,
    /// First byte in the Scene's data buffer.
    pub(crate) start: u32,
    /// Number of bytes.
    pub(crate) len: u32,
}

/// One recorded command.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Start clipping to a shape.
    PushClip(Clip),
    /// End the innermost clip.
    PopClip,
    /// Start applying a transform, after the current one.
    PushTransform(Affine),
    /// End the innermost transform.
    PopTransform,
    /// Start a group composited as one.
    PushLayer(Layer),
    /// End the innermost layer.
    PopLayer,
    /// Fill a (rounded) rect.
    Fill {
        /// The shape.
        shape: RoundedRect,
        /// Fill color.
        color: Color,
    },
    /// Stroke a (rounded) rect, the stroke lying inside the shape.
    Stroke {
        /// The shape; the stroke's outer edge.
        shape: RoundedRect,
        /// Stroke width, measured inward.
        width: f32,
        /// Stroke color.
        color: Color,
    },
    /// A box shadow.
    BoxShadow(BoxShadow),
    /// An image.
    Image(ImageDraw),
    /// A run of glyphs.
    GlyphRun(GlyphRun),
    /// A custom command.
    Custom(CustomDraw),
}

/// A command plus the element it was painted for and its z-index.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The element the command was painted for, if any.
    pub element: Option<ElementId>,
    /// Paint order among siblings in the same scope; lower first.
    pub z_index: i32,
    /// The command.
    pub command: Command,
}
