//! Keyboard events and focus: focusable elements, key dispatch to the focused path, Tab
//! traversal and focus changes. Spec: `docs/specs/view/focus.md`.

use std::any::TypeId;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use tantu_scene::ElementId;

use crate::shortcuts::{ActionBox, IntentBox};
use crate::{BuildCx, Handled, Phase, SingleActivator, ViewTree};

/// A registered key handler.
type KeyHandler = Rc<dyn Fn(&KeyCx<'_>) -> Handled>;
/// A registered focus-change handler.
type FocusHandler = Rc<dyn Fn(bool)>;

/// The tree's focus state: focusable elements, handlers and the focused element.
#[derive(Default)]
pub(crate) struct FocusState {
    focusables: HashMap<ElementId, FocusOptions>,
    keys: HashMap<ElementId, Vec<(Phase, KeyHandler)>>,
    changes: HashMap<ElementId, Vec<FocusHandler>>,
    pub(crate) shortcuts: HashMap<ElementId, Vec<(SingleActivator, IntentBox)>>,
    pub(crate) actions: HashMap<ElementId, Vec<(TypeId, ActionBox)>>,
    focused: Option<ElementId>,
}

impl FocusState {
    /// Forgets everything registered on `id` (VIEW-FOCUS-07).
    pub(crate) fn forget(&mut self, id: ElementId) {
        self.focusables.remove(&id);
        self.keys.remove(&id);
        self.changes.remove(&id);
        self.shortcuts.remove(&id);
        self.actions.remove(&id);
    }
}

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
        if self.tree.contains(element) {
            self.tree.focus_state.focusables.insert(element, options);
        }
    }

    /// Calls `handler` for key events in `phase` on the focused path through `element`.
    pub fn on_key(
        &mut self,
        element: ElementId,
        phase: Phase,
        handler: impl Fn(&KeyCx<'_>) -> Handled + 'static,
    ) {
        if self.tree.contains(element) {
            let handlers = self.tree.focus_state.keys.entry(element).or_default();
            handlers.push((phase, Rc::new(handler)));
        }
    }

    /// Calls `handler(true)` when `element` gains focus and `handler(false)` when it loses it.
    pub fn on_focus_change(&mut self, element: ElementId, handler: impl Fn(bool) + 'static) {
        if self.tree.contains(element) {
            let handlers = self.tree.focus_state.changes.entry(element).or_default();
            handlers.push(Rc::new(handler));
        }
    }
}

impl ViewTree {
    /// Focuses `element` if it is focusable; returns whether it did.
    pub fn focus(&mut self, element: ElementId) -> bool {
        if !self.focus_state.focusables.contains_key(&element) || !self.contains(element) {
            return false;
        }
        self.set_focus(Some(element));
        true
    }

    /// The focused element.
    pub fn focused(&self) -> Option<ElementId> {
        self.focus_state.focused
    }

    /// Removes focus.
    pub fn unfocus(&mut self) {
        self.set_focus(None);
    }

    /// Moves focus to the next traversable element in tree order, wrapping; returns the focus.
    pub fn focus_next(&mut self) -> Option<ElementId> {
        self.traverse(true)
    }

    /// Moves focus to the previous traversable element in tree order, wrapping.
    pub fn focus_previous(&mut self) -> Option<ElementId> {
        self.traverse(false)
    }

    /// Delivers `event` to the focused path (capture, then bubble); an unhandled Tab or
    /// Shift+Tab press moves focus. Returns whether anything handled it.
    pub fn dispatch_key(&mut self, event: KeyEvent) -> bool {
        let path = self.focus_path();
        let (ran, stopped) = self.deliver_key(&path, &event);
        // An unhandled Tab moves focus (VIEW-FOCUS-05).
        if !stopped && event.pressed && event.key == LogicalKey::Named(NamedKey::Tab) {
            if event.modifiers.shift {
                self.focus_previous();
            } else {
                self.focus_next();
            }
            return true;
        }
        ran
    }

    /// Moves the focus to `next`, reporting the change: the old element first (VIEW-FOCUS-02).
    fn set_focus(&mut self, next: Option<ElementId>) {
        let previous = self.focus_state.focused;
        if previous == next {
            return;
        }
        self.focus_state.focused = next;
        let handlers = |id: Option<ElementId>| -> Vec<FocusHandler> {
            id.and_then(|id| self.focus_state.changes.get(&id))
                .cloned()
                .unwrap_or_default()
        };
        let (lost, gained) = (handlers(previous), handlers(next));
        self.enter(|| {
            for handler in &lost {
                handler(false);
            }
            for handler in &gained {
                handler(true);
            }
        });
    }

    /// The next (or previous) traversable element in tree order after the focus, wrapping
    /// (VIEW-FOCUS-03).
    fn traverse(&mut self, forward: bool) -> Option<ElementId> {
        let order = self.preorder();
        let traversable = |id: &ElementId| {
            self.focus_state
                .focusables
                .get(id)
                .is_some_and(|o| o.traversable)
        };
        let current = self
            .focus_state
            .focused
            .and_then(|f| order.iter().position(|id| *id == f));
        let n = order.len();
        let candidate = (1..=n).find_map(|step| {
            let index = match (current, forward) {
                (Some(i), true) => (i + step) % n,
                (Some(i), false) => (i + n - step % n) % n,
                (None, true) => step - 1,
                (None, false) => n - step,
            };
            Some(order[index]).filter(traversable)
        });
        if let Some(next) = candidate {
            self.set_focus(Some(next));
        }
        self.focus_state.focused
    }

    /// Every element, depth first with children in order, from the root.
    fn preorder(&self) -> Vec<ElementId> {
        let mut order = Vec::new();
        let mut stack = vec![self.root()];
        while let Some(id) = stack.pop() {
            order.push(id);
            stack.extend(self.children(id).iter().rev());
        }
        order
    }

    /// The focused element and its ancestors, or the root alone, innermost first.
    pub(crate) fn focus_path(&self) -> Vec<ElementId> {
        match self.focus_state.focused {
            Some(focused) => self.path_to(focused),
            None => vec![self.root()],
        }
    }

    /// `id` and its ancestors, innermost first.
    fn path_to(&self, id: ElementId) -> Vec<ElementId> {
        let mut path = vec![id];
        let mut next = self.parent(id);
        while let Some(parent) = next {
            path.push(parent);
            next = self.parent(parent);
        }
        path
    }

    /// Runs capture handlers root to target, then bubble handlers back (VIEW-FOCUS-04); returns
    /// (whether any ran, whether one stopped).
    fn deliver_key(&self, path: &[ElementId], event: &KeyEvent) -> (bool, bool) {
        let capture = path.iter().rev().map(|id| (*id, Phase::Capture));
        let bubble = path.iter().map(|id| (*id, Phase::Bubble));
        let (mut ran, mut stopped) = (false, false);
        self.enter(|| {
            for (id, phase) in capture.chain(bubble) {
                // Shortcuts on this element, before its bubble handlers (VIEW-SHORT-02).
                if phase == Phase::Bubble && event.pressed && self.try_shortcuts(id, path, event) {
                    ran = true;
                    stopped = true;
                    return;
                }
                let Some(handlers) = self.focus_state.keys.get(&id) else {
                    continue;
                };
                let cx = KeyCx {
                    event,
                    element: id,
                    phase,
                };
                for (_, handler) in handlers.iter().filter(|(p, _)| *p == phase) {
                    ran = true;
                    if handler(&cx) == Handled::Stop {
                        stopped = true;
                        return;
                    }
                }
            }
        });
        (ran, stopped)
    }

    /// Focuses the innermost element on a pressed path that asks for it (VIEW-FOCUS-06).
    pub(crate) fn focus_on_press(&mut self, path: &[ElementId]) {
        let target = path.iter().copied().find(|id| {
            self.focus_state
                .focusables
                .get(id)
                .is_some_and(|o| o.focus_on_press)
        });
        if let Some(target) = target {
            self.set_focus(Some(target));
        }
    }

    /// Clears the focus if it is `id` or inside it, before `id` is removed (VIEW-FOCUS-07).
    pub(crate) fn release_focus_within(&mut self, id: ElementId) {
        if let Some(focused) = self.focus_state.focused {
            if self.path_to(focused).contains(&id) {
                self.set_focus(None);
            }
        }
    }
}
