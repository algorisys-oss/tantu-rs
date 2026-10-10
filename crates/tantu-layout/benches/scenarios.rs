//! Benchmark scenarios, spec `docs/specs/layout/benchmarks.md`.
//!
//! Shared by the bench target (`benches/layout.rs`) and the test target
//! (`tests/bench_scenarios.rs`) through `#[path]`, so every scenario that is timed is also checked.
//! `new()` builds the tree (not timed); `step()` is the timed part.

#![allow(dead_code)]

use tantu_core::Size;

/// LAYOUT-BENCH-01: a 10k-node dashboard; each step lays out every node again.
pub struct FullLayout {}

impl FullLayout {
    /// Builds the tree and lays it out once.
    pub fn new() -> Self {
        todo!()
    }

    /// Lays out with the other root width.
    pub fn step(&mut self) -> Size {
        todo!()
    }

    /// Number of nodes in the tree.
    pub fn node_count(&self) -> usize {
        todo!()
    }

    /// Leaves laid out during the last step.
    pub fn leaf_layouts(&self) -> u64 {
        todo!()
    }

    /// The root's size for the last step's constraints.
    pub fn expected_root_size(&self) -> Size {
        todo!()
    }

    /// True if every leaf lies inside its row (offsets and sizes within the row's bounds).
    pub fn leaves_inside_rows(&self) -> bool {
        todo!()
    }
}

/// LAYOUT-BENCH-02: the dashboard; each step changes one leaf and lays out.
pub struct RelayoutOneLeaf {}

impl RelayoutOneLeaf {
    /// Builds and lays out the tree.
    pub fn new() -> Self {
        todo!()
    }

    /// Changes one leaf's size and lays out.
    pub fn step(&mut self) -> Size {
        todo!()
    }

    /// Leaves laid out during the last step.
    pub fn leaf_layouts(&self) -> u64 {
        todo!()
    }

    /// The changed leaf's laid-out size, and the size it was given in the last step.
    pub fn changed_leaf(&self) -> (Size, Size) {
        todo!()
    }
}

/// LAYOUT-BENCH-03: the dashboard; each step lays out with nothing changed.
pub struct NoOpLayout {}

impl NoOpLayout {
    /// Builds and lays out the tree.
    pub fn new() -> Self {
        todo!()
    }

    /// Lays out again.
    pub fn step(&mut self) -> Size {
        todo!()
    }

    /// Leaves laid out during the last step.
    pub fn leaf_layouts(&self) -> u64 {
        todo!()
    }
}

/// LAYOUT-BENCH-04: one row with 10 000 children, every other one expanded.
pub struct WideFlex {}

impl WideFlex {
    /// Builds and lays out the row.
    pub fn new() -> Self {
        todo!()
    }

    /// Lays out with the other width.
    pub fn step(&mut self) -> Size {
        todo!()
    }

    /// Children laid out during the last step.
    pub fn child_layouts(&self) -> u64 {
        todo!()
    }

    /// The expanded children's total width, and the free space they share.
    pub fn expanded_width_and_free_space(&self) -> (f32, f32) {
        todo!()
    }

    /// The row's overflow after the last step.
    pub fn overflow(&self) -> f32 {
        todo!()
    }
}

/// LAYOUT-BENCH-05: 100 × 100 paragraphs (50 distinct strings) through a `MeasureCache`.
pub struct TextGrid {}

impl TextGrid {
    /// Builds and lays out the grid.
    pub fn new() -> Self {
        todo!()
    }

    /// Lays out with the other width.
    pub fn step(&mut self) -> Size {
        todo!()
    }

    /// Paragraphs laid out during the last step.
    pub fn paragraph_layouts(&self) -> u64 {
        todo!()
    }

    /// Calls to the inner measurer during the last step.
    pub fn inner_calls(&self) -> u64 {
        todo!()
    }
}
