# HANDOFF.md

Where the project stands, so the next session (human or agent) can pick up without re-reading the
whole history. Update this file in every commit (see `AGENTS.md` → General rules).

_Last updated: 2026-09-29_

## Current state

- **Phase:** pre-Phase 0. There are only docs so far, no Rust code or Cargo workspace yet.
- **Repo:** https://github.com/algorisys-oss/dkui-rs (private; will go public at the 0.1 release,
  once there's a working copy). Branch: `main`.
- **Files:**
  - `AGENTS.md`: architecture, crate layout, dependency rules, conventions, workflow
  - `CLAUDE.md`: Claude Code notes (imports AGENTS.md and PLAN.md)
  - `PLAN.md`: roadmap, comparisons with Knots and Clay, ADR list, phases
  - `README.md`: public-facing overview
  - `LICENSE-MIT`, `LICENSE-APACHE`: dual license

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
8. **License:** MIT OR Apache-2.0. Copyright line currently "The dkui Authors".
9. (Looked at and dropped: Liferay's clayui.com. That was the wrong Clay.)

## Commit log

| Commit | Summary |
|---|---|
| `e2f3e82` | Initial AGENTS.md, CLAUDE.md, PLAN.md |
| `c3e477d` | Plan → Spec → Unit tests → Implementation workflow |
| `08261d1` | README + dual MIT/Apache-2.0 license |
| `584bb1d` | Adopted Clay ideas (incl. sizing vocabulary) |
| `c02ae26` | Reverted to Flutter structure as the layout API; Clay for internals only |
| _this commit_ | Added HANDOFF.md and the rule to keep it updated |

A commit can't contain its own hash, so the newest row says _this commit_. The next update replaces
that with the real hash from `git log`.

## Next steps (Phase 0 in PLAN.md)

1. Cargo workspace skeleton: `Cargo.toml` with `[workspace.package]` (edition 2024,
   `license = "MIT OR Apache-2.0"`), `rust-toolchain.toml`, empty crates per the AGENTS.md table.
2. CI (GitHub Actions): fmt, clippy `-D warnings`, test on Linux/Windows/macOS.
3. `docs/specs/TEMPLATE.md` and `docs/adr/` with ADRs 0001–0006.
4. First spec → tests → code: `dkui-core` geometry (`Point`, `Size`, `Rect`, `Insets`, `Affine`, `Color`).
5. Then `dkui-reactive` spec (signals, memos, effects, batching, disposal).

## Open questions

- Copyright holder in `LICENSE-MIT`: keep "The dkui Authors", or use Algorisys / a named person?
- MSRV to pin (the local toolchain is rustc 1.95.0).
- Is the `dkui` name available on crates.io? Check before the first publish.
