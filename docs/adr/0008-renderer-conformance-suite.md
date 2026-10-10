# 0008. Renderer conformance suite

- **Status:** Accepted (the user left the choice to the agent, "your pick", 2026-10-10)
- **Date:** 2026-10-10
- **Related:** [ADR 0003](0003-scene-as-the-renderer-contract.md) (Scene as the renderer
  contract), spec [`render-conformance/conformance.md`](../specs/render-conformance/conformance.md)

## Context

ADR 0003 makes the `Scene` the contract between the UI and every renderer, and promises that one
Scene looks the same on all of them. Tantu has two pixel backends today (wgpu and tiny-skia) and
plans more: a browser backend (WebGPU or canvas), a painter that lets a Tantu UI be hosted inside
eframe/egui, remote rendering. Some of these may be written outside this repository.

Each backend's spec restates the drawing rules, and its tests probe pixels well inside and
outside shapes. That leaves edges, blur falloff and whole frames unchecked, and it gives a new
backend no ready-made way to prove it matches. The Phase 1 milestone (wgpu against the software
goldens) needed exactly such a check, and the reference Scenes and goldens were test files
inside `tantu-render-soft`, where no other crate could reach them.

Pixel equality across backends is not achievable: rasterizers anti-alias and blur differently,
and GPUs round differently.

## Decision

We will keep a renderer conformance kit in its own crate, `tantu-render-conformance`, holding:

- the reference Scenes (one per area of the command set, more added as the Scene grows, for
  example text in Phase 2);
- their goldens, rendered by `tantu-render-soft` (the reference backend) and embedded in the
  crate;
- `match_images`, a comparison of premultiplied pixels that holds interior pixels to a tight
  tolerance and edge pixels to a looser one, with a cap on how many edge pixels may differ
  noticeably.

The crate depends only on `tantu-core`, `tantu-scene` and a PNG decoder, never on a rasterizer.
Every backend takes it as a dev-dependency and has one test that renders each reference Scene
and checks it against its golden with `MatchTolerance::CROSS_BACKEND`. A backend conforms when
that test passes. The software renderer is held to a stricter check against its own goldens.

## Consequences

- A new backend, including a third party's, gets a conformance test from one dev-dependency and a
  dozen lines, without pulling in tiny-skia or wgpu. Embedded goldens also work where there is
  no file system (wasm test runners).
- The software renderer is the reference. A change to its output means regenerating the goldens
  (`TANTU_UPDATE_GOLDENS=1`) and then checking that the other backends still conform.
- Adding a Scene command means adding (or extending) a reference Scene and its golden, so every
  backend is checked for it.
- The tolerances are a judgment call backed by measurements; loosening them needs a spec change
  with new measurements.
- One more crate in the workspace. It is test tooling, but it is published with the others when
  the time comes, so third-party backends can use it.

## Alternatives considered

- **Reference Scenes as a test file included by path** from the wgpu tests. No new crate, but
  only works inside this repository and doesn't help any other backend.
- **A public module in `tantu-render-soft`.** Puts test data into a renderer's API and makes
  every backend's tests depend on tiny-skia.
- **Putting it in `tantu-test`.** That crate is for widget tests and depends on `tantu-view`; a
  renderer shouldn't need the widget layer to be tested. `tantu-test` will use this crate instead.
- **Exact or near-exact pixel equality** (as the software renderer's own goldens use). Fails on
  anti-aliasing noise between any two rasterizers, so it would force tolerances so loose they
  would also hide real errors.
