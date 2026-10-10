//! [`WidgetTester`] and [`Finder`].

use std::path::Path;

use tantu_render_headless::RecordedFrame;
use tantu_view::core::Rect;
use tantu_view::layout::RenderBox;
use tantu_view::{ElementId, Key, PointerEvent, View, ViewTree};

/// Which elements to find.
pub struct Finder {
    description: String,
    matches: Box<dyn Fn(&ViewTree, ElementId) -> bool>,
}

impl Finder {
    /// Elements with this key ([`Keyed`](tantu_view::Keyed)).
    pub fn key(key: impl Into<Key>) -> Self {
        let _ = key.into();
        todo!()
    }

    /// `Text` elements showing exactly `text`.
    pub fn text(text: impl Into<String>) -> Self {
        let _ = text.into();
        todo!()
    }

    /// Render elements whose render object is an `R`.
    pub fn render<R: RenderBox>() -> Self {
        todo!()
    }
}

impl std::fmt::Debug for Finder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = &self.matches;
        f.write_str(&self.description)
    }
}

/// Builds a view in a test window and drives it: frames, pointer input, finders, goldens.
pub struct WidgetTester {
    tree: ViewTree,
}

impl WidgetTester {
    /// Builds `app` in an 800 × 600 window (logical, scale 1) with the test font as default
    /// family, and pumps the first frame.
    pub fn new<V: View>(app: impl FnOnce() -> V) -> Self {
        WidgetTester::with_size(800.0, 600.0, app)
    }

    /// Same, in a `width` × `height` window.
    pub fn with_size<V: View>(width: f32, height: f32, app: impl FnOnce() -> V) -> Self {
        let _ = (width, height, app);
        todo!()
    }

    /// Runs one frame (apply updates, layout, paint, record); returns whether anything had
    /// changed.
    pub fn pump(&mut self) -> bool {
        todo!()
    }

    /// The last recorded frame.
    pub fn frame(&self) -> &RecordedFrame {
        todo!()
    }

    /// Primary press and release at the center of the single element `finder` matches, then a
    /// pump. Panics if it matches no element or several.
    pub fn tap(&mut self, finder: &Finder) {
        let _ = finder;
        todo!()
    }

    /// A pointer move to the center of the single element `finder` matches, then a pump.
    pub fn hover(&mut self, finder: &Finder) {
        let _ = finder;
        todo!()
    }

    /// Dispatches a raw pointer event (no pump).
    pub fn pointer(&mut self, event: PointerEvent) -> bool {
        let _ = event;
        todo!()
    }

    /// The single element `finder` matches (panics otherwise, naming the finder).
    pub fn find(&self, finder: &Finder) -> ElementId {
        let _ = finder;
        todo!()
    }

    /// Every element `finder` matches, in tree order.
    pub fn find_all(&self, finder: &Finder) -> Vec<ElementId> {
        let _ = (finder, &self.tree);
        todo!()
    }

    /// The element's bounds in window coordinates (from the last layout).
    pub fn rect(&self, element: ElementId) -> Rect {
        let _ = element;
        todo!()
    }

    /// The text a `Text` element shows.
    pub fn text(&self, element: ElementId) -> Option<String> {
        let _ = element;
        todo!()
    }

    /// Runs `f` with the tree's runtime current.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _ = f;
        todo!()
    }

    /// Renders the last frame with the software renderer and compares it with the PNG at
    /// `path` (tolerance 2); with `TANTU_UPDATE_GOLDENS=1` it writes the PNG instead.
    pub fn matches_golden(&mut self, path: impl AsRef<Path>) {
        let _ = path.as_ref();
        todo!()
    }
}
