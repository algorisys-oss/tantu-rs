//! [`Toolbar`] and [`StatusBar`]. Spec: `docs/specs/widgets/menu-bar.md`.

use std::cell::Cell;
use std::rc::Rc;

use tantu_theme::ColorScheme;
use tantu_view::core::{Color, EdgeInsets, Rect, Size};
use tantu_view::layout::{BoxConstraints, CrossAxisAlignment, RenderConstrainedBox};
use tantu_view::reactive::{Signal, signal, untrack};
use tantu_view::{
    AnyView, BuildCx, Dyn, ElementId, Handled, LogicalKey, Modifiers, NamedKey, Paint, PaintCx,
    Phase, Role, Semantics, View,
};

use crate::theme::ThemeHandle;
use crate::{Padding, Row, Spacer};

/// A toolbar's minimum height (WIDGETS-TOOLBAR-01).
const TOOLBAR_HEIGHT: f32 = 48.0;
/// Space between toolbar entries.
const TOOLBAR_GAP: f32 = 4.0;
/// A toolbar's side padding.
const TOOLBAR_PADDING: f32 = 8.0;
/// A separator's line height and its margin on each side.
const SEPARATOR_HEIGHT: f32 = 24.0;
const SEPARATOR_MARGIN: f32 = 4.0;
/// A status bar's height (WIDGETS-STATUSBAR-01).
const STATUS_HEIGHT: f32 = 28.0;
/// Space between status bar items, and from the edges.
const STATUS_GAP: f32 = 16.0;
const STATUS_PADDING: f32 = 12.0;

/// One entry of a toolbar.
enum ToolbarEntry {
    Child(AnyView),
    Separator,
}

/// A row of tool controls with separators (WIDGETS-TOOLBAR-01..03).
pub struct Toolbar {
    entries: Vec<ToolbarEntry>,
}

impl Default for Toolbar {
    fn default() -> Self {
        Toolbar::new()
    }
}

impl Toolbar {
    /// An empty toolbar.
    pub fn new() -> Self {
        Toolbar {
            entries: Vec::new(),
        }
    }

    /// Adds a control.
    pub fn child(mut self, child: impl View) -> Self {
        self.entries.push(ToolbarEntry::Child(AnyView::new(child)));
        self
    }

    /// A vertical separator between groups of controls.
    pub fn separator(mut self) -> Self {
        self.entries.push(ToolbarEntry::Separator);
        self
    }
}

/// What the toolbar's slots share: each control's element, the one holding the focus, and
/// where the focus should go next.
#[derive(Clone)]
struct Slots {
    controls: Rc<std::cell::RefCell<Vec<Option<ElementId>>>>,
    focused: Rc<Cell<Option<usize>>>,
    target: Rc<Cell<Option<ElementId>>>,
    moves: Signal<u32>,
}

/// A control in a toolbar, tracking whether the focus is inside it (WIDGETS-TOOLBAR-02).
struct Slot {
    index: usize,
    child: AnyView,
    slots: Slots,
}

impl View for Slot {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let control = self.child.build(cx);
        if let Some(entry) = self.slots.controls.borrow_mut().get_mut(self.index) {
            *entry = Some(control);
        }
        let (index, focused) = (self.index, Rc::clone(&self.slots.focused));
        cx.on_focus_within(control, move |inside| {
            if inside {
                focused.set(Some(index));
            } else if focused.get() == Some(index) {
                focused.set(None);
            }
        });
        control
    }
}

/// A separator: a 1 × 24 px line with 4 px on each side.
struct Separator;

impl View for Separator {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(
                2.0 * SEPARATOR_MARGIN + 1.0,
                SEPARATOR_HEIGHT,
            ))),
            [],
        );
        cx.set_paint(
            id,
            Line {
                color: theme.colors().outline_variant,
            },
        );
        cx.bind(id, theme.color_prop(|c| c.outline_variant), |e, color| {
            e.update_paint::<Line>(|p| p.color = color);
        });
        id
    }
}

/// A vertical 1 px line in the middle of its element.
struct Line {
    color: Color,
}

impl Paint for Line {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let x = (size.width - 1.0) / 2.0;
        cx.scene()
            .fill_rect(Rect::from_ltrb(x, 0.0, x + 1.0, size.height), self.color);
    }
}

/// Focuses the pending target once, at a frame.
struct FocusMove {
    target: Rc<Cell<Option<ElementId>>>,
}

impl View for FocusMove {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if let Some(target) = self.target.take() {
            cx.focus(target);
        }
        cx.region(|_| {})
    }
}

impl View for Toolbar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let count = self.entries.len();
        let slots = Slots {
            controls: Rc::new(std::cell::RefCell::new(vec![None; count])),
            focused: Rc::default(),
            target: Rc::default(),
            moves: signal(0),
        };
        let mut row = Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(TOOLBAR_GAP);
        for (index, entry) in self.entries.into_iter().enumerate() {
            row = match entry {
                ToolbarEntry::Child(child) => row.child(Slot {
                    index,
                    child,
                    slots: slots.clone(),
                }),
                ToolbarEntry::Separator => row.child(Separator),
            };
        }
        let content = Padding::new(EdgeInsets::symmetric(TOOLBAR_PADDING, 0.0)).child(row);
        let focus_move = {
            let (target, moves) = (Rc::clone(&slots.target), slots.moves);
            Dyn::new(move || {
                moves.get();
                FocusMove {
                    target: Rc::clone(&target),
                }
            })
        };
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::new(
                0.0,
                f32::INFINITY,
                TOOLBAR_HEIGHT,
                f32::INFINITY,
            )),
            [AnyView::new(content), AnyView::new(focus_move)],
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Toolbar,
                ..Default::default()
            },
        );
        // Arrows move the focus between controls (WIDGETS-TOOLBAR-02).
        cx.on_key(id, Phase::Bubble, move |k| {
            let event = &k.event;
            if !event.pressed || event.modifiers != Modifiers::default() {
                return Handled::Continue;
            }
            let Some(current) = slots.focused.get() else {
                return Handled::Continue;
            };
            let controls: Vec<(usize, ElementId)> = slots
                .controls
                .borrow()
                .iter()
                .enumerate()
                .filter_map(|(i, c)| c.map(|c| (i, c)))
                .collect();
            let Some(position) = controls.iter().position(|(i, _)| *i == current) else {
                return Handled::Continue;
            };
            let n = controls.len();
            let next = match &event.key {
                LogicalKey::Named(NamedKey::ArrowLeft) => (position + n - 1) % n,
                LogicalKey::Named(NamedKey::ArrowRight) => (position + 1) % n,
                LogicalKey::Named(NamedKey::Home) => 0,
                LogicalKey::Named(NamedKey::End) => n - 1,
                _ => return Handled::Continue,
            };
            slots.target.set(Some(controls[next].1));
            untrack(|| slots.moves.update(|m| *m = m.wrapping_add(1)));
            Handled::Stop
        });
        id
    }
}

/// A strip along the window's bottom: leading items on the left, trailing on the right
/// (WIDGETS-STATUSBAR-01..02).
pub struct StatusBar {
    leading: Vec<AnyView>,
    trailing: Vec<AnyView>,
}

impl Default for StatusBar {
    fn default() -> Self {
        StatusBar::new()
    }
}

impl StatusBar {
    /// An empty status bar.
    pub fn new() -> Self {
        StatusBar {
            leading: Vec::new(),
            trailing: Vec::new(),
        }
    }

    /// Adds an item on the left.
    pub fn leading(mut self, item: impl View) -> Self {
        self.leading.push(AnyView::new(item));
        self
    }

    /// Adds an item on the right.
    pub fn trailing(mut self, item: impl View) -> Self {
        self.trailing.push(AnyView::new(item));
        self
    }
}

/// The status bar's background and top line.
struct StatusPaint {
    colors: ColorScheme,
}

impl Paint for StatusPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let scene = cx.scene();
        scene.fill_rect(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            self.colors.surface_container_highest,
        );
        scene.fill_rect(
            Rect::from_ltrb(0.0, 0.0, size.width, 1.0),
            self.colors.outline_variant,
        );
    }
}

impl View for StatusBar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let leading = Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(STATUS_GAP)
            .children(self.leading);
        let trailing = Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(STATUS_GAP)
            .children(self.trailing);
        let content = Padding::new(EdgeInsets::symmetric(STATUS_PADDING, 0.0)).child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .child(leading)
                .child(Spacer::new())
                .child(trailing),
        );
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::new(
                0.0,
                f32::INFINITY,
                STATUS_HEIGHT,
                STATUS_HEIGHT,
            )),
            [AnyView::new(content)],
        );
        cx.set_paint(
            id,
            StatusPaint {
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<StatusPaint>(|p| p.colors = colors);
        });
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Status,
                ..Default::default()
            },
        );
        id
    }
}
