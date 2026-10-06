---
name: code-to-docs
description: Generate Tantu's interactive documentation site - one self-contained HTML page (docs/interactive/index.html) built from the current code, specs, ADRs, PLAN.md and HANDOFF.md, with a clickable architecture diagram, crate panels, a spec explorer with rule-to-test coverage, ADRs, roadmap and API search. Use when the user asks for "interactive docs", "generate documentation site", "code to docs", "document the project", or invokes /code-to-docs.
---

# code-to-docs

Builds one HTML page that documents Tantu as the repository stands right now. Every fact on the
page is read from the repo at generation time. Nothing is invented, and nothing in the repo is
edited.

Files in this skill directory:

- `collect.py`: reads the repo and fills the template. Read-only except for the `--out` file.
- `template.html`: the page (inline CSS and JS, no network, light and dark, phone width). The
  data goes in at the `/*__TANTU_DATA__*/null` marker.
- `verify.py`: checks a generated page against the repo and `cargo xtask spec-coverage`.

## Rules

- Do not edit code, specs, ADRs, `PLAN.md`, `HANDOFF.md`, `AGENTS.md` or anything else in the
  repo. The only file written is the output page (default `docs/interactive/index.html`, or the
  path the user gives).
- Do not fix inconsistencies you find. Report them (see Report).
- Do not commit. The output is generated; see `.gitignore` advice in the Report section.
- These helpers are dev tooling outside the Rust workspace and are not covered by the spec
  workflow. Don't turn them into an `xtask` command without going through Plan → Spec → Tests.

## Steps

Run everything from the repo root.

1. **Generate.**

   ```bash
   python3 .claude/skills/code-to-docs/collect.py --root . --out docs/interactive/index.html
   ```

   It prints the counts, the spec-coverage summary it computed, and any problems. To look at
   the raw facts, use `--json` instead of `--out`.

2. **Verify.**

   ```bash
   python3 .claude/skills/code-to-docs/verify.py --root . docs/interactive/index.html
   ```

   It checks that the HTML parses, the embedded data is valid JSON, the page loads nothing from
   the network, the coverage line equals the last line of `cargo xtask spec-coverage`, and the
   crate list equals `cargo metadata --no-deps`. If it fails, the xtask and cargo are right:
   fix `collect.py` (the parsing mirrors `xtask/src/lib.rs`), not the repo, and regenerate.

3. **Spot-check what the scripts can't.** Read the generated JSON (`--json`) for anything that
   looks wrong and check it against the source:
   - the pipeline stages in `template.html` (`STAGES`, `FLOWS`) still match the diagram in
     `AGENTS.md` → "Architecture (the pipeline)" and `docs/architecture.md`. If a crate was
     added, renamed or removed, update `STAGES` in the template, not the repo;
   - a crate marked `placeholder` really has no API, and an `implemented` one does;
   - the current phase matches `HANDOFF.md`.

   If you have a browser tool, open the page and check light mode, dark mode and a 375 px wide
   viewport. A `file://` URL works; if the tool blocks it, serve the directory with
   `python3 -m http.server` on 127.0.0.1 and stop it afterwards.

4. **Share (only if asked).** If the user asks to share or publish the page, load the
   `artifact-design` skill, then publish the generated file with the Artifact tool. Otherwise
   just write the file.

## What the page contains

| Tab | Source |
|---|---|
| Overview | `AGENTS.md` "What this project is" and "Authoring style"; `HANDOFF.md` "Resume here", next steps, open questions; counts; current phase from `PLAN.md` checkboxes; consistency problems |
| Architecture | Pipeline SVG (stages from `AGENTS.md`), crate dependency graph from each `Cargo.toml`, layered by dependency depth. Clicking a crate opens its panel: responsibility and allowed deps (`AGENTS.md` table), actual deps, used-by, external and dev deps, status, specs, `lib.rs` docs, public items with file:line |
| Specs | Every `docs/specs/**.md` except README/TEMPLATE: status, crate, plan item, every rule with its text and the tests named after it (`CORE-GEOM-03` → `core_geom_03` or `core_geom_03_*`, as `cargo xtask spec-coverage` matches them). Filter by crate, status and "rules without a test"; search |
| ADRs | `docs/adr/NNNN-*.md`: status, date, full text |
| Roadmap | `PLAN.md` phases, items and sub-items, with progress |
| API | Search over every `pub` item in each crate's `src/` (structs, enums, traits, functions, methods, consts, modules, re-exports), with signature, doc comment and file:line |

Public items are read from source, not rustdoc JSON (that needs nightly). Methods are attributed
to the `impl` block they sit in; items behind `#[cfg(test)]` are skipped. `pub(crate)` is not
public.

## Report

Tell the user, briefly:

- **Generated:** the output path and size, the git branch and commit it reflects, and whether
  the tree had uncommitted changes.
- **Counts:** crates (with an API / total), public items, specs, active rules and how many have
  tests, ADRs, current phase and its progress.
- **Verification:** the `verify.py` result, including the `cargo xtask spec-coverage` line.
- **Inconsistencies found, not fixed:** each spec-coverage problem (rule without a test, missing
  status, duplicate id), each dependency problem (a crate depending on something the `AGENTS.md`
  table doesn't allow, `wgpu`/`tiny-skia`/`winit` outside `tantu-render-*`/`tantu-platform-*`, a
  crate missing from the table or from the workspace), public items with no doc comment if any,
  and anything you noticed in step 3. Name the workflow step that would fix each (`spec:`/`test:`
  commit, `AGENTS.md` change to discuss, and so on).
- **Git:** the page is a generated file. Suggest `/docs/interactive/` in `.gitignore` unless the
  user wants it committed (for example, to publish with GitHub Pages).
