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
        edge_threshold: 0,
        interior: 0,
        edge: 0,
        loose_edge: 0,
        loose_edge_fraction: 0.0,
    };
}

impl Default for MatchTolerance {
    /// [`MatchTolerance::CROSS_BACKEND`].
    fn default() -> Self {
        todo!()
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
    let _ = (reference, actual, tolerance);
    todo!()
}
