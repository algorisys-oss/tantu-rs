//! Keyboard events and focus: focusable elements, key dispatch to the focused path, Tab
//! traversal and focus changes. Spec: `docs/specs/view/focus.md`.

use std::sync::Arc;

use tantu_scene::ElementId;

use crate::{BuildCx, Handled, Phase, ViewTree};

/// A key that isn't a character.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedKey {
    /// Enter (Return).
    Enter,
    /// Tab.
    Tab,
    /// Escape.
    Escape,
    /// Backspace.
    Backspace,
    /// Delete (forward delete).
    Delete,
    /// Insert.
    Insert,
    /// Left arrow.
    ArrowLeft,
    /// Right arrow.
    ArrowRight,
    /// Up arrow.
    ArrowUp,
    /// Down arrow.
    ArrowDown,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// A function key, F1 to F24.
    F(u8),
    /// Any other named key (context menu, print screen, ...).
    Other,
}

/// What a key means (Flutter's `LogicalKeyboardKey`, simplified). Space is `Character(" ")`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalKey {
    /// A named key.
    Named(NamedKey),
    /// A key that types a character, as the layout produces it.
    Character(Arc<str>),
}

/// Modifier keys held during a key event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Shift.
    pub shift: bool,
    /// Control.
    pub control: bool,
    /// Alt (Option on macOS).
    pub alt: bool,
    /// Super (Windows key, Command on macOS).
    pub super_key: bool,
}

/// A key press or release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key.
    pub key: LogicalKey,
    /// True for a press, false for a release.
    pub pressed: bool,
    /// True for automatic repeats while the key is held.
    pub repeat: bool,
    /// The modifiers held.
    pub modifiers: Modifiers,
    /// Text the key produced, if any (until IME).
    pub text: Option<Arc<str>>,
}

impl KeyEvent {
    /// A press of `key` with `modifiers`, no repeat, no text.
    pub fn press(key: LogicalKey, modifiers: Modifiers) -> Self {
        KeyEvent {
            key,
            pressed: true,
            repeat: false,
            modifiers,
            text: None,
        }
    }

    /// A release of `key` with `modifiers`.
    pub fn release(key: LogicalKey, modifiers: Modifiers) -> Self {
        KeyEvent {
            pressed: false,
            ..KeyEvent::press(key, modifiers)
        }
    }
}

/// How an element takes part in focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusOptions {
    /// Tab and Shift+Tab stop here.
    pub traversable: bool,
    /// A primary press on the element focuses it.
    pub focus_on_press: bool,
}

impl Default for FocusOptions {
    /// Traversable, not focused on press.
    fn default() -> Self {
        FocusOptions {
            traversable: true,
            focus_on_press: false,
        }
    }
}

/// What a key handler receives.
pub struct KeyCx<'a> {
    /// The event.
    pub event: &'a KeyEvent,
    /// The element the handler belongs to.
    pub element: ElementId,
    /// The phase it is delivered in.
    pub phase: Phase,
}

impl BuildCx<'_> {
    /// Makes `element` focusable.
    pub fn focusable(&mut self, element: ElementId, options: FocusOptions) {
        let _ = (element, options);
        todo!()
    }

    /// Calls `handler` for key events in `phase` on the focused path through `element`.
    pub fn on_key(
        &mut self,
        element: ElementId,
        phase: Phase,
        handler: impl Fn(&KeyCx<'_>) -> Handled + 'static,
    ) {
        let _ = (element, phase, handler);
        todo!()
    }

    /// Calls `handler(true)` when `element` gains focus and `handler(false)` when it loses it.
    pub fn on_focus_change(&mut self, element: ElementId, handler: impl Fn(bool) + 'static) {
        let _ = (element, handler);
        todo!()
    }
}

impl ViewTree {
    /// Focuses `element` if it is focusable; returns whether it did.
    pub fn focus(&mut self, element: ElementId) -> bool {
        let _ = element;
        todo!()
    }

    /// The focused element.
    pub fn focused(&self) -> Option<ElementId> {
        todo!()
    }

    /// Removes focus.
    pub fn unfocus(&mut self) {
        todo!()
    }

    /// Moves focus to the next traversable element in tree order, wrapping; returns the focus.
    pub fn focus_next(&mut self) -> Option<ElementId> {
        todo!()
    }

    /// Moves focus to the previous traversable element in tree order, wrapping.
    pub fn focus_previous(&mut self) -> Option<ElementId> {
        todo!()
    }

    /// Delivers `event` to the focused path (capture, then bubble); an unhandled Tab or
    /// Shift+Tab press moves focus. Returns whether anything handled it.
    pub fn dispatch_key(&mut self, event: KeyEvent) -> bool {
        let _ = event;
        todo!()
    }
}
