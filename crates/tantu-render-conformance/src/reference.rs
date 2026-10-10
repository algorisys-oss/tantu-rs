//! [`ReferenceScene`]: the reference Scenes and their embedded goldens.

use std::fmt;
use std::io::Cursor;
use std::path::PathBuf;

use tantu_scene::{ImageData, Resources, Scene};

use crate::compare::{ImageMatch, MatchTolerance, match_images};
use crate::scenes;

/// One reference Scene and its golden image.
pub struct ReferenceScene {
    pub(crate) name: &'static str,
    pub(crate) golden_png: &'static [u8],
    pub(crate) record: fn(&mut Resources) -> Scene,
}

impl fmt::Debug for ReferenceScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenceScene")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl ReferenceScene {
    /// Its name, which is also the golden's file stem (`shapes_and_strokes`, …).
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The target size in physical pixels to render it at.
    pub fn target_size(&self) -> (u32, u32) {
        let side = (scenes::LOGICAL_SIZE * scenes::SCALE_FACTOR) as u32;
        (side, side)
    }

    /// The scale factor to render it at.
    pub fn scale_factor(&self) -> f32 {
        scenes::SCALE_FACTOR
    }

    /// Records the Scene, with the resources it draws (images are added to a new `Resources`).
    pub fn record(&self) -> (Scene, Resources) {
        let mut resources = Resources::new();
        let scene = (self.record)(&mut resources);
        (scene, resources)
    }

    /// The golden PNG bytes, embedded in the crate.
    pub fn golden_png(&self) -> &'static [u8] {
        self.golden_png
    }

    /// The golden decoded as straight-alpha RGBA8.
    pub fn golden(&self) -> Result<ImageData, GoldenError> {
        decode_rgba8(self.golden_png)
    }

    /// Where the golden lives in the source tree, for writing it with `TANTU_UPDATE_GOLDENS=1`
    /// and for writing failed renderings next to it. Only meaningful in a checkout of the
    /// repository.
    pub fn golden_path(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("goldens")
            .join(format!("{}.png", self.name))
    }

    /// `match_images(golden, actual, tolerance)`, or [`GoldenError::SizeMismatch`] when the
    /// sizes differ.
    pub fn check(
        &self,
        actual: &ImageData,
        tolerance: &MatchTolerance,
    ) -> Result<ImageMatch, GoldenError> {
        let golden = self.golden()?;
        match_images(&golden, actual, tolerance).ok_or(GoldenError::SizeMismatch {
            golden: (golden.width(), golden.height()),
            actual: (actual.width(), actual.height()),
        })
    }
}

/// All reference Scenes, in a fixed order.
pub fn reference_scenes() -> &'static [ReferenceScene] {
    &scenes::REFERENCE_SCENES
}

/// The reference Scene called `name`, if there is one.
pub fn reference_scene(name: &str) -> Option<&'static ReferenceScene> {
    reference_scenes().iter().find(|r| r.name == name)
}

/// Why a golden couldn't be used.
#[derive(Debug)]
#[non_exhaustive]
pub enum GoldenError {
    /// The embedded PNG doesn't decode (it was replaced by something that isn't an 8-bit RGBA
    /// PNG).
    Decode(Box<dyn std::error::Error + Send + Sync>),
    /// The image checked has a different size from the golden.
    SizeMismatch {
        /// The golden's width and height.
        golden: (u32, u32),
        /// The checked image's width and height.
        actual: (u32, u32),
    },
}

impl fmt::Display for GoldenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GoldenError::Decode(e) => write!(f, "golden PNG doesn't decode: {e}"),
            GoldenError::SizeMismatch { golden, actual } => write!(
                f,
                "image is {}×{}, golden is {}×{}",
                actual.0, actual.1, golden.0, golden.1
            ),
        }
    }
}

impl std::error::Error for GoldenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GoldenError::Decode(e) => Some(e.as_ref()),
            GoldenError::SizeMismatch { .. } => None,
        }
    }
}

/// Decodes an 8-bit RGBA PNG (what `tantu-render-soft` writes) into `ImageData`.
fn decode_rgba8(bytes: &[u8]) -> Result<ImageData, GoldenError> {
    let decode = |e: png::DecodingError| GoldenError::Decode(Box::new(e));
    let mut reader = png::Decoder::new(Cursor::new(bytes))
        .read_info()
        .map_err(decode)?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| GoldenError::Decode("PNG image too large".into()))?;
    let mut pixels = vec![0; size];
    let info = reader.next_frame(&mut pixels).map_err(decode)?;
    if (info.color_type, info.bit_depth) != (png::ColorType::Rgba, png::BitDepth::Eight) {
        return Err(GoldenError::Decode(
            format!(
                "expected an 8-bit RGBA PNG, got {:?} at {:?}",
                info.color_type, info.bit_depth
            )
            .into(),
        ));
    }
    pixels.truncate(info.buffer_size());
    ImageData::rgba8(info.width, info.height, pixels).map_err(|e| GoldenError::Decode(Box::new(e)))
}
