# Tantu

**Compose once. Render your way.**

Tantu is a general-purpose GUI library for building desktop apps in Rust: declarative widgets,
fine-grained reactivity and independent rendering backends. Its API is Flutter-style and its core
is renderer-agnostic. It is meant to carry anything from a small utility to a large
enterprise application (data grids, docking, forms, accessibility, i18n).

*Tantu* (pronounced "tan-too") is Sanskrit for "thread": small pieces (widgets, state, layout,
rendering) woven into something larger. The name is about composition, not multithreading.
By Algorisys Technologies.

> **Status: pre-alpha.** Phases 0 to 2 are done and Phase 3 (the core widget set) is well under
> way: windows, layout, text, focus, overlays and the first widgets work on Linux, Windows and
> macOS. The API is still moving and nothing is published to crates.io yet. See
> [PLAN.md](PLAN.md) for the roadmap and [HANDOFF.md](HANDOFF.md) for the current state.

## Goals

- **One codebase, every desktop.** Windows, macOS and Linux (Wayland and X11).
- **Renderer independence.** UI, layout and widget logic produce a plain display list (`Scene`).
  Renderers consume it: GPU via wgpu (Vulkan/Metal/DX12/GL), a CPU renderer via tiny-skia, and a
  headless recorder for tests. More targets (web, remote rendering) can be added without touching
  app code.
- **Flutter-like authoring.** Declarative, composable widget trees written in plain Rust, with
  Flutter's layout model: constraints go down, sizes go up, and the parent positions its children.
- **General purpose.** Everyday desktop apps first: windows, text, input, scrolling, theming,
  and a small, fast-starting binary.
- **Scales to large apps.** Retained element tree with fine-grained reactive signals, so only what
  changed is rebuilt, re-laid out or repainted. Virtualized lists and a data grid (a million
  rows, frozen, sortable, resizable and reorderable columns), accessibility (AccessKit) and
  text shaping with IME and right-to-left support today; planned: docking, multi-window,
  i18n.

## Using Tantu in your app

Tantu isn't on crates.io yet, so depend on the repository. The `tantu` crate is the facade: it
re-exports everything an app needs and brings the default winit shell and wgpu renderer.

```toml
[package]
name = "my-app"
edition = "2024"
rust-version = "1.88"

[dependencies]
tantu = { git = "https://github.com/algorisys-oss/tantu-rs" }
```

A window shows a *view*: a function building a tree of widgets, with state in signals.
`src/main.rs`:

```rust
use tantu::prelude::*;

fn app() -> impl View {
    let name = TextEditingController::new("");
    let greeting = move || {
        let text = name.text();
        if text.is_empty() { "Type your name".to_string() } else { format!("Hello, {text}!") }
    };
    Padding::all(16.0).child(
        Column::new()
            .spacing(12.0)
            .child(Text::new(greeting).style(TextStyle::title()))
            .child(TextField::new(name).placeholder("Name"))
            .child(
                Row::new()
                    .spacing(8.0)
                    .child(Button::new("Clear").on_press(move || name.set_text("")))
                    .child(Icon::new(Icons::CHECK)),
            ),
    )
}

fn main() -> tantu::Result<()> {
    App::new().window(Window::new("Hello").size(480.0, 320.0), app).run()
}
```

`cargo run` opens the window. What you can build with today:

- **Layout:** `Row`, `Column`, `Expanded`, `Flexible`, `Spacer`, `Padding`, `Align`, `Center`,
  `SizedBox`, `ConstrainedBox`, `LimitedBox`, `Stack`, `Positioned`, `LayoutBuilder`.
- **Widgets:** `Text`, `RichText`, `Button`, `IconButton`, `Icon` (Material Icons), `Image`
  (PNG), `Checkbox`, `Radio`, `Switch`, `Slider`, `Divider`, `TextField` (with undo and IME),
  `Tooltip`.
- **State:** `signal`, `memo`, `effect`; every widget prop takes a value, a closure or a signal.
- **Input:** pointer events, keyboard focus and Tab traversal, `Shortcuts` and `Actions`,
  overlays anchored to widgets.
- **Scrolling:** `ScrollView` with a scrollbar, and `ListView::builder` that builds only the
  rows in view (100 000 rows cost what a dozen do).
- **Theming:** `Theme::light()` and `Theme::dark()` (Material 3 colors, text presets, shape,
  density); every window gets a `Signal<Theme>` it can switch at run time, and `ThemeProvider`
  scopes a theme to a subtree.
- **Animation:** `AnimationController`, `Tween` and `Curve` as in Flutter, `animated()`, which
  turns any reactive prop's changes into transitions, `Opacity` and a cross-fading
  `AnimatedSwitcher`; timers and tickers underneath. The caret blinks, double-click selects a
  word and the wheel scrolls smoothly.
- **Testing:** `tantu-test::WidgetTester` pumps frames, taps, types, advances a fake clock and
  compares goldens without a window.

`cargo run -p gallery` opens the widget gallery (`examples/gallery`), every widget in one
window with a dark-mode switch; `examples/counter` and `examples/layout-demo` are smaller.
Things still to come are listed in [PLAN.md](PLAN.md): the data grid's selection and cell
editors, and the other enterprise widgets (trees, tabs, docking, menus, dialogs, forms).

`cargo run --release -p data-grid-stress` opens the standalone 1,000,000 × 20 DataGrid
workload, with Start/Stop, beginning/middle/end jumps, optional frozen columns and redraw
cadence statistics. The kitchen sink's grid also demonstrates resize/reorder, range
selection and editable Name/Score cells with validation feedback. See the
[stress measurement procedure](examples/data-grid-stress/README.md) for repeatable trials;
UI tick cadence and the headless CPU benchmark do not certify GPU/compositor delivery.

## The counter, in full

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
