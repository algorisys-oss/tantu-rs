# AGENTS.md — dkui-rs

Guidance for any coding agent (Claude Code, Codex, Cursor, etc.) working in this repository.
Tool-specific notes live in `CLAUDE.md`; the roadmap lives in `PLAN.md`.

## What this project is

**dkui** is a cross-platform desktop UI framework in Rust for building both large enterprise
applications (data grids, docking layouts, forms, thousands of widgets, accessibility, i18n) and
general-purpose apps. It is inspired by [Knots](https://github.com/knots-ui/knots) (Zig) and Flutter.

Two non-negotiable design goals:

1. **Renderer independence.** UI, layout and widget logic never touch a graphics API. They emit a
   backend-neutral `Scene` (display list). Any renderer (wgpu, software, headless/test, future
   web/canvas) consumes that `Scene`.
2. **Flutter-like authoring.** Declarative, composable widget trees built in plain Rust, with
   Flutter's layout protocol (constraints go down, sizes go up, parent positions children).

## Architecture (the pipeline)

```
 App code (widgets, signals)
        │  build
        ▼
 View tree  (cheap, declarative descriptions — like Flutter Widgets)
        │  reconcile
        ▼
 Element tree  (retained, identity + state, lives in an arena)
        │  layout (BoxConstraints ↓, Size ↑)   paint           a11y
        ▼
 Render tree ──────────────► Scene (display list) ──► Renderer trait ──► wgpu / tiny-skia / headless
        ▲                                     └────► AccessKit tree
        │ events (hit-test, focus, IME)
 Platform trait ◄── winit (default) / other shells
```

- **Retained, not immediate-mode.** Unlike Knots, we keep a retained element tree. Enterprise UIs
  need stable identity for focus, IME, accessibility, animations, scroll position and incremental
  re-layout. Rebuilding everything every frame does not scale to 50k-row grids.
- **Fine-grained reactivity.** State lives in signals (`Signal<T>`, `Memo<T>`, `Effect`). A signal
  change marks only its dependent elements dirty → rebuild/relayout/repaint just those subtrees.
  There is no whole-subtree `setState` rebuild by default.
- **Scene is data, not calls.** `Scene` is a flat, serializable list of commands (rects, rounded
  rects, paths, glyph runs, images, clips, transforms, layers). Renderers must not call back into
  the UI. This mirrors Knots' `render.Packet` and makes golden tests and remote rendering trivial.

## Workspace layout (target)

| Crate | Responsibility | May depend on |
|---|---|---|
| `dkui-core` | geometry, color, ids, arena, errors | — |
| `dkui-reactive` | signals, memos, effects, scheduler | core |
| `dkui-scene` | `Scene` display list, `Renderer` trait, resource handles | core |
| `dkui-text` | font loading, shaping, bidi, line breaking (parley/swash) | core, scene |
| `dkui-layout` | `BoxConstraints`, layout protocol, Flex/Stack/Grid algorithms | core |
| `dkui-view` | View/Element/RenderObject traits, reconciler, event dispatch, focus | core, reactive, scene, text, layout |
| `dkui-widgets` | standard widget set (Text, Button, TextField, ListView, DataGrid, …) | view |
| `dkui-theme` | design tokens, Material-ish + Fluent-ish default themes | view |
| `dkui-a11y` | AccessKit tree generation | view |
| `dkui-platform` | `Platform` trait: windows, input, clipboard, IME, dialogs, menus | core |
| `dkui-platform-winit` | winit implementation | platform |
| `dkui-render-wgpu` | GPU renderer (Vulkan/Metal/DX12/GL) | scene, text |
| `dkui-render-soft` | CPU renderer via tiny-skia (fallback, CI, remote desktop) | scene, text |
| `dkui-render-headless` | records Scenes, used by tests | scene |
| `dkui-test` | widget tester, golden images, event simulation | view, render-soft, render-headless |
| `dkui` | facade crate, `App` runner, prelude | everything above |
| `examples/` | gallery, todo, enterprise dashboard, data grid stress test | dkui |

**Dependency rule (enforced in review):** nothing below `dkui-view` may know about widgets; nothing
except `dkui-render-*` may depend on `wgpu`/`tiny-skia`; nothing except `dkui-platform-*` may depend
on `winit`. If you need to break this rule, stop and discuss.

## Authoring style we are aiming for

```rust
use dkui::prelude::*;

fn counter() -> impl View {
    let count = signal(0);
    Column::new()
        .spacing(8.0)
        .cross_align(CrossAxis::Center)
        .child(Text::new(move || format!("Count: {}", count.get())).style(TextStyle::title()))
        .child(Button::new("Increment").on_press(move || count.update(|c| *c += 1)))
}

fn main() -> dkui::Result<()> {
    App::new().window(Window::new("Counter").size(400.0, 300.0), counter).run()
}
```

- Builders take `self` by value and return `Self`; `.child()`/`.children()` for composition.
- Reactive props accept either a value or a closure (`impl IntoProp<T>`).
- No macros are required to build UI. A `view!{}` DSL may come later as sugar, never as the only way.

## Coding conventions

- Rust edition 2024, stable toolchain, MSRV pinned in `rust-toolchain.toml`.
- `#![forbid(unsafe_code)]` in every crate except `dkui-render-*`, `dkui-platform-*`, and `dkui-core`
  arena internals. Every `unsafe` block needs a `// SAFETY:` comment.
- No `unwrap()`/`expect()` in library code paths reachable by users; return `dkui::Error` or handle.
  `expect` is allowed for true invariants with a message explaining the invariant.
- Public API: every public item has a doc comment; crate roots have an overview with an example.
- Avoid allocation in the per-frame hot path (layout, paint, hit-test). Reuse buffers; use arenas
  and `SmallVec` where measured to help.
- Logical pixels (`f32`, DPI-independent) everywhere above the renderer; renderers convert to physical.
- Use `tracing` for diagnostics, never `println!` in library code.
- Keep feature flags additive. Default features: `wgpu`, `winit`, `default-theme`.

## Commands

```bash
cargo build --workspace
cargo test --workspace                     # unit + widget tests (headless, no GPU needed)
cargo test -p dkui-test -- --ignored       # golden image tests (software renderer)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo run -p gallery                       # widget gallery example
cargo bench -p dkui-layout                 # layout / reactive benchmarks
```

Before declaring a task done: `fmt`, `clippy -D warnings`, and `test` must pass for the crates you touched.

## Testing expectations

- Layout algorithms: pure unit tests on constraint → size/position results.
- Widgets: `dkui-test::WidgetTester` (pump frames, simulate pointer/keyboard, query by key/semantics).
- Rendering: golden PNGs via `dkui-render-soft`; update with `DKUI_UPDATE_GOLDENS=1`.
- Reactive system: tests for glitch-freedom, disposal, and no leaks (count live nodes).
- New widgets require: at least one behavior test, one golden, and a gallery entry.

## When working on a task

1. Read `PLAN.md` to find the current milestone; do not jump ahead to later-phase features.
2. Keep changes within one crate where possible; respect the dependency rule above.
3. Prefer small, reviewable commits. Update `PLAN.md` checkboxes when a milestone item lands.
4. If a design decision here seems wrong, raise it. Do not silently diverge. Record accepted
   decisions in `docs/adr/NNNN-title.md`.
