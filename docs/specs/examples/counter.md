# Counter example

- **Status:** Implemented (the user said "continue" on the draft; the file layout was simplified while agreeing, review)
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

- `examples/counter`: a binary whose `src/main.rs` is the AGENTS.md snippet.
- Tests that pull `main.rs` into a module (`include!`) to reach `counter()`, and drive it
  through `App::handler`, `FakePlatform` and the headless renderer (a dev-dependency).

Out of scope: styling beyond the snippet, a layout demo (the milestone item).

## Public API

```rust
// examples/counter/src/main.rs: the AGENTS.md snippet
use tantu::prelude::*;

fn counter() -> impl View;
fn main() -> tantu::Result<()>; // App::new().window(Window::new("Counter").size(400.0, 300.0), counter).run()
```

## Behavior

- **COUNTER-01:** The example's `main.rs` equals the first Rust code block under "Authoring
  style we are aiming for" in AGENTS.md, ignoring all whitespace. rustfmt splits the snippet's
  last chain over three lines, so the file stays rustfmt-clean and still matches the snippet
  token for token. Editing one without the other fails this test.
- **COUNTER-02:** Run on `FakePlatform` with the test font (`without_system_fonts` +
  Liberation Sans), the first frame shows "Count: 0" and a button. Its first glyph run has the
  glyph ids of "Count: 0" in `TextStyle::title()`. A press and release on the button's center
  gives a frame showing "Count: 1", and two more give "Count: 3".

## Open questions

Resolved while agreeing (2026-10-10; review): **no lib/bin split.** `main.rs` is the snippet
verbatim (up to rustfmt's whitespace), and the tests reach `counter()` by including `main.rs`
in a module. The example lists the workspace lints itself, minus `missing_docs`, because the
snippet has no crate doc comment.
