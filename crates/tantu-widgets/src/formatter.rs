//! Input formatters: [`TextInputFormatter`], [`FilteringFormatter`],
//! [`LengthLimitingFormatter`] and [`MaskFormatter`].
//! Spec: `docs/specs/widgets/masked-input.md`.

use std::rc::Rc;

use crate::form::Validator;
use crate::text_editing::EditingValue;

/// Rewrites a user's edit before the field keeps it (Flutter's `TextInputFormatter`).
///
/// A `TextField` runs its formatters, in the order added, on every edit the user makes that
/// changes the text (WIDGETS-MASK-01).
pub trait TextInputFormatter {
    /// The value to keep, given the value before the edit and the proposed one.
    fn format(&self, old: &EditingValue, new: EditingValue) -> EditingValue;
}

impl<F: Fn(&EditingValue, EditingValue) -> EditingValue> TextInputFormatter for F {
    fn format(&self, old: &EditingValue, new: EditingValue) -> EditingValue {
        self(old, new)
    }
}

/// Keeps or drops characters (Flutter's `FilteringTextInputFormatter`, WIDGETS-MASK-02).
#[derive(Clone)]
pub struct FilteringFormatter {
    keep: Rc<dyn Fn(char) -> bool>,
}

impl FilteringFormatter {
    /// Keeps only the characters `keep` accepts.
    pub fn allow(keep: impl Fn(char) -> bool + 'static) -> Self {
        FilteringFormatter {
            keep: Rc::new(keep),
        }
    }

    /// Drops the characters `drop` accepts.
    pub fn deny(drop: impl Fn(char) -> bool + 'static) -> Self {
        Self::allow(move |c| !drop(c))
    }

    /// Keeps ASCII digits only.
    pub fn digits_only() -> Self {
        Self::allow(|c| c.is_ascii_digit())
    }
}

impl TextInputFormatter for FilteringFormatter {
    fn format(&self, _old: &EditingValue, _new: EditingValue) -> EditingValue {
        let _ = &self.keep;
        todo!("WIDGETS-MASK-02")
    }
}

/// Limits the text to a number of characters (Flutter's
/// `LengthLimitingTextInputFormatter`, WIDGETS-MASK-02).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LengthLimitingFormatter {
    max_chars: usize,
}

impl LengthLimitingFormatter {
    /// At most `max_chars` characters.
    pub fn new(max_chars: usize) -> Self {
        LengthLimitingFormatter { max_chars }
    }
}

impl TextInputFormatter for LengthLimitingFormatter {
    fn format(&self, _old: &EditingValue, _new: EditingValue) -> EditingValue {
        let _ = self.max_chars;
        todo!("WIDGETS-MASK-02")
    }
}

/// Shapes the text to a mask (WIDGETS-MASK-03..06): `#` a digit, `A` a letter, `*` a letter
/// or digit, `\` makes the next character literal, anything else is a literal the formatter
/// inserts.
///
/// ```
/// use tantu_widgets::{MaskFormatter, TextInputFormatter, EditingValue};
/// let phone = MaskFormatter::new("(###) ###-####");
/// let typed = phone.format(&EditingValue::default(), EditingValue::new("5551234567"));
/// assert_eq!(typed.text, "(555) 123-4567");
/// assert_eq!(phone.unmask(&typed.text), "5551234567");
/// assert!(phone.is_complete(&typed.text));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskFormatter {
    mask: String,
}

impl MaskFormatter {
    /// A formatter for `mask`.
    pub fn new(mask: &str) -> Self {
        MaskFormatter {
            mask: mask.to_string(),
        }
    }

    /// The characters the user typed into the slots of `text`, without the literals.
    pub fn unmask(&self, _text: &str) -> String {
        todo!("WIDGETS-MASK-06")
    }

    /// Whether every slot of the mask is filled in `text`.
    pub fn is_complete(&self, _text: &str) -> bool {
        todo!("WIDGETS-MASK-06")
    }

    /// A validator failing with `message` when the text is non-empty but incomplete.
    pub fn validator(&self, _message: impl Into<String>) -> Validator {
        todo!("WIDGETS-MASK-06")
    }
}

impl TextInputFormatter for MaskFormatter {
    fn format(&self, _old: &EditingValue, _new: EditingValue) -> EditingValue {
        let _ = &self.mask;
        todo!("WIDGETS-MASK-04")
    }
}
