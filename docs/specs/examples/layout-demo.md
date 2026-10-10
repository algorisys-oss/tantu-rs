# Layout demo

- **Status:** Draft
- **Crate:** `examples/layout-demo`
- **Plan item:** Phase 2, "**Milestone:** counter + layout demo run on Linux (Wayland + X11),
  Windows and macOS"
- **Related:** [layout widgets](../widgets/layout.md), [basic widgets](../widgets/basic.md),
  [app runner](../facade/app.md), [WidgetTester](../test/widget-tester.md),
  [counter](counter.md)

## Purpose

The second app for the Phase 2 milestone. The counter shows reactivity. This demo shows that
Flutter-style layout composes the way a Flutter developer expects, and that it reflows when the
window resizes. One window holds:

- a **header row**: a title on the left, a `Spacer`, then two buttons on the right;
- a **body row**: a fixed 160 px sidebar column of buttons, then an `Expanded` content area;
- in the **content area**:
  - a `Column` of three proportional bands (`Expanded` flex 1, 2 and 1);
  - a `Stack` with a `Positioned` badge in its top-right corner;
- a **footer**: centered text showing the window's state (a selected section, changed by the
  sidebar buttons).

Layout widgets draw nothing, so the bands and badge are colored boxes made from a small local
`Swatch` view (a `SizedBox` with a fill). That gives a reason to show a custom `Paint`, which
is how apps draw their own content today.

## Scope

In scope:

- `examples/layout-demo`: `src/main.rs` (runs the app) and `src/lib.rs` (`layout_demo() ->
  impl View`, `Swatch`), depending only on `tantu`.
- Tests with `WidgetTester` (needs `tantu-test` as a dev-dependency) and a golden at 800 × 600.

Out of scope: theming, scrolling, text input. These are Phase 3.

## Public API

```rust
// examples/layout-demo/src/lib.rs
/// The demo's view.
pub fn layout_demo() -> impl View;

/// A box filled with one color (an example of a custom `Paint`).
pub struct Swatch { /* color, optional size */ }
impl Swatch {
    pub fn new(color: Color) -> Self;
}
```

## Behavior

- **LAYOUT-DEMO-01:** At 800 × 600, the header row is the full width; the sidebar is 160 px wide;
  the content area fills the rest; the three bands' heights are in ratio 1 : 2 : 1; the badge
  sits in the stack's top-right corner.
- **LAYOUT-DEMO-02:** At 1200 × 800 the same structure reflows: the sidebar stays 160 px, the
  content area takes the extra width, and the bands keep their ratio.
- **LAYOUT-DEMO-03:** Tapping a sidebar button changes the footer text to that section's name.
- **LAYOUT-DEMO-04:** The 800 × 600 frame matches its golden (software renderer, test font).

## Milestone runs (manual)

`cargo run -p counter` and `cargo run -p layout-demo` must open, draw and respond on:

- Linux Wayland (default) and X11 (`WAYLAND_DISPLAY= cargo run ...`);
- Windows;
- macOS.

The agent can run the Linux ones. The Windows and macOS runs need the user, or a CI job that
only builds the examples. The PLAN.md milestone is ticked once all four have been seen working.

## Open questions (for the user)

1. **What the demo shows:** header, sidebar, proportional bands, stack and badge, footer.
   Proposal: as above. It exercises every Phase 2 layout widget.
2. **`Swatch` with a custom `Paint`** in the example, until `DecoratedBox`/`Container` arrive
   with the theme. Proposal: yes. It shows the extension point.
3. **Windows and macOS checks:** will you run the two examples there, or should CI build them
   on those runners (it can't open windows)? Proposal: CI already builds every example on all
   three OSes (`cargo test --workspace` compiles them), so the manual runs are the only
   missing piece.
