# PLAN.md — dkui roadmap

## Vision

A Rust desktop UI framework with Flutter-style authoring and a renderer-agnostic core. It should be
able to ship a 200-screen enterprise app (grids, docking, forms, a11y, i18n) and also a small
utility app that starts fast and is a single small binary.

## What we take from Knots, and what we change

| Knots (Zig) | dkui (Rust) | Why |
|---|---|---|
| UI engine decoupled from window + GPU; emits `render.Packet` | Keep: UI emits a data-only `Scene`; `Renderer` trait consumes it | Core idea: same UI logic, many targets |
| Immediate mode (rebuild every frame) | Retained element tree + fine-grained signals | Stable identity for focus/IME/a11y/animation; incremental work for huge UIs |
| Simple text (no shaping, bidi, IME, fallback) | Full text stack (parley/swash): shaping, bidi, fallback, IME from day one of text work | Enterprise and i18n hard requirement |
| Wayland only on Linux | winit: Wayland **and** X11, Windows, macOS | Enterprise Linux desktops still run X11 |
| Vulkan/WebGPU only | wgpu (Vulkan/Metal/DX12/GL) + CPU renderer (tiny-skia) | Runs on VMs, RDP/Citrix, old GPUs, CI |
| AccessKit (native only) | AccessKit as a first-class part of the pipeline | Accessibility compliance (Section 508 / EN 301 549) |
| HMR via WASM modules | Later phase: hot-reload of view code + live theme reload | Nice to have, not foundational |

## Key design decisions (initial ADRs to write)

1. **Retained + reactive.** View → Element → RenderObject, driven by signals. (ADR-0001)
2. **Flutter layout protocol.** `BoxConstraints` down, `Size` up, parent sets offsets; single-pass
   with intrinsic-size queries as an explicit opt-in; relayout boundaries. (ADR-0002)
3. **Scene as the renderer contract.** Flat, versioned, serializable command list with layers,
   clips, transforms, glyph runs, images, and a damage region. (ADR-0003)
4. **Platform trait.** Windowing, input, IME, clipboard, DnD, menus, dialogs, tray are all behind
   `dkui-platform`; winit is the default implementation. (ADR-0004)
5. **Text stack = parley + swash + fontique.** (ADR-0005)
6. **Default GPU backend = wgpu; evaluate vello** for path-heavy content once it is stable enough. (ADR-0006)

## Phases

### Phase 0 — Foundations (weeks 1–2)
- [ ] Cargo workspace, `rust-toolchain.toml`, CI (fmt, clippy, test on Linux/Windows/macOS)
- [ ] `docs/adr/` with ADRs 0001–0006
- [ ] `docs/specs/` with a spec template (purpose, API, numbered rules, perf, open questions)
- [ ] `dkui-core`: `Point/Size/Rect/Insets/Affine`, `Color`, `Id`, generational arena
- [ ] `dkui-reactive`: `Signal`, `Memo`, `Effect`, batch, scoped disposal, and tests for glitch-freedom
- [ ] Reactive micro-benchmarks

### Phase 1 — Pixels on screen (weeks 3–5)
- [ ] `dkui-scene`: command set, layers, clip stack, `Renderer` trait, image/font resource handles
- [ ] `dkui-render-headless` (recording) and `dkui-render-soft` (tiny-skia → PNG)
- [ ] `dkui-platform` trait + `dkui-platform-winit`: window, resize, DPI, pointer, keyboard
- [ ] `dkui-render-wgpu`: rects, rounded rects, borders, shadows, clips, images
- [ ] **Milestone:** a hand-built Scene renders identically in wgpu and software (golden diff)

### Phase 2 — Layout, views and text (weeks 6–10)
- [ ] `dkui-layout`: `BoxConstraints`, `Padding`, `Align`, `SizedBox`, `Flex` (Row/Column/Expanded/Flexible), `Stack`/`Positioned`
- [ ] `dkui-view`: View/Element/RenderObject traits, keyed reconciliation, dirty tracking, relayout boundaries
- [ ] `dkui-text`: shaping, line breaking, bidi, font fallback, glyph-run output into Scene
- [ ] Event dispatch: hit-testing, bubbling/capture, pointer capture, cursor icons
- [ ] `dkui` facade + `App` runner; `counter` example matching the snippet in AGENTS.md
- [ ] `dkui-test::WidgetTester` (pump, tap, type, find by key)
- [ ] **Milestone:** counter + layout demo run on Linux (Wayland + X11), Windows and macOS

### Phase 3 — Core widget set and interaction (weeks 11–16)
- [ ] Focus system, keyboard navigation, shortcuts/command registry
- [ ] Widgets: Text, RichText, Button, IconButton, Checkbox, Radio, Switch, Slider, TextField (with IME, selection, undo), Image, Icon, Divider, Tooltip
- [ ] Scroll: `ScrollView`, scrollbars, kinetic/wheel/trackpad handling
- [ ] Virtualized `ListView` (100k items at 60fps)
- [ ] `dkui-theme`: design tokens, light/dark, density, live theme switching
- [ ] Animation: tickers, tweens, curves, implicit animations (`AnimatedOpacity`-style)
- [ ] `dkui-a11y`: AccessKit tree for every standard widget
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
- [ ] Inspector / devtools window (element tree, layout bounds, signals graph)
- [ ] Optional `view!{}` macro DSL as sugar
- [ ] Packaging guide (MSI/MSIX, .app/.dmg + notarization, AppImage/Flatpak/deb)
- [ ] Docs site + book, API stabilisation, 0.1 release, **make repo public**

### Later / exploratory
- Web target (wgpu on WebGPU + canvas platform shell), which the Scene/Platform split makes possible
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
  rendering layer, and Knots' Packet design.
