//! [`Color`]: sRGB with straight (not premultiplied) alpha, stored as `f32`.
//!
//! Components are stored as given, like the [`geometry`](crate::geometry) types: nothing panics
//! and NaN propagates. Only [`Color::clamp`] and the conversions to 8-bit and linear light clamp.
//!
//! Spec: `docs/specs/core/color.md`.

/// An sRGB color with straight (not premultiplied) alpha.
///
/// Components are nominally in `0.0..=1.0` but are stored as given; see [`Color::clamp`] and
/// [`Color::to_rgba8`].
///
/// ```
/// use tantu_core::Color;
///
/// let blue = Color::from_argb32(0xFF2196F3);
/// assert_eq!(blue.to_rgba8(), [0x21, 0x96, 0xF3, 0xFF]);
/// assert_eq!(blue.with_alpha(0.0).to_argb32(), 0x002196F3);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red, sRGB-encoded.
    pub r: f32,
    /// Green, sRGB-encoded.
    pub g: f32,
    /// Blue, sRGB-encoded.
    pub b: f32,
    /// Alpha (opacity): 0.0 is fully transparent, 1.0 fully opaque.
    pub a: f32,
}

impl Color {
    /// Fully transparent black, (0, 0, 0, 0).
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
    /// Opaque black, (0, 0, 0, 1).
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    /// Opaque white, (1, 1, 1, 1).
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

    /// From sRGB-encoded components and straight alpha, stored as given.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    /// From 8-bit sRGB components and 8-bit alpha (each value / 255).
    #[inline]
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// From 8-bit sRGB components, opaque.
    #[inline]
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba8(r, g, b, 255)
    }

    /// From a packed `0xAARRGGBB` value, as in Flutter's `Color(0xFF2196F3)`. A zero alpha byte
    /// gives a fully transparent color.
    #[inline]
    pub const fn from_argb32(argb: u32) -> Color {
        let [a, r, g, b] = argb.to_be_bytes();
        Color::from_rgba8(r, g, b, a)
    }

    /// 8-bit `[r, g, b, a]`: each component clamped to 0..=1 (NaN becomes 0), times 255,
    /// rounded to nearest.
    #[inline]
    pub fn to_rgba8(self) -> [u8; 4] {
        let c = self.clamp();
        [c.r, c.g, c.b, c.a].map(|v| (v * 255.0).round() as u8)
    }

    /// Packed `0xAARRGGBB`, from the same 8-bit values as [`Color::to_rgba8`].
    #[inline]
    pub fn to_argb32(self) -> u32 {
        let [r, g, b, a] = self.to_rgba8();
        u32::from_be_bytes([a, r, g, b])
    }

    /// The same color with alpha replaced by `a`.
    #[inline]
    pub const fn with_alpha(self, a: f32) -> Color {
        Color::new(self.r, self.g, self.b, a)
    }

    /// Each component clamped to 0..=1; NaN becomes 0.
    #[inline]
    pub fn clamp(self) -> Color {
        Color::new(
            clamp01(self.r),
            clamp01(self.g),
            clamp01(self.b),
            clamp01(self.a),
        )
    }

    /// True if alpha ≥ 1.
    #[inline]
    pub fn is_opaque(self) -> bool {
        self.a >= 1.0
    }

    /// True if alpha ≤ 0.
    #[inline]
    pub fn is_transparent(self) -> bool {
        self.a <= 0.0
    }

    /// Straight interpolation from `a` to `b` in sRGB space, all four components. `t` is not
    /// clamped, so values outside 0..=1 extrapolate.
    #[inline]
    pub fn lerp(a: Color, b: Color, t: f32) -> Color {
        // This form (not `a + (b - a)·t`) is exact at both ends for finite inputs.
        let s = 1.0 - t;
        Color::new(
            a.r * s + b.r * t,
            a.g * s + b.g * t,
            a.b * s + b.b * t,
            a.a * s + b.a * t,
        )
    }

    /// `[r·a, g·a, b·a, a]`: sRGB-encoded premultiplied components, not clamped.
    #[inline]
    pub fn to_premultiplied(self) -> [f32; 4] {
        [self.r * self.a, self.g * self.a, self.b * self.a, self.a]
    }

    /// `[r, g, b, a]` with the sRGB transfer function removed from r, g and b (linear light);
    /// alpha unchanged. Components are clamped to 0..=1 first.
    #[inline]
    pub fn to_linear(self) -> [f32; 4] {
        let c = self.clamp();
        [decode(c.r), decode(c.g), decode(c.b), c.a]
    }

    /// The inverse of [`Color::to_linear`]: from linear-light `[r, g, b, a]`, clamped to 0..=1
    /// first.
    #[inline]
    pub fn from_linear(rgba: [f32; 4]) -> Color {
        let [r, g, b, a] = rgba.map(clamp01);
        Color::new(encode(r), encode(g), encode(b), a)
    }
}

impl Default for Color {
    #[inline]
    fn default() -> Self {
        Color::TRANSPARENT
    }
}

/// Clamps to 0..=1; NaN becomes 0.
#[inline]
fn clamp01(v: f32) -> f32 {
    // `>=` is false for NaN, so NaN takes the else branch.
    if v >= 0.0 { v.min(1.0) } else { 0.0 }
}

/// sRGB decoding (electro-optical transfer function) for one component in 0..=1.
#[inline]
fn decode(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB encoding, the inverse of [`decode`], for one component in 0..=1.
#[inline]
fn encode(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        // In f64, so that 1.0 encodes to exactly 1.0 (f32 gives 0.99999994).
        (1.055 * f64::from(c).powf(1.0 / 2.4) - 0.055) as f32
    }
}
