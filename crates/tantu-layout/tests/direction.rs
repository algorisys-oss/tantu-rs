//! Tests for `docs/specs/widgets/directionality.md`, rule LAYOUT-DIR-01.

use tantu_core::EdgeInsets;
use tantu_layout::{Alignment, AlignmentDirectional, EdgeInsetsDirectional, TextDirection};

#[test]
fn layout_dir_01_resolve() {
    assert_eq!(TextDirection::default(), TextDirection::Ltr);
    let a = AlignmentDirectional::new(-0.5, 1.0);
    assert_eq!(a.resolve(TextDirection::Ltr), Alignment::new(-0.5, 1.0));
    assert_eq!(a.resolve(TextDirection::Rtl), Alignment::new(0.5, 1.0));
    assert_eq!(
        AlignmentDirectional::CENTER_START.resolve(TextDirection::Rtl),
        Alignment::CENTER_RIGHT
    );
    assert_eq!(
        AlignmentDirectional::TOP_END.resolve(TextDirection::Ltr),
        Alignment::TOP_RIGHT
    );
    let e = EdgeInsetsDirectional::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(
        e.resolve(TextDirection::Ltr),
        EdgeInsets::from_ltrb(1.0, 2.0, 3.0, 4.0)
    );
    assert_eq!(
        e.resolve(TextDirection::Rtl),
        EdgeInsets::from_ltrb(3.0, 2.0, 1.0, 4.0)
    );
}
