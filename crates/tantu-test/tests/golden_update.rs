//! TEST-WT-06's update mode, in its own test binary so setting the variable can't race other
//! tests.

use tantu_test::WidgetTester;
use tantu_widgets::SizedBox;

#[test]
fn test_wt_06_update_mode_writes_the_png() {
    let dir = std::env::temp_dir().join(format!("tantu-test-golden-{}", std::process::id()));
    let path = dir.join("nested").join("box.png");
    let _ = std::fs::remove_dir_all(&dir);
    // SAFETY: this binary has one test, so no other thread reads the environment meanwhile.
    unsafe { std::env::set_var("TANTU_UPDATE_GOLDENS", "1") };
    let mut tester = WidgetTester::with_size(20.0, 10.0, || SizedBox::shrink());
    tester.matches_golden(&path);
    // SAFETY: as above.
    unsafe { std::env::remove_var("TANTU_UPDATE_GOLDENS") };
    assert!(path.exists());
    // Compared, it now matches.
    tester.matches_golden(&path);
    let _ = std::fs::remove_dir_all(&dir);
}
