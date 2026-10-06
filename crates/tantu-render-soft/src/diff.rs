//! [`diff_images`]: channel-by-channel comparison for golden tests.

use tantu_scene::ImageData;

/// How two images differ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageDiff {
    /// Pixels where some channel differs by more than the tolerance.
    pub differing_pixels: u64,
    /// The largest channel difference anywhere.
    pub max_channel_delta: u8,
}

/// Compares two images channel by channel. `None` if their sizes differ.
pub fn diff_images(a: &ImageData, b: &ImageData, tolerance: u8) -> Option<ImageDiff> {
    todo!()
}
