//! [`Checkbox`], [`Radio`] and [`Switch`]. Spec: `docs/specs/widgets/toggles.md`.

use std::rc::Rc;

use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

/// A box that is checked or not (Flutter's `Checkbox`, Material 3 look).
///
/// The checkbox is controlled: it shows `value` and reports `!value` through `on_change`
/// when activated; the app writes its signal and the box follows at the next frame.
pub struct Checkbox {
    value: Prop<bool>,
    indeterminate: Prop<bool>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(bool)>>,
}

impl Checkbox {
    /// A checkbox showing `value`, enabled, reporting nothing (so drawn disabled until
    /// `on_change` is set).
    pub fn new(value: impl IntoProp<bool>) -> Self {
        Checkbox {
            value: value.into_prop(),
            indeterminate: Prop::Value(false),
            enabled: Prop::Value(true),
            on_change: None,
        }
    }

    /// Shows a dash instead of the check or the empty box while true (a partial selection).
    pub fn indeterminate(mut self, indeterminate: impl IntoProp<bool>) -> Self {
        self.indeterminate = indeterminate.into_prop();
        self
    }

    /// A disabled checkbox ignores input and draws dimmed. No `on_change` disables it too.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with `!value` when the user activates the checkbox.
    pub fn on_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl View for Checkbox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.value,
            self.indeterminate,
            self.enabled,
            self.on_change,
        );
        todo!("WIDGETS-TOGGLE-01")
    }
}

/// One choice of a group, selected while the group's value equals its own (Flutter's
/// `Radio<T>`, Material 3 look). Controlled: `on_change` reports this radio's value.
pub struct Radio<T> {
    value: T,
    group: Prop<T>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(T)>>,
}

impl<T: Copy + PartialEq + 'static> Radio<T> {
    /// A radio for `value`, selected while `group` (a closure or a signal) equals it.
    pub fn new(value: T, group: impl IntoProp<T>) -> Self {
        Radio {
            value,
            group: group.into_prop(),
            enabled: Prop::Value(true),
            on_change: None,
        }
    }

    /// A disabled radio ignores input and draws dimmed. No `on_change` disables it too.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with this radio's `value` when the user activates it.
    pub fn on_change(mut self, f: impl Fn(T) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl<T: Copy + PartialEq + 'static> View for Radio<T> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.value, self.group, self.enabled, self.on_change);
        todo!("WIDGETS-TOGGLE-01")
    }
}

/// An on/off switch (Flutter's `Switch`, Material 3 look). Controlled: `on_change` reports
/// `!value`.
pub struct Switch {
    value: Prop<bool>,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(bool)>>,
}

impl Switch {
    /// A switch showing `value`, enabled, reporting nothing (so drawn disabled until
    /// `on_change` is set).
    pub fn new(value: impl IntoProp<bool>) -> Self {
        Switch {
            value: value.into_prop(),
            enabled: Prop::Value(true),
            on_change: None,
        }
    }

    /// A disabled switch ignores input and draws dimmed. No `on_change` disables it too.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with `!value` when the user activates the switch.
    pub fn on_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl View for Switch {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.value, self.enabled, self.on_change);
        todo!("WIDGETS-TOGGLE-01")
    }
}
