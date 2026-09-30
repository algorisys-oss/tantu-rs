# CLAUDE.md

@AGENTS.md
@PLAN.md
@HANDOFF.md

## Claude Code specifics

- `AGENTS.md` holds the architecture, crate boundaries, conventions and commands. Follow it and
  do not repeat it here. `PLAN.md` holds the phased roadmap. Work on the current phase only.
- Always follow the **Plan → Spec → Unit tests → Implementation** workflow in `AGENTS.md`. When
  asked to "implement X", first check for `docs/specs/.../X.md`. If there isn't one, write the spec
  and stop for review before writing tests or code, unless the user says to go straight through.
  Show the failing test run before implementing. This applies to all code, including dev tooling.
- Commit each workflow step separately (`spec:`, `test:`, `impl:`), as described in `AGENTS.md` →
  "One commit per step". Never combine them in one commit.
- Before editing a crate, read its `lib.rs` module docs and the relevant section of `AGENTS.md`.
- After changes, run `cargo fmt --all`, `cargo clippy -p <crate> --all-targets -- -D warnings`
  and `cargo test -p <crate>`. Report failures verbatim; don't paper over them.
- GPU tests may not run in this environment. Prefer `tantu-render-soft` / `tantu-render-headless`
  for verification, and say so when a change was not checked on a real GPU.
- When a task changes a public API, update the doc comments, the affected examples, and
  `examples/gallery` in the same change.
- When adding a dependency, check it against the dependency rule in `AGENTS.md`, prefer crates
  already used in the Rust GUI ecosystem (winit, wgpu, parley, swash, tiny-skia, accesskit, taffy),
  and note why in the commit message.
- When a design decision changes the architecture, update `docs/architecture.md` and the diagram
  (`docs/architecture/tantu-architecture.svg` → regenerate the PNG) in the same change.
- Record significant design decisions as ADRs in `docs/adr/`. If a decision in `AGENTS.md` needs
  to change, propose it first rather than making the change unilaterally.
- Do not commit or push unless asked. Branch off `main` for feature work.
- At the start of a session, read `HANDOFF.md`. Before every push, update `HANDOFF.md`
  (state, decisions, commit log rows, next steps, open questions) and include it in the last commit
  of the push.
