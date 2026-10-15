//! Tests for `docs/specs/widgets/layout-grid.md`, rules LAYOUT-LGRID-01..06.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{
    BoxConstraints, GridPlacementData, LayoutId, LayoutTree, RenderLayoutGrid, TextDirection,
    TrackSize,
};

const INF: f32 = f32::INFINITY;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// A child: its natural size and optional placement.
type Kid = (f32, f32, Option<GridPlacementData>);

/// Placement at (`column`, `row`) with spans.
fn at(
    column: Option<usize>,
    row: Option<usize>,
    columns: usize,
    rows: usize,
) -> Option<GridPlacementData> {
    Some(GridPlacementData {
        column,
        row,
        column_span: columns,
        row_span: rows,
    })
}

struct Grid {
    tree: LayoutTree,
    root: LayoutId,
    kids: Vec<LayoutId>,
}

fn grid(render: RenderLayoutGrid, kids: &[Kid]) -> Grid {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let ids: Vec<LayoutId> = kids
        .iter()
        .map(|&(w, h, data)| {
            let id = tree.insert(leaf(&log, "kid", w, h));
            if let Some(data) = data {
                tree.set_parent_data(id, Some(data));
            }
            id
        })
        .collect();
    tree.set_children(root, &ids).expect("valid");
    Grid {
        tree,
        root,
        kids: ids,
    }
}

impl Grid {
    fn layout(&mut self, c: BoxConstraints) -> Size {
        self.tree.layout(self.root, c)
    }

    fn offsets(&self) -> Vec<Vec2> {
        self.kids
            .iter()
            .map(|id| self.tree.offset(*id).expect("exists"))
            .collect()
    }

    fn sizes(&self) -> Vec<Size> {
        self.kids
            .iter()
            .map(|id| self.tree.size(*id).expect("laid out"))
            .collect()
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn layout_lgrid_01_track_sizes() {
    assert_eq!(TrackSize::px(10.0), TrackSize::Fixed(10.0));
    assert_eq!(TrackSize::fr(2.0), TrackSize::Flex(2.0));
    assert_eq!(TrackSize::AUTO, TrackSize::Auto);
    // Negative and NaN extents count as 0.
    let mut g = grid(
        RenderLayoutGrid::new(
            vec![
                TrackSize::px(-5.0),
                TrackSize::px(f32::NAN),
                TrackSize::px(10.0),
            ],
            vec![],
        ),
        &[(5.0, 5.0, None), (5.0, 5.0, None), (5.0, 5.0, None)],
    );
    g.layout(BoxConstraints::loose(s(100.0, 100.0)));
    let widths: Vec<f32> = g.sizes().iter().map(|s| s.width).collect();
    assert_eq!(widths, [0.0, 0.0, 10.0]);
    // No columns: one flexible column.
    let mut g = grid(RenderLayoutGrid::new(vec![], vec![]), &[(5.0, 5.0, None)]);
    g.layout(BoxConstraints::loose(s(100.0, 100.0)));
    assert_eq!(g.sizes()[0], s(100.0, 5.0));
}

#[test]
fn layout_lgrid_02_placement() {
    let tens = vec![TrackSize::px(10.0); 3];
    let kids: [Kid; 7] = [
        (1.0, 10.0, at(Some(1), Some(0), 1, 1)), // A: explicit
        (1.0, 10.0, None),                       // B: auto
        (1.0, 10.0, None),                       // C: auto
        (1.0, 10.0, at(None, None, 2, 1)),       // D: auto, two columns
        (1.0, 10.0, at(Some(2), None, 1, 1)),    // E: column only
        (1.0, 10.0, at(Some(5), Some(3), 1, 1)), // F: too far right
        (1.0, 10.0, at(None, None, 5, 0)),       // G: span clamped, row span 0 is 1
    ];
    let mut g = grid(RenderLayoutGrid::new(tens, vec![]), &kids);
    let size = g.layout(BoxConstraints::loose(s(100.0, 100.0)));
    assert_eq!(
        g.offsets(),
        [
            v(10.0, 0.0),
            v(0.0, 0.0),
            v(20.0, 0.0),
            v(0.0, 10.0),
            v(20.0, 10.0),
            v(20.0, 30.0),
            v(0.0, 20.0),
        ]
    );
    let widths: Vec<f32> = g.sizes().iter().map(|s| s.width).collect();
    assert_eq!(widths, [10.0, 10.0, 10.0, 20.0, 10.0, 10.0, 30.0]);
    assert_eq!(size, s(30.0, 40.0), "four implicit auto rows");
}

#[test]
fn layout_lgrid_03_columns() {
    let columns = vec![
        TrackSize::AUTO,
        TrackSize::fr(1.0),
        TrackSize::fr(2.0),
        TrackSize::px(20.0),
    ];
    let kids: [Kid; 6] = [
        (30.0, 10.0, at(Some(0), Some(0), 1, 1)),
        (5.0, 10.0, at(Some(1), Some(0), 1, 1)),
        (5.0, 10.0, at(Some(2), Some(0), 1, 1)),
        (5.0, 10.0, at(Some(3), Some(0), 1, 1)),
        (50.0, 10.0, at(Some(0), Some(1), 1, 1)),
        // Spanning items don't grow auto columns.
        (500.0, 10.0, at(Some(0), Some(2), 2, 1)),
    ];
    let mut render = RenderLayoutGrid::new(columns.clone(), vec![]);
    render.column_gap = 10.0;
    let mut g = grid(render.clone(), &kids);
    g.layout(BoxConstraints::loose(s(200.0, 200.0)));
    let widths: Vec<f32> = g.sizes().iter().map(|s| s.width).collect();
    // Free width: 200 - 50 - 20 - 3 gaps = 100, shared 1 : 2.
    let expected = [
        50.0,
        100.0 / 3.0,
        200.0 / 3.0,
        20.0,
        50.0,
        50.0 + 10.0 + 100.0 / 3.0,
    ];
    for (w, e) in widths.iter().zip(expected) {
        assert!(close(*w, e), "{widths:?}");
    }
    let xs: Vec<f32> = g.offsets().iter().map(|o| o.x).collect();
    assert!(close(xs[2], 50.0 + 10.0 + 100.0 / 3.0 + 10.0), "{xs:?}");
    // Unbounded: flexible columns size to their items.
    let mut g = grid(render, &kids);
    let size = g.layout(BoxConstraints::loose(s(INF, 200.0)));
    let widths: Vec<f32> = g.sizes().iter().take(4).map(|s| s.width).collect();
    assert_eq!(widths, [50.0, 5.0, 5.0, 20.0]);
    assert_eq!(size.width, 50.0 + 5.0 + 5.0 + 20.0 + 30.0);
}

#[test]
fn layout_lgrid_04_rows() {
    let rows = vec![TrackSize::px(30.0), TrackSize::AUTO, TrackSize::fr(1.0)];
    let kids: [Kid; 4] = [
        (5.0, 10.0, at(Some(0), Some(0), 1, 1)),
        (5.0, 25.0, at(Some(0), Some(1), 1, 1)),
        (5.0, 10.0, at(Some(0), Some(2), 1, 1)),
        // Spanning the auto row: laid out with its own height.
        (5.0, 7.0, at(Some(1), Some(0), 1, 2)),
    ];
    let render = RenderLayoutGrid::new(vec![TrackSize::px(10.0), TrackSize::px(10.0)], rows);
    let mut g = grid(render.clone(), &kids);
    let size = g.layout(BoxConstraints::loose(s(100.0, 200.0)));
    let heights: Vec<f32> = g.sizes().iter().map(|s| s.height).collect();
    assert_eq!(heights, [30.0, 25.0, 145.0, 7.0]);
    let ys: Vec<f32> = g.offsets().iter().map(|o| o.y).collect();
    assert_eq!(ys, [0.0, 30.0, 55.0, 0.0]);
    assert_eq!(size, s(20.0, 200.0));
    // Unbounded: the flexible row sizes to its item.
    let mut g = grid(render, &kids);
    let size = g.layout(BoxConstraints::loose(s(100.0, INF)));
    assert_eq!(g.sizes()[2].height, 10.0);
    assert_eq!(size.height, 30.0 + 25.0 + 10.0);
}

#[test]
fn layout_lgrid_05_size_and_right_to_left() {
    let mut render = RenderLayoutGrid::new(vec![TrackSize::px(10.0), TrackSize::px(20.0)], vec![]);
    render.column_gap = 5.0;
    render.row_gap = 3.0;
    let kids: [Kid; 3] = [(1.0, 10.0, None), (1.0, 10.0, None), (1.0, 4.0, None)];
    let mut g = grid(render.clone(), &kids);
    assert_eq!(
        g.layout(BoxConstraints::loose(s(100.0, 100.0))),
        s(35.0, 17.0)
    );
    assert_eq!(g.offsets(), [v(0.0, 0.0), v(15.0, 0.0), v(0.0, 13.0)]);
    // Constrained: a tight 100 × 50 box.
    assert_eq!(
        g.layout(BoxConstraints::tight(s(100.0, 50.0))),
        s(100.0, 50.0)
    );
    render.text_direction = TextDirection::Rtl;
    let mut g = grid(render, &kids);
    g.layout(BoxConstraints::loose(s(100.0, 100.0)));
    assert_eq!(g.offsets(), [v(25.0, 0.0), v(0.0, 0.0), v(25.0, 13.0)]);
}

#[test]
fn layout_lgrid_06_intrinsics() {
    let mut render = RenderLayoutGrid::new(
        vec![TrackSize::px(20.0), TrackSize::AUTO, TrackSize::fr(1.0)],
        vec![],
    );
    render.column_gap = 4.0;
    render.row_gap = 2.0;
    let kids: [Kid; 4] = [
        (1.0, 10.0, at(Some(0), Some(0), 1, 1)),
        (30.0, 25.0, at(Some(1), Some(0), 1, 1)),
        (50.0, 10.0, at(Some(2), Some(0), 1, 1)),
        (5.0, 5.0, at(Some(0), Some(1), 1, 1)),
    ];
    let mut g = grid(render, &kids);
    assert_eq!(
        g.tree.max_intrinsic_width(g.root, INF),
        20.0 + 30.0 + 50.0 + 8.0
    );
    assert_eq!(
        g.tree.min_intrinsic_width(g.root, INF),
        20.0 + 30.0 + 50.0 + 8.0
    );
    assert_eq!(g.tree.max_intrinsic_height(g.root, 200.0), 25.0 + 5.0 + 2.0);
    assert_eq!(g.tree.min_intrinsic_height(g.root, 200.0), 25.0 + 5.0 + 2.0);
}
