//! Binding props to elements, the update queue and [`ViewTree::frame`] (ADR 0011, point 6).
//! Spec: `docs/specs/view/frame.md`.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_layout::{BoxConstraints, RenderBox};
use tantu_reactive::effect;
use tantu_scene::{ElementId, Scene};

use crate::tree::{Kind, arena_id};
use crate::{BuildCx, Paint, Prop, TextContext, ViewTree};

/// A queued change: applies itself to the tree and says whether it applied a value.
type Update = Box<dyn FnOnce(&mut ViewTree) -> bool>;

/// What effects share with their tree: the update queue and the frame request state.
#[derive(Default)]
pub(crate) struct Shared {
    queue: RefCell<Vec<Update>>,
    needs_frame: Cell<bool>,
    requester: RefCell<Option<Rc<dyn Fn()>>>,
}

impl Shared {
    /// Queues `update` and requests a frame if none was needed yet (VIEW-FRAME-04).
    pub(crate) fn push(&self, update: Update) {
        self.queue.borrow_mut().push(update);
        if !self.needs_frame.replace(true) {
            let requester = self.requester.borrow().clone();
            if let Some(requester) = requester {
                requester();
            }
        }
    }
}

/// Mutable access to one element while applying a prop.
pub struct ElementMut<'a> {
    pub(crate) tree: &'a mut ViewTree,
    pub(crate) id: ElementId,
}

impl ElementMut<'_> {
    /// The element.
    pub fn id(&self) -> ElementId {
        self.id
    }

    /// The element's layout object, if it is an `R`.
    pub fn render<R: RenderBox>(&self) -> Option<&R> {
        let layout = self.tree.layout_id(self.id)?;
        self.tree.layout.get::<R>(layout)
    }

    /// Changes a copy of the layout object with `f` and stores it with `LayoutTree::set`, so
    /// the node is re-laid out only if it changed. Returns whether it changed (false if the
    /// element isn't a render element with an `R`).
    pub fn update_render<R: RenderBox + Clone + PartialEq>(
        &mut self,
        f: impl FnOnce(&mut R),
    ) -> bool {
        let Some(layout) = self.tree.layout_id(self.id) else {
            return false;
        };
        let Some(mut copy) = self.tree.layout.get::<R>(layout).cloned() else {
            return false;
        };
        f(&mut copy);
        self.tree.layout.set(layout, copy)
    }

    /// The element's paint behavior, if it is a `P`.
    pub fn paint<P: Paint>(&self) -> Option<&P> {
        let element = self.tree.elements.get(arena_id(self.id)?)?;
        match &element.kind {
            Kind::Render(_, paint) => (paint.as_ref() as &dyn Any).downcast_ref::<P>(),
            Kind::Region => None,
        }
    }

    /// Changes the paint behavior in place. Returns false if it isn't a `P`.
    pub fn update_paint<P: Paint>(&mut self, f: impl FnOnce(&mut P)) -> bool {
        let Some(element) = arena_id(self.id).and_then(|a| self.tree.elements.get_mut(a)) else {
            return false;
        };
        let Kind::Render(_, paint) = &mut element.kind else {
            return false;
        };
        match (paint.as_mut() as &mut dyn Any).downcast_mut::<P>() {
            Some(paint) => {
                f(paint);
                true
            }
            None => false,
        }
    }
}

impl BuildCx<'_> {
    /// Connects `prop` to `element`: calls `apply` with its value now, and for a dynamic prop,
    /// again at the next frame after the signals it reads change (with the latest value).
    pub fn bind<T: 'static>(
        &mut self,
        element: ElementId,
        prop: Prop<T>,
        apply: impl Fn(&mut ElementMut<'_>, T) + 'static,
    ) {
        let f = match prop {
            Prop::Value(value) => {
                apply(
                    &mut ElementMut {
                        tree: &mut *self.tree,
                        id: element,
                    },
                    value,
                );
                return;
            }
            Prop::Dynamic(f) => f,
        };
        let Some(scope) = self.tree.scope(element) else {
            return;
        };
        let apply = Rc::new(apply);
        // The first run's value is applied right here; later values wait in `slot` for a frame
        // (VIEW-FRAME-03), one queue entry per binding until it is applied.
        let initial: Rc<RefCell<Option<T>>> = Rc::default();
        let slot: Rc<RefCell<Option<T>>> = Rc::default();
        let first = Cell::new(true);
        let shared = Rc::clone(&self.tree.shared);
        {
            let (initial, apply) = (Rc::clone(&initial), Rc::clone(&apply));
            scope.run(|| {
                effect(move || {
                    let value = f();
                    if first.replace(false) {
                        *initial.borrow_mut() = Some(value);
                        return;
                    }
                    let was_empty = slot.borrow_mut().replace(value).is_none();
                    if was_empty {
                        let (slot, apply) = (Rc::clone(&slot), Rc::clone(&apply));
                        shared.push(Box::new(move |tree: &mut ViewTree| {
                            let Some(value) = slot.borrow_mut().take() else {
                                return false;
                            };
                            if !tree.contains(element) {
                                return false;
                            }
                            apply(&mut ElementMut { tree, id: element }, value);
                            true
                        }));
                    }
                });
            });
        }
        let value = initial.borrow_mut().take();
        if let Some(value) = value {
            apply(
                &mut ElementMut {
                    tree: &mut *self.tree,
                    id: element,
                },
                value,
            );
        }
    }
}

/// What a frame did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameReport {
    /// Prop values applied.
    pub applied: usize,
}

impl ViewTree {
    /// Applies the queued updates present now (not ones they queue) inside the runtime;
    /// returns how many applied something.
    fn apply_queue(&mut self) -> usize {
        let pending = std::mem::take(&mut *self.shared.queue.borrow_mut());
        let mut applied = 0;
        let runtime = Rc::clone(&self.runtime);
        runtime.enter(|| {
            for update in pending {
                applied += usize::from(update(self));
            }
        });
        applied
    }

    /// Applies updates queued by the tree itself during layout (`LayoutBuilder` rebuilds),
    /// leaving the frame request as it was when nothing else is pending.
    pub(crate) fn apply_internal_updates(&mut self) {
        let requested = self.shared.needs_frame.get();
        self.apply_queue();
        if !requested && self.shared.queue.borrow().is_empty() {
            self.shared.needs_frame.set(false);
        }
    }

    /// True when something changed since the last frame.
    pub fn needs_frame(&self) -> bool {
        self.shared.needs_frame.get()
    }

    /// Called once each time the tree goes from not needing a frame to needing one (the app
    /// runner asks the platform for a redraw).
    pub fn set_frame_requester(&mut self, requester: impl Fn() + 'static) {
        *self.shared.requester.borrow_mut() = Some(Rc::new(requester));
    }

    /// Applies the queued prop values, lays out with `constraints` (the window) and `text`,
    /// and repaints `scene` from scratch, sized to the window (or to the root's size when the
    /// constraints are unbounded).
    pub fn frame(
        &mut self,
        constraints: BoxConstraints,
        text: &mut dyn TextContext,
        scene: &mut Scene,
    ) -> FrameReport {
        // Cleared first, so values queued while applying ask for the next frame.
        self.shared.needs_frame.set(false);
        let applied = self.apply_queue();
        let size = self.layout(constraints, &mut *text);
        let window = constraints.biggest();
        let scene_size = if window.is_finite() { window } else { size };
        let mut builder = scene.begin(scene_size);
        self.paint(&mut builder, &mut *text);
        if let Err(error) = builder.finish() {
            tracing::warn!(?error, "a Paint left the Scene's scopes unbalanced");
        }
        FrameReport { applied }
    }
}
