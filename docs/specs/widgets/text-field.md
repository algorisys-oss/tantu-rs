# TextField

- **Status:** Agreed (decided by the agent while the user was away, 2026-10-11; review the
  Open questions)
- **Crates:** `tantu-widgets` (`TextField`), `tantu-layout` (`TextGeometry`), `tantu-text`
  (geometry from parley), `tantu-view` (`TextPainter::text_geometry`), `tantu-test`
  (`WidgetTester::enter_text`)
- **Plan item:** Phase 3, "Widgets" → "`TextField`, in parts" → "the widget: caret, selection
  painting, keys, pointer"
- **Related:** [text editing model](text-editing.md), [RichText](rich-text.md) (spans),
  [focus](../view/focus.md), [shortcuts](../view/shortcuts.md), [events](../view/events.md),
  [text in views](../view/text.md), Flutter's `TextField`/`EditableText`

## Purpose

A single- or multi-line text input on top of the editing model: it shows the controller's
value, paints the selection and the caret, and turns keys and pointer input into edits. The
model has no layout, so the widget needs one thing from the text stack: the *geometry* of the
laid-out paragraph (lines and cluster boxes). The frame's painter produces it as plain data
(`TextGeometry`); the widget keeps the last one and answers "which offset is under this
point" and "where is the caret" from it, between frames, without parley. Those queries are
pure functions on the geometry, so they are tested on hand-built geometries.

The look is a Material 3 outlined field until `tantu-theme`.

## Scope

In scope:

- `TextGeometry` (`tantu-layout`): lines with cluster boxes; `caret`, `hit_test`,
  `selection_rects`, `line_of`, `offset_at`.
- `TextSystem::geometry` (parley), `TextPainter::text_geometry` (default `None`),
  `PaintCx::text_geometry`.
- `TextField::new(controller)`, `.placeholder`, `.max_lines`, `.enabled`, `.on_change`,
  `.on_submit`, `.style`; keys, chords, pointer, focus, painting, a golden.
- `WidgetTester::enter_text`.

Out of scope (and where it goes):

- Clipboard: the platform has none yet (HANDOFF open question); `Cut`/`Copy`/`Paste` chords
  come with it.
- IME composition: the next part. Obscured text, formatters, max length: forms.
- Scrolling long single-line text to keep the caret visible, and multi-line scrolling:
  the scroll item (the field clips for now).
- Double-click word selection and triple-click (need timers), shift-click (pointer events
  carry no modifiers yet), drag-and-drop of text, a context menu: later.
- Bidi caret logic and affinity at soft wraps: with RTL work in Phase 4 (the caret sits at
  the start of the next line at a soft wrap).
- Accessibility: `tantu-a11y`. Theming (filled variant, labels, helper text): `tantu-theme`.

## Public API

```rust
// tantu-layout
/// One laid-out cluster: its byte range and horizontal extent in the paragraph.
#[derive(Clone, Debug, PartialEq)]
pub struct ClusterGeometry { pub range: Range<usize>, pub left: f32, pub right: f32 }

/// One laid-out line.
#[derive(Clone, Debug, PartialEq)]
pub struct LineGeometry {
    pub range: Range<usize>,      // the line's byte range (a trailing newline belongs to it)
    pub top: f32,
    pub height: f32,
    pub baseline: f32,
    pub clusters: Vec<ClusterGeometry>,   // in visual order (left to right)
}

/// The geometry of a laid-out paragraph (what `TextField` needs between frames).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextGeometry { pub lines: Vec<LineGeometry> }

impl TextGeometry {
    /// The line holding `offset` (the last line whose range starts at or before it; a line's
    /// end offset belongs to the next line when one starts there). `None` without lines.
    pub fn line_of(&self, offset: usize) -> Option<usize>;
    /// The caret rect for `offset`: zero width, the line's top and height, at the left edge
    /// of the cluster starting at `offset`, or the right edge of the line's last cluster.
    /// Without lines: a zero rect at the origin.
    pub fn caret(&self, offset: usize) -> Rect;
    /// The offset under `point`: the line containing `point.y` (the first above, the last
    /// below), then the cluster boundary nearest to `point.x` (before a cluster's midpoint
    /// its start, after it its end); left of the line its start, right of it its end
    /// (before a trailing newline).
    pub fn hit_test(&self, point: Point) -> usize;
    /// `hit_test` within line `line` at `x` (for up/down movement).
    pub fn offset_at(&self, line: usize, x: f32) -> usize;
    /// The rects covering `start..end`, one per line touched (from the first to the last
    /// cluster inside the range, the full line height; a line with no cluster in the range,
    /// such as an empty line, gives a 2 px wide rect at its start).
    pub fn selection_rects(&self, start: usize, end: usize) -> Vec<Rect>;
}

// tantu-text
impl TextSystem {
    /// The geometry of `text` laid out as `measure_rich` lays it out (TEXT-SYS-12).
    pub fn geometry(&mut self, text: &str, style: TextStyleKey, spans: &[StyleSpan], max_width: f32, max_lines: Option<u32>) -> TextGeometry;
}

// tantu-view
pub trait TextPainter {
    // ...
    /// The paragraph's geometry; `None` for a painter with no text system. Default `None`.
    fn text_geometry(&mut self, text: &str, style: TextStyleKey, spans: &[StyleSpan], max_width: f32, max_lines: Option<u32>) -> Option<TextGeometry>;
}
impl PaintCx<'_, '_> { pub fn text_geometry(&mut self, /* same */) -> Option<TextGeometry>; }

// tantu-widgets
/// A text input (Flutter's `TextField`, Material 3 outlined look). Controlled through a
/// `TextEditingController`.
pub struct TextField { /* controller, placeholder, max_lines, enabled, on_change, on_submit, style */ }

impl TextField {
    /// A single-line field editing `controller`.
    pub fn new(controller: TextEditingController) -> Self;
    /// Shown in the field (dimmed) while the text is empty.
    pub fn placeholder(self, text: impl Into<String>) -> Self;
    /// `None` for any number of lines (Enter inserts a newline); default `Some(1)`.
    pub fn max_lines(self, max_lines: Option<u32>) -> Self;
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
    /// Called with the text after every edit that changes it.
    pub fn on_change(self, f: impl Fn(String) + 'static) -> Self;
    /// Called with the text on Enter in a single-line field.
    pub fn on_submit(self, f: impl Fn(String) + 'static) -> Self;
    /// The text style (default `TextStyle::body()` at 16 px).
    pub fn style(self, style: TextStyle) -> Self;
}

/// Intents the field binds to chords (and apps can bind elsewhere).
pub struct SelectAllIntent;
pub struct UndoIntent;
pub struct RedoIntent;

// tantu-test
impl WidgetTester {
    /// Taps the found element (focusing a field), then presses and releases a key with the
    /// text of each character of `text`, pumping after each (TEST-WT-09).
    pub fn enter_text(&mut self, finder: &Finder, text: &str);
}
```

Structure: the *box* is a render element with a `RenderLimitedBox` (`max_width` 280) around
a render element with `RenderConstrainedBox(BoxConstraints::new(∞, ∞, 40, ∞))`, so the field
fills a bounded width (280 px in an unbounded one) and is at least 40 px tall. The outer
element, which `Keyed` and finders see and which has the same size, holds the focus, the
pointer and key handlers, the chords and the `FieldPaint` (outline). Inside, a `Padding` of 16 × 8
holds the *editor* element: a `RenderParagraph` of the controller's text (the base style, no
spans; `soft_wrap` false and `max_lines` 1 for a single-line field) with an `EditorPaint`
that paints the selection rects, the text (or the placeholder), the caret, and records the
geometry for the handlers. The editor's origin is at (16, 8) in the box.

Look (Material 3 outlined, 16 px body text):

| Part | Enabled | Focused | Disabled |
|---|---|---|---|
| outline (radius 4, a stroke inside the bounds) | 1 px outline `0xFF79747E` | 2 px primary `0xFF6750A4` | 1 px on-surface 12 % `0x1F1C1B1F` |
| text | on-surface `0xFF1C1B1F` | same | on-surface 38 % `0x611C1B1F` |
| placeholder | on-surface-variant `0xFF49454F` | same | on-surface 38 % |
| selection | primary at 40 % (`with_alpha(0.4)`) | same | none |
| caret | – | 2 px wide primary, the line's height | – |

The caret shows only while focused (and enabled); there is no blink until the animation
item. The box clips its content to its bounds.

Keys, on a focused, enabled field (all on presses, repeats included; handled keys stop):

| Key | Edit |
|---|---|
| a key with `text` (no control/super held), or a `Character` key with none | `insert(text)` |
| Backspace / Delete | `delete(Backward / Forward, Grapheme)`; with control, `Word` |
| Left / Right | `move_caret(…, Grapheme)`; with control `Word`; with shift the selecting form |
| Home / End | `move_caret(…, LineBoundary)`; with shift selecting |
| Up / Down | the offset at the caret's x on the previous/next line (the text's start/end from the first/last line); with shift selecting |
| Enter | single-line: `on_submit(text)`; multi-line: `insert("\n")` |
| primary + A, primary + Z, primary + Shift + Z and primary + Y | `SelectAllIntent`, `UndoIntent`, `RedoIntent` through the shortcut bindings on the box |
| Tab, Escape, others | not handled |

Edits go through `controller.edit` (text changes) or `controller.select` (movement), so undo
coalescing works as the model specifies.

## Behavior

- **LAYOUT-TEXT-13:** `TextGeometry` queries on a hand-built geometry: `line_of`, `caret`,
  `hit_test`, `offset_at` and `selection_rects` behave as documented in the API, including a
  geometry with no lines, points outside the text, and an offset at the end of a line that
  another line continues.
- **TEXT-SYS-12:** `geometry` returns one `LineGeometry` per laid-out line (the same lines as
  `measure_rich`, respecting `max_lines`) with the line's byte range, top, height and
  baseline, and a cluster per shaped cluster with its byte range and left/right extent in
  layout coordinates, so `caret(text.len())` lies at the measured width on the last line and
  `hit_test` at a cluster's left edge gives its start.
- **VIEW-TEXT-08:** `TextPainter::text_geometry` defaults to `None` (`NoTextMeasure`);
  `SystemText` returns the system's geometry; `PaintCx::text_geometry` forwards.
- **WIDGETS-TEXTFIELD-01:** A `TextField` builds the structure above: at least 40 px tall, as
  wide as a bounded width (280 in an unbounded one), with the editor paragraph at (16, 8)
  showing the controller's text (`Finder::text` finds it), single-line by default. The
  paragraph follows the controller: `set_text` shows at the next frame. An empty field shows
  the placeholder in on-surface-variant.
- **WIDGETS-TEXTFIELD-02:** Painting: the outline per the table; while focused, the selection
  rects (primary 40 %) under the text, then the text, then a 2 px caret at the caret
  rect; nothing of those while unfocused. Checked against a golden
  (`crates/tantu-widgets/tests/goldens/text-field.png`).
- **WIDGETS-TEXTFIELD-03:** Focus: the field is focusable and traversable while enabled, and
  a primary press focuses it (`focus_on_press`). Disabled (`enabled(false)`) it draws the
  disabled column, isn't focusable, and ignores keys and the pointer.
- **WIDGETS-TEXTFIELD-04:** Keys edit as the table says: typing "ab", Backspace, Left,
  Shift+Right, control+Left, Home/End, Enter (submit in single-line, newline in multi-line),
  Up/Down between lines at the caret's x. Handled keys stop; Tab moves focus on.
  `on_change` is called with the text after each change of it (not on movement).
- **WIDGETS-TEXTFIELD-05:** Chords: primary + A selects all, primary + Z undoes, primary +
  Shift + Z and primary + Y redo, through `SelectAllIntent`/`UndoIntent`/`RedoIntent`
  bindings on the box (so an app's `Actions` ancestor could override them).
- **WIDGETS-TEXTFIELD-06:** Pointer: a primary press at a point puts the caret at the offset
  under it (in the editor's coordinates, from the last geometry); moving while pressed
  extends the selection to the offset under the pointer (the base stays), also outside the
  field (captured); the cursor over the field is `CursorIcon::Text`.
- **TEST-WT-09:** `enter_text` taps the found element, then presses and releases, for each
  character of `text`, a `Character` key carrying that character as `text`, pumping after
  each, so a focused field ends up with the text appended at its caret.

## Performance and allocation

The geometry is rebuilt at each paint of the editor (one allocation per line and cluster) and
kept in an `Rc<RefCell<..>>` the handlers read; a field repaints on every value change.

## Open questions

Decided by the agent while the user was away (review):

1. **Geometry as plain data from the painter**, kept by the widget, instead of letting
   handlers query the text system (handlers have no text context) or caching parley layouts in
   the widget (the widget crate doesn't see parley). The queries are pure and testable.
2. **Keys through `on_key`, chords through `Shortcuts`/`Actions` intents**: arrows and
   deletion want repeats and modifiers, while the chords are what apps rebind.
3. **No clipboard, no double-click, no scrolling yet** (see Scope), to land the widget now.
4. **The field fills a bounded width** like Flutter's `TextField` in a column, with 280 px in
   an unbounded one (Flutter throws there).
5. **A plain `insert` for typed text**: keys with `text` insert it; a `Character` key without
   `text` (as `WidgetTester::press_key` sends) inserts its character.
