//! The overlay layer, [`OverlayPortal`] and [`Anchor`]: content drawn above everything,
//! anchored to a target. Spec: `docs/specs/view/overlay.md`.

use std::rc::Rc;

use tantu_core::Vec2;
use tantu_layout::Alignment;
use tantu_scene::ElementId;

use crate::{AnyView, BuildCx, IntoProp, Prop, View};

/// Where an overlay goes relative to its target (Flutter's follower, Clay's attach points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    /// The point on the target.
    pub target: Alignment,
    /// The point on the overlay placed at the target point.
    pub follower: Alignment,
    /// Added after anchoring, in logical pixels.
    pub offset: Vec2,
    /// Mirror vertically (then horizontally) when the overlay would leave the window.
    pub flip: bool,
    /// Shift the overlay to stay inside the window (after flipping).
    pub clamp: bool,
}

impl Anchor {
    /// `target` to `follower`, no offset, flip and clamp on.
    pub fn new(target: Alignment, follower: Alignment) -> Self {
        Anchor {
            target,
            follower,
            offset: Vec2::ZERO,
            flip: true,
            clamp: true,
        }
    }

    /// Below the target, start edges aligned (dropdowns, menus).
    pub fn below() -> Self {
        Anchor::new(Alignment::BOTTOM_LEFT, Alignment::TOP_LEFT)
    }

    /// Above the target, centered (tooltips).
    pub fn above() -> Self {
        Anchor::new(Alignment::TOP_CENTER, Alignment::BOTTOM_CENTER)
    }

    /// With `offset` added after anchoring.
    pub fn offset(self, dx: f32, dy: f32) -> Self {
        Anchor {
            offset: Vec2::new(dx, dy),
            ..self
        }
    }

    /// With flipping on or off.
    pub fn flip(self, flip: bool) -> Self {
        Anchor { flip, ..self }
    }

    /// With clamping on or off.
    pub fn clamp(self, clamp: bool) -> Self {
        Anchor { clamp, ..self }
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::below()
    }
}

/// Whether an overlay takes pointer events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayPointer {
    /// It is hit-tested like other elements (menus, popovers).
    #[default]
    Capture,
    /// Pointer events pass through it to what's below (tooltips).
    Passthrough,
}

/// Builds overlay content.
type ContentFn = Rc<dyn Fn() -> AnyView>;

/// Shows `child` in place and, while `visible`, `overlay` in the overlay layer, anchored to the
/// child (Flutter's `OverlayPortal`).
pub struct OverlayPortal<V> {
    child: V,
    visible: Prop<bool>,
    content: Option<ContentFn>,
    anchor: Anchor,
    z_index: i32,
    pointer: OverlayPointer,
    on_outside_press: Option<Rc<dyn Fn()>>,
}

impl<V: View> OverlayPortal<V> {
    /// A portal around `child`, hidden, with no overlay content yet.
    pub fn new(child: V) -> Self {
        OverlayPortal {
            child,
            visible: Prop::Value(false),
            content: None,
            anchor: Anchor::below(),
            z_index: 0,
            pointer: OverlayPointer::Capture,
            on_outside_press: None,
        }
    }

    /// Whether the overlay shows (a value, a closure or a signal).
    pub fn visible(mut self, visible: impl IntoProp<bool>) -> Self {
        self.visible = visible.into_prop();
        self
    }

    /// Builds the overlay content each time it becomes visible (dropped when hidden).
    pub fn overlay<W: View>(mut self, content: impl Fn() -> W + 'static) -> Self {
        self.content = Some(Rc::new(move || AnyView::new(content())));
        self
    }

    /// Where the overlay goes relative to the child.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Higher draws above lower; equal values keep insertion order.
    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    /// Whether the overlay takes pointer events.
    pub fn pointer(mut self, pointer: OverlayPointer) -> Self {
        self.pointer = pointer;
        self
    }

    /// Called on a primary press outside both the overlay content and the child.
    pub fn on_outside_press(mut self, f: impl Fn() + 'static) -> Self {
        self.on_outside_press = Some(Rc::new(f));
        self
    }
}

impl<V: View> View for OverlayPortal<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.child,
            self.visible,
            self.content,
            self.anchor,
            self.z_index,
            self.pointer,
            self.on_outside_press,
            cx,
        );
        todo!()
    }
}
