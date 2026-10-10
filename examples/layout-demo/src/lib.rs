//! The Phase 2 layout demo: a header, a fixed sidebar, proportional bands, a stack with a
//! badge and a footer, composed from Flutter-style layout widgets. Spec:
//! `docs/specs/examples/layout-demo.md`.

use tantu::prelude::*;
use tantu::view::{BuildCx, ElementId};

/// The demo's view.
pub fn layout_demo() -> impl View {
    SizedBox::shrink()
}

/// A box filled with one color (an example of a custom `Paint`).
pub struct Swatch {
    color: Color,
}

impl Swatch {
    /// A box filling its constraints with `color`.
    pub fn new(color: Color) -> Self {
        Swatch { color }
    }
}

impl View for Swatch {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.color, cx);
        todo!()
    }
}
