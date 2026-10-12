# HANDOFF.md

Where the project stands, so the next session (human or agent) can pick up without re-reading the
whole history. Update this file in every commit (see `AGENTS.md` → General rules).

_Last updated: 2026-10-12 (overlay inheritance failing regressions committed; implementation next)_

## Resume here (2026-10-12)

DataGrid parts 1 to 7 are complete. The user accepted the stress-example proposals with
"continue", requested a kitchen-sink update, and authorized commit/push when done.
The checked implementation is committed on `phase3/focus` and fast-forwarded into `main`
for the authorized push, following decision 14. Commit `9d78f30` is pushed; its CI run
was queued when checked. The user explicitly requires performance stress tests to stay local;
GitHub runs only deterministic bounded correctness tests and normal build/lint checks.

`cargo run --release -p data-grid-stress` opens the 1M × 20 text-cell workload with Start/Stop,
beginning/middle/end jumps, optional two frozen columns and bounded redraw-tick cadence stats.
`cargo run -p gallery` → Kitchen sink shows the updated grid: resize/reorder, range selection,
editable Name/Score, validation feedback and the stress command. Native GPU/compositor
60 fps on recorded mid-range hardware remains **unverified**; no native desktop display is
exported to this command environment. Software goldens were rendered and visually inspected.
Current feature: overlay inheritance (VIEW-OVERLAY-08..10, ADR 0014), accepted on the user's
"ok, then continue". Add failing generic-value/signal/nested-popup/lifecycle tests, a tooltip
target-theme regression, and the GALLERY-19 local-theme popup; then implement and validate.
Feature branch: `phase4/overlay-inheritance`. CI for `3c7da2f` on main is green.
When resuming, tell the agent: "Read HANDOFF.md and continue."

**Where things stand:**

- **Phases 0, 1 and 2 are done** (Phase 2 on 2026-10-10, checked on Linux by the agent and on
  Windows and macOS by the user).
- **Phase 3 is done.** Included: focus, keyboard navigation and shortcuts; the overlay and
  anchored positioning; the whole widget item (`Text`, `Button`, `Tooltip`, `Divider`, `Slider`,
  `Checkbox`/`Radio`/`Switch`, `Icon`/`IconButton`, `Image`, `RichText`, `TextField` with undo
  and IME); `ScrollView` with wheel, scrollbar and keyboard scrolling; fixed-extent
  `ListView::builder`; inherited values; `examples/gallery`; on 2026-10-11 **the theme**
  (decision 62), **timers and tickers** (decision 63: the reactive runtime has a clock, the
  platform wakes the loop up at deadlines, the tester has `advance`, the tooltip waits 500 ms)
  **the animation controller, tweens, curves and `animated()` props** (decision 64; the
  gallery has an Animation section) and **transitions** (decision 65: `Opacity`,
  `AnimatedSwitcher`, the caret blink, double-click word selection, smooth wheel scrolling
  and `ScrollController::animate_to`), and on 2026-10-12 **the `ListView` leftovers**
  (decision 66: per-item extents, separators, selection and keys) and **accessibility**
  (decisions 67 and 68, ADR 0013: `Semantics` on elements, the AccessKit tree built by
  `tantu-a11y`, actions routed back as synthesized input, every widget annotated, the
  platform events and `update_accessibility`, the `accesskit_winit` adapter per window, the
  runner sending a tree per frame while a screen reader is attached), and **focus scopes
  and scrolling into view** (decision 69: `ensure_visible` through every enclosing
  `ScrollView`, Tab reveals the focus, `FocusScope` confines Tab and remembers the focus).
  **Phase 3 is done** (2026-10-12), except that the gallery is **not checked with a real
  screen reader** (see open questions).
- **The end goal** (the user, 2026-10-11): a general-purpose GUI library for building desktop
  apps, from small utilities to enterprise applications. README, AGENTS.md and the
  architecture doc now say so.
- Workspace: 77 specs, every rule covered (`cargo xtask spec-coverage`); `fmt`, `clippy -D
  warnings`, `cargo test --workspace`, `cargo test -p tantu --no-default-features` and the touched examples' rustdoc
  (`-D warnings`) are green locally (checked 2026-10-12 before this push). **CI:** inspect the
  new run on `main` after this push; the prior macOS Control/Command chord issue is fixed.

**Working pattern:** the agent writes each spec as a Draft with open questions and proposals and
stops; the user says "continue" to accept the proposals; the agent then does
`spec:`/`test:`/`impl:` commits, checks with `fmt`, `clippy -D warnings`, `test` and
`spec-coverage`, and pushes with this file updated. When the user is away, the agent takes the
recommended option at forks, marks it for review and pushes at each milestone (decision 34).
Additions made while agreeing a spec are marked "review" in its Open questions.

**For the user to review:** decisions 49 to 68 (made by the agent, marked in the specs), and the
open questions below. **Please run the gallery under a screen reader** (Orca on Linux,
Narrator on Windows, VoiceOver on macOS) and report what is read and what isn't.

**Phase 4 has started.** **`DataGrid` part 1 is done** (decision 70, 2026-10-12): the data
source, columns, pinned header, two-dimensional virtualization and scrolling, roles, the
gallery's Data grid section and the `GridScroll` benchmark (1.30 ms per one-row wheel step on a
1 000 000 × 20 grid at 1280 × 720, target under 8 ms). PLAN.md splits the item in seven parts.

**`DataGrid` part 2, frozen columns, is done** too (decision 71, 2026-10-12, on the user's
"continue and then commit and push"; its three proposals are marked for review).

**Next (PLAN.md order): Phase 4, enterprise features.**

1. **`DataGrid` part 3, sorting and filtering, is done** (decision 72, 2026-10-12; the user
   said "continue" on the draft's seven proposals).
2. **The kitchen sink is done** (decision 73, 2026-10-12; the user asked for "a kitchen sink
   like demo to show case all component examples" and said "continue" on the draft's six
   proposals): the gallery's last section, "Kitchen sink", is one scrolling page of twelve
   cards with every exported widget; `cargo run -p gallery` shows it.
3. **`DataGrid` part 4, column resize and reorder, is done** (decision 74, 2026-10-12; the
   user said "continue" on the draft's seven proposals): widths and display order as shared
   signals, a 6 px resize handle, an opt-in reorder drag with a drop line, sorting on the
   click. The gallery's Data grid section echoes "Name width: …" and "Order: …".
4. **`DataGrid` part 5 is done** (decision 75, 2026-10-12, spec
   `docs/specs/widgets/data-grid-selection.md`). The user said "YOu have all the permissions"
   after the draft; the agent took its recommended options and continued autonomously.
   Opt-in None/Row/Cell/Range selection, an app-visible controller with source-cell mapping,
   Shift ranges, a single Tab stop, F6 header navigation, keyboard sort/resize/reorder,
   immediate active-cell reveal, accessibility dimensions/indices/values/sort direction and
   validated active descendants. Pointer modifiers are forwarded per window; the tester has
   `tap_with_modifiers`. The gallery offers all modes and has row/range selection goldens.
   Selection resets on sort/filter/revision/actual reorder; widths and scrolling preserve it.
   **Validation:** workspace tests, strict workspace Clippy, formatting, spec coverage
   (75 checked, no draft/problems), no-default-feature facade tests and rustdoc with
   warnings denied pass. Standalone GridScroll median **0.793 ms** (part 4: 0.797 ms);
   the initial concurrent validation measurement was noisy. No dependency/pipeline change.
5. **`DataGrid` part 6 is done** (decision 76, 2026-10-12, spec
   `docs/specs/widgets/data-grid-editing.md`). The user said "continue" on the eight
   proposals. Columns opt into editing; double-click/F2 opens a focused single-line
   TextField, Enter commits typed values, Escape/blur/Tab cancel. Default sources remain
   read-only. Rejected writes preserve the draft and report source coordinates and errors.
   Sort/filter/order/revision changes and complete viewport exit cancel safely. A successful
   write refreshes this grid without an app revision bump, recomputing active row maps.
   The million-row gallery uses sparse overrides, rejects Score outside 0..=100, and has
   an editor golden. Integration amendment (review): `BuildCx::focus` (VIEW-FOCUS-10)
   permits entry/restoration after building without exposing mutable tree access.
   **Validation:** 16 editor tests, the focus amendment, gallery behaviors/goldens, all
   workspace tests, strict workspace Clippy, formatting, spec coverage (76, no problems),
   no-default-feature facade tests and rustdoc with warnings denied pass. Standalone
   GridScroll estimate **0.804 ms** (0.795–0.814 ms), no measured performance regression.
   Existing WIDGETS-GRID-06 behavior is preserved: a custom builder refreshes on revision
   even if its CellValue is unchanged. No architecture edges or pipeline stages changed.
6. **`DataGrid` part 7 is done** (decision 77, 2026-10-12, spec
   `docs/specs/examples/data-grid-stress.md`). A standalone 1M × 20 generated text-cell app
   scrolls one row per scoped ticker callback, with Start/Stop, three jumps and optional
   two frozen columns. Cadence collection uses a fixed 600-interval ring, two-second warmup,
   nearest-rank p95, exact 60 Hz budget comparison and status refreshes at most twice per
   second. Seven tests cover generation, viewport dimensions, lifecycle, jumps, freezing,
   manual two-axis scrolling, statistics, bounded cell reads/live cells and a software golden.
   The kitchen-sink GALLERY-18 amendment (review) adds resize/reorder, range selection,
   sparse Name/Score edits, Score validation, echoes and the stress command; two new behavior
   tests and a grid golden pass, and the top/end scrollbar goldens are refreshed.
   **Validation:** workspace tests, strict workspace Clippy, formatting, coverage (77, no
   drafts/problems), no-default-feature facade tests and touched-example rustdoc pass.
   Latest CPU GridScroll estimate **1.593 ms** (1.540–1.651 ms), below the 8 ms target;
   this is slower than the prior 0.804 ms host measurement despite unchanged framework and
   benchmark code. It is not evidence of native presentation. No new library API/edge;
   architecture docs/diagram now identify the new facade-only example.
   **Integration detail:** Expanded cannot attach flex data to a Dyn region; a concrete
   SizedBox::expand holds the grid region's flex so the viewport stays bounded. Initial
   testing exposed the unbounded configuration; the dimensions/bounded-work tests now pass.
2. Before menus and dialogs: the overlay-inheritance amendment (open question below), since
   both will need the theme inside overlays; `FocusScope` is ready for them.

## Current state

- **Name:** **Tantu** (pronounced "tan-too", Sanskrit for "thread"). Tagline: *Compose once. Render
  your way.* Crates: `tantu`, `tantu-core`, `tantu-reactive`, `tantu-layout`, `tantu-widgets`,
  `tantu-render-wgpu`, … (full list in AGENTS.md). App import: `use tantu::prelude::*;`
- **Phase:** Phases 0 to 3 are done (Phase 2 on 2026-10-10, Phase 3 on 2026-10-12); Phase 4
  (enterprise features) is next, see "Resume here". Earlier detail: Phase 2 (Layout, views
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
  `tantu-theme` is implemented (decision 62) and so is `tantu-a11y` (decision 67).
- **Repo:** https://github.com/algorisys-oss/tantu-rs (public). Branch: `main` for the authorized push; feature work was on `phase3/focus`.
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
  - `rust-toolchain.toml`: pins Rust 1.88, which is also the MSRV, with `rustfmt` and `clippy`
    (rust-analyzer was removed on 2026-10-12 at the user's request: it slowed the machine, and
    the agent works from cargo; `rust-analyzer.toml` stays for an editor's own server); `clippy.toml`
  - `crates/`: one directory per crate; `crates/tantu` is the facade
  - `docs/adr/`: ADRs 0001–0013 plus `README.md` (index, template, how to supersede)
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
    (OFL) so they don't depend on installed fonts. fontique's `fontconfig-dlopen` feature loads
    fontconfig at run time on Linux, so building needs no fontconfig development package.
37. **Text in views** (2026-10-10, spec `docs/specs/view/text.md`, decided by the agent while the
    user was away; review): `TextPainter` and `TextContext` (= `TextMeasure + TextPainter`) live
    in `tantu-view`; `NoTextMeasure` implements `TextPainter`; `PaintCx::paint_text` exists
    because a `Paint` can't borrow `text()` and `scene()` at once; paint wraps at the layout
    constraints; `ParagraphPaint` reads the `RenderParagraph` instead of duplicating it.
38. **Event dispatch** (2026-10-10, spec `docs/specs/view/events.md`; the draft's proposals,
    taken when the user said "continue"): `tantu-view` has its own `PointerEvent` (the runner
    converts platform events); handlers per element and phase (capture root to target, then
    bubble), `Handled::Stop` ends dispatch; hit-testing by layout bounds only, regions
    transparent, a subtree searched only inside its parent's bounds; a handled `Down` captures
    its path until the matching `Up`; hover changes only on `Move`/`Leave` and `Enter`/`Leave`
    go to that element alone (bubble phase); a window `Leave` only ends hover. Handlers can't
    touch the tree, only signals, so removal during dispatch is safe by construction.
39. **Facade groundwork** (2026-10-10; the drafts' proposals, taken when the user said
    "continue"; additions made while agreeing are marked for review):
    - Shared text styles: `TextStyles` (an `Rc<RefCell<Vec<TextStyle>>>` handle) owned by the
      `TextSystem` and cloned into each `ViewTree` (`with_text_styles`); `BuildCx::text_style`;
      `TextStyle::body/title/label` with Material 3 sizes. `TextSystem::text_style` returns a copy.
    - Layout widgets in `tantu-widgets` over the layout render objects; every prop reactive via
      `bind` + `update_render`; parent data (`flex`, `Positioned`) plain values; a region child
      of `Expanded`/`Positioned` warns. **Review:** `tantu-view` re-exports `core`, `layout`,
      `reactive`, `scene` and `text` so `tantu-widgets` keeps its single AGENTS.md edge (its
      private `text` module became `text_context`); `BoxConstraints` is an `IntoProp` value.
    - `Text` and `Button`: Material 3 filled-button metrics and colors until `tantu-theme`;
      `on_press` on a primary release inside (`PointerCx::size`, VIEW-EVENT-07); a centered
      `Align` (factors 1) keeps the label centered. New widgets lack the goldens and gallery
      entries AGENTS.md asks for: the gallery is a Phase 3 item; add both then.
40. **Facade and counter** (2026-10-10; proposals taken when the user said "continue",
    additions while agreeing marked for review): `App` (windows, fonts, `without_system_fonts`),
    `run` (winit + wgpu; `Unsupported` without those features), `run_with`, and **`App::handler`
    → `AppHandler`** (a `PlatformHandler`, so tests use `FakePlatform::run_logged` and other
    shells can drive it; review). One `TextSystem` + `Resources` per app; redraws by polling
    `needs_frame()` after events and at idle; 40 px per wheel line; Back/Forward as
    `Other(3)`/`Other(4)`; `TargetLost` retried. **FACADE-APP-09** (review): the first
    registered font's family becomes the default family (generic families find no registered
    font). CI also runs `cargo test -p tantu --no-default-features`. `examples/counter`:
    `main.rs` is the AGENTS.md snippet, compared ignoring whitespace (rustfmt splits its last
    chain); tests `include!` it in a module; the example lists the workspace lints minus
    `missing_docs` (the snippet has no crate docs); `tantu-render-headless` as a dev-dependency.
41. **WidgetTester** (2026-10-10, the draft's proposals): `Keyed` wrapper view and
    `ViewTree::find_key` (no `.key()` on widgets yet); a third copy of Liberation Sans in
    `tantu-test`; finders and asserts panic with the finder's description; a region's `rect` is
    the union of its children; goldens at scale 1, tolerance 2, written with
    `TANTU_UPDATE_GOLDENS=1` (the update mode is tested in its own test binary because it sets
    an environment variable).
42. **Layout demo and window backgrounds** (2026-10-10): the demo uses a local `Swatch` (a box
    with a custom `Paint`) until decorated boxes come with the theme; its tests use
    `WidgetTester` and a golden. FACADE-APP-10: the runner wraps each window's content in a
    background element (fills with `Window::background`, default `Color::WHITE`), so the color
    is part of the Scene (renderer-independent, visible in goldens) rather than a renderer clear
    color.
43. **Bidi and font fallback** (2026-10-10, spec `docs/specs/text/scripts.md`, the draft's
    proposals): bidi needed only rules (parley handles it). Fallback order: the style's family,
    the default family, the other registered families in registration order, then system
    fallback. Noto Sans Hebrew (OFL 1.1) is a second test font. CJK and Thai segmentation
    (parley's `complex-scripts`) is deferred to Phase 4 i18n. A debug build printed an ICU4X
    "No segmentation model" message to stderr for Japanese; find where it comes from then.
44. **Keyboard events and focus** (2026-10-10, spec `docs/specs/view/focus.md`, the draft's
    proposals): own `LogicalKey`/`NamedKey`/`Modifiers`/`KeyEvent` in `tantu-view` (`Key` is the
    element key from `Keyed`); traversal in tree order; buttons don't take focus on press; one
    focus per window, no scopes; with nothing focused, keys go to the root's handlers; a key
    event counts as handled when any handler ran (as for pointers), and Tab moves focus unless a
    handler returned `Stop`.
45. **Shortcuts and commands** (2026-10-10, spec `docs/specs/view/shortcuts.md`, the draft's
    proposals): Flutter's three parts; exact modifiers, characters without case; actions always
    enabled until menus; bindings are tried per element in the bubble phase before that
    element's key handlers, so capture handlers can intercept first. An element holds one
    `Keyed` key; an outer `Keyed` on the same element replaces an inner one.
46. **Button keyboard** (2026-10-10, spec `docs/specs/widgets/button-keyboard.md`, the draft's
    proposals): Material 3 focus outline (3 px, 2 px outside) and 10 % focus layer; activation
    on press (not release), repeats ignored; indicator whenever focused. **Review:**
    VIEW-FOCUS-08 `ElementMut::set_focusable`, added so a disabled button leaves traversal.
47. **Overlay** (2026-10-11, spec `docs/specs/view/overlay.md`, the draft's proposals):
    declarative `OverlayPortal`; `Alignment` for both anchor points; flip vertically, then
    horizontally, then clamp; one overlay layer per window, created lazily so trees without
    overlays are unchanged; each overlay sits in a wrapper render element (so regions work as
    content); placement runs after layout, with one extra pass when it changed.
48. **Tooltip** (2026-10-11, spec `docs/specs/widgets/tooltip.md`, the draft's proposals): no
    show delay until timers exist (animation item); shows on hover and on focus within the
    child; Material 3 plain-tooltip look; passthrough overlay 4 px below (or above). **Review:**
    VIEW-FOCUS-09 `BuildCx::on_focus_within`, added for it.
49. **Divider** (2026-10-11, spec `docs/specs/widgets/divider.md`, the draft's proposals):
    Flutter's `LimitedBox` added to `tantu-layout` and the layout widgets rather than a private
    leaf render object, so "fill a bounded width, else a fixed one" is one Flutter idiom (a
    `LimitedBox` around an expanding box; the Slider reuses it); default thickness 1 px
    (Material 3), not Flutter's hairline, until paint can see the scale factor; one golden per
    widget from here on under `crates/tantu-widgets/tests/goldens/` (`Button` and `Tooltip`
    get theirs with the gallery).
50. **Slider** (2026-10-11, spec `docs/specs/widgets/slider.md`, the draft's proposals): `f64`
    values; strictly controlled like Flutter (the thumb shows `value`, `on_change` reports);
    Flutter's keyboard model (5 % or one-division steps, Home/End, repeats, `on_change` only);
    focus on press; a slider without `on_change` counts as disabled; a fixed 48 px height
    where Flutter takes the bounded maximum height; `WidgetTester::drag` (TEST-WT-08).
51. **Checkbox, Radio, Switch** (2026-10-11, spec `docs/specs/widgets/toggles.md`, decided by
    the agent while the user was away; review): the check mark is two rotated bars in Scene
    transform scopes (no Scene path command yet, no icon font yet); app-driven
    `indeterminate` instead of Flutter's user-cycled `tristate`, activation always reports
    `!value`; desktop tap targets (40 × 40, switch 60 × 40); direct Space/Enter handlers as
    `Button` (shortcuts fire on repeats and intents carry no repeat flag, so the shared
    `ActivateIntent` stays deferred); `Radio::new(value, group)` as Flutter's `groupValue`;
    no focus on press, as `Button`. Material 3 colors and metrics are in the spec's table.
52. **Icon and IconButton** (2026-10-11, spec `docs/specs/widgets/icon.md`, decided by the
    agent while the user was away; review): Material Icons Regular embedded in
    `tantu-widgets` (feature `material-icons`, default on, forwarded by the facade) with a
    generated constant table (names upper snake case, digit-leading names prefixed `N`, the
    first of duplicate names wins; regenerate by hand from the codepoints file until an
    `xtask icons` exists); **fonts queued through the shared `TextStyles` handle**
    (`ensure_font`, once per table by address; the `TextSystem` registers queued fonts
    before its next layout), so a widget or theme registers its own font with no app or
    tester setup; `IconData { code_point, family, font }`; `Icon::size` not reactive and the
    family fixed at build; `IconButton` on the toggles' pressable core (no `on_press` =
    disabled, on-surface state layer).
53. **Image** (2026-10-11, specs `docs/specs/widgets/image.md` and `docs/specs/layout/image.md`,
    decided by the agent while the user was away; review): `RenderImage` sizing in
    `tantu-layout` (Flutter's, with `BoxConstraints::constrain_size_and_attempt_to_preserve_aspect_ratio`),
    the fit paint-only in the widget (Flutter's `applyBoxFit` table in the spec); PNG decoding
    in `tantu-widgets` with the `png` crate (`decode_png`; the software renderer keeps its
    copy); images reach the renderer's resources through the frame's painter
    (`TextPainter::image_id` → `Resources::intern_image`, once per pixel buffer; renaming the
    trait `FramePainter` is a later cleanup); no eviction yet; `Image::new` takes decoded
    `ImageData`.
54. **RichText** (2026-10-11, spec `docs/specs/widgets/rich-text.md`, decided by the agent
    while the user was away; review): spans as a parallel `&[StyleSpan]` list (byte range,
    style key, optional color) rather than a new text model, with `measure_rich` /
    `min_intrinsic_width_rich` / `paint_rich` defaulting to the plain methods so every
    measurer and painter stays valid; colors ride as parley brushes (`Layout<SpanBrush>`), so
    runs split by color and the painter emits one glyph run per parley run; `TextSpan` sets
    a full style or field overrides (`size`, `bold`, `italic`, `family`, `color`) instead of
    Flutter's partial `TextStyle.merge`; the measure and layout caches key by the spans too;
    underline/strikethrough, inline widgets and tap recognizers deferred to `TextField`/links.
55. **Text editing model** (2026-10-11, spec `docs/specs/widgets/text-editing.md`, decided by
    the agent while the user was away; review): byte offsets on character boundaries (not
    Flutter's UTF-16 units); `unicode-segmentation` 1.13 (already a transitive dependency)
    for graphemes and words, a word being a UAX #29 segment with an alphanumeric character;
    forward-by-word goes to the next word's end, backward to the previous word's start;
    undo coalescing by edit kind (single graphemes typed at the caret; whitespace joins and
    ends the group) instead of Flutter's 500 ms throttle; `edit`s that only move the
    selection record no step, like `select`; the model lives in `tantu-widgets`.
56. **TextField widget** (2026-10-11, spec `docs/specs/widgets/text-field.md`, decided by the
    agent while the user was away; review): the paragraph's geometry as plain data from the
    frame's painter (`TextGeometry`), kept by the widget so handlers (which have no text
    context) hit-test and move vertically between frames; keys through `on_key` (repeats and
    modifiers), chords through the shortcut machinery (apps can rebind); the outer element
    holds focus, handlers and outline (as the Slider); the field fills a bounded width, 280 px
    in an unbounded one; typed text comes from `KeyEvent::text` or the `Character` key; no
    clipboard (platform), double-click (timers), scrolling (scroll item) or IME (next part).
57. **TextField IME** (2026-10-12, spec `docs/specs/widgets/text-field-ime.md`, the draft's
    proposals): winit's event shape as the platform's (byte offsets); the IME allowed only
    while the focused element provides an area (a provider registered at build time,
    `BuildCx::ime_area`, since the caret rect is paint-time data); the composing underline as
    a 1 px Scene rect; composition and commit one undo step; `Disabled` commits the pending
    preedit. The runner syncs the area after each frame, so a widget wanting the IME must
    request a frame when its focus changes (text fields do through their `focused` signal).
58. **ScrollView** (2026-10-12, specs `docs/specs/layout/viewport.md` and
    `docs/specs/widgets/scroll.md`, decided by the agent while the user was away; review):
    the scroll offset is the child's *layout* offset (hit-testing and culling need nothing
    new; a changed offset is a cheap re-layout); the viewport doesn't clamp, the widget does
    from the extents it records at paint (which is why paint now enters the runtime,
    VIEW-PAINT-07); deltas apply at once (no kinetic phase until timers); the innermost view
    that can move takes the wheel (it stops the event only when it moved); only the axis's
    delta component scrolls; a `Copy` controller of two signals; a fixed-look scrollbar shown
    whenever there is overflow, draggable, 24 px minimum thumb.
59. **ListView** (2026-10-12, specs `docs/specs/layout/list.md` and
    `docs/specs/widgets/list-view.md`, decided by the agent while the user was away; review):
    fixed item extents only (the fast path); the content is a box as long as all items
    (`count · extent`) holding only the built ones, placed by `ListIndex` parent data; the
    visible range comes from the scroll controller's offset and viewport extent with one item
    of buffer each side (not Flutter's 250 px cache extent); items keyed by index keep their
    elements across scrolls; the first frame shows no items because the extents come from it.
60. **Gallery** (2026-10-12, spec `docs/specs/examples/gallery.md`, decided by the agent while
    the user was away; review): six sections keyed by name, state echoed as text so tests and
    readers see it, a generated gradient image (no asset file), goldens of two sections.
61. **Inherited values** (2026-10-12, spec `docs/specs/view/inherited.md`, decided by the agent
    while the user was away; review): values attach to the element being built (for a leaf,
    its parent) and are found by walking up at build time; reactivity comes from providing
    `Copy` signal handles; no reactive re-lookup (a provider must be above its consumers when
    they build).
62. **Theme** (2026-10-11, spec `docs/specs/theme/theme.md`, ADR 0012; the user said
    "continue" on the draft's five proposals; additions marked *review*): `tantu-theme` holds
    `Theme { colors: ColorScheme, text: TextTheme, shape: Shape, density: Density }` with
    `Theme::light()` (the widgets' old constants; `surface` is white, not Material's
    `0xFFFFFBFE`) and `Theme::dark()` (Material 3 baseline dark). The carrier is a
    `Signal<Theme>` provided as an inherited value (`Theme::of`, `Theme::signal`,
    `ThemeProvider`); **AGENTS.md table change:** `tantu-widgets | view, theme` and
    `tantu-test | view, theme, render-soft, render-headless`. Widgets keep a `ColorScheme`
    snapshot in their paints, bound to the signal; density (4 px per step, clamped to ±3)
    adjusts the minimum extents of buttons, toggles, icon buttons and text fields; text presets
    are read at build time only. The runner provides the light theme per window
    (`default-theme` feature, now a behavior flag, `tantu-theme` always compiled) and the
    window background follows the theme's `surface` unless `Window::background` was set; the
    tester provides it too (`tester.theme()`) and composites goldens over the surface (every
    golden regenerated, opaque now). `Builder` (Flutter's) added to `tantu-view` so app code
    reaches a `BuildCx`. The tooltip captures the theme at the portal, because the overlay
    layer hangs off the root and inherits nothing from the app (open question). The gallery
    has a "Dark" switch and a dark golden.
63. **Timers and tickers** (2026-10-11, spec `docs/specs/reactive/timers.md`; the user said
    "continue" on the draft's seven proposals): the clock lives in `tantu-reactive`
    (`timer`, `Timer`, `ticker`, `Ticker`, `Wake`, `Runtime::tick/now/next_wake`), timers and
    tickers being arena nodes owned like signals, so handlers can create them and scopes
    dispose them; plain `std::time::Instant`/`Duration`; one tick per frame at the start of
    the redraw, `idle` only scheduling (a due tree gets a redraw, a future deadline goes to
    `PlatformContext::wake_at`, a passed or changed one is cleared, since winit's `WaitUntil`
    would fire again at once); the fake platform has a clock moved by `FakeStep::Advance` and
    logs `wakes`; `WidgetTester::advance` is one frame; the tooltip waits Flutter's 500 ms on
    hover (`wait_duration`), focus shows at once, and its timer runs in the portal's scope
    (`BuildCx::tree().scope(id)`), the first workaround for the handler-ownership question.
    PLATFORM-WINIT-14 was checked in a real window on Wayland.
64. **Animation** (2026-10-11, spec `docs/specs/view/animation.md`; the user said "ok" on the
    draft's six proposals): the module lives in `tantu-view`; `Curve` is an enum (Flutter's
    named curves plus `Cubic`, solved by bisection); `Lerp` for the core value types and
    `Tween`; `AnimationController` is a `Copy` handle over two signals, a plan signal and a
    ticker (Flutter's `forward`/`reverse`/`animate_to`/`animate_back`/`repeat`/`stop`/
    `reset`/`set_value`, remaining-fraction durations, repeats carrying the overshoot over);
    `Animation<T>` and `Animated<T>` are `IntoProp`, `animated(target, d)` being the implicit
    form (a transition from the shown value on each target change, default `EaseInOut`).
    **VIEW-ANIM-08**, found when the gallery hung: control methods and `animated()`'s first
    read are untracked, otherwise a `Dyn` building them subscribes to the controller's value
    (or the target) and rebuilds on every tick. Rule of thumb for the future: anything
    constructed while views are built must read signals untracked unless it means to re-run
    the enclosing `Dyn`. The gallery's Animation section has a painted `ColorBox` (a local
    custom-paint view, the first in an example since the layout demo's `Swatch`).
65. **Transitions** (2026-10-11, spec `docs/specs/widgets/transitions.md`; the user said "next"
    on the draft's seven proposals): `Opacity` pushes a Scene layer only between 0 and 1
    (nothing at 0, no layer at 1; hit-testing unchanged); `AnimatedSwitcher` is a `Stack` over
    a keyed `For` of entries (key, fade controller, leaving flag) with per-entry effects that
    drop an entry once its fade out reaches `Dismissed`, so a key coming back reverses its
    fade in place; the animated `Show` is a switcher on a `bool`, not a change to `Show`; the
    caret blinks with a chain of 500 ms timers in the field's scope, restarted by an effect on
    focus, text and selection; double-click uses a pending 500 ms timer as the click interval
    (handlers have no clock), Flutter's 4 px slop, and UAX #29 segments (`segment_at`), with
    drags extending by whole segments; wheel steps animate to an accumulating target over
    150 ms with `EaseOut` (`ScrollView::wheel_duration`, zero for the old behavior), driven by
    an `AnimationController` inside `ScrollController` whose effect writes the offset while
    animating; `jump_to`, keys, the thumb and `animate_to` interrupt it. Kinetic fling is
    deferred until touch or trackpad gesture events exist. The older wheel tests set a zero
    wheel duration.
66. **ListView extents, separators and keys** (2026-10-12, spec
    `docs/specs/widgets/list-view-advanced.md`; the user said "continue" on the draft's five
    proposals): `ListExtents` (`Fixed` or a `Table` of prefix sums, `Arc<[f32]>`) on
    `RenderListContent`, which is no longer `Copy`; the widget maps "slots" (items, or items
    interleaved with separators) to a table built by a memo once per count change (the
    extent builder runs exactly `count` times) and finds the visible range by binary search,
    excluding the slot that starts exactly at the viewport's end; `builder_with_extents` and
    `separated` share the engine; selection is an app-owned `Signal<Option<usize>>`, the
    list's capture-phase key handler (before the scroll view's own keys) moves it with
    arrows, Home/End, PageUp/Down (items that fit) and jumps it into view, Enter and
    double-click call `on_activate`, a press selects and still bubbles to the item. Measured
    extents stay a later mode. The gallery's Lists section has a contacts list with headers.

67. **Semantics and the AccessKit tree** (2026-10-12, spec `docs/specs/a11y/semantics.md`,
    ADR 0013; the user said "continue" on the draft's seven proposals; VIEW-SEM-03 amended
    twice while implementing): `Semantics { role, label, value, states, range }` with a
    `Role` enum is plain data on elements in `tantu-view` (`BuildCx::set_semantics`,
    `ElementMut::update_semantics`), so the view layer and the widgets never see AccessKit;
    `tantu-a11y` walks the element tree and emits one node per semantic element (unsemantic
    layout and region elements are skipped and their descendants re-parented, so trees stay
    shallow), node ids are the element ids' bits, bounds come from the layout tree, focus
    from the tree; full-tree updates, AccessKit diffs; `Focus`, `Click`, `Increment` and
    `Decrement` are synthesized as user input (a click is a press and release at the
    center, increment an arrow key), while `SetValue` and `ScrollIntoView` go to `on_action`
    handlers that take an `ActionCx` and bubble up from the target (the scroll view's offset
    lives in the widget), and `ScrollIntoView` is accepted on any semantic element;
    `semantics_label` builders on `Icon`, `IconButton` and `Image` instead of a `Semantics`
    wrapper widget; `WidgetTester::a11y_tree` and `perform_action` (**AGENTS.md table
    change:** `tantu-test` may depend on `tantu-a11y`); a gallery test checks every widget
    has a node. AccessKit 0.24 as a plain dependency of `tantu-a11y` and `tantu-platform`.
    Text-editing semantics (caret and selection ranges) are a later addition.
68. **Accessibility through the platform** (2026-10-12, spec `docs/specs/a11y/platform.md`;
    the user said "continue" on the draft's four proposals): AccessKit's own `TreeUpdate`
    and `ActionRequest` cross the `Platform` trait, as `AccessibilityEvent::{Activated,
    Deactivated, Action}` in `WindowEvent::Accessibility` and
    `PlatformContext::update_accessibility`; `FakePlatform` scripts the events and records
    every update in `FakeWindow::accessibility_updates`; the winit shell creates an
    `accesskit_winit` 0.32 adapter per window before showing it (the window is created
    hidden, then shown), its events arrive through an event-loop proxy as the loop's user
    event, every winit window event goes to the adapter first, and updates use
    `update_if_active`; the runner sends a full tree on activation and after every frame of
    an active window, nothing while inactive, and maps actions with
    `AccessibilityTree::map_action` to `ViewTree::perform_action`; the app name is the first
    window's title. Checked by the opt-in window test (an update is sent after the adapter
    requests the tree), not by a screen reader.
69. **Focus scopes and scrolling into view** (2026-10-12, spec
    `docs/specs/view/focus-scope.md`; decided by the agent while the user was away, review):
    `ViewTree::ensure_visible(element)` delivers the `ScrollIntoView` action to the
    `on_action` handlers of every ancestor, parent to root, and a container that handles it
    becomes the target for the containers above it (Flutter's `Scrollable.ensureVisible`
    walk-up, so nested scroll views both scroll from current bounds); `focus_next` and
    `focus_previous` (so Tab) call it on the new focus, while `focus`, a focusing press and
    `focus_into` don't; `perform_action(ScrollIntoView)` is `ensure_visible` (VIEW-SEM-03
    amended: every container, not the first). The `ScrollView` aligns a target's start when
    it is longer than the viewport or starts before it, else its end, immediately and
    clamped, and takes its extents from the last layout so a culled (never painted) view
    still scrolls. Focus scopes: `BuildCx::focus_scope` marks a render or region element;
    traversal is confined to the innermost scope enclosing the focus (nested scopes'
    contents included, so Tab enters a nested scope and is then trapped there, as in
    Flutter); every enclosing scope remembers the focus and `ViewTree::focus_into(scope)`
    restores it or focuses the first traversable descendant; removal forgets. `FocusScope`
    widget (a region), with no golden or gallery entry since it paints nothing (review).
    `WidgetTester::ensure_visible`. Phase 3 is complete with this item.
70. **DataGrid part 1** (2026-10-12, specs `docs/specs/layout/grid.md` and
    `docs/specs/widgets/data-grid.md`; the user said "continue" on the drafts' twelve
    proposals): `RenderGridBody` in `tantu-layout` is the two-dimensional
    `RenderListContent` (row and column `ListExtents`, a `Vec2` offset, `GridSlot` parent
    data, the incoming maximum on bounded axes, frozen columns placed from the start);
    `DataSource` is synchronous and pull-based (`row_count`, `cell(row, column) ->
    CellValue`) with an optional `revision` signal for changes (default text cells re-read
    in place, builder cells rebuild through a `Dyn`); `FnSource` for generated data;
    `GridColumn` (title, width, optional cell builder); `DataGrid` composes a column of two
    `RenderGridBody` instances (a one-row header strip following the horizontal offset only,
    and the body) and paints its own two scrollbars, sharing the scroll view's bar geometry
    and controller internals (`pub(crate)` in `scroll.rs`); cells are one `For` keyed by
    (row, column) over the visible range with one of buffer each side, each a clipping
    wrapper with grid lines; wheel `y` scrolls rows and `x` columns (the smooth wheel
    animation applies), the thumbs drag, provisional keys scroll (arrows one row or 40 px,
    PageUp/Down the body, Home/End) until the selection part; roles `Grid`, `ColumnHeader`
    and `Cell` in `tantu-view` and the AccessKit mapping; the gallery's Data grid section
    (every gallery golden regenerated because the sidebar gained a button). **Benchmark:**
    `cargo bench -p tantu-widgets`, scenario shared with a test; 1.30 ms per step. The first
    measurement read 30 ms because `WidgetTester::scroll` looks its element up by key on
    every call and that walk dominated; the scenario now dispatches at a cached position
    (open question below). Found while writing the tests: cells appear one frame after the
    first paint records the extents (as list items), so the tests pump twice.
71. **DataGrid frozen columns** (2026-10-12, WIDGETS-GRID-09 in
    `docs/specs/widgets/data-grid.md`; decided by the agent, review): `frozen_columns(n)`
    (clamped to the column count) sets `RenderGridBody::frozen_columns` on the header and
    the body, so the first `n` columns are placed at their starts (LAYOUT-GRID-04); the
    visible-column list is the scrolling columns between the frozen width and the right edge
    (buffer one each side, never a frozen one) followed by the frozen ones, so the `For`
    orders frozen cells last and they paint on top (element order rather than a z-index,
    which set inside a cell's paint wouldn't reorder its scope among its siblings); frozen
    cells fill a `surface` background (opaque, since `paint_children` can't clip the
    scrolling cells per child) and the last frozen column's right line is the theme's
    `outline`; the horizontal extents are unchanged. The gallery freezes its Id column with
    wider columns (its golden regenerated). Found while testing: a cell one buffer row below
    the window is culled (no commands), and the gallery's grid scrolls horizontally by only
    its overflow, so the tests count painted rows and check direction, not distance.
72. **DataGrid sorting and filtering** (2026-10-12, spec
    `docs/specs/widgets/data-grid-sort.md`, its own file because the coverage check reads a
    file's status and draft rules can't sit in an Implemented spec; the user said
    "continue" on the draft's seven proposals): the sort state is a `Signal<Option<SortState>>`
    the grid owns or shares (`sort_state`), cycled by a primary press on a sortable header
    (ascending, descending, none; sortable = `GridColumn::sortable(true)` or `client_sort`),
    shown as a painted two-bar chevron in `on_surface_variant` (no font); ordering is the
    app's job by default (read the state, reorder the source, bump `revision`), and
    `client_sort(true)` makes the grid order a row map by `CellValue::compare` (Empty <
    Bool < numbers as f64 with NaN lowest < Text in byte order; a stable sort, descending
    reversing the comparison); `filter(signal)` holds an `Rc<dyn Fn(&dyn DataSource, usize)
    -> bool>` applied before sorting, in its own memo so a sort change doesn't re-run it;
    display rows map to source rows through `Rc<[usize]>` memos (none without a filter or
    client sort), the default text re-reading in place and builders seeing source rows; the
    gallery's columns are sortable with a "Sort: …" echo and no client sort. Found while
    testing: the gallery's Score header sits mostly past the grid's edge (its test taps
    City), and one-letter column titles collided with one-letter cell texts in a test.
73. **Kitchen sink** (2026-10-12, spec `docs/specs/examples/kitchen-sink.md`; the user said
    "continue" on the draft's six proposals): a ninth gallery section, "Kitchen sink", rather
    than a new example crate; one `ScrollView` (key `sink`, inside an `Expanded` so the
    content column bounds it) over a 720 px column of twelve cards (`sink-<card>`: buttons,
    toggles, sliders, typography, fields, media, lists, grid, layout, scrolling, animation,
    focus), each a title, samples keyed `sink-<widget>` and an echo `Text` keyed
    `sink-<card>-state`; the public `SINK_CARDS` constant lists the card keys and their
    sample keys and the test iterates it (no reflection over exports); the grid card is the
    first use of `client_sort` with a filter switch (1 000 rows); the focus card wraps its
    buttons in a local `FocusEcho` view registering `on_focus_change`; two goldens (top, and
    the end after a tap near the top focuses the page and End scrolls it). Found while
    testing: card keys must not equal sample keys (`sliders`, `typography`), a tap or wheel
    at the center of a card below the window hits nothing (the test taps the "Toggles"
    title), and buttons don't take focus on press (the test uses `WidgetTester::focus`).
    Every gallery golden was regenerated for the sidebar's new button.
74. **DataGrid column resize and reorder** (2026-10-12, spec
    `docs/specs/widgets/data-grid-columns.md`; the user said "continue" on the draft's seven
    proposals): column widths (`column_widths`) and the display order (`column_order`) are
    `Signal<Vec<f32>>` / `Signal<Vec<usize>>` indexed by source column, the grid's own
    unless the app shares them, so sort state, builders, filters and widths keep addressing
    columns by their stable source index; the grid's geometry holds a memo of the display
    layout (normalized order plus an extent table in display order; a short widths vector
    falls back to `GridColumn::width`, entries are cleaned then floored at `min_width`,
    default 40 px), both strips bind their `RenderGridBody` extents to it, and cells are
    keyed by (row, display, source) so a moved column's cells are rebuilt at their new
    position; a header cell's one pointer handler tells apart a resize (a press in the 6 px
    handle inside the column's right edge, live width updates, `resizable(false)` has no
    handle), a reorder drag (over 4 px with `reorderable(true)`: a 2 px `primary` drop line
    at the display boundary nearest the pointer, painted by the header strip, and the held
    header filled `surface_container_highest`; a drop at the column's own boundary or the
    next changes nothing) and a click (press and release within the slop, which is what
    sorts now, WIDGETS-GRID-15 amending -10); frozen columns count display positions. No
    resize cursor (`CursorIcon` has no such variant; a later view/platform amendment).
    Three slips fixed while agreeing: the handle lies inside the column it resizes, the
    scheme has no `surface_variant`, and the gallery's City drag needs 200 px to pass the
    middle of Name. Found while testing: a press at x = 300 in a 300 px window hits
    nothing (WIDGETS-GRID-14's grid got 600 px). The README's "today/planned" lines were
    stale since Phase 3 and now mention the grid and accessibility.

75. **DataGrid selection and keyboard navigation** (2026-10-12, spec
    `docs/specs/widgets/data-grid-selection.md`; recommended decisions taken on the user's
    authorization to continue autonomously). Display-coordinate anchor/active endpoints
    represent Row intervals, single Cell selection or Range rectangles, without materializing
    the selected area. `GridSelectionController` maps any display cell through the shared
    row map and normalized column order. Selection is opt-in; model changes clear it because
    `DataSource` has no stable row identities. Keyboard focus stays on the grid while cells
    virtualize; F6 switches its body/header cursor and semantics exposes a validated
    `active_descendant`. Header keys sort, resize and reorder. Shift-click required adding
    modifiers to view pointer events and per-window runner snapshots. Existing theme tokens
    supply an opaque 12% primary tint over surface; the draft's nonexistent secondary
    tokens were corrected in a spec/test amendment. `ensure_visible` walks from a buffered
    cell's descendant to its cell metadata. No architecture edges or stages changed.

76. **DataGrid cell editors** (2026-10-12, spec `docs/specs/widgets/data-grid-editing.md`;
    the user accepted eight proposals with "continue"). Editable columns replace one cell's
    content with a scoped TextField; sessions snapshot source indices and the original value
    type. `DataSource::set_cell(&self, ...)` is synchronous with a default ReadOnly error;
    `GridEditError` and `on_edit_error` let the app surface conversion/source rejection.
    Enter writes once, composition blocks submission, and failed writes retain drafts.
    Cancellation follows Escape, Tab, blur, model changes and complete viewport exit; no
    hidden editor is pinned. Local refresh keeps read-only scrolling unchanged and rebuilds
    active client maps after successful writes. Field controllers/undo live in the dynamic
    cell's version scope. The integration amendment `BuildCx::focus` is marked review;
    it follows ViewTree::focus and enables post-build entry and restoration. The gallery
    stores only edited overrides and validates Score; its editor and selection goldens
    show the existing 32 px rows. Benchmark: 0.804 ms, no measured regression.

77. **DataGrid stress example and kitchen-sink refresh** (2026-10-12; four stress proposals
    accepted by the user's "continue"; kitchen sink and commit/push explicitly requested).
    Match the existing CPU workload, keep source generation constant-space, and use the
    existing owner-scoped ticker rather than add a Phase 5 profiler. Report UI tick cadence
    honestly and document a repeatable native measurement procedure. The example is complete;
    the project-wide native hardware criterion stays open. The kitchen sink composes the
    completed grid APIs and shares sparse editable-source logic with the Data grid section.

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
| `ab86cee` | docs: HANDOFF.md for the text system (CI red on Linux: fontconfig) |
| `73999ee` | spec: text in views (VIEW-TEXT-01..05); paint and frame take a text context |
| `cd9d06b` | spec: `PaintCx::paint_text` |
| `c5d5a13` | test: text in views, 5 failing tests |
| `a19b609` | fix: load fontconfig at run time on Linux (fontique `fontconfig-dlopen`) |
| `50a80c8` | impl: text in views |
| `af91790` | docs: HANDOFF.md for text in views and the fontconfig fix |
| `88afc14` | spec: glyph rasterizer (TEXT-RASTER-01..04); PLAN.md glyph item split |
| `6582f9c` | test: glyph rasterizer, 4 failing tests; swash dependency |
| `74ac56f` | impl: glyph rasterizer |
| `1612df7` | docs: HANDOFF.md for the glyph rasterizer |
| `ce8eaa6` | spec: glyph runs in tantu-render-soft (RENDER-SOFT-27, -28); TEXT-RASTER-05 |
| `f1e73ed` | test: glyph runs in render-soft, readable fonts; 3 failing tests |
| `fc17fee` | impl: glyph runs in tantu-render-soft; `GlyphRasterizer::readable` |
| `d8de131` | docs: HANDOFF.md for text in the software renderer |
| `c478d34` | spec: glyph runs in tantu-render-wgpu (RENDER-WGPU-18, -19) |
| `b95b516` | test: glyph runs in render-wgpu, 2 failing tests |
| `49ebfa7` | impl: glyph runs in tantu-render-wgpu with a glyph atlas |
| `d5d987d` | docs: HANDOFF.md for text in the wgpu renderer |
| `aaf28fe` | spec: a text reference Scene (RENDER-CONF-14); CONF-09 allows fresh font handles |
| `9ef307a` | test: text reference Scene, 2 failing tests |
| `a0afa0c` | test: RENDER-CONF-08's unknown-name examples no longer include `text` |
| `c6d7a2f` | impl: text reference Scene and golden; PLAN.md glyph item ticked |
| `b2cf088` | docs: HANDOFF.md for the text reference Scene |
| `f387d8b` | docs: draft event-dispatch spec for review |
| `58dd2b1` | docs: HANDOFF.md, waiting on the event-dispatch review |
| `19b77d9` | spec: event dispatch (VIEW-EVENT-01..06) |
| `1ad5e1b` | test: event dispatch, stubs + 7 failing tests |
| `d1b19b7` | impl: event dispatch in tantu-view |
| `038eec0` | docs: HANDOFF.md for event dispatch |
| `42db279` | docs: draft facade specs for review; PLAN.md facade item split in five |
| `e45d8a9` | docs: HANDOFF.md, waiting on the facade spec review |
| `63a53a5` | spec: shared text styles (TEXT-STYLES-01..04, VIEW-STYLES-01) |
| `72d703b` | test: shared text styles, stubs + 5 failing tests |
| `5c773c7` | impl: shared text styles |
| `b08c85e` | spec: layout widgets (WIDGETS-LAYOUT-01..06) |
| `678ad21` | test: layout widgets, stubs + 7 failing tests |
| `dcdcb3f` | impl: layout widgets |
| `d109e27` | spec: Text and Button (WIDGETS-TEXT, WIDGETS-BUTTON, VIEW-EVENT-07) |
| `36c5397` | test: Text and Button, stubs + 7 failing tests |
| `58edd71` | impl: Text and Button |
| `e6354c5` | docs: HANDOFF.md for the widgets |
| `8e78e6b` | spec: the tantu facade and app runner (FACADE-APP-01..09) |
| `8936752` | test: the tantu facade, stubs + 9 failing tests |
| `3dca84a` | test: FACADE-APP-03 finds the button with a transform stack |
| `59303ae` | impl: the tantu facade and app runner |
| `af04a04` | chore: CI tests the facade without default features |
| `a999bf2` | spec: examples/counter (COUNTER-01..02) |
| `b0d9eb4` | test: examples/counter, stub + 2 failing tests |
| `e0eea67` | impl: examples/counter is the AGENTS.md snippet |
| `5a6052b` | docs: HANDOFF.md for the facade and the counter |
| `1f1ac46` | docs: draft WidgetTester spec for review |
| `3bcfa50` | docs: HANDOFF.md, waiting on the WidgetTester review |
| `cf760f3` | spec: WidgetTester (TEST-WT-01..06, VIEW-KEY-01) |
| `e6ce21e` | test: WidgetTester and Keyed, stubs + 8 failing tests |
| `a90a104` | test: TEST-WT-06 stops after writing the golden in update mode |
| `869a795` | impl: WidgetTester and Keyed |
| `43a00ab` | docs: HANDOFF.md for WidgetTester |
| `c4aad84` | docs: draft layout-demo spec for review |
| `69fa90d` | spec: layout demo (LAYOUT-DEMO-01..04) |
| `148c374` | test: layout demo, stub + 4 failing tests |
| `0dd28f5` | impl: layout demo |
| `2f8e370` | spec: windows paint a background color (FACADE-APP-10) |
| `d8da396` | test: windows paint a background, failing |
| `6a3e5b0` | fix: windows paint their background color |
| `548dc60` | docs: HANDOFF.md for the layout demo and the background fix |
| `af7b73c` | docs: Phase 2 milestone ticked (user checked Windows and macOS) |
| `893bfae` | docs: draft bidi and font fallback spec for review; PLAN.md sub-item |
| `6247bd6` | spec: bidi and font fallback (TEXT-SCRIPT-01..04) |
| `c3e4fb5` | test: bidi and font fallback, 1 failing test |
| `aa4809b` | fix: per-character font fallback through every registered family; Phase 2 done |
| `0fe42c8` | docs: HANDOFF.md for the end of Phase 2 |
| `6c0f861` | docs: draft focus spec for review; PLAN.md Phase 3 focus item split in three |
| `40c84a8` | spec: keyboard events and focus (VIEW-FOCUS-01..07, FACADE-APP-11, TEST-WT-07) |
| `245319a` | test: keyboard events and focus, stubs + 9 failing tests |
| `ea9f69a` | test: VIEW-FOCUS-07 expects the root's handlers to remain |
| `d384d4a` | test: focus fixture registers root key handlers from the top view |
| `3cd7918` | impl: keyboard events and focus |
| `16dd666` | docs: HANDOFF.md for keyboard events and focus |
| `98d74db` | docs: draft shortcuts spec for review |
| `89f861e` | spec: shortcuts and commands (VIEW-SHORT-01..06) |
| `6f2fa3e` | test: shortcuts and commands, stubs + 6 failing tests |
| `55560ee` | test: shortcuts fixture keeps the field's own key |
| `c349569` | impl: shortcuts and commands |
| `e27fbea` | docs: HANDOFF.md for shortcuts and commands |
| `6a0018b` | docs: draft Button keyboard spec for review |
| `b91b405` | spec: Button keyboard activation and focus indicator (WIDGETS-BUTTON-05..07, VIEW-FOCUS-08) |
| `a883753` | test: Button keyboard activation and focus indicator, 4 failing tests |
| `f54d9a7` | test: WIDGETS-BUTTON-06 counts the ignored repeat that bubbles on |
| `a9094bf` | impl: Button keyboard activation and focus indicator |
| `d410de0` | docs: HANDOFF.md for Button keyboard activation |
| `2f04b5e` | docs: draft overlay spec for review |
| `bf722ec` | spec: overlay and anchored positioning (VIEW-OVERLAY-01..07) |
| `c8e8cdf` | test: overlay and anchored positioning, stubs + 7 failing tests |
| `e58b3d5` | impl: overlay and anchored positioning |
| `f883007` | docs: HANDOFF.md for the overlay |
| `f0c975e` | docs: PLAN.md widget item split; draft Tooltip spec for review |
| `29487fd` | spec: Tooltip (WIDGETS-TOOLTIP-01..06, VIEW-FOCUS-09) |
| `341acce` | test: Tooltip and focus within, stubs + 7 failing tests |
| `ed76c10` | impl: Tooltip and focus within |
| `3aa5372` | docs: HANDOFF.md, paused (Tooltip done) |
| `f90a4ed` | docs: draft Divider and Slider specs for review; PLAN.md item split in two |
| `14163c2` | spec: Divider and VerticalDivider (WIDGETS-DIVIDER-01..04); RenderLimitedBox (LAYOUT-SINGLE-16), LimitedBox (WIDGETS-LAYOUT-07) |
| `2bd1ad8` | test: Divider, VerticalDivider, RenderLimitedBox and LimitedBox, stubs + 6 failing tests |
| `fece7dc` | spec: LAYOUT-TREE-13 holds for a boundary cleaned by an ancestor's pass |
| `3e63827` | test: a boundary cleaned by an ancestor's pass is queued again, failing |
| `96a932b` | fix: dirty boundaries leave the queue when a pass handles them |
| `6dfb402` | impl: Divider and VerticalDivider, RenderLimitedBox and LimitedBox; divider golden |
| `8b879cb` | docs: HANDOFF.md for the Divider |
| `5ace032` | spec: Slider (WIDGETS-SLIDER-01..08) and WidgetTester::drag (TEST-WT-08) |
| `aa62f9a` | test: Slider and WidgetTester::drag, stubs + 9 failing tests |
| `54428be` | spec: the Slider's outer element holds its handlers, focus and paint |
| `377570f` | test: TEST-WT-08's recorder ignores the hover enter after the release |
| `b04a955` | impl: Slider and WidgetTester::drag; slider golden |
| `50f567f` | docs: HANDOFF.md for the Slider |
| `ddf600a` | spec: Checkbox, Radio and Switch (WIDGETS-TOGGLE-01..09) |
| `c78fb2d` | test: Checkbox, Radio and Switch, stubs + 9 failing tests |
| `2001951` | test: toggle tests read the toggle's own entries and compare sizes (the column centers it) |
| `b63ed0e` | impl: Checkbox, Radio and Switch; three goldens |
| `fa91fb4` | docs: HANDOFF.md for the toggles |
| `916fd49` | spec: Icon and IconButton (WIDGETS-ICON-01..04); TextStyles::ensure_font (TEXT-STYLES-05, VIEW-STYLES-02) |
| `a554d35` | test: Icon, IconButton and the font queue, stubs + 5 failing tests; Material Icons font and Icons table |
| `501226d` | spec: an Icon's family and font come from its first value (WIDGETS-ICON-03) |
| `ce84ffa` | impl: Icon and IconButton, the font queue in TextStyles; icon goldens |
| `ae06e43` | docs: HANDOFF.md for Icon and IconButton |
| `630f3ae` | spec: Image (WIDGETS-IMAGE-01..05), RenderImage (LAYOUT-IMAGE-01..02, LAYOUT-CONS-18), Resources::intern_image (SCENE-RES-11), PaintCx::image_id (VIEW-TEXT-06) |
| `f0fba6a` | test: Image, RenderImage, intern_image and image_id, stubs + 11 failing tests |
| `83fd47b` | impl: Image and decode_png, RenderImage, Resources::intern_image, PaintCx::image_id; image golden |
| `f951983` | docs: HANDOFF.md for Image |
| `61304f7` | spec: RichText and TextSpan (WIDGETS-RICHTEXT-01..03); style spans in measuring and painting (LAYOUT-TEXT-11..12, TEXT-SYS-10..11, VIEW-TEXT-07) |
| `e61beb3` | test: RichText, TextSpan and style spans, stubs + 8 failing tests |
| `503f051` | spec: BuildCx::text_styles (VIEW-STYLES-03), for reactive spans |
| `7b27a42` | test: BuildCx::text_styles, stub + 1 failing test |
| `8330414` | test: WIDGETS-RICHTEXT-02 expects the bigger span as its own run |
| `97502d4` | impl: RichText and TextSpan; style spans through measuring, the text system (parley ranges and brushes) and painting; rich-text golden |
| `5a266d2` | docs: HANDOFF.md for RichText |
| `83e9521` | spec: text editing model (WIDGETS-EDIT-01..08); PLAN.md TextField item split in three |
| `1c0bcf7` | spec: WIDGETS-EDIT-07, a whitespace grapheme joins the coalescing group and ends it |
| `4d2ee26` | test: text editing model, stubs + 8 failing tests |
| `661ba04` | test: WIDGETS-EDIT-08 expects redo to restore the moved caret |
| `0b04547` | impl: text editing model (EditingValue, TextEditingController with undo and coalescing) |
| `fe9f690` | docs: HANDOFF.md for the text editing model |
| `441bec9` | spec: TextField widget (WIDGETS-TEXTFIELD-01..06), TextGeometry (LAYOUT-TEXT-13, TEXT-SYS-12, VIEW-TEXT-08), WidgetTester::enter_text (TEST-WT-09) |
| `4d3e95c` | test: TextField, TextGeometry, geometry queries and enter_text, stubs + 9 failing tests |
| `6e0d221` | test: TextField tests compare paint in the editor's coordinates and give Tab a next element |
| `3fe08e2` | spec: the TextField's outer element holds its focus, handlers and outline |
| `f1664ce` | impl: TextField widget, TextGeometry and its queries, TextSystem::geometry, PaintCx::text_geometry, WidgetTester::enter_text; text-field golden |
| `219157b` | docs: HANDOFF.md for the TextField widget |
| `a7f25e6` | docs: draft TextField IME spec for review |
| `c925aff` | docs: HANDOFF.md, paused at the IME draft |
| `a7f25e6` | docs: draft TextField IME spec for review |
| `c925aff` | docs: HANDOFF.md, paused at the IME draft |
| `55cb337` | spec: TextField IME composition (PLATFORM-IME-01, PLATFORM-FAKE-11, PLATFORM-WINIT-13, VIEW-IME-01..02, FACADE-APP-12, WIDGETS-EDIT-09, WIDGETS-TEXTFIELD-07..08, TEST-WT-10) |
| `421af9f` | spec: VIEW-IME-02 uses an IME area provider registered at build time |
| `23ee41d` | test: TextField IME composition, stubs + 11 failing tests |
| `a0668cc` | impl: TextField IME composition: platform IME events and context calls, winit conversion, view dispatch and IME area, runner sync, compose/commit edits, composing underline, WidgetTester::ime |
| `981277b` | docs: README, how to use Tantu in an app (git dependency, a hello app, what works today) |
| `a06a919` | docs: HANDOFF.md for IME and the README |
| `9b28f44` | spec: RenderViewport (LAYOUT-VIEWPORT-01..02) and ScrollView (WIDGETS-SCROLL-01..05, TEST-WT-11); PLAN.md scroll item split |
| `48a56f4` | test: RenderViewport, ScrollView and WidgetTester::scroll, stubs + 8 failing tests |
| `62fe08e` | spec: paint runs with the runtime current (VIEW-PAINT-07) |
| `1c0d25a` | test: a Paint reads and writes signals (VIEW-PAINT-07), failing |
| `6c3c1a4` | test: WIDGETS-SCROLL-05 content tall enough for the 400 px offset |
| `fa0f963` | impl: RenderViewport, ScrollView with wheel scrolling and a draggable scrollbar, WidgetTester::scroll; paint runs with the runtime current |
| `518e412` | docs: HANDOFF.md for ScrollView |
| `03d30b0` | spec: RenderListContent (LAYOUT-LIST-01..02) and ListView::builder (WIDGETS-LIST-01..03); PLAN.md item split |
| `54a96b2` | spec: WIDGETS-LIST-02 example count at offset 0 is 6 |
| `0f41423` | test: RenderListContent and ListView, stubs + 5 failing tests |
| `ff0f989` | impl: RenderListContent and ListView::builder (fixed-extent virtualization over the viewport) |
| `5929626` | docs: HANDOFF.md for ListView |
| `5e946cd` | spec: the widget gallery example (GALLERY-01..04) |
| `c29c985` | test: the gallery example, stub + 4 failing tests |
| `9100315` | impl: the widget gallery example with two goldens; README points at it |
| `437cd8f` | docs: HANDOFF.md for the gallery |
| `fa0cff3` | spec: single-line TextField caret following (WIDGETS-TEXTFIELD-09) |
| `39c612a` | test: TextField caret following (WIDGETS-TEXTFIELD-09), failing |
| `8af9726` | test: WIDGETS-TEXTFIELD-09 checks the wrapped text in the multi-line case |
| `fcd37bf` | impl: single-line TextField caret following (content shift in a transform scope) |
| `22d919b` | docs: HANDOFF.md for caret following |
| `6e6c652` | spec: inherited values (VIEW-INHERIT-01..03), the groundwork for themes |
| `12d8165` | test: inherited values, stubs + 3 failing tests |
| `e568355` | test: VIEW-INHERIT-02's leaf provides on its parent |
| `f138bbd` | impl: inherited values (BuildCx::provide/inherit, Provide, ViewTree::inherited_on) |
| `5791724` | docs: HANDOFF.md for inherited values |
| `563b0f7` | docs: draft theme spec for review (proposes a dependency-table change) |
| `ddd4190` | spec: ScrollView keyboard scrolling (WIDGETS-SCROLL-06) |
| `6f35fe5` | test: ScrollView keyboard scrolling (WIDGETS-SCROLL-06), failing |
| `6eb1958` | impl: ScrollView keyboard scrolling (arrows, PageUp/Down, Home/End on a focused view) |
| `cfb3d60` | docs: HANDOFF.md for keyboard scrolling and the theme draft |
| `afa673e` | spec: theme (THEME-01..03, WIDGETS-THEME-01..04, FACADE-APP-13, TEST-WT-12, VIEW-TREE-12, GALLERY-05); ADR 0012, AGENTS.md table, architecture doc and diagram |
| `8d3c0f1` | test: theme, stubs + 11 failing tests |
| `09863bf` | spec: WIDGETS-THEME-03 allows content taller than the minimum; goldens render over the theme's surface |
| `121f8ff` | impl: theme: tokens and default themes, Builder, widgets reading the theme, runner and tester providing it, goldens over the surface, the gallery's dark-mode switch; PLAN.md ticked |
| `1769a69` | docs: general-purpose framing (README, AGENTS.md, architecture), CLAUDE.md skill note, HANDOFF.md cleanup |
| `4e58571` | docs: draft timers and tickers spec for review; PLAN.md animation item split in three |
| `a44d2de` | spec: timers and tickers (REACTIVE-TIME-01..05, PLATFORM-TIME-01, PLATFORM-FAKE-12, PLATFORM-WINIT-14, VIEW-TIME-01, FACADE-APP-14, TEST-WT-13, WIDGETS-TOOLTIP-07) |
| `ebbc95d` | test: timers and tickers, stubs + 11 failing tests |
| `62af943` | impl: timers and tickers: the runtime's clock, platform now/wake_at, tree and tester forwarding, the runner's scheduling, the tooltip's show delay |
| `53cdbed` | docs: HANDOFF.md for timers and tickers |
| `612d9dd` | docs: draft animation spec for review |
| `e29b27a` | spec: animation (VIEW-ANIM-01..07, GALLERY-06) |
| `3de03ba` | test: animation, stubs + 8 failing tests |
| `5bd2f91` | spec: VIEW-ANIM-08, control methods never subscribe their caller |
| `3423d11` | test: VIEW-ANIM-08, a controller inside a Dyn builds once |
| `f3911fd` | spec: VIEW-ANIM-08 covers animated()'s first read |
| `bb26bce` | test: VIEW-ANIM-08, an animated prop inside a Dyn doesn't rebuild it |
| `633c536` | impl: animation: curves, Lerp/Tween, AnimationController, Animation and animated(); the gallery's Animation section |
| `742be89` | docs: HANDOFF.md for the animation controller |
| `9b10155` | docs: draft transitions spec for review |
| `b84aa22` | spec: transitions (WIDGETS-OPACITY-01..02, WIDGETS-SWITCHER-01..03, WIDGETS-TEXTFIELD-10..11, WIDGETS-SCROLL-07..08, GALLERY-07) |
| `a2b5398` | test: transitions, stubs + 11 failing tests |
| `872b5d7` | impl: transitions: Opacity, AnimatedSwitcher, caret blink, double-click, smooth wheel, animate_to; the gallery's switcher |
| `6c82565` | docs: HANDOFF.md for the transitions |
| `7065d41` | docs: draft ListView extents, separators and keys spec for review |
| `604e01d` | spec: ListView variable extents, separators and keyboard navigation (LAYOUT-LIST-03, WIDGETS-LIST-04..06, GALLERY-08) |
| `c79b3cc` | test: ListView variable extents, separators and keys, stubs + 5 failing tests |
| `cf8d447` | impl: ListView extent table, separators, selection and keys; the gallery's contacts list |
| `196781e` | docs: HANDOFF.md for the ListView leftovers |
| `9cf4076` | docs: draft accessibility specs and ADR 0013 for review; PLAN.md a11y item split in two |
| `71102b7` | spec: semantics and the AccessKit tree (VIEW-SEM-01..03, A11Y-TREE-01..04, WIDGETS-SEM-01..02, TEST-WT-14, GALLERY-09); ADR 0013 accepted, AGENTS.md rows and dependency rule, architecture doc and diagram |
| `c11b5ee` | spec: VIEW-SEM-03 routes SetValue and ScrollIntoView to bubbling on_action handlers |
| `86eff45` | test: semantics and the AccessKit tree, stubs + 12 failing tests |
| `ade8e36` | spec: on_action handlers take an ActionCx |
| `6d8d2f5` | test: VIEW-SEM-03 handlers take an ActionCx (still failing) |
| `69713ca` | spec: VIEW-SEM-03 accepts ScrollIntoView on any semantic element |
| `1f441bd` | impl: semantics on elements, the AccessKit tree in tantu-a11y, WidgetTester::a11y_tree and perform_action, every widget annotated |
| `1c7a8ab` | spec: accessibility through the platform (PLATFORM-A11Y-01, PLATFORM-FAKE-13, PLATFORM-WINIT-15, FACADE-APP-15) |
| `1d4ded5` | test: accessibility through the platform, stubs + 3 failing tests |
| `798032d` | impl: accessibility through the platform: FakePlatform recording, the accesskit_winit adapter per window, the runner sending trees and routing actions; PLAN.md tantu-a11y ticked |
| `3e0b2ed` | docs: HANDOFF.md for accessibility |
| `9f8111f` | spec: focus scopes and scrolling into view (VIEW-REVEAL-01..03, VIEW-SCOPE-01..04, WIDGETS-SCROLL-09, WIDGETS-FOCUSSCOPE-01, TEST-WT-15); VIEW-SEM-03 amended |
| `aa5bf32` | test: focus scopes and scrolling into view, stubs + 13 failing tests |
| `709425d` | spec: WIDGETS-SCROLL-09 takes the extents from the last layout (a culled view still reveals) |
| `9f451a4` | test: focus-scope fixture and expectations follow VIEW-SCOPE-02/03 and VIEW-REVEAL-03 |
| `202a8d4` | impl: ensure_visible, scoped traversal, focus_into, the ScrollView's reveal, FocusScope, WidgetTester::ensure_visible; Phase 3 ticked |
| `3faa2c5` | docs: HANDOFF.md for focus scopes; Phase 3 done |
| `80b650f` | test: WIDGETS-TEXTFIELD-05 presses the platform's primary modifier (fixes CI on macOS) |
| `e81504e` | docs: HANDOFF.md, the macOS CI fix |
| `94c2b74` | chore: rust-analyzer builds in its own target directory (`rust-analyzer.toml`) |
| `be5605a` | docs: draft DataGrid part 1 specs for review; PLAN.md DataGrid item split in seven |
| `557c9fc` | spec: DataGrid part 1: grid body layout (LAYOUT-GRID-01..04) and the widget (WIDGETS-GRID-01..08, GALLERY-10); roles Grid, ColumnHeader and Cell |
| `4b056e1` | test: DataGrid part 1, stubs + 13 failing tests |
| `0524bce` | test: grid tests pump a second frame after construction |
| `b0270d5` | test: GALLERY-10 lets the wheel animation finish |
| `3103820` | test: LAYOUT-GRID-03 checks the moved cell's offset |
| `c5c7b25` | test: WIDGETS-GRID-03 and -04 read cells that exist |
| `903f3cf` | test: WIDGETS-GRID-03's builder asks for one unwrapped line |
| `b73b950` | test: WIDGETS-GRID-03 checks the clip, not the paragraph's constrained rect |
| `074ff46` | test: the GridScroll step dispatches the wheel at a cached position (the tester's lookup hid the frame's cost) |
| `ae4575f` | impl: DataGrid part 1: RenderGridBody, CellValue/DataSource/FnSource/GridColumn/DataGrid, roles, the gallery section, the benchmark at 1.30 ms per step |
| `935cf25` | chore: drop rust-analyzer from the pinned toolchain's components |
| `7db8aac` | docs: HANDOFF.md for DataGrid part 1; rust-analyzer removed |
| `90eea5c` | spec: DataGrid part 2, frozen columns (WIDGETS-GRID-09; GALLERY-10 amended) |
| `875ffe4` | test: DataGrid frozen columns and the gallery's frozen Id column, 2 failing tests |
| `2d9d899` | test: WIDGETS-GRID-09 counts painted frozen rows; GALLERY-10 checks Name moved left |
| `9b9bb52` | impl: DataGrid frozen columns; the gallery freezes its Id column; PLAN.md part 2 ticked |
| `cb28bed` | docs: HANDOFF.md for DataGrid part 2 |
| `0da4197` | docs: draft DataGrid part 3 (sorting and filtering) for review |
| `99bfded` | docs: the part 3 draft as its own Draft spec (the coverage check reads a file's status) |
| `634165e` | spec: DataGrid part 3, sorting and filtering (WIDGETS-GRID-10..12, GALLERY-11) |
| `fdd69aa` | test: DataGrid sorting and filtering, stubs + 4 failing tests |
| `bec8fdb` | spec: GALLERY-11's example taps the City header |
| `6d40e59` | test: WIDGETS-GRID-11 names its columns Key and Value; GALLERY-11 taps City |
| `82cb7e3` | impl: DataGrid sorting and filtering: sort state and header presses, the chevron, client_sort over a row map, RowFilter, source rows for builders; the gallery's sortable headers; PLAN.md part 3 ticked |
| `f54a5cf` | docs: HANDOFF.md for DataGrid part 3 |
| `8ac3175` | docs: draft kitchen-sink spec for review; PLAN.md item |
| `d14fd15` | spec: kitchen sink (GALLERY-12..14) |
| `f107208` | spec: GALLERY-14's end golden is reached with the End key |
| `616a2fa` | test: kitchen sink, the SINK_CARDS inventory + 3 failing tests |
| `4bdfda5` | spec: GALLERY-13 focuses the focus card's button with WidgetTester::focus |
| `eadfdfa` | test: GALLERY-13 focuses the focus card's button with the tester |
| `aa98d04` | spec: kitchen-sink card keys never equal a sample key |
| `f971964` | test: kitchen-sink inventory key sliders |
| `b647d47` | impl: the gallery's Kitchen sink section: twelve cards, SINK_CARDS, two goldens; PLAN.md ticked |
| `37be05d` | test: GALLERY-14 taps the typography card |
| `3cab8f5` | test: GALLERY-14 checks the page reached its end |
| `f44b178` | test: GALLERY-14 taps the Toggles title near the top; end golden regenerated |
| `495bf94` | docs: HANDOFF.md for the kitchen sink |
| `452f0f7` | docs: draft DataGrid part 4 (column resize and reorder) for review |
| `066d5e6` | spec: DataGrid part 4, column resize and reorder (WIDGETS-GRID-13..15, GALLERY-15); WIDGETS-GRID-10 sorts on the click |
| `56286c9` | spec: WIDGETS-GRID-13's handle lies inside the column it resizes; GALLERY-15 drags City 200 px |
| `9c0a401` | spec: WIDGETS-GRID-14's held header uses surface_container_highest |
| `93e7eb3` | test: DataGrid column resize and reorder, stubs + 4 failing tests |
| `2966a07` | test: WIDGETS-GRID-14's main grid gets a 600 px window |
| `4ef964f` | impl: DataGrid column resize and reorder; the gallery's echoes; PLAN.md part 4 ticked |
| `8c9c477` | docs: HANDOFF.md for DataGrid part 4 (decision 74) |
| `579e046` | docs: draft DataGrid part 5 selection and keyboard navigation spec for review; PLAN.md link and HANDOFF.md resume point |
| `56456fa` | spec: DataGrid selection, navigation, table semantics and pointer modifiers |
| `e81f109` | test: selection/input/a11y/gallery tests with compiling stubs; the missing behaviors fail |
| `41c3cd3` | spec: selection paint uses existing surface and primary tokens |
| `bcfdfd0` | test: selection tint is absent from the scene |
| `8559080` | spec: buffered-cell reveal accepts descendants |
| `d6e998d` | test: mapped sorting/filtering, controller disposal, header sorting and failing descendant reveal |
| `583b5ed` | spec: selection controller Default signature |
| `8de5d10` | impl: DataGrid part 5 selection/navigation/reveal, pointer modifiers and table semantics; gallery controls and goldens; checks pass, PLAN.md ticked and HANDOFF.md current |
| `1a833eb` | spec: DataGrid part 6 cell editors agreed; PLAN.md spec link and HANDOFF.md resume point |
| `cfcc8aa` | test: cell-editor API stubs, 12 failing widget tests and failing GALLERY-17; missing editors/default setter/status UI confirmed |
| `a3205fb` | spec: BuildCx::focus integration amendment, VIEW-FOCUS-10 (review); required for agreed editor entry/restoration |
| `d080eff` | test: BuildCx::focus stub fails the registered focusable target assertion; invalid-target and handler checks compile |
| `ca36dc8` | test: editor blur fixture explicitly moves focus (Button clicks do not request focus); Tab and pointer entry remain separately exercised |
| `d0f1332` | test: inclusive 500 ms double-click boundary and unchanged-value builder revision fail; additional frozen/nonfinite/stale-submit regressions |
| `aa2574f` | impl: typed cell editors, BuildCx::focus, local refresh, cancellation and errors; sparse editable gallery and goldens; all checks pass, PLAN.md ticked and HANDOFF.md current |
| `2649063` | spec: DataGrid part 7 stress example agreed; kitchen-sink GALLERY-18 composition amendment, PLAN.md and HANDOFF.md |
| `46b7c41` | test: seven stress tests and two kitchen-sink GALLERY-18 tests with compiling stubs; missing source/workload/cadence/status behavior confirmed |
| `2900d3c` | test: measure header/body cell centers correctly (text is centered in 36/32 px cells) |
| `9d78f30` | impl: bounded stress app/cadence stats, kitchen-sink grid refresh, goldens, README and architecture diagram; all checks pass, PLAN.md ticked and HANDOFF.md current |
| `3c7da2f` | docs: record the user's local-only performance testing preference; GitHub CI has no benchmark or native stress-app run |
| `714e70d` | spec: overlay portal inheritance agreed, ADR 0014, tooltip target-theme and GALLERY-19 amendments; architecture and PLAN.md current |
| `2592da3` | test: VIEW-OVERLAY-08..10, tooltip target-theme and GALLERY-19; all five new tests compile and fail for missing context/demo behavior |
| _this commit_ | spec: place the kitchen-sink popup beside compact controls so feedback stays readable |

A commit can't contain its own hash, so the newest row says _this commit_ (or _uncommitted_ for work not yet committed). The next update replaces
that with the real hash from `git log`.

## Next steps

Inspect CI for the authorized `main` push (decision 14).
Next feature: overlay-inheritance amendment before menus/dialogs; otherwise PLAN.md order.
Run benchmarks and the native 60 fps protocol **locally only**, on recorded mid-range hardware
when a desktop/profiler is available. Do not add performance stress runs to GitHub Actions.

## Open questions

- **DataGrid native 60 fps remains unverified.** Part 7 implements repeatable scrolling
  and redraw-tick cadence statistics, with GPU/compositor delivery checked on recorded hardware.
  Headless CPU timing is separate evidence. The example is complete; native presentation remains a manual check.
- **Accessibility is unchecked with a real screen reader.** The tree and the adapter are
  tested through `FakePlatform` and the opt-in window test only. The user should run the
  gallery under Orca (Linux), Narrator (Windows) and VoiceOver (macOS) and note what each
  reads and what it misses; text-editing semantics (caret and selection as AccessKit text
  ranges) are a known gap (ADR 0013).
- **`WidgetTester` finders walk every element on every call** (found with the grid
  benchmark): `find`/`find_all` evaluate the finder against each element, and `Finder::key`
  took about 30 ms on a tree of about 1 500 elements, which dwarfs a 1.4 ms frame. Harmless
  for tests, misleading for benchmarks. Proposal: a key index on `ViewTree` (`find_key` is
  already a lookup) and `Finder::key` using it, as a `test/widget-tester.md` amendment.
- **Nested focus scopes trap Tab** (decision 69, VIEW-SCOPE-02): Tab enters a nested
  `FocusScope` from its parent scope and then cycles inside it; leaving needs `focus_into`
  or `focus` from the app (Flutter behaves the same, and dialogs want exactly that). If a
  grouping that Tab can leave is wanted, add Flutter's `FocusTraversalGroup` as a separate
  concept rather than changing scopes.
- **`FocusScope` has no gallery entry or golden** (decision 69): it paints nothing. Decide
  whether AGENTS.md's "golden and gallery entry" rule should say "visible widgets".
- **Overlays don't inherit from the app.** The overlay layer is the root's last child, so
  content shown through `OverlayPortal` (tooltips, later menus and dialogs) sees no value
  provided inside the app, including the theme; the tooltip captures the theme at the portal.
  Proposal: make overlay content inherit from the portal's element (build it with the portal
  as its inherited-value parent), as a `view/overlay.md` amendment before menus.
- **Reactive text presets:** a theme's `text` is read at build time; changing font sizes live
  would need paragraphs re-keyed (decision 62). Revisit with the theme's second user (Fluent)
  or an accessibility font-scale setting.
- **Clipboard for `TextField`:** the `Platform` trait has no clipboard yet (PLAN.md puts it
  in Phase 4), so `TextField` has no copy/paste. Proposal: add `Platform::clipboard_text` /
  `set_clipboard_text` as a small platform spec amendment, before Phase 4 if the user wants
  copy/paste sooner.
- **Handler ownership:** pointer, key and focus handlers live on the element they are
  registered on. A component that registers on another element (e.g. its parent, or the root)
  leaves its handlers there after it is removed, and they may read its disposed signals (a
  panic per ADR 0007). Proposal: tie each registration to the registering scope (remove it on
  that scope's cleanup). Found while testing focus; not fixed yet.
- **Autopilot decisions to review:** decisions 20/21 and ADR 0007 (reactive shape, disposed
  handles panic), 26 (render-soft), 28 (platform), 29 (wgpu), 33 (flex/stack/wrap), 35 and
  ADR 0011 (view layer), 36 (text system, the `tantu-text` → `layout` edge, MSRV 1.88), ADR
  0008 (renderer conformance), 49 to 69 (the Phase 3 widgets, the theme, animation,
  lists, accessibility and focus scopes), 70 to 72 (DataGrid parts 1 to 3), 73 (the
  kitchen sink) 74 (column resize and reorder) and 75 (selection and keyboard navigation); decision 76's BuildCx::focus and decision 77's kitchen-sink composition amendments are also marked for review.
- **Draw-command validity is checked in three places** (`RenderReport::for_scene` and private
  copies in the soft and wgpu renderers). A public helper in `tantu-scene` (e.g.
  `Command::is_drawable`) would remove the copies; it is an API addition, so it needs a spec
  change.
- **Shadows with a radius over half a side** still use the quadrant approximation in the wgpu
  blur shader (Evan Wallace's closed form supports one radius per quadrant). Not covered by a
  reference Scene yet; check when shadows get a closer look.
- **A debug build printed an ICU4X "No segmentation model" message** to stderr for Japanese
  text (decision 43); find where it comes from with Phase 4 i18n.
- **No resize cursor on the grid's column handles** (decision 74): `CursorIcon` has
  `Default`, `Pointer`, `Text`, `Grab`, `Grabbing` and `NotAllowed` only. Proposal: add
  `ColResize`/`RowResize` (winit has them) as a `view/events.md` + `platform-winit` amendment
  when the split view needs them too.
- **`.agents/` is untracked** in the working tree (a `skills` folder, not project code); decide
  whether to gitignore or commit it.
- Is `tantu` (and `tantu-*`) available on crates.io? Check, and consider reserving it, before the
  first publish. Same for a domain / GitHub org name if wanted.
