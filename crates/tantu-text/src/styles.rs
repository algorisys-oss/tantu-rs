//! [`TextStyles`]: a style table shared by a text system and view trees, and the
//! [`TextStyle`] presets. Spec: `docs/specs/text/styles.md`.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use tantu_layout::TextStyleKey;

use crate::TextStyle;

/// A table of text styles shared by a text system and the view trees that use it. Cloning is
/// cheap (a reference count) and every clone sees the same table. Not `Send` (one per UI
/// thread).
#[derive(Clone, Debug, Default)]
pub struct TextStyles {
    styles: Rc<RefCell<Vec<TextStyle>>>,
    fonts: Rc<RefCell<FontQueue>>,
}

/// Fonts queued by `ensure_font` (TEXT-STYLES-05), with the addresses ever queued.
#[derive(Debug, Default)]
struct FontQueue {
    pending: Vec<&'static [u8]>,
    seen: HashSet<usize>,
}

impl TextStyles {
    /// An empty table.
    pub fn new() -> Self {
        TextStyles::default()
    }

    /// The key for `style`: equal styles (after sanitizing) get the same key.
    pub fn key(&self, style: TextStyle) -> TextStyleKey {
        let style = style.sanitized();
        let mut styles = self.styles.borrow_mut();
        let index = match styles.iter().position(|s| *s == style) {
            Some(index) => index,
            None => {
                styles.push(style);
                styles.len() - 1
            }
        };
        TextStyleKey(index as u64)
    }

    /// The style behind `key` (a copy), `None` for a key this table never gave out.
    pub fn get(&self, key: TextStyleKey) -> Option<TextStyle> {
        let index = usize::try_from(key.0).ok()?;
        self.styles.borrow().get(index).cloned()
    }

    /// Number of distinct styles.
    pub fn len(&self) -> usize {
        self.styles.borrow().len()
    }

    /// True if the table has no styles.
    pub fn is_empty(&self) -> bool {
        self.styles.borrow().is_empty()
    }

    /// Queues `font` for registration by the text system before its next measure or paint
    /// (TEXT-STYLES-05). The same data (by address) is queued once per table.
    pub fn ensure_font(&self, font: &'static [u8]) {
        let mut queue = self.fonts.borrow_mut();
        if queue.seen.insert(font.as_ptr() as usize) {
            queue.pending.push(font);
        }
    }

    /// Takes the queued fonts (what the text system calls before measuring).
    pub fn take_pending_fonts(&self) -> Vec<&'static [u8]> {
        std::mem::take(&mut self.fonts.borrow_mut().pending)
    }
}

impl TextStyle {
    /// Body text: sans-serif, 14 px, regular (Flutter's `bodyMedium`).
    pub fn body() -> Self {
        TextStyle {
            size: 14.0,
            ..TextStyle::default()
        }
    }

    /// A title: sans-serif, 22 px, regular (Flutter's `titleLarge`).
    pub fn title() -> Self {
        TextStyle {
            size: 22.0,
            ..TextStyle::default()
        }
    }

    /// Labels on controls: sans-serif, 14 px, weight 500 (Flutter's `labelLarge`).
    pub fn label() -> Self {
        TextStyle {
            size: 14.0,
            weight: 500.0,
            ..TextStyle::default()
        }
    }
}
