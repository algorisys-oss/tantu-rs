//! The view-side clipboard: [`Clipboard`] and [`MemoryClipboard`]. Spec:
//! `docs/specs/platform/clipboard.md` (VIEW-CLIP-01).

use std::cell::RefCell;
use std::rc::Rc;

use tantu_scene::ImageData;

use crate::{BuildCx, ViewTree};

/// The clipboard as views see it (the runner bridges the platform's): text (VIEW-CLIP-01),
/// images, HTML and the primary selection (VIEW-CLIP-02). The newer methods have defaults.
pub trait Clipboard {
    /// The clipboard's text, or `None` when it is empty or holds no text.
    fn read_text(&self) -> Option<String>;
    /// Replaces the clipboard's content with `text`; false when it couldn't be written.
    fn write_text(&self, text: &str) -> bool;
    /// The clipboard's image, if it holds one. Default: none.
    fn read_image(&self) -> Option<ImageData> {
        todo!("VIEW-CLIP-02")
    }
    /// Puts an image on the clipboard. Default: unsupported (false).
    fn write_image(&self, image: &ImageData) -> bool {
        let _ = image;
        todo!("VIEW-CLIP-02")
    }
    /// Puts HTML on the clipboard with a plain-text alternative. Default: the text only.
    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        let _ = (html, alt_text);
        todo!("VIEW-CLIP-02")
    }
    /// The clipboard's HTML, if it holds any. Default: none.
    fn read_html(&self) -> Option<String> {
        todo!("VIEW-CLIP-02")
    }
    /// The primary selection, where there is one. Default: none.
    fn read_primary(&self) -> Option<String> {
        todo!("VIEW-CLIP-02")
    }
    /// Sets the primary selection; false where there is none. Default: false.
    fn write_primary(&self, text: &str) -> bool {
        let _ = text;
        todo!("VIEW-CLIP-02")
    }
}

/// An in-memory clipboard: every view tree starts with one.
#[derive(Default)]
pub struct MemoryClipboard {
    text: RefCell<Option<String>>,
}

impl Clipboard for MemoryClipboard {
    fn read_text(&self) -> Option<String> {
        self.text.borrow().clone()
    }

    fn write_text(&self, text: &str) -> bool {
        *self.text.borrow_mut() = (!text.is_empty()).then(|| text.to_owned());
        true
    }
}

/// The tree's current clipboard; the handles views get forward through it.
pub(crate) struct ClipboardSlot(RefCell<Rc<dyn Clipboard>>);

impl Default for ClipboardSlot {
    fn default() -> Self {
        ClipboardSlot(RefCell::new(Rc::new(MemoryClipboard::default())))
    }
}

impl Clipboard for ClipboardSlot {
    fn read_text(&self) -> Option<String> {
        let current = Rc::clone(&self.0.borrow());
        current.read_text()
    }

    fn write_text(&self, text: &str) -> bool {
        let current = Rc::clone(&self.0.borrow());
        current.write_text(text)
    }
}

impl ViewTree {
    /// Replaces the tree's clipboard; handles taken earlier follow (VIEW-CLIP-01).
    pub fn set_clipboard(&mut self, clipboard: Rc<dyn Clipboard>) {
        *self.clipboard.0.borrow_mut() = clipboard;
    }
}

impl BuildCx<'_> {
    /// The tree's clipboard: a handle that always reaches its current one (VIEW-CLIP-01).
    pub fn clipboard(&self) -> Rc<dyn Clipboard> {
        Rc::clone(&self.tree.clipboard) as Rc<dyn Clipboard>
    }
}
