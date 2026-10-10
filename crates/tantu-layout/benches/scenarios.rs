//! Benchmark scenarios, spec `docs/specs/layout/benchmarks.md`.
//!
//! Shared by the bench target (`benches/layout.rs`) and the test target
//! (`tests/bench_scenarios.rs`) through `#[path]`, so every scenario that is timed is also checked.
//! `new()` builds the tree (not timed); `step()` is the timed part.

#![allow(dead_code)]

use std::cell::Cell;
use std::rc::Rc;

use tantu_core::{EdgeInsets, Size};
use tantu_layout::{
    BoxConstraints, CrossAxisAlignment, FlexParentData, LayoutChildren, LayoutId, LayoutTree,
    MeasureCache, RenderBox, RenderFlex, RenderPadding, RenderParagraph, RenderPositionedBox,
    TextMeasure, TextMetrics, TextStyleKey, TextWidthBasis,
};

/// A leaf of a fixed size that counts its layouts in a shared counter.
struct CountingLeaf {
    size: Size,
    layouts: Rc<Cell<u64>>,
}

impl PartialEq for CountingLeaf {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

impl RenderBox for CountingLeaf {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        self.layouts.set(self.layouts.get() + 1);
        c.constrain(self.size)
    }
}

const ROWS: usize = 100;
const CELLS: usize = 32;
const LEAF: Size = Size::new(4.0, 8.0);

/// One dashboard cell: padding → centered box → counting leaf.
struct Cell3 {
    padding: LayoutId,
    centered: LayoutId,
    leaf: LayoutId,
}

/// The 10k-node dashboard shared by the first three scenarios: a column (children stretched to
/// its width) of 100 padded rows of 32 expanded cells.
struct Dashboard {
    tree: LayoutTree,
    root: LayoutId,
    rows: Vec<(LayoutId, Vec<Cell3>)>,
    layouts: Rc<Cell<u64>>,
}

impl Dashboard {
    fn new() -> Self {
        let layouts = Rc::new(Cell::new(0));
        let mut tree = LayoutTree::new();
        let mut column = RenderFlex::column();
        column.cross_axis_alignment = CrossAxisAlignment::Stretch;
        let root = tree.insert(column);
        let mut row_paddings = Vec::with_capacity(ROWS);
        let mut rows = Vec::with_capacity(ROWS);
        for _ in 0..ROWS {
            let row_padding = tree.insert(RenderPadding::new(EdgeInsets::all(2.0)));
            let mut flex = RenderFlex::row();
            flex.spacing = 2.0;
            let row = tree.insert(flex);
            let cells: Vec<Cell3> = (0..CELLS)
                .map(|_| {
                    let padding = tree.insert(RenderPadding::new(EdgeInsets::all(1.0)));
                    let centered = tree.insert(RenderPositionedBox::center());
                    let leaf = tree.insert(CountingLeaf {
                        size: LEAF,
                        layouts: layouts.clone(),
                    });
                    tree.set_children(padding, &[centered])
                        .expect("fresh nodes");
                    tree.set_children(centered, &[leaf]).expect("fresh nodes");
                    tree.set_parent_data(padding, Some(FlexParentData::expanded(1)));
                    Cell3 {
                        padding,
                        centered,
                        leaf,
                    }
                })
                .collect();
            let cell_ids: Vec<LayoutId> = cells.iter().map(|c| c.padding).collect();
            tree.set_children(row, &cell_ids).expect("fresh nodes");
            tree.set_children(row_padding, &[row]).expect("fresh nodes");
            row_paddings.push(row_padding);
            rows.push((row, cells));
        }
        tree.set_children(root, &row_paddings).expect("fresh nodes");
        let mut dashboard = Dashboard {
            tree,
            root,
            rows,
            layouts,
        };
        dashboard.layout(Dashboard::constraints(0));
        dashboard
    }

    /// The root constraints for step parity `k`: the width alternates.
    fn constraints(k: u64) -> BoxConstraints {
        BoxConstraints::tight(Size::new(1600.0 + (k % 2) as f32, 2000.0))
    }

    fn layout(&mut self, c: BoxConstraints) -> Size {
        self.tree.layout(self.root, c)
    }

    /// Runs `f` and returns its result with the number of leaf layouts it caused.
    fn counting<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> (T, u64) {
        let before = self.layouts.get();
        let out = f(self);
        (out, self.layouts.get() - before)
    }
}

/// LAYOUT-BENCH-01: a 10k-node dashboard; each step lays out every node again.
pub struct FullLayout {
    dashboard: Dashboard,
    steps: u64,
    last_layouts: u64,
}

impl FullLayout {
    /// Builds the tree and lays it out once.
    pub fn new() -> Self {
        FullLayout {
            dashboard: Dashboard::new(),
            steps: 0,
            last_layouts: 0,
        }
    }

    /// Lays out with the other root width.
    pub fn step(&mut self) -> Size {
        self.steps += 1;
        let c = Dashboard::constraints(self.steps);
        let (size, layouts) = self.dashboard.counting(|d| d.layout(c));
        self.last_layouts = layouts;
        size
    }

    /// Number of nodes in the tree.
    pub fn node_count(&self) -> usize {
        self.dashboard.tree.len()
    }

    /// Leaves laid out during the last step.
    pub fn leaf_layouts(&self) -> u64 {
        self.last_layouts
    }

    /// The root's size for the last step's constraints.
    pub fn expected_root_size(&self) -> Size {
        Dashboard::constraints(self.steps).biggest()
    }

    /// True if every leaf lies inside its row (offsets and sizes within the row's bounds).
    pub fn leaves_inside_rows(&self) -> bool {
        let tree = &self.dashboard.tree;
        self.dashboard.rows.iter().all(|(row, cells)| {
            let Some(row_size) = tree.size(*row) else {
                return false;
            };
            cells.iter().all(|cell| {
                let (Some(cell_at), Some(centered_at), Some(leaf_at), Some(leaf)) = (
                    tree.offset(cell.padding),
                    tree.offset(cell.centered),
                    tree.offset(cell.leaf),
                    tree.size(cell.leaf),
                ) else {
                    return false;
                };
                let x = cell_at.x + centered_at.x + leaf_at.x;
                x >= 0.0 && x + leaf.width <= row_size.width + 1e-3
            })
        })
    }
}

/// LAYOUT-BENCH-02: the dashboard; each step changes one leaf and lays out.
pub struct RelayoutOneLeaf {
    dashboard: Dashboard,
    leaf: LayoutId,
    given: Size,
    last_layouts: u64,
}

impl RelayoutOneLeaf {
    /// Builds and lays out the tree.
    pub fn new() -> Self {
        let dashboard = Dashboard::new();
        let leaf = dashboard.rows[ROWS / 2].1[CELLS / 3].leaf;
        RelayoutOneLeaf {
            dashboard,
            leaf,
            given: LEAF,
            last_layouts: 0,
        }
    }

    /// Changes one leaf's size and lays out.
    pub fn step(&mut self) -> Size {
        self.given = if self.given == LEAF {
            Size::new(LEAF.width + 1.0, LEAF.height)
        } else {
            LEAF
        };
        let render = CountingLeaf {
            size: self.given,
            layouts: self.dashboard.layouts.clone(),
        };
        let leaf = self.leaf;
        let ((), _) = self.dashboard.counting(|d| {
            d.tree.set(leaf, render);
        });
        let (size, layouts) = self
            .dashboard
            .counting(|d| d.layout(Dashboard::constraints(0)));
        self.last_layouts = layouts;
        size
    }

    /// Leaves laid out during the last step.
    pub fn leaf_layouts(&self) -> u64 {
        self.last_layouts
    }

    /// The changed leaf's laid-out size, and the size it was given in the last step.
    pub fn changed_leaf(&self) -> (Size, Size) {
        (
            self.dashboard.tree.size(self.leaf).unwrap_or(Size::ZERO),
            self.given,
        )
    }
}

/// LAYOUT-BENCH-03: the dashboard; each step lays out with nothing changed.
pub struct NoOpLayout {
    dashboard: Dashboard,
    last_layouts: u64,
}

impl NoOpLayout {
    /// Builds and lays out the tree.
    pub fn new() -> Self {
        NoOpLayout {
            dashboard: Dashboard::new(),
            last_layouts: 0,
        }
    }

    /// Lays out again.
    pub fn step(&mut self) -> Size {
        let (size, layouts) = self
            .dashboard
            .counting(|d| d.layout(Dashboard::constraints(0)));
        self.last_layouts = layouts;
        size
    }

    /// Leaves laid out during the last step.
    pub fn leaf_layouts(&self) -> u64 {
        self.last_layouts
    }
}

const WIDE: usize = 10_000;

/// LAYOUT-BENCH-04: one row with 10 000 children, every other one expanded.
pub struct WideFlex {
    tree: LayoutTree,
    root: LayoutId,
    kids: Vec<LayoutId>,
    layouts: Rc<Cell<u64>>,
    steps: u64,
    last_layouts: u64,
}

impl WideFlex {
    /// Builds and lays out the row.
    pub fn new() -> Self {
        let layouts = Rc::new(Cell::new(0));
        let mut tree = LayoutTree::new();
        let root = tree.insert(RenderFlex::row());
        let kids: Vec<LayoutId> = (0..WIDE)
            .map(|i| {
                let id = tree.insert(CountingLeaf {
                    size: Size::new(1.0, 1.0),
                    layouts: layouts.clone(),
                });
                if i % 2 == 1 {
                    tree.set_parent_data(id, Some(FlexParentData::expanded(1)));
                }
                id
            })
            .collect();
        tree.set_children(root, &kids).expect("fresh nodes");
        tree.layout(root, WideFlex::constraints(0));
        WideFlex {
            tree,
            root,
            kids,
            layouts,
            steps: 0,
            last_layouts: 0,
        }
    }

    /// Width and height alternate, so every child's constraints change.
    fn constraints(k: u64) -> BoxConstraints {
        let odd = (k % 2) as f32;
        BoxConstraints::tight(Size::new(20_000.0 + odd, 100.0 + odd))
    }

    /// Lays out with the other width.
    pub fn step(&mut self) -> Size {
        self.steps += 1;
        let before = self.layouts.get();
        let size = self
            .tree
            .layout(self.root, WideFlex::constraints(self.steps));
        self.last_layouts = self.layouts.get() - before;
        size
    }

    /// Children laid out during the last step.
    pub fn child_layouts(&self) -> u64 {
        self.last_layouts
    }

    /// The expanded children's total width, and the free space they share.
    pub fn expanded_width_and_free_space(&self) -> (f32, f32) {
        let expanded: f32 = self
            .kids
            .iter()
            .skip(1)
            .step_by(2)
            .filter_map(|id| self.tree.size(*id))
            .map(|s| s.width)
            .sum();
        let width = WideFlex::constraints(self.steps).max_width;
        (expanded, width - (WIDE / 2) as f32)
    }

    /// The row's overflow after the last step.
    pub fn overflow(&self) -> f32 {
        self.tree
            .get::<RenderFlex>(self.root)
            .map_or(f32::NAN, RenderFlex::overflow)
    }
}

/// A fake text system: 6 per character, 14 per line, words break at spaces. Counts its calls.
#[derive(Default)]
struct FakeText {
    calls: u64,
}

impl TextMeasure for FakeText {
    fn measure(
        &mut self,
        text: &str,
        _: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        self.calls += 1;
        let (mut lines, mut line, mut widest) = (0u32, 0.0f32, 0.0f32);
        for word in text.split(' ') {
            let w = 6.0 * word.len() as f32;
            if line > 0.0 && line + 6.0 + w > max_width {
                widest = widest.max(line);
                lines += 1;
                line = w;
            } else {
                line += if line > 0.0 { 6.0 + w } else { w };
            }
        }
        widest = widest.max(line);
        lines += 1;
        if let Some(max) = max_lines {
            lines = lines.min(max);
        }
        TextMetrics {
            size: Size::new(widest, 14.0 * lines as f32),
            line_count: lines,
            first_baseline: 11.0,
            last_baseline: 14.0 * (lines - 1) as f32 + 11.0,
        }
    }

    fn min_intrinsic_width(&mut self, text: &str, _: TextStyleKey) -> f32 {
        self.calls += 1;
        text.split(' ')
            .map(|w| 6.0 * w.len() as f32)
            .fold(0.0, f32::max)
    }
}

/// Counts the measure calls that reach the cache (one per paragraph layout with the
/// `LongestLine` width basis).
struct Counting {
    cache: MeasureCache<FakeText>,
    calls: u64,
}

impl TextMeasure for Counting {
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        self.calls += 1;
        self.cache.measure(text, style, max_width, max_lines)
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        self.cache.min_intrinsic_width(text, style)
    }
}

const GRID: usize = 100;

/// LAYOUT-BENCH-05: 100 × 100 paragraphs (50 distinct strings) through a `MeasureCache`.
pub struct TextGrid {
    tree: LayoutTree,
    root: LayoutId,
    text: Counting,
    steps: u64,
    last_layouts: u64,
    last_inner: u64,
}

impl TextGrid {
    /// Builds and lays out the grid.
    pub fn new() -> Self {
        let mut tree = LayoutTree::new();
        let mut column = RenderFlex::column();
        column.cross_axis_alignment = CrossAxisAlignment::Stretch;
        let root = tree.insert(column);
        let strings: Vec<String> = (0..50)
            .map(|k| format!("cell {k} value {}", k * 7))
            .collect();
        let rows: Vec<LayoutId> = (0..GRID)
            .map(|r| {
                let row = tree.insert(RenderFlex::row());
                let cells: Vec<LayoutId> = (0..GRID)
                    .map(|c| {
                        let mut p = RenderParagraph::new(
                            strings[(r * GRID + c) % strings.len()].as_str(),
                            TextStyleKey(1),
                        );
                        p.text_width_basis = TextWidthBasis::LongestLine;
                        let id = tree.insert(p);
                        tree.set_parent_data(id, Some(FlexParentData::expanded(1)));
                        id
                    })
                    .collect();
                tree.set_children(row, &cells).expect("fresh nodes");
                row
            })
            .collect();
        tree.set_children(root, &rows).expect("fresh nodes");
        let mut grid = TextGrid {
            tree,
            root,
            text: Counting {
                cache: MeasureCache::new(FakeText::default(), 4096),
                calls: 0,
            },
            steps: 0,
            last_layouts: 0,
            last_inner: 0,
        };
        grid.layout(TextGrid::constraints(0));
        grid
    }

    fn constraints(k: u64) -> BoxConstraints {
        BoxConstraints::tight(Size::new(3000.0 + (k % 2) as f32, 5000.0))
    }

    fn layout(&mut self, c: BoxConstraints) -> Size {
        self.tree.with_text(&mut self.text).layout(self.root, c)
    }

    /// Lays out with the other width.
    pub fn step(&mut self) -> Size {
        self.steps += 1;
        let (calls, inner) = (self.text.calls, self.text.cache.inner().calls);
        let size = self.layout(TextGrid::constraints(self.steps));
        self.last_layouts = self.text.calls - calls;
        self.last_inner = self.text.cache.inner().calls - inner;
        size
    }

    /// Paragraphs laid out during the last step.
    pub fn paragraph_layouts(&self) -> u64 {
        self.last_layouts
    }

    /// Calls to the inner measurer during the last step.
    pub fn inner_calls(&self) -> u64 {
        self.last_inner
    }
}
