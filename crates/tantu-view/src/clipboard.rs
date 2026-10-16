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
        None
    }
    /// Puts an image on the clipboard. Default: unsupported (false).
    fn write_image(&self, image: &ImageData) -> bool {
        let _ = image;
        false
    }
    /// Puts HTML on the clipboard with a plain-text alternative. Default: the text only.
    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        let _ = html;
        self.write_text(alt_text)
    }
    /// The clipboard's HTML, if it holds any. Default: none.
    fn read_html(&self) -> Option<String> {
        None
    }
    /// The primary selection, where there is one. Default: none.
    fn read_primary(&self) -> Option<String> {
        None
    }
    /// Sets the primary selection; false where there is none. Default: false.
    fn write_primary(&self, text: &str) -> bool {
        let _ = text;
        false
    }
}

/// An in-memory clipboard: every view tree starts with one. It keeps text, an image, HTML
/// and the primary selection side by side; writing HTML also sets the text to its
/// alternative (VIEW-CLIP-02).
#[derive(Default)]
pub struct MemoryClipboard {
    text: RefCell<Option<String>>,
    image: RefCell<Option<ImageData>>,
    html: RefCell<Option<String>>,
    primary: RefCell<Option<String>>,
}

/// Empty text counts as none.
fn non_empty(text: &str) -> Option<String> {
    (!text.is_empty()).then(|| text.to_owned())
}

impl Clipboard for MemoryClipboard {
    fn read_text(&self) -> Option<String> {
        self.text.borrow().clone()
    }

    fn write_text(&self, text: &str) -> bool {
        *self.text.borrow_mut() = non_empty(text);
        true
    }

    fn read_image(&self) -> Option<ImageData> {
        self.image.borrow().clone()
    }

    fn write_image(&self, image: &ImageData) -> bool {
        *self.image.borrow_mut() = Some(image.clone());
        true
    }

    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        *self.html.borrow_mut() = non_empty(html);
        *self.text.borrow_mut() = non_empty(alt_text);
        true
    }

    fn read_html(&self) -> Option<String> {
        self.html.borrow().clone()
    }

    fn read_primary(&self) -> Option<String> {
        self.primary.borrow().clone()
    }

    fn write_primary(&self, text: &str) -> bool {
        *self.primary.borrow_mut() = non_empty(text);
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
        self.current().read_text()
    }

    fn write_text(&self, text: &str) -> bool {
        self.current().write_text(text)
    }

    fn read_image(&self) -> Option<ImageData> {
        self.current().read_image()
    }

    fn write_image(&self, image: &ImageData) -> bool {
        self.current().write_image(image)
    }

    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        self.current().write_html(html, alt_text)
    }

    fn read_html(&self) -> Option<String> {
        self.current().read_html()
    }

    fn read_primary(&self) -> Option<String> {
        self.current().read_primary()
    }

    fn write_primary(&self, text: &str) -> bool {
        self.current().write_primary(text)
    }
}

impl ClipboardSlot {
    /// The installed clipboard (cloned out, so a call can't hold the slot borrowed).
    fn current(&self) -> Rc<dyn Clipboard> {
        Rc::clone(&self.0.borrow())
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
