//! Shortcuts and commands: [`SingleActivator`], [`Shortcuts`], [`Actions`] and
//! [`ViewTree::invoke`] (Flutter's intents, shortcuts and actions). Spec:
//! `docs/specs/view/shortcuts.md`.

use std::any::Any;
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
        todo!()
    }

    /// Whether a key press matches.
    pub fn accepts(&self, event: &KeyEvent) -> bool {
        let _ = event;
        todo!()
    }
}

/// An intent value, type-erased.
type IntentBox = Rc<dyn Any>;
/// An action, type-erased: runs when given an intent of its type.
type ActionBox = Rc<dyn Fn(&dyn Any)>;

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
        let _ = (self.child, self.bindings, cx);
        todo!()
    }
}

/// Handles intents of given types for focus inside its child (and `invoke` from it).
pub struct Actions<V> {
    child: V,
    handlers: Vec<(std::any::TypeId, ActionBox)>,
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
        self.handlers.push((std::any::TypeId::of::<I>(), action));
        self
    }
}

impl<V: View> View for Actions<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.child, self.handlers, cx);
        todo!()
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
        let _ = (element, activator, intent);
        todo!()
    }

    /// Handles intents of type `I` on `element` (what `Actions` uses).
    pub fn action<I: Any>(&mut self, element: ElementId, handler: impl Fn(&I) + 'static) {
        let _ = (element, handler);
        todo!()
    }
}

impl ViewTree {
    /// Invokes `intent` from the focused element (the root when nothing is focused): the
    /// nearest action for `I` on the path up runs. Returns whether one did.
    pub fn invoke<I: Any>(&mut self, intent: &I) -> bool {
        let _ = intent;
        todo!()
    }
}
