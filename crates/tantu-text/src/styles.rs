//! [`TextStyles`]: a style table shared by a text system and view trees, and the
//! [`TextStyle`] presets. Spec: `docs/specs/text/styles.md`.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_layout::TextStyleKey;

use crate::TextStyle;

/// A table of text styles shared by a text system and the view trees that use it. Cloning is
/// cheap (a reference count) and every clone sees the same table. Not `Send` (one per UI
/// thread).
#[derive(Clone, Debug, Default)]
pub struct TextStyles {
    styles: Rc<RefCell<Vec<TextStyle>>>,
}

impl TextStyles {
    /// An empty table.
    pub fn new() -> Self {
        todo!()
    }

    /// The key for `style`: equal styles (after sanitizing) get the same key.
    pub fn key(&self, style: TextStyle) -> TextStyleKey {
        let _ = (style, &self.styles);
        todo!()
    }

    /// The style behind `key` (a copy), `None` for a key this table never gave out.
    pub fn get(&self, key: TextStyleKey) -> Option<TextStyle> {
        let _ = key;
        todo!()
    }

    /// Number of distinct styles.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True if the table has no styles.
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

impl TextStyle {
    /// Body text: sans-serif, 14 px, regular (Flutter's `bodyMedium`).
    pub fn body() -> Self {
        todo!()
    }

    /// A title: sans-serif, 22 px, regular (Flutter's `titleLarge`).
    pub fn title() -> Self {
        todo!()
    }

    /// Labels on controls: sans-serif, 14 px, weight 500 (Flutter's `labelLarge`).
    pub fn label() -> Self {
        todo!()
    }
}
