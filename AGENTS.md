# AGENTS.md — tantu-rs

Guidance for any coding agent (Claude Code, Codex, Cursor, etc.) working in this repository.
Tool-specific notes live in `CLAUDE.md`; the roadmap lives in `PLAN.md`.

## What this project is

**Tantu** (`tantu`) is a cross-platform desktop UI framework in Rust for building both large enterprise
applications (data grids, docking layouts, forms, thousands of widgets, accessibility, i18n) and
general-purpose apps. It is inspired by [Knots](https://github.com/knots-ui/knots) (Zig) and Flutter.

Two non-negotiable design goals:

1. **Renderer independence.** UI, layout and widget logic never touch a graphics API. They emit a
   backend-neutral `Scene` (display list). Any renderer (wgpu, software, headless/test, future
   web/canvas) consumes that `Scene`.
2. **Flutter-like authoring.** Declarative, composable widget trees built in plain Rust, with
   Flutter's layout protocol (constraints go down, sizes go up, parent positions children).

## Architecture (the pipeline)

Full write-up and diagram: [`docs/architecture.md`](docs/architecture.md).

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
  rects, paths, glyph runs, images, clips, transforms, layers, custom). Each command carries its
  element id and z-index, and off-screen elements are culled. Renderers must not call back into
  the UI. This mirrors Knots' `render.Packet` and Clay's render commands, and makes golden tests,
  retained backends (diff by id) and remote rendering straightforward.
- **Flutter structure is the API.** Layout is done by composing layout widgets (`Row`, `Column`,
  `Expanded`, `Flexible`, `Padding`, `Align`, `SizedBox`, `Stack`, `Positioned`, …) using Flutter's
  constraint protocol. Custom layouts implement the same `RenderBox`-style protocol. Don't introduce
  an alternative layout vocabulary alongside it. Text is measured through a `TextMeasure` trait,
  so layout never depends on the text crate.

## Workspace layout (target)

| Crate | Responsibility | May depend on |
|---|---|---|
| `tantu-core` | geometry, color, ids, arena, errors | — |
| `tantu-reactive` | signals, memos, effects, scheduler | core |
| `tantu-scene` | `Scene` display list, `Renderer` trait, resource handles | core |
| `tantu-text` | font loading, shaping, bidi, line breaking (parley/swash) | core, scene |
| `tantu-layout` | `BoxConstraints` protocol, Flex/Stack/Wrap/Grid/Overlay algorithms, `TextMeasure` trait | core |
| `tantu-view` | View/Element/RenderObject traits, reconciler, event dispatch, focus | core, reactive, scene, text, layout |
| `tantu-widgets` | standard widget set (Text, Button, TextField, ListView, DataGrid, …) | view |
| `tantu-theme` | design tokens, Material-ish + Fluent-ish default themes | view |
| `tantu-a11y` | AccessKit tree generation | view |
| `tantu-platform` | `Platform` trait: windows, input, clipboard, IME, dialogs, menus | core |
| `tantu-platform-winit` | winit implementation | platform |
| `tantu-render-wgpu` | GPU renderer (Vulkan/Metal/DX12/GL) | scene, text |
| `tantu-render-soft` | CPU renderer via tiny-skia (fallback, CI, remote desktop) | scene, text |
| `tantu-render-headless` | records Scenes, used by tests | scene |
| `tantu-test` | widget tester, golden images, event simulation | view, render-soft, render-headless |
| `tantu` | facade crate, `App` runner, prelude | everything above |
| `examples/` | gallery, todo, enterprise dashboard, data grid stress test | tantu |

**Dependency rule (enforced in review):** nothing below `tantu-view` may know about widgets; nothing
except `tantu-render-*` may depend on `wgpu`/`tiny-skia`; nothing except `tantu-platform-*` may depend
on `winit`. If you need to break this rule, stop and discuss.

## Authoring style we are aiming for

```rust
use tantu::prelude::*;

fn counter() -> impl View {
    let count = signal(0);
    Padding::all(16.0).child(
        Column::new()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(8.0)
            .child(Text::new(move || format!("Count: {}", count.get())).style(TextStyle::title()))
            .child(Button::new("Increment").on_press(move || count.update(|c| *c += 1))),
    )
}

fn main() -> tantu::Result<()> {
    App::new().window(Window::new("Counter").size(400.0, 300.0), counter).run()
}
```

- Builders take `self` by value and return `Self`; `.child()`/`.children()` for composition.
- Reactive props accept either a value or a closure (`impl IntoProp<T>`).
- Follow Flutter's names and semantics for layout widgets and their properties
  (`main_axis_alignment`, `cross_axis_alignment`, `main_axis_size`, `flex`, …), converted to
  snake_case. Someone who knows Flutter should be able to read Tantu code straight away.
- Overlays (tooltips, menus, popovers, dialogs) go through `Overlay` + anchored positioning.
- No macros are required to build UI. A `view!{}` DSL may come later as sugar, never as the only way.

## Coding conventions

- Rust edition 2024, stable toolchain, MSRV pinned in `rust-toolchain.toml`.
- License: `MIT OR Apache-2.0`. Every crate sets `license = "MIT OR Apache-2.0"` (inherit via `[workspace.package]`).
- `#![forbid(unsafe_code)]` in every crate except `tantu-render-*`, `tantu-platform-*`, and `tantu-core`
  arena internals. Every `unsafe` block needs a `// SAFETY:` comment.
- No `unwrap()`/`expect()` in library code paths reachable by users; return `tantu::Error` or handle.
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
cargo test -p tantu-test -- --ignored       # golden image tests (software renderer)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo run -p gallery                       # widget gallery example
cargo bench -p tantu-layout                 # layout / reactive benchmarks
```

Before declaring a task done: `fmt`, `clippy -D warnings`, and `test` must pass for the crates you touched.

## Testing expectations

- Layout algorithms: pure unit tests on constraint → size/position results.
- Widgets: `tantu-test::WidgetTester` (pump frames, simulate pointer/keyboard, query by key/semantics).
- Rendering: golden PNGs via `tantu-render-soft`; update with `TANTU_UPDATE_GOLDENS=1`.
- Reactive system: tests for glitch-freedom, disposal, and no leaks (count live nodes).
- New widgets require: at least one behavior test, one golden, and a gallery entry.

## Development workflow: Plan → Spec → Unit tests → Implementation

We combine spec-driven development with TDD. Every feature goes through these four steps in order.
Do not skip ahead: no implementation without a spec and failing tests.

1. **Plan.** Find the item in `PLAN.md` for the current phase. Don't start later-phase work. If the
   item is missing or too big, update `PLAN.md` first, splitting it into items that each fit one spec.
2. **Spec.** Write or update `docs/specs/<crate>/<feature>.md` before any code. A spec states:
   - purpose and scope (and what is explicitly out of scope)
   - public API: type and function signatures, with doc-comment-level descriptions
   - behavior as numbered, testable rules (e.g. `LAYOUT-FLEX-03: Expanded children share
     remaining main-axis space in proportion to flex`), including edge cases and error cases
   - performance or allocation constraints, if any
   - open questions

   Get the spec reviewed/agreed before moving on. Design decisions that go beyond one feature
   go in `docs/adr/NNNN-title.md`.
3. **Unit tests.** Turn each numbered rule into one or more tests, named after the rule id where
   practical (`fn layout_flex_03_expanded_shares_space()`). Add signatures or `todo!()` stubs so the
   tests compile, then run them and confirm they **fail for the right reason**.
4. **Implementation.** Write the minimum code that makes the tests pass, then refactor with the
   tests green. If implementation reveals the spec is wrong or incomplete, go back and update the
   spec and tests first. Don't let code and spec drift apart.

Done means all of the following:
- the spec matches the code
- every spec rule has a test
- `fmt`, `clippy -D warnings` and `test` pass for the touched crates
- the `PLAN.md` checkbox is ticked

Bug fixes follow the same loop in miniature: add the missing rule to the spec, write a failing
test that reproduces the bug, then fix it.

## General rules

- Keep changes within one crate where possible; respect the dependency rule above.
- Prefer small, reviewable commits. A commit may contain spec + tests + implementation for one
  item, but they must be written in that order.
- If a design decision here seems wrong, raise it. Do not silently diverge.
- **Keep `HANDOFF.md` current.** Every commit that gets pushed must include an updated
  `HANDOFF.md`, covering:
  - current state and phase
  - decisions made
  - a row in the commit log
  - next steps
  - open questions

  Read it first when starting a session.
- **Keep the architecture doc current.** `docs/architecture.md` and its diagram
  (`docs/architecture/tantu-architecture.svg`, plus the regenerated `.png`) describe the design.
  Update them in the same change as any design decision that affects the architecture:
  - a new, removed or renamed crate
  - a changed dependency edge
  - a change to the pipeline stages
  - a new backend or platform shell
  - a new or superseded ADR

  A change that leaves them stale is not done.
