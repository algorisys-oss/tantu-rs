//! Tests for `docs/specs/core/geometry.md`, one or more per rule CORE-GEOM-NN.

use std::f32::consts::{FRAC_PI_2, PI};

use tantu_core::{Affine, EdgeInsets, Point, Rect, Size, Vec2};

const NAN: f32 = f32::NAN;
const INF: f32 = f32::INFINITY;

fn close(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

fn assert_point_close(a: Point, b: Point, tol: f32) {
    assert!(
        close(a.x, b.x, tol) && close(a.y, b.y, tol),
        "{a:?} != {b:?} (tol {tol})"
    );
}

fn assert_coeffs_close(a: [f32; 6], b: [f32; 6], tol: f32) {
    for i in 0..6 {
        assert!(close(a[i], b[i], tol), "{a:?} != {b:?} at {i} (tol {tol})");
    }
}

/// Deterministic pseudo-random numbers (xorshift32), so tests are reproducible.
struct Rng(u32);

impl Rng {
    fn next_unit(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_unit()
    }
}

// ---- General ----------------------------------------------------------------------------

#[test]
fn core_geom_01_defaults() {
    assert_eq!(Point::default(), Point::new(0.0, 0.0));
    assert_eq!(Point::default(), Point::ZERO);
    assert_eq!(Vec2::default(), Vec2::new(0.0, 0.0));
    assert_eq!(Vec2::default(), Vec2::ZERO);
    assert_eq!(Size::default(), Size::new(0.0, 0.0));
    assert_eq!(Size::default(), Size::ZERO);
    assert_eq!(Rect::default(), Rect::from_ltrb(0.0, 0.0, 0.0, 0.0));
    assert_eq!(Rect::default(), Rect::ZERO);
    assert_eq!(EdgeInsets::default(), EdgeInsets::all(0.0));
    assert_eq!(EdgeInsets::default(), EdgeInsets::ZERO);
    assert_eq!(Affine::default(), Affine::IDENTITY);
}

#[test]
fn core_geom_02_no_panics_on_odd_input() {
    let odd = [
        NAN,
        INF,
        -INF,
        -1.0,
        0.0,
        -0.0,
        f32::MAX,
        f32::MIN,
        f32::MIN_POSITIVE,
    ];
    for &a in &odd {
        for &b in &odd {
            let p = Point::new(a, b);
            let v = Vec2::new(a, b);
            let s = Size::new(a, b);
            let r = Rect::from_ltrb(a, b, b, a);
            let i = EdgeInsets::from_ltrb(a, b, a, b);
            let t = Affine::new([a, b, b, a, a, b]);

            let _ = (p + v, p - v, p - p, p.to_vec2(), p.is_finite());
            let mut q = p;
            q += v;
            q -= v;
            let _ = (
                v + v,
                v - v,
                -v,
                v * a,
                v.length(),
                v.to_point(),
                v.is_finite(),
            );
            let mut w = v;
            w += v;
            w -= v;
            let _ = (s.is_empty(), s.is_finite(), s.min(s), s.max(s), s.to_rect());
            let _ = (Rect::from_ltwh(a, b, a, b), Rect::from_origin_size(p, s));
            let _ = (r.width(), r.height(), r.size(), r.origin(), r.center());
            let _ = (r.is_empty(), r.is_finite(), r.contains(p), r.overlaps(r));
            let _ = (
                r.intersect(r),
                r.union(r),
                r.translate(v),
                r.inflate(a),
                r.deflate(a),
            );
            let _ = (EdgeInsets::all(a), EdgeInsets::symmetric(a, b), i + i);
            let _ = (
                i.horizontal(),
                i.vertical(),
                i.deflate_size(s),
                i.inflate_size(s),
            );
            let _ = (i.deflate_rect(r), i.inflate_rect(r));
            let _ = (
                Affine::translate(v),
                Affine::scale(a),
                Affine::scale_non_uniform(a, b),
            );
            let _ = (Affine::rotate(a), t.coeffs(), t.determinant(), t.inverse());
            let _ = (
                t * t,
                t * p,
                t.transform_vec(v),
                t.transform_rect_bbox(r),
                t.is_finite(),
            );
        }
    }
}

#[test]
fn core_geom_02_nan_propagates() {
    assert!((Point::new(NAN, 1.0) + Vec2::new(1.0, 1.0)).x.is_nan());
    assert!((Vec2::new(1.0, NAN) * 2.0).y.is_nan());
    assert!(Vec2::new(NAN, 0.0).length().is_nan());
    assert!(Rect::from_ltrb(NAN, 0.0, 1.0, 1.0).width().is_nan());
    assert!(
        (Affine::new([NAN, 0.0, 0.0, 1.0, 0.0, 0.0]) * Point::new(1.0, 0.0))
            .x
            .is_nan()
    );
}

#[test]
fn core_geom_03_constructors_store_as_given() {
    let p = Point::new(-3.0, 7.5);
    assert_eq!((p.x, p.y), (-3.0, 7.5));
    let v = Vec2::new(-1.0, 2.0);
    assert_eq!((v.x, v.y), (-1.0, 2.0));
    let s = Size::new(-4.0, INF);
    assert_eq!((s.width, s.height), (-4.0, INF));
    let r = Rect::from_ltrb(10.0, 20.0, 5.0, 1.0);
    assert_eq!((r.left, r.top, r.right, r.bottom), (10.0, 20.0, 5.0, 1.0));
    let i = EdgeInsets::from_ltrb(-1.0, 2.0, -3.0, 4.0);
    assert_eq!((i.left, i.top, i.right, i.bottom), (-1.0, 2.0, -3.0, 4.0));
    let c = [0.0, -2.0, 3.0, 0.0, -5.0, 6.0];
    assert_eq!(Affine::new(c).coeffs(), c);
}

#[test]
fn core_geom_04_is_finite() {
    assert!(Point::new(1.0, 2.0).is_finite());
    assert!(!Point::new(NAN, 2.0).is_finite());
    assert!(!Point::new(1.0, -INF).is_finite());

    assert!(Vec2::new(1.0, 2.0).is_finite());
    assert!(!Vec2::new(INF, 2.0).is_finite());
    assert!(!Vec2::new(1.0, NAN).is_finite());

    assert!(Size::new(1.0, 2.0).is_finite());
    assert!(!Size::INFINITY.is_finite());
    assert!(!Size::new(1.0, NAN).is_finite());

    assert!(Rect::from_ltrb(0.0, 0.0, 1.0, 1.0).is_finite());
    for k in 0..4 {
        let mut e = [0.0, 0.0, 1.0, 1.0];
        e[k] = if k % 2 == 0 { NAN } else { INF };
        assert!(
            !Rect::from_ltrb(e[0], e[1], e[2], e[3]).is_finite(),
            "edge {k}"
        );
    }

    assert!(Affine::IDENTITY.is_finite());
    for k in 0..6 {
        let mut c = Affine::IDENTITY.coeffs();
        c[k] = if k % 2 == 0 { INF } else { NAN };
        assert!(!Affine::new(c).is_finite(), "coefficient {k}");
    }
}

// ---- Point and Vec2 ---------------------------------------------------------------------

#[test]
fn core_geom_05_point_vec2_ops() {
    let p = Point::new(1.0, 2.0);
    let v = Vec2::new(10.0, 20.0);
    assert_eq!(p + v, Point::new(11.0, 22.0));
    assert_eq!(p - v, Point::new(-9.0, -18.0));
    assert_eq!(Point::new(5.0, 7.0) - p, Vec2::new(4.0, 5.0));

    let mut q = p;
    q += v;
    assert_eq!(q, Point::new(11.0, 22.0));
    q -= v;
    assert_eq!(q, p);

    assert_eq!(p.to_vec2(), Vec2::new(1.0, 2.0));
    assert_eq!(v.to_point(), Point::new(10.0, 20.0));
}

#[test]
fn core_geom_06_vec2_ops_and_length() {
    let a = Vec2::new(1.0, -2.0);
    let b = Vec2::new(3.0, 5.0);
    assert_eq!(a + b, Vec2::new(4.0, 3.0));
    assert_eq!(a - b, Vec2::new(-2.0, -7.0));
    assert_eq!(-a, Vec2::new(-1.0, 2.0));
    assert_eq!(a * 3.0, Vec2::new(3.0, -6.0));

    let mut c = a;
    c += b;
    assert_eq!(c, Vec2::new(4.0, 3.0));
    c -= b;
    assert_eq!(c, a);

    assert_eq!(Vec2::new(3.0, 4.0).length(), 5.0);
    assert_eq!(Vec2::ZERO.length(), 0.0);
    assert_eq!(Vec2::new(-3.0, -4.0).length(), 5.0);
}

// ---- Size -------------------------------------------------------------------------------

#[test]
fn core_geom_07_size_is_empty() {
    assert!(!Size::new(1.0, 1.0).is_empty());
    assert!(Size::ZERO.is_empty());
    assert!(Size::new(0.0, 5.0).is_empty());
    assert!(Size::new(5.0, -1.0).is_empty());
    assert!(Size::new(NAN, 5.0).is_empty());
    assert!(Size::new(5.0, NAN).is_empty());
    assert!(!Size::INFINITY.is_empty());
    assert!(Size::new(-INF, 5.0).is_empty());
}

#[test]
fn core_geom_08_size_min_max() {
    let a = Size::new(1.0, 10.0);
    let b = Size::new(5.0, 2.0);
    assert_eq!(a.min(b), Size::new(1.0, 2.0));
    assert_eq!(a.max(b), Size::new(5.0, 10.0));
    assert_eq!(Size::INFINITY.min(a), a);
    assert_eq!(Size::new(NAN, 3.0).min(a), Size::new(1.0, 3.0));
    assert_eq!(a.max(Size::new(4.0, NAN)), Size::new(4.0, 10.0));
}

#[test]
fn core_geom_09_size_to_rect() {
    assert_eq!(
        Size::new(3.0, 4.0).to_rect(),
        Rect::from_ltrb(0.0, 0.0, 3.0, 4.0)
    );
}

// ---- Rect -------------------------------------------------------------------------------

#[test]
fn core_geom_10_rect_constructors() {
    let expected = Rect::from_ltrb(10.0, 20.0, 40.0, 60.0);
    assert_eq!(Rect::from_ltwh(10.0, 20.0, 30.0, 40.0), expected);
    assert_eq!(
        Rect::from_origin_size(Point::new(10.0, 20.0), Size::new(30.0, 40.0)),
        expected
    );
    let reversed = Rect::from_ltrb(5.0, 5.0, 1.0, 2.0);
    assert_eq!((reversed.right, reversed.bottom), (1.0, 2.0));
}

#[test]
fn core_geom_11_rect_accessors() {
    let r = Rect::from_ltrb(10.0, 20.0, 40.0, 60.0);
    assert_eq!(r.width(), 30.0);
    assert_eq!(r.height(), 40.0);
    assert_eq!(r.size(), Size::new(30.0, 40.0));
    assert_eq!(r.origin(), Point::new(10.0, 20.0));
    assert_eq!(r.center(), Point::new(25.0, 40.0));

    let reversed = Rect::from_ltrb(5.0, 5.0, 1.0, 2.0);
    assert_eq!(reversed.width(), -4.0);
    assert_eq!(reversed.height(), -3.0);
}

#[test]
fn core_geom_12_rect_is_empty() {
    assert!(!Rect::from_ltwh(0.0, 0.0, 1.0, 1.0).is_empty());
    assert!(Rect::ZERO.is_empty());
    assert!(Rect::from_ltwh(0.0, 0.0, 0.0, 5.0).is_empty());
    assert!(Rect::from_ltwh(0.0, 0.0, 5.0, 0.0).is_empty());
    assert!(Rect::from_ltrb(5.0, 0.0, 1.0, 5.0).is_empty());
    for k in 0..4 {
        let mut e = [0.0, 0.0, 10.0, 10.0];
        e[k] = NAN;
        assert!(
            Rect::from_ltrb(e[0], e[1], e[2], e[3]).is_empty(),
            "edge {k}"
        );
    }
    assert!(!Rect::from_ltrb(-INF, -INF, INF, INF).is_empty());
}

#[test]
fn core_geom_13_rect_contains() {
    let r = Rect::from_ltrb(0.0, 0.0, 10.0, 10.0);
    assert!(r.contains(Point::new(0.0, 0.0)));
    assert!(r.contains(Point::new(5.0, 9.999)));
    assert!(!r.contains(Point::new(10.0, 5.0)));
    assert!(!r.contains(Point::new(5.0, 10.0)));
    assert!(!r.contains(Point::new(-0.001, 5.0)));
    assert!(!r.contains(Point::new(NAN, 5.0)));
    assert!(!r.contains(Point::new(5.0, NAN)));

    let empty = Rect::from_ltrb(0.0, 0.0, 0.0, 10.0);
    assert!(!empty.contains(Point::new(0.0, 5.0)));
    let reversed = Rect::from_ltrb(10.0, 10.0, 0.0, 0.0);
    assert!(!reversed.contains(Point::new(5.0, 5.0)));
}

#[test]
fn core_geom_14_rect_overlaps() {
    let a = Rect::from_ltrb(0.0, 0.0, 10.0, 10.0);
    assert!(a.overlaps(Rect::from_ltrb(5.0, 5.0, 15.0, 15.0)));
    assert!(a.overlaps(Rect::from_ltrb(2.0, 2.0, 3.0, 3.0)));
    assert!(a.overlaps(a));
    // touching along an edge, at a corner, and apart
    assert!(!a.overlaps(Rect::from_ltrb(10.0, 0.0, 20.0, 10.0)));
    assert!(!a.overlaps(Rect::from_ltrb(10.0, 10.0, 20.0, 20.0)));
    assert!(!a.overlaps(Rect::from_ltrb(20.0, 20.0, 30.0, 30.0)));
    // empty rects overlap nothing, even when inside
    assert!(!a.overlaps(Rect::from_ltrb(5.0, 5.0, 5.0, 8.0)));
    assert!(!Rect::from_ltrb(5.0, 5.0, 5.0, 8.0).overlaps(a));
    assert!(!a.overlaps(Rect::from_ltrb(8.0, 8.0, 2.0, 2.0)));
    assert!(!a.overlaps(Rect::from_ltrb(NAN, 0.0, 5.0, 5.0)));
}

#[test]
fn core_geom_15_rect_intersect() {
    let a = Rect::from_ltrb(0.0, 0.0, 10.0, 10.0);
    assert_eq!(
        a.intersect(Rect::from_ltrb(5.0, -5.0, 15.0, 8.0)),
        Some(Rect::from_ltrb(5.0, 0.0, 10.0, 8.0))
    );
    assert_eq!(a.intersect(a), Some(a));
    assert_eq!(a.intersect(Rect::from_ltrb(10.0, 0.0, 20.0, 10.0)), None);
    assert_eq!(a.intersect(Rect::from_ltrb(20.0, 20.0, 30.0, 30.0)), None);
    assert_eq!(a.intersect(Rect::from_ltrb(5.0, 5.0, 5.0, 8.0)), None);
}

#[test]
fn core_geom_16_rect_union() {
    let a = Rect::from_ltrb(0.0, 0.0, 10.0, 10.0);
    let b = Rect::from_ltrb(5.0, -5.0, 20.0, 8.0);
    assert_eq!(a.union(b), Rect::from_ltrb(0.0, -5.0, 20.0, 10.0));
    assert_eq!(b.union(a), Rect::from_ltrb(0.0, -5.0, 20.0, 10.0));

    let empty = Rect::from_ltrb(100.0, 100.0, 100.0, 200.0);
    assert_eq!(a.union(empty), a);
    assert_eq!(empty.union(a), a);
    let other_empty = Rect::from_ltrb(-50.0, -50.0, -60.0, -60.0);
    assert_eq!(empty.union(other_empty), empty);
    assert_eq!(other_empty.union(empty), other_empty);
}

#[test]
fn core_geom_17_rect_translate() {
    let r = Rect::from_ltrb(1.0, 2.0, 3.0, 4.0);
    assert_eq!(
        r.translate(Vec2::new(10.0, -20.0)),
        Rect::from_ltrb(11.0, -18.0, 13.0, -16.0)
    );
}

#[test]
fn core_geom_18_rect_inflate_deflate() {
    let r = Rect::from_ltrb(10.0, 10.0, 20.0, 30.0);
    assert_eq!(r.inflate(2.0), Rect::from_ltrb(8.0, 8.0, 22.0, 32.0));
    assert_eq!(r.deflate(2.0), Rect::from_ltrb(12.0, 12.0, 18.0, 28.0));
    assert_eq!(r.deflate(2.0), r.inflate(-2.0));
    let past = r.deflate(6.0);
    assert_eq!(past, Rect::from_ltrb(16.0, 16.0, 14.0, 24.0));
    assert!(past.is_empty());
}

// ---- EdgeInsets -------------------------------------------------------------------------

#[test]
fn core_geom_19_edge_insets_constructors_and_sums() {
    assert_eq!(
        EdgeInsets::all(3.0),
        EdgeInsets::from_ltrb(3.0, 3.0, 3.0, 3.0)
    );
    let s = EdgeInsets::symmetric(4.0, 7.0);
    assert_eq!((s.left, s.top, s.right, s.bottom), (4.0, 7.0, 4.0, 7.0));
    let i = EdgeInsets::from_ltrb(1.0, 2.0, 3.0, 4.0);
    assert_eq!((i.left, i.top, i.right, i.bottom), (1.0, 2.0, 3.0, 4.0));
    assert_eq!(i.horizontal(), 4.0);
    assert_eq!(i.vertical(), 6.0);
    assert_eq!(
        i + EdgeInsets::from_ltrb(10.0, 20.0, 30.0, 40.0),
        EdgeInsets::from_ltrb(11.0, 22.0, 33.0, 44.0)
    );
}

#[test]
fn core_geom_20_edge_insets_sizes() {
    let i = EdgeInsets::from_ltrb(1.0, 2.0, 3.0, 4.0);
    assert_eq!(i.deflate_size(Size::new(10.0, 10.0)), Size::new(6.0, 4.0));
    assert_eq!(i.deflate_size(Size::new(3.0, 5.0)), Size::new(0.0, 0.0));
    assert_eq!(i.deflate_size(Size::new(NAN, 10.0)), Size::new(0.0, 4.0));
    assert_eq!(i.inflate_size(Size::new(10.0, 10.0)), Size::new(14.0, 16.0));
    assert_eq!(
        EdgeInsets::all(-5.0).inflate_size(Size::new(4.0, 4.0)),
        Size::new(-6.0, -6.0)
    );
    assert_eq!(i.deflate_size(Size::INFINITY), Size::INFINITY);
    assert_eq!(i.inflate_size(Size::INFINITY), Size::INFINITY);
}

#[test]
fn core_geom_21_edge_insets_rects() {
    let i = EdgeInsets::from_ltrb(1.0, 2.0, 3.0, 4.0);
    let r = Rect::from_ltrb(0.0, 0.0, 10.0, 10.0);
    assert_eq!(i.deflate_rect(r), Rect::from_ltrb(1.0, 2.0, 7.0, 6.0));
    assert_eq!(i.inflate_rect(r), Rect::from_ltrb(-1.0, -2.0, 13.0, 14.0));
    // not clamped
    assert_eq!(
        EdgeInsets::all(8.0).deflate_rect(r),
        Rect::from_ltrb(8.0, 8.0, 2.0, 2.0)
    );
    // negative insets move edges the other way
    assert_eq!(
        EdgeInsets::all(-1.0).deflate_rect(r),
        Rect::from_ltrb(-1.0, -1.0, 11.0, 11.0)
    );
}

// ---- Affine -----------------------------------------------------------------------------

#[test]
fn core_geom_22_affine_coeffs_and_point_mapping() {
    let c = [2.0, 3.0, 5.0, 7.0, 11.0, 13.0];
    let t = Affine::new(c);
    assert_eq!(t.coeffs(), c);
    // (a·x + c·y + e, b·x + d·y + f) at (1, 10)
    assert_eq!(t * Point::new(1.0, 10.0), Point::new(63.0, 86.0));

    assert_eq!(Affine::IDENTITY.coeffs(), [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    for p in [Point::ZERO, Point::new(-3.5, 8.25), Point::new(1e6, -1e-6)] {
        assert_eq!(Affine::IDENTITY * p, p);
    }
}

#[test]
fn core_geom_23_affine_constructors() {
    assert_eq!(
        Affine::translate(Vec2::new(3.0, -4.0)).coeffs(),
        [1.0, 0.0, 0.0, 1.0, 3.0, -4.0]
    );
    assert_eq!(Affine::scale(2.5).coeffs(), [2.5, 0.0, 0.0, 2.5, 0.0, 0.0]);
    assert_eq!(
        Affine::scale_non_uniform(2.0, -3.0).coeffs(),
        [2.0, 0.0, 0.0, -3.0, 0.0, 0.0]
    );

    let theta = 0.3_f32;
    let (s, c) = theta.sin_cos();
    assert_coeffs_close(
        Affine::rotate(theta).coeffs(),
        [c, s, -s, c, 0.0, 0.0],
        1e-6,
    );
    assert_point_close(
        Affine::rotate(theta) * Point::new(1.0, 0.0),
        Point::new(c, s),
        1e-6,
    );
    // y points down, so a quarter turn takes +x to +y (clockwise on screen)
    assert_point_close(
        Affine::rotate(FRAC_PI_2) * Point::new(1.0, 0.0),
        Point::new(0.0, 1.0),
        1e-6,
    );
}

#[test]
fn core_geom_24_affine_composition_order() {
    let t = Affine::translate(Vec2::new(10.0, 0.0));
    let s = Affine::scale(2.0);
    let p = Point::new(1.0, 1.0);
    // right operand first
    assert_eq!((t * s) * p, Point::new(12.0, 2.0));
    assert_eq!((s * t) * p, Point::new(22.0, 2.0));

    let mut rng = Rng(0x1234_5678);
    for _ in 0..200 {
        let a = Affine::new(std::array::from_fn(|_| rng.range(-10.0, 10.0)));
        let b = Affine::new(std::array::from_fn(|_| rng.range(-10.0, 10.0)));
        let p = Point::new(rng.range(-100.0, 100.0), rng.range(-100.0, 100.0));
        assert_point_close((a * b) * p, a * (b * p), 1e-2);
    }
}

#[test]
fn core_geom_25_affine_transform_vec() {
    let t = Affine::new([2.0, 3.0, 5.0, 7.0, 11.0, 13.0]);
    assert_eq!(t.transform_vec(Vec2::new(1.0, 10.0)), Vec2::new(52.0, 73.0));
    assert_eq!(
        Affine::translate(Vec2::new(100.0, 100.0)).transform_vec(Vec2::new(1.0, 2.0)),
        Vec2::new(1.0, 2.0)
    );
}

#[test]
fn core_geom_26_affine_transform_rect_bbox() {
    let r = Rect::from_ltrb(0.0, 0.0, 10.0, 20.0);
    // translations and positive scales: exact
    assert_eq!(
        Affine::translate(Vec2::new(5.0, -5.0)).transform_rect_bbox(r),
        Rect::from_ltrb(5.0, -5.0, 15.0, 15.0)
    );
    assert_eq!(
        Affine::scale_non_uniform(2.0, 3.0).transform_rect_bbox(r),
        Rect::from_ltrb(0.0, 0.0, 20.0, 60.0)
    );
    // negative scale flips; bbox keeps left ≤ right, top ≤ bottom
    assert_eq!(
        Affine::scale(-1.0).transform_rect_bbox(r),
        Rect::from_ltrb(-10.0, -20.0, 0.0, 0.0)
    );
    // 45° rotation of a unit square around the origin
    let bbox = Affine::rotate(PI / 4.0).transform_rect_bbox(Rect::from_ltrb(0.0, 0.0, 1.0, 1.0));
    let h = std::f32::consts::FRAC_1_SQRT_2;
    let tol = 1e-6;
    assert!(close(bbox.left, -h, tol), "{bbox:?}");
    assert!(close(bbox.top, 0.0, tol), "{bbox:?}");
    assert!(close(bbox.right, h, tol), "{bbox:?}");
    assert!(close(bbox.bottom, 2.0 * h, tol), "{bbox:?}");
}

#[test]
fn core_geom_27_affine_determinant() {
    assert_eq!(
        Affine::new([2.0, 3.0, 5.0, 7.0, 11.0, 13.0]).determinant(),
        2.0 * 7.0 - 3.0 * 5.0
    );
    assert_eq!(Affine::IDENTITY.determinant(), 1.0);
    assert_eq!(Affine::scale_non_uniform(2.0, 0.0).determinant(), 0.0);
}

#[test]
fn core_geom_28_affine_inverse_none_when_not_invertible() {
    assert_eq!(Affine::scale(0.0).inverse(), None);
    // the f32 determinant underflows to 0
    assert_eq!(Affine::scale(1e-30).determinant(), 0.0);
    assert_eq!(Affine::scale(1e-30).inverse(), None);
    assert_eq!(Affine::new([1.0, 2.0, 2.0, 4.0, 5.0, 6.0]).inverse(), None);
    assert_eq!(Affine::new([NAN, 0.0, 0.0, 1.0, 0.0, 0.0]).inverse(), None);
    assert_eq!(Affine::new([INF, 0.0, 0.0, 1.0, 0.0, 0.0]).inverse(), None);
    assert_eq!(Affine::new([1.0, 0.0, 0.0, 1.0, INF, 0.0]).inverse(), None);
    // determinant 1, but the inverse's translation (−1e50) overflows f32
    assert_eq!(
        Affine::new([1e-20, 0.0, 0.0, 1e20, 1e30, 0.0]).inverse(),
        None
    );
}

#[test]
fn core_geom_28_affine_inverse_round_trip() {
    let exact = Affine::translate(Vec2::new(3.0, 4.0)) * Affine::scale(2.0);
    let inv = exact.inverse().expect("invertible");
    assert_eq!((exact * inv).coeffs(), Affine::IDENTITY.coeffs());

    let mut rng = Rng(0x9e37_79b9);
    for _ in 0..1000 {
        let t = Vec2::new(rng.range(-1000.0, 1000.0), rng.range(-1000.0, 1000.0));
        let sx = 10f32.powf(rng.range(-1.0, 1.0));
        let sy = 10f32.powf(rng.range(-1.0, 1.0));
        let a = Affine::translate(t)
            * Affine::rotate(rng.range(-PI, PI))
            * Affine::scale_non_uniform(sx, sy);
        let inv = a
            .inverse()
            .expect("well-conditioned transform is invertible");
        let [.., e, f] = a.coeffs();
        let tol_t = 1e-4 * e.abs().max(f.abs()).max(1.0);
        for product in [a * inv, inv * a] {
            let got = product.coeffs();
            let want = Affine::IDENTITY.coeffs();
            for k in 0..4 {
                assert!(close(got[k], want[k], 1e-4), "{a:?}: {got:?} at {k}");
            }
            for k in 4..6 {
                assert!(close(got[k], want[k], tol_t), "{a:?}: {got:?} at {k}");
            }
        }
    }
}

// ---- Performance and allocation ---------------------------------------------------------

#[test]
fn core_geom_29_plain_value_sizes() {
    use std::mem::size_of;
    assert_eq!(size_of::<Point>(), 8);
    assert_eq!(size_of::<Vec2>(), 8);
    assert_eq!(size_of::<Size>(), 8);
    assert_eq!(size_of::<Rect>(), 16);
    assert_eq!(size_of::<EdgeInsets>(), 16);
    assert_eq!(size_of::<Affine>(), 24);
}
