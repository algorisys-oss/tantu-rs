//! Small helpers shared by the widgets.

use std::rc::Rc;

use tantu_view::Prop;

/// A prop as a function of its current value (a constant for a value).
pub(crate) fn dynamic<T: Copy + 'static>(prop: Prop<T>) -> Rc<dyn Fn() -> T> {
    match prop {
        Prop::Value(value) => Rc::new(move || value),
        Prop::Dynamic(f) => Rc::from(f),
    }
}

/// A prop of a clonable value as a shared function (a clone per call for a value).
pub(crate) fn shared<T: Clone + 'static>(prop: Prop<T>) -> Rc<dyn Fn() -> T> {
    match prop {
        Prop::Value(value) => Rc::new(move || value.clone()),
        Prop::Dynamic(f) => Rc::from(f),
    }
}

/// `disabled` semantics from an `enabled` function, as a prop.
pub(crate) fn disabled_prop(enabled: &Rc<dyn Fn() -> bool>) -> Prop<bool> {
    let enabled = Rc::clone(enabled);
    Prop::Dynamic(Box::new(move || !enabled()))
}
