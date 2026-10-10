//! [`match_images`]: the edge-aware comparison of a rendering with a reference.

use tantu_scene::ImageData;

/// Tolerances for comparing a rendering with a reference produced by another backend.
///
/// Pixel values are compared premultiplied (see [`match_images`]). A pixel is an *edge pixel*
/// when the reference changes by more than `edge_threshold` somewhere in its 3 × 3
/// neighbourhood; every other pixel is an *interior pixel*.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchTolerance {
    /// How much the reference must change between neighbours to make an edge.
    pub edge_threshold: u8,
    /// Largest difference allowed at an interior pixel.
    pub interior: u8,
    /// Largest difference allowed at an edge pixel.
    pub edge: u8,
    /// An edge pixel differing by more than this counts as a *loose* edge pixel.
    pub loose_edge: u8,
    /// Largest fraction (0..=1) of the edge pixels that may be loose. NaN or negative counts as
    /// 0, above 1 as 1.
    pub loose_edge_fraction: f32,
}

impl MatchTolerance {
    /// The tolerance for comparing a backend with the software goldens, from the measurements
    /// in the spec: edge threshold 8, interior 10, edge 96, loose edge 16, loose edge
    /// fraction 0.05.
    pub const CROSS_BACKEND: MatchTolerance = MatchTolerance {
        edge_threshold: 8,
        interior: 10,
        edge: 96,
        loose_edge: 16,
        loose_edge_fraction: 0.05,
    };
}

impl Default for MatchTolerance {
    /// [`MatchTolerance::CROSS_BACKEND`].
    fn default() -> Self {
        MatchTolerance::CROSS_BACKEND
    }
}

/// The result of [`match_images`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageMatch {
    /// Whether the images match within the tolerance.
    pub passed: bool,
    /// Number of edge pixels (taken from the reference).
    pub edge_pixels: u64,
    /// Largest difference at an interior pixel.
    pub max_interior_delta: u8,
    /// Interior pixels differing by more than `interior`.
    pub interior_failures: u64,
    /// Largest difference at an edge pixel.
    pub max_edge_delta: u8,
    /// Edge pixels differing by more than `edge`.
    pub edge_failures: u64,
    /// Edge pixels differing by more than `loose_edge`.
    pub loose_edge_pixels: u64,
    /// The most loose edge pixels allowed: `floor(loose_edge_fraction × edge_pixels)`.
    pub loose_edge_allowance: u64,
}

/// Compares `actual` with `reference` using `tolerance`. `None` if their sizes differ.
///
/// Pixels are compared premultiplied (each color channel times alpha / 255, rounded); the
/// difference of two pixels is the largest absolute difference over the four premultiplied
/// channels. Edges are found in `reference` only, so the comparison is not symmetric: pass the
/// trusted image (a golden) as `reference`.
pub fn match_images(
    reference: &ImageData,
    actual: &ImageData,
    tolerance: &MatchTolerance,
) -> Option<ImageMatch> {
    let (width, height) = (reference.width() as usize, reference.height() as usize);
    if (actual.width(), actual.height()) != (reference.width(), reference.height()) {
        return None;
    }
    let premultiplied: Vec<[u8; 4]> = reference
        .pixels()
        .chunks_exact(4)
        .map(premultiply)
        .collect();

    let mut result = ImageMatch::default();
    for (i, actual_px) in actual.pixels().chunks_exact(4).enumerate() {
        let (x, y) = (i % width, i / width);
        let here = premultiplied[i];
        let delta = difference(here, premultiply(actual_px));
        let is_edge = (y.saturating_sub(1)..=(y + 1).min(height - 1)).any(|ny| {
            (x.saturating_sub(1)..=(x + 1).min(width - 1)).any(|nx| {
                difference(here, premultiplied[ny * width + nx]) > tolerance.edge_threshold
            })
        });
        if is_edge {
            result.edge_pixels += 1;
            result.max_edge_delta = result.max_edge_delta.max(delta);
            result.edge_failures += u64::from(delta > tolerance.edge);
            result.loose_edge_pixels += u64::from(delta > tolerance.loose_edge);
        } else {
            result.max_interior_delta = result.max_interior_delta.max(delta);
            result.interior_failures += u64::from(delta > tolerance.interior);
        }
    }

    // NaN fails both comparisons and becomes 0.
    let fraction = if tolerance.loose_edge_fraction > 0.0 {
        f64::from(tolerance.loose_edge_fraction.min(1.0))
    } else {
        0.0
    };
    // Exact for any pixel count an image can have (well below 2^53).
    result.loose_edge_allowance = (fraction * result.edge_pixels as f64).floor() as u64;
    result.passed = result.interior_failures == 0
        && result.edge_failures == 0
        && result.loose_edge_pixels <= result.loose_edge_allowance;
    Some(result)
}

/// A straight-alpha RGBA8 pixel premultiplied, each color channel rounded to nearest.
fn premultiply(px: &[u8]) -> [u8; 4] {
    let a = u32::from(px[3]);
    // Round half up; at most 255 · 255 + 127, so it fits, and the quotient is at most 255.
    let m = |c: u8| ((u32::from(c) * a + 127) / 255) as u8;
    [m(px[0]), m(px[1]), m(px[2]), px[3]]
}

/// The largest channel difference between two pixels.
fn difference(a: [u8; 4], b: [u8; 4]) -> u8 {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(y))
        .max()
        .unwrap_or(0)
}
