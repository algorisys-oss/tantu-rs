//! Tests for `docs/specs/platform/file-drop.md`, rule WIDGETS-FILEDROP-01.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use tantu_test::WidgetTester;
use tantu_view::core::Point;
use tantu_view::reactive::{effect, signal};
use tantu_widgets::{FileDropTarget, SizedBox};

#[test]
fn widgets_filedrop_01_target() {
    let dropped: Rc<RefCell<Vec<Vec<PathBuf>>>> = Rc::default();
    let hovering_seen: Rc<RefCell<Vec<bool>>> = Rc::default();
    let mut t = {
        let (dropped, hovering_seen) = (dropped.clone(), hovering_seen.clone());
        WidgetTester::with_size(300.0, 200.0, move || {
            let hovering = signal(false);
            effect(move || hovering_seen.borrow_mut().push(hovering.get()));
            FileDropTarget::new(SizedBox::new(120.0, 80.0))
                .hovering(hovering)
                .on_drop(move |paths| dropped.borrow_mut().push(paths.to_vec()))
        })
    };
    let paths = vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.png")];
    t.drop_files(&paths, Some(Point::new(10.0, 10.0)));
    assert_eq!(*dropped.borrow(), [paths]);
    assert_eq!(
        *hovering_seen.borrow(),
        [false, true, false],
        "hovering while the files are over it"
    );
}
