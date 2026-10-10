//! Tests for `docs/specs/layout/constraints.md`, rules LAYOUT-CONS-01..17.

use tantu_core::{EdgeInsets, Size};
use tantu_layout::BoxConstraints;

const INF: f32 = f32::INFINITY;
const NAN: f32 = f32::NAN;

fn c(min_w: f32, max_w: f32, min_h: f32, max_h: f32) -> BoxConstraints {
    BoxConstraints::new(min_w, max_w, min_h, max_h)
}

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

/// The four values, for comparisons that must hold with NaN (`NaN != NaN`).
fn bits(b: BoxConstraints) -> [u32; 4] {
    [
        b.min_width.to_bits(),
        b.max_width.to_bits(),
        b.min_height.to_bits(),
        b.max_height.to_bits(),
    ]
}

/// A spread of constraints, normalized and not.
fn samples() -> Vec<BoxConstraints> {
    vec![
        c(0.0, INF, 0.0, INF),
        c(10.0, 100.0, 20.0, 50.0),
        c(30.0, 30.0, 40.0, 40.0),
        c(0.0, 0.0, 0.0, 0.0),
        c(5.0, INF, 0.0, 7.5),
        c(80.0, 20.0, 50.0, 10.0),
        c(-10.0, 5.0, -1.0, -0.5),
        c(INF, INF, 3.0, 4.0),
        c(NAN, 10.0, 0.0, NAN),
        c(1.0, NAN, NAN, 2.0),
    ]
}

#[test]
fn layout_cons_01_new_stores_as_given() {
    let b = c(-3.0, NAN, INF, -INF);
    assert_eq!(b.min_width, -3.0);
    assert!(b.max_width.is_nan());
    assert_eq!(b.min_height, INF);
    assert_eq!(b.max_height, -INF);
    assert_eq!(BoxConstraints::UNCONSTRAINED, c(0.0, INF, 0.0, INF));
    assert_eq!(BoxConstraints::default(), c(0.0, INF, 0.0, INF));
    const _: BoxConstraints = BoxConstraints::new(1.0, 2.0, 3.0, 4.0);
}

#[test]
fn layout_cons_02_tight_and_loose() {
    assert_eq!(
        BoxConstraints::tight(s(30.0, 40.0)),
        c(30.0, 30.0, 40.0, 40.0)
    );
    assert_eq!(
        BoxConstraints::loose(s(30.0, 40.0)),
        c(0.0, 30.0, 0.0, 40.0)
    );
    assert_eq!(BoxConstraints::tight(s(INF, 0.0)), c(INF, INF, 0.0, 0.0));
    assert_eq!(BoxConstraints::loose(s(INF, 5.0)), c(0.0, INF, 0.0, 5.0));
}

#[test]
fn layout_cons_03_tight_for_and_tight_for_finite() {
    assert_eq!(
        BoxConstraints::tight_for(Some(10.0), None),
        c(10.0, 10.0, 0.0, INF)
    );
    assert_eq!(
        BoxConstraints::tight_for(None, Some(20.0)),
        c(0.0, INF, 20.0, 20.0)
    );
    assert_eq!(
        BoxConstraints::tight_for(None, None),
        BoxConstraints::UNCONSTRAINED
    );
    assert_eq!(
        BoxConstraints::tight_for(Some(1.0), Some(2.0)),
        BoxConstraints::tight(s(1.0, 2.0))
    );
    assert_eq!(
        BoxConstraints::tight_for_finite(10.0, INF),
        c(10.0, 10.0, 0.0, INF)
    );
    assert_eq!(
        BoxConstraints::tight_for_finite(NAN, 20.0),
        c(0.0, INF, 20.0, 20.0)
    );
    assert_eq!(
        BoxConstraints::tight_for_finite(-INF, INF),
        BoxConstraints::UNCONSTRAINED
    );
    assert_eq!(
        BoxConstraints::tight_for_finite(3.0, 4.0),
        BoxConstraints::tight(s(3.0, 4.0))
    );
}

#[test]
fn layout_cons_04_expand() {
    assert_eq!(BoxConstraints::expand(None, None), c(INF, INF, INF, INF));
    assert_eq!(
        BoxConstraints::expand(Some(10.0), None),
        c(10.0, 10.0, INF, INF)
    );
    assert_eq!(
        BoxConstraints::expand(None, Some(5.0)),
        c(INF, INF, 5.0, 5.0)
    );
}

#[test]
fn layout_cons_05_constrain_normalized() {
    let b = c(10.0, 100.0, 20.0, 50.0);
    assert_eq!(b.constrain_width(50.0), 50.0);
    assert_eq!(b.constrain_width(10.0), 10.0);
    assert_eq!(b.constrain_width(100.0), 100.0);
    assert_eq!(b.constrain_width(5.0), 10.0);
    assert_eq!(b.constrain_width(-INF), 10.0);
    assert_eq!(b.constrain_width(500.0), 100.0);
    assert_eq!(b.constrain_width(INF), 100.0);
    assert_eq!(b.constrain_height(30.0), 30.0);
    assert_eq!(b.constrain_height(0.0), 20.0);
    assert_eq!(b.constrain_height(INF), 50.0);
    let unbounded = BoxConstraints::UNCONSTRAINED;
    assert_eq!(unbounded.constrain_width(INF), INF);
    assert_eq!(unbounded.constrain_height(1e30), 1e30);
    assert_eq!(unbounded.constrain_width(-5.0), 0.0);
}

#[test]
fn layout_cons_06_conflicts_and_nan() {
    // The minimum wins.
    let conflict = c(80.0, 20.0, 50.0, 10.0);
    for v in [-INF, 0.0, 15.0, 20.0, 50.0, 80.0, 1000.0, INF] {
        assert_eq!(conflict.constrain_width(v), 80.0, "{v}");
        assert_eq!(conflict.constrain_height(v), 50.0, "{v}");
    }
    // A NaN input constrains to the minimum.
    let b = c(10.0, 100.0, 20.0, 50.0);
    assert_eq!(b.constrain_width(NAN), 10.0);
    assert_eq!(b.constrain_height(NAN), 20.0);
    assert_eq!(conflict.constrain_width(NAN), 80.0);
    // A NaN bound is ignored on its side.
    let nan_max = c(10.0, NAN, 0.0, NAN);
    assert_eq!(nan_max.constrain_width(1e6), 1e6);
    assert_eq!(nan_max.constrain_width(5.0), 10.0);
    assert_eq!(nan_max.constrain_height(INF), INF);
    let nan_min = c(NAN, 100.0, NAN, 50.0);
    assert_eq!(nan_min.constrain_width(-40.0), -40.0);
    assert_eq!(nan_min.constrain_width(400.0), 100.0);
    assert_eq!(nan_min.constrain_height(7.0), 7.0);
    // Nothing panics, whatever the input.
    for b in samples() {
        for v in [NAN, -INF, -1.0, 0.0, 1.0, 1e9, INF] {
            let _ = b.constrain_width(v);
            let _ = b.constrain_height(v);
            let _ = b.constrain(s(v, v));
        }
    }
}

#[test]
fn layout_cons_07_constrain_biggest_smallest() {
    for b in samples() {
        for size in [s(0.0, 0.0), s(15.0, 45.0), s(1e6, -3.0), s(INF, NAN)] {
            let got = b.constrain(size);
            assert_eq!(
                got.width.to_bits(),
                b.constrain_width(size.width).to_bits(),
                "{b:?} {size:?}"
            );
            assert_eq!(
                got.height.to_bits(),
                b.constrain_height(size.height).to_bits(),
                "{b:?} {size:?}"
            );
        }
        let big = b.biggest();
        assert_eq!(
            big.width.to_bits(),
            b.constrain_width(INF).to_bits(),
            "{b:?}"
        );
        assert_eq!(
            big.height.to_bits(),
            b.constrain_height(INF).to_bits(),
            "{b:?}"
        );
        let small = b.smallest();
        assert_eq!(
            small.width.to_bits(),
            b.constrain_width(0.0).to_bits(),
            "{b:?}"
        );
        assert_eq!(
            small.height.to_bits(),
            b.constrain_height(0.0).to_bits(),
            "{b:?}"
        );
    }
    assert_eq!(c(10.0, 100.0, 20.0, 50.0).biggest(), s(100.0, 50.0));
    assert_eq!(c(10.0, 100.0, 20.0, 50.0).smallest(), s(10.0, 20.0));
    assert_eq!(BoxConstraints::UNCONSTRAINED.biggest(), s(INF, INF));
    assert_eq!(BoxConstraints::UNCONSTRAINED.smallest(), s(0.0, 0.0));
}

#[test]
fn layout_cons_08_tight_bounded_infinite() {
    let tight = c(30.0, 30.0, 40.0, 40.0);
    assert!(tight.has_tight_width() && tight.has_tight_height() && tight.is_tight());
    let half = c(30.0, 30.0, 0.0, 40.0);
    assert!(half.has_tight_width() && !half.has_tight_height() && !half.is_tight());
    let conflict = c(80.0, 20.0, 50.0, 10.0);
    assert!(conflict.is_tight());
    let loose = c(0.0, 100.0, 0.0, INF);
    assert!(!loose.has_tight_width());
    assert!(loose.has_bounded_width() && !loose.has_bounded_height());
    assert!(!loose.has_infinite_width() && !loose.has_infinite_height());
    let expand = BoxConstraints::expand(None, Some(5.0));
    assert!(expand.has_infinite_width() && !expand.has_infinite_height());
    assert!(expand.has_tight_width() && !expand.has_bounded_width());
    // NaN: every comparison is false.
    let nan = c(NAN, NAN, NAN, NAN);
    assert!(!nan.has_tight_width() && !nan.has_tight_height() && !nan.is_tight());
    assert!(!nan.has_bounded_width() && !nan.has_bounded_height());
    assert!(!nan.has_infinite_width() && !nan.has_infinite_height());
}

#[test]
fn layout_cons_09_is_satisfied_by() {
    let b = c(10.0, 100.0, 20.0, 50.0);
    assert!(b.is_satisfied_by(s(10.0, 20.0)));
    assert!(b.is_satisfied_by(s(100.0, 50.0)));
    assert!(b.is_satisfied_by(s(55.0, 35.0)));
    assert!(!b.is_satisfied_by(s(9.9, 30.0)));
    assert!(!b.is_satisfied_by(s(50.0, 50.1)));
    assert!(!b.is_satisfied_by(s(NAN, 30.0)));
    assert!(BoxConstraints::UNCONSTRAINED.is_satisfied_by(s(INF, 0.0)));
    assert!(!c(NAN, 100.0, 0.0, 10.0).is_satisfied_by(s(5.0, 5.0)));
    assert!(!c(80.0, 20.0, 0.0, 10.0).is_satisfied_by(s(50.0, 5.0)));
}

#[test]
fn layout_cons_10_is_normalized() {
    assert!(BoxConstraints::UNCONSTRAINED.is_normalized());
    assert!(BoxConstraints::tight(s(3.0, 4.0)).is_normalized());
    assert!(BoxConstraints::loose(s(3.0, 4.0)).is_normalized());
    assert!(BoxConstraints::tight(s(0.0, 0.0)).is_normalized());
    assert!(c(5.0, INF, 0.0, 7.5).is_normalized());
    assert!(!BoxConstraints::expand(None, Some(4.0)).is_normalized());
    assert!(!c(-1.0, 5.0, 0.0, 5.0).is_normalized());
    assert!(!c(0.0, 5.0, 0.0, -1.0).is_normalized());
    assert!(!c(6.0, 5.0, 0.0, 5.0).is_normalized());
    assert!(!c(NAN, 5.0, 0.0, 5.0).is_normalized());
    assert!(!c(0.0, NAN, 0.0, 5.0).is_normalized());
    assert!(!c(0.0, 5.0, 0.0, NAN).is_normalized());
}

#[test]
fn layout_cons_11_normalize() {
    for b in samples() {
        let n = b.normalize();
        assert!(n.is_normalized(), "{b:?} → {n:?}");
        if b.is_normalized() {
            assert_eq!(bits(n), bits(b));
        }
    }
    assert_eq!(c(-10.0, 5.0, -1.0, -0.5).normalize(), c(0.0, 5.0, 0.0, 0.0));
    assert_eq!(
        c(80.0, 20.0, 50.0, 10.0).normalize(),
        c(80.0, 80.0, 50.0, 50.0)
    );
    assert_eq!(c(NAN, 10.0, 0.0, NAN).normalize(), c(0.0, 10.0, 0.0, INF));
    assert_eq!(c(INF, INF, 3.0, 4.0).normalize(), c(0.0, INF, 3.0, 4.0));
    assert_eq!(c(-INF, -INF, 2.0, 1.0).normalize(), c(0.0, 0.0, 2.0, 2.0));
}

#[test]
fn layout_cons_12_loosen() {
    assert_eq!(
        c(10.0, 100.0, 20.0, 50.0).loosen(),
        c(0.0, 100.0, 0.0, 50.0)
    );
    assert_eq!(
        BoxConstraints::tight(s(3.0, 4.0)).loosen(),
        BoxConstraints::loose(s(3.0, 4.0))
    );
    assert_eq!(
        BoxConstraints::expand(None, None).loosen(),
        c(0.0, INF, 0.0, INF)
    );
}

#[test]
fn layout_cons_13_enforce() {
    let outer = c(10.0, 100.0, 20.0, 50.0);
    // Definition, value by value.
    for b in samples() {
        let e = b.enforce(outer);
        assert_eq!(
            bits(e),
            bits(c(
                outer.constrain_width(b.min_width),
                outer.constrain_width(b.max_width),
                outer.constrain_height(b.min_height),
                outer.constrain_height(b.max_height),
            )),
            "{b:?}"
        );
    }
    // Properties for normalized inputs.
    let normalized: Vec<_> = samples()
        .into_iter()
        .filter(|b| b.is_normalized())
        .collect();
    for b in &normalized {
        for o in &normalized {
            let e = b.enforce(*o);
            assert!(e.is_normalized(), "{b:?} in {o:?}");
            assert!(
                o.min_width <= e.min_width
                    && e.max_width <= o.max_width
                    && o.min_height <= e.min_height
                    && e.max_height <= o.max_height,
                "{b:?} in {o:?} → {e:?}"
            );
        }
    }
    let inside = c(20.0, 80.0, 25.0, 40.0);
    assert_eq!(inside.enforce(outer), inside);
    assert_eq!(BoxConstraints::UNCONSTRAINED.enforce(outer), outer);
    // expand() fills a bounded parent.
    assert_eq!(
        BoxConstraints::expand(None, None).enforce(outer),
        BoxConstraints::tight(s(100.0, 50.0))
    );
}

#[test]
fn layout_cons_14_tighten() {
    let b = c(10.0, 100.0, 20.0, 50.0);
    assert_eq!(b.tighten(Some(40.0), None), c(40.0, 40.0, 20.0, 50.0));
    assert_eq!(b.tighten(None, Some(70.0)), c(10.0, 100.0, 50.0, 50.0));
    assert_eq!(b.tighten(Some(1.0), Some(30.0)), c(10.0, 10.0, 30.0, 30.0));
    assert_eq!(b.tighten(None, None), b);
    assert_eq!(b.tighten(Some(NAN), None), c(10.0, 10.0, 20.0, 50.0));
}

#[test]
fn layout_cons_15_deflate() {
    let insets = EdgeInsets::from_ltrb(5.0, 1.0, 15.0, 9.0); // horizontal 20, vertical 10
    assert_eq!(
        c(30.0, 100.0, 5.0, 50.0).deflate(insets),
        c(10.0, 80.0, 0.0, 40.0)
    );
    // The maximum never drops below the new minimum.
    assert_eq!(
        c(0.0, 15.0, 0.0, 4.0).deflate(insets),
        c(0.0, 0.0, 0.0, 0.0)
    );
    // Infinite maxima stay infinite.
    assert_eq!(
        BoxConstraints::UNCONSTRAINED.deflate(insets),
        BoxConstraints::UNCONSTRAINED
    );
    // Negative insets grow both ends.
    assert_eq!(
        c(10.0, 20.0, 10.0, 20.0).deflate(EdgeInsets::all(-5.0)),
        c(20.0, 30.0, 20.0, 30.0)
    );
    for b in samples().into_iter().filter(|b| b.is_normalized()) {
        for i in [EdgeInsets::ZERO, insets, EdgeInsets::all(1e6)] {
            assert!(b.deflate(i).is_normalized(), "{b:?} - {i:?}");
        }
    }
}

#[test]
fn layout_cons_16_flip() {
    assert_eq!(c(1.0, 2.0, 3.0, 4.0).flip(), c(3.0, 4.0, 1.0, 2.0));
    for b in samples() {
        assert_eq!(bits(b.flip().flip()), bits(b));
    }
}

#[test]
fn layout_cons_17_axis_constraints() {
    let b = c(10.0, 100.0, 20.0, 50.0);
    assert_eq!(b.width_constraints(), c(10.0, 100.0, 0.0, INF));
    assert_eq!(b.height_constraints(), c(0.0, INF, 20.0, 50.0));
}
