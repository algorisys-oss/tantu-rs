# HANDOFF.md

Where the project stands, so the next session (human or agent) can pick up without re-reading the
whole history. Update this file in every commit (see `AGENTS.md` → General rules).

_Last updated: 2026-09-30_

## Resume here (session of 2026-09-30)

Phase 0 is under way. On `main`: the Cargo workspace skeleton, CI (green on Linux, Windows and
macOS), the spec template, ADRs 0001–0006, the one-commit-per-step workflow rule and the
spec-coverage check (`cargo xtask spec-coverage`, also a CI job), and the first feature code:
`tantu-core` geometry (`Point`, `Vec2`, `Size`, `Rect`, `EdgeInsets`, `Affine`). The next item is
`tantu-core` `Color`, starting with its spec.

When resuming, tell the agent: "Read HANDOFF.md and continue."

## Current state

- **Name:** **Tantu** (pronounced "tan-too", Sanskrit for "thread"). Tagline: *Compose once. Render
  your way.* Crates: `tantu`, `tantu-core`, `tantu-reactive`, `tantu-layout`, `tantu-widgets`,
  `tantu-render-wgpu`, … (full list in AGENTS.md). App import: `use tantu::prelude::*;`
- **Phase:** Phase 0 (Foundations). The workspace skeleton exists: 16 empty crates under `crates/`
  (the AGENTS.md table), with the internal dependency edges from that table already declared.
  `tantu-core` has the geometry module (spec `docs/specs/core/geometry.md`, Implemented); the
  other crates are still empty.
- **Repo:** https://github.com/algorisys-oss/tantu-rs (public). Branch: `main`.
- **Files:**
  - `AGENTS.md`: architecture, crate layout, dependency rules, conventions, workflow
  - `CLAUDE.md`: Claude Code notes (imports AGENTS.md and PLAN.md)
  - `PLAN.md`: roadmap, comparisons with Knots and Clay, ADR list, phases
  - `README.md`: public-facing overview
  - `docs/architecture.md`: architecture write-up; diagram in `docs/architecture/`
    (`tantu-architecture.svg` is the source, `.png` is rendered from it, `rough-sketch.png` is the original sketch)
  - `LICENSE-MIT`, `LICENSE-APACHE`: dual license
  - `Cargo.toml`: workspace manifest (`[workspace.package]`, internal crates in
    `[workspace.dependencies]`, shared lints in `[workspace.lints]`)
  - `rust-toolchain.toml`: pins Rust 1.85, which is also the MSRV; `clippy.toml`
  - `crates/`: one directory per crate; `crates/tantu` is the facade
  - `docs/adr/`: ADRs 0001–0006 plus `README.md` (index, template, how to supersede)
  - `docs/specs/`: `TEMPLATE.md` and `README.md` (location, rule-id and test-name conventions)
  - `.github/workflows/ci.yml`: fmt, clippy + rustdoc (`-D warnings`), test on Linux/Windows/macOS,
    spec coverage
  - `xtask/`: dev tooling (`cargo xtask spec-coverage`); alias in `.cargo/config.toml`

## Decisions made so far

1. **Flutter-like structure is the API.** Apps compose widgets; layout uses Flutter's layout widgets
   (`Row`, `Column`, `Expanded`, `Padding`, `Stack`, …) and its constraint protocol, with Flutter
   names in snake_case. No alternative layout vocabulary.
2. **Retained tree + fine-grained signals.** Not immediate mode, unlike Knots and Clay.
3. **Renderer independence via `Scene`,** a data-only display list (from Knots' `render.Packet`
   and Clay's render commands). Backends: wgpu, tiny-skia (software), headless.
4. **From Clay, only internals:**
   - text measurement through a trait, with a per-word cache
   - element id + z-index on each Scene command
   - culling of off-screen content
   - anchored overlay positioning
   - arena storage, no allocation per frame
   - devtools drawn as Scene commands

   Clay's Fit/Grow sizing was explicitly **rejected**.
5. **Platform:** winit behind a `Platform` trait; Linux targets both Wayland and X11.
6. **Text:** parley/swash/fontique. **A11y:** AccessKit.
7. **Workflow:** Plan → Spec (`docs/specs/`) → failing unit tests → implementation.
8. **License:** MIT OR Apache-2.0. Copyright: "Rajesh Pillai - Algorisys Technologies"
   (use the same in `[workspace.package] authors`).
9. **Architecture doc is kept current.** `docs/architecture.md` and its diagram must be updated in
   the same change as any design decision that affects them (rule in AGENTS.md).
10. **Name: Tantu** (was dkui). Chosen over YappyUI, Vayra and keeping dkui: short, has a story
    (the thread metaphor stands for composition, not multithreading), and doesn't tie the brand to
    Rust or a backend. Room for an ecosystem: Tantu Inspector, Studio, Themes, Gallery. Attribution
    "by Algorisys". Crate prefix `tantu-`; golden-update env var `TANTU_UPDATE_GOLDENS`.
11. **Public from the start** (2026-09-30), not at 0.1 as first planned. Repo: `algorisys-oss/tantu-rs`.
12. **MSRV = Rust 1.85** (the first release with edition 2024), pinned in `rust-toolchain.toml`, so
    local and CI builds run the MSRV itself. `rust-version` in `Cargo.toml` must match. Raise both
    together when a dependency needs a newer compiler (wgpu may be the first to force this).
13. **Workspace setup:** crates live in `crates/<name>`. Internal crates are declared once in
    `[workspace.dependencies]` and each crate depends only on what the AGENTS.md table allows.
    Shared lints: `missing_docs`, `unsafe_op_in_unsafe_fn`, `clippy::undocumented_unsafe_blocks`,
    `clippy::unwrap_used`, `clippy::print_stdout`/`print_stderr` (unwrap/print allowed in tests via
    `clippy.toml`). `#![forbid(unsafe_code)]` in every crate except `tantu-core`,
    `tantu-platform-winit` and `tantu-render-*`. Facade features: `wgpu`, `winit`, `default-theme`
    (default) and `soft`. `Cargo.lock` is committed.
14. **No PRs for now.** Rajesh is the only developer: do feature work on a branch, then
    fast-forward merge into `main` and push. CI runs on the push to `main`.
15. **ADRs and specs conventions.** ADRs 0001–0006 are Accepted and record decisions 1–6 above
    (retained + signals, Flutter layout, Scene contract, Platform trait, text stack, wgpu). Accepted
    ADRs are not rewritten; a new ADR supersedes them. Specs live in `docs/specs/<crate>/<feature>.md`
    (crate name without `tantu-`), with rule ids like `LAYOUT-FLEX-03` that are never renumbered,
    and statuses Draft → Agreed → Implemented.
16. **Spec → TDD → code for all code, one commit per step** (2026-09-30). Every item lands as
    `spec:` (spec Agreed), `test:` (failing tests) and `impl:` commits, pushed together. This
    covers tooling too. `HANDOFF.md` is updated in the last commit of each push (it used to be every
    commit). `cargo xtask spec-coverage` (in CI) fails when a rule of an Agreed/Implemented spec has
    no test named after it (`LAYOUT-FLEX-03` → `layout_flex_03` or `layout_flex_03_*`). `xtask` is
    dev tooling: in the AGENTS.md table, not in the architecture diagram.
17. **Geometry shape** (2026-09-30, spec `docs/specs/core/geometry.md`):
    - `Point` (where) and `Vec2` (how far) are separate types, unlike Flutter's single `Offset`.
      The parent-to-child offset in layout is a `Vec2`.
    - `EdgeInsets`, Flutter's name, not `Insets`.
    - `Rect` stores edges (`left, top, right, bottom`) like Flutter.
    - Values are stored as given: no clamping, no reordering, nothing panics, NaN propagates. The
      one exception is `EdgeInsets::deflate_size`, which clamps to 0 as layout needs.
    - `Affine::inverse` is computed in f64, but whether a transform is invertible follows the f32
      `determinant()`, so the two agree. Its accuracy is only promised for well-conditioned
      transforms (CORE-GEOM-28 was corrected when writing the tests).
18. (Looked at and dropped: Liferay's clayui.com. That was the wrong Clay.)

## Commit log

| Commit | Summary |
|---|---|
| `e2f3e82` | Initial AGENTS.md, CLAUDE.md, PLAN.md |
| `c3e477d` | Plan → Spec → Unit tests → Implementation workflow |
| `08261d1` | README + dual MIT/Apache-2.0 license |
| `584bb1d` | Adopted Clay ideas (incl. sizing vocabulary) |
| `c02ae26` | Reverted to Flutter structure as the layout API; Clay for internals only |
| `34c12e3` | Added HANDOFF.md and the rule to keep it updated |
| `594513f` | Set copyright holder to Rajesh Pillai - Algorisys Technologies |
| `a2c71a4` | Architecture doc + diagram (`docs/architecture.md`, `docs/architecture/`), rule to keep them current (AGENTS.md, CLAUDE.md); renamed project dkui → Tantu across all docs, README tagline; moved to new public repo `algorisys-oss/tantu-rs` |
| `9703361` | Removed references to the old repo from HANDOFF.md |
| `f3b4adb` | Phase 0: Cargo workspace skeleton (16 crates), `rust-toolchain.toml` (1.85), shared lints, CI workflow |
| `6d7c732` | Merged the workspace skeleton into `main`; no-PR workflow noted |
| `6084f23` | CI green on Linux/Windows/macOS; ticked the workspace/CI item in PLAN.md |
| `e21e4b3` | Spec template and README (`docs/specs/`), ADRs 0001–0006 (`docs/adr/`), ADR links in `docs/architecture.md` |
| `c3aa616` | docs: one-commit-per-step rule (AGENTS.md, CLAUDE.md, specs README) |
| `0678f59` | spec: spec-coverage check (XTASK-COV-01..18), PLAN.md item |
| `ba79621` | test: spec-coverage check, 20 failing tests |
| `8f51517` | impl: `xtask` spec-coverage check, CI job, AGENTS.md row/commands |
| `19cd9ed` | spec: tantu-core geometry (CORE-GEOM-01..29); PLAN.md tantu-core item split into geometry, color, id + arena |
| `bdeb293` | spec: realistic accuracy promise for `Affine::inverse` (CORE-GEOM-28) |
| `3e43794` | test: tantu-core geometry, stubs + 31 tests (30 failing on `todo!()`) |
| _this commit_ | impl: tantu-core geometry; spec Implemented, PLAN.md ticked |

A commit can't contain its own hash, so the newest row says _this commit_ (or _uncommitted_ for work not yet committed). The next update replaces
that with the real hash from `git log`.

## Next steps (Phase 0 in PLAN.md)

1. `docs/specs/core/color.md` (`Color`: representation, color space, constructors, alpha), then
   stop for review before tests.
2. Then `arena.md` (`Id`, generational arena) the same way.
3. Then `tantu-reactive` spec (signals, memos, effects, batching, disposal).

## Open questions

- Is `tantu` (and `tantu-*`) available on crates.io? Check, and consider reserving it, before the
  first publish. Same for a domain / GitHub org name if wanted.
