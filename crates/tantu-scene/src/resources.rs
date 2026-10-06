//! [`Resources`]: the image and font data behind a Scene's [`ImageId`] and [`FontId`] handles.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

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
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .filter(|_| width > 0 && height > 0)
            .ok_or(ResourceError::InvalidImageSize { width, height })?;
        let pixels = pixels.into();
        if pixels.len() != expected {
            return Err(ResourceError::PixelDataLength {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(ImageData {
            width,
            height,
            pixels,
        })
    }

    /// Width in pixels, ≥ 1.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels, ≥ 1.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The pixels, `width · height · 4` bytes.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

impl fmt::Debug for ImageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageData")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
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
        let bytes = bytes.into();
        if bytes.is_empty() {
            return Err(ResourceError::EmptyFontData);
        }
        Ok(FontData { bytes, index })
    }

    /// The font file.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Face index within a collection; 0 for a single-face file.
    pub fn index(&self) -> u32 {
        self.index
    }
}

impl fmt::Debug for FontData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontData")
            .field("len", &self.bytes.len())
            .field("index", &self.index)
            .finish_non_exhaustive()
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
        match *self {
            ResourceError::InvalidImageSize { width, height } => {
                write!(f, "invalid image size {width}×{height}")
            }
            ResourceError::PixelDataLength { expected, actual } => {
                write!(f, "image pixel data is {actual} bytes, expected {expected}")
            }
            ResourceError::EmptyFontData => f.write_str("font data is empty"),
        }
    }
}

impl std::error::Error for ResourceError {}

/// Source of image and font handles: unique in the process, never reused (SCENE-RES-06). Holds
/// no data. Starts at 1 because handles are non-zero.
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

/// A new handle value, never returned before in this process.
fn next_handle() -> u64 {
    // Relaxed is enough: only uniqueness matters, not ordering with other memory.
    NEXT_HANDLE.fetch_add(1, Ordering::Relaxed)
}

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
        Resources::default()
    }

    /// Stores an image and returns its handle.
    pub fn add_image(&mut self, image: ImageData) -> ImageId {
        let id =
            ImageId::from_raw(next_handle()).expect("handle counter starts at 1 and never wraps");
        self.images.insert(id, image);
        self.revision += 1;
        id
    }

    /// The image for `id`, or `None` if it was removed or never added here.
    pub fn image(&self, id: ImageId) -> Option<&ImageData> {
        self.images.get(&id)
    }

    /// Removes and returns the image for `id`.
    pub fn remove_image(&mut self, id: ImageId) -> Option<ImageData> {
        let image = self.images.remove(&id)?;
        self.revision += 1;
        Some(image)
    }

    /// Stores a font face and returns its handle.
    pub fn add_font(&mut self, font: FontData) -> FontId {
        let id =
            FontId::from_raw(next_handle()).expect("handle counter starts at 1 and never wraps");
        self.fonts.insert(id, font);
        self.revision += 1;
        id
    }

    /// The font for `id`, or `None` if it was removed or never added here.
    pub fn font(&self, id: FontId) -> Option<&FontData> {
        self.fonts.get(&id)
    }

    /// Removes and returns the font for `id`.
    pub fn remove_font(&mut self, id: FontId) -> Option<FontData> {
        let font = self.fonts.remove(&id)?;
        self.revision += 1;
        Some(font)
    }

    /// Increases whenever an image or font is added or removed. Renderers compare it with the
    /// value they last saw to know when to re-check their caches.
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
