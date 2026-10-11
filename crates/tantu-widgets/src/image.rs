//! [`Image`], [`BoxFit`] and [`decode_png`]. Spec: `docs/specs/widgets/image.md`.

use std::fmt;
use std::io::Cursor;

use tantu_view::core::{Point, Rect, Size, Vec2};
use tantu_view::layout::{Alignment, RenderImage};
use tantu_view::scene::{ImageData, ImageDraw, ImageSampling};
use tantu_view::{BuildCx, ElementId, IntoProp, Paint, PaintCx, Prop, Role, Semantics, View};

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
    let decode = |e: png::DecodingError| ImageError(e.to_string());
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(decode)?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| ImageError("PNG image too large".into()))?;
    let mut buf = vec![0; size];
    let info = reader.next_frame(&mut buf).map_err(decode)?;
    buf.truncate(info.buffer_size());
    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Indexed => {
            return Err(ImageError("indexed PNG was not expanded".into()));
        }
    };
    ImageData::rgba8(info.width, info.height, rgba).map_err(|e| ImageError(e.to_string()))
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
    label: Option<String>,
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
            label: None,
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

    /// The accessible name (WIDGETS-SEM-01).
    pub fn semantics_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
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
        let label = self.label;
        let natural = Size::new(self.data.width() as f32, self.data.height() as f32);
        let render = RenderImage {
            natural,
            width: self.width,
            height: self.height,
        };
        let id = cx.render(render, []);
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Image,
                name: label,
                ..Default::default()
            },
        );
        cx.set_paint(
            id,
            ImagePaint {
                data: self.data,
                fit: self.fit,
                alignment: self.alignment,
                opacity: 1.0,
            },
        );
        cx.bind(id, self.opacity, |e, opacity| {
            e.update_paint::<ImagePaint>(|p| p.opacity = opacity);
        });
        id
    }
}

/// Paints the image fitted into the element (WIDGETS-IMAGE-03, -04).
struct ImagePaint {
    data: ImageData,
    fit: BoxFit,
    alignment: Alignment,
    opacity: f32,
}

impl Paint for ImagePaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let Some(image) = cx.image_id(&self.data) else {
            return;
        };
        let input = Size::new(self.data.width() as f32, self.data.height() as f32);
        let output = cx.size();
        let (source, destination) = apply_box_fit(self.fit, input, output);
        if !(source.width > 0.0 && source.height > 0.0) {
            return;
        }
        let whole = Rect::from_origin_size(Point::ZERO, input);
        let src = inscribe(self.alignment, source, whole);
        let dest = inscribe(
            self.alignment,
            destination,
            Rect::from_origin_size(Point::ZERO, output),
        );
        cx.scene().image(ImageDraw {
            image,
            src: (src != whole).then_some(src),
            dest,
            sampling: ImageSampling::Linear,
            opacity: self.opacity,
        });
    }
}

/// A rect of `size` placed in `rect` by `alignment` (Flutter's `Alignment.inscribe`).
fn inscribe(alignment: Alignment, size: Size, rect: Rect) -> Rect {
    let free = Vec2::new(rect.width() - size.width, rect.height() - size.height);
    let offset = alignment.along_offset(free);
    Rect::from_origin_size(Point::new(rect.left + offset.x, rect.top + offset.y), size)
}

/// Flutter's `applyBoxFit`: the source size (in image pixels) and the destination size.
fn apply_box_fit(fit: BoxFit, input: Size, output: Size) -> (Size, Size) {
    let usable = |s: Size| s.width > 0.0 && s.height > 0.0;
    if !(usable(input) && usable(output)) {
        return (Size::ZERO, Size::ZERO);
    }
    let contain = || {
        if output.width / input.width <= output.height / input.height {
            (
                input,
                Size::new(output.width, input.height * output.width / input.width),
            )
        } else {
            (
                input,
                Size::new(input.width * output.height / input.height, output.height),
            )
        }
    };
    let cover = || {
        if output.width / output.height > input.width / input.height {
            (
                Size::new(input.width, input.width * output.height / output.width),
                output,
            )
        } else {
            (
                Size::new(input.height * output.width / output.height, input.height),
                output,
            )
        }
    };
    match fit {
        BoxFit::Fill => (input, output),
        BoxFit::Contain => contain(),
        BoxFit::Cover => cover(),
        BoxFit::FitWidth => {
            if output.width / output.height > input.width / input.height {
                cover()
            } else {
                contain()
            }
        }
        BoxFit::FitHeight => {
            if output.width / output.height <= input.width / input.height {
                cover()
            } else {
                contain()
            }
        }
        BoxFit::None => {
            let source = Size::new(
                input.width.min(output.width),
                input.height.min(output.height),
            );
            (source, source)
        }
        BoxFit::ScaleDown => {
            let mut destination = input;
            let aspect = input.width / input.height;
            if destination.height > output.height {
                destination = Size::new(output.height * aspect, output.height);
            }
            if destination.width > output.width {
                destination = Size::new(output.width, output.width / aspect);
            }
            (input, destination)
        }
    }
}
