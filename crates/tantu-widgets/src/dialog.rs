//! Modal dialogs: [`Dialog`], [`DialogController`] and [`AlertDialog`]. Spec:
//! `docs/specs/widgets/dialog.md`.

use std::cell::Cell;
use std::rc::Rc;

use tantu_theme::ColorScheme;
use tantu_view::core::{Color, Rect, Size, Vec2};
use tantu_view::layout::{
    BoxConstraints, LayoutChildren, MainAxisSize, RenderBox, RenderConstrainedBox,
};
use tantu_view::reactive::{Signal, effect, signal, untrack};
use tantu_view::scene::{BorderRadius, BoxShadow, RoundedRect};
use tantu_view::{
    AnyView, BuildCx, Dyn, ElementId, FocusOptions, Handled, LogicalKey, Modifiers, NamedKey,
    OverlayPortal, Paint, PaintCx, Phase, PointerButton, PointerKind, Role, Semantics, View,
};

use crate::theme::ThemeHandle;
use crate::{Row, Text};

/// The barrier's opacity (WIDGETS-DIALOG-01).
const BARRIER_ALPHA: f32 = 0.32;
/// The space kept around a dialog's content.
const MARGIN: f32 = 24.0;
/// `AlertDialog` metrics (WIDGETS-DIALOG-04).
const PADDING: f32 = 24.0;
const MIN_WIDTH: f32 = 280.0;
const MAX_WIDTH: f32 = 560.0;
const RADIUS: f32 = 28.0;
const TITLE_GAP: f32 = 16.0;
const ACTIONS_GAP: f32 = 24.0;
const ACTION_SPACING: f32 = 8.0;

/// Opens and closes a dialog; `Copy`, owned by the scope that created it.
#[derive(Clone, Copy)]
pub struct DialogController {
    open: Signal<bool>,
}

impl Default for DialogController {
    fn default() -> Self {
        DialogController::new()
    }
}

impl DialogController {
    /// A closed dialog.
    pub fn new() -> Self {
        DialogController {
            open: signal(false),
        }
    }

    /// Opens the dialog.
    pub fn open(&self) {
        if !self.open.get_untracked() {
            self.open.set(true);
        }
    }

    /// Closes the dialog.
    pub fn close(&self) {
        if self.open.get_untracked() {
            self.open.set(false);
        }
    }

    /// Whether the dialog is open (tracked).
    pub fn is_open(&self) -> bool {
        self.open.get()
    }
}

/// A modal dialog shown over the window while its controller is open. Place it anywhere in
/// the tree; it takes no space (WIDGETS-DIALOG-01..05).
pub struct Dialog {
    controller: DialogController,
    content: Rc<dyn Fn() -> AnyView>,
    dismissible: bool,
    on_dismiss: Option<Rc<dyn Fn()>>,
    label: Option<String>,
}

impl Dialog {
    /// A dialog showing `content`, built each time it opens.
    pub fn new<V: View>(controller: DialogController, content: impl Fn() -> V + 'static) -> Self {
        Dialog {
            controller,
            content: Rc::new(move || AnyView::new(content())),
            dismissible: true,
            on_dismiss: None,
            label: None,
        }
    }

    /// Whether a press on the barrier and Escape close the dialog (default true).
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Called when the user dismisses the dialog (barrier or Escape), after it closes.
    pub fn on_dismiss(mut self, f: impl Fn() + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }

    /// The accessible name (an `AlertDialog` names itself after its title).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// What a dialog's parts share.
struct Shared {
    controller: DialogController,
    content: Rc<dyn Fn() -> AnyView>,
    dismissible: bool,
    on_dismiss: Option<Rc<dyn Fn()>>,
    label: Option<String>,
    /// The element focused before the dialog opened.
    previous: Cell<Option<ElementId>>,
}

impl Shared {
    /// Closes the dialog and tells the app it was dismissed (WIDGETS-DIALOG-02, -03).
    fn dismiss(&self) {
        if !self.dismissible {
            return;
        }
        self.controller.close();
        if let Some(f) = &self.on_dismiss {
            f();
        }
    }
}

/// Focuses the pending target once, at a frame, if nothing else took the focus.
struct Restore {
    target: Rc<Cell<Option<ElementId>>>,
}

impl View for Restore {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if let Some(target) = self.target.take() {
            if cx.tree().focused().is_none() {
                cx.focus(target);
            }
        }
        cx.region(|_| {})
    }
}

/// A zero-size element: the dialog's place in the tree (it takes no space).
struct Anchorless;

impl View for Anchorless {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::ZERO)),
            [],
        )
    }
}

impl View for Dialog {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let controller = self.controller;
        let shared = Rc::new(Shared {
            controller,
            content: self.content,
            dismissible: self.dismissible,
            on_dismiss: self.on_dismiss,
            label: self.label,
            previous: Cell::new(None),
        });
        let portal = {
            let shared = Rc::clone(&shared);
            OverlayPortal::new(Anchorless)
                .visible(move || controller.is_open())
                .cover_window(true)
                .overlay(move || DialogFrame {
                    shared: Rc::clone(&shared),
                })
        };
        // The focus goes back when the dialog closes (WIDGETS-DIALOG-03).
        let target: Rc<Cell<Option<ElementId>>> = Rc::default();
        let moves = signal(0u32);
        {
            let (shared, target) = (Rc::clone(&shared), Rc::clone(&target));
            let was_open = Cell::new(false);
            effect(move || {
                let open = controller.is_open();
                if was_open.replace(open) && !open {
                    if let Some(previous) = shared.previous.take() {
                        target.set(Some(previous));
                        untrack(|| moves.update(|m| *m = m.wrapping_add(1)));
                    }
                }
            });
        }
        cx.region(|cx| {
            portal.build(cx);
            Dyn::new(move || {
                moves.get();
                Restore {
                    target: Rc::clone(&target),
                }
            })
            .build(cx);
        })
    }
}

/// The barrier and the centred content (WIDGETS-DIALOG-01..03, -05).
struct DialogFrame {
    shared: Rc<Shared>,
}

/// Holds the dialog's content and reports its own and the content's root element.
struct ContentHolder {
    shared: Rc<Shared>,
    out: Rc<Cell<Option<(ElementId, ElementId)>>>,
}

impl View for ContentHolder {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let content = untrack(|| (self.shared.content)());
        let holder = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [content],
        );
        let root = cx
            .tree()
            .children(holder)
            .first()
            .copied()
            .unwrap_or(holder);
        self.out.set(Some((holder, root)));
        holder
    }
}

impl View for DialogFrame {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let shared = self.shared;
        let out: Rc<Cell<Option<(ElementId, ElementId)>>> = Rc::default();
        let frame = cx.render(
            FrameRender,
            [
                AnyView::new(Barrier {
                    shared: Rc::clone(&shared),
                }),
                AnyView::new(ContentHolder {
                    shared: Rc::clone(&shared),
                    out: Rc::clone(&out),
                }),
            ],
        );
        let Some((holder, root)) = out.get() else {
            return frame;
        };
        // An AlertDialog names itself; any other content gets a modal Dialog around it
        // (WIDGETS-DIALOG-05).
        let alert = cx
            .tree()
            .semantics(root)
            .is_some_and(|s| s.role == Role::AlertDialog);
        let target = if alert { root } else { holder };
        if !alert {
            cx.set_semantics(
                holder,
                Semantics {
                    role: Role::Dialog,
                    name: shared.label.clone(),
                    modal: true,
                    focusable: true,
                    ..Default::default()
                },
            );
        }
        // The dialog takes the focus and keeps Tab inside (WIDGETS-DIALOG-03).
        cx.focusable(
            target,
            FocusOptions {
                traversable: false,
                focus_on_press: false,
            },
        );
        cx.focus_scope(target);
        {
            let shared = Rc::clone(&shared);
            cx.on_key(target, Phase::Bubble, move |k| {
                if k.event.pressed
                    && k.event.modifiers == Modifiers::default()
                    && k.event.key == LogicalKey::Named(NamedKey::Escape)
                    && shared.dismissible
                {
                    shared.dismiss();
                    return Handled::Stop;
                }
                Handled::Continue
            });
        }
        if shared.previous.get().is_none() {
            shared.previous.set(cx.tree().focused());
        }
        cx.focus(target);
        frame
    }
}

/// The barrier fills the window; the content is centred, loose within the window less the
/// margin (WIDGETS-DIALOG-01).
#[derive(Clone, PartialEq)]
struct FrameRender;

impl RenderBox for FrameRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let size = c.biggest();
        let size = if size.width.is_finite() && size.height.is_finite() {
            size
        } else {
            c.smallest()
        };
        if !children.is_empty() {
            children.layout(0, BoxConstraints::tight(size));
            children.set_offset(0, Vec2::ZERO);
        }
        if children.len() > 1 {
            let room = Size::new(
                (size.width - 2.0 * MARGIN).max(0.0),
                (size.height - 2.0 * MARGIN).max(0.0),
            );
            let content = children.layout(1, BoxConstraints::loose(room));
            children.set_offset(
                1,
                Vec2::new(
                    (size.width - content.width) / 2.0,
                    (size.height - content.height) / 2.0,
                ),
            );
        }
        size
    }
}

/// The barrier: dims the window, absorbs the pointer and dismisses on a press.
struct Barrier {
    shared: Rc<Shared>,
}

/// Material's basic dialog layout: a title, a message and buttons (WIDGETS-DIALOG-04).
pub struct AlertDialog {
    title: Option<String>,
    content: Option<AnyView>,
    actions: Vec<AnyView>,
}

impl Default for AlertDialog {
    fn default() -> Self {
        AlertDialog::new()
    }
}

impl AlertDialog {
    /// An empty alert dialog.
    pub fn new() -> Self {
        AlertDialog {
            title: None,
            content: None,
            actions: Vec::new(),
        }
    }

    /// The title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The message or other content under the title.
    pub fn content(mut self, content: impl View) -> Self {
        self.content = Some(AnyView::new(content));
        self
    }

    /// Adds a button (or any view) to the actions row, in order, aligned to the end.
    pub fn action(mut self, action: impl View) -> Self {
        self.actions.push(AnyView::new(action));
        self
    }
}

impl View for AlertDialog {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let parts = [
            self.title.is_some(),
            self.content.is_some(),
            !self.actions.is_empty(),
        ];
        let mut children: Vec<AnyView> = Vec::new();
        if let Some(title) = &self.title {
            children.push(AnyView::new(
                Text::new(title.clone()).style(theme.snapshot().text.title),
            ));
        }
        if let Some(content) = self.content {
            children.push(content);
        }
        if !self.actions.is_empty() {
            children.push(AnyView::new(
                Row::new()
                    .main_axis_size(MainAxisSize::Min)
                    .spacing(ACTION_SPACING)
                    .children(self.actions),
            ));
        }
        let id = cx.render(AlertRender { parts }, children);
        cx.set_paint(
            id,
            AlertPaint {
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<AlertPaint>(|p| p.colors = colors);
        });
        cx.set_semantics(
            id,
            Semantics {
                role: Role::AlertDialog,
                name: self.title,
                modal: true,
                focusable: true,
                ..Default::default()
            },
        );
        id
    }
}

/// Title, content and actions stacked with Material's gaps (WIDGETS-DIALOG-04).
#[derive(Clone, PartialEq)]
struct AlertRender {
    /// Which of title, content and actions are present, in that order.
    parts: [bool; 3],
}

impl RenderBox for AlertRender {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let max_width = MAX_WIDTH.min(c.max_width);
        let inner = (max_width - 2.0 * PADDING).max(0.0);
        let mut natural = 0.0f32;
        let sizes: Vec<Size> = (0..children.len())
            .map(|i| {
                let size = children.layout(i, BoxConstraints::new(0.0, inner, 0.0, f32::INFINITY));
                natural = natural.max(size.width);
                size
            })
            .collect();
        let width = (natural + 2.0 * PADDING).clamp(MIN_WIDTH.min(max_width), max_width);
        let kinds: Vec<usize> = (0..3).filter(|k| self.parts[*k]).collect();
        let mut y = PADDING;
        for (i, (size, kind)) in sizes.iter().zip(&kinds).enumerate() {
            if i > 0 {
                y += if *kind == 2 { ACTIONS_GAP } else { TITLE_GAP };
            }
            let x = if *kind == 2 {
                width - PADDING - size.width
            } else {
                PADDING
            };
            children.set_offset(i, Vec2::new(x, y));
            y += size.height;
        }
        c.constrain(Size::new(width, y + PADDING))
    }
}

/// The alert's surface and shadow.
struct AlertPaint {
    colors: ColorScheme,
}

impl Paint for AlertPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let shape = RoundedRect::new(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            BorderRadius::circular(RADIUS),
        );
        let scene = cx.scene();
        // Material's level-3 elevation.
        for (offset, blur, spread, alpha) in [(1.0, 3.0, 0.0, 0.3), (4.0, 8.0, 3.0, 0.15)] {
            scene.box_shadow(BoxShadow {
                shape,
                color: Color::BLACK.with_alpha(alpha),
                offset: Vec2::new(0.0, offset),
                blur_radius: blur,
                spread_radius: spread,
            });
        }
        scene.fill(shape, self.colors.surface);
    }

    fn overflow(&self) -> f32 {
        14.0
    }
}

impl View for Barrier {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.set_paint(id, BarrierPaint);
        let shared = self.shared;
        // Every pointer event over the barrier stops here (WIDGETS-DIALOG-02).
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if p.event.kind == PointerKind::Down(PointerButton::Primary) {
                shared.dismiss();
            }
            Handled::Stop
        });
        id
    }
}

/// The barrier's dimming.
struct BarrierPaint;

impl Paint for BarrierPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().fill_rect(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            Color::BLACK.with_alpha(BARRIER_ALPHA),
        );
    }
}
