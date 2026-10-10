# Shared text styles

- **Status:** Draft
- **Crates:** `tantu-text` (the table, presets), `tantu-view` (the build-time hook)
- **Plan item:** Phase 2, `tantu` facade → "Shared text styles"
- **Related:** [text system](system.md), [text in views](../view/text.md),
  [ADR 0010](../../adr/0010-text-measurement-in-layout.md), [basic widgets](../widgets/basic.md)

## Purpose

Layout and paint name text styles by `TextStyleKey`, and only `TextSystem::style` hands out keys.
Views build without access to the text system, yet the counter snippet writes
`Text::new(..).style(TextStyle::title())`, so a widget must turn a `TextStyle` into a key while
it builds. This spec moves the style table out of `TextSystem` into a cheap shared handle,
`TextStyles`. The text system and a view tree hold clones of the same table, and views get keys
through `BuildCx::text_style`. It also adds a few style presets for Phase 2, until
`tantu-theme` provides themed text styles.

## Scope

In scope:

- `TextStyles` (tantu-text): a shared, interior-mutable table from `TextStyle` to `TextStyleKey`.
- `TextSystem::styles`, `TextSystem::with_styles` (tantu-text).
- `TextStyle::body`, `TextStyle::title`, `TextStyle::label` presets (tantu-text).
- `ViewTree::with_text_styles`, `ViewTree::text_styles`, `BuildCx::text_style` (tantu-view).

Out of scope:

- Themed text styles, so that `title` follows the theme: `tantu-theme` (Phase 3).
- Removing styles from the table. Styles are small, and an app uses few distinct ones.

## Public API

```rust
// tantu-text
/// A table of text styles shared by a text system and the view trees that use it. Cloning is
/// cheap (a reference count) and every clone sees the same table. Not `Send` (one per UI thread).
#[derive(Clone, Default)]
pub struct TextStyles { /* Rc<RefCell<Vec<TextStyle>>> */ }

impl TextStyles {
    /// An empty table.
    pub fn new() -> Self;
    /// The key for `style`: equal styles (after sanitizing, TEXT-SYS-02) get the same key.
    pub fn key(&self, style: TextStyle) -> TextStyleKey;
    /// The style behind `key` (a copy), `None` for a key this table never gave out.
    pub fn get(&self, key: TextStyleKey) -> Option<TextStyle>;
    /// Number of distinct styles.
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

impl TextSystem {
    /// The system's style table (a clone of the handle).
    pub fn styles(&self) -> TextStyles;
    /// Uses `styles` as the system's table instead of its own.
    pub fn with_styles(self, styles: TextStyles) -> Self;
}

impl TextStyle {
    /// Body text: sans-serif, 14 px, regular. (Flutter's `bodyMedium`.)
    pub fn body() -> Self;
    /// A title: sans-serif, 22 px, regular. (Flutter's `titleLarge`.)
    pub fn title() -> Self;
    /// Labels on controls: sans-serif, 14 px, weight 500. (Flutter's `labelLarge`.)
    pub fn label() -> Self;
}

// tantu-view
impl ViewTree {
    /// Like `new`, with `styles` as the tree's style table (pass the text system's, so keys
    /// built here mean the same styles when measured and painted).
    pub fn with_text_styles<V: View>(styles: TextStyles, app: impl FnOnce() -> V) -> Self;
    /// The tree's style table (`new` creates an empty one of its own).
    pub fn text_styles(&self) -> &TextStyles;
}

impl BuildCx<'_> {
    /// The key for `style` in the tree's style table.
    pub fn text_style(&mut self, style: TextStyle) -> TextStyleKey;
}
```

`TextSystem::style` and `TextSystem::text_style` stay. They go through the shared table, and
`text_style` returns a copy, `Option<TextStyle>`, instead of a reference. That return type is a
breaking change, but nothing outside tantu-text calls it today.

## Behavior

- **TEXT-STYLES-01:** `TextStyles::key` gives equal keys for equal styles after sanitizing and
  different keys for different styles, and `get` returns the sanitized style. `len` counts
  distinct styles. A key the table never gave out returns `None`.
- **TEXT-STYLES-02:** Clones share the table: a key made through one clone resolves through
  every other clone, and `len` agrees.
- **TEXT-STYLES-03:** A `TextSystem` measures and paints keys from its table. After
  `with_styles(t)`, keys made with `t` directly, before or after the call, measure exactly like
  keys made with `TextSystem::style`, and `styles()` returns a handle to `t`.
- **TEXT-STYLES-04:** The presets have the documented family, size, weight and upright style, and
  no line height.
- **VIEW-STYLES-01:** `BuildCx::text_style` returns the key from the tree's table, so it resolves
  through the `TextStyles` given to `with_text_styles`. `ViewTree::new` gives the tree a table
  of its own.

## Performance and allocation

`key` is a linear search of a short list, done while building, never per frame.

## Open questions

1. **Shared handle vs. a style resolver passed into building.** Proposal: the shared handle.
   It's the smallest change: views stay plain values, and `ViewTree::new` keeps its shape.
2. **Presets as constructors on `TextStyle`**, which the AGENTS.md snippet uses, with Flutter's
   Material 3 sizes. Proposal: yes. A theme can later map them to themed styles.
