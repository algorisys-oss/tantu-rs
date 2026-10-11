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
