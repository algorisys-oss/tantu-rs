//! [`Prop`] and [`IntoProp`]: widget properties that are fixed values or follow signals.
//! Spec: `docs/specs/view/frame.md`.

use std::sync::Arc;

use tantu_core::{Color, EdgeInsets, Point, Rect, Size, Vec2};
use tantu_layout::{
    Alignment, Axis, CrossAxisAlignment, FlexFit, MainAxisAlignment, MainAxisSize, StackFit,
    TextStyleKey, TextWidthBasis, WrapAlignment, WrapCrossAlignment,
};
use tantu_reactive::{Memo, Signal};

/// A property value: fixed, or computed (re-read when the signals it reads change).
pub enum Prop<T> {
    /// A fixed value.
    Value(T),
    /// A closure, called again when the signals it read change.
    Dynamic(Box<dyn Fn() -> T>),
}

impl<T> Prop<T> {
    /// The current value (calls the closure for `Dynamic`, tracking its signals if inside an
    /// effect).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        todo!()
    }

    /// True for `Dynamic`.
    pub fn is_dynamic(&self) -> bool {
        todo!()
    }
}

impl<T> std::fmt::Debug for Prop<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Prop::Value(_) => "Prop::Value(..)",
            Prop::Dynamic(_) => "Prop::Dynamic(..)",
        })
    }
}

/// What a widget setter accepts: a value, a closure returning one, or a signal or memo.
pub trait IntoProp<T> {
    /// The prop.
    fn into_prop(self) -> Prop<T>;
}

impl<T, F: Fn() -> T + 'static> IntoProp<T> for F {
    fn into_prop(self) -> Prop<T> {
        todo!()
    }
}

impl<T: Clone + 'static> IntoProp<T> for Signal<T> {
    fn into_prop(self) -> Prop<T> {
        todo!()
    }
}

impl<T: Clone + PartialEq + 'static> IntoProp<T> for Memo<T> {
    fn into_prop(self) -> Prop<T> {
        todo!()
    }
}

impl<T> IntoProp<T> for Prop<T> {
    fn into_prop(self) -> Prop<T> {
        todo!()
    }
}

/// `IntoProp<T> for T` for value types (a blanket impl would conflict with the closure impl).
macro_rules! value_props {
    ($($ty:ty),* $(,)?) => {
        $(
            impl IntoProp<$ty> for $ty {
                fn into_prop(self) -> Prop<$ty> {
                    todo!()
                }
            }
        )*
    };
}

value_props!(
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
    String,
    Arc<str>,
    Color,
    Size,
    Point,
    Vec2,
    Rect,
    EdgeInsets,
    Alignment,
    Axis,
    MainAxisAlignment,
    MainAxisSize,
    CrossAxisAlignment,
    FlexFit,
    StackFit,
    WrapAlignment,
    WrapCrossAlignment,
    TextWidthBasis,
    TextStyleKey,
);

impl IntoProp<Arc<str>> for &'static str {
    fn into_prop(self) -> Prop<Arc<str>> {
        todo!()
    }
}

impl IntoProp<String> for &'static str {
    fn into_prop(self) -> Prop<String> {
        todo!()
    }
}
