//! Selection models for the virtualized data grid.

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
    pub fn single(_cell: GridCell) -> Self {
        todo!("WIDGETS-GRID-16")
    }
    /// Whether `cell` is selected in `mode`, without enumerating cells.
    pub fn contains(self, _cell: GridCell, _mode: GridSelectionMode) -> bool {
        todo!("WIDGETS-GRID-16")
    }
}

/// Shares selection and display-to-source mapping with an application.
/// Create within a reactive runtime and use one controller per mounted grid.
#[derive(Clone)]
pub struct GridSelectionController;

impl GridSelectionController {
    /// Starts without a selection or attached grid.
    pub fn new() -> Self {
        todo!("WIDGETS-GRID-16")
    }
    /// A tracked read of the current display-coordinate selection.
    pub fn selection(&self) -> Option<GridSelection> {
        todo!("WIDGETS-GRID-16")
    }
    /// Changes selection; the grid validates endpoints on its next frame.
    pub fn set_selection(&self, _selection: Option<GridSelection>) {
        todo!("WIDGETS-GRID-16")
    }
    /// Maps a display cell using the latest pumped grid model; `None` if detached or invalid.
    pub fn source_cell(&self, _display: GridCell) -> Option<GridCell> {
        todo!("WIDGETS-GRID-21")
    }
}

impl Default for GridSelectionController {
    fn default() -> Self {
        Self::new()
    }
}
