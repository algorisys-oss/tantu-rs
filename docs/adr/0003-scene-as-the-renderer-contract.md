# 0003. Scene as the renderer contract

- **Status:** Accepted
- **Date:** 2026-09-30

## Context

Renderer independence is one of Tantu's two non-negotiable goals. The same UI code must draw with
a GPU backend (wgpu), a CPU backend (tiny-skia) and a headless recorder for tests, and later
possibly on the web or on a remote machine.

Knots does this by emitting a `render.Packet` and Clay by emitting a list of render commands.
Both show that a UI can hand a plain data structure to whatever draws it.

## Decision

UI, layout and widget code produce a **`Scene`**: a flat, versioned, serializable list of draw
commands. A `Renderer` trait in `tantu-scene` consumes it.

- **Commands** cover rects, rounded rects, borders, shadows, paths, glyph runs, images, clips,
  transforms, layers (opacity, blend), an overlay color, and **custom** commands as the escape
  hatch for charts, 3D viewports and other user-drawn content.
- **Every command carries its element id and z-index.** Retained backends can diff commands by id,
  and hit areas, debugging and damage tracking can map commands back to elements.
- Images and fonts are referenced by **resource handles**, not embedded in the Scene.
- The Scene carries a **damage region**, so a backend can repaint only what changed.
- **Off-screen content is culled** before it reaches the Scene.
- **Renderers never call back into the UI.** Drawing is a pure function of the Scene and the
  resources.

The exact command set and its encoding are defined in the `tantu-scene` spec (Phase 1).

## Consequences

- One UI codebase, many backends. Adding a backend means implementing `Renderer`, with no changes
  to widgets.
- Tests can assert on recorded Scenes (headless), and golden tests can compare PNGs (software).
  The Phase 1 milestone checks that a hand-built Scene renders identically in wgpu and software.
- Streaming a Scene to a remote client, or diffing it into the DOM, becomes possible later.
- Text is shaped before it enters the Scene (glyph runs), so every backend needs glyph
  rasterization. Sharing a glyph cache between backends is a question for the text and renderer
  specs.
- Building a Scene each frame costs memory. The command buffer must be reused between frames, in
  line with the no-allocation-per-frame rule.
- Each backend must decide what to do with custom commands it can't draw. The `tantu-scene` spec
  has to define that.

## Alternatives considered

- **A drawing trait called during paint** (`canvas.draw_rect(...)` straight into the backend).
  Couples paint timing to the backend, makes recording and golden tests harder, and rules out
  diffing and remote rendering.
- **A retained, mutable scene graph** as the contract (a compositor tree). More complex for every
  backend. Layer caching can still be added on top of the flat Scene later (Phase 5).
