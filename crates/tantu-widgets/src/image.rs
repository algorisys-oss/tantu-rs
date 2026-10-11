//! [`Image`], [`BoxFit`] and [`decode_png`]. Spec: `docs/specs/widgets/image.md`.

use std::fmt;

use tantu_view::layout::Alignment;
use tantu_view::scene::ImageData;
use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

/// Why an image couldn't be decoded.
#[derive(Debug)]
pub struct ImageError(String);

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ImageError {}

/// Decodes a PNG into straight-alpha RGBA8 (any color type, 16-bit normalized to 8).
pub fn decode_png(bytes: &[u8]) -> Result<ImageData, ImageError> {
    let _ = bytes;
    todo!("WIDGETS-IMAGE-01")
}

/// How an image is inscribed in its box (Flutter's `BoxFit`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoxFit {
    /// Stretch to fill the box, ignoring the aspect ratio.
    Fill,
    /// As large as possible inside the box, keeping the aspect ratio.
    Contain,
    /// As small as possible while covering the box, keeping the aspect ratio (cropped).
    Cover,
    /// The box's width, keeping the aspect ratio (cropped or letterboxed vertically).
    FitWidth,
    /// The box's height, keeping the aspect ratio.
    FitHeight,
    /// The natural size, cropped to the box.
    None,
    /// The natural size, scaled down (never up) to fit the box.
    #[default]
    ScaleDown,
}

/// Shows a decoded image (Flutter's `Image`).
pub struct Image {
    data: ImageData,
    width: Option<f32>,
    height: Option<f32>,
    fit: BoxFit,
    alignment: Alignment,
    opacity: Prop<f32>,
}

impl Image {
    /// `data` at its natural size, `BoxFit::ScaleDown`, centered, opaque.
    pub fn new(data: ImageData) -> Self {
        Image {
            data,
            width: None,
            height: None,
            fit: BoxFit::default(),
            alignment: Alignment::CENTER,
            opacity: Prop::Value(1.0),
        }
    }

    /// An explicit width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// An explicit height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// An explicit width and height.
    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    /// How the image is inscribed in its box.
    pub fn fit(mut self, fit: BoxFit) -> Self {
        self.fit = fit;
        self
    }

    /// Where the image sits in its box when it doesn't fill it.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Multiplies the image's alpha (default 1).
    pub fn opacity(mut self, opacity: impl IntoProp<f32>) -> Self {
        self.opacity = opacity.into_prop();
        self
    }
}

impl View for Image {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.data,
            self.width,
            self.height,
            self.fit,
            self.alignment,
            self.opacity,
        );
        todo!("WIDGETS-IMAGE-02")
    }
}
