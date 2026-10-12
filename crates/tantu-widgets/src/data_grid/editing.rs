//! One scoped editor session, independent of the virtualized cell elements.

use super::*;
use crate::{TextEditingController, TextField, TextSelection};
use tantu_view::reactive::{Scope, Timer, timer};

type Model = (Option<SortState>, Option<u64>, Option<usize>, Rc<[usize]>);
pub(super) type ErrorHandler = Rc<dyn Fn(GridCell, GridEditError)>;

struct Session {
    display: GridCell,
    source: GridCell,
    original: CellValue,
    model: Model,
    controller: Cell<Option<TextEditingController>>,
    field: Cell<Option<ElementId>>,
    closed: Cell<bool>,
}

pub(super) struct Editing {
    pub target: Signal<Option<GridCell>>,
    pub refresh: Signal<u64>,
    pub wake: Signal<u64>,
    pub grid: Cell<Option<ElementId>>,
    pub scope: Cell<Option<Scope>>,
    nav: Rc<Navigation>,
    source: Rc<dyn DataSource>,
    map: Memo<Option<Rc<[usize]>>>,
    revision: Option<Signal<u64>>,
    filter: Option<Signal<Option<RowFilter>>>,
    client_sort: bool,
    error: Option<ErrorHandler>,
    session: RefCell<Option<Rc<Session>>>,
    restore: Cell<Option<ElementId>>,
    click: Cell<Option<(GridCell, Point)>>,
    interval: Cell<Option<Timer>>,
}

impl Editing {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nav: Rc<Navigation>,
        source: Rc<dyn DataSource>,
        map: Memo<Option<Rc<[usize]>>>,
        revision: Option<Signal<u64>>,
        filter: Option<Signal<Option<RowFilter>>>,
        client_sort: bool,
        refresh: Signal<u64>,
        error: Option<ErrorHandler>,
    ) -> Self {
        Self {
            target: signal(None),
            refresh,
            wake: signal(0),
            grid: Cell::new(None),
            scope: Cell::new(None),
            nav,
            source,
            map,
            revision,
            filter,
            client_sort,
            error,
            session: RefCell::new(None),
            restore: Cell::new(None),
            click: Cell::new(None),
            interval: Cell::new(None),
        }
    }

    fn model(&self) -> Model {
        (
            self.nav.sort.get_untracked(),
            self.revision.map(|r| r.get_untracked()),
            self.filter
                .and_then(|f| f.get_untracked())
                .map(|f| Rc::as_ptr(&f) as *const () as usize),
            self.nav
                .geometry
                .layout
                .with_untracked(|l| Rc::clone(&l.order)),
        )
    }

    fn visible(&self, cell: GridCell) -> bool {
        let g = &self.nav.geometry;
        if cell.row >= g.row_count.get_untracked()
            || cell.column >= g.column_count
            || g.row_height <= 0.0
        {
            return false;
        }
        let top = cell.row as f32 * g.row_height - self.nav.vertical.offset_untracked();
        let height = self.nav.vertical.viewport_extent_untracked();
        if top >= height || top + g.row_height <= 0.0 || height <= 0.0 {
            return false;
        }
        g.layout.with_untracked(|l| {
            let frozen = l.extents.start(g.frozen_columns);
            let pinned = cell.column < g.frozen_columns;
            let left = l.extents.start(cell.column)
                - if pinned {
                    0.0
                } else {
                    g.offset_untracked(self.nav.horizontal)
                };
            let right = left + l.extents.extent(cell.column);
            let clip_left = if pinned { 0.0 } else { frozen };
            let viewport = self.nav.horizontal.viewport_extent_untracked();
            let clip_right = if pinned {
                frozen.min(viewport)
            } else {
                viewport
            };
            right.min(clip_right) > left.max(clip_left)
        })
    }

    fn valid(&self, session: &Session) -> bool {
        if session.model != self.model() || !self.visible(session.display) {
            return false;
        }
        self.nav.mode == GridSelectionMode::None
            || self
                .nav
                .controller
                .state
                .get_untracked()
                .is_some_and(|s| s.active == session.display)
    }

    pub fn check(&self) {
        let session = self.session.borrow().clone();
        if session.as_ref().is_some_and(|s| !self.valid(s)) {
            self.close(true);
        }
    }

    pub fn reset_click(&self) {
        self.click.set(None);
        if let Some(timer) = self.interval.take() {
            timer.cancel();
        }
    }

    pub fn begin(&self, cell: GridCell) -> bool {
        if self.target.get_untracked().is_some() {
            return false;
        }
        let Some(column) = self
            .nav
            .geometry
            .layout
            .with_untracked(|l| l.order.get(cell.column).copied())
        else {
            return false;
        };
        if !self.nav.columns[column].editable {
            return false;
        }
        self.nav.reveal(cell);
        if !self.visible(cell) {
            return false;
        }
        let row = self.map.with_untracked(|m| {
            m.as_ref()
                .and_then(|m| m.get(cell.row).copied())
                .unwrap_or(cell.row)
        });
        let session = Rc::new(Session {
            display: cell,
            source: GridCell { row, column },
            original: self.source.cell(row, column),
            model: self.model(),
            controller: Cell::new(None),
            field: Cell::new(None),
            closed: Cell::new(false),
        });
        if self.nav.mode != GridSelectionMode::None {
            self.nav.select(cell, false);
        }
        self.reset_click();
        self.restore.set(None);
        *self.session.borrow_mut() = Some(session);
        self.target.set(Some(cell));
        true
    }

    pub fn press(&self, cell: GridCell, position: Point) -> bool {
        let editable = self.nav.geometry.layout.with_untracked(|l| {
            l.order
                .get(cell.column)
                .is_some_and(|&column| self.nav.columns[column].editable)
        });
        if !editable {
            self.reset_click();
            return false;
        }
        let second = self.interval.get().is_some_and(Timer::is_pending)
            && self.click.get().is_some_and(|(previous, point)| {
                previous == cell && (position.x - point.x).hypot(position.y - point.y) <= DRAG_SLOP
            });
        self.reset_click();
        if second {
            return self.begin(cell);
        }
        if let Some(scope) = self.scope.get().filter(|s| !s.is_disposed()) {
            self.click.set(Some((cell, position)));
            self.interval.set(Some(scope.run(|| {
                timer(Duration::from_millis(500) + Duration::from_nanos(1), || {})
            })));
        }
        false
    }

    pub fn pointer_capture(&self, kind: PointerKind, position: Point, body: bool, plain: bool) {
        if matches!(kind, PointerKind::Down(_)) && (!body || !plain) {
            self.reset_click();
        }
        if kind == PointerKind::Move
            && self
                .click
                .get()
                .is_some_and(|(_, p)| (position.x - p.x).hypot(position.y - p.y) > DRAG_SLOP)
        {
            self.reset_click();
        }
    }

    pub fn close(&self, restore: bool) {
        let session = self.session.borrow_mut().take();
        let Some(session) = session else {
            return;
        };
        session.closed.set(true);
        self.restore
            .set(if restore { session.field.get() } else { None });
        self.target.set(None);
        if self.restore.get().is_some() {
            self.wake.update(|n| *n = n.wrapping_add(1));
        }
    }

    fn submit(&self, session: &Session, text: String) {
        if session.closed.get() {
            return;
        }
        if !self.valid(session) {
            self.close(true);
            return;
        }
        if session
            .controller
            .get()
            .is_some_and(|c| c.value().composing.is_some())
        {
            return;
        }
        if text == session.original.to_string() {
            self.close(true);
            return;
        }
        let result = convert(&session.original, text).and_then(|value| {
            self.source
                .set_cell(session.source.row, session.source.column, value)
        });
        match result {
            Err(error) => {
                if let Some(handler) = &self.error {
                    handler(session.source, error);
                }
            }
            Ok(()) => {
                self.close(true);
                if self.client_sort || self.filter.is_some_and(|f| f.get_untracked().is_some()) {
                    self.nav.controller.set_selection(None);
                }
                self.refresh.update(|n| *n = n.wrapping_add(1));
            }
        }
    }
}

fn convert(original: &CellValue, text: String) -> Result<CellValue, GridEditError> {
    let invalid = || GridEditError::InvalidValue;
    match original {
        CellValue::Text(_) => Ok(CellValue::Text(text)),
        CellValue::Empty => Ok(if text.is_empty() {
            CellValue::Empty
        } else {
            CellValue::Text(text)
        }),
        CellValue::Int(_) => text
            .trim()
            .parse()
            .map(CellValue::Int)
            .map_err(|_| invalid()),
        CellValue::Float(_) => text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(CellValue::Float)
            .ok_or_else(invalid),
        CellValue::Bool(_) if text.trim().eq_ignore_ascii_case("true") => Ok(CellValue::Bool(true)),
        CellValue::Bool(_) if text.trim().eq_ignore_ascii_case("false") => {
            Ok(CellValue::Bool(false))
        }
        CellValue::Bool(_) => Err(invalid()),
    }
}

pub(super) struct CellEditor {
    pub editing: Rc<Editing>,
    pub cell: GridCell,
}
impl View for CellEditor {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let editing = self.editing;
        let session = editing.session.borrow().clone();
        let Some(session) = session.filter(|s| s.display == self.cell && !s.closed.get()) else {
            return cx.region(|_| {});
        };
        let controller = TextEditingController::new(session.original.to_string());
        controller.select(TextSelection::new(0, session.original.to_string().len()));
        session.controller.set(Some(controller));
        let (submit_editing, submit_session) = (Rc::clone(&editing), Rc::clone(&session));
        let title = &editing.nav.columns[session.source.column].title;
        let id = TextField::new(controller)
            .placeholder(format!("{title}, row {}", self.cell.row))
            .on_submit(move |text| submit_editing.submit(&submit_session, text))
            .build(cx);
        session.field.set(Some(id));
        let (blur_editing, blur_session) = (Rc::clone(&editing), Rc::clone(&session));
        cx.on_focus_change(id, move |focused| {
            if !focused && !blur_session.closed.get() {
                let restore = !blur_editing.valid(&blur_session);
                blur_editing.close(restore);
            }
        });
        // Focus after the field and its handlers exist; the Dyn version owns its controller.
        cx.focus(id);
        id
    }
}

pub(super) struct RestoreFocus(pub Rc<Editing>);
impl View for RestoreFocus {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        if let Some(expected) = self.0.restore.take() {
            let current = cx.tree().focused();
            if (current.is_none() || current == Some(expected))
                && let Some(grid) = self.0.grid.get()
            {
                cx.focus(grid);
            }
        }
        cx.region(|_| {})
    }
}
