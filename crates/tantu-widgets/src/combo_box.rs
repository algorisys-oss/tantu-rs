//! [`ComboBox`]: a text field with a filtered list of options. Spec:
//! `docs/specs/widgets/combo-box.md`.

use crate::localizations::string;
use std::cell::RefCell;
use std::rc::Rc;
use tantu_i18n::Localization;

use tantu_view::core::{Color, EdgeInsets, Rect, Vec2};
use tantu_view::layout::{Alignment, BoxConstraints, CrossAxisAlignment, RenderConstrainedBox};
use tantu_view::reactive::{Memo, Signal, effect, memo, signal, untrack};
use tantu_view::scene::{BorderRadius, BoxShadow, RoundedRect};
use tantu_view::{
    Anchor, AnyView, BuildCx, Dyn, ElementId, Handled, LogicalKey, Modifiers, NamedKey,
    OverlayPortal, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics, View,
};

use crate::text_field::FieldDecoration;
use crate::theme::ThemeHandle;
use crate::{
    Align, Center, Column, Icon, Icons, Padding, PositionedDirectional, ScrollController,
    ScrollView, SizedBox, Stack, Text, TextEditingController, TextField,
};

/// A text field with a filtered list of options (Flutter's `DropdownMenu` with filtering;
/// WIDGETS-COMBO-01..07).
pub struct ComboBox {
    selected: Signal<Option<usize>>,
    options: Rc<[String]>,
    width: f32,
    placeholder: Option<String>,
    label: Option<String>,
    on_change: Option<Rc<dyn Fn(usize)>>,
}

impl ComboBox {
    /// Shows `options`; `selected` holds the chosen index (`None`: nothing chosen).
    pub fn new(
        selected: Signal<Option<usize>>,
        options: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        ComboBox {
            selected,
            options: options.into_iter().map(Into::into).collect(),
            width: 240.0,
            placeholder: None,
            label: None,
            on_change: None,
        }
    }

    /// The field's width and the popup's (default 240 px).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Shown in the empty field, and its accessible name.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// The label naming the field for assistive technology (default: the placeholder).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Called with the chosen index after the user picks an option.
    pub fn on_change(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// An option row's height (WIDGETS-COMBO-02).
const ROW: f32 = 40.0;
/// How many rows show before the list scrolls.
const VISIBLE_ROWS: usize = 6;
/// The text shown when nothing matches.
const NO_MATCHES: &str = "No matches";

/// What the combo box's parts share.
#[derive(Clone)]
struct Combo {
    selected: Signal<Option<usize>>,
    options: Rc<[String]>,
    controller: TextEditingController,
    open: Signal<bool>,
    /// Whether the user's text filters the list (WIDGETS-COMBO-03).
    filtering: Signal<bool>,
    /// The highlighted row: a position in the listed options.
    highlight: Signal<Option<usize>>,
    listed: Memo<Rc<[usize]>>,
    on_change: Option<Rc<dyn Fn(usize)>>,
    rows: Rc<RefCell<Vec<Option<ElementId>>>>,
    /// Bumped when rows register, so the field's active descendant follows.
    rows_version: Signal<u32>,
    scroll: ScrollController,
}

impl Combo {
    fn label(&self, index: Option<usize>) -> String {
        index
            .and_then(|i| self.options.get(i).cloned())
            .unwrap_or_default()
    }

    /// Opens the list, every option listed, the selection highlighted (WIDGETS-COMBO-02, -04).
    fn open_all(&self) {
        self.filtering.set(false);
        let listed = self.listed.get_untracked();
        let selected = self.selected.get_untracked();
        let position = selected.and_then(|s| listed.iter().position(|i| *i == s));
        self.highlight
            .set(position.or_else(|| (!listed.is_empty()).then_some(0)));
        self.open.set(true);
        self.reveal();
    }

    /// Closes the list and shows the selection's text again (WIDGETS-COMBO-06).
    fn close(&self) {
        if self.open.get_untracked() {
            self.open.set(false);
        }
        self.filtering.set(false);
        let text = self.label(self.selected.get_untracked());
        if self.controller.text() != text {
            self.controller.set_text(text);
        }
    }

    /// Moves the highlight (WIDGETS-COMBO-04).
    fn step(&self, to: impl Fn(Option<usize>, usize) -> usize) {
        let n = self.listed.get_untracked().len();
        if n == 0 {
            return;
        }
        self.highlight
            .set(Some(to(self.highlight.get_untracked(), n)));
        self.reveal();
    }

    /// Scrolls the list so the highlighted row shows.
    fn reveal(&self) {
        let Some(row) = self.highlight.get_untracked() else {
            return;
        };
        let (top, bottom) = (row as f32 * ROW, (row + 1) as f32 * ROW);
        let viewport = VISIBLE_ROWS as f32 * ROW;
        let offset = self.scroll.offset_untracked();
        if top < offset {
            self.scroll.jump_to(top);
        } else if bottom > offset + viewport {
            self.scroll.jump_to(bottom - viewport);
        }
    }

    /// Picks the option at listed position `row` (WIDGETS-COMBO-05).
    fn choose(&self, row: usize) {
        let Some(index) = self.listed.get_untracked().get(row).copied() else {
            return;
        };
        let changed = self.selected.get_untracked() != Some(index);
        if changed {
            self.selected.set(Some(index));
        }
        self.close();
        if changed {
            if let Some(f) = &self.on_change {
                f(index);
            }
        }
    }
}

impl View for ComboBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (selected, options) = (self.selected, self.options);
        let initial = untrack(|| selected.get())
            .and_then(|i| options.get(i).cloned())
            .unwrap_or_default();
        let controller = TextEditingController::new(initial);
        let filtering = signal(false);
        let listed = {
            let options = Rc::clone(&options);
            memo(move || -> Rc<[usize]> {
                if filtering.get() {
                    let query = controller.text().to_lowercase();
                    (0..options.len())
                        .filter(|i| options[*i].to_lowercase().contains(&query))
                        .collect()
                } else {
                    (0..options.len()).collect()
                }
            })
        };
        let combo = Combo {
            selected,
            options,
            controller,
            open: signal(false),
            filtering,
            highlight: signal(None),
            listed,
            on_change: self.on_change,
            rows: Rc::default(),
            rows_version: signal(0),
            scroll: ScrollController::new(),
        };
        // The field follows the selection while the list is closed (WIDGETS-COMBO-01).
        {
            let combo = combo.clone();
            effect(move || {
                let index = combo.selected.get();
                if !untrack(|| combo.open.get()) {
                    let text = combo.label(index);
                    untrack(|| {
                        if combo.controller.text() != text {
                            combo.controller.set_text(text);
                        }
                    });
                }
            });
        }
        let name = self.label.or_else(|| self.placeholder.clone());
        let mut field = TextField::new(controller).decoration(FieldDecoration {
            role: Some(Role::ComboBox),
            expanded: Some({
                let open = combo.open;
                Rc::new(move || open.get())
            }),
            active: Some({
                let (highlight, open, rows, version) = (
                    combo.highlight,
                    combo.open,
                    Rc::clone(&combo.rows),
                    combo.rows_version,
                );
                Rc::new(move || {
                    version.get();
                    if !open.get() {
                        return None;
                    }
                    highlight
                        .get()
                        .and_then(|row| rows.borrow().get(row).copied().flatten())
                })
            }),
            ..FieldDecoration::named(name)
        });
        if let Some(placeholder) = self.placeholder {
            field = field.placeholder(placeholder);
        }
        {
            let combo = combo.clone();
            // Typing filters and opens the list, the first match highlighted (WIDGETS-COMBO-03).
            field = field.on_change(move |_| {
                combo.filtering.set(true);
                let any = !combo.listed.get_untracked().is_empty();
                combo.highlight.set(any.then_some(0));
                combo.scroll.jump_to(0.0);
                if !combo.open.get_untracked() {
                    combo.open.set(true);
                }
            });
        }
        let width = self.width;
        let theme = ThemeHandle::of(cx);
        // At the end edge, so the left under RTL (WIDGETS-RTL-06).
        let arrow = PositionedDirectional::new(
            Center::new().child(
                Icon::new(Icons::ARROW_DROP_DOWN)
                    .size(24.0)
                    .color(move || theme.colors().on_surface_variant),
            ),
        )
        .top(0.0)
        .bottom(0.0)
        .end(8.0)
        .width(24.0);
        // The popup belongs to the field itself, so its options are the field's popup
        // (A11Y-TREE-13); the arrow sits under the field, which takes every press.
        let target = ComboField {
            combo: combo.clone(),
            field,
        };
        let open = combo.open;
        let outside = combo.clone();
        let popup = combo.clone();
        let portal = OverlayPortal::new(target)
            .visible(move || open.get())
            .anchor(Anchor::below())
            .on_outside_press(move || outside.close())
            .overlay(move || Popup {
                combo: popup.clone(),
                width,
            });
        SizedBox::width(width)
            .child(Stack::new().child(arrow).child(portal))
            .build(cx)
    }
}

/// The field and its arrow, with the combo box's keys and presses (WIDGETS-COMBO-02..06).
struct ComboField {
    combo: Combo,
    field: TextField,
}

impl View for ComboField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let combo = self.combo;
        // The handlers sit on the field's own element (capture runs before its handlers).
        let id = self.field.build(cx);
        // A press opens the list; its release selects the text, so typing searches
        // (WIDGETS-COMBO-02, -03). Capture phase: the editor stops the release.
        let opened = Rc::new(std::cell::Cell::new(false));
        {
            let (combo, opened) = (combo.clone(), Rc::clone(&opened));
            cx.on_pointer(id, Phase::Capture, move |p| {
                if p.event.kind == PointerKind::Down(PointerButton::Primary)
                    && !combo.open.get_untracked()
                {
                    combo.open_all();
                    opened.set(true);
                }
                Handled::Continue
            });
        }
        {
            let combo = combo.clone();
            cx.on_pointer(id, Phase::Capture, move |p| {
                if p.event.kind == PointerKind::Up(PointerButton::Primary) && opened.replace(false)
                {
                    combo.controller.edit(|v| v.select_all());
                }
                Handled::Continue
            });
        }
        {
            let combo = combo.clone();
            cx.on_key(id, Phase::Capture, move |k| {
                let event = &k.event;
                if !event.pressed {
                    return Handled::Continue;
                }
                let plain = event.modifiers == Modifiers::default();
                let alt_only = event.modifiers
                    == Modifiers {
                        alt: true,
                        ..Default::default()
                    };
                let open = combo.open.get_untracked();
                match &event.key {
                    LogicalKey::Named(NamedKey::ArrowDown) if !open && (plain || alt_only) => {
                        combo.open_all();
                    }
                    LogicalKey::Named(NamedKey::ArrowDown) if open && plain => {
                        combo.step(|h, n| h.map_or(0, |h| (h + 1) % n));
                    }
                    LogicalKey::Named(NamedKey::ArrowUp) if open && plain => {
                        combo.step(|h, n| h.map_or(n - 1, |h| (h + n - 1) % n));
                    }
                    LogicalKey::Named(NamedKey::Home) if open && plain => combo.step(|_, _| 0),
                    LogicalKey::Named(NamedKey::End) if open && plain => combo.step(|_, n| n - 1),
                    LogicalKey::Named(NamedKey::Enter) if open && plain => {
                        if let Some(row) = combo.highlight.get_untracked() {
                            combo.choose(row);
                        }
                    }
                    LogicalKey::Named(NamedKey::Escape) if open && plain => combo.close(),
                    LogicalKey::Named(NamedKey::Tab) if open => {
                        combo.close();
                        return Handled::Continue;
                    }
                    _ => return Handled::Continue,
                }
                Handled::Stop
            });
        }
        cx.on_focus_within(id, move |inside| {
            if !inside && combo.open.get_untracked() {
                combo.close();
            }
        });
        id
    }
}

/// The list under the field (WIDGETS-COMBO-02, -04, -05, -07).
struct Popup {
    combo: Combo,
    width: f32,
}

impl View for Popup {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (combo, width) = (self.combo, self.width);
        let theme = ThemeHandle::of(cx);
        let loc = Localization::of(cx);
        let rows = {
            let combo = combo.clone();
            Dyn::new(move || {
                let listed = combo.listed.get();
                *combo.rows.borrow_mut() = vec![None; listed.len()];
                let shown = listed.len().clamp(1, VISIBLE_ROWS);
                let mut column = Column::new().cross_axis_alignment(CrossAxisAlignment::Stretch);
                if listed.is_empty() {
                    column = column.child(OptionRow {
                        combo: combo.clone(),
                        row: None,
                        label: string(loc, "tantu-combo-no-matches", None, || {
                            NO_MATCHES.to_owned()
                        }),
                    });
                }
                for (row, index) in listed.iter().enumerate() {
                    column = column.child(OptionRow {
                        combo: combo.clone(),
                        row: Some((row, *index)),
                        label: combo.options[*index].clone(),
                    });
                }
                SizedBox::new(width, shown as f32 * ROW).child(
                    ScrollView::new(column)
                        .controller(combo.scroll)
                        .wheel_duration(std::time::Duration::ZERO),
                )
            })
        };
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [AnyView::new(rows)],
        );
        cx.set_paint(
            id,
            PopupPaint {
                colors: theme.colors(),
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<PopupPaint>(|p| p.colors = colors);
        });
        cx.set_semantics(
            id,
            Semantics {
                role: Role::ListBox,
                ..Default::default()
            },
        );
        id
    }
}

/// One option (or the "No matches" row).
struct OptionRow {
    combo: Combo,
    /// Its listed position and original index; `None` for "No matches".
    row: Option<(usize, usize)>,
    label: String,
}

impl View for OptionRow {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (combo, row) = (self.combo, self.row);
        let theme = ThemeHandle::of(cx);
        let selected = combo.selected;
        let text = Text::new(self.label.clone())
            .style(theme.snapshot().text.label)
            .color(move || {
                let colors = theme.colors();
                match row {
                    None => colors.on_surface_variant,
                    Some((_, index)) if selected.get() == Some(index) => colors.primary,
                    Some(_) => colors.on_surface,
                }
            });
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::new(0.0, f32::INFINITY, ROW, ROW)),
            [AnyView::new(Align::new(Alignment::CENTER_LEFT).child(
                Padding::new(EdgeInsets::symmetric(12.0, 0.0)).child(text),
            ))],
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::ListBoxOption,
                name: Some(self.label),
                disabled: row.is_none(),
                ..Default::default()
            },
        );
        let Some((position, index)) = row else {
            return id;
        };
        if let Some(slot) = combo.rows.borrow_mut().get_mut(position) {
            *slot = Some(id);
        }
        let version = combo.rows_version;
        untrack(|| version.update(|v| *v = v.wrapping_add(1)));
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || selected.get() == Some(index))),
            |e, chosen| {
                e.update_semantics(|s| s.selected = Some(chosen));
            },
        );
        let highlight = combo.highlight;
        cx.set_paint(
            id,
            RowPaint {
                highlighted: false,
                colors: theme.colors(),
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (highlight.get() == Some(position), theme.colors())
            })),
            |e, (on, colors)| {
                e.update_paint::<RowPaint>(|p| {
                    p.highlighted = on;
                    p.colors = colors;
                });
            },
        );
        let pressed = std::cell::Cell::new(false);
        cx.on_pointer(id, Phase::Bubble, move |p| {
            match p.event.kind {
                PointerKind::Enter => combo.highlight.set(Some(position)),
                PointerKind::Down(PointerButton::Primary) => pressed.set(true),
                PointerKind::Up(PointerButton::Primary) => {
                    let inside = p.local.x >= 0.0
                        && p.local.y >= 0.0
                        && p.local.x <= p.size.width
                        && p.local.y <= p.size.height;
                    if pressed.replace(false) && inside {
                        combo.choose(position);
                    }
                }
                _ => return Handled::Continue,
            }
            Handled::Stop
        });
        id
    }
}

/// An option's highlight layer.
struct RowPaint {
    highlighted: bool,
    colors: tantu_theme::ColorScheme,
}

impl Paint for RowPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        if self.highlighted {
            let size = cx.size();
            cx.scene().fill_rect(
                Rect::from_ltrb(0.0, 0.0, size.width, size.height),
                self.colors.on_surface.with_alpha(0.08),
            );
        }
    }
}

/// The popup's surface and shadow (shared with the date picker's popup).
pub(crate) struct PopupPaint {
    pub(crate) colors: tantu_theme::ColorScheme,
}

impl Paint for PopupPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let shape = RoundedRect::new(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            BorderRadius::circular(4.0),
        );
        let scene = cx.scene();
        for (offset, blur, spread, alpha) in [(1.0, 2.0, 0.0, 0.3), (2.0, 6.0, 2.0, 0.15)] {
            scene.box_shadow(BoxShadow {
                shape,
                color: Color::BLACK.with_alpha(alpha),
                offset: Vec2::new(0.0, offset),
                blur_radius: blur,
                spread_radius: spread,
            });
        }
        scene.fill(shape, self.colors.surface);
        cx.paint_children();
    }

    fn overflow(&self) -> f32 {
        10.0
    }
}
