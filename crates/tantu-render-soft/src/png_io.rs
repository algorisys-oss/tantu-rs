//! Lossless PNG encoding and decoding of straight-alpha RGBA8 images.

use std::fmt;

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
        todo!()
    }
}

impl std::error::Error for PngError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        todo!()
    }
}

/// Lossless PNG (8-bit RGBA, straight alpha) of `image`.
pub fn encode_png(image: &ImageData) -> Result<Vec<u8>, PngError> {
    todo!()
}

/// Decodes a PNG into straight-alpha RGBA8.
///
/// Accepts 8-bit RGBA, RGB, gray and gray-alpha; other formats (16-bit, palette) are expanded to
/// RGBA8.
pub fn decode_png(bytes: &[u8]) -> Result<ImageData, PngError> {
    todo!()
}
