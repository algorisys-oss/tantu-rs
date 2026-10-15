# Dialogs, part 1: modal dialogs

- **Status:** Agreed (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-view` (window-covering overlays, dialog roles, `Semantics::modal`),
  `tantu-a11y` (mapping), `tantu-widgets` (`Dialog`, `DialogController`, `AlertDialog`),
  `examples/gallery`
- **Plan item:** Phase 4, "Dialogs, native file dialogs, notifications, system tray",
  part 1 (in-window modal dialogs)
- **Related:** [overlay](../view/overlay.md), [focus scopes](../view/focus-scope.md),
  [menus](menu.md) (focus restore), [semantics](../a11y/semantics.md),
  [gallery](../examples/gallery.md), [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Confirmations ("Delete 3 files?"), small forms and settings sheets interrupt the window
until the user answers. A modal dialog draws over a dimming barrier that blocks the rest
of the window, takes the focus and keeps Tab inside itself, closes with Escape (or a press
on the barrier, when allowed), and gives the focus back when it closes. `Dialog` provides
that frame around any content; `AlertDialog` is the common title–message–buttons layout.

## Scope

In scope: a window-covering overlay mode in the view layer; `Dialog` with a
`DialogController`; barrier dismissal and Escape; focus trapping and restore; `AlertDialog`;
dialog semantics with a modal flag; gallery samples.

Out of scope (and where it goes): native file dialogs, notifications and the system tray
(parts 2–3: they need platform dependencies and an ADR, deferred for the user); dialogs in
their own OS windows (multi-window); open/close animations; full-screen dialogs and
bottom sheets (later); blocking the window's own shortcuts while a dialog is open (they
still bubble to the root; review).

## Public API

```rust
// tantu-view
impl<V: View> OverlayPortal<V> {
    /// The overlay covers the window: laid out tight to the window at its top-left,
    /// ignoring the anchor (VIEW-OVERLAY-13).
    pub fn cover_window(self, cover: bool) -> Self;
}
pub enum Role { /* existing, */ Dialog, AlertDialog }
pub struct Semantics { /* existing fields, */ pub modal: bool }

// tantu-widgets
/// Opens and closes a dialog; `Copy`.
#[derive(Clone, Copy)]
pub struct DialogController { /* a Signal<bool> */ }
impl DialogController {
    pub fn new() -> Self;
    pub fn open(&self);
    pub fn close(&self);
    pub fn is_open(&self) -> bool; // tracked
}
impl Default for DialogController;

/// A modal dialog shown over the window while its controller is open. Place it anywhere in
/// the tree; it takes no space.
pub struct Dialog { /* private */ }
impl Dialog {
    /// `content` is built each time the dialog opens.
    pub fn new<V: View>(controller: DialogController, content: impl Fn() -> V + 'static) -> Self;
    /// Whether a press on the barrier and Escape close the dialog (default true).
    pub fn dismissible(self, dismissible: bool) -> Self;
    /// Called when the user dismisses the dialog (barrier or Escape), after it closes.
    pub fn on_dismiss(self, f: impl Fn() + 'static) -> Self;
    /// The accessible name (an `AlertDialog` names itself after its title).
    pub fn label(self, label: impl Into<String>) -> Self;
}

/// Material's basic dialog layout: a title, a message and buttons.
pub struct AlertDialog { /* private */ }
impl AlertDialog {
    pub fn new() -> Self;
    pub fn title(self, title: impl Into<String>) -> Self;
    pub fn content(self, content: impl View) -> Self;
    /// Adds a button (or any view) to the actions row, in order, aligned to the end.
    pub fn action(self, action: impl View) -> Self;
}
impl Default for AlertDialog;
```

## Behavior

- **VIEW-OVERLAY-13:** An overlay with `cover_window(true)` is laid out with tight window
  constraints at the window's top-left whatever its anchor, and follows window resizes;
  hit-testing, painting and focus are as for any overlay.
- **WIDGETS-DIALOG-01:** While its controller is open, a `Dialog` shows a barrier over the
  whole window (black at 32 %) and its content centred in it, laid out loosely within the
  window less 24 px on each side. The `Dialog` itself takes no space where it is placed.
  Closed, nothing shows; the content is built when the dialog opens and disposed when it
  closes.
- **WIDGETS-DIALOG-02:** Modality: pointer events over the barrier never reach the window's
  content (hover included). A primary press on the barrier (outside the content) closes a
  dismissible dialog and then calls `on_dismiss`; a non-dismissible dialog ignores it.
- **WIDGETS-DIALOG-03:** Focus: an opening dialog takes the focus (its content root is a
  focus scope, so Tab and Shift+Tab cycle within it, VIEW-SCOPE-02) and, when it closes,
  gives it back to the element focused before it opened if that still exists and nothing
  else has taken the focus. Escape inside a dismissible dialog closes it and calls
  `on_dismiss`.
- **WIDGETS-DIALOG-04:** `AlertDialog`: on the theme's `surface` with a 28 px radius and a
  level-3 shadow, 24 px padding, between 280 and 560 px wide (its content's natural width
  within those); the title in the theme's `title` style, 16 px above the content (in the
  `body` style when it is text), and the actions 24 px below it in a row aligned to the
  end, 8 px apart. Missing parts take no space.
- **WIDGETS-DIALOG-05:** Semantics: a dialog's content root is a `Dialog` (an `AlertDialog`
  when the content is one) with `modal` set, named by `label` or the alert's title.
- **A11Y-TREE-11:** `Role::Dialog` and `AlertDialog` map to AccessKit's roles of the same
  names, and `Semantics::modal` to AccessKit's modal flag.
- **GALLERY-33:** The Buttons section gains a "Delete…" button opening an `AlertDialog`
  ("Delete file?", a message, Cancel and Delete), with a "Dialog: …" echo (`dialog-echo`):
  "Dialog: deleted", "Dialog: cancelled" or "Dialog: dismissed". Tests: Delete, Cancel,
  Escape and a barrier press; the content behind can't be pressed while open; the focus
  returns to the button. Golden `gallery-dialog.png` with the dialog open.
- **GALLERY-34:** A "dialogs" kitchen-sink card after "bars" (nineteen `SINK_CARDS` entries,
  sample `dialog`): a "Rename…" button opening a non-dismissible dialog with a text field
  and OK, echoed by `sink-dialogs-state`.

## Performance and allocation

Dialogs build when opened and dispose when closed; nothing runs per frame while closed.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **Declarative `Dialog` + controller** (like `MenuAnchor`), not a `show_dialog` call
   returning a future: Tantu has no async runtime in the view layer.
2. **A window-covering overlay mode** (VIEW-OVERLAY-13) for the barrier, instead of a
   separate modal layer.
3. **Modality by barrier and focus scope**; window-level shortcuts still reach the root
   while a dialog is open (blocking them needs key routing changes; later).
4. **Native file dialogs, notifications and the tray** deferred (platform crates such as
   `rfd`, `notify-rust`, `tray-icon`, and an ADR).
5. **Material 3 basic-dialog metrics** (28 px radius, 24 px padding, 280–560 px).
