# PLAN.md — Tantu roadmap

## Vision

A Rust desktop UI framework with Flutter-style authoring and a renderer-agnostic core. It should be
able to ship a 200-screen enterprise app (grids, docking, forms, a11y, i18n) and also a small
utility app that starts fast and is a single small binary.

## What we take from Knots, and what we change

| Knots (Zig) | Tantu (Rust) | Why |
|---|---|---|
| UI engine decoupled from window + GPU; emits `render.Packet` | Keep: UI emits a data-only `Scene`; `Renderer` trait consumes it | Core idea: same UI logic, many targets |
| Immediate mode (rebuild every frame) | Retained element tree + fine-grained signals | Stable identity for focus/IME/a11y/animation; incremental work for huge UIs |
| Simple text (no shaping, bidi, IME, fallback) | Full text stack (parley/swash): shaping, bidi, fallback, IME from day one of text work | Enterprise and i18n hard requirement |
| Wayland only on Linux | winit: Wayland **and** X11, Windows, macOS | Enterprise Linux desktops still run X11 |
| Vulkan/WebGPU only | wgpu (Vulkan/Metal/DX12/GL) + CPU renderer (tiny-skia) | Runs on VMs, RDP/Citrix, old GPUs, CI |
| AccessKit (native only) | AccessKit as a first-class part of the pipeline | Accessibility compliance (Section 508 / EN 301 549) |
| HMR via WASM modules | Later phase: hot-reload of view code + live theme reload | Nice to have, not foundational |

## What we take from Clay (nicbarker/clay), and what we change

[Clay](https://github.com/nicbarker/clay) is a small C layout library: renderer-agnostic,
arena-allocated, with layout times measured in microseconds. **The user-facing structure stays
Flutter's.** Flutter's widget composition and constraint protocol can express any layout, and that
is the API we commit to. From Clay we take only implementation techniques that make that structure
faster or more portable without changing how apps are written.

| Clay (C) | Tantu (Rust) | Why |
|---|---|---|
| Text measured through a callback, with a word-level measure cache | Text layout goes through a paragraph-level `TextMeasure` trait with a cache keyed by (text, style, width) and a one-line shortcut; the word-level cache lives inside `tantu-text`, where shaping happens (ADR 0010) | Keeps `tantu-layout` independent of `tantu-text`; big win for grids and lists; line breaking stays with the shaper, so bidi and CJK stay correct |
| Render commands carry element `id`, `z_index`, bounding box; `Scissor`, `OverlayColor`, `Custom` | `Scene` commands carry element id + z-index; add overlay-color and custom (user-drawn) commands | Stable ids let retained backends diff commands; custom commands are the escape hatch for charts / 3D viewports |
| Floating elements: 9-point attach anchors, offset, z-index, pointer capture or passthrough | Used as the anchoring options of Flutter-style `Overlay` + anchored positioning (`CompositedTransformTarget`/`Follower` equivalent) | Tooltips, menus, drop-downs and popovers share one positioning model |
| Visibility culling on by default | Cull off-screen render objects before emitting `Scene` | Cheap win for large scroll areas |
| Declarative enter/exit transitions keyed by stable id | Informs our implicit animations and `AnimatedSwitcher`/`AnimatedList`-style widgets | Common list/dialog animations without hand-written tickers |
| Debug inspector drawn as extra render commands | Devtools overlay emits ordinary `Scene` commands, so it works with every renderer | No per-backend debug tooling |
| Static arena, no malloc per frame, explicit context (multiple instances) | Arena-backed element/render storage, no per-frame allocation, no hidden global state (one context per app/window) | Predictable performance; testable; multi-window |
| `Fit`/`Grow`/`Fixed`/`Percent` sizing on every container | **Not adopted.** We use Flutter's `Expanded`, `Flexible`, `SizedBox`, `FractionallySizedBox`, `ConstrainedBox`, etc. | One layout model, Flutter's, which is proven for any kind of app |
| Immediate mode, C macros, no a11y / IME / text shaping | **Not adopted.** Retained + reactive tree, plain Rust builders, AccessKit, parley | Same reasons as for Knots above |

## Key design decisions (initial ADRs to write)

1. **Retained + reactive.** View → Element → RenderObject, driven by signals. (ADR-0001)
2. **Flutter structure and layout protocol.** Apps are built from composable widgets the way
   Flutter apps are: layout widgets (`Row`, `Column`, `Expanded`, `Flexible`, `Padding`, `Align`,
   `SizedBox`, `Stack`, …) wrap other widgets. `BoxConstraints` go down, `Size` comes up and the
   parent sets offsets, with relayout boundaries and intrinsic-size queries as an explicit opt-in.
   Custom layouts implement the same protocol (`RenderBox`-style), so any layout is possible. (ADR-0002)
3. **Scene as the renderer contract.** Flat, versioned, serializable command list with layers,
   clips, transforms, glyph runs, images, custom commands and a damage region. Every command
   carries its element id and z-index, and off-screen content is culled. (ADR-0003)
4. **Platform trait.** Windowing, input, IME, clipboard, DnD, menus, dialogs, tray are all behind
   `tantu-platform`; winit is the default implementation. (ADR-0004)
5. **Text stack = parley + swash + fontique.** (ADR-0005)
6. **Default GPU backend = wgpu; evaluate vello** for path-heavy content once it is stable enough. (ADR-0006)

## Phases

### Phase 0 — Foundations (weeks 1–2) — done 2026-09-30
- [x] Cargo workspace, `rust-toolchain.toml`, CI (fmt, clippy, test on Linux/Windows/macOS)
- [x] `docs/adr/` with ADRs 0001–0006
- [x] `docs/specs/` with a spec template (purpose, API, numbered rules, perf, open questions)
- [x] Spec-coverage check in CI (`cargo xtask spec-coverage`): every rule of an agreed spec has a test
- [x] `tantu-core`:
  - [x] geometry: `Point`, `Vec2`, `Size`, `Rect`, `EdgeInsets`, `Affine` (spec `docs/specs/core/geometry.md`)
  - [x] `Color` (spec `docs/specs/core/color.md`)
  - [x] `Id` and generational arena (spec `docs/specs/core/arena.md`)
- [x] `tantu-reactive`: `Signal`, `Memo`, `Effect`, batch, scoped disposal, and tests for glitch-freedom
- [x] Reactive micro-benchmarks (spec `docs/specs/reactive/benchmarks.md`)

### Phase 1 — Pixels on screen (weeks 3–5) — done 2026-10-10
- [x] `tantu-scene`:
  - [x] `Scene` and its builder: command set (with element id + z-index), clip/transform/layer scopes, overlay color, glyph runs, custom commands, resource handle types, damage, culling query (spec `docs/specs/scene/scene.md`)
  - [x] `Renderer` trait, image/font resource registry, custom-command handlers (spec `docs/specs/scene/renderer.md`)
- [x] `tantu-render-headless` and `tantu-render-soft`:
  - [x] Shared `RenderReport` counting in `tantu-scene` (amends spec `docs/specs/scene/renderer.md`)
  - [x] `tantu-render-headless`: recording renderer for tests (spec `docs/specs/render-headless/recorder.md`)
  - [x] `tantu-render-soft`: tiny-skia renderer, PNG output, golden-image helpers (spec `docs/specs/render-soft/renderer.md`)
- [x] `tantu-platform` trait + `tantu-platform-winit`: window, resize, DPI, pointer, keyboard
  - [x] `tantu-platform`: `Platform` trait, window/input event types, `FakePlatform` for tests (spec `docs/specs/platform/platform.md`)
  - [x] `tantu-platform-winit`: winit 0.30 shell, event conversion, window handles for renderers (spec `docs/specs/platform-winit/shell.md`)
- [x] `tantu-render-wgpu`: rects, rounded rects, borders, shadows, clips, images (spec `docs/specs/render-wgpu/renderer.md`)
- [x] `examples/scene-window`: Phase 1 demo, a hand-built Scene drawn in a winit window with wgpu (spec `docs/specs/examples/scene-window.md`; depends on the crates directly until the `tantu` facade exists in Phase 2)
- [x] **Milestone:** a hand-built Scene renders the same in wgpu and software (golden diff within the cross-backend tolerance); reference Scenes and goldens in a renderer conformance crate every backend is tested against (spec `docs/specs/render-conformance/conformance.md`, ADR 0008)

### Phase 2 — Layout, views and text (weeks 6–10) — done 2026-10-10
- [x] `tantu-layout`: `BoxConstraints` protocol, `RenderBox`-style trait for custom layouts; `Padding`, `Align`/`Center`, `SizedBox`, `ConstrainedBox`, `FractionallySizedBox`, `AspectRatio`, Flex (`Row`/`Column` with `Expanded`/`Flexible`/`Spacer`, main/cross-axis alignment, `spacing`), `Stack`/`Positioned`, `Wrap` (the layout tree lives in `tantu-layout`, ADR 0009)
  - [x] `BoxConstraints` (spec `docs/specs/layout/constraints.md`)
  - [x] Layout tree and protocol: `RenderBox` trait, `LayoutTree` arena, parent data, layout pass with caching, `mark_needs_layout`, relayout boundaries, opt-in intrinsic sizes (spec `docs/specs/layout/tree.md`)
  - [x] Single-child layouts: `Alignment`, `RenderPadding`, `RenderPositionedBox` (`Align`/`Center`), `RenderConstrainedBox` (`SizedBox`/`ConstrainedBox`), `RenderFractionallySizedBox`, `RenderAspectRatio` (spec `docs/specs/layout/single-child.md`)
  - [x] Flex: `RenderFlex` (`Row`/`Column`), flex parent data (`Expanded`/`Flexible`/`Spacer`), main/cross-axis alignment and size, `spacing`, overflow reporting (spec `docs/specs/layout/flex.md`)
  - [x] `RenderStack` with `Positioned` parent data (spec `docs/specs/layout/stack.md`)
  - [x] `RenderWrap` (spec `docs/specs/layout/wrap.md`)
- [x] `TextMeasure`, `MeasureCache` and `RenderParagraph`; the measurer passed into the layout pass (spec `docs/specs/layout/text.md`, ADR 0010)
- [x] Layout benchmark: 10k render objects, target < 1 ms full layout (635 µs; spec `docs/specs/layout/benchmarks.md`)
- [x] `tantu-view`: View/Element/RenderObject traits, keyed reconciliation, dirty tracking, relayout boundaries; `LayoutBuilder` (it builds views during layout, so it needs the element tree); visibility culling while painting (technique from Clay; `Scene::is_culled` exists). Design: ADR 0011
  - [x] View tree: `View`/`AnyView`, `BuildCx`, `ViewTree` (runtime, element arena, layout tree), render and region elements, element scopes, removal (spec `docs/specs/view/tree.md`)
  - [x] Paint: `Paint` trait, `PaintCx`, traversal with offsets and element ids, culling (spec `docs/specs/view/paint.md`)
  - [x] Reactive props and frames: `Prop<T>`, the update queue, `ViewTree::frame` (apply, layout, paint), needs-frame notification (spec `docs/specs/view/frame.md`)
  - [x] Dynamic content: `Dyn`, `Show`, `For` with keyed reconciliation (spec `docs/specs/view/dynamic.md`)
  - [x] `LayoutBuilder` (spec `docs/specs/view/layout-builder.md`)
- [x] `tantu-text`: shaping, line breaking, bidi, font fallback, glyph-run output into Scene
  - [x] Text system: fonts (registered and system, via fontique), styles as `TextStyleKey`s, `TextMeasure` over parley, glyph runs into a Scene (spec `docs/specs/text/system.md`)
  - [x] Text in views: frames carry a text context that measures and paints; the paragraph's paint (spec `docs/specs/view/text.md`)
  - [x] Bidi and font fallback: rules for right-to-left and mixed paragraphs; per-character fallback through every registered family (spec `docs/specs/text/scripts.md`)
- [x] Glyph rasterization: `tantu-render-soft` (swash) and `tantu-render-wgpu` (glyph atlas); text reference Scenes in `tantu-render-conformance`
  - [x] Glyph rasterizer: swash coverage masks with quarter-pixel subpixel positioning, cached, shared by both renderers (spec `docs/specs/text/raster.md`)
  - [x] Glyph runs in `tantu-render-soft` (amends `docs/specs/render-soft/renderer.md`)
  - [x] Glyph runs in `tantu-render-wgpu` with a glyph atlas (amends `docs/specs/render-wgpu/renderer.md`)
  - [x] A text reference Scene and golden in `tantu-render-conformance` (RENDER-CONF-14)
- [x] Event dispatch: hit-testing, bubbling/capture, pointer capture, cursor icons (spec `docs/specs/view/events.md`)
- [x] `tantu` facade + `App` runner; `counter` example matching the snippet in AGENTS.md
  - [x] Shared text styles: one style table for the text system and the view tree, `BuildCx::text_style`, style presets (spec `docs/specs/text/styles.md`)
  - [x] Layout widgets: `Padding`, `Center`, `Align`, `SizedBox`, `ConstrainedBox`, `Row`, `Column`, `Expanded`, `Flexible`, `Spacer`, `Stack`, `Positioned` (spec `docs/specs/widgets/layout.md`)
  - [x] `Text` and `Button` (fixed Phase 2 look until `tantu-theme`) (spec `docs/specs/widgets/basic.md`)
  - [x] `tantu` facade: `App`, `Window`, the runner (platform + view tree + renderer), `Error`, `prelude` (spec `docs/specs/facade/app.md`)
  - [x] `examples/counter`: the AGENTS.md snippet, verbatim (spec `docs/specs/examples/counter.md`)
- [x] `tantu-test::WidgetTester` (pump, tap, find by key/text/type, goldens; typing text moves to Phase 3 with focus and `TextField`) (spec `docs/specs/test/widget-tester.md`)
- [x] **Milestone:** counter + layout demo run on Linux (Wayland + X11), Windows and macOS (Linux checked by the agent, Windows and macOS by the user, 2026-10-10; `examples/counter`, `examples/layout-demo`, spec `docs/specs/examples/layout-demo.md`)

### Phase 3 — Core widget set and interaction (weeks 11–16)
- [x] Focus system, keyboard navigation, shortcuts/command registry
  - [x] Keyboard events and focus: focusable elements, key dispatch to the focused path, Tab traversal, focus changes, the runner forwarding keys, `WidgetTester::press_key` (spec `docs/specs/view/focus.md`)
  - [x] Shortcuts and commands: Flutter's `Shortcuts`/`Actions`/`Intent` (spec `docs/specs/view/shortcuts.md`)
  - [x] `Button`: Space/Enter activation when focused, focus indicator (spec `docs/specs/widgets/button-keyboard.md`)
- [ ] `Overlay` + anchored positioning (target/follower): 9-point anchors, offset, z-index, pointer passthrough, flip/clamp to window (spec `docs/specs/view/overlay.md`)
- [ ] Widgets: Text, RichText, Button, IconButton, Checkbox, Radio, Switch, Slider, TextField (with IME, selection, undo), Image, Icon, Divider, Tooltip
- [ ] Scroll: `ScrollView`, scrollbars, kinetic/wheel/trackpad handling
- [ ] Virtualized `ListView` (100k items at 60fps)
- [ ] `tantu-theme`: design tokens, light/dark, density, live theme switching
- [ ] Animation: tickers, tweens, curves, implicit property transitions keyed by stable id, enter/exit transitions
- [ ] `tantu-a11y`: AccessKit tree for every standard widget
- [ ] `examples/gallery`

### Phase 4 — Enterprise features (weeks 17–26)
- [ ] `DataGrid`: virtualized rows **and** columns, frozen columns, sort/filter, resize/reorder, cell editors, selection models, 1M-row data source trait
- [ ] `TreeView`, `TabView`, `SplitView`, docking layout manager (drag panels, persist layout)
- [ ] Menus (native + custom), context menus, toolbars, status bar
- [ ] Dialogs, native file dialogs, notifications, system tray
- [ ] Multi-window, drag & drop (intra-app + OS), clipboard (text, rich, images)
- [ ] Forms: validation, field binding, masked input, date/time pickers, combo box with search
- [ ] i18n: Fluent-based localisation, RTL layout mirroring, locale-aware formatting
- [ ] `Grid` layout (possibly via taffy) for dashboard-style screens
- [ ] **Milestone:** `examples/enterprise-dashboard` (docking + grid + forms + charts area)

### Phase 5 — Polish, performance and tooling (weeks 27+)
- [ ] Damage tracking and partial repaint; layer caching; GPU texture atlas tuning
- [ ] Profiling overlay (frame time, rebuild/relayout counts); `tracing` spans throughout
- [ ] Hot reload (theme and assets first, then view code via dylib or WASM host, the Knots-style approach)
- [ ] Inspector / devtools (element tree, layout bounds, signals graph), drawn as ordinary Scene commands so it works on every renderer
- [ ] Optional `view!{}` macro DSL as sugar
- [ ] Packaging guide (MSI/MSIX, .app/.dmg + notarization, AppImage/Flatpak/deb)
- [ ] Docs site + book, API stabilisation, 0.1 release

### Later / exploratory
- Web target (wgpu on WebGPU + canvas platform shell), which the Scene/Platform split makes possible;
  a DOM renderer that diffs Scene commands by element id, like Clay's HTML renderer
- Remote/thin-client rendering by streaming `Scene` over the network
- Mobile shells (Android/iOS) through the same Platform trait
- Vello backend for vector-heavy apps; charts library on top of Scene paths

## Success criteria for 0.1

- Same app source runs on Windows, macOS, Linux (Wayland + X11) with GPU and software renderers
- 60fps scrolling a 1M-row DataGrid on mid-range hardware; cold start < 300 ms
- Screen readers (Narrator, VoiceOver, Orca) can operate the gallery
- Full IME (CJK) and RTL text in TextField
- Every widget has tests + goldens; CI green on all three OSes

## Risks

- **Scope.** A full UI toolkit takes years. Mitigate by staying phase-disciplined and building on
  proven crates (winit, wgpu, parley, accesskit) rather than rewriting them.
- **Text and IME** are where most toolkits fall short. Start them early in Phase 2, not at the end.
- **Reactivity + retained tree** ownership in Rust (no GC). Keep elements in an arena with ids,
  and have closures capture `Copy` signal handles, as Leptos/Floem/Xilem do.
- **Prior art to study:** Xilem/Masonry, Floem, Iced, Slint, egui, GPUI (Zed), Flutter's
  rendering layer, Knots' Packet design, and Clay's render commands and measure caching.
