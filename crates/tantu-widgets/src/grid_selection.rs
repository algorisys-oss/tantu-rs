//! Selection models for the virtualized data grid.

use std::rc::Rc;
use tantu_view::reactive::{Signal, signal};

/// How a grid selects body cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridSelectionMode {
    /// Selection is disabled (the default).
    #[default]
    None,
    /// Select a row or contiguous Shift-extended row interval.
    Row,
    /// Select exactly one cell.
    Cell,
    /// Select an inclusive rectangle.
    Range,
}

/// A zero-based display cell, or a source cell returned by the controller's mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridCell {
    /// The row index.
    pub row: usize,
    /// The column index.
    pub column: usize,
}

/// Display-coordinate endpoints of a selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSelection {
    /// The fixed endpoint during Shift extension.
    pub anchor: GridCell,
    /// The moving endpoint and keyboard cursor.
    pub active: GridCell,
}

impl GridSelection {
    /// A selection whose two endpoints are `cell`.
    pub fn single(cell: GridCell) -> Self {
        Self {
            anchor: cell,
            active: cell,
        }
    }
    /// Whether `cell` is selected in `mode`, without enumerating cells.
    pub fn contains(self, cell: GridCell, mode: GridSelectionMode) -> bool {
        let between = |value, a, b| value >= usize::min(a, b) && value <= usize::max(a, b);
        match mode {
            GridSelectionMode::None => false,
            GridSelectionMode::Cell => cell == self.active,
            GridSelectionMode::Row => between(cell.row, self.anchor.row, self.active.row),
            GridSelectionMode::Range => {
                between(cell.row, self.anchor.row, self.active.row)
                    && between(cell.column, self.anchor.column, self.active.column)
            }
        }
    }
}

/// Shares selection and display-to-source mapping with an application.
/// Create within a reactive runtime and use one controller per mounted grid.
#[derive(Clone)]
pub struct GridSelectionController {
    pub(crate) state: Signal<Option<GridSelection>>,
    pub(crate) mapping: Signal<Option<Mapping>>,
}

/// Shared presentation mapping. A unique owner protects a newer attachment on cleanup.
#[derive(Clone)]
pub(crate) struct Mapping {
    pub(crate) owner: Rc<()>,
    pub(crate) rows: usize,
    pub(crate) map: Option<Rc<[usize]>>,
    pub(crate) order: Rc<[usize]>,
}

impl GridSelectionController {
    /// Starts without a selection or attached grid.
    pub fn new() -> Self {
        Self {
            state: signal(None),
            mapping: signal(None),
        }
    }
    /// A tracked read of the current display-coordinate selection.
    pub fn selection(&self) -> Option<GridSelection> {
        self.state.get()
    }
    /// Changes selection; the grid validates endpoints on its next frame.
    pub fn set_selection(&self, selection: Option<GridSelection>) {
        if self.state.get_untracked() != selection {
            self.state.set(selection);
        }
    }
    /// Maps a display cell using the latest pumped grid model; `None` if detached or invalid.
    pub fn source_cell(&self, display: GridCell) -> Option<GridCell> {
        self.mapping.with(|mapping| {
            let m = mapping.as_ref()?;
            if display.row >= m.rows {
                return None;
            }
            let column = *m.order.get(display.column)?;
            let row = match &m.map {
                Some(rows) => *rows.get(display.row)?,
                None => display.row,
            };
            Some(GridCell { row, column })
        })
    }
}

impl Default for GridSelectionController {
    fn default() -> Self {
        Self::new()
    }
}
