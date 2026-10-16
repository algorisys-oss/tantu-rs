# Damage tracking and partial repaint (part 1)

- **Status:** Draft (2026-10-16; drafted by the agent for the user's review)
- **Crates:** `tantu-scene` (damage from two Scenes, report counts), `tantu-render-soft`
  (partial repaint), `tantu-render-headless`, `tantu` (the runner keeps the previous Scene)
- **Plan item:** Phase 5, "Damage tracking and partial repaint; layer caching; GPU texture
  atlas tuning", part 1 (damage, and partial repaint in the software renderer)
- **Related:** [Scene](scene.md) (`Damage`, `add_damage`), [renderer](renderer.md),
  [render-soft](../render-soft/renderer.md), [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md)

## Purpose

Every frame today repaints the whole window, even when a caret blinks or a counter's text
changes. The `Scene` has carried a damage region since Phase 1 (`Damage::Full` or a list of
rects), but nothing fills it in and no renderer reads it. Partial repaint matters most for
the software renderer (VMs, remote desktops, CI), which spends its time rasterizing pixels,
and it is the first step of the Phase 5 performance work.

## Design

Damage is computed **by diffing Scenes, by element id**, in `tantu-scene`, the way a
retained backend would (ADR 0003): every draw command carries its element id, so two
frames can be compared per element without the view layer tracking what it repainted.
For each element whose draw commands changed (the command, or its transform, clip, layer
opacity or z-order context), the damage is the union of where it was drawn and where it is
drawn now, in window coordinates. The view layer stays unchanged.

## Scope

In scope: `Scene::diff_damage`; conservative window-space bounds for every draw command
(glyph runs from their glyph positions); coalescing; the runner keeping the previous
Scene and asking for full damage when it must (first frame, resize, scale change, lost
target); the software renderer repainting only the damaged rects of its retained pixmap,
pixel-identical to a full repaint; counts in `RenderReport`.

Out of scope (later parts of the item): partial repaint in the wgpu renderer (its
swapchain images aren't preserved between frames, so it needs a retained target; until
then it repaints fully, which the contract allows); presenting damage to the compositor;
layer caching; glyph-atlas tuning.

## Public API

```rust
// tantu-scene
impl Scene {
    /// Sets this frame's damage from what changed since `previous` (SCENE-DAMAGE-01..05),
    /// replacing any damage recorded while building. A different size gives full damage.
    pub fn diff_damage(&mut self, previous: &Scene);
}
impl RenderReport {
    /// Pixels the renderer repainted this frame (0 when the backend doesn't count).
    pub repainted_pixels: u64,
}
```

## Behavior

- **SCENE-DAMAGE-01:** Identical Scenes give empty damage (`Damage::Rects(&[])`); a Scene
  of a different size than `previous` gives `Damage::Full`.
- **SCENE-DAMAGE-02:** Draw commands are grouped by element id (commands without one form
  one group). A group whose sequence of (command, accumulated transform, clip, layer
  opacity and overlay, glyph or custom data) differs between the Scenes is damaged: the
  union of its commands' bounds in both Scenes. Unchanged groups add nothing, wherever
  their commands sit in the list.
- **SCENE-DAMAGE-03:** A command's bounds are its shape's (fills, strokes widened by half
  the stroke width), a shadow's blurred and spread rect, an image's destination, a custom
  command's bounds, or for a glyph run the box around its glyph positions widened by the
  font size on every side; mapped by the transform's bounding box and clipped to the
  current clip's bounding box and to the frame.
- **SCENE-DAMAGE-04:** Damage rects are coalesced: overlapping or touching rects merge,
  and when more than 16 remain, or their area exceeds 60 % of the frame, the damage is
  `Full`.
- **SCENE-DAMAGE-05:** `diff_damage` replaces damage recorded while building and doesn't
  change the entries; it allocates nothing once its buffers are warm (it keeps them in
  the Scene).
- **SCENE-RENDER-09:** `RenderReport::repainted_pixels` counts the device pixels a
  renderer repainted; renderers that don't track it report 0.
- **RENDER-SOFT-29:** The software renderer keeps its pixmap between frames. With
  `Damage::Rects` on a frame of the same size and scale as the last one, it clears and
  repaints only the damaged rects (in device pixels, rounded outward), drawing every
  command clipped to them; with `Full`, a new size or scale factor, or after an error, it
  repaints everything. `repainted_pixels` reports the area.
- **RENDER-SOFT-30:** A partial repaint is pixel-identical to a full repaint of the same
  Scene (checked over the conformance Scenes changed one element at a time).
- **RENDER-HEADLESS-11:** The headless renderer reports the damage's area as
  `repainted_pixels` (the whole frame for `Full`), so tests can see what would repaint.
- **FACADE-APP-22:** The runner keeps each window's previous Scene and calls
  `diff_damage` before rendering; it asks for full damage on a window's first frame,
  after a resize or scale change, and after a lost target.

## Performance and allocation

Diffing walks both Scenes once (hash per group, kept in reusable buffers); it is measured
against the 10 k-object paint in the layout benchmark and must stay under 10 % of a
frame's paint time there.

## Open questions

Proposals from the agent, for review:

1. **Diff Scenes by element id** in `tantu-scene` rather than dirty-tracking in the view
   layer: it needs no bookkeeping in widgets, can't miss a change a widget forgot to
   report, and works for any producer of Scenes.
2. **Conservative glyph-run bounds** from glyph positions widened by the font size (no
   font metrics in `tantu-scene`).
3. **Coalescing at 16 rects / 60 % area**, then full.
4. **Software renderer first**; wgpu partial repaint (a retained target texture plus a
   copy to the surface) and compositor damage are later parts.
5. **Pixel-identical partial repaint** is the correctness rule, tested against full
   repaints, not just "looks right".
6. **The runner keeps the previous Scene** (two Scenes per window, swapped), so the view
   layer and `ViewTree::frame` are unchanged.
