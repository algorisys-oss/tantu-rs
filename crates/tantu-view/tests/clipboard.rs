//! Tests for `docs/specs/platform/clipboard.md`, rule VIEW-CLIP-01.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_layout::RenderConstrainedBox;
use tantu_view::{BuildCx, Clipboard, ElementId, MemoryClipboard, View, ViewTree};

type Out = Rc<RefCell<Option<Rc<dyn Clipboard>>>>;

/// Hands out the clipboard it sees at build time.
struct Reader(Out);

impl View for Reader {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        *self.0.borrow_mut() = Some(cx.clipboard());
        cx.render(RenderConstrainedBox::expand(), [])
    }
}

/// A clipboard that records writes.
#[derive(Default)]
struct Recording(RefCell<Vec<String>>);

impl Clipboard for Recording {
    fn read_text(&self) -> Option<String> {
        self.0.borrow().last().cloned()
    }

    fn write_text(&self, text: &str) -> bool {
        self.0.borrow_mut().push(text.to_owned());
        true
    }
}

#[test]
fn view_clip_01_tree_clipboards() {
    // A memory clipboard: empty, then what was written; empty text empties it.
    let memory = MemoryClipboard::default();
    assert_eq!(memory.read_text(), None);
    assert!(memory.write_text("one"));
    assert_eq!(memory.read_text().as_deref(), Some("one"));
    assert!(memory.write_text(""));
    assert_eq!(memory.read_text(), None);

    // Every tree has its own in-memory clipboard.
    let out: Out = Rc::default();
    let o = out.clone();
    let tree = ViewTree::new(move || Reader(o));
    let clipboard = out.borrow_mut().take().expect("built");
    clipboard.write_text("tree");
    assert_eq!(clipboard.read_text().as_deref(), Some("tree"));
    drop(tree);

    // `set_clipboard` replaces it; handles taken at build time follow.
    let out: Out = Rc::default();
    let o = out.clone();
    let mut tree = ViewTree::new(move || Reader(o));
    let handle = out.borrow_mut().take().expect("built");
    handle.write_text("before");
    let recording = Rc::new(Recording::default());
    tree.set_clipboard(recording.clone());
    assert_eq!(handle.read_text(), None);
    assert!(handle.write_text("through the tree"));
    assert_eq!(*recording.0.borrow(), ["through the tree"]);
    assert_eq!(handle.read_text().as_deref(), Some("through the tree"));
}
