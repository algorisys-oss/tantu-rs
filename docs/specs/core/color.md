# Color

- **Status:** Agreed
- **Crate:** `tantu-core`
- **Plan item:** Phase 0, "`tantu-core` → `Color`"
- **Related:** [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md) (Scene),
  [geometry](geometry.md) (same conventions for NaN and for values stored as given)

## Purpose

One color type used by every Scene command, every theme token and every widget property that
takes a color. App developers write colors, widget and theme authors pass them through and
interpolate them for animations, and renderer authors convert them to what their backend wants:
8-bit values for tiny-skia and PNG goldens, premultiplied or linear floats for wgpu.

## Scope

In scope:

- `Color`: sRGB-encoded components with straight (not premultiplied) alpha, stored as `f32`.
- Constructors from floats, 8-bit components and a packed `0xAARRGGBB` value (Flutter's
  `Color(0xFF2196F3)` form).
- Conversions renderers need: to 8-bit, to premultiplied, to and from linear light.
- `lerp` for animations and theme transitions, `with_alpha`, `clamp`, opacity tests.

Out of scope:

- Named palettes (`Colors.blue` and friends). They belong to `tantu-theme`.
- HSL/HSV/OKLCH, wide-gamut color spaces (Display P3), HDR.
- Parsing CSS color strings.
- Gradients, blend modes and color filters. They are Scene commands (Phase 1).
- A GPU-layout (`#[repr(C)]`, `bytemuck`) guarantee. Renderers copy into their own vertex types.

## Public API

Module `tantu_core::color`, re-exported at the crate root as `tantu_core::Color`.

```rust
/// An sRGB color with straight (not premultiplied) alpha.
///
/// Components are nominally in 0.0..=1.0 but are stored as given; see `clamp` and `to_rgba8`.
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
    pub const TRANSPARENT: Color;
    /// Opaque black, (0, 0, 0, 1).
    pub const BLACK: Color;
    /// Opaque white, (1, 1, 1, 1).
    pub const WHITE: Color;

    /// From sRGB-encoded components and straight alpha, stored as given.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color;
    /// From 8-bit sRGB components and 8-bit alpha (each value / 255).
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Color;
    /// From 8-bit sRGB components, opaque.
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Color;
    /// From a packed `0xAARRGGBB` value, as in Flutter's `Color(0xFF2196F3)`.
    pub const fn from_argb32(argb: u32) -> Color;

    /// 8-bit `[r, g, b, a]`: each component clamped to 0..=1 (NaN becomes 0), times 255, rounded.
    pub fn to_rgba8(self) -> [u8; 4];
    /// Packed `0xAARRGGBB`, from the same 8-bit values as `to_rgba8`.
    pub fn to_argb32(self) -> u32;

    /// The same color with alpha replaced by `a`.
    pub const fn with_alpha(self, a: f32) -> Color;
    /// Each component clamped to 0..=1; NaN becomes 0.
    pub fn clamp(self) -> Color;
    /// True if alpha ≥ 1.
    pub fn is_opaque(self) -> bool;
    /// True if alpha ≤ 0.
    pub fn is_transparent(self) -> bool;

    /// Straight interpolation from `a` to `b` in sRGB space, all four components. `t` is not
    /// clamped, so values outside 0..=1 extrapolate.
    pub fn lerp(a: Color, b: Color, t: f32) -> Color;

    /// `[r·a, g·a, b·a, a]`, sRGB-encoded premultiplied components.
    pub fn to_premultiplied(self) -> [f32; 4];
    /// `[r, g, b, a]` with the sRGB transfer function removed from r, g and b (linear light);
    /// alpha unchanged. Components are clamped to 0..=1 first.
    pub fn to_linear(self) -> [f32; 4];
    /// The inverse of `to_linear`: from linear-light `[r, g, b, a]`, clamped to 0..=1 first.
    pub fn from_linear(rgba: [f32; 4]) -> Color;
}

impl Default for Color { /* Color::TRANSPARENT */ }
```

## Behavior

General

- **CORE-COLOR-01:** `Color::default()` is `TRANSPARENT`. `TRANSPARENT` is (0, 0, 0, 0), `BLACK` is
  (0, 0, 0, 1) and `WHITE` is (1, 1, 1, 1).
- **CORE-COLOR-02:** No function panics, whatever the input: NaN, infinities, negative or > 1
  components. NaN propagates through `new`, `with_alpha`, `lerp` and `to_premultiplied`.
- **CORE-COLOR-03:** `new` and `with_alpha` store components as given: nothing is clamped. Only
  `clamp`, `to_rgba8`, `to_argb32`, `to_linear` and `from_linear` clamp.

Constructors and 8-bit conversions

- **CORE-COLOR-04:** `from_rgba8(r, g, b, a)` sets each component to `value as f32 / 255.0`.
  `from_rgb8(r, g, b)` is `from_rgba8(r, g, b, 255)`.
- **CORE-COLOR-05:** `from_argb32(0xAARRGGBB)` is `from_rgba8(0xRR, 0xGG, 0xBB, 0xAA)`. So
  `from_argb32(0xFF2196F3)` is opaque (0x21, 0x96, 0xF3), and `from_argb32(0x2196F3)` is fully
  transparent, as in Flutter.
- **CORE-COLOR-06:** `to_rgba8` clamps each component to 0..=1 (NaN becomes 0), multiplies by 255
  and rounds to the nearest integer, halves away from zero. `to_argb32` packs the same four bytes
  as `0xAARRGGBB`.
- **CORE-COLOR-07:** 8-bit round trips are exact: `from_rgba8(r, g, b, a).to_rgba8() == [r, g, b, a]`
  and `from_argb32(x).to_argb32() == x`, for every byte value and every `u32`.

Operations

- **CORE-COLOR-08:** `with_alpha(x)` replaces alpha with `x` and leaves r, g and b unchanged.
- **CORE-COLOR-09:** `clamp` clamps each of the four components to 0..=1; NaN becomes 0.
- **CORE-COLOR-10:** `is_opaque` is `a >= 1.0` and `is_transparent` is `a <= 0.0`. With a NaN alpha
  both are false. (Renderers may skip transparent draws and treat opaque ones as occluding.)
- **CORE-COLOR-11:** `lerp(a, b, t)` is `a·(1 − t) + b·t` per component, alpha included, in sRGB
  space with straight alpha (Flutter's `Color.lerp`, without its clamping). For finite inputs
  `t = 0` gives `a` and `t = 1` gives `b` exactly. `t` outside 0..=1 extrapolates and is not
  clamped.

Renderer conversions

- **CORE-COLOR-12:** `to_premultiplied` is `[r·a, g·a, b·a, a]`, with no clamping.
- **CORE-COLOR-13:** `to_linear` clamps each component to 0..=1 (NaN becomes 0), then applies the
  sRGB decoding function to r, g and b: `c / 12.92` if `c ≤ 0.04045`, else
  `((c + 0.055) / 1.055)^2.4`. Alpha is only clamped. 0 and 1 map to exactly 0 and 1.
- **CORE-COLOR-14:** `from_linear` clamps each component to 0..=1 (NaN becomes 0), then applies the
  sRGB encoding function to r, g and b: `12.92·c` if `c ≤ 0.0031308`, else
  `1.055·c^(1/2.4) − 0.055`. Alpha is only clamped. 0 and 1 map to exactly 0 and 1, so black and
  white survive the round trip unchanged. `Color::from_linear(c.to_linear())` equals
  `c.clamp()` within 1e-5 per component, and for every 8-bit color the round trip gives back the
  same `to_rgba8` bytes.

## Performance and allocation

- **CORE-COLOR-15:** `Color` is a plain value: `size_of::<Color>()` is 16 bytes.
- No function allocates. All are `#[inline]`; `new`, `from_rgba8`, `from_rgb8`, `from_argb32` and
  `with_alpha` are `const` so themes can declare colors as constants.

## Open questions

Resolved (2026-09-30, the proposals were accepted):

1. **`f32` or `u8` storage?** `f32` (16 bytes). Animations and theme transitions interpolate
   without banding, wgpu takes floats, and it leaves room for wide gamut later (Flutter itself
   moved from a packed 32-bit value to float components in 3.27). tiny-skia and PNG goldens
   convert with `to_rgba8`.
2. **Packed hex order.** Flutter's `0xAARRGGBB` only, named `from_argb32` so the order is in the
   name. CSS's `#RRGGBBAA` order is left out to avoid two easily confused forms; a `from_rgba32`
   can be added later if wanted.
3. **Linear conversion here or in the renderers?** Here. Every GPU backend needs the same function,
   and golden tests should agree across backends. It returns `[f32; 4]` rather than a separate
   `LinearColor` type, to keep "a `Color` is always sRGB" simple.
4. **Exact endpoints for `from_linear` (CORE-COLOR-14).** Found while implementing: evaluated in
   `f32`, the encoding function maps 1.0 to 0.99999994, so white would not survive the linear
   round trip exactly. Rule 14 now promises exact 0 and 1, like rule 13 does for `to_linear`.
