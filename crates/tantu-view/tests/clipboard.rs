//! Tests for `docs/specs/platform/clipboard.md`, rules VIEW-CLIP-01..02 (the latter from `docs/specs/platform/clipboard-rich.md`).

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

fn image() -> tantu_view::scene::ImageData {
    tantu_view::scene::ImageData::rgba8(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255])
        .expect("a 2 × 1 image")
}

#[test]
fn view_clip_02_images_html_and_primary() {
    // The memory clipboard keeps all four kinds; HTML sets the text too.
    let memory = MemoryClipboard::default();
    assert_eq!(memory.read_image(), None);
    assert!(memory.write_image(&image()));
    assert_eq!(memory.read_image(), Some(image()));
    assert!(memory.write_html("<b>T</b>", "T"));
    assert_eq!(memory.read_html().as_deref(), Some("<b>T</b>"));
    assert_eq!(memory.read_text().as_deref(), Some("T"));
    assert!(memory.write_primary("sel"));
    assert_eq!(memory.read_primary().as_deref(), Some("sel"));
    // A text-only clipboard gets the defaults.
    let text_only = Recording::default();
    assert_eq!(text_only.read_image(), None);
    assert!(!text_only.write_image(&image()));
    assert!(text_only.write_html("<b>x</b>", "x"));
    assert_eq!(text_only.read_text().as_deref(), Some("x"));
    assert_eq!(text_only.read_primary(), None);
    // Handles from the build forward to the tree's current clipboard.
    let out: Out = Rc::default();
    let mut tree = {
        let out = out.clone();
        ViewTree::new(move || Reader(out))
    };
    let handle = out.borrow_mut().take().expect("built");
    let installed = Rc::new(MemoryClipboard::default());
    tree.set_clipboard(installed.clone());
    assert!(handle.write_image(&image()));
    assert!(handle.write_primary("p"));
    assert!(handle.write_html("<i>h</i>", "h"));
    assert_eq!(installed.read_image(), Some(image()));
    assert_eq!(installed.read_primary().as_deref(), Some("p"));
    assert_eq!(handle.read_html().as_deref(), Some("<i>h</i>"));
}
