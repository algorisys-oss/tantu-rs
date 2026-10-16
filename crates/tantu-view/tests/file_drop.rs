//! Tests for `docs/specs/platform/file-drop.md`, rules VIEW-FILEDROP-01..02.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use tantu_core::{Point, Size};
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_scene::Scene;
use tantu_view::{AnyView, BuildCx, ElementId, FileDrop, FileDropEvent, View, ViewTree};

type Log = Rc<RefCell<Vec<String>>>;

/// A box of `size` holding `children`, a file-drop target named `name` (offstage when
/// `hidden`).
struct Target {
    name: &'static str,
    size: (f32, f32),
    children: Vec<AnyView>,
    hidden: bool,
    log: Log,
}

impl View for Target {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::sized(Some(self.size.0), Some(self.size.1)),
            self.children,
        );
        let (name, log) = (self.name, self.log);
        cx.on_file_drop(id, move |event| {
            let entry = match event {
                FileDropEvent::Hovered(paths) => format!("{name}:hover:{}", names(paths)),
                FileDropEvent::Cancelled => format!("{name}:cancel"),
                FileDropEvent::Dropped(paths) => format!("{name}:drop:{}", names(paths)),
            };
            log.borrow_mut().push(entry);
        });
        if self.hidden {
            cx.set_offstage(id, true);
        }
        id
    }
}

fn names(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or("?"))
        .collect::<Vec<_>>()
        .join(",")
}

/// A row of children 50 px apart, top-aligned.
struct Row(Vec<AnyView>);

impl View for Row {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let mut flex = RenderFlex::row();
        flex.spacing = 50.0;
        flex.cross_axis_alignment = tantu_layout::CrossAxisAlignment::Start;
        cx.render(flex, self.0)
    }
}

/// Z (offstage, first in tree order), O at x 0..200 holding I at 0..50, B at 250..300;
/// everything 50 px tall.
fn fixture() -> (ViewTree, Log) {
    let log: Log = Rc::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            let target = |name, size, children, hidden| Target {
                name,
                size,
                children,
                hidden,
                log: log.clone(),
            };
            let inner = target("I", (50.0, 50.0), vec![], false);
            Row(vec![
                AnyView::new(target("Z", (50.0, 50.0), vec![], true)),
                AnyView::new(target(
                    "O",
                    (200.0, 50.0),
                    vec![AnyView::new(Row(vec![AnyView::new(inner)]))],
                    false,
                )),
                AnyView::new(target("B", (50.0, 50.0), vec![], false)),
            ])
        })
    };
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(500.0, 100.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    (tree, log)
}

fn files() -> Vec<PathBuf> {
    vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.png")]
}

fn at(x: f32) -> Option<Point> {
    Some(Point::new(x, 25.0))
}

fn drop_at(tree: &mut ViewTree, position: Option<Point>) {
    tree.dispatch_file_drop(FileDrop::Dropped {
        paths: files(),
        position,
    });
}

#[test]
fn view_filedrop_01_targets() {
    let (mut tree, log) = fixture();
    // The innermost target under the position.
    drop_at(&mut tree, at(25.0));
    drop_at(&mut tree, at(100.0));
    drop_at(&mut tree, at(275.0));
    // No position, or a position over no target: the outermost target in tree order,
    // skipping the offstage Z.
    drop_at(&mut tree, None);
    drop_at(&mut tree, at(450.0));
    assert_eq!(
        *log.borrow(),
        [
            "I:drop:a.txt,b.png",
            "O:drop:a.txt,b.png",
            "B:drop:a.txt,b.png",
            "O:drop:a.txt,b.png",
            "O:drop:a.txt,b.png",
        ]
    );
    // A tree without targets ignores drops.
    let mut empty = ViewTree::new(|| Row(vec![]));
    drop_at(&mut empty, None);
}

#[test]
fn view_filedrop_02_hover_gesture() {
    let (mut tree, log) = fixture();
    let hover = |tree: &mut ViewTree, position| {
        tree.dispatch_file_drop(FileDrop::Hovered {
            paths: files(),
            position,
        })
    };
    // Hovering moves between targets: the old one is cancelled first.
    hover(&mut tree, at(25.0));
    hover(&mut tree, at(25.0));
    hover(&mut tree, at(275.0));
    // The drop goes to the hovered target, whatever its position says.
    drop_at(&mut tree, None);
    // A cancel without a hover goes nowhere; after a hover it ends the gesture.
    tree.dispatch_file_drop(FileDrop::Cancelled);
    hover(&mut tree, at(100.0));
    tree.dispatch_file_drop(FileDrop::Cancelled);
    // A drop without a hover picks its target as VIEW-FILEDROP-01.
    drop_at(&mut tree, at(25.0));
    assert_eq!(
        *log.borrow(),
        [
            "I:hover:a.txt,b.png",
            "I:hover:a.txt,b.png",
            "I:cancel",
            "B:hover:a.txt,b.png",
            "B:drop:a.txt,b.png",
            "O:hover:a.txt,b.png",
            "O:cancel",
            "I:drop:a.txt,b.png",
        ]
    );
}
