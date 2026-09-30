//! Tests for `docs/specs/core/color.md`, one or more per rule CORE-COLOR-NN.

use tantu_core::Color;

const NAN: f32 = f32::NAN;
const INF: f32 = f32::INFINITY;

fn close(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

fn components(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

// ---- General ----------------------------------------------------------------------------

#[test]
fn core_color_01_default_and_constants() {
    assert_eq!(Color::default(), Color::TRANSPARENT);
    assert_eq!(components(Color::TRANSPARENT), [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(components(Color::BLACK), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(components(Color::WHITE), [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn core_color_02_no_panics_on_odd_input() {
    let odd = [NAN, INF, -INF, -1.0, 0.0, 0.5, 1.0, 2.0, f32::MAX, f32::MIN];
    for &x in &odd {
        for &y in &odd {
            let c = Color::new(x, y, x, y);
            let _ = (c.to_rgba8(), c.to_argb32(), c.with_alpha(x), c.clamp());
            let _ = (c.is_opaque(), c.is_transparent(), Color::lerp(c, c, x));
            let _ = (
                c.to_premultiplied(),
                c.to_linear(),
                Color::from_linear([x, y, x, y]),
            );
        }
    }
}

#[test]
fn core_color_02_nan_propagates() {
    let c = Color::new(NAN, 0.5, 0.5, 1.0);
    assert!(c.r.is_nan());
    assert!(c.with_alpha(0.5).r.is_nan());
    assert!(Color::WHITE.with_alpha(NAN).a.is_nan());
    assert!(Color::lerp(c, Color::BLACK, 0.5).r.is_nan());
    assert!(Color::lerp(Color::BLACK, Color::WHITE, NAN).g.is_nan());
    assert!(c.to_premultiplied()[0].is_nan());
    assert!(Color::new(0.5, 0.5, 0.5, NAN).to_premultiplied()[1].is_nan());
}

#[test]
fn core_color_03_stored_as_given() {
    assert_eq!(
        components(Color::new(-0.5, 2.0, INF, 1.5)),
        [-0.5, 2.0, INF, 1.5]
    );
    assert_eq!(Color::BLACK.with_alpha(3.0).a, 3.0);
    assert_eq!(Color::BLACK.with_alpha(-1.0).a, -1.0);
}

// ---- Constructors and 8-bit conversions -------------------------------------------------

#[test]
fn core_color_04_from_rgba8_and_rgb8() {
    assert_eq!(
        components(Color::from_rgba8(0, 51, 255, 102)),
        [0.0, 51.0 / 255.0, 1.0, 102.0 / 255.0]
    );
    for v in 0..=255u8 {
        let expected = v as f32 / 255.0;
        let c = Color::from_rgba8(v, v, v, v);
        assert_eq!(components(c), [expected; 4], "value {v}");
    }
    assert_eq!(Color::from_rgb8(1, 2, 3), Color::from_rgba8(1, 2, 3, 255));
    assert_eq!(Color::from_rgb8(0, 0, 0), Color::BLACK);
    assert_eq!(Color::from_rgb8(255, 255, 255), Color::WHITE);
}

#[test]
fn core_color_05_from_argb32_is_flutter_order() {
    assert_eq!(
        Color::from_argb32(0xFF2196F3),
        Color::from_rgba8(0x21, 0x96, 0xF3, 0xFF)
    );
    assert_eq!(
        Color::from_argb32(0x80123456),
        Color::from_rgba8(0x12, 0x34, 0x56, 0x80)
    );
    // no alpha byte means fully transparent, as in Flutter
    let c = Color::from_argb32(0x2196F3);
    assert_eq!(c.a, 0.0);
    assert!(c.is_transparent());
}

#[test]
fn core_color_06_to_rgba8_clamps_and_rounds() {
    assert_eq!(
        Color::new(0.0, 1.0, 0.5, 1.0).to_rgba8(),
        [0, 255, 128, 255]
    );
    // 0.5 * 255 = 127.5 rounds away from zero; 0.498 * 255 = 126.99 rounds to 127
    assert_eq!(
        Color::new(0.498, 0.5, 0.0, 0.0).to_rgba8(),
        [127, 128, 0, 0]
    );
    assert_eq!(Color::new(-1.0, 2.0, NAN, INF).to_rgba8(), [0, 255, 0, 255]);
    assert_eq!(Color::new(-INF, 0.0, 0.0, -0.0).to_rgba8(), [0, 0, 0, 0]);

    assert_eq!(
        Color::from_rgba8(0x21, 0x96, 0xF3, 0xFF).to_argb32(),
        0xFF2196F3
    );
    assert_eq!(Color::new(2.0, NAN, 0.0, 0.5).to_argb32(), 0x80FF0000);
}

#[test]
fn core_color_07_8bit_round_trips() {
    for v in 0..=255u8 {
        let w = 255 - v;
        assert_eq!(Color::from_rgba8(v, w, v, w).to_rgba8(), [v, w, v, w]);
    }
    for x in [
        0u32, 1, 0xFF2196F3, 0x00FFFFFF, 0xFFFFFFFF, 0x80808080, 0x7F01FE00,
    ] {
        assert_eq!(Color::from_argb32(x).to_argb32(), x, "{x:#010x}");
    }
    // a spread of u32 values
    let mut x: u32 = 0x1234_5678;
    for _ in 0..10_000 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        assert_eq!(Color::from_argb32(x).to_argb32(), x, "{x:#010x}");
    }
}

// ---- Operations -------------------------------------------------------------------------

#[test]
fn core_color_08_with_alpha() {
    let c = Color::new(0.1, 0.2, 0.3, 0.4);
    assert_eq!(c.with_alpha(0.9), Color::new(0.1, 0.2, 0.3, 0.9));
    assert_eq!(Color::WHITE.with_alpha(0.0), Color::new(1.0, 1.0, 1.0, 0.0));
}

#[test]
fn core_color_09_clamp() {
    assert_eq!(
        Color::new(-0.5, 1.5, NAN, 0.25).clamp(),
        Color::new(0.0, 1.0, 0.0, 0.25)
    );
    assert_eq!(
        Color::new(INF, -INF, 0.5, NAN).clamp(),
        Color::new(1.0, 0.0, 0.5, 0.0)
    );
    let inside = Color::new(0.0, 0.3, 0.7, 1.0);
    assert_eq!(inside.clamp(), inside);
}

#[test]
fn core_color_10_is_opaque_is_transparent() {
    assert!(Color::BLACK.is_opaque());
    assert!(!Color::BLACK.is_transparent());
    assert!(Color::TRANSPARENT.is_transparent());
    assert!(!Color::TRANSPARENT.is_opaque());
    assert!(Color::WHITE.with_alpha(1.5).is_opaque());
    assert!(Color::WHITE.with_alpha(-0.5).is_transparent());
    let half = Color::WHITE.with_alpha(0.5);
    assert!(!half.is_opaque() && !half.is_transparent());
    let nan = Color::WHITE.with_alpha(NAN);
    assert!(!nan.is_opaque() && !nan.is_transparent());
}

#[test]
fn core_color_11_lerp() {
    let a = Color::new(0.0, 0.2, 1.0, 0.0);
    let b = Color::new(1.0, 0.6, 0.0, 1.0);
    let mid = Color::lerp(a, b, 0.5);
    for (got, want) in components(mid).into_iter().zip([0.5, 0.4, 0.5, 0.5]) {
        assert!(close(got, want, 1e-6), "{mid:?}");
    }
    // exact at the endpoints for finite inputs
    let c = Color::new(0.123, 0.456, 0.789, 0.321);
    let d = Color::new(0.987, 0.654, 0.012, 0.9);
    assert_eq!(Color::lerp(c, d, 0.0), c);
    assert_eq!(Color::lerp(c, d, 1.0), d);
    // t is not clamped
    let over = Color::lerp(Color::BLACK, Color::WHITE, 1.5);
    assert_eq!(components(over), [1.5, 1.5, 1.5, 1.0]);
    let under = Color::lerp(Color::TRANSPARENT, Color::WHITE, -0.5);
    assert_eq!(components(under), [-0.5, -0.5, -0.5, -0.5]);
}

// ---- Renderer conversions ---------------------------------------------------------------

#[test]
fn core_color_12_to_premultiplied() {
    assert_eq!(
        Color::new(1.0, 0.5, 0.25, 0.5).to_premultiplied(),
        [0.5, 0.25, 0.125, 0.5]
    );
    assert_eq!(Color::WHITE.to_premultiplied(), [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(Color::TRANSPARENT.to_premultiplied(), [0.0; 4]);
    // no clamping
    assert_eq!(
        Color::new(2.0, -1.0, 1.0, 2.0).to_premultiplied(),
        [4.0, -2.0, 2.0, 2.0]
    );
}

fn srgb_to_linear_reference(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[test]
fn core_color_13_to_linear() {
    assert_eq!(Color::BLACK.to_linear(), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(Color::WHITE.to_linear(), [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(
        Color::new(-1.0, 2.0, NAN, 0.25).to_linear(),
        [0.0, 1.0, 0.0, 0.25]
    );
    assert_eq!(Color::new(0.0, 0.0, 0.0, 3.0).to_linear()[3], 1.0);
    assert_eq!(Color::new(0.0, 0.0, 0.0, NAN).to_linear()[3], 0.0);

    for v in 0..=255u8 {
        let c = Color::from_rgb8(v, v, v);
        let want = srgb_to_linear_reference(f64::from(v) / 255.0) as f32;
        let got = c.to_linear();
        assert!(close(got[0], want, 1e-6), "value {v}: {got:?} vs {want}");
        assert_eq!(got[0], got[1]);
        assert_eq!(got[1], got[2]);
    }
    // the linear segment and the curve
    assert!(close(
        Color::new(0.04, 0.0, 0.0, 1.0).to_linear()[0],
        0.04 / 12.92,
        1e-7
    ));
    assert!(close(
        Color::new(0.5, 0.0, 0.0, 1.0).to_linear()[0],
        0.214_041,
        1e-5
    ));
}

#[test]
fn core_color_14_from_linear() {
    assert_eq!(Color::from_linear([0.0, 0.0, 0.0, 1.0]), Color::BLACK);
    assert_eq!(Color::from_linear([1.0, 1.0, 1.0, 1.0]), Color::WHITE);
    assert_eq!(
        Color::from_linear([-1.0, 2.0, NAN, 1.5]),
        Color::new(0.0, 1.0, 0.0, 1.0)
    );
    assert!(close(
        Color::from_linear([0.003, 0.0, 0.0, 1.0]).r,
        12.92 * 0.003,
        1e-7
    ));
    assert!(close(
        Color::from_linear([0.214_041, 0.0, 0.0, 1.0]).r,
        0.5,
        1e-5
    ));

    // round trip within 1e-5 of the clamped color
    let mut x: u32 = 0xdead_beef;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        (x >> 8) as f32 / (1u32 << 24) as f32 * 1.4 - 0.2 // some values outside 0..=1
    };
    for _ in 0..10_000 {
        let c = Color::new(next(), next(), next(), next());
        let back = Color::from_linear(c.to_linear());
        for (got, want) in components(back).into_iter().zip(components(c.clamp())) {
            assert!(close(got, want, 1e-5), "{c:?} -> {back:?}");
        }
    }

    // every 8-bit value survives the round trip
    for v in 0..=255u8 {
        let c = Color::from_rgba8(v, v, v, v);
        assert_eq!(
            Color::from_linear(c.to_linear()).to_rgba8(),
            [v; 4],
            "value {v}"
        );
    }
}

// ---- Performance and allocation ---------------------------------------------------------

#[test]
fn core_color_15_plain_value_size() {
    assert_eq!(std::mem::size_of::<Color>(), 16);
}
