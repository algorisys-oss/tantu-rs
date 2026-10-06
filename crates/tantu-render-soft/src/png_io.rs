//! Lossless PNG encoding and decoding of straight-alpha RGBA8 images.

use std::fmt;
use std::io::Cursor;

use tantu_scene::ImageData;

/// Why PNG encoding or decoding failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum PngError {
    /// The bytes are not a PNG this crate can read.
    Decode(Box<dyn std::error::Error + Send + Sync>),
    /// The encoder failed.
    Encode(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for PngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PngError::Decode(e) => write!(f, "PNG decoding failed: {e}"),
            PngError::Encode(e) => write!(f, "PNG encoding failed: {e}"),
        }
    }
}

impl std::error::Error for PngError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PngError::Decode(e) | PngError::Encode(e) => Some(e.as_ref()),
        }
    }
}

/// Lossless PNG (8-bit RGBA, straight alpha) of `image`.
pub fn encode_png(image: &ImageData) -> Result<Vec<u8>, PngError> {
    let encode = |e: png::EncodingError| PngError::Encode(Box::new(e));
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, image.width(), image.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(encode)?;
    writer.write_image_data(image.pixels()).map_err(encode)?;
    writer.finish().map_err(encode)?;
    Ok(out)
}

/// Decodes a PNG into straight-alpha RGBA8.
///
/// Accepts 8-bit RGBA, RGB, gray and gray-alpha; other formats (16-bit, palette) are expanded to
/// RGBA8.
pub fn decode_png(bytes: &[u8]) -> Result<ImageData, PngError> {
    let decode = |e: png::DecodingError| PngError::Decode(Box::new(e));
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(decode)?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| PngError::Decode("PNG image too large".into()))?;
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
            return Err(PngError::Decode("indexed PNG was not expanded".into()));
        }
    };
    ImageData::rgba8(info.width, info.height, rgba).map_err(|e| PngError::Decode(Box::new(e)))
}
