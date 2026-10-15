//! The view-side clipboard: [`Clipboard`] and [`MemoryClipboard`]. Spec:
//! `docs/specs/platform/clipboard.md` (VIEW-CLIP-01).

use std::rc::Rc;

use crate::{BuildCx, ViewTree};

/// A plain-text clipboard as views see it (the runner bridges the platform's).
pub trait Clipboard {
    /// The clipboard's text, or `None` when it is empty or holds no text.
    fn read_text(&self) -> Option<String>;
    /// Replaces the clipboard's content with `text`; false when it couldn't be written.
    fn write_text(&self, text: &str) -> bool;
}

/// An in-memory clipboard: every view tree starts with one.
#[derive(Default)]
pub struct MemoryClipboard {
    text: std::cell::RefCell<Option<String>>,
}

impl Clipboard for MemoryClipboard {
    fn read_text(&self) -> Option<String> {
        let _ = &self.text;
        todo!("VIEW-CLIP-01")
    }

    fn write_text(&self, text: &str) -> bool {
        let _ = text;
        todo!("VIEW-CLIP-01")
    }
}

impl ViewTree {
    /// Replaces the tree's clipboard (call it before the views build; VIEW-CLIP-01).
    pub fn set_clipboard(&mut self, clipboard: Rc<dyn Clipboard>) {
        let _ = clipboard;
        todo!("VIEW-CLIP-01")
    }
}

impl BuildCx<'_> {
    /// The tree's clipboard (VIEW-CLIP-01).
    pub fn clipboard(&self) -> Rc<dyn Clipboard> {
        todo!("VIEW-CLIP-01")
    }
}
