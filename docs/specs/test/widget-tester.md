# WidgetTester

- **Status:** Agreed (the user said "continue" on the draft, taking its proposals)
- **Crate:** `tantu-test` (plus `Keyed` in `tantu-view`)
- **Plan item:** Phase 2, "`tantu-test::WidgetTester` (pump, tap, type, find by key)"
- **Related:** [view tree](../view/tree.md), [frame](../view/frame.md),
  [events](../view/events.md), [basic widgets](../widgets/basic.md),
  [software renderer](../render-soft/renderer.md), AGENTS.md "Testing expectations"

## Purpose

AGENTS.md says widgets are tested with `tantu-test::WidgetTester`: pump frames, simulate
input, and find elements. Today each widget test builds its own text system, frame loop,
pointer events and element lookup (see `crates/tantu-widgets/tests/basic.rs`). `WidgetTester`
packages that as one object, Flutter's `WidgetTester` with Rust names:

- it owns a view tree, a text system with a bundled test font, and a headless recorder;
- `pump` runs a frame;
- `tap` and `hover` send pointer events at a found element's center;
- finders locate elements by key, by text or by their render object's type;
- `matches_golden` renders the last frame with the software renderer and compares it with a
  PNG.

## Scope

In scope:

- `WidgetTester`: `new`, `with_size`, `pump`, `frame`, `tap`, `hover`, `pointer`, `find`,
  `find_all`, `rect`, `text`, `enter` (run code in the tree's runtime), `matches_golden`.
- `Finder`: `Finder::key`, `Finder::text`, `Finder::render::<R>()`.
- `Keyed` (tantu-view): gives the element its child builds a `Key`, which finders and later
  tools (inspector, accessibility) look up.
- A bundled test font (Liberation Sans, OFL), so text in tests doesn't depend on installed
  fonts.

Out of scope (and where it goes):

- **"type"** (keyboard text entry): needs focus and keyboard dispatch, Phase 3 "Focus
  system". `enter_text` arrives with `TextField`.
- Semantics finders (by role or label): with `tantu-a11y`, Phase 3.
- Gestures with timing (long press, fling), animations and fake time: Phase 3 animation.

## Public API

```rust
// tantu-view
/// An identity for an element, for finding it (tests, inspector).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(pub Arc<str>);

/// Gives the element `child` builds the key `key`. A region child (`Dyn`, ...) is keyed as
/// the region element.
pub struct Keyed<V> { /* key, child */ }
impl<V: View> Keyed<V> { pub fn new(key: impl Into<Key>, child: V) -> Self; }

impl ViewTree {
    /// The elements with `key`, in tree order (removed elements drop their key).
    pub fn find_key(&self, key: &Key) -> Vec<ElementId>;
}

// tantu-test
pub struct WidgetTester { /* tree, text system, resources, recorder, size */ }

impl WidgetTester {
    /// Builds `app` in an 800 × 600 window (logical, scale 1) with the test font as default
    /// family, and pumps the first frame.
    pub fn new<V: View>(app: impl FnOnce() -> V) -> Self;
    /// Same, in a `width` × `height` window.
    pub fn with_size<V: View>(width: f32, height: f32, app: impl FnOnce() -> V) -> Self;
    /// Runs one frame (apply updates, layout, paint, record); returns whether anything had
    /// changed (`ViewTree::needs_frame` before it).
    pub fn pump(&mut self) -> bool;
    /// The last recorded frame.
    pub fn frame(&self) -> &RecordedFrame;
    /// Primary press and release at the center of the single element `finder` matches, then a
    /// pump. Panics with a readable message if it matches no element or several.
    pub fn tap(&mut self, finder: &Finder);
    /// A pointer move to the element's center, then a pump.
    pub fn hover(&mut self, finder: &Finder);
    /// Dispatches a raw pointer event (no pump).
    pub fn pointer(&mut self, event: PointerEvent) -> bool;
    /// The single element `finder` matches (panics otherwise, naming the finder).
    pub fn find(&self, finder: &Finder) -> ElementId;
    /// Every element `finder` matches, in tree order.
    pub fn find_all(&self, finder: &Finder) -> Vec<ElementId>;
    /// The element's bounds in window coordinates (from the last layout).
    pub fn rect(&self, element: ElementId) -> Rect;
    /// The text a `Text` element shows (its `RenderParagraph`'s text).
    pub fn text(&self, element: ElementId) -> Option<String>;
    /// Runs `f` with the tree's runtime current (read or write signals).
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R;
    /// Renders the last frame with the software renderer and compares it with the PNG at
    /// `path` (tolerance 2, as the renderer goldens). With `TANTU_UPDATE_GOLDENS=1` it writes
    /// the PNG instead. Panics on a mismatch, naming the file and the difference.
    pub fn matches_golden(&mut self, path: impl AsRef<Path>);
}

/// Which elements to find.
pub enum Finder { /* private */ }

impl Finder {
    /// Elements with this key (`Keyed`).
    pub fn key(key: impl Into<Key>) -> Self;
    /// `Text` elements showing exactly `text`.
    pub fn text(text: impl Into<String>) -> Self;
    /// Render elements whose render object is an `R`.
    pub fn render<R: RenderBox>() -> Self;
}
```

## Behavior

- **TEST-WT-01:** `new` builds the view, pumps one frame at 800 × 600 and records it.
  `with_size` uses the given size. Text in the bundled font has glyphs, so a `Text("Hi")` frame
  holds a glyph run with 2 glyphs.
- **TEST-WT-02:** `pump` applies queued updates, lays out, paints and records one frame. It
  returns `true` when the tree needed a frame and `false` otherwise (it still records).
- **TEST-WT-03:** Finders: `key` finds `Keyed` elements; `text` finds `Text` elements by exact
  text; `render::<R>()` finds elements by render object type; results are in tree order.
  `find` panics, naming the finder, when there are zero or several matches.
- **TEST-WT-04:** `tap` presses and releases the primary button at the center of the found
  element, then pumps. Tapping a `Button` calls its `on_press` once, and a counter built like
  the AGENTS.md snippet shows "Count: 1" afterwards. `hover` moves the pointer there and pumps,
  so the button shows its hover color.
- **TEST-WT-05:** `rect` is the element's window-space bounds from the last layout (offsets of
  its render ancestors summed); `text` is a `Text` element's current text, `None` for other
  elements.
- **TEST-WT-06:** `matches_golden` passes when the software rendering of the last frame matches
  the PNG within tolerance 2. It panics on a mismatch or a missing file, naming the file. It
  writes the PNG when `TANTU_UPDATE_GOLDENS=1`.
- **VIEW-KEY-01:** `Keyed` gives the element its child builds the key; `find_key` returns the
  keyed elements in tree order; removing an element removes its key; a region child is keyed as
  the region.

## Open questions

Resolved (2026-10-10; the user said "continue" on the draft):

1. **"type" is deferred to Phase 3.**
2. **Keys go through a `Keyed` wrapper view.** A `key:` sugar can come later.
3. **`tantu-test` bundles its own copy of the test font.**
4. **Finders and asserts panic**, as test helpers do.

Settled while agreeing: a region's `rect` is the union of its render children's rects (empty
at the origin when there are none). A golden's size is the window size at scale 1.
