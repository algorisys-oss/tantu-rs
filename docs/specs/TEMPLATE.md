# <Feature name>

- **Status:** Draft <!-- Draft | Agreed | Implemented -->
- **Crate:** `tantu-<crate>`
- **Plan item:** <the PLAN.md checkbox this spec belongs to>
- **Related:** <ADRs and other specs, as links; or "none">

## Purpose

One or two paragraphs: what this feature is for and who uses it (app developers, widget authors,
renderer authors, other crates).

## Scope

In scope:

- ...

Out of scope (and where it is handled, if anywhere):

- ...

## Public API

Every public item with its signature and a doc-comment-level description. Include builders,
traits, error types and feature flags. Private helpers don't belong here.

```rust
/// What this type is for.
pub struct Example { /* fields, if public */ }

impl Example {
    /// What this does, what it returns, and when it fails.
    pub fn new(/* ... */) -> Self;
}
```

## Behavior

Numbered, testable rules. One behavior per rule. Say what happens, not how it is implemented.
Cover the normal cases, the edge cases (empty, zero, negative, infinite, NaN, unbounded
constraints, overflow) and the error cases (what is returned or reported, never a panic in user
code paths).

- **AREA-FEATURE-01:** ...
- **AREA-FEATURE-02:** ...
- ~~**AREA-FEATURE-03:** ...~~ Removed: <reason>. <!-- ids are never reused -->

## Performance and allocation

Complexity limits, allocation rules (for example "no allocation per frame after warm-up") and
benchmark targets, if any. Write "None beyond the general rules in AGENTS.md" if there are none.

## Open questions

Anything not yet decided. Each one must be resolved, or explicitly deferred, before the status
moves to **Agreed**.

- ...
