//! [`Resources`]: the image and font data behind a Scene's [`ImageId`] and [`FontId`] handles.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::handles::{FontId, ImageId};

/// RGBA8 pixels, sRGB-encoded, straight (not premultiplied) alpha, rows top to bottom, no
/// padding. Cloning shares the pixel buffer.
#[derive(Clone, PartialEq, Eq)]
pub struct ImageData {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl ImageData {
    /// Checks the size and the buffer length (`width · height · 4`).
    pub fn rgba8(
        width: u32,
        height: u32,
        pixels: impl Into<Arc<[u8]>>,
    ) -> Result<ImageData, ResourceError> {
        todo!()
    }

    /// Width in pixels, ≥ 1.
    pub fn width(&self) -> u32 {
        todo!()
    }

    /// Height in pixels, ≥ 1.
    pub fn height(&self) -> u32 {
        todo!()
    }

    /// The pixels, `width · height · 4` bytes.
    pub fn pixels(&self) -> &[u8] {
        todo!()
    }
}

impl fmt::Debug for ImageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// A font file (TTF/OTF, or a collection) and the index of the face to use. Cloning shares the
/// bytes. The bytes are not parsed here.
#[derive(Clone, PartialEq, Eq)]
pub struct FontData {
    bytes: Arc<[u8]>,
    index: u32,
}

impl FontData {
    /// Fails only for empty `bytes`.
    pub fn new(bytes: impl Into<Arc<[u8]>>, index: u32) -> Result<FontData, ResourceError> {
        todo!()
    }

    /// The font file.
    pub fn bytes(&self) -> &[u8] {
        todo!()
    }

    /// Face index within a collection; 0 for a single-face file.
    pub fn index(&self) -> u32 {
        todo!()
    }
}

impl fmt::Debug for FontData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Why image or font data was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceError {
    /// Width or height is 0, or `width · height · 4` overflows `usize`.
    InvalidImageSize {
        /// The width given.
        width: u32,
        /// The height given.
        height: u32,
    },
    /// The pixel buffer is not `width · height · 4` bytes.
    PixelDataLength {
        /// `width · height · 4`.
        expected: usize,
        /// The buffer's length.
        actual: usize,
    },
    /// The font file is empty.
    EmptyFontData,
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for ResourceError {}

/// The images and fonts a Scene's handles refer to.
///
/// One per app, shared by its windows' renderers (wrap it in `Arc`/`RwLock` as the app runner
/// needs). Handles are unique in the process and never reused.
#[derive(Clone, Debug, Default)]
pub struct Resources {
    images: HashMap<ImageId, ImageData>,
    fonts: HashMap<FontId, FontData>,
    revision: u64,
}

impl Resources {
    /// No resources, revision 0.
    pub fn new() -> Resources {
        todo!()
    }

    /// Stores an image and returns its handle.
    pub fn add_image(&mut self, image: ImageData) -> ImageId {
        todo!()
    }

    /// The image for `id`, or `None` if it was removed or never added here.
    pub fn image(&self, id: ImageId) -> Option<&ImageData> {
        todo!()
    }

    /// Removes and returns the image for `id`.
    pub fn remove_image(&mut self, id: ImageId) -> Option<ImageData> {
        todo!()
    }

    /// Stores a font face and returns its handle.
    pub fn add_font(&mut self, font: FontData) -> FontId {
        todo!()
    }

    /// The font for `id`, or `None` if it was removed or never added here.
    pub fn font(&self, id: FontId) -> Option<&FontData> {
        todo!()
    }

    /// Removes and returns the font for `id`.
    pub fn remove_font(&mut self, id: FontId) -> Option<FontData> {
        todo!()
    }

    /// Increases whenever an image or font is added or removed. Renderers compare it with the
    /// value they last saw to know when to re-check their caches.
    pub fn revision(&self) -> u64 {
        todo!()
    }
}
