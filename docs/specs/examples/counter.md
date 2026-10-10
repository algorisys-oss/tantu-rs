# Counter example

- **Status:** Draft
- **Crate:** `examples/counter`
- **Plan item:** Phase 2, `tantu` facade → "`examples/counter`"
- **Related:** [app runner](../facade/app.md), [basic widgets](../widgets/basic.md),
  [layout widgets](../widgets/layout.md), AGENTS.md "Authoring style we are aiming for"

## Purpose

The first app written the way Tantu apps are meant to be written. Its source is the snippet in
AGENTS.md, character for character, so the documented authoring style is something that
compiles and runs, not an aspiration. Run it with `cargo run -p counter`.

## Scope

In scope:

- `examples/counter`: a library with `counter()`, and a binary whose `main` runs it.
  The two files together hold the AGENTS.md snippet.
- A test that drives the app through `run_with`, `FakePlatform` and the headless renderer.

Out of scope: styling beyond the snippet, a layout demo (the milestone item).

## Public API

```rust
// examples/counter/src/lib.rs
use tantu::prelude::*;

/// The counter: a title showing the count, and a button that increments it.
pub fn counter() -> impl View;

// examples/counter/src/main.rs
fn main() -> tantu::Result<()>; // App::new().window(Window::new("Counter").size(400.0, 300.0), counter).run()
```

## Behavior

- **COUNTER-01:** The AGENTS.md snippet, with `use counter::counter;` added and the library's
  doc comments removed, equals the concatenation of the example's `lib.rs` and `main.rs`,
  ignoring blank lines and leading/trailing whitespace. Editing one without the other fails
  this test.
- **COUNTER-02:** Run on `FakePlatform` with the test font (`without_system_fonts` +
  Liberation Sans): the first frame shows "Count: 0" (a glyph run whose glyph count is 8) and a
  button. A press and release on the button's center gives a frame showing "Count: 1", and two
  more presses give "Count: 3".

## Open questions

1. **Lib + bin split**, so the test can call `counter()`, as `scene-window` does. Proposal: yes.
   COUNTER-01 allows for the extra `use` line and the doc comments.
