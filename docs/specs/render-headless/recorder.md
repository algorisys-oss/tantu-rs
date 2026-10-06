# Headless recording renderer

- **Status:** Agreed
- **Crate:** `tantu-render-headless`
- **Plan item:** Phase 1, "`tantu-render-headless`: recording renderer for tests"
- **Related:** [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md),
  [scene](../scene/scene.md), [renderer and resources](../scene/renderer.md) (the `Renderer`
  trait, the renderer contract and `RenderReport::for_scene`)

## Purpose

A `Renderer` that draws no pixels and keeps a copy of every Scene it is given, with the target
size, scale factor and report at that moment. Tests use it to assert on what was painted
("element 42 drew a red fill at (10, 10)") without a GPU or a rasterizer, and to test code
that drives a renderer, such as the app runner's resize and retry logic. `tantu-test`'s
`WidgetTester` will build on it (Phase 2).

## Scope

In scope:

- `HeadlessRenderer`: implements `Renderer`; records frames; reports like every other backend
  (`RenderReport::for_scene`); registers custom kinds as handled; can be told to fail the next
  frame.
- `RecordedFrame`: the Scene copy, target size, scale factor, resources revision and report.
- Small queries tests need: the last frame, taking all frames, entries of one element.

Out of scope:

- Pixels. Golden images come from `tantu-render-soft`.
- Recording resources (image pixels, font bytes). Only the revision is kept; tests that need
  the data look it up in their own `Resources`.
- Limits on how many frames are kept. Tests take frames as they go; see open question 2.
- Pretty-printing Scenes or snapshot files. Possible later, with the serde question in
  scene.md.

## Public API

Crate root `tantu_render_headless`.

```rust
use tantu_scene::{CustomKind, ElementId, Entry, RenderError, RenderReport, Renderer, Resources, Scene};

/// One frame given to a `HeadlessRenderer`.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedFrame {
    /// A copy of the Scene as rendered.
    pub scene: Scene,
    /// Target width in physical pixels at the time.
    pub width: u32,
    /// Target height in physical pixels at the time.
    pub height: u32,
    /// Scale factor in effect (after replacing an invalid one with 1).
    pub scale_factor: f32,
    /// `Resources::revision()` at the time.
    pub resources_revision: u64,
    /// The report `render` returned.
    pub report: RenderReport,
}

impl RecordedFrame {
    /// The entries painted for `element`, in paint order.
    pub fn entries_for(&self, element: ElementId) -> impl Iterator<Item = &Entry> + '_;
}

/// A renderer that records Scenes instead of drawing them.
#[derive(Debug)]
pub struct HeadlessRenderer { /* private */ }

impl HeadlessRenderer {
    /// A renderer with a `width × height` physical-pixel target, scale factor 1, no frames.
    pub fn new(width: u32, height: u32) -> HeadlessRenderer;

    /// Current target width and height in physical pixels.
    pub fn size(&self) -> (u32, u32);
    /// Current scale factor.
    pub fn scale_factor(&self) -> f32;

    /// Treat `kind` as having a handler: its custom commands are not counted as unhandled.
    pub fn register_custom(&mut self, kind: CustomKind);

    /// Make the next `render` call return `Err(error)` and record nothing.
    pub fn fail_next_render(&mut self, error: RenderError);

    /// Frames recorded and not yet taken, oldest first.
    pub fn frames(&self) -> &[RecordedFrame];
    /// The most recent frame not yet taken.
    pub fn last_frame(&self) -> Option<&RecordedFrame>;
    /// Removes and returns the frames recorded so far, oldest first.
    pub fn take_frames(&mut self) -> Vec<RecordedFrame>;
    /// Frames recorded since creation, including taken ones.
    pub fn frame_count(&self) -> u64;
}

impl Renderer for HeadlessRenderer { /* resize, render */ }
```

## Behavior

- **RENDER-HEADLESS-01:** `new(w, h)` has `size() == (w, h)`, `scale_factor() == 1.0`, no frames
  and `frame_count() == 0`. Any `w` and `h` are accepted, 0 included.
- **RENDER-HEADLESS-02:** `resize(w, h, s)` sets the size to `(w, h)` and the scale factor to `s`,
  or to 1 if `s` is not finite or not positive.
- **RENDER-HEADLESS-03:** `render(scene, resources)` appends a `RecordedFrame` with a copy of
  `scene` (equal to it), the current size and scale factor, `resources.revision()` and the
  report; increments `frame_count`; and returns `Ok` with that same report. Later changes to
  `scene` don't change the recorded copy.
- **RENDER-HEADLESS-04:** The report is
  `RenderReport::for_scene(scene, resources, &|kind| <kind was registered>)`.
- **RENDER-HEADLESS-05:** With a width or height of 0, `render` still records the frame, but its
  report is `RenderReport::default()` (contract item 3: nothing is drawn, so nothing is missing).
- **RENDER-HEADLESS-06:** `register_custom(kind)` makes later frames treat `kind` as handled.
  Registering a kind twice is the same as once. Kinds not registered are counted in
  `unhandled_custom`.
- **RENDER-HEADLESS-07:** After `fail_next_render(e)`, the next `render` returns `Err(e)`, records
  nothing and leaves `frame_count` unchanged; the render after that behaves normally. A second
  `fail_next_render` before that render replaces the first error.
- **RENDER-HEADLESS-08:** `frames()` lists untaken frames oldest first and `last_frame()` is the
  newest of them. `take_frames()` returns them in the same order and leaves none;
  `frame_count()` is unaffected by taking.
- **RENDER-HEADLESS-09:** `RecordedFrame::entries_for(e)` yields, in paint order, exactly the
  entries of the recorded Scene whose `element` is `Some(e)`.
- **RENDER-HEADLESS-10:** `HeadlessRenderer` and `RecordedFrame` are `Send`, and the renderer
  works as a `Box<dyn Renderer>`.

## Performance and allocation

Recording copies the Scene, so `render` allocates on every frame. This renderer is for tests
and is exempt from the no-allocation-per-frame rule. `RenderReport::for_scene` itself does not
allocate.

## Open questions

Resolved (2026-10-06, the proposals were accepted):

1. **Rule id area.** `RENDER-HEADLESS-NN` (crate name in capitals, no separate feature segment),
   so tests read `render_headless_07_...`. The soft renderer uses `RENDER-SOFT-NN`.
2. **Unbounded history.** Every frame is kept until `take_frames`. A test pumping thousands of
   frames takes them as it goes. A `max_frames` ring buffer can be added if `tantu-test` needs
   one.
3. **`fail_next_render`.** A test hook for the app runner's handling of `TargetLost`. Only one
   error can be queued.
4. **`#![forbid(unsafe_code)]`.** Forbidden here even though AGENTS.md exempts `tantu-render-*`
   crates: this one needs no `unsafe`.
