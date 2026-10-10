//! Tests for `docs/specs/examples/layout-demo.md`, rules LAYOUT-DEMO-01..04.

use layout_demo::layout_demo;
use tantu_test::{Finder, WidgetTester};

fn rect(tester: &WidgetTester, key: &str) -> tantu::core::Rect {
    tester.rect(tester.find(&Finder::key(key)))
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

/// The structure rules, at any window size.
fn check_structure(tester: &WidgetTester, width: f32) {
    assert!(close(rect(tester, "header").width(), width));
    let sidebar = rect(tester, "sidebar");
    assert!(close(sidebar.width(), 160.0), "{sidebar:?}");
    let content = rect(tester, "content");
    assert!(close(content.left, 160.0), "{content:?}");
    assert!(close(content.width(), width - 160.0), "{content:?}");
    let (b1, b2, b3) = (
        rect(tester, "band1"),
        rect(tester, "band2"),
        rect(tester, "band3"),
    );
    assert!(b1.height() > 0.0);
    assert!(close(b2.height(), 2.0 * b1.height()), "{b1:?} {b2:?}");
    assert!(close(b3.height(), b1.height()), "{b1:?} {b3:?}");
    let (stack, badge) = (rect(tester, "stack"), rect(tester, "badge"));
    assert!(close(badge.right, stack.right - 8.0), "{stack:?} {badge:?}");
    assert!(close(badge.top, stack.top + 8.0), "{stack:?} {badge:?}");
}

#[test]
fn layout_demo_01_structure_at_800_by_600() {
    let tester = WidgetTester::with_size(800.0, 600.0, layout_demo);
    check_structure(&tester, 800.0);
}

#[test]
fn layout_demo_02_reflows_at_1200_by_800() {
    let tester = WidgetTester::with_size(1200.0, 800.0, layout_demo);
    check_structure(&tester, 1200.0);
    let small = WidgetTester::with_size(800.0, 600.0, layout_demo);
    assert!(rect(&tester, "band2").height() > rect(&small, "band2").height());
}

#[test]
fn layout_demo_03_sidebar_selects_a_section() {
    let mut tester = WidgetTester::with_size(800.0, 600.0, layout_demo);
    tester.find(&Finder::text("Section: Overview"));
    tester.tap(&Finder::key("section-settings"));
    tester.find(&Finder::text("Section: Settings"));
    tester.tap(&Finder::key("section-reports"));
    tester.find(&Finder::text("Section: Reports"));
}

#[test]
fn layout_demo_04_golden() {
    let mut tester = WidgetTester::with_size(800.0, 600.0, layout_demo);
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/layout-demo.png"
    ));
}
