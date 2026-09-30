# HANDOFF.md

Where the project stands, so the next session (human or agent) can pick up without re-reading the
whole history. Update this file in every commit (see `AGENTS.md` → General rules).

_Last updated: 2026-09-30_

## Resume here (session of 2026-09-30)

Phase 0 is under way. On `main`: the Cargo workspace skeleton, CI (green on Linux, Windows and
macOS), the spec template and ADRs 0001–0006. The next item is the first real feature, `tantu-core` geometry, starting with its spec.

When resuming, tell the agent: "Read HANDOFF.md and continue."

## Current state

- **Name:** **Tantu** (pronounced "tan-too", Sanskrit for "thread"). Tagline: *Compose once. Render
  your way.* Crates: `tantu`, `tantu-core`, `tantu-reactive`, `tantu-layout`, `tantu-widgets`,
  `tantu-render-wgpu`, … (full list in AGENTS.md). App import: `use tantu::prelude::*;`
- **Phase:** Phase 0 (Foundations). The workspace skeleton exists: 16 empty crates under `crates/`
  (the AGENTS.md table), with the internal dependency edges from that table already declared. No
  feature code yet.
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
  - `.github/workflows/ci.yml`: fmt, clippy + rustdoc (`-D warnings`), test on Linux/Windows/macOS

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
16. (Looked at and dropped: Liferay's clayui.com. That was the wrong Clay.)

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
| _this commit_ | Spec template and README (`docs/specs/`), ADRs 0001–0006 (`docs/adr/`), ADR links in `docs/architecture.md` |

A commit can't contain its own hash, so the newest row says _this commit_ (or _uncommitted_ for work not yet committed). The next update replaces
that with the real hash from `git log`.

## Next steps (Phase 0 in PLAN.md)

1. First spec → tests → code: `tantu-core` geometry (`Point`, `Size`, `Rect`, `Insets`, `Affine`, `Color`).
   Write `docs/specs/core/geometry.md` from the template and stop for review before tests.
2. Then `tantu-reactive` spec (signals, memos, effects, batching, disposal).

## Open questions

- Is `tantu` (and `tantu-*`) available on crates.io? Check, and consider reserving it, before the
  first publish. Same for a domain / GitHub org name if wanted.
