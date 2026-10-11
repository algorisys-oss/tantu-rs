# Accessibility through the platform

- **Status:** Draft (for review; proposals in Open questions)
- **Crates:** `tantu-platform` (events and the update call, `FakePlatform` recording),
  `tantu-platform-winit` (the `accesskit_winit` adapter), `tantu` (the runner sending trees
  and routing actions)
- **Plan item:** Phase 3, "`tantu-a11y`", sub-item "the platform side"
- **Related:** [ADR 0013](../../adr/0013-accessibility-pipeline.md), [semantics](semantics.md),
  [platform](../platform/platform.md), [winit shell](../platform-winit/shell.md),
  [facade](../facade/app.md), `accesskit_winit` 0.32

## Purpose

The semantics spec builds AccessKit trees; this one delivers them. A screen reader asks the
platform for a window's tree (activation), receives updates after each change, and sends
actions back. All of it crosses the `Platform` trait as events and one context call, so the
runner stays shell-independent and `FakePlatform` can script a screen reader in tests.

## Scope

In scope: `AccessibilityEvent`, `WindowEvent::Accessibility`,
`PlatformContext::update_accessibility`; `FakePlatform` scripting and logging; the winit
adapter; the runner building and sending trees while active and routing actions.

Out of scope: platform-specific tuning (Windows UIA and macOS NSAccessibility are handled by
AccessKit's adapters), screen-reader-specific announcements, the gallery under a real screen
reader (manual check by the user; noted in HANDOFF.md).

## Public API

```rust
// tantu-platform
/// What assistive technology does with a window.
#[derive(Clone, Debug, PartialEq)]
pub enum AccessibilityEvent {
    /// Assistive technology wants the window's tree: send one with `update_accessibility`
    /// now and after every change.
    Activated,
    /// No consumer any more; updates can stop.
    Deactivated,
    /// A request on a node.
    Action(accesskit::ActionRequest),
}

pub enum WindowEvent { /* ... */ Accessibility(AccessibilityEvent) }

pub trait PlatformContext {
    /// Sends a tree update to the window's assistive technology (ignored while inactive or
    /// for an unknown window).
    fn update_accessibility(&mut self, window: WindowId, update: accesskit::TreeUpdate);
}

pub struct FakeWindow {
    /// The updates sent, in order (PLATFORM-FAKE-13).
    pub accessibility_updates: Vec<accesskit::TreeUpdate>,
}
```

## Behavior

- **PLATFORM-A11Y-01:** `AccessibilityEvent` and `WindowEvent::Accessibility` are delivered like
  other window events; `update_accessibility` passes the update to the window's adapter. The
  types are AccessKit's own (`ActionRequest`, `TreeUpdate`), so no conversion is lossy.
- **PLATFORM-FAKE-13:** `FakePlatform` scripts `Accessibility` events as any event and records
  every `update_accessibility` call in the window's `accessibility_updates` (also while no
  activation was scripted, so tests see what the handler sent).
- **PLATFORM-WINIT-15:** Each window gets an `accesskit_winit::Adapter` created with the
  window and an event-loop proxy before the window is shown; every winit window event is
  passed to the adapter first (`process_event`); the adapter's `InitialTreeRequested`,
  `ActionRequested` and `AccessibilityDeactivated` arrive as `Activated`, `Action` and
  `Deactivated`; `update_accessibility` calls `update_if_active`. Checked in the opt-in
  window test only as far as creation goes (no screen reader in CI).
- **FACADE-APP-15:** On `Activated` the runner marks the window active and sends a full tree at
  once; after every later frame of an active window it sends the tree again (one update per
  frame; nothing while inactive); on `Deactivated` it stops; on `Action` it maps the request
  with `AccessibilityTree::map_action` and calls `ViewTree::perform_action`, then requests a
  frame if the tree needs one. The app name is the first window's title.

## Performance and allocation

One full tree per frame per active window (the semantics spec's cost); nothing while no
assistive technology is attached, which is the common case.

## Open questions

Proposals (say "continue" to take them, or change them):

1. **AccessKit's own types on the platform trait** (`TreeUpdate`, `ActionRequest`), as the
   IME events use winit's shape: `tantu-platform` depends on the `accesskit` data crate
   (ADR 0013), and only `tantu-platform-winit` on `accesskit_winit`.
2. **A tree per frame while active**, pushed by the runner, rather than on demand: AccessKit
   wants updates after every change and diffs them itself; a frame is the change unit.
3. **Activation is per window**, mirroring `accesskit_winit`'s adapter per window.
4. **The app name comes from the first window's title**; `App::name(..)` can come when
   multi-window needs it.
