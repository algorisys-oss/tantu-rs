//! [`CalendarDatePicker`] and [`DatePickerField`]. Spec:
//! `docs/specs/widgets/date-picker.md`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_view::core::{EdgeInsets, Rect, Size};
use tantu_view::layout::{Alignment, BoxConstraints, CrossAxisAlignment, RenderConstrainedBox};
use tantu_view::reactive::{Signal, effect, signal, untrack};
use tantu_view::scene::{BorderRadius, RoundedRect};
use tantu_view::{
    Anchor, AnyView, BuildCx, Dyn, ElementId, FocusOptions, Handled, KeyEvent, LogicalKey,
    Modifiers, NamedKey, OverlayPortal, Paint, PaintCx, Phase, PointerButton, PointerKind, Prop,
    Role, Semantics, View,
};

use crate::combo_box::PopupPaint;
use crate::date::{Date, Weekday};
use crate::formatter::MaskFormatter;
use crate::text_field::FieldDecoration;
use crate::theme::{ALPHA_12, ALPHA_38, ThemeHandle, at};
use crate::{
    Align, Center, Column, Expanded, IconButton, Icons, Padding, Row, SizedBox, Text,
    TextEditingController, TextField,
};

/// A day cell's side, and the calendar's width (WIDGETS-DATE-02).
const CELL: f32 = 40.0;
const HEADER: f32 = 48.0;
const WEEKDAY_ROW: f32 = 32.0;
const WEEKS: usize = 6;
const WIDTH: f32 = CELL * 7.0;
const HEIGHT: f32 = HEADER + WEEKDAY_ROW + CELL * WEEKS as f32;
/// The day circle's diameter (WIDGETS-DATE-03).
const CIRCLE: f32 = 36.0;
/// The calendar button's width beside the field (WIDGETS-DATE-07).
const BUTTON: f32 = 40.0;
/// The field's error description.
const INVALID: &str = "Invalid date";

/// The range and markers a calendar shares with its field.
#[derive(Clone)]
struct Options {
    first: Date,
    last: Date,
    today: Option<Date>,
    first_day: Weekday,
    on_change: Option<Rc<dyn Fn(Date)>>,
}

impl Options {
    fn new() -> Self {
        Options {
            first: Date::from_ymd_unchecked(1900, 1, 1),
            last: Date::from_ymd_unchecked(2100, 12, 31),
            today: None,
            first_day: Weekday::Monday,
            on_change: None,
        }
    }

    /// The options with a well-ordered range.
    fn normalized(mut self) -> Self {
        if self.last < self.first {
            self.last = self.first;
        }
        self
    }
}

/// What happens to a calendar key (WIDGETS-DATE-05).
#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyOutcome {
    Ignored,
    Moved,
    Chose,
}

/// What a calendar's parts share.
#[derive(Clone)]
struct Calendar {
    selected: Signal<Option<Date>>,
    options: Rc<Options>,
    /// The displayed (year, month).
    month: Signal<(i32, u32)>,
    cursor: Signal<Date>,
    hover: Signal<Option<Date>>,
    /// Whether the keyboard cursor shows (the picker or its field has the focus).
    show_cursor: Signal<bool>,
    /// The displayed month's day cells, by day.
    cells: Rc<RefCell<Vec<Option<ElementId>>>>,
    /// Bumped when cells register, so active descendants follow.
    cells_version: Signal<u32>,
    /// Called after a day is chosen (the field closes its popup).
    after_choose: Option<Rc<dyn Fn(Date)>>,
}

impl Calendar {
    fn new(
        selected: Signal<Option<Date>>,
        options: Rc<Options>,
        after_choose: Option<Rc<dyn Fn(Date)>>,
    ) -> Self {
        let start = Self::start(&options, untrack(|| selected.get()));
        Calendar {
            selected,
            month: signal((start.year(), start.month())),
            cursor: signal(start),
            hover: signal(None),
            show_cursor: signal(false),
            cells: Rc::default(),
            cells_version: signal(0),
            options,
            after_choose,
        }
    }

    /// The cursor's first place: the selection, else today, else the first date, clamped
    /// (WIDGETS-DATE-02, -05).
    fn start(options: &Options, selected: Option<Date>) -> Date {
        let date = selected.or(options.today).unwrap_or(options.first);
        date.max(options.first).min(options.last)
    }

    /// Puts the cursor and the displayed month back on the selection (the field's popup).
    fn reset(&self) {
        let start = Self::start(&self.options, self.selected.get_untracked());
        self.move_to(start);
    }

    fn in_range(&self, date: Date) -> bool {
        (self.options.first..=self.options.last).contains(&date)
    }

    /// Moves the cursor to `date` (clamped), the displayed month following.
    fn move_to(&self, date: Date) {
        let date = date.max(self.options.first).min(self.options.last);
        if self.cursor.get_untracked() != date {
            self.cursor.set(date);
        }
        let month = (date.year(), date.month());
        if self.month.get_untracked() != month {
            self.month.set(month);
        }
    }

    /// Moves the cursor by `months` (the header buttons).
    fn step_month(&self, months: i32) {
        let cursor = self.cursor.get_untracked();
        if let Some(date) = cursor.add_months(months) {
            self.move_to(date);
        }
    }

    /// Chooses `date` if it can be chosen (WIDGETS-DATE-04).
    fn choose(&self, date: Date) -> bool {
        if !self.in_range(date) {
            return false;
        }
        self.move_to(date);
        let changed = self.selected.get_untracked() != Some(date);
        if changed {
            self.selected.set(Some(date));
        }
        if let Some(f) = &self.after_choose {
            f(date);
        }
        if changed {
            if let Some(f) = &self.options.on_change {
                f(date);
            }
        }
        true
    }

    /// A key press on the picker, or on the field while its popup is open (WIDGETS-DATE-05).
    fn key(&self, event: &KeyEvent) -> KeyOutcome {
        let shift = Modifiers {
            shift: true,
            ..Modifiers::default()
        };
        let plain = event.modifiers == Modifiers::default();
        let cursor = self.cursor.get_untracked();
        let target = match &event.key {
            LogicalKey::Named(NamedKey::ArrowLeft) if plain => cursor.add_days(-1),
            LogicalKey::Named(NamedKey::ArrowRight) if plain => cursor.add_days(1),
            LogicalKey::Named(NamedKey::ArrowUp) if plain => cursor.add_days(-7),
            LogicalKey::Named(NamedKey::ArrowDown) if plain => cursor.add_days(7),
            LogicalKey::Named(NamedKey::PageUp) if plain => cursor.add_months(-1),
            LogicalKey::Named(NamedKey::PageDown) if plain => cursor.add_months(1),
            LogicalKey::Named(NamedKey::PageUp) if event.modifiers == shift => {
                cursor.add_months(-12)
            }
            LogicalKey::Named(NamedKey::PageDown) if event.modifiers == shift => {
                cursor.add_months(12)
            }
            LogicalKey::Named(NamedKey::Home) if plain => cursor.add_days(1 - cursor.day() as i64),
            LogicalKey::Named(NamedKey::End) if plain => {
                let last = Date::days_in_month(cursor.year(), cursor.month());
                cursor.add_days(i64::from(last) - i64::from(cursor.day()))
            }
            LogicalKey::Named(NamedKey::Enter) if plain => {
                self.choose(cursor);
                return KeyOutcome::Chose;
            }
            LogicalKey::Character(c) if plain && &**c == " " => {
                self.choose(cursor);
                return KeyOutcome::Chose;
            }
            _ => return KeyOutcome::Ignored,
        };
        if let Some(date) = target {
            self.move_to(date);
        }
        KeyOutcome::Moved
    }

    /// "October 2026" (tracked).
    fn month_label(&self) -> String {
        let (year, month) = self.month.get();
        let first = Date::new(year, month, 1).unwrap_or(self.options.first);
        format!("{} {year}", first.month_name())
    }

    /// The cursor's cell, if the cursor's month is displayed (tracked).
    fn cursor_cell(&self) -> Option<ElementId> {
        self.cells_version.get();
        let cursor = self.cursor.get();
        if self.month.get() != (cursor.year(), cursor.month()) {
            return None;
        }
        self.cells
            .borrow()
            .get(cursor.day() as usize - 1)
            .copied()
            .flatten()
    }
}

/// "Friday, October 16, 2026".
fn long_name(date: Date) -> String {
    format!(
        "{}, {} {}, {}",
        date.weekday().name(),
        date.month_name(),
        date.day(),
        date.year()
    )
}

/// A month calendar for choosing a date (Flutter's `CalendarDatePicker`,
/// WIDGETS-DATE-02..06).
pub struct CalendarDatePicker {
    selected: Signal<Option<Date>>,
    options: Options,
}

impl CalendarDatePicker {
    /// A calendar choosing into `selected`.
    pub fn new(selected: Signal<Option<Date>>) -> Self {
        CalendarDatePicker {
            selected,
            options: Options::new(),
        }
    }

    /// The earliest choosable date (default 1900-01-01).
    pub fn first_date(mut self, date: Date) -> Self {
        self.options.first = date;
        self
    }

    /// The latest choosable date (default 2100-12-31).
    pub fn last_date(mut self, date: Date) -> Self {
        self.options.last = date;
        self
    }

    /// The date marked as today (default: none marked).
    pub fn today(mut self, date: Date) -> Self {
        self.options.today = Some(date);
        self
    }

    /// The first column's weekday (default Monday).
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.options.first_day = day;
        self
    }

    /// Called with the chosen date after the user picks one that differs.
    pub fn on_change(mut self, f: impl Fn(Date) + 'static) -> Self {
        self.options.on_change = Some(Rc::new(f));
        self
    }
}

impl View for CalendarDatePicker {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let calendar = Calendar::new(self.selected, Rc::new(self.options.normalized()), None);
        CalendarView {
            calendar,
            focusable: true,
        }
        .build(cx)
    }
}

/// The calendar grid, standalone (focusable, with its own keys) or in the field's popup.
struct CalendarView {
    calendar: Calendar,
    focusable: bool,
}

impl View for CalendarView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let cal = self.calendar;
        let theme = ThemeHandle::of(cx);
        let options = Rc::clone(&cal.options);
        let month_index = |date: Date| i64::from(date.year()) * 12 + i64::from(date.month());
        let shown = {
            let month = cal.month;
            move || {
                let (year, m) = month.get();
                i64::from(year) * 12 + i64::from(m)
            }
        };
        let header = SizedBox::new(WIDTH, HEADER).child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .child(Expanded::new(
                    Padding::new(EdgeInsets::from_ltrb(12.0, 0.0, 0.0, 0.0)).child(
                        Align::new(Alignment::CENTER_LEFT).child(
                            Text::new({
                                let cal = cal.clone();
                                move || cal.month_label()
                            })
                            .style(tantu_view::text::TextStyle {
                                size: 16.0,
                                ..theme.snapshot().text.title
                            })
                            .color(move || theme.colors().on_surface),
                        ),
                    ),
                ))
                .child(
                    IconButton::new(Icons::CHEVRON_LEFT)
                        .semantics_label("Previous month")
                        .enabled({
                            let first = month_index(options.first);
                            move || shown() > first
                        })
                        .on_press({
                            let cal = cal.clone();
                            move || cal.step_month(-1)
                        }),
                )
                .child(
                    IconButton::new(Icons::CHEVRON_RIGHT)
                        .semantics_label("Next month")
                        .enabled({
                            let last = month_index(options.last);
                            move || shown() < last
                        })
                        .on_press({
                            let cal = cal.clone();
                            move || cal.step_month(1)
                        }),
                ),
        );
        let mut weekdays = Row::new();
        for column in 0..7 {
            weekdays = weekdays.child(WeekdayCell {
                day: options.first_day.plus(column),
                theme,
            });
        }
        let body = {
            let cal = cal.clone();
            Dyn::new(move || {
                let (year, month) = cal.month.get();
                *cal.cells.borrow_mut() = vec![None; 31];
                let first = Date::new(year, month, 1).unwrap_or(cal.options.first);
                let offset = (first.weekday().index() + 7 - cal.options.first_day.index()) % 7;
                let days = Date::days_in_month(year, month) as usize;
                let mut weeks = Column::new();
                for row in 0..WEEKS {
                    let mut week = Row::new();
                    for column in 0..7 {
                        let slot = row * 7 + column;
                        let date = (slot >= offset && slot - offset < days)
                            .then(|| first.add_days((slot - offset) as i64))
                            .flatten();
                        week = week.child(match date {
                            Some(date) => AnyView::new(DayCell {
                                cal: cal.clone(),
                                date,
                                row,
                                column,
                                theme,
                            }),
                            None => AnyView::new(SizedBox::new(CELL, CELL)),
                        });
                    }
                    weeks = weeks.child(week);
                }
                weeks
            })
        };
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(WIDTH, HEIGHT))),
            [AnyView::new(
                Column::new()
                    .child(header)
                    .child(SizedBox::new(WIDTH, WEEKDAY_ROW).child(weekdays))
                    .child(body),
            )],
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Grid,
                row_count: Some(WEEKS),
                column_count: Some(7),
                ..Default::default()
            },
        );
        {
            let cal = cal.clone();
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || (cal.month_label(), cal.cursor_cell()))),
                |e, (name, active)| {
                    e.update_semantics(|s| {
                        s.name = Some(name);
                        s.active_descendant = active;
                    });
                },
            );
        }
        if self.focusable {
            cx.focusable(
                id,
                FocusOptions {
                    traversable: true,
                    focus_on_press: true,
                },
            );
            {
                let cal = cal.clone();
                cx.on_key(id, Phase::Bubble, move |k| {
                    if !k.event.pressed || cal.key(k.event) == KeyOutcome::Ignored {
                        return Handled::Continue;
                    }
                    Handled::Stop
                });
            }
            let show = cal.show_cursor;
            cx.on_focus_change(id, move |focused| show.set(focused));
        }
        id
    }
}

/// A weekday's short name over a column (WIDGETS-DATE-02, -06).
struct WeekdayCell {
    day: Weekday,
    theme: ThemeHandle,
}

impl View for WeekdayCell {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = self.theme;
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(CELL, WEEKDAY_ROW))),
            [AnyView::new(
                Center::new().child(
                    Text::new(self.day.short_name().to_owned())
                        .style(theme.snapshot().text.label)
                        .color(move || theme.colors().on_surface_variant),
                ),
            )],
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::ColumnHeader,
                name: Some(self.day.name().to_owned()),
                ..Default::default()
            },
        );
        id
    }
}

/// One day of the displayed month (WIDGETS-DATE-03, -04, -06).
struct DayCell {
    cal: Calendar,
    date: Date,
    row: usize,
    column: usize,
    theme: ThemeHandle,
}

impl View for DayCell {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (cal, date, theme) = (self.cal, self.date, self.theme);
        let enabled = cal.in_range(date);
        let today = cal.options.today == Some(date);
        let selected = cal.selected;
        let text = Text::new(date.day().to_string())
            .style(theme.snapshot().text.label)
            .color(move || {
                let colors = theme.colors();
                if selected.get() == Some(date) {
                    colors.on_primary
                } else if !enabled {
                    at(colors.on_surface, ALPHA_38)
                } else if today {
                    colors.primary
                } else {
                    colors.on_surface
                }
            });
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(CELL, CELL))),
            [AnyView::new(Center::new().child(text))],
        );
        if let Some(slot) = cal.cells.borrow_mut().get_mut(date.day() as usize - 1) {
            *slot = Some(id);
        }
        let version = cal.cells_version;
        untrack(|| version.update(|v| *v = v.wrapping_add(1)));
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Cell,
                name: Some(long_name(date)),
                row_index: Some(self.row),
                column_index: Some(self.column),
                disabled: !enabled,
                ..Default::default()
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || selected.get() == Some(date))),
            |e, chosen| {
                e.update_semantics(|s| s.selected = Some(chosen));
            },
        );
        cx.set_paint(
            id,
            DayPaint {
                selected: false,
                today,
                enabled,
                hovered: false,
                cursor: false,
                colors: theme.snapshot().colors,
            },
        );
        {
            let (hover, cursor, show) = (cal.hover, cal.cursor, cal.show_cursor);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    (
                        selected.get() == Some(date),
                        hover.get() == Some(date),
                        show.get() && cursor.get() == date,
                        theme.colors(),
                    )
                })),
                |e, (selected, hovered, cursor, colors)| {
                    e.update_paint::<DayPaint>(|p| {
                        p.selected = selected;
                        p.hovered = hovered;
                        p.cursor = cursor;
                        p.colors = colors;
                    });
                },
            );
        }
        let pressed = Cell::new(false);
        cx.on_pointer(id, Phase::Bubble, move |p| {
            match p.event.kind {
                PointerKind::Enter => cal.hover.set(Some(date)),
                PointerKind::Leave => {
                    if cal.hover.get_untracked() == Some(date) {
                        cal.hover.set(None);
                    }
                }
                PointerKind::Down(PointerButton::Primary) => pressed.set(true),
                PointerKind::Up(PointerButton::Primary) => {
                    let inside = p.local.x >= 0.0
                        && p.local.y >= 0.0
                        && p.local.x <= p.size.width
                        && p.local.y <= p.size.height;
                    if pressed.replace(false) && inside {
                        cal.choose(date);
                    }
                }
                _ => return Handled::Continue,
            }
            Handled::Stop
        });
        id
    }
}

/// A day's circle: selection fill, today's outline, hover and cursor layers.
struct DayPaint {
    selected: bool,
    today: bool,
    enabled: bool,
    hovered: bool,
    cursor: bool,
    colors: tantu_theme::ColorScheme,
}

impl Paint for DayPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let inset = ((size.width - CIRCLE) / 2.0, (size.height - CIRCLE) / 2.0);
        let circle = RoundedRect::new(
            Rect::from_ltrb(inset.0, inset.1, inset.0 + CIRCLE, inset.1 + CIRCLE),
            BorderRadius::circular(CIRCLE / 2.0),
        );
        let colors = self.colors;
        let scene = cx.scene();
        if self.selected {
            scene.fill(circle, colors.primary);
        } else if self.today {
            scene.stroke(circle, 1.0, colors.primary);
        }
        if self.enabled && self.cursor {
            scene.fill(circle, at(colors.on_surface, ALPHA_12));
        } else if self.enabled && self.hovered {
            scene.fill(circle, colors.on_surface.with_alpha(0.08));
        }
    }
}

/// A date typed as `YYYY-MM-DD` or picked from a calendar popup (WIDGETS-DATE-07..09).
pub struct DatePickerField {
    selected: Signal<Option<Date>>,
    options: Options,
    label: String,
    width: f32,
}

impl DatePickerField {
    /// A field choosing into `selected`.
    pub fn new(selected: Signal<Option<Date>>) -> Self {
        DatePickerField {
            selected,
            options: Options::new(),
            label: "Date".to_owned(),
            width: 240.0,
        }
    }

    /// The accessible name (default "Date").
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// The field's width, the button included (default 240 px).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// The earliest choosable date (default 1900-01-01).
    pub fn first_date(mut self, date: Date) -> Self {
        self.options.first = date;
        self
    }

    /// The latest choosable date (default 2100-12-31).
    pub fn last_date(mut self, date: Date) -> Self {
        self.options.last = date;
        self
    }

    /// The date marked as today in the calendar (default: none marked).
    pub fn today(mut self, date: Date) -> Self {
        self.options.today = Some(date);
        self
    }

    /// The calendar's first column (default Monday).
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.options.first_day = day;
        self
    }

    /// Called with the chosen date after the user picks or types one that differs.
    pub fn on_change(mut self, f: impl Fn(Date) + 'static) -> Self {
        self.options.on_change = Some(Rc::new(f));
        self
    }
}

/// What the date field's parts share.
#[derive(Clone)]
struct Field {
    selected: Signal<Option<Date>>,
    controller: TextEditingController,
    open: Signal<bool>,
    invalid: Signal<bool>,
    calendar: Calendar,
}

impl Field {
    fn text(date: Option<Date>) -> String {
        date.map(|d| d.to_string()).unwrap_or_default()
    }

    fn open(&self) {
        self.calendar.reset();
        self.calendar.show_cursor.set(true);
        self.open.set(true);
    }

    fn close(&self) {
        if self.open.get_untracked() {
            self.open.set(false);
        }
        self.calendar.show_cursor.set(false);
    }

    /// Shows the selected date again and clears the error (WIDGETS-DATE-07).
    fn restore(&self) {
        let text = Self::text(self.selected.get_untracked());
        if self.controller.text() != text {
            self.controller.set_text(text);
        }
        if self.invalid.get_untracked() {
            self.invalid.set(false);
        }
    }
}

impl View for DatePickerField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let selected = self.selected;
        let options = Rc::new(self.options.normalized());
        let controller = TextEditingController::new(Field::text(untrack(|| selected.get())));
        let (open, invalid, show_cursor) = (signal(false), signal(false), signal(false));
        // Choosing a day closes the popup and shows the date (WIDGETS-DATE-08).
        let after_choose: Rc<dyn Fn(Date)> = Rc::new(move |date: Date| {
            open.set(false);
            show_cursor.set(false);
            controller.set_text(date.to_string());
            invalid.set(false);
        });
        let mut calendar = Calendar::new(selected, Rc::clone(&options), Some(after_choose));
        calendar.show_cursor = show_cursor;
        let field = Field {
            selected,
            controller,
            open,
            invalid,
            calendar,
        };
        let focused = signal(false);
        // The text follows the selection while the field doesn't have the focus.
        {
            let field = field.clone();
            effect(move || {
                let date = selected.get();
                if !untrack(|| focused.get()) {
                    let text = Field::text(date);
                    untrack(|| {
                        if field.controller.text() != text {
                            field.controller.set_text(text);
                        }
                    });
                }
            });
        }
        let mask = MaskFormatter::new("####-##-##");
        let decoration = FieldDecoration {
            error: Rc::new(move || invalid.get().then(|| INVALID.to_owned())),
            role: Some(Role::ComboBox),
            expanded: Some(Rc::new(move || open.get())),
            active: Some({
                let calendar = field.calendar.clone();
                Rc::new(move || {
                    if open.get() {
                        calendar.cursor_cell()
                    } else {
                        None
                    }
                })
            }),
            ..FieldDecoration::named(Some(self.label))
        };
        let text_field = TextField::new(controller)
            .placeholder("YYYY-MM-DD")
            .input_formatter(mask.clone())
            .decoration(decoration)
            .on_change({
                let field = field.clone();
                move |text| {
                    if !mask.is_complete(&text) {
                        field.invalid.set(false);
                        return;
                    }
                    let date = Date::parse_iso(&text).filter(|d| field.calendar.in_range(*d));
                    field.invalid.set(date.is_none());
                    let Some(date) = date else {
                        return;
                    };
                    if field.open.get_untracked() {
                        field.calendar.move_to(date);
                    }
                    if field.selected.get_untracked() != Some(date) {
                        field.selected.set(Some(date));
                        if let Some(f) = &field.calendar.options.on_change {
                            f(date);
                        }
                    }
                }
            });
        let field_id: Rc<Cell<Option<ElementId>>> = Rc::default();
        let focus_request = signal(0u32);
        let button = IconButton::new(Icons::CALENDAR_TODAY)
            .semantics_label("Choose date")
            .on_press({
                let field = field.clone();
                move || {
                    if field.open.get_untracked() {
                        field.close();
                    } else {
                        field.open();
                        focus_request.update(|n| *n = n.wrapping_add(1));
                    }
                }
            });
        let restore_focus = {
            let field_id = Rc::clone(&field_id);
            Dyn::new(move || FocusRequest {
                target: field_id.get(),
                pending: focus_request.get() > 0,
            })
        };
        // The portal's target holds the field and its button (WIDGETS-DATE-09).
        let target = SizedBox::width(self.width).child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .child(Expanded::new(DateField {
                    field: field.clone(),
                    text_field,
                    focused,
                    out: field_id,
                }))
                .child(SizedBox::width(BUTTON).child(Center::new().child(button)))
                .child(restore_focus),
        );
        let outside = field.clone();
        let popup = field.calendar.clone();
        OverlayPortal::new(target)
            .visible(move || open.get())
            .anchor(Anchor::below())
            .on_outside_press(move || outside.close())
            .overlay(move || DatePopup {
                calendar: popup.clone(),
            })
            .build(cx)
    }
}

/// Focuses the field after the calendar button opened the popup.
struct FocusRequest {
    target: Option<ElementId>,
    pending: bool,
}

impl View for FocusRequest {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if let (true, Some(target)) = (self.pending, self.target) {
            cx.focus(target);
        }
        cx.region(|_| {})
    }
}

/// The text field with the date field's keys and focus handling (WIDGETS-DATE-07, -08).
struct DateField {
    field: Field,
    text_field: TextField,
    focused: Signal<bool>,
    out: Rc<Cell<Option<ElementId>>>,
}

impl View for DateField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let field = self.field;
        let id = self.text_field.build(cx);
        self.out.set(Some(id));
        {
            let field = field.clone();
            cx.on_key(id, Phase::Capture, move |k| {
                let event = k.event;
                if !event.pressed {
                    return Handled::Continue;
                }
                let alt = Modifiers {
                    alt: true,
                    ..Modifiers::default()
                };
                let open = field.open.get_untracked();
                match &event.key {
                    LogicalKey::Named(NamedKey::ArrowDown) if !open && event.modifiers == alt => {
                        field.open();
                        Handled::Stop
                    }
                    LogicalKey::Named(NamedKey::Escape) if open => {
                        field.close();
                        Handled::Stop
                    }
                    LogicalKey::Named(NamedKey::Tab) if open => {
                        field.close();
                        Handled::Continue
                    }
                    _ if open => match field.calendar.key(event) {
                        KeyOutcome::Ignored => Handled::Continue,
                        _ => Handled::Stop,
                    },
                    _ => Handled::Continue,
                }
            });
        }
        let focused = self.focused;
        cx.on_focus_within(id, move |inside| {
            focused.set(inside);
            if !inside {
                field.close();
                field.restore();
            }
        });
        id
    }
}

/// The calendar popup under the field (WIDGETS-DATE-08).
struct DatePopup {
    calendar: Calendar,
}

impl View for DatePopup {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = ThemeHandle::of(cx);
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [AnyView::new(Padding::new(EdgeInsets::all(8.0)).child(
                CalendarView {
                    calendar: self.calendar,
                    focusable: false,
                },
            ))],
        );
        cx.set_paint(
            id,
            PopupPaint {
                colors: theme.snapshot().colors,
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<PopupPaint>(|p| p.colors = colors);
        });
        id
    }
}
