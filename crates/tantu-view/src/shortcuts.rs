//! Shortcuts and commands: [`SingleActivator`], [`Shortcuts`], [`Actions`] and
//! [`ViewTree::invoke`] (Flutter's intents, shortcuts and actions). Spec:
//! `docs/specs/view/shortcuts.md`.

use std::any::{Any, TypeId};
use std::rc::Rc;

use tantu_scene::ElementId;

use crate::{BuildCx, KeyEvent, LogicalKey, View, ViewTree};

/// A key with exact modifiers (Flutter's `SingleActivator`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SingleActivator {
    key: LogicalKey,
    control: bool,
    shift: bool,
    alt: bool,
    super_key: bool,
}

impl SingleActivator {
    /// `key` with no modifiers.
    pub fn new(key: LogicalKey) -> Self {
        SingleActivator {
            key,
            control: false,
            shift: false,
            alt: false,
            super_key: false,
        }
    }

    /// `SingleActivator::new(LogicalKey::Character(c))`.
    pub fn character(c: &str) -> Self {
        SingleActivator::new(LogicalKey::Character(c.into()))
    }

    /// With Control held.
    pub fn control(self) -> Self {
        SingleActivator {
            control: true,
            ..self
        }
    }

    /// With Shift held.
    pub fn shift(self) -> Self {
        SingleActivator {
            shift: true,
            ..self
        }
    }

    /// With Alt held.
    pub fn alt(self) -> Self {
        SingleActivator { alt: true, ..self }
    }

    /// With Super (Windows key, Command) held.
    pub fn super_key(self) -> Self {
        SingleActivator {
            super_key: true,
            ..self
        }
    }

    /// With the platform's primary modifier: Command (`super_key`) on macOS, Control elsewhere.
    pub fn primary(self) -> Self {
        if cfg!(target_os = "macos") {
            self.super_key()
        } else {
            self.control()
        }
    }

    /// Whether a key press matches.
    pub fn accepts(&self, event: &KeyEvent) -> bool {
        let modifiers = event.modifiers;
        let key_matches = match (&self.key, &event.key) {
            (LogicalKey::Character(a), LogicalKey::Character(b)) => {
                a.to_lowercase() == b.to_lowercase()
            }
            (a, b) => a == b,
        };
        event.pressed
            && key_matches
            && (
                modifiers.control,
                modifiers.shift,
                modifiers.alt,
                modifiers.super_key,
            ) == (self.control, self.shift, self.alt, self.super_key)
    }
}

/// An intent value, type-erased.
pub(crate) type IntentBox = Rc<dyn Any>;
/// An action, type-erased: runs when given an intent of its type.
pub(crate) type ActionBox = Rc<dyn Fn(&dyn Any)>;

/// Binds activators to intents for key events on the focused path through its child.
pub struct Shortcuts<V> {
    child: V,
    bindings: Vec<(SingleActivator, IntentBox)>,
}

impl<V: View> Shortcuts<V> {
    /// Shortcuts around `child`, with no bindings yet.
    pub fn new(child: V) -> Self {
        Shortcuts {
            child,
            bindings: Vec::new(),
        }
    }

    /// When `activator` matches, `intent` is invoked.
    pub fn bind<I: Any + Clone>(mut self, activator: SingleActivator, intent: I) -> Self {
        self.bindings.push((activator, Rc::new(intent)));
        self
    }
}

impl<V: View> View for Shortcuts<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.child.build(cx);
        for (activator, intent) in self.bindings {
            cx.shortcut_box(id, activator, intent);
        }
        id
    }
}

/// Handles intents of given types for focus inside its child (and `invoke` from it).
pub struct Actions<V> {
    child: V,
    handlers: Vec<(TypeId, ActionBox)>,
}

impl<V: View> Actions<V> {
    /// Actions around `child`, with no handlers yet.
    pub fn new(child: V) -> Self {
        Actions {
            child,
            handlers: Vec::new(),
        }
    }

    /// Runs `handler` for intents of type `I`.
    pub fn on<I: Any>(mut self, handler: impl Fn(&I) + 'static) -> Self {
        let action: ActionBox = Rc::new(move |intent: &dyn Any| {
            if let Some(intent) = intent.downcast_ref::<I>() {
                handler(intent);
            }
        });
        self.handlers.push((TypeId::of::<I>(), action));
        self
    }
}

impl<V: View> View for Actions<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.child.build(cx);
        for (type_id, action) in self.handlers {
            cx.action_box(id, type_id, action);
        }
        id
    }
}

impl BuildCx<'_> {
    /// Binds `activator` to `intent` on `element` (what `Shortcuts` uses).
    pub fn shortcut<I: Any + Clone>(
        &mut self,
        element: ElementId,
        activator: SingleActivator,
        intent: I,
    ) {
        self.shortcut_box(element, activator, Rc::new(intent));
    }

    /// Handles intents of type `I` on `element` (what `Actions` uses).
    pub fn action<I: Any>(&mut self, element: ElementId, handler: impl Fn(&I) + 'static) {
        let action: ActionBox = Rc::new(move |intent: &dyn Any| {
            if let Some(intent) = intent.downcast_ref::<I>() {
                handler(intent);
            }
        });
        self.action_box(element, TypeId::of::<I>(), action);
    }
}

impl ViewTree {
    /// Invokes `intent` from the focused element (the root when nothing is focused): the
    /// nearest action for `I` on the path up runs. Returns whether one did.
    pub fn invoke<I: Any>(&mut self, intent: &I) -> bool {
        let path = self.focus_path();
        self.invoke_on(&path, intent)
    }
}

impl BuildCx<'_> {
    /// Registers a type-erased binding on `element`.
    fn shortcut_box(&mut self, element: ElementId, activator: SingleActivator, intent: IntentBox) {
        if self.tree.contains(element) {
            let bindings = self.tree.focus_state.shortcuts.entry(element).or_default();
            bindings.push((activator, intent));
        }
    }

    /// Registers a type-erased action on `element`.
    fn action_box(&mut self, element: ElementId, type_id: TypeId, action: ActionBox) {
        if self.tree.contains(element) {
            let actions = self.tree.focus_state.actions.entry(element).or_default();
            actions.push((type_id, action));
        }
    }
}

impl ViewTree {
    /// Runs the nearest action for `intent`'s type on `path` (innermost first), with the
    /// runtime current (VIEW-SHORT-05). Returns whether one ran.
    pub(crate) fn invoke_on(&self, path: &[ElementId], intent: &dyn Any) -> bool {
        let type_id = intent.type_id();
        let action = path.iter().find_map(|id| {
            self.focus_state
                .actions
                .get(id)?
                .iter()
                .find(|(t, _)| *t == type_id)
                .map(|(_, action)| Rc::clone(action))
        });
        match action {
            Some(action) => {
                self.enter(|| action(intent));
                true
            }
            None => false,
        }
    }

    /// Tries `element`'s bindings for `event`, innermost binding first; true if one invoked an
    /// action (VIEW-SHORT-02..04).
    pub(crate) fn try_shortcuts(
        &self,
        element: ElementId,
        path: &[ElementId],
        event: &KeyEvent,
    ) -> bool {
        let Some(bindings) = self.focus_state.shortcuts.get(&element) else {
            return false;
        };
        bindings
            .iter()
            .filter(|(activator, _)| activator.accepts(event))
            .any(|(_, intent)| self.invoke_on(path, intent.as_ref()))
    }
}
