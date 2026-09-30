# Spec coverage check

- **Status:** Agreed
- **Crate:** `xtask` (dev tooling, not published)
- **Plan item:** Phase 0, "Spec-coverage check in CI (`cargo xtask spec-coverage`)"
- **Related:** [`AGENTS.md`](../../../AGENTS.md) → "Development workflow";
  [`docs/specs/README.md`](../README.md)

## Purpose

"Every spec rule has a test" is part of the definition of done in `AGENTS.md`. Today only review
checks it. This command checks it mechanically so CI fails when an agreed spec has a rule that no
test is named after. It is run by contributors locally and by CI.

## Scope

In scope:

- Finding the specs, reading their status and their rule ids.
- Finding function names in the workspace's Rust sources.
- Reporting every rule of an Agreed or Implemented spec that has no test named after it.

Out of scope:

- Checking that a matching function is actually a test (`#[test]`), or that it passes or fails.
  `cargo test` covers pass/fail; the name match is a deliberate, cheap approximation.
- Orphan tests: tests named after a rule id that doesn't exist in any spec. Hard to tell apart from
  ordinary names containing two-digit numbers. See open questions.
- Checking commit order (`spec:` → `test:` → `impl:`) in git history. See open questions.
- Checking that the spec matches the code beyond rule coverage. That stays with review.

## Public API

A new workspace member `xtask/` (package `xtask`, `publish = false`), run through a Cargo alias:

```toml
# .cargo/config.toml
[alias]
xtask = "run --quiet --package xtask --"
```

Command line:

```text
cargo xtask spec-coverage
```

The crate is split into a library (tested) and a thin `main.rs`. Library API:

```rust
/// Status of a spec, from its `- **Status:** <value>` line.
pub enum Status { Draft, Agreed, Implemented }

/// One rule found in a spec.
pub struct Rule {
    /// The rule id, e.g. `LAYOUT-FLEX-03`.
    pub id: String,
    /// True if the rule is struck through (removed); removed rules need no test.
    pub removed: bool,
    /// 1-based line number in the spec file.
    pub line: usize,
}

/// A parsed spec file.
pub struct Spec {
    /// The status, or `None` if the status line is missing or has an unknown value.
    pub status: Option<Status>,
    /// Rules in file order.
    pub rules: Vec<Rule>,
}

/// Parses the text of one spec file. Never fails; problems show up as `status: None`.
pub fn parse_spec(text: &str) -> Spec;

/// The test-name prefix for a rule id: lowercase, `-` replaced by `_` (`LAYOUT-FLEX-03` →
/// `layout_flex_03`).
pub fn test_name(rule_id: &str) -> String;

/// Names of all functions defined in one Rust source file.
pub fn fn_names(source: &str) -> Vec<String>;

/// One problem found by the check.
pub struct Problem {
    /// Path of the spec, relative to the workspace root, with `/` separators.
    pub path: String,
    /// Human-readable description, e.g. `LAYOUT-FLEX-03 (line 42) has no test named layout_flex_03 or layout_flex_03_*`.
    pub message: String,
}

/// Result of a full check.
pub struct Report {
    /// Specs with status Agreed or Implemented.
    pub checked: usize,
    /// Specs with status Draft.
    pub skipped: usize,
    /// Problems, sorted by path, then by line.
    pub problems: Vec<Problem>,
}

/// Runs the whole check against the workspace at `root`.
pub fn check(root: &std::path::Path) -> Report;

/// Runs the command line with `args` (without the program name) against the workspace at
/// `root`, writing normal output to `out` and errors to `err`. Returns the exit status.
/// `main.rs` only calls this with the real arguments and workspace root.
pub fn run(
    args: &[String],
    root: &std::path::Path,
    out: &mut dyn std::io::Write,
    err: &mut dyn std::io::Write,
) -> i32;
```

`Status`, `Rule`, `Spec`, `Problem` and `Report` derive `Debug`, `Clone` and `PartialEq`
(`Status` also `Copy` and `Eq`).

No dependencies beyond `std`.

## Behavior

Finding specs

- **XTASK-COV-01:** The specs are all `.md` files under `<root>/docs/specs/`, searched recursively,
  except files named `README.md` or `TEMPLATE.md`.
- **XTASK-COV-02:** If `<root>/docs/specs/` does not exist, the report has one problem
  (`docs/specs: directory not found`) and no specs.
- **XTASK-COV-03:** A spec file that can't be read as UTF-8 text is a problem for that path; the
  check carries on with the other files. Nothing panics.

Reading a spec

- **XTASK-COV-04:** The status is taken from the first line of the form `- **Status:** <value>`.
  The value is the first word after the colon (trailing HTML comments and text are ignored).
  `Draft`, `Agreed` and `Implemented` are recognized, case-sensitive.
- **XTASK-COV-05:** A spec with no status line, or with an unrecognized value, is a problem
  (`no valid status line`). It counts as neither checked nor skipped.
- **XTASK-COV-06:** Draft specs are skipped: their rules are not checked and they count in
  `skipped`.
- **XTASK-COV-07:** A rule is a list item whose text, after optional indentation, starts with
  `- **ID:**` (active) or `- ~~**ID:**` (removed). Rule ids mentioned anywhere else (prose, tables,
  and any line inside a fenced code block, which starts and ends with a line beginning with
  ```` ``` ````) are not rules.
- **XTASK-COV-08:** A rule id is two or more segments separated by `-`: every segment but the last
  is an uppercase letter followed by uppercase letters or digits, and the last segment is two or
  more digits. `LAYOUT-FLEX-03` and `A11Y-TREE-12` are ids; `FLEX-3`, `layout-flex-03` and `03` are
  not, and list items starting with them are not rules.
- **XTASK-COV-09:** An id that appears as a rule more than once in the same spec (active or removed)
  is a problem (`duplicate rule id`), reported once per extra occurrence.
- **XTASK-COV-10:** An Agreed or Implemented spec with no active rules is a problem
  (`no active rules`).

Matching tests

- **XTASK-COV-11:** Test names are collected from every `.rs` file under `<root>`, recursively,
  skipping `target/` and any directory whose name starts with `.`.
- **XTASK-COV-12:** A function name is the identifier after the keyword `fn` (preceded by start of
  line or whitespace and followed by whitespace). `async fn`, `pub fn`, `pub(crate) fn` and
  indented definitions all count.
- **XTASK-COV-13:** An active rule is covered if some function is named exactly `test_name(id)` or
  starts with `test_name(id)` followed by `_`. `layout_flex_03` and `layout_flex_03_shares_space`
  cover `LAYOUT-FLEX-03`; `layout_flex_030` and `layout_flex_3` don't.
- **XTASK-COV-14:** Each active rule of an Agreed or Implemented spec that is not covered is a
  problem, with the rule id, its line number and the expected name in the message. Removed rules
  are never problems.

Command line

- **XTASK-COV-15:** `cargo xtask spec-coverage` runs `check` on the workspace root (the parent of the
  `xtask` package directory), whatever the current directory is.
- **XTASK-COV-16:** `run` with `["spec-coverage"]` prints each problem as `<path>: <message>` on its own line to stdout, then a
  summary line `spec-coverage: <checked> checked, <skipped> draft, <n> problem(s)`. It exits with
  status 0 when there are no problems and 1 otherwise.
- **XTASK-COV-17:** `run` with no subcommand or an unknown one prints a usage message to stderr,
  listing `spec-coverage`, and exits with status 2.

CI

- **XTASK-COV-18:** The CI workflow runs `cargo xtask spec-coverage` on Linux on every push to
  `main` and every pull request, and the job fails when the command fails.

## Performance and allocation

None beyond finishing in well under a second on this repository. The per-frame allocation rules
in `AGENTS.md` don't apply to tooling.

## Open questions

Deferred (not needed for this version):

- **Orphan tests.** Flag tests named like a rule id (`core_geom_07_…`) where no spec has that
  rule, to catch typos. Needs a way to tell rule-named tests apart from ordinary names, for example
  by requiring the prefix of an existing spec area.
- **Commit order.** Check in CI that an `impl:` commit is preceded by a `test:` commit for the same
  item. Commit messages don't yet identify items reliably enough.

Resolved:

- **Placement in docs.** `xtask` is dev tooling, not part of the framework. It gets a tooling row
  in the `AGENTS.md` workspace table and is left out of the architecture diagram.
