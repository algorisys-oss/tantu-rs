//! [`Draggable`] and [`DragTarget`]: drag and drop inside the app. Spec:
//! `docs/specs/widgets/drag-and-drop.md` (WIDGETS-DND-01..02).

use std::marker::PhantomData;
use std::rc::Rc;

use tantu_view::reactive::Signal;
use tantu_view::{AnyView, BuildCx, ElementId, View};

/// Decides whether a target accepts hovering data.
type WillAccept<T> = Rc<dyn Fn(&T) -> bool>;

/// Builds a view on demand.
type ViewFn = Rc<dyn Fn() -> AnyView>;

/// A child the user can drag, carrying `data` (Flutter's `Draggable`, WIDGETS-DND-01).
pub struct Draggable<T> {
    data: T,
    child: AnyView,
    feedback: Option<ViewFn>,
    child_when_dragging: Option<ViewFn>,
    on_drag_started: Option<Rc<dyn Fn()>>,
    on_drag_end: Option<Rc<dyn Fn(bool)>>,
}

impl<T: Clone + 'static> Draggable<T> {
    /// `child`, draggable, carrying `data`.
    pub fn new(data: T, child: impl View) -> Self {
        Draggable {
            data,
            child: AnyView::new(child),
            feedback: None,
            child_when_dragging: None,
            on_drag_started: None,
            on_drag_end: None,
        }
    }

    /// What follows the pointer while dragging (default: nothing).
    pub fn feedback<V: View>(mut self, feedback: impl Fn() -> V + 'static) -> Self {
        self.feedback = Some(Rc::new(move || AnyView::new(feedback())));
        self
    }

    /// What stays in the child's place while dragging (default: the child).
    pub fn child_when_dragging<V: View>(mut self, view: impl Fn() -> V + 'static) -> Self {
        self.child_when_dragging = Some(Rc::new(move || AnyView::new(view())));
        self
    }

    /// Called when a drag starts.
    pub fn on_drag_started(mut self, f: impl Fn() + 'static) -> Self {
        self.on_drag_started = Some(Rc::new(f));
        self
    }

    /// Called when the drag ends, with whether a target accepted the data.
    pub fn on_drag_end(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_drag_end = Some(Rc::new(f));
        self
    }
}

impl<T: Clone + 'static> View for Draggable<T> {
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.data,
            self.child,
            self.feedback,
            self.child_when_dragging,
            self.on_drag_started,
            self.on_drag_end,
        );
        todo!("WIDGETS-DND-01")
    }
}

/// A region that may accept dragged data of type `T` (Flutter's `DragTarget`,
/// WIDGETS-DND-02).
pub struct DragTarget<T> {
    child: AnyView,
    will_accept: Option<WillAccept<T>>,
    on_accept: Option<Rc<dyn Fn(T)>>,
    candidate: Option<Signal<bool>>,
    _data: PhantomData<T>,
}

impl<T: Clone + 'static> DragTarget<T> {
    /// `child` as a drop target.
    pub fn new(child: impl View) -> Self {
        DragTarget {
            child: AnyView::new(child),
            will_accept: None,
            on_accept: None,
            candidate: None,
            _data: PhantomData,
        }
    }

    /// Whether to accept the data hovering over it (default: always).
    pub fn on_will_accept(mut self, f: impl Fn(&T) -> bool + 'static) -> Self {
        self.will_accept = Some(Rc::new(f));
        self
    }

    /// Called with the data dropped on it.
    pub fn on_accept(mut self, f: impl Fn(T) + 'static) -> Self {
        self.on_accept = Some(Rc::new(f));
        self
    }

    /// Set to true while acceptable data hovers over it, false otherwise.
    pub fn candidate(mut self, signal: Signal<bool>) -> Self {
        self.candidate = Some(signal);
        self
    }
}

impl<T: Clone + 'static> View for DragTarget<T> {
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.child, self.will_accept, self.on_accept, self.candidate);
        todo!("WIDGETS-DND-02")
    }
}
