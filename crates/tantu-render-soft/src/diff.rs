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
    if (a.width(), a.height()) != (b.width(), b.height()) {
        return None;
    }
    let mut diff = ImageDiff::default();
    for (pa, pb) in a.pixels().chunks_exact(4).zip(b.pixels().chunks_exact(4)) {
        let delta = pa
            .iter()
            .zip(pb)
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        diff.max_channel_delta = diff.max_channel_delta.max(delta);
        if delta > tolerance {
            diff.differing_pixels += 1;
        }
    }
    Some(diff)
}
