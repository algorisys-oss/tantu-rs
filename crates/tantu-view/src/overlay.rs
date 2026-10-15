//! The overlay layer, [`OverlayPortal`] and [`Anchor`]: content drawn above everything,
//! anchored to a target. Spec: `docs/specs/view/overlay.md`.

use std::cell::Cell;
use std::rc::Rc;

use tantu_core::{Rect, Size, Vec2};
use tantu_layout::{Alignment, BoxConstraints, LayoutChildren, RenderBox, RenderConstrainedBox};
use tantu_reactive::Scope;
use tantu_scene::ElementId;

use crate::tree::Kind;
use crate::{AnyView, BuildCx, IntoProp, NoPaint, Prop, View, ViewTree};

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
    anchor: Prop<Anchor>,
    z_index: i32,
    pointer: OverlayPointer,
    on_outside_press: Option<Rc<dyn Fn()>>,
    cover: bool,
}

impl<V: View> OverlayPortal<V> {
    /// A portal around `child`, hidden, with no overlay content yet.
    pub fn new(child: V) -> Self {
        OverlayPortal {
            child,
            visible: Prop::Value(false),
            content: None,
            anchor: Prop::Value(Anchor::below()),
            z_index: 0,
            pointer: OverlayPointer::Capture,
            on_outside_press: None,
            cover: false,
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

    /// Where the overlay goes relative to the child: a value, closure or signal; changes
    /// re-place the overlay at the next frame (VIEW-OVERLAY-11).
    pub fn anchor(mut self, anchor: impl IntoProp<Anchor>) -> Self {
        self.anchor = anchor.into_prop();
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

    /// The overlay covers the window: laid out tight to the window at its top-left,
    /// ignoring the anchor (VIEW-OVERLAY-13).
    pub fn cover_window(mut self, cover: bool) -> Self {
        self.cover = cover;
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
        let target = self.child.build(cx);
        let Some(content) = self.content else {
            return target;
        };
        // The anchor is shared with the entry, so a new value re-places the overlay at the
        // next layout (VIEW-OVERLAY-11).
        let anchor = Rc::new(Cell::new(Anchor::below()));
        {
            let anchor = Rc::clone(&anchor);
            cx.bind(target, self.anchor, move |_, value| anchor.set(value));
        }
        let config = EntryConfig {
            anchor,
            z_index: self.z_index,
            pointer: self.pointer,
            on_outside_press: self.on_outside_press,
            cover: self.cover,
        };
        cx.bind(target, self.visible, move |e, visible| {
            if visible {
                e.tree.show_overlay(target, &content, &config);
            } else {
                e.tree.hide_overlay(target);
            }
        });
        target
    }
}

/// How a portal's overlay is placed and behaves.
#[derive(Clone)]
struct EntryConfig {
    anchor: Rc<Cell<Anchor>>,
    z_index: i32,
    pointer: OverlayPointer,
    on_outside_press: Option<Rc<dyn Fn()>>,
    /// Laid out over the whole window (VIEW-OVERLAY-13).
    cover: bool,
}

/// One shown overlay.
struct Entry {
    /// The wrapper render element under the layer that holds the content.
    root: ElementId,
    /// The portal's child, which the overlay is anchored to.
    target: ElementId,
    /// Owns what the content closure created (disposed when the overlay hides).
    scope: Scope,
    config: EntryConfig,
    /// Insertion order, for equal z-indexes.
    seq: u64,
}

/// The tree's overlays: the layer element (created on first use) and the shown entries.
#[derive(Default)]
pub(crate) struct OverlayState {
    layer: Option<ElementId>,
    entries: Vec<Entry>,
    next_seq: u64,
}

/// The overlay layer's layout: as large as the window, children laid out with loose window
/// constraints at the offsets in their parent data.
struct OverlayLayer;

/// An overlay's position in the layer, and whether it covers the window (parent data).
#[derive(Clone, Copy, Debug, PartialEq)]
struct OverlayOffset(Vec2, bool);

impl RenderBox for OverlayLayer {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let biggest = constraints.biggest();
        let size = if biggest.is_finite() {
            biggest
        } else {
            constraints.constrain(Size::ZERO)
        };
        let loose = BoxConstraints::loose(size);
        for i in 0..children.len() {
            let placement = children
                .parent_data::<OverlayOffset>(i)
                .copied()
                .unwrap_or(OverlayOffset(Vec2::ZERO, false));
            if placement.1 {
                // A window-covering overlay (VIEW-OVERLAY-13).
                children.layout(i, BoxConstraints::tight(size));
                children.set_offset(i, Vec2::ZERO);
            } else {
                children.layout(i, loose);
                children.set_offset(i, placement.0);
            }
        }
        size
    }
}

/// The point at `alignment` in `size`, from its top-left corner.
fn point_in(size: Size, alignment: Alignment) -> Vec2 {
    Vec2::new(
        (alignment.x + 1.0) / 2.0 * size.width,
        (alignment.y + 1.0) / 2.0 * size.height,
    )
}

/// Where an overlay of `size` goes for `target` in a `window` (VIEW-OVERLAY-01, -03).
fn place(anchor: &Anchor, target: Rect, size: Size, window: Size) -> Vec2 {
    let at = |a: &Anchor| {
        let t = point_in(target.size(), a.target);
        let f = point_in(size, a.follower);
        Vec2::new(
            target.left + t.x - f.x + a.offset.x,
            target.top + t.y - f.y + a.offset.y,
        )
    };
    let fits = |start: f32, length: f32, limit: f32| start >= 0.0 && start + length <= limit;
    let mut anchor = *anchor;
    let mut position = at(&anchor);
    if anchor.flip && !fits(position.y, size.height, window.height) {
        let mut mirrored = anchor;
        mirrored.target.y = -anchor.target.y;
        mirrored.follower.y = -anchor.follower.y;
        mirrored.offset.y = -anchor.offset.y;
        let candidate = at(&mirrored);
        if fits(candidate.y, size.height, window.height) {
            anchor = mirrored;
            position = candidate;
        }
    }
    if anchor.flip && !fits(position.x, size.width, window.width) {
        let mut mirrored = anchor;
        mirrored.target.x = -anchor.target.x;
        mirrored.follower.x = -anchor.follower.x;
        mirrored.offset.x = -anchor.offset.x;
        let candidate = at(&mirrored);
        if fits(candidate.x, size.width, window.width) {
            position = candidate;
        }
    }
    if anchor.clamp {
        let clamp = |p: f32, length: f32, limit: f32| {
            if length > limit {
                0.0
            } else {
                p.clamp(0.0, limit - length)
            }
        };
        position = Vec2::new(
            clamp(position.x, size.width, window.width),
            clamp(position.y, size.height, window.height),
        );
    }
    position
}

impl ViewTree {
    /// The overlay layer, created as the root's last child on first use.
    fn overlay_layer(&mut self) -> ElementId {
        if let Some(layer) = self.overlays.layer.filter(|l| self.contains(*l)) {
            return layer;
        }
        let layout = self.layout.insert(OverlayLayer);
        let layer = self.add_element(self.root(), Kind::Render(layout, Box::new(NoPaint)));
        self.overlays.layer = Some(layer);
        self.sync_layout_children(self.root());
        layer
    }

    /// Shows `target`'s overlay if it isn't shown (VIEW-OVERLAY-01).
    fn show_overlay(&mut self, target: ElementId, content: &ContentFn, config: &EntryConfig) {
        if self.overlays.entries.iter().any(|e| e.target == target) {
            return;
        }
        let Some(owner) = self.scope(target) else {
            return;
        };
        let layer = self.overlay_layer();
        let scope = owner.run(Scope::new);
        let wrapper_layout = self
            .layout
            .insert(RenderConstrainedBox::new(BoxConstraints::default()));
        let wrapper = self.add_element(layer, Kind::Render(wrapper_layout, Box::new(NoPaint)));
        // Install the context path before invoking the builder (VIEW-OVERLAY-08).
        // The wrapper stays under the layer for every structural operation.
        if let Some(element) = self.element_mut(wrapper) {
            element.inherited_parent = Some(target);
        }
        scope.run(|| {
            let view = content();
            view.build(&mut BuildCx {
                tree: &mut *self,
                parent: wrapper,
            });
        });
        self.sync_layout_children(wrapper);
        let seq = self.overlays.next_seq;
        self.overlays.next_seq += 1;
        self.overlays.entries.push(Entry {
            root: wrapper,
            target,
            scope,
            config: config.clone(),
            seq,
        });
        self.order_overlays(layer);
        self.sync_overlay_offstage();
    }

    /// Hides each overlay whose target is offstage and shows it again when the target
    /// returns, without rebuilding it (VIEW-OFFSTAGE-05).
    pub(crate) fn sync_overlay_offstage(&mut self) {
        let changes: Vec<(ElementId, bool)> = self
            .overlays
            .entries
            .iter()
            .map(|e| (e.root, self.is_offstage(e.target)))
            .filter(|(root, hidden)| self.element(*root).is_some_and(|r| r.offstage != *hidden))
            .collect();
        for (root, hidden) in changes {
            self.set_offstage_flag(root, hidden);
        }
    }

    /// Sorts the layer's children by z-index, then insertion order (VIEW-OVERLAY-02).
    fn order_overlays(&mut self, layer: ElementId) {
        let mut entries: Vec<(i32, u64, ElementId)> = self
            .overlays
            .entries
            .iter()
            .map(|e| (e.config.z_index, e.seq, e.root))
            .collect();
        entries.sort_by_key(|(z, seq, _)| (*z, *seq));
        let order = entries.into_iter().map(|(_, _, root)| root).collect();
        self.set_child_order(layer, order);
    }

    /// Hides `target`'s overlay: removes its elements and disposes its scope.
    fn hide_overlay(&mut self, target: ElementId) {
        let Some(index) = self
            .overlays
            .entries
            .iter()
            .position(|e| e.target == target)
        else {
            return;
        };
        let entry = self.overlays.entries.remove(index);
        self.remove(entry.root);
        let runtime = std::rc::Rc::clone(&self.runtime);
        runtime.enter(|| entry.scope.dispose());
    }

    /// Hides the overlays whose targets were removed (VIEW-OVERLAY-06).
    pub(crate) fn drop_overlays_of(&mut self, removed: &[ElementId]) {
        let targets: Vec<ElementId> = self
            .overlays
            .entries
            .iter()
            .filter(|e| removed.contains(&e.target) || removed.contains(&e.root))
            .map(|e| e.target)
            .collect();
        for target in targets {
            self.hide_overlay(target);
        }
    }

    /// Places every overlay for the last layout; true if a placement changed (VIEW-OVERLAY-06).
    pub(crate) fn place_overlays(&mut self, window: Size) -> bool {
        let mut changed = false;
        let placements: Vec<(ElementId, OverlayOffset)> = self
            .overlays
            .entries
            .iter()
            .filter_map(|e| {
                let target = self.window_rect(e.target)?;
                let size = self
                    .layout_id(e.root)
                    .and_then(|l| self.layout_tree().size(l))?;
                let placement = if e.config.cover {
                    OverlayOffset(Vec2::ZERO, true)
                } else {
                    OverlayOffset(place(&e.config.anchor.get(), target, size, window), false)
                };
                Some((e.root, placement))
            })
            .collect();
        for (root, placement) in placements {
            let Some(layout) = self.layout_id(root) else {
                continue;
            };
            if self.layout.parent_data::<OverlayOffset>(layout) != Some(&placement) {
                self.layout.set_parent_data(layout, Some(placement));
                changed = true;
            }
        }
        changed
    }

    /// An element's bounds in window coordinates; a region's is its children's union.
    fn window_rect(&self, id: ElementId) -> Option<Rect> {
        match self.layout_id(id) {
            Some(layout) => {
                let size = self.layout_tree().size(layout)?;
                let origin = self.origin(id);
                Some(Rect::from_ltwh(origin.x, origin.y, size.width, size.height))
            }
            None => self
                .children(id)
                .iter()
                .filter_map(|c| self.window_rect(*c))
                .reduce(Rect::union),
        }
    }

    /// True for the overlay layer element (transparent to hit-testing, VIEW-OVERLAY-04).
    pub(crate) fn is_overlay_layer(&self, id: ElementId) -> bool {
        self.overlays.layer == Some(id)
    }

    /// True for an overlay wrapper with `Passthrough` (skipped by hit-testing).
    pub(crate) fn is_passthrough(&self, id: ElementId) -> bool {
        self.overlays
            .entries
            .iter()
            .any(|e| e.root == id && e.config.pointer == OverlayPointer::Passthrough)
    }

    /// Runs the `on_outside_press` callbacks of overlays a primary press on `path` misses
    /// (VIEW-OVERLAY-05).
    /// Whether a press on `path` is inside the overlay rooted at `root`: in its content, or in
    /// an overlay nested in it (one whose target lies inside it, at any depth) (VIEW-OVERLAY-12).
    fn press_inside(&self, root: ElementId, path: &[ElementId]) -> bool {
        let mut roots = vec![root];
        let mut i = 0;
        while let Some(&current) = roots.get(i) {
            if path.contains(&current) {
                return true;
            }
            for entry in &self.overlays.entries {
                if !roots.contains(&entry.root) && self.is_within(entry.target, current) {
                    roots.push(entry.root);
                }
            }
            i += 1;
        }
        false
    }

    /// Whether `element` is `ancestor` or one of its descendants.
    fn is_within(&self, element: ElementId, ancestor: ElementId) -> bool {
        let mut next = Some(element);
        while let Some(e) = next {
            if e == ancestor {
                return true;
            }
            next = self.parent(e);
        }
        false
    }

    pub(crate) fn outside_presses(&self, path: &[ElementId]) {
        let callbacks: Vec<Rc<dyn Fn()>> = self
            .overlays
            .entries
            .iter()
            .filter(|e| !path.contains(&e.target) && !self.press_inside(e.root, path))
            .filter(|e| !self.is_offstage(e.root))
            .filter_map(|e| e.config.on_outside_press.clone())
            .collect();
        if !callbacks.is_empty() {
            self.enter(|| {
                for callback in &callbacks {
                    callback();
                }
            });
        }
    }
}
