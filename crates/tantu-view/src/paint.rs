//! [`Paint`] and [`PaintCx`]: how render elements become Scene commands (ADR 0011, point 8).
//! Spec: `docs/specs/view/paint.md`.

use std::any::Any;

use tantu_core::{Color, Point, Size};
use tantu_layout::{BoxConstraints, RenderBox, TextStyleKey};
use tantu_scene::{ElementId, SceneBuilder};

use crate::{TextPainter, ViewTree};

/// How a render element draws itself and where its children are painted. (`Any`, so
/// [`ElementMut::paint`](crate::ElementMut::paint) can downcast it; every `'static` type is
/// `Any`.)
pub trait Paint: Any {
    /// Draws the element. `cx` is in the element's coordinates ((0, 0) is its top-left corner,
    /// its size is `cx.size()`), with its id as the current element. The children are painted
    /// where this calls [`PaintCx::paint_children`], or after it returns if it doesn't (unless
    /// it calls [`PaintCx::skip_children`]). It must leave the builder's scopes balanced.
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let _ = cx;
    }

    /// How far the element's own drawing may extend beyond its bounds (shadows, focus rings),
    /// for culling. Default 0.
    fn overflow(&self) -> f32 {
        0.0
    }

    /// True if the element clips its children to its bounds, so culling may skip the whole
    /// subtree with it. Default false.
    fn clips_children(&self) -> bool {
        false
    }
}

/// Paints nothing (the default for render elements).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoPaint;

impl Paint for NoPaint {}

/// Whether an element's children have been painted yet.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Children {
    Pending,
    Painted,
    Skipped,
}

/// What a [`Paint`] draws with.
pub struct PaintCx<'t, 'b> {
    pub(crate) tree: &'t ViewTree,
    pub(crate) element: ElementId,
    pub(crate) size: Size,
    pub(crate) scene: &'t mut SceneBuilder<'b>,
    pub(crate) text: &'t mut dyn TextPainter,
    pub(crate) children: Children,
}

impl<'b> PaintCx<'_, 'b> {
    /// The element being painted.
    pub fn element(&self) -> ElementId {
        self.element
    }

    /// Its size from the last layout.
    pub fn size(&self) -> Size {
        self.size
    }

    /// The Scene builder, in the element's coordinates.
    pub fn scene(&mut self) -> &mut SceneBuilder<'b> {
        self.scene
    }

    /// Paints the element's children here (once; later calls do nothing).
    pub fn paint_children(&mut self) {
        if self.children != Children::Pending {
            return;
        }
        self.children = Children::Painted;
        for child in self.tree.children(self.element) {
            self.tree.paint_element(*child, self.scene, &mut *self.text);
        }
    }

    /// The text painter of this paint pass.
    pub fn text(&mut self) -> &mut dyn TextPainter {
        &mut *self.text
    }

    /// Paints text through the painter into this context's Scene builder (`text()` and
    /// `scene()` can't be borrowed at the same time).
    pub fn paint_text(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
        color: Color,
        origin: Point,
    ) {
        self.text
            .paint_text(self.scene, text, style, max_width, max_lines, color, origin);
    }

    /// The element's constraints from the last layout.
    pub fn constraints(&self) -> BoxConstraints {
        self.tree
            .layout_id(self.element)
            .and_then(|layout| self.tree.layout_tree().constraints(layout))
            .unwrap_or_default()
    }

    /// The element's layout object, if it is an `R`.
    pub fn render<R: RenderBox>(&self) -> Option<&R> {
        let layout = self.tree.layout_id(self.element)?;
        self.tree.layout_tree().get::<R>(layout)
    }

    /// Don't paint the children at all.
    pub fn skip_children(&mut self) {
        if self.children == Children::Pending {
            self.children = Children::Skipped;
        }
    }
}
