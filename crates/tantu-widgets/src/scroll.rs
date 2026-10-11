//! [`ScrollView`] and [`ScrollController`]. Spec: `docs/specs/widgets/scroll.md`.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use tantu_view::core::{Color, Point, Rect, Size};
use tantu_view::layout::{RenderViewport, ScrollAxis};
use tantu_view::reactive::{Signal, effect, signal};
use tantu_view::scene::{BorderRadius, Clip, RoundedRect};
use tantu_view::{
    AnimationController, AnyView, BuildCx, Curve, ElementId, FocusOptions, Handled, LogicalKey,
    NamedKey, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics,
    SemanticsAction, View,
};

use crate::theme::{ThemeHandle, at};

/// The thumb's thickness.
const THUMB_WIDTH: f32 = 8.0;
/// The thumb's distance from the trailing edge, and the track's margins.
const MARGIN: f32 = 2.0;
/// The thumb's shortest length.
const MIN_THUMB: f32 = 24.0;
/// The thumb's alpha (on-surface at 40 %).
const THUMB_ALPHA: u8 = 0x66;
/// How far an arrow key scrolls.
const KEY_STEP: f32 = 40.0;

/// A scroll view's offset and extents, shared between the app and the view (Flutter's
/// `ScrollController`). `Copy`; create it inside a runtime.
#[derive(Clone, Copy)]
pub struct ScrollController {
    offset: Signal<f32>,
    /// (viewport extent, max scroll extent) from the last frame.
    extents: Signal<(f32, f32)>,
    /// The running animation's driver (WIDGETS-SCROLL-07, -08).
    fade: AnimationController,
    /// What the animation moves between, and its curve.
    plan: Signal<(f32, f32, Curve)>,
    /// Where accumulated wheel steps are heading, while a wheel animation runs.
    wheel_target: Signal<Option<f32>>,
}

impl ScrollController {
    /// A controller at offset 0 with no extents yet.
    pub fn new() -> Self {
        let offset = signal(0.0f32);
        let fade = AnimationController::new(Duration::from_millis(150));
        let plan = signal((0.0f32, 0.0f32, Curve::Linear));
        // The animation writes the offset while it runs; a jump or a step stops it first.
        effect(move || {
            let t = fade.progress();
            if !fade.is_animating() && t < 1.0 {
                return;
            }
            let (from, to, curve) = plan.get_untracked();
            let next = from + (to - from) * curve.transform(t) as f32;
            if next != offset.get_untracked() {
                offset.set(next);
            }
        });
        ScrollController {
            offset,
            extents: signal((0.0, 0.0)),
            fade,
            plan,
            wheel_target: signal(None),
        }
    }

    /// The offset in logical pixels (a tracked read).
    pub fn offset(&self) -> f32 {
        self.offset.get()
    }

    /// Sets the offset; clamped to `0..=max_scroll_extent` at the next frame.
    pub fn jump_to(&self, offset: f32) {
        self.interrupt();
        self.offset.set(offset);
    }

    /// Animates the offset to `offset` (clamped to the extents at each frame) over `duration`
    /// along `curve`; a wheel step or a jump during the animation takes over
    /// (WIDGETS-SCROLL-08). A zero duration is `jump_to`.
    pub fn animate_to(&self, offset: f32, duration: Duration, curve: Curve) {
        self.interrupt();
        self.start(offset, duration, curve);
    }

    /// Stops a running animation and forgets the wheel target.
    fn interrupt(&self) {
        self.fade.stop();
        if self.wheel_target.get_untracked().is_some() {
            self.wheel_target.set(None);
        }
    }

    /// Animates from the current offset to `to` (clamped), or jumps for a zero duration.
    fn start(&self, to: f32, duration: Duration, curve: Curve) {
        let max = self.extents.get_untracked().1.max(0.0);
        let to = if to.is_finite() {
            to.clamp(0.0, max)
        } else {
            0.0
        };
        if duration.is_zero() {
            self.offset.set(to);
            return;
        }
        let from = self.offset.get_untracked();
        self.plan.set((from, to, curve));
        self.fade.set_value(0.0);
        self.fade.animate_to(1.0, Some(duration));
    }

    /// A wheel step: animates to the accumulated target over `duration` (WIDGETS-SCROLL-07);
    /// true when the target moved.
    fn wheel(&self, delta: f32, duration: Duration) -> bool {
        if duration.is_zero() {
            return self.scroll_by(delta);
        }
        let max = self.extents.get_untracked().1.max(0.0);
        let base = self
            .wheel_target
            .get_untracked()
            .unwrap_or_else(|| self.offset.get_untracked());
        let target = (base + delta).clamp(0.0, max);
        if target == base {
            return false;
        }
        self.fade.stop();
        self.wheel_target.set(Some(target));
        self.start(target, duration, Curve::EaseOut);
        true
    }

    /// The largest offset: content minus viewport along the axis, at least 0 (0 before the
    /// first frame). A tracked read.
    pub fn max_scroll_extent(&self) -> f32 {
        self.extents.get().1
    }

    /// The offset without subscribing (for handlers).
    pub(crate) fn offset_untracked(&self) -> f32 {
        self.offset.get_untracked()
    }

    /// The viewport's extent without subscribing (for handlers).
    pub(crate) fn viewport_extent_untracked(&self) -> f32 {
        self.extents.get_untracked().0
    }

    /// The viewport's extent along the axis, from the last frame.
    pub fn viewport_extent(&self) -> f32 {
        self.extents.get().0
    }

    /// Records the extents after a layout, and clamps the offset to them (once per change).
    fn set_extents(&self, viewport: f32, max: f32) {
        if self.extents.get_untracked() != (viewport, max) {
            self.extents.set((viewport, max));
        }
        let offset = self.offset.get_untracked();
        let clamped = offset.clamp(0.0, max.max(0.0));
        if clamped != offset {
            self.offset.set(clamped);
        }
    }

    /// Moves by `delta` at once within the extents; true when the offset changed.
    fn scroll_by(&self, delta: f32) -> bool {
        self.interrupt();
        let max = self.extents.get_untracked().1.max(0.0);
        let before = self.offset.get_untracked();
        let after = (before + delta).clamp(0.0, max);
        if after != before {
            self.offset.set(after);
            true
        } else {
            false
        }
    }
}

impl Default for ScrollController {
    fn default() -> Self {
        ScrollController::new()
    }
}

/// A scrollable box around one child (Flutter's `SingleChildScrollView` with a `Scrollbar`).
pub struct ScrollView {
    child: AnyView,
    horizontal: bool,
    controller: Option<ScrollController>,
    wheel: Duration,
}

impl ScrollView {
    /// A vertical scroll view around `child`, with its own controller.
    pub fn new(child: impl View) -> Self {
        ScrollView {
            child: AnyView::new(child),
            horizontal: false,
            controller: None,
            wheel: Duration::from_millis(150),
        }
    }

    /// How long a wheel step takes (default 150 ms; zero applies deltas at once)
    /// (WIDGETS-SCROLL-07).
    pub fn wheel_duration(mut self, duration: Duration) -> Self {
        self.wheel = duration;
        self
    }

    /// Scrolls left and right instead.
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Shares `controller` with the app.
    pub fn controller(mut self, controller: ScrollController) -> Self {
        self.controller = Some(controller);
        self
    }
}

/// The scrollbar's geometry along the axis (WIDGETS-SCROLL-04).
#[derive(Clone, Copy)]
struct Scrollbar {
    axis: ScrollAxis,
    viewport: f32,
    content: f32,
    cross: f32,
}

impl Scrollbar {
    fn from(axis: ScrollAxis, size: Size, content: Size) -> Self {
        let (viewport, extent, cross) = match axis {
            ScrollAxis::Vertical => (size.height, content.height, size.width),
            ScrollAxis::Horizontal => (size.width, content.width, size.height),
        };
        Scrollbar {
            axis,
            viewport,
            content: extent,
            cross,
        }
    }

    fn max(&self) -> f32 {
        (self.content - self.viewport).max(0.0)
    }

    fn track(&self) -> f32 {
        (self.viewport - 2.0 * MARGIN).max(0.0)
    }

    fn thumb_length(&self) -> f32 {
        if self.content <= 0.0 {
            return 0.0;
        }
        (self.track() * self.viewport / self.content)
            .max(MIN_THUMB)
            .min(self.track())
    }

    /// Content pixels per track pixel.
    fn ratio(&self) -> f32 {
        let room = self.track() - self.thumb_length();
        if room > 0.0 { self.max() / room } else { 0.0 }
    }

    fn thumb_rect(&self, offset: f32) -> Option<Rect> {
        let max = self.max();
        if max <= 0.0 {
            return None;
        }
        let along = MARGIN + (self.track() - self.thumb_length()) * (offset / max).clamp(0.0, 1.0);
        let near = self.cross - MARGIN - THUMB_WIDTH;
        Some(match self.axis {
            ScrollAxis::Vertical => Rect::from_ltwh(near, along, THUMB_WIDTH, self.thumb_length()),
            ScrollAxis::Horizontal => {
                Rect::from_ltwh(along, near, self.thumb_length(), THUMB_WIDTH)
            }
        })
    }
}

/// What the handlers share with the paint: the last layout's size and content.
type Shared = Rc<Cell<(Size, Size)>>;

impl View for ScrollView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let controller = self.controller.unwrap_or_default();
        let axis = if self.horizontal {
            ScrollAxis::Horizontal
        } else {
            ScrollAxis::Vertical
        };
        let theme = ThemeHandle::of(cx);
        let wheel = self.wheel;
        let id = cx.render(RenderViewport::new(axis), [self.child]);
        let shared: Shared = Rc::default();
        // Semantics: a scroll view that scrolls a descendant into view on request
        // (WIDGETS-SEM-02).
        cx.set_semantics(
            id,
            Semantics {
                role: Role::ScrollView,
                ..Default::default()
            },
        );
        cx.on_action(id, move |a| {
            if *a.action != SemanticsAction::ScrollIntoView {
                return false;
            }
            let (Some(own), Some(target)) = (a.tree.bounds(a.element), a.tree.bounds(a.target))
            else {
                return false;
            };
            let (own_start, own_end, start, end) = match axis {
                ScrollAxis::Vertical => (own.top, own.bottom, target.top, target.bottom),
                ScrollAxis::Horizontal => (own.left, own.right, target.left, target.right),
            };
            let delta = if start < own_start {
                start - own_start
            } else if end > own_end {
                end - own_end
            } else {
                0.0
            };
            if delta != 0.0 {
                controller.scroll_by(delta);
            }
            true
        });
        cx.set_paint(
            id,
            ViewportPaint {
                axis,
                controller,
                shared: Rc::clone(&shared),
                thumb: Color::TRANSPARENT,
            },
        );
        cx.bind(
            id,
            theme.color_prop(|c| at(c.on_surface, THUMB_ALPHA)),
            |e, color| {
                e.update_paint::<ViewportPaint>(|p| p.thumb = color);
            },
        );
        // The viewport's offset follows the controller, clamped to the last extents.
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                let max = controller.max_scroll_extent().max(0.0);
                controller.offset().clamp(0.0, max)
            })),
            move |e, offset| {
                e.update_render::<RenderViewport>(|v| v.offset = offset);
            },
        );
        // Keys (WIDGETS-SCROLL-06): a press focuses the view, Tab doesn't stop at it.
        cx.focusable(
            id,
            FocusOptions {
                traversable: false,
                focus_on_press: true,
            },
        );
        {
            let shared = Rc::clone(&shared);
            cx.on_key(id, Phase::Bubble, move |k| {
                if !k.event.pressed {
                    return Handled::Continue;
                }
                let (size, _) = shared.get();
                let viewport = match axis {
                    ScrollAxis::Vertical => size.height,
                    ScrollAxis::Horizontal => size.width,
                };
                let max = controller.extents.get_untracked().1.max(0.0);
                let current = controller.offset.get_untracked();
                let (forward, back) = match axis {
                    ScrollAxis::Vertical => (NamedKey::ArrowDown, NamedKey::ArrowUp),
                    ScrollAxis::Horizontal => (NamedKey::ArrowRight, NamedKey::ArrowLeft),
                };
                let delta = match &k.event.key {
                    LogicalKey::Named(key) if *key == forward => KEY_STEP,
                    LogicalKey::Named(key) if *key == back => -KEY_STEP,
                    LogicalKey::Named(NamedKey::PageDown) => viewport,
                    LogicalKey::Named(NamedKey::PageUp) => -viewport,
                    LogicalKey::Named(NamedKey::End) => max - current,
                    LogicalKey::Named(NamedKey::Home) => -current,
                    _ => return Handled::Continue,
                };
                if controller.scroll_by(delta) {
                    Handled::Stop
                } else {
                    Handled::Continue
                }
            });
        }
        // Wheel and thumb (WIDGETS-SCROLL-02, -04).
        let drag: Rc<Cell<Option<(f32, f32)>>> = Rc::default();
        cx.on_pointer(id, Phase::Bubble, move |p| {
            let (size, content) = shared.get();
            let bar = Scrollbar::from(axis, size, content);
            let along = |point: Point| match axis {
                ScrollAxis::Vertical => point.y,
                ScrollAxis::Horizontal => point.x,
            };
            match p.event.kind {
                PointerKind::Scroll(delta) => {
                    if controller.wheel(along(Point::new(delta.x, delta.y)), wheel) {
                        Handled::Stop
                    } else {
                        Handled::Continue
                    }
                }
                PointerKind::Down(PointerButton::Primary) => {
                    let offset = controller.offset.get_untracked();
                    match bar.thumb_rect(offset) {
                        Some(thumb) if thumb.contains(p.local) => {
                            drag.set(Some((along(p.local), offset)));
                            Handled::Stop
                        }
                        _ => Handled::Continue,
                    }
                }
                PointerKind::Move => match drag.get() {
                    Some((start, offset)) => {
                        let moved = (along(p.local) - start) * bar.ratio();
                        let target = (offset + moved).clamp(0.0, bar.max());
                        if target != controller.offset.get_untracked() {
                            controller.offset.set(target);
                        }
                        Handled::Stop
                    }
                    None => Handled::Continue,
                },
                PointerKind::Up(PointerButton::Primary) if drag.get().is_some() => {
                    drag.set(None);
                    Handled::Stop
                }
                _ => Handled::Continue,
            }
        });
        id
    }
}

/// Clips the content, paints it, then the scrollbar (WIDGETS-SCROLL-03, -04).
struct ViewportPaint {
    axis: ScrollAxis,
    controller: ScrollController,
    shared: Shared,
    /// The thumb's color (the theme's `on_surface` at 40 %).
    thumb: Color,
}

impl Paint for ViewportPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let content = cx
            .render::<RenderViewport>()
            .map_or(Size::ZERO, |v| v.content());
        self.shared.set((size, content));
        let bar = Scrollbar::from(self.axis, size, content);
        self.controller.set_extents(bar.viewport, bar.max());
        let bounds = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        cx.scene().push_clip(Clip::Rect(bounds));
        cx.paint_children();
        cx.scene().pop();
        if let Some(thumb) = bar.thumb_rect(self.controller.offset.get_untracked()) {
            cx.scene().fill(
                RoundedRect::new(thumb, BorderRadius::circular(THUMB_WIDTH / 2.0)),
                self.thumb,
            );
        }
    }

    fn clips_children(&self) -> bool {
        true
    }
}
