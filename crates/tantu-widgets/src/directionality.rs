//! [`Directionality`] and [`PositionedDirectional`]: right-to-left layout. Spec:
//! `docs/specs/widgets/directionality.md`.

use std::rc::Rc;

use tantu_view::layout::{StackParentData, TextDirection};
use tantu_view::reactive::memo;
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Prop, Provide, View};

/// The direction a [`Directionality`] provides: a tracked read.
#[derive(Clone)]
struct Direction(Rc<dyn Fn() -> TextDirection>);

/// The nearest direction as a prop: dynamic under a [`Directionality`], `Ltr` otherwise
/// (WIDGETS-DIR-02, -03).
pub(crate) fn direction_prop(cx: &BuildCx<'_>) -> Prop<TextDirection> {
    match cx.inherit::<Direction>() {
        Some(Direction(read)) => Prop::Dynamic(Box::new(move || read())),
        None => Prop::Value(TextDirection::Ltr),
    }
}

/// `value` resolved against the nearest direction, as a prop following both.
pub(crate) fn resolved<T: Copy + 'static, R: 'static>(
    cx: &BuildCx<'_>,
    value: Prop<T>,
    resolve: fn(T, TextDirection) -> R,
) -> Prop<R> {
    match (value, direction_prop(cx)) {
        (Prop::Value(v), Prop::Value(d)) => Prop::Value(resolve(v, d)),
        (value, direction) => {
            let (value, direction) = (crate::util::dynamic(value), crate::util::dynamic(direction));
            Prop::Dynamic(Box::new(move || resolve(value(), direction())))
        }
    }
}

/// Gives its subtree a text direction (Flutter's `Directionality`, WIDGETS-DIR-01).
pub struct Directionality {
    direction: Prop<TextDirection>,
    child: AnyView,
}

impl Directionality {
    /// Lays `child` out in `direction` (a value, closure or signal).
    pub fn new(direction: impl IntoProp<TextDirection>, child: impl View) -> Self {
        Directionality {
            direction: direction.into_prop(),
            child: AnyView::new(child),
        }
    }

    /// The nearest direction above the element being built (`Ltr` without one); a tracked
    /// read.
    pub fn of(cx: &BuildCx<'_>) -> TextDirection {
        cx.inherit::<Direction>()
            .map_or(TextDirection::Ltr, |Direction(read)| read())
    }
}

impl View for Directionality {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let read: Rc<dyn Fn() -> TextDirection> = match self.direction {
            Prop::Value(direction) => Rc::new(move || direction),
            // A memo, so readers rerun only when the direction actually changes.
            Prop::Dynamic(f) => {
                let direction = memo(f);
                Rc::new(move || direction.get())
            }
        };
        Provide::new(Direction(read), self.child).build(cx)
    }
}

/// [`Positioned`](crate::Positioned) with start and end instead of left and right
/// (Flutter's `PositionedDirectional`, WIDGETS-DIR-03).
pub struct PositionedDirectional {
    child: AnyView,
    start: Option<f32>,
    top: Option<f32>,
    end: Option<f32>,
    bottom: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
}

impl PositionedDirectional {
    /// Wraps `child` with no edges set.
    pub fn new(child: impl View) -> Self {
        PositionedDirectional {
            child: AnyView::new(child),
            start: None,
            top: None,
            end: None,
            bottom: None,
            width: None,
            height: None,
        }
    }

    /// Distance from the stack's start edge.
    pub fn start(mut self, value: f32) -> Self {
        self.start = Some(value);
        self
    }

    /// Distance from the stack's top edge.
    pub fn top(mut self, value: f32) -> Self {
        self.top = Some(value);
        self
    }

    /// Distance from the stack's end edge.
    pub fn end(mut self, value: f32) -> Self {
        self.end = Some(value);
        self
    }

    /// Distance from the stack's bottom edge.
    pub fn bottom(mut self, value: f32) -> Self {
        self.bottom = Some(value);
        self
    }

    /// The child's width.
    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(value);
        self
    }

    /// The child's height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(value);
        self
    }
}

impl View for PositionedDirectional {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let direction = direction_prop(cx);
        let id = self.child.build(cx);
        let (start, top, end, bottom, width, height) = (
            self.start,
            self.top,
            self.end,
            self.bottom,
            self.width,
            self.height,
        );
        let data = move |direction| {
            let (left, right) = match direction {
                TextDirection::Ltr => (start, end),
                TextDirection::Rtl => (end, start),
            };
            StackParentData {
                left,
                top,
                right,
                bottom,
                width,
                height,
            }
        };
        if !cx.set_parent_data(id, data(TextDirection::Ltr)) {
            tracing::warn!(
                "PositionedDirectional around a dynamic region has no element to hold its \
                 position; the child is laid out as non-positioned"
            );
            return id;
        }
        cx.bind(id, direction, move |e, direction| {
            e.set_parent_data(data(direction));
        });
        id
    }
}
