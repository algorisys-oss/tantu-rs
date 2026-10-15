# Forms, part 3: input formatters and masked input

- **Status:** Agreed (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (`TextInputFormatter`, `FilteringFormatter`,
  `LengthLimitingFormatter`, `MaskFormatter`; `TextField` and `TextFormField`),
  `examples/gallery`
- **Plan item:** Phase 4, "Forms: …, masked input", part 3
- **Related:** [text editing](text-editing.md), [TextField](text-field.md),
  [forms part 1](form.md), [gallery](../examples/gallery.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Phone numbers, dates, postal codes and account numbers have a fixed shape. A masked field
lets the user type only the characters that belong in it and inserts the separators
itself: typing `5551234567` shows `(555) 123-4567`. Flutter does this with
`TextInputFormatter`s, functions from the old and the proposed value to the value the field
keeps, chained on a `TextField`. Tantu takes the same model and adds a mask formatter, which
Flutter leaves to packages.

## Scope

In scope: the formatter trait and its chaining on `TextField` and `TextFormField`; filtering
and length-limiting formatters; a mask formatter with digit, letter and alphanumeric slots;
reading the raw input back and checking completeness; a mask validator; gallery samples.

Out of scope: showing the unfilled mask as a hint inside the field (apps use
`placeholder`); locale-aware number formatting (i18n item); optional or repeating mask
parts; formatting text set by the app through the controller (as in Flutter, formatters
apply to the user's edits only).

## Public API

```rust
// tantu-widgets
/// Rewrites a user's edit before the field keeps it (Flutter's `TextInputFormatter`).
pub trait TextInputFormatter {
    /// The value to keep, given the value before the edit and the proposed one.
    fn format(&self, old: &EditingValue, new: EditingValue) -> EditingValue;
}
/// Any `Fn(&EditingValue, EditingValue) -> EditingValue` is a formatter.
impl<F: Fn(&EditingValue, EditingValue) -> EditingValue> TextInputFormatter for F;

/// Keeps or drops characters (Flutter's `FilteringTextInputFormatter`).
#[derive(Clone)]
pub struct FilteringFormatter { /* private */ }
impl FilteringFormatter {
    /// Keeps only the characters `keep` accepts.
    pub fn allow(keep: impl Fn(char) -> bool + 'static) -> Self;
    /// Drops the characters `drop` accepts.
    pub fn deny(drop: impl Fn(char) -> bool + 'static) -> Self;
    /// Keeps ASCII digits only.
    pub fn digits_only() -> Self;
}

/// Limits the text to a number of characters (Flutter's `LengthLimitingTextInputFormatter`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LengthLimitingFormatter { /* private */ }
impl LengthLimitingFormatter {
    pub fn new(max_chars: usize) -> Self;
}

/// Shapes the text to a mask: `#` a digit, `A` a letter, `*` a letter or digit, `\` makes
/// the next character literal, anything else is a literal the formatter inserts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskFormatter { /* private */ }
impl MaskFormatter {
    pub fn new(mask: &str) -> Self;
    /// The characters the user typed into the slots of `text`, without the literals.
    pub fn unmask(&self, text: &str) -> String;
    /// Whether every slot of the mask is filled in `text`.
    pub fn is_complete(&self, text: &str) -> bool;
    /// A validator failing with `message` when the text is non-empty but incomplete.
    pub fn validator(&self, message: impl Into<String>) -> Validator;
}

impl TextField {
    /// Adds a formatter; formatters run in the order added.
    pub fn input_formatter(self, formatter: impl TextInputFormatter + 'static) -> Self;
}
impl TextFormField {
    /// Adds a formatter to the field's `TextField`.
    pub fn input_formatter(self, formatter: impl TextInputFormatter + 'static) -> Self;
}
```

## Behavior

- **WIDGETS-MASK-01:** A `TextField`'s formatters run, in the order added, on every edit the
  user makes that changes the text (typing, deleting, Enter's newline, an IME commit,
  paste, cut, the `SetValue` action), each receiving the value before the edit and the
  previous formatter's result; the controller keeps the last result, with its selection
  clamped to character boundaries. Edits that change only the selection, and IME
  compositions in progress, are not formatted. `on_change` reports the formatted text, and
  nothing when formatting left the text unchanged. Text the app sets through the
  controller is not formatted. `TextFormField::input_formatter` adds to its field.
- **WIDGETS-MASK-02:** `FilteringFormatter` removes the characters it doesn't keep from the
  proposed text; the selection's offsets move back by the bytes removed before them.
  `LengthLimitingFormatter` truncates the proposed text to its first `max_chars`
  characters when it is longer, clamping the selection; an edit within the limit passes
  unchanged.
- **WIDGETS-MASK-03:** A mask is a sequence of slots (`#` an ASCII digit, `A` an alphabetic
  character, `*` either) and literals (any other character, or any character after `\`). A
  trailing lone `\` is ignored.
- **WIDGETS-MASK-04:** `MaskFormatter` reads the input characters from the proposed text by
  walking it against the mask: a text character equal to the current literal consumes it;
  otherwise the literal is skipped; a character a slot accepts fills the slot; any other
  character is dropped. Input beyond the last slot is dropped. The result is the mask with
  its slots filled in order, each literal emitted only when a filled slot follows it (so
  the text never ends in a literal, and empty input gives empty text). Example with
  `(###) ###-####`: typing `5551234567` gives `(555) 123-4567`; pasting `555.123.4567` gives
  the same; typing a letter changes nothing.
- **WIDGETS-MASK-05:** Caret and selection: each end of the formatted selection sits after
  as many input characters as preceded it in the proposed text (at 0 when none did). A
  deletion that removes only literals (the text got shorter but the input characters are
  unchanged) also removes the input character before the caret, so Backspace never sticks
  on a separator.
- **WIDGETS-MASK-06:** `unmask` returns the input characters of a text read as in
  WIDGETS-MASK-04; `is_complete` is true when they fill every slot; `validator` fails with
  its message when the text is non-empty and incomplete (combine with
  `validators::required` for a required field).
- **GALLERY-40:** The Text field section's form gains a "Phone" field after Email, masked
  `(###) ###-####`, optional but validated with the mask's validator, and a "Phone: …"
  echo (`mask-echo`: "Phone: none", "Phone: incomplete", "Phone: 5551234567"). Test:
  typing `555x1234567` shows `(555) 123-4567` and echoes the digits.
- **GALLERY-41:** A "mask" kitchen-sink card after "combo" (twenty-two `SINK_CARDS` entries,
  sample `masked_field`): a date field masked `##/##/####`, echoed by `sink-mask-state`
  ("Date: none", "Date: incomplete", "Date: 12/31/2026").

## Performance and allocation

Formatting allocates the new text once per edit and is linear in its length; nothing runs
per frame.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **Flutter's formatter chain** on `TextField` rather than a dedicated `MaskedTextField`
   widget, so masks combine with filters, limits, forms and combo-free fields alike.
2. **Mask syntax `#`, `A`, `*`, `\`**, the common one in mask libraries.
3. **Literals appear only before a filled slot**: the separator shows when the next
   character is typed, so Backspace deletes characters, not separators.
4. **App-set text is not formatted** (Flutter's behavior); apps format it themselves if
   needed.
