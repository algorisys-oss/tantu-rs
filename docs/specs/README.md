# Specs

Every feature gets a spec here before any tests or code are written. This is step 2 of the
Plan → Spec → Unit tests → Implementation workflow in `AGENTS.md`.

- **Location:** `docs/specs/<crate>/<feature>.md`, with the crate name without the `tantu-` prefix.
  Example: `docs/specs/layout/flex.md`.
- **Start from** [`TEMPLATE.md`](TEMPLATE.md).
- **Rule ids** look like `LAYOUT-FLEX-03`: area, feature, then a two-digit number. The area is the
  crate name in capitals without the prefix (`CORE`, `REACTIVE`, `SCENE`, `LAYOUT`, …).
- **Tests** are named after the rule they check, in snake_case: `layout_flex_03_expanded_shares_space`.
  A rule may have several tests; every rule has at least one.
- **Stable ids.** Never renumber rules. A new rule gets the next free number, even if it belongs
  between two others. A dropped rule stays in the list, struck through, with the reason.
- **Status** moves from **Draft** to **Agreed** (reviewed, tests can be written) to
  **Implemented** (code and tests match the spec). If implementation shows the spec is wrong,
  update the spec and tests first.
- **Coverage.** `cargo xtask spec-coverage` (run in CI) fails if a spec marked Agreed or
  Implemented has a rule with no test named after it. Draft specs are skipped.
- **Commits.** The spec, the failing tests and the implementation are separate commits
  (`spec:`, `test:`, `impl:`); see `AGENTS.md` → "One commit per step".

Decisions that reach beyond one feature go in an ADR (`docs/adr/`), and the spec links to it.
