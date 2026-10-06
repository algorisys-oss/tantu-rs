//! # Tantu: Headless renderer
//!
//! A [`Renderer`] that draws no pixels and records every Scene it is given, with the target
//! size, scale factor and [`RenderReport`] at that moment. Tests assert on what was painted
//! without a GPU or a rasterizer, and test code that drives a renderer (resize, retry after
//! [`RenderError::TargetLost`]). The spec is `docs/specs/render-headless/recorder.md`.
//!
//! ```
//! use tantu_core::{Color, Rect, Size};
//! use tantu_render_headless::HeadlessRenderer;
//! use tantu_scene::{ElementId, Renderer, Resources, Scene};
//!
//! let button = ElementId::from_raw(42).expect("non-zero");
//! let mut scene = Scene::new();
//! let mut b = scene.begin(Size::new(100.0, 50.0));
//! b.set_element(Some(button));
//! b.fill_rect(Rect::from_ltwh(10.0, 10.0, 80.0, 30.0), Color::BLACK);
//! b.finish().expect("balanced scopes");
//!
//! let mut renderer = HeadlessRenderer::new(200, 100);
//! renderer.resize(200, 100, 2.0);
//! let report = renderer.render(&scene, &Resources::new()).expect("headless never fails");
//! assert!(report.is_clean());
//!
//! let frame = renderer.last_frame().expect("one frame recorded");
//! assert_eq!(frame.scale_factor, 2.0);
//! assert_eq!(frame.entries_for(button).count(), 1);
//! ```

#![forbid(unsafe_code)]

use std::collections::HashSet;

use tantu_scene::{
    CustomKind, ElementId, Entry, RenderError, RenderReport, Renderer, Resources, Scene,
};

/// One frame given to a [`HeadlessRenderer`].
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedFrame {
    /// A copy of the Scene as rendered.
    pub scene: Scene,
    /// Target width in physical pixels at the time.
    pub width: u32,
    /// Target height in physical pixels at the time.
    pub height: u32,
    /// Scale factor in effect (after replacing an invalid one with 1).
    pub scale_factor: f32,
    /// `Resources::revision()` at the time.
    pub resources_revision: u64,
    /// The report `render` returned.
    pub report: RenderReport,
}

impl RecordedFrame {
    /// The entries painted for `element`, in paint order.
    pub fn entries_for(&self, element: ElementId) -> impl Iterator<Item = &Entry> + '_ {
        self.scene.entries().iter().filter(move |_| todo!())
    }
}

/// A renderer that records Scenes instead of drawing them.
#[derive(Debug)]
pub struct HeadlessRenderer {
    width: u32,
    height: u32,
    scale_factor: f32,
    custom: HashSet<CustomKind>,
    pending_error: Option<RenderError>,
    frames: Vec<RecordedFrame>,
    frame_count: u64,
}

impl HeadlessRenderer {
    /// A renderer with a `width × height` physical-pixel target, scale factor 1, no frames.
    pub fn new(width: u32, height: u32) -> HeadlessRenderer {
        todo!()
    }

    /// Current target width and height in physical pixels.
    pub fn size(&self) -> (u32, u32) {
        todo!()
    }

    /// Current scale factor.
    pub fn scale_factor(&self) -> f32 {
        todo!()
    }

    /// Treat `kind` as having a handler: its custom commands are not counted as unhandled.
    pub fn register_custom(&mut self, kind: CustomKind) {
        todo!()
    }

    /// Make the next `render` call return `Err(error)` and record nothing.
    pub fn fail_next_render(&mut self, error: RenderError) {
        todo!()
    }

    /// Frames recorded and not yet taken, oldest first.
    pub fn frames(&self) -> &[RecordedFrame] {
        todo!()
    }

    /// The most recent frame not yet taken.
    pub fn last_frame(&self) -> Option<&RecordedFrame> {
        todo!()
    }

    /// Removes and returns the frames recorded so far, oldest first.
    pub fn take_frames(&mut self) -> Vec<RecordedFrame> {
        todo!()
    }

    /// Frames recorded since creation, including taken ones.
    pub fn frame_count(&self) -> u64 {
        todo!()
    }
}

impl Renderer for HeadlessRenderer {
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        todo!()
    }

    fn render(
        &mut self,
        scene: &Scene,
        resources: &Resources,
    ) -> Result<RenderReport, RenderError> {
        todo!()
    }
}
