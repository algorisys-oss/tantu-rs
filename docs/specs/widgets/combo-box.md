# Forms, part 2: combo box with search

- **Status:** Agreed (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-widgets` (`ComboBox`), `tantu-view` (roles), `tantu-a11y` (mapping),
  `examples/gallery`
- **Plan item:** Phase 4, "Forms: …, combo box with search", part 2
- **Related:** [TextField](text-field.md), [overlay](../view/overlay.md),
  [ListView](list-view.md), [menus](menu.md), [forms part 1](form.md),
  [semantics](../a11y/semantics.md), [gallery](../examples/gallery.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Choosing one value from a long list (a country, a customer, a currency) is faster when the
user can type to narrow it down. A `ComboBox` is a text field with a list of options under
it: typing filters the list, arrows move through it, Enter or a click picks an option, and
the field then shows the chosen option. The selection is an index in a shared signal, like
`Radio` groups and `TabBar`s.

## Scope

In scope: a list of string options; the selected index as a shared signal; filtering by a
case-insensitive substring of the typed text; keyboard and pointer use; a popup of at most
six rows that scrolls; restoring the text when the popup closes without a choice; semantics;
gallery samples.

Out of scope (and where it goes): options with icons or custom rows; multiple selection
(chips); free text that isn't one of the options (an "editable" combo box); asynchronous
option loading; grouping (later, if wanted).

## Public API

```rust
// tantu-widgets
/// A text field with a filtered list of options (Flutter's `DropdownMenu` with filtering).
pub struct ComboBox { /* private */ }
impl ComboBox {
    /// Shows `options`; `selected` holds the chosen index (`None`: nothing chosen).
    pub fn new(selected: Signal<Option<usize>>, options: impl IntoIterator<Item = impl Into<String>>) -> Self;
    /// The field's width and the popup's (default 240 px).
    pub fn width(self, width: f32) -> Self;
    /// Shown in the empty field, and its accessible name.
    pub fn placeholder(self, text: impl Into<String>) -> Self;
    /// The label naming the field for assistive technology (default: the placeholder).
    pub fn label(self, label: impl Into<String>) -> Self;
    /// Called with the chosen index after the user picks an option.
    pub fn on_change(self, f: impl Fn(usize) + 'static) -> Self;
}

// tantu-view
pub enum Role { /* existing, */ ComboBox, ListBox, ListBoxOption }
```

## Behavior

- **WIDGETS-COMBO-01:** The field shows the selected option's text (empty for `None`, or an
  index out of range), and follows `selected` when the app changes it while the popup is
  closed. It is a `TextField` of the given width with a trailing arrow
  (`Icons::ARROW_DROP_DOWN`).
- **WIDGETS-COMBO-02:** The popup opens below the field (`Anchor::below()`, flip and clamp)
  on a primary press in the field, on Down or Alt+Down while the field has the focus, and
  when the user types. It lists the matching options, 40 px each, at most six visible (the
  rest scroll), on the theme's `surface` with a 4 px radius and a level-2 shadow, as wide as
  the field. An empty match list shows one "No matches" row that can't be chosen.
- **WIDGETS-COMBO-03:** Filtering: opened by a press or a key without typing, the popup
  lists every option; once the user edits the text, it lists the options containing the
  text, ignoring case, in their original order.
- **WIDGETS-COMBO-04:** Highlight: opening highlights the selected option if it is listed,
  else the first; Down/Up move the highlight (wrapping), Home/End go to the ends, the list
  scrolls to keep it visible, and hovering an option highlights it. The highlighted option
  has an `on_surface` 8 % layer; the selected one is drawn in `primary`.
- **WIDGETS-COMBO-05:** Choosing: Enter on a highlighted option, or a primary click on an
  option, sets `selected` to its index in the original list, puts its text in the field,
  closes the popup and calls `on_change` (only when the index changed). The focus stays in
  the field throughout.
- **WIDGETS-COMBO-06:** Closing without a choice (Escape, a press outside, Tab moving the
  focus away) closes the popup and restores the selected option's text in the field.
- **WIDGETS-COMBO-07:** Semantics: the field is a `ComboBox` named by the label (or the
  placeholder), with the field text as its value and `expanded` while the popup is open;
  the popup is a `ListBox` whose options are `ListBoxOption`s named by their text, with
  `selected` for the chosen one; the field's `active_descendant` is the highlighted option.
- **A11Y-TREE-13:** `Role::ComboBox`, `ListBox` and `ListBoxOption` map to AccessKit's roles
  of the same names.
- **GALLERY-38:** The Selection section gains a "Fruit" `ComboBox` of twelve fruits with a
  "Fruit: …" echo (`combo-echo`). Tests: typing "an" lists Banana, Mango and Orange (Banana
  highlighted); Down, Enter picks Mango; Escape after typing restores the field. Golden `gallery-combo.png`
  with the filtered popup open.
- **GALLERY-39:** A "combo" kitchen-sink card after "form" (twenty-one `SINK_CARDS` entries,
  sample `combo_box`): a currency combo box echoed by `sink-combo-state`.

## Performance and allocation

Filtering is linear in the options per edit; the popup's list builds only visible rows.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **String options and an index signal**, not a generic item type: the common case, and
   apps map indices to their values.
2. **Substring, case-insensitive filtering**, not prefix or fuzzy matching.
3. **The focus stays in the field**; the popup's highlight is exposed through
   `active_descendant` (the WAI-ARIA combobox pattern).
4. **No free text**: closing without a choice restores the chosen option's text.
