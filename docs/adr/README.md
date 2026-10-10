# Architecture decision records

Each ADR records one design decision that goes beyond a single feature: the context, what we
decided, what it costs us and what we rejected. Feature-level behavior goes in `docs/specs/`
instead.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-retained-tree-and-fine-grained-reactivity.md) | Retained tree and fine-grained reactivity | Accepted (disposed-handle consequence superseded by 0007) |
| [0002](0002-flutter-structure-and-layout-protocol.md) | Flutter structure and layout protocol | Accepted |
| [0003](0003-scene-as-the-renderer-contract.md) | Scene as the renderer contract | Accepted |
| [0004](0004-platform-trait-with-winit-by-default.md) | Platform trait, with winit by default | Accepted |
| [0005](0005-text-stack-parley-swash-fontique.md) | Text stack: parley, swash and fontique | Accepted |
| [0006](0006-wgpu-as-the-default-gpu-backend.md) | wgpu as the default GPU backend | Accepted |
| [0007](0007-using-disposed-reactive-handles.md) | Using disposed reactive handles | Accepted |
| [0008](0008-renderer-conformance-suite.md) | Renderer conformance suite | Accepted |
| [0009](0009-layout-tree-in-tantu-layout.md) | The layout tree lives in `tantu-layout` | Accepted |
| [0010](0010-text-measurement-in-layout.md) | Text measurement in layout | Accepted |

## Writing a new ADR

1. Copy the template below to `NNNN-short-title.md`, using the next free number.
2. Start it as **Proposed** and get it agreed before code depends on it.
3. Add it to the index above and to the decisions table in `docs/architecture.md`, and update
   the diagram if the decision changes the architecture (rule in `AGENTS.md`).

ADRs are not rewritten after they are accepted. To change a decision, write a new ADR, set the
old one's status to **Superseded by NNNN**, and link both ways. Typos and broken links can be
fixed in place.

Status values: **Proposed**, **Accepted**, **Rejected**, **Superseded by NNNN**.

## Template

```markdown
# NNNN. Title

- **Status:** Proposed
- **Date:** YYYY-MM-DD

## Context

What problem are we solving, and what forces are in play? Facts, constraints, requirements.

## Decision

What we will do, stated plainly ("We will ...").

## Consequences

What gets easier, what gets harder, and what we now have to do because of this decision.

## Alternatives considered

Each serious option we rejected, and why.
```
