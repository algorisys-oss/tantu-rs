# HANDOFF.md

Where the project stands, so the next session (human or agent) can pick up without re-reading the
whole history. Update this file in every commit (see `AGENTS.md` → General rules).

_Last updated: 2026-10-10_

## Resume here (session of 2026-10-10)

**Phase 1 is done. Phase 2: `tantu-layout` (all layouts), text measurement (ADR 0010) and the
layout benchmark are done** (10k-node full layout in 635 µs, target < 1 ms). Specs in
`docs/specs/layout/` are all Implemented.

The user is away for ~5 hours from 2026-10-10 and asked the agent to continue and push at each
milestone, deciding forks with the recommended option and marking them for review (decision
34). Their answers before leaving: components **run once with fine-grained reactive props**
(Solid/Leptos style), order **as PLAN.md** (tantu-view, then tantu-text, events, facade,
WidgetTester), and AGENTS.md now says every crate may depend on `tantu-core`.

Progress while the user is away: **the `tantu-view` item is done** (ADR 0011, five specs in
`docs/specs/view/`, all Implemented): `tree.md` (VIEW-TREE-01..11), `paint.md`
(VIEW-PAINT-01..06), `frame.md` (VIEW-FRAME-01..08), `dynamic.md` (VIEW-DYN-01..07),
`layout-builder.md` (VIEW-LB-01..06). **`tantu-text`'s text system is done**
(`docs/specs/text/system.md`, TEXT-SYS-01..09): parley 0.12 behind `TextMeasure`, glyph runs
into Scenes. MSRV raised to 1.88 for parley (decision 12), and AGENTS.md now lets `tantu-text`
depend on `tantu-layout` (decision 36, review). Next: "text in views" (frames that measure and
paint text), then glyph rasterization in the renderers, event dispatch, the facade with the
counter example, `WidgetTester`.

When resuming, tell the agent: "Read HANDOFF.md and continue."

## Current state

- **Name:** **Tantu** (pronounced "tan-too", Sanskrit for "thread"). Tagline: *Compose once. Render
  your way.* Crates: `tantu`, `tantu-core`, `tantu-reactive`, `tantu-layout`, `tantu-widgets`,
  `tantu-render-wgpu`, … (full list in AGENTS.md). App import: `use tantu::prelude::*;`
- **Phase:** Phases 0 (Foundations) and 1 (Pixels on screen) are done; Phase 2 (Layout, views
  and text) is in progress: `tantu-layout` (constraints, layout tree, all Flutter layout
  objects in scope, text measurement, benchmarks) is done. Phase 1 delivered:
  `tantu-scene`, `tantu-render-headless`, `tantu-render-soft`, `tantu-platform`,
  `tantu-platform-winit`, `tantu-render-wgpu`, `examples/scene-window` and the milestone
  (`tantu-render-conformance`, decision 31). The workspace has 17 crates under `crates/`
  (the AGENTS.md table), with the internal dependency edges from that table already declared.
  `tantu-core` is complete: geometry, color and arena modules (specs in `docs/specs/core/`,
  all Implemented). `tantu-reactive` is implemented (specs `docs/specs/reactive/signals.md` and
  `benchmarks.md`, both Implemented; `cargo bench -p tantu-reactive`). `tantu-scene` is
  implemented (see decisions 23 and 24), and so are `tantu-render-headless` and
  `tantu-render-soft` (decisions 25 and 26), and `tantu-platform` with `tantu-platform-winit`
  (decision 28), `tantu-render-wgpu` (decision 29), the `scene-window` demo (decision 30) and
  `tantu-render-conformance` (decision 31).
  The other crates are still empty.
- **Repo:** https://github.com/algorisys-oss/tantu-rs (public). Branch: `main`.
- **Files:**
  - `AGENTS.md`: architecture, crate layout, dependency rules, conventions, workflow
  - `CLAUDE.md`: Claude Code notes (imports AGENTS.md and PLAN.md)
  - `PLAN.md`: roadmap, comparisons with Knots and Clay, ADR list, phases
  - `README.md`: public-facing overview
  - `docs/architecture.md`: architecture write-up; diagram in `docs/architecture/`
    (`tantu-architecture.svg` is the source, `.png` is rendered from it, `rough-sketch.png` is the original sketch)
  - `LICENSE-MIT`, `LICENSE-APACHE`: dual license
  - `Cargo.toml`: workspace manifest (`[workspace.package]`, internal crates in
    `[workspace.dependencies]`, shared lints in `[workspace.lints]`)
  - `rust-toolchain.toml`: pins Rust 1.85, which is also the MSRV; `clippy.toml`
  - `crates/`: one directory per crate; `crates/tantu` is the facade
  - `docs/adr/`: ADRs 0001–0010 plus `README.md` (index, template, how to supersede)
  - `docs/specs/`: `TEMPLATE.md` and `README.md` (location, rule-id and test-name conventions)
  - `.github/workflows/ci.yml`: fmt, clippy + rustdoc (`-D warnings`), test on Linux/Windows/macOS,
    spec coverage
  - `xtask/`: dev tooling (`cargo xtask spec-coverage`); alias in `.cargo/config.toml`

## Decisions made so far

1. **Flutter-like structure is the API.** Apps compose widgets; layout uses Flutter's layout widgets
   (`Row`, `Column`, `Expanded`, `Padding`, `Stack`, …) and its constraint protocol, with Flutter
   names in snake_case. No alternative layout vocabulary.
2. **Retained tree + fine-grained signals.** Not immediate mode, unlike Knots and Clay.
3. **Renderer independence via `Scene`,** a data-only display list (from Knots' `render.Packet`
   and Clay's render commands). Backends: wgpu, tiny-skia (software), headless.
4. **From Clay, only internals:**
   - text measurement through a trait, with a per-word cache
   - element id + z-index on each Scene command
   - culling of off-screen content
   - anchored overlay positioning
   - arena storage, no allocation per frame
   - devtools drawn as Scene commands

   Clay's Fit/Grow sizing was explicitly **rejected**.
5. **Platform:** winit behind a `Platform` trait; Linux targets both Wayland and X11.
6. **Text:** parley/swash/fontique. **A11y:** AccessKit.
7. **Workflow:** Plan → Spec (`docs/specs/`) → failing unit tests → implementation.
8. **License:** MIT OR Apache-2.0. Copyright: "Rajesh Pillai - Algorisys Technologies"
   (use the same in `[workspace.package] authors`).
9. **Architecture doc is kept current.** `docs/architecture.md` and its diagram must be updated in
   the same change as any design decision that affects them (rule in AGENTS.md).
10. **Name: Tantu** (was dkui). Chosen over YappyUI, Vayra and keeping dkui: short, has a story
    (the thread metaphor stands for composition, not multithreading), and doesn't tie the brand to
    Rust or a backend. Room for an ecosystem: Tantu Inspector, Studio, Themes, Gallery. Attribution
    "by Algorisys". Crate prefix `tantu-`; golden-update env var `TANTU_UPDATE_GOLDENS`.
11. **Public from the start** (2026-09-30), not at 0.1 as first planned. Repo: `algorisys-oss/tantu-rs`.
12. **MSRV = Rust 1.87** (was 1.85, the first release with edition 2024; raised on 2026-10-06
    for wgpu 30), pinned in `rust-toolchain.toml`, so local and CI builds run the MSRV itself.
    `rust-version` in `Cargo.toml` must match. Raise both together when a dependency needs a
    newer compiler.
13. **Workspace setup:** crates live in `crates/<name>`. Internal crates are declared once in
    `[workspace.dependencies]` and each crate depends only on what the AGENTS.md table allows.
    Shared lints: `missing_docs`, `unsafe_op_in_unsafe_fn`, `clippy::undocumented_unsafe_blocks`,
    `clippy::unwrap_used`, `clippy::print_stdout`/`print_stderr` (unwrap/print allowed in tests via
    `clippy.toml`). `#![forbid(unsafe_code)]` in every crate except `tantu-platform-*` and
    `tantu-render-*` (`tantu-core` lost its exception with the arena, decision 19). Facade features: `wgpu`, `winit`, `default-theme`
    (default) and `soft`. `Cargo.lock` is committed.
14. **No PRs for now.** Rajesh is the only developer: do feature work on a branch, then
    fast-forward merge into `main` and push. CI runs on the push to `main`.
15. **ADRs and specs conventions.** ADRs 0001–0006 are Accepted and record decisions 1–6 above
    (retained + signals, Flutter layout, Scene contract, Platform trait, text stack, wgpu). Accepted
    ADRs are not rewritten; a new ADR supersedes them. Specs live in `docs/specs/<crate>/<feature>.md`
    (crate name without `tantu-`), with rule ids like `LAYOUT-FLEX-03` that are never renumbered,
    and statuses Draft → Agreed → Implemented.
16. **Spec → TDD → code for all code, one commit per step** (2026-09-30). Every item lands as
    `spec:` (spec Agreed), `test:` (failing tests) and `impl:` commits, pushed together. This
    covers tooling too. `HANDOFF.md` is updated in the last commit of each push (it used to be every
    commit). `cargo xtask spec-coverage` (in CI) fails when a rule of an Agreed/Implemented spec has
    no test named after it (`LAYOUT-FLEX-03` → `layout_flex_03` or `layout_flex_03_*`). `xtask` is
    dev tooling: in the AGENTS.md table, not in the architecture diagram.
17. **Geometry shape** (2026-09-30, spec `docs/specs/core/geometry.md`):
    - `Point` (where) and `Vec2` (how far) are separate types, unlike Flutter's single `Offset`.
      The parent-to-child offset in layout is a `Vec2`.
    - `EdgeInsets`, Flutter's name, not `Insets`.
    - `Rect` stores edges (`left, top, right, bottom`) like Flutter.
    - Values are stored as given: no clamping, no reordering, nothing panics, NaN propagates. The
      one exception is `EdgeInsets::deflate_size`, which clamps to 0 as layout needs.
    - `Affine::inverse` is computed in f64, but whether a transform is invertible follows the f32
      `determinant()`, so the two agree. Its accuracy is only promised for well-conditioned
      transforms (CORE-GEOM-28 was corrected when writing the tests).
18. **Color shape** (2026-09-30, spec `docs/specs/core/color.md`):
    - `Color { r, g, b, a: f32 }`, sRGB-encoded with straight alpha, 16 bytes, stored as given.
      Chosen over packed `u8` for banding-free animation, wgpu's float input and room for wide
      gamut (Flutter made the same move in 3.27).
    - Packed form is Flutter's `0xAARRGGBB` only (`from_argb32`/`to_argb32`); no CSS-order variant.
    - sRGB ↔ linear conversion lives in `tantu-core` (`to_linear`/`from_linear`, returning and
      taking `[f32; 4]`) so every renderer uses the same function. Both map 0 and 1 exactly
      (encoding is evaluated in f64 for that).
    - Named palettes belong to `tantu-theme`, gradients and blend modes to the Scene.
19. **Arena shape** (2026-09-30, spec `docs/specs/core/arena.md`):
    - One untyped `Id { index: u32, generation: NonZeroU32 }` (8 bytes, `Option<Id>` too).
      Crates wrap it in newtypes (`ElementId(Id)`) for type safety.
    - Stable `to_bits`: generation in the high 32 bits, index in the low 32 bits, never 0. Used
      for Scene element ids and AccessKit node ids.
    - Written in-house, no `slotmap` (`tantu-core` stays dependency-free) and no `unsafe`: slots
      are a `Vec` of an enum, `get_pair_mut` uses `split_at_mut`. `tantu-core` is
      `#![forbid(unsafe_code)]`, and the AGENTS.md exception for it was removed.
    - Freed slots are reused LIFO. A slot whose generation would overflow is retired, never
      reused. Stale and foreign ids return `None`, never panic; no `Index` impls.
20. **Reactive shape** (2026-09-30, spec `docs/specs/reactive/signals.md`, decided in autopilot):
    - An explicit `Runtime` per app/window, entered with `Runtime::enter`; a thread-local stack
      records the current one, so view code writes `signal(0)` with no context argument.
    - `Copy` handles (`Signal`, `Memo`, `Effect`, `Scope`): arena `Id` + runtime id, 16 bytes.
      A handle never resolves in another runtime.
    - Push-pull, glitch-free graph (clean/check/dirty, as in Reactively/Leptos). Memos are lazy
      with `PartialEq` cut-off; `Signal::set` always notifies. Effects run synchronously when the
      write or outermost `batch` ends; a frame scheduler is left to `tantu-view`.
    - Ownership: nodes belong to the scope or computation that created them; re-runs dispose
      the previous run's nodes; `on_cleanup` runs children first, then in registration order.
    - **ADR 0007**: reads of disposed handles panic, `try_*` reads return `None`, writes are
      ignored. It supersedes ADR 0001's "no panics in release builds", which `get() -> T` can't
      meet. Review it; it was decided without you.
21. **Reactive benchmarks** (2026-09-30, spec `docs/specs/reactive/benchmarks.md`, autopilot):
    - `criterion` 0.7 as a dev-dependency of `tantu-reactive` with default features off (no
      plotting or rayon). It is the benchmark harness used by taffy, kurbo and vello.
    - Scenarios live in `benches/scenarios.rs` and are included by both the bench and
      `tests/bench_scenarios.rs`, so what is timed is also checked. `autobenches = false`.
    - No timing assertions in CI; baselines are in the spec. Compare by hand, and investigate
      regressions over ~20 %.
    - The first run found fan-out, fan-in and diamonds quadratic in `n`. Dependency tracking was
      reworked so a re-run that reads the same sources doesn't touch subscriber lists (3–6× faster
      at n = 1 000, node creation ~10 % slower).
    - Memo reads recurse: a chain overflows a 2 MiB stack at about 1 200 levels in debug and 4 000
      in release. The spec promises 500 in debug; an iterative update is deferred.
22. (Looked at and dropped: Liferay's clayui.com. That was the wrong Clay.)
23. **Scene shape** (2026-10-06, spec `docs/specs/scene/scene.md`, agreed):
    - A reusable `Scene` recorded through `scene.begin(size) -> SceneBuilder`; `finish()` ends
      the frame. A dropped builder leaves the Scene empty. Buffers are kept between frames, and
      recording does not allocate once warm (checked with a counting allocator).
    - Each `Entry` = command + `Option<ElementId>` + `z_index: i32`. Element id and z-index are
      sticky builder state, restored by `pop`.
    - Clip, transform and layer are scopes (`push_*`/`pop`). **Z-index orders siblings within a
      scope** (CSS stacking contexts); a nested scope moves as one unit. Popups that must escape
      a clip go through `Overlay` at the root, as in Flutter.
    - Unbalanced scopes don't panic: `finish` auto-closes and returns `Err(SceneError)` with
      counts; the Scene is always well-formed.
    - Phase 1 commands: `Fill`/`Stroke` of a `RoundedRect` (zero radii = plain rect),
      `BoxShadow` (Flutter's), `Image`, `GlyphRun` (glyphs in a side buffer), `Custom`
      (kind + bounds + bytes in a side buffer). A layer = group opacity + optional overlay color
      (source-atop). Gradients, paths, blend modes and inner shadows come later.
    - `is_culled(bounds)` is a conservative query (transformed bounding box vs. clip bounding
      boxes); recording never culls. Damage is `Full` or a list of rects.
    - No serde yet; `Scene::FORMAT_VERSION = 1`.
    - Two spec corrections while implementing: the `Entry` size budget is 96 bytes, not 64
      (`BoxShadow` alone is 64), and `Scene::glyphs` only promises an empty slice for runs that
      index past the buffer (a frame stamp would break Scene equality).
24. **Renderer and resources shape** (2026-10-06, spec `docs/specs/scene/renderer.md`, agreed):
    - `Resources` (one per app) holds `ImageData` (straight-alpha RGBA8 only) and `FontData`
      (file bytes + face index, not parsed), with `Arc` buffers. Handles come from **one
      process-wide atomic counter**, so they are unique across `Resources` and never reused.
      `revision()` lets renderers prune their caches. No internal locking: the app runner
      shares it (`Arc<RwLock<_>>`).
    - `Renderer` is object-safe: `resize(width, height, scale_factor)` and
      `render(&Scene, &Resources) -> Result<RenderReport, RenderError>`. Missing images, fonts,
      custom handlers and invalid values are counted in `RenderReport`, not errors; `Err` is
      only for target/backend failures.
    - Custom-command handlers are registered per backend crate (they need the backend's API).
    - No in-place image updates (add a new image, remove the old one); revisit for video and
      canvases.
    - **Shared counting** (amendment, agreed): `RenderReport::for_scene(scene, resources,
      handles_custom)` gives every backend the same counts. Non-finite transforms/clips count
      once and hide their scope; non-finite draws are invalid; then missing images, fonts and
      unhandled custom kinds. Backends add their own failures on top.
25. **Headless renderer** (2026-10-06, spec `docs/specs/render-headless/recorder.md`, agreed):
    records a `RecordedFrame` (Scene copy, size, scale factor, resources revision, report) per
    frame, keeps them until `take_frames`, `register_custom` marks kinds handled,
    `fail_next_render` injects one error. Rule ids `RENDER-HEADLESS-NN` (and `RENDER-SOFT-NN`).
    `#![forbid(unsafe_code)]` even though render crates are exempt.
26. **Software renderer** (2026-10-06, spec `docs/specs/render-soft/renderer.md`, open questions
    decided in autopilot):
    - `tiny-skia` 0.12 (std + simd, no png-format; BSD-3-Clause) and `png` 0.18 for lossless
      straight-alpha PNGs (tiny-skia's PNG path premultiplies and loses precision).
    - Everything is rasterized in device space; points are clamped to ±2^24 px because tiny-skia
      asserts on paths spanning ~±1e30.
    - Box-shadow blur: three box blurs per axis on a padded mask (margin 3 sigma, capped at the
      target size), sigma scaled by the transform's average scale.
    - Layers draw into pooled target-sized offscreens; overlay color is a SourceAtop fill.
    - No text in Phase 1: glyph runs with a present font count as `missing_fonts`.
    - Custom handlers get `tiny_skia` types (re-exported as `tantu_render_soft::tiny_skia`).
    - Goldens run in plain `cargo test`, tolerance 2, no differing pixels. A 1e-6 rad change
      moved 9 pixel-boundary edge pixels by 14, so cross-architecture CI may need a small
      differing-pixel allowance.
    - Depends on `tantu-core` directly (see open questions).
27. **code-to-docs skill** (2026-10-06): `.claude/skills/code-to-docs/` generates one
    self-contained interactive HTML page (architecture and dependency graph, crate panels, spec
    explorer with rule-to-test coverage, ADRs, roadmap, API search) from the repo, read-only.
    Python helpers outside the Rust workspace; `verify.py` checks the page against
    `cargo xtask spec-coverage` and `cargo metadata`. The page, `.claude/worktrees/` and
    `.playwright-mcp/` are gitignored.
28. **Platform shape** (2026-10-06, specs `docs/specs/platform/platform.md` and
    `docs/specs/platform-winit/shell.md`, open questions decided in autopilot):
    - `Platform::run(self, &mut dyn PlatformHandler)`; the handler gets `started`,
      `window_event` and `idle` with a `&mut dyn PlatformContext` (create/close windows, request
      redraw, size, scale factor, title, `surface_target`, exit). winit's model: the shell owns
      the loop.
    - Tantu's own event types, Flutter/W3C-shaped: logical pointer positions, physical sizes in
      `Resized`, `Key::Named`/`Character` (Space is `" "`), `Modifiers` as four bools, positive
      wheel `y` scrolls down. Mouse only; IME, touch, clipboard, cursors come later.
    - A new window always gets one `RedrawRequested`; read its size and scale factor from the
      context (winit doesn't send an initial `Resized` everywhere). `CloseRequested` doesn't
      close anything by itself.
    - `SurfaceTarget = Arc<dyn WindowHandles>` (raw-window-handle 0.6), so `tantu-render-wgpu`
      doesn't depend on the platform crates.
    - `FakePlatform` (in `tantu-platform`) runs a handler against a script: coalesced redraw
      rounds after each step, then `idle`, and a `FakeLog` of what the handler asked for.
    - winit 0.30.13 with default features. Its Wayland decorations pull in tiny-skia 0.11
      transitively; accepted (winit drawing its own title bars). Synthetic key events dropped.
    - Window tests need the main thread: `tests/window.rs` with `harness = false`, opt-in via
      `TANTU_WINDOW_TESTS=1`. Passed locally on Wayland and X11; not run in CI.
29. **wgpu renderer** (2026-10-06, spec `docs/specs/render-wgpu/renderer.md`, open questions
    decided in autopilot):
    - wgpu 30 (MSRV 1.87), `pollster`; `new_offscreen` (readback via `snapshot`) and
      `for_window` (any raw-window-handle 0.6 window, so no dependency on the platform crates;
      the platform's `SurfaceTarget` passes straight in). wgpu's env vars apply (`WGPU_BACKEND`).
    - Rules are the software renderer's, restated; the tests reuse its pixel probes. Passed on
      Vulkan (Intel Iris Xe), Vulkan (llvmpipe) and GL locally. Linux CI installs lavapipe and
      sets `TANTU_REQUIRE_GPU=1`; Windows/macOS run the tests only if an adapter is found.
    - Rendering: a frame is planned (ops + instance data) then encoded in order. Rounded-rect
      SDF quads for fills and strokes, Evan Wallace's closed-form blurred rounded rect for
      shadows, R8 clip masks (parent × clip), layer offscreens with source-atop overlay,
      `Rgba8Unorm`/non-sRGB surfaces so blending matches the software renderer.
    - Anti-aliasing width comes from the derivatives of the local position (the distance's own
      gradient jumps at corners and left fully covered corner pixels at 85 %).
    - Validation errors are caught (`on_uncaptured_error`) and returned from `render`.
    - Custom handlers get device, queue, encoder, target view, transform and a scissor-sized
      clip box.
30. **scene-window demo** (2026-10-06, spec `docs/specs/examples/scene-window.md`): the first
    runnable example; depends on the crates directly until the facade exists (agreed with the
    user as a Phase 1 exception to "examples depend only on `tantu`"). Workspace members now
    include `examples/*`. Its frame from wgpu and from the software renderer differ in 0.14 % of
    pixels by more than 8 levels.
31. **Renderer conformance** (2026-10-10, ADR 0008, spec
    `docs/specs/render-conformance/conformance.md`; the user said "your pick", aiming for an
    extensible framework that will later render in a browser and inside eframe):
    - New crate `tantu-render-conformance` (core, scene, png; no rasterizer): reference Scenes,
      embedded goldens, `match_images`. Backends take it as a dev-dependency. The software
      renderer produces the goldens (`TANTU_UPDATE_GOLDENS=1 cargo test -p tantu-render-soft
      --test goldens`) and is held to them strictly (tolerance 2).
    - `match_images` compares premultiplied pixels (straight alpha gave 255-level differences on
      near-transparent edges). Edge pixels = a reference change > 8 in the 3 × 3 neighbourhood.
      `CROSS_BACKEND`: interior ≤ 10, edge ≤ 96, at most 5 % of edge pixels > 16. Measured
      worst cases: interior 8 (llvmpipe), edge 58, 1.8 %.
    - The demo frame is compared live (soft vs. wgpu), no golden.
    - PLAN.md milestone reworded from "identically" to "the same … within the cross-backend
      tolerance".
    - Bug found and fixed: the wgpu shape shader picked corner radii by quadrant; now each
      corner owns the square of its radius.
    - Tantu does not use egui; an egui/eframe host would be a future `Renderer` backend checked
      by this crate.

32. **Layout tree** (2026-10-10, ADR 0009, specs `docs/specs/layout/constraints.md` and
    `tree.md`, agreed with the user):
    - `tantu-layout` owns a `LayoutTree` arena (`LayoutId`, 8 bytes) of nodes holding a
      `Box<dyn RenderBox>`, children, typed parent data (`Any`) and the last constraints, size
      and offset. `tantu-view` will keep it in step with elements and paint and hit-test from
      its geometry. Built-in layouts use Flutter's render-object names (`RenderPadding`, …).
    - `BoxConstraints` is Flutter's API; malformed values are defined, not asserted ("the
      minimum wins", NaN rules), never panic.
    - Instead of Flutter's property setters: `set(id, value)` replaces a layout object and marks
      it only when it differs (`PartialEq`); `get_mut` marks unconditionally. Layout objects
      hold only layout-affecting state (paint state lives in `tantu-view`).
    - Boundaries as in Flutter (root, size ignored, tight constraints, `sized_by_parent`);
      marking queues where it stops, and `layout()` runs the root then the queued boundaries in
      its subtree, shallowest first. Cached intrinsics make marking continue past a boundary.
    - A size outside the constraints is constrained, with a debug `tracing` warning. First
      direct `tracing` dependency (0.1, default features off).
    - `remove` drops a subtree; `set_children` only detaches. Foreign ids aren't detected (as
      CORE-ARENA-07).
    - `LayoutBuilder` moved to the `tantu-view` item (it builds views during layout).

33. **Layout objects** (2026-10-10, specs `single-child.md` (agreed with the user), `flex.md`,
    `stack.md`, `wrap.md` (open questions decided by the agent; review)):
    - Flutter's render objects and semantics; malformed properties get defined fallbacks
      (negative/NaN factors and spacings count as 0, a bad aspect ratio gives the smallest
      size, a NaN `Positioned` field counts as not set), never panics.
    - Value types with public fields deriving `PartialEq`, updated through `LayoutTree::set`;
      `RenderFlex` keeps its last overflow privately (`overflow()`), excluded from equality.
    - Flexible children in an unbounded main axis are laid out as inflexible (debug warning).
    - A stack with only positioned children takes the minimum on an unbounded axis.
    - `RenderWrap` intrinsic heights use Flutter's pre-dry-layout estimate; runs are found by
      one scan used to measure and to place, so layout allocates nothing.
    - `LayoutChildren::size` (LAYOUT-TREE-19) was added for flex and wrap (Flutter's
      `child.size`).
    - Directional variants (RTL start/end) are deferred to Phase 4.

34. **Text measurement and benchmarks** (2026-10-10): ADR 0010 (the user's three choices:
    paragraph-level `TextMeasure` cached in layout, the measurer passed into the layout pass,
    `RenderParagraph` in `tantu-layout`); spec `text.md` (two-generation cache, `max_lines` in
    the key, `TextStyleKey(u64)`, agreed); spec `benchmarks.md` (baselines recorded; "leaves count
    their own layouts" decided by the agent). **Autonomy:** the user, going away, chose "decide
    and mark for review" for architecture-level forks and asked for a push at each milestone.
    **Reactivity model for `tantu-view`:** components run once; reactive props are fine-grained
    effects; dynamic parts are explicit reactive views reconciled by key (the user's choice).

35. **View layer** (2026-10-10, ADR 0011; reactivity model by the user, the rest decided by the
    agent while the user was away; review): views are one-shot builders (`View::build(self,
    &mut BuildCx) -> ElementId`, `AnyView`); a `ViewTree` per window owns an `Rc<Runtime>`, the
    element arena and the `LayoutTree`; render elements own a layout node, region elements
    don't and are flattened into their render ancestor's layout children; every element has a
    reactive scope owned by its parent's; effects will queue updates applied per frame;
    element ids are Scene `ElementId`s; an implicit root element passes the window constraints
    to the app's view. Paint: a `Paint` trait per render element, a transform scope per render
    element (restores element id and z-index), children painted once (explicitly or after
    `paint`), conservative culling of leaves and clipping elements. Frames: `Prop<T>` /
    `IntoProp` (closures, `Signal`, `Memo`, a macro-listed set of value types, since a blanket
    value impl would conflict with the closure impl); `BuildCx::bind` applies the first value at
    once and later values at the next `frame`, coalesced per binding; `frame` applies, lays out
    and repaints the whole Scene; a frame requester tells the app runner to redraw. Dynamic
    content: `Dyn`/`Show`/`For` are regions; each `Dyn` version's content is owned by a scope
    under the region (not by the effect), so old content keeps working until the frame that
    removes it; `For` keeps elements (and their first item value) for kept keys.
    `LayoutBuilder`: records its constraints; its content is a `Dyn` over a constraints signal;
    `ViewTree::layout` rebuilds after the pass and lays out again (bits comparison, 16 rounds).
36. **Text system** (2026-10-10, spec `docs/specs/text/system.md`, decided by the agent while
    the user was away; review): parley 0.12 (needs Rust 1.88: MSRV and toolchain raised in
    `533335a`); `TextSystem::new()` (system fonts via fontique) or `without_system_fonts()`;
    `register_font`; a default family used as the fallback in a two-entry family stack;
    interned, sanitized `TextStyle`s as `TextStyleKey(index)`; shaped layouts cached by
    (text, style), re-broken per width; `paint` emits one Scene glyph run per parley run with
    fonts added to `Resources` once. **AGENTS.md change:** the `tantu-text` row may depend on
    `layout` (for `TextMeasure`, as ADR 0010 says). Tests use a committed Liberation Sans
    (OFL) so they don't depend on installed fonts.

## Commit log

| Commit | Summary |
|---|---|
| `e2f3e82` | Initial AGENTS.md, CLAUDE.md, PLAN.md |
| `c3e477d` | Plan → Spec → Unit tests → Implementation workflow |
| `08261d1` | README + dual MIT/Apache-2.0 license |
| `584bb1d` | Adopted Clay ideas (incl. sizing vocabulary) |
| `c02ae26` | Reverted to Flutter structure as the layout API; Clay for internals only |
| `34c12e3` | Added HANDOFF.md and the rule to keep it updated |
| `594513f` | Set copyright holder to Rajesh Pillai - Algorisys Technologies |
| `a2c71a4` | Architecture doc + diagram (`docs/architecture.md`, `docs/architecture/`), rule to keep them current (AGENTS.md, CLAUDE.md); renamed project dkui → Tantu across all docs, README tagline; moved to new public repo `algorisys-oss/tantu-rs` |
| `9703361` | Removed references to the old repo from HANDOFF.md |
| `f3b4adb` | Phase 0: Cargo workspace skeleton (16 crates), `rust-toolchain.toml` (1.85), shared lints, CI workflow |
| `6d7c732` | Merged the workspace skeleton into `main`; no-PR workflow noted |
| `6084f23` | CI green on Linux/Windows/macOS; ticked the workspace/CI item in PLAN.md |
| `e21e4b3` | Spec template and README (`docs/specs/`), ADRs 0001–0006 (`docs/adr/`), ADR links in `docs/architecture.md` |
| `c3aa616` | docs: one-commit-per-step rule (AGENTS.md, CLAUDE.md, specs README) |
| `0678f59` | spec: spec-coverage check (XTASK-COV-01..18), PLAN.md item |
| `ba79621` | test: spec-coverage check, 20 failing tests |
| `8f51517` | impl: `xtask` spec-coverage check, CI job, AGENTS.md row/commands |
| `19cd9ed` | spec: tantu-core geometry (CORE-GEOM-01..29); PLAN.md tantu-core item split into geometry, color, id + arena |
| `bdeb293` | spec: realistic accuracy promise for `Affine::inverse` (CORE-GEOM-28) |
| `3e43794` | test: tantu-core geometry, stubs + 31 tests (30 failing on `todo!()`) |
| `67c8680` | impl: tantu-core geometry; spec Implemented, PLAN.md ticked |
| `7c8006e` | spec: tantu-core `Color` (CORE-COLOR-01..15) |
| `5ca4f17` | test: tantu-core `Color`, stubs + 16 tests (15 failing on `todo!()`) |
| `7ffce33` | spec: exact 0/1 endpoints for `Color::from_linear` (CORE-COLOR-14) |
| `712a1cb` | impl: tantu-core `Color`; spec Implemented, PLAN.md ticked |
| `eade771` | impl: named, documented constants for the sRGB transfer function and 8-bit scale in `color.rs` (no behavior change) |
| `2d8ebb2` | spec: tantu-core `Id` and generational arena (CORE-ARENA-01..17); `tantu-core` unsafe exception dropped from AGENTS.md |
| `1b92306` | test: tantu-core arena, stubs + 20 tests + doctest (19 tests and the doctest failing on `todo!()`) |
| `ce0ee6e` | impl: tantu-core `Id` and `Arena`; spec Implemented, PLAN.md `tantu-core` ticked |
| `03fff8e` | spec: tantu-reactive (REACTIVE-SIG-01..23); ADR 0007 on disposed handles |
| `f58406a` | test: tantu-reactive, stubs + 43 tests (42 failing on `todo!()`) + doctest |
| `91ecb7c` | impl: tantu-reactive runtime, signals, memos, effects, scopes; spec Implemented, PLAN.md ticked |
| `c72e6ac` | spec: reactive micro-benchmarks (REACTIVE-BENCH-01..08) |
| `fac7685` | test: benchmark scenarios, `todo!()` stubs + 8 failing tests |
| `cc475c1` | spec: deep-chain floor lowered to 500 levels in debug (1 000 overflowed the test stack) |
| `b68482b` | test: deep-chain test at 500 levels |
| `13565b5` | test: reordered/repeated dependency reads and subscriber-list growth (pass on the old tracking) |
| `1c2f2f7` | impl: dependency tracking without re-subscribing on every re-run (fixes quadratic fan-out/fan-in) |
| `c1e019a` | impl: benchmark scenarios + criterion harness, baselines in the spec; Phase 0 ticked in PLAN.md |
| `9aeba65` | docs: HANDOFF.md refreshed for the next session |
| `6fd29d1` | spec: tantu-scene Scene and builder (SCENE-SCENE-01..24); PLAN.md tantu-scene item split in two |
| `f01c8a3` | spec: `Entry` size budget of 96 bytes |
| `5fa2047` | test: tantu-scene Scene and builder, stubs + 24 tests (23 failing on `todo!()`) + alloc test + doctest |
| `c6a6f7e` | spec: `Scene::glyphs`/`custom_data` doc matches SCENE-SCENE-11 |
| `8d204be` | impl: tantu-scene Scene and builder; spec Implemented |
| `eca7815` | spec: tantu-scene Renderer trait and resources (SCENE-RES-01..10, SCENE-RENDER-01..03) |
| `ab6e0a0` | test: Renderer trait and resources, stubs + 13 failing tests |
| `24f49df` | impl: Renderer trait and resources; spec Implemented, PLAN.md tantu-scene ticked |
| `6d6e608` | docs: HANDOFF.md for the end of the tantu-scene item |
| `ae77083` | spec: shared `RenderReport` counting (SCENE-RENDER-04..08); PLAN.md render item split in three |
| `4202f9e` | test: shared counting, 5 failing tests |
| `686f7b5` | impl: `RenderReport::for_scene` |
| `da5b5a5` | spec: tantu-render-headless (RENDER-HEADLESS-01..10) |
| `1fdb904` | test: headless renderer, 10 failing tests + doctest |
| `b6e4c08` | test: compare a NaN-holding Scene by Debug output |
| `10bdd22` | impl: tantu-render-headless |
| `52782bf` | spec: tantu-render-soft (RENDER-SOFT-01..26), tiny-skia + png |
| `191e5c9` | test: render-soft, 31 failing tests + doctest |
| `123b868` | spec: device-space coordinate clamping (RENDER-SOFT-24) |
| `33024eb` | impl: tantu-render-soft, five goldens; PLAN.md render item ticked |
| `6e65b53` | chore: code-to-docs skill, .gitignore entries |
| `973ad62` | docs: HANDOFF.md for the end of the render item |
| `181c490` | spec: tantu-platform trait and winit shell; PLAN.md platform item split in two |
| `2ce952b` | test: tantu-platform, 16 failing tests + doctest |
| `f049a73` | impl: tantu-platform types and `FakePlatform` |
| `f025907` | test: tantu-platform-winit, 8 failing conversion tests + opt-in window test |
| `754fd93` | impl: tantu-platform-winit shell; PLAN.md platform item ticked |
| `89f6cba` | docs: HANDOFF.md for the end of the platform item |
| `b5d8997` | chore: MSRV and toolchain 1.87 (for wgpu 30) |
| `adee517` | spec: tantu-render-wgpu (RENDER-WGPU-01..17); PLAN.md demo item |
| `76ddc5c` | test: wgpu renderer, 17 failing tests |
| `af293d6` | spec: wgpu anti-aliasing width and backend selection notes |
| `26c2176` | impl: tantu-render-wgpu |
| `09ad152` | chore: lavapipe and `TANTU_REQUIRE_GPU=1` in Linux CI |
| `d774f96` | spec: scene-window demo (SCENE-WINDOW-01..03) |
| `caa9a2c` | test: scene-window, 3 failing tests |
| `18b09b0` | impl: scene-window demo |
| `0383fe6` | docs: HANDOFF.md for the wgpu renderer and the demo |
| `cbd2141` | docs: HANDOFF.md, paused for the day (CI green, wgpu tests ran on all runners) |
| `00b228b` | chore: `rust-analyzer` in the pinned toolchain's components; HANDOFF.md |
| `776dfd7` | spec: renderer conformance crate and the Phase 1 milestone (RENDER-CONF-01..13); ADR 0008, AGENTS.md row, architecture doc + diagram |
| `49c885b` | spec: RENDER-CONF-01 without 0 × 0 images, RENDER-CONF-09 with fresh image handles |
| `e0e0e9b` | test: renderer conformance, 13 failing tests; goldens moved into the new crate |
| `46dec03` | test: wgpu corner radius over half a side (RENDER-WGPU-08), failing |
| `758c3b3` | fix: wgpu rounded corners with a radius over half a side |
| `2f4bb8d` | impl: renderer conformance crate; Phase 1 milestone met, PLAN.md ticked |
| `dc6302f` | docs: HANDOFF.md for the end of Phase 1 |
| `d73c3ec` | spec: BoxConstraints (LAYOUT-CONS-01..17); ADR 0009, PLAN.md tantu-layout split |
| `c0ef913` | test: BoxConstraints, 17 failing tests + doctest |
| `6085d2d` | impl: BoxConstraints |
| `fbdeebb` | spec: layout tree and protocol (LAYOUT-TREE-01..18) |
| `8d70132` | spec: LAYOUT-TREE-02 follows CORE-ARENA-07 for foreign ids |
| `340b794` | test: layout tree, 19 failing tests |
| `0d499cb` | impl: layout tree and protocol; `tracing` dependency |
| `53854ea` | docs: HANDOFF.md for the layout tree |
| `93d801b` | spec: single-child layouts (LAYOUT-SINGLE-01..15) |
| `986552c` | test: single-child layouts, 15 failing tests |
| `ce5265e` | impl: single-child layouts |
| `5b2922d` | spec: flex layout (LAYOUT-FLEX-01..14) |
| `258497f` | test: flex layout, 14 failing tests |
| `7bb5430` | spec: `LayoutChildren::size` (LAYOUT-TREE-19) |
| `3ddccd9` | test: `LayoutChildren::size`, 1 failing test |
| `28df04f` | impl: `LayoutChildren::size` |
| `0dd3cbf` | test: LAYOUT-FLEX-11 expectation follows LAYOUT-FLEX-06 |
| `8f3b15f` | impl: flex layout; Row-with-Expanded doctest |
| `e6d1616` | spec: stack layout (LAYOUT-STACK-01..08) |
| `dd91212` | test: stack layout, 8 failing tests |
| `512954e` | impl: stack layout |
| `b3d7e28` | spec: wrap layout (LAYOUT-WRAP-01..09) |
| `e5a528d` | test: wrap layout, 9 failing tests |
| `7cdbc63` | impl: wrap layout; PLAN.md `tantu-layout` item ticked |
| `7d2a9c0` | docs: HANDOFF.md for the end of the `tantu-layout` item |
| `201c12b` | spec: TextMeasure, MeasureCache and RenderParagraph (LAYOUT-TEXT-01..10); ADR 0010 |
| `ebde600` | test: text measurement, 10 failing tests |
| `818b633` | impl: TextMeasure, MeasureCache and RenderParagraph |
| `0f6eb86` | spec: layout benchmarks (LAYOUT-BENCH-01..05) |
| `37e8e3a` | test: layout benchmarks, 5 failing tests |
| `94839cb` | impl: layout benchmarks; 10k-node full layout in 635 µs |
| `2a9c3a4` | docs: AGENTS.md (every crate may depend on tantu-core); HANDOFF.md |
| `64cd972` | spec: view tree (VIEW-TREE-01..11); ADR 0011, PLAN.md tantu-view split |
| `63e8e09` | test: view tree, 11 failing tests |
| `db138c8` | impl: view tree |
| `314cb27` | docs: HANDOFF.md for the view tree |
| `45f8950` | spec: view paint (VIEW-PAINT-01..06) |
| `e13d9f7` | test: view paint, 6 failing tests |
| `76666ea` | test: VIEW-PAINT-02 offsets follow the column's centered cross alignment |
| `109e2ed` | impl: view paint |
| `b9cf9ef` | docs: HANDOFF.md for view paint |
| `829fbc2` | spec: reactive props and frames (VIEW-FRAME-01..08) |
| `df47794` | test: reactive props and frames, 8 failing tests |
| `27d89ed` | spec: `Paint` is `Any` (for `ElementMut` paint downcasts) |
| `dcc0ef5` | impl: reactive props and frames |
| `d23eade` | docs: HANDOFF.md for reactive props and frames |
| `d19a126` | spec: dynamic content (VIEW-DYN-01..07) |
| `6d4f357` | test: dynamic content, 7 failing tests |
| `c732122` | impl: dynamic content |
| `0c06a4b` | docs: HANDOFF.md for dynamic content |
| `57f183e` | spec: LayoutBuilder (VIEW-LB-01..06) |
| `f818a3c` | test: LayoutBuilder, 7 failing tests |
| `fcd06c4` | test: VIEW-LB-05 round limit with 20 nested builders |
| `291923e` | impl: LayoutBuilder; PLAN.md `tantu-view` item ticked |
| `c7a43dd` | docs: HANDOFF.md for the end of the `tantu-view` item |
| `533335a` | chore: MSRV and toolchain 1.88 (for parley) |
| `5e54abb` | spec: text system (TEXT-SYS-01..09); AGENTS.md `tantu-text` may use `layout` |
| `b35330c` | test: text system, 9 failing tests; Liberation Sans test font |
| `dd60c6d` | impl: text system over parley |
| _this commit_ | docs: HANDOFF.md for the text system |

A commit can't contain its own hash, so the newest row says _this commit_ (or _uncommitted_ for work not yet committed). The next update replaces
that with the real hash from `git log`.

## Next steps (Phase 2 in PLAN.md)

1. (`tantu-view` done.)
   content (`dynamic.md`), `LayoutBuilder` (`layout-builder.md`).
3. Text in views (`docs/specs/view/text.md`), glyph rasterization (render-soft with swash,
   render-wgpu with a glyph atlas, text reference Scenes), event dispatch, the `tantu` facade
   with the counter example, `WidgetTester`.

## Open questions

- **Autopilot decisions to review:** decisions 26 (render-soft), 28 (platform) and 29 (wgpu), as
  well as 20/21 and ADR 0007.
- **Draw-command validity is checked in three places** (`RenderReport::for_scene` and private
  copies in the soft and wgpu renderers). A public helper in `tantu-scene` (e.g.
  `Command::is_drawable`) would remove the copies; it is an API addition, so it needs a spec
  change.
- **CLAUDE.md mention of the skill** (proposed, not applied): "`/code-to-docs`
  (`.claude/skills/code-to-docs/`) generates the interactive docs page
  `docs/interactive/index.html` from the repo (read-only; it reports spec-coverage and
  dependency-rule problems, it doesn't fix them)."

- ADR 0007 (reads of disposed handles panic) and the reactive shape (decision 20) were decided in
  autopilot. Confirm or change them before `tantu-view` depends on them.
- Glyph rendering in the soft renderer: tiny-skia has no text. Rasterizing glyphs needs a font
  rasterizer (swash, per ADR 0005, or similar) in `tantu-render-soft`, or glyph runs are left
  for Phase 2 and counted as `missing_fonts` until then. Decide in the render-soft spec.
- **Shadows with a radius over half a side** still use the quadrant approximation in the wgpu
  blur shader (Evan Wallace's closed form supports one radius per quadrant). Not covered by a
  reference Scene yet; check when shadows get a closer look.
- **Review decision 36** (text system, the AGENTS.md `tantu-text` → `layout` edge, MSRV 1.88).
- **Review ADR 0011 and the view-tree spec** (decided by the agent while the user was away,
  decision 35).
- **Review the agent's decisions in the flex, stack and wrap specs** (decision 33).
- **Review ADR 0008** and the new AGENTS.md row for `tantu-render-conformance` (made under
  "your pick").

- Is `tantu` (and `tantu-*`) available on crates.io? Check, and consider reserving it, before the
  first publish. Same for a domain / GitHub org name if wanted.
