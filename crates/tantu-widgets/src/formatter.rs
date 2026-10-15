//! Input formatters: [`TextInputFormatter`], [`FilteringFormatter`],
//! [`LengthLimitingFormatter`] and [`MaskFormatter`].
//! Spec: `docs/specs/widgets/masked-input.md`.

use std::rc::Rc;

use crate::form::Validator;
use crate::text_editing::{EditingValue, TextSelection};

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
    fn format(&self, _old: &EditingValue, new: EditingValue) -> EditingValue {
        let mut text = String::with_capacity(new.text.len());
        // Bytes removed before each selection end.
        let (mut base, mut extent) = (new.selection.base, new.selection.extent);
        for (i, ch) in new.text.char_indices() {
            if (self.keep)(ch) {
                text.push(ch);
                continue;
            }
            let len = ch.len_utf8();
            if i < new.selection.base {
                base -= len;
            }
            if i < new.selection.extent {
                extent -= len;
            }
        }
        EditingValue {
            text,
            selection: TextSelection::new(base, extent),
            composing: None,
        }
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
    fn format(&self, _old: &EditingValue, new: EditingValue) -> EditingValue {
        let Some((cut, _)) = new.text.char_indices().nth(self.max_chars) else {
            return new;
        };
        let selection =
            TextSelection::new(new.selection.base.min(cut), new.selection.extent.min(cut));
        let mut text = new.text;
        text.truncate(cut);
        EditingValue {
            text,
            selection,
            composing: None,
        }
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
    tokens: Rc<[Token]>,
}

/// One part of a mask (WIDGETS-MASK-03).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    /// `#`: an ASCII digit.
    Digit,
    /// `A`: an alphabetic character.
    Letter,
    /// `*`: either.
    Any,
    /// A character the formatter inserts.
    Literal(char),
}

impl Token {
    fn accepts(self, ch: char) -> bool {
        match self {
            Token::Digit => ch.is_ascii_digit(),
            Token::Letter => ch.is_alphabetic(),
            Token::Any => ch.is_ascii_digit() || ch.is_alphabetic(),
            Token::Literal(_) => false,
        }
    }
}

/// The input characters read from a text, and how many preceded two offsets.
struct Read {
    inputs: Vec<char>,
    before: [usize; 2],
}

impl MaskFormatter {
    /// A formatter for `mask`.
    pub fn new(mask: &str) -> Self {
        let mut tokens = Vec::new();
        let mut chars = mask.chars();
        while let Some(ch) = chars.next() {
            tokens.push(match ch {
                '#' => Token::Digit,
                'A' => Token::Letter,
                '*' => Token::Any,
                // A trailing lone `\` is ignored.
                '\\' => match chars.next() {
                    Some(literal) => Token::Literal(literal),
                    None => break,
                },
                literal => Token::Literal(literal),
            });
        }
        MaskFormatter {
            tokens: tokens.into(),
        }
    }

    /// The characters the user typed into the slots of `text`, without the literals.
    pub fn unmask(&self, text: &str) -> String {
        self.read(text, [0, 0]).inputs.into_iter().collect()
    }

    /// Whether every slot of the mask is filled in `text`.
    pub fn is_complete(&self, text: &str) -> bool {
        self.read(text, [0, 0]).inputs.len() == self.slots()
    }

    /// A validator failing with `message` when the text is non-empty but incomplete.
    pub fn validator(&self, message: impl Into<String>) -> Validator {
        let (mask, message) = (self.clone(), message.into());
        Rc::new(move |text: &str| {
            (!text.is_empty() && !mask.is_complete(text)).then(|| message.clone())
        })
    }

    fn slots(&self) -> usize {
        self.tokens
            .iter()
            .filter(|t| !matches!(t, Token::Literal(_)))
            .count()
    }

    /// Reads `text` against the mask (WIDGETS-MASK-04), counting the inputs before each of
    /// `offsets`.
    fn read(&self, text: &str, offsets: [usize; 2]) -> Read {
        let mut inputs = Vec::new();
        let mut before = [usize::MAX; 2];
        let mut m = 0;
        for (i, ch) in text.char_indices() {
            for (b, &offset) in before.iter_mut().zip(&offsets) {
                if *b == usize::MAX && offset <= i {
                    *b = inputs.len();
                }
            }
            while let Some(&token) = self.tokens.get(m) {
                match token {
                    Token::Literal(literal) => {
                        m += 1;
                        if ch == literal {
                            break;
                        }
                    }
                    slot => {
                        if slot.accepts(ch) {
                            inputs.push(ch);
                            m += 1;
                        }
                        break;
                    }
                }
            }
        }
        for b in &mut before {
            if *b == usize::MAX {
                *b = inputs.len();
            }
        }
        Read { inputs, before }
    }

    /// The mask filled with `inputs`, and the offset after each count of inputs.
    fn render(&self, inputs: &[char]) -> (String, Vec<usize>) {
        let mut text = String::new();
        let mut after = vec![0];
        let mut pending = String::new();
        let mut n = 0;
        for &token in self.tokens.iter() {
            if n == inputs.len() {
                break;
            }
            match token {
                Token::Literal(literal) => pending.push(literal),
                _ => {
                    text.push_str(&pending);
                    pending.clear();
                    text.push(inputs[n]);
                    n += 1;
                    after.push(text.len());
                }
            }
        }
        (text, after)
    }
}

impl TextInputFormatter for MaskFormatter {
    fn format(&self, old: &EditingValue, new: EditingValue) -> EditingValue {
        let selection = new.selection;
        let Read {
            mut inputs,
            before: [mut base, mut extent],
        } = self.read(&new.text, [selection.base, selection.extent]);
        // A deletion of literals alone also deletes the input before the caret
        // (WIDGETS-MASK-05).
        if new.text.len() < old.text.len() && inputs == self.read(&old.text, [0, 0]).inputs {
            let caret = base.min(extent);
            if caret > 0 {
                inputs.remove(caret - 1);
                base = caret - 1;
                extent = caret - 1;
            }
        }
        let (text, after) = self.render(&inputs);
        let end = after.len() - 1;
        EditingValue {
            text,
            selection: TextSelection::new(after[base.min(end)], after[extent.min(end)]),
            composing: None,
        }
    }
}
