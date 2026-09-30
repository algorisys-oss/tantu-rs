# Tantu

**Compose once. Render your way.**

Tantu is a cross-platform UI framework for Rust, designed around declarative widgets, fine-grained
reactivity, and independent rendering backends. Its API is Flutter-style and its core is
renderer-agnostic.

*Tantu* (pronounced "tan-too") is Sanskrit for "thread": small pieces (widgets, state, layout,
rendering) woven into something larger. The name is about composition, not multithreading.
By Algorisys Technologies.

> **Status: pre-alpha, design phase.** There is no usable code yet. The API shown below is the
> target, not something you can run today. See [PLAN.md](PLAN.md) for the roadmap.

## Goals

- **One codebase, every desktop.** Windows, macOS and Linux (Wayland and X11).
- **Renderer independence.** UI, layout and widget logic produce a plain display list (`Scene`).
  Renderers consume it: GPU via wgpu (Vulkan/Metal/DX12/GL), a CPU renderer via tiny-skia, and a
  headless recorder for tests. More targets (web, remote rendering) can be added without touching
  app code.
- **Flutter-like authoring.** Declarative, composable widget trees written in plain Rust, with
  Flutter's layout model: constraints go down, sizes go up, and the parent positions its children.
- **Built for large apps.** Retained element tree with fine-grained reactive signals, so only what
  changed is rebuilt, re-laid out or repainted. Planned: virtualized lists and data grids, docking,
  multi-window, accessibility (AccessKit), full text shaping with IME and right-to-left support,
  i18n and theming.
- **Also for small apps.** Fast startup and a single, reasonably small binary.

## A taste of the API (planned)

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

## How it works

```
App code (views + signals)
   → Element tree (retained: identity, state, focus)
   → Layout (BoxConstraints ↓, Size ↑)
   → Scene (backend-neutral display list)  → wgpu | tiny-skia | headless
   → AccessKit tree                         → platform screen readers
Platform layer (winit by default): windows, input, IME, clipboard, menus, dialogs
```

The crate breakdown and dependency rules are in [AGENTS.md](AGENTS.md).

## Inspiration and prior art

- [Knots](https://github.com/knots-ui/knots): its UI engine is decoupled from windowing and graphics
  APIs through a plain render packet. Tantu keeps that idea but uses a retained, reactive tree instead
  of immediate mode.
- [Flutter](https://flutter.dev): widget composition and the constraint-based layout protocol.
- [Clay](https://github.com/nicbarker/clay): renderer-agnostic render commands, text measure caching,
  anchored floating elements and arena-based performance techniques.
- Rust GUI projects we learn from: Xilem/Masonry, Floem, Iced, Slint, egui, GPUI.

## Roadmap

In brief (full detail in [PLAN.md](PLAN.md)):

1. Foundations: geometry, reactive signals, CI
2. Pixels on screen: Scene, wgpu and software renderers, winit platform layer
3. Layout, views and text
4. Core widgets, focus, scrolling, theming, animation, accessibility
5. Enterprise features: DataGrid, TreeView, docking, menus, dialogs, i18n
6. Performance, devtools, hot reload, packaging, 0.1 release

## Contributing

Development follows **Plan → Spec → Unit tests → Implementation**. Every feature starts from a
written spec in `docs/specs/`, then gets failing tests, and only then an implementation. Details
and coding conventions are in [AGENTS.md](AGENTS.md).

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
