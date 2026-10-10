//! Binding props to elements, the update queue and [`ViewTree::frame`] (ADR 0011, point 6).
//! Spec: `docs/specs/view/frame.md`.

use tantu_layout::{BoxConstraints, RenderBox, TextMeasure};
use tantu_scene::{ElementId, Scene};

use crate::{BuildCx, Paint, Prop, ViewTree};

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
        let _ = &self.tree;
        todo!()
    }

    /// Changes a copy of the layout object with `f` and stores it with `LayoutTree::set`, so
    /// the node is re-laid out only if it changed. Returns whether it changed (false if the
    /// element isn't a render element with an `R`).
    pub fn update_render<R: RenderBox + Clone + PartialEq>(
        &mut self,
        f: impl FnOnce(&mut R),
    ) -> bool {
        let _ = f;
        todo!()
    }

    /// The element's paint behavior, if it is a `P`.
    pub fn paint<P: Paint>(&self) -> Option<&P> {
        todo!()
    }

    /// Changes the paint behavior in place. Returns false if it isn't a `P`.
    pub fn update_paint<P: Paint>(&mut self, f: impl FnOnce(&mut P)) -> bool {
        let _ = f;
        todo!()
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
        let _ = (element, prop, apply);
        todo!()
    }
}

/// What a frame did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameReport {
    /// Prop values applied.
    pub applied: usize,
}

impl ViewTree {
    /// True when something changed since the last frame.
    pub fn needs_frame(&self) -> bool {
        todo!()
    }

    /// Called once each time the tree goes from not needing a frame to needing one (the app
    /// runner asks the platform for a redraw).
    pub fn set_frame_requester(&mut self, requester: impl Fn() + 'static) {
        let _ = requester;
        todo!()
    }

    /// Applies the queued prop values, lays out with `constraints` (the window) and `text`,
    /// and repaints `scene` from scratch, sized to the window (or to the root's size when the
    /// constraints are unbounded).
    pub fn frame(
        &mut self,
        constraints: BoxConstraints,
        text: &mut dyn TextMeasure,
        scene: &mut Scene,
    ) -> FrameReport {
        let _ = (constraints, text, scene);
        todo!()
    }
}
