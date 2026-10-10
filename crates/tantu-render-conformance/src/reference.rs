//! [`ReferenceScene`]: the reference Scenes and their embedded goldens.

use std::fmt;
use std::path::PathBuf;

use tantu_scene::{ImageData, Resources, Scene};

use crate::compare::{ImageMatch, MatchTolerance};

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
        todo!()
    }

    /// The target size in physical pixels to render it at.
    pub fn target_size(&self) -> (u32, u32) {
        todo!()
    }

    /// The scale factor to render it at.
    pub fn scale_factor(&self) -> f32 {
        todo!()
    }

    /// Records the Scene, with the resources it draws (images are added to a new `Resources`).
    pub fn record(&self) -> (Scene, Resources) {
        todo!()
    }

    /// The golden PNG bytes, embedded in the crate.
    pub fn golden_png(&self) -> &'static [u8] {
        todo!()
    }

    /// The golden decoded as straight-alpha RGBA8.
    pub fn golden(&self) -> Result<ImageData, GoldenError> {
        todo!()
    }

    /// Where the golden lives in the source tree, for writing it with `TANTU_UPDATE_GOLDENS=1`
    /// and for writing failed renderings next to it. Only meaningful in a checkout of the
    /// repository.
    pub fn golden_path(&self) -> PathBuf {
        todo!()
    }

    /// `match_images(golden, actual, tolerance)`, or [`GoldenError::SizeMismatch`] when the
    /// sizes differ.
    pub fn check(
        &self,
        actual: &ImageData,
        tolerance: &MatchTolerance,
    ) -> Result<ImageMatch, GoldenError> {
        let _ = (actual, tolerance);
        todo!()
    }
}

/// All reference Scenes, in a fixed order.
pub fn reference_scenes() -> &'static [ReferenceScene] {
    todo!()
}

/// The reference Scene called `name`, if there is one.
pub fn reference_scene(name: &str) -> Option<&'static ReferenceScene> {
    let _ = name;
    todo!()
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
