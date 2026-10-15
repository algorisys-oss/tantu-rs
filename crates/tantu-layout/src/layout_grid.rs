//! [`RenderLayoutGrid`]: children on column and row tracks. Spec:
//! `docs/specs/widgets/layout-grid.md`.

use tantu_core::{Size, Vec2};

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox, TextDirection};

/// How a column or row is sized (LAYOUT-LGRID-01).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackSize {
    /// A fixed extent (negative or NaN counts as 0).
    Fixed(f32),
    /// A share of the free space, in proportion to the factor (negative or NaN counts as 0).
    Flex(f32),
    /// The largest natural extent of the items in it.
    Auto,
}

impl TrackSize {
    /// A fixed extent.
    pub const fn px(extent: f32) -> Self {
        TrackSize::Fixed(extent)
    }

    /// A flexible share.
    pub const fn fr(factor: f32) -> Self {
        TrackSize::Flex(factor)
    }

    /// Sized to its items.
    pub const AUTO: TrackSize = TrackSize::Auto;
}

/// Where an item sits (parent data of a [`RenderLayoutGrid`] child, LAYOUT-LGRID-02).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridPlacementData {
    /// The first column, or `None` for automatic placement.
    pub column: Option<usize>,
    /// The first row, or `None` for automatic placement.
    pub row: Option<usize>,
    /// Columns covered (0 counts as 1).
    pub column_span: usize,
    /// Rows covered (0 counts as 1).
    pub row_span: usize,
}

/// Lays children out on column and row tracks (CSS Grid's track sizing, Flutter's
/// `flutter_layout_grid`; LAYOUT-LGRID-01..06).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderLayoutGrid {
    /// The column tracks.
    pub columns: Vec<TrackSize>,
    /// The row tracks; rows past them are implicit auto rows.
    pub rows: Vec<TrackSize>,
    /// Space between columns (negative or NaN counts as 0).
    pub column_gap: f32,
    /// Space between rows (negative or NaN counts as 0).
    pub row_gap: f32,
    /// `Rtl` runs the columns from the right.
    pub text_direction: TextDirection,
}

impl RenderLayoutGrid {
    /// A grid with these tracks, no gaps, left to right.
    pub fn new(columns: Vec<TrackSize>, rows: Vec<TrackSize>) -> Self {
        RenderLayoutGrid {
            columns,
            rows,
            column_gap: 0.0,
            row_gap: 0.0,
            text_direction: TextDirection::Ltr,
        }
    }
}

/// An extent or factor with negative and NaN values counted as 0 (LAYOUT-LGRID-01).
fn clean(value: f32) -> f32 {
    if value.is_nan() || value < 0.0 {
        0.0
    } else {
        value
    }
}

/// An item's cells: its first column and row and how many it covers.
#[derive(Clone, Copy, Debug)]
struct Item {
    column: usize,
    row: usize,
    columns: usize,
    rows: usize,
}

/// Which cells are taken, growing downward as rows are used.
struct Occupancy {
    columns: usize,
    cells: Vec<bool>,
}

impl Occupancy {
    fn fits(&self, item: Item) -> bool {
        (item.row..item.row + item.rows).all(|row| {
            (item.column..item.column + item.columns).all(|column| {
                !self
                    .cells
                    .get(row * self.columns + column)
                    .copied()
                    .unwrap_or(false)
            })
        })
    }

    fn take(&mut self, item: Item) {
        let needed = (item.row + item.rows) * self.columns;
        if self.cells.len() < needed {
            self.cells.resize(needed, false);
        }
        for row in item.row..item.row + item.rows {
            for column in item.column..item.column + item.columns {
                self.cells[row * self.columns + column] = true;
            }
        }
    }
}

/// Track extents from their sizes, the space available along the axis, the gap and the
/// auto contributions (LAYOUT-LGRID-03, -04).
fn size_tracks(tracks: &[TrackSize], available: f32, gap: f32, contributions: &[f32]) -> Vec<f32> {
    let gaps = gap * tracks.len().saturating_sub(1) as f32;
    let bounded = available.is_finite();
    let mut extents: Vec<f32> = tracks
        .iter()
        .zip(contributions)
        .map(|(track, contribution)| match *track {
            TrackSize::Fixed(extent) => clean(extent),
            TrackSize::Flex(_) if bounded => 0.0,
            TrackSize::Flex(_) | TrackSize::Auto => *contribution,
        })
        .collect();
    if bounded {
        let total: f32 = tracks
            .iter()
            .map(|t| match *t {
                TrackSize::Flex(factor) => clean(factor),
                _ => 0.0,
            })
            .sum();
        let free = (available - extents.iter().sum::<f32>() - gaps).max(0.0);
        if total > 0.0 {
            for (extent, track) in extents.iter_mut().zip(tracks) {
                if let TrackSize::Flex(factor) = *track {
                    *extent = free * clean(factor) / total;
                }
            }
        }
    }
    extents
}

/// The start of each track (gaps included) and the total extent.
fn starts(extents: &[f32], gap: f32) -> (Vec<f32>, f32) {
    let mut starts = Vec::with_capacity(extents.len());
    let mut position = 0.0;
    for (i, extent) in extents.iter().enumerate() {
        if i > 0 {
            position += gap;
        }
        starts.push(position);
        position += extent;
    }
    (starts, position)
}

/// The extent of `count` tracks from `first`, with the gaps between them.
fn span(extents: &[f32], gap: f32, first: usize, count: usize) -> f32 {
    extents[first..first + count].iter().sum::<f32>() + gap * count.saturating_sub(1) as f32
}

impl RenderLayoutGrid {
    /// The column tracks, one flexible column when there are none (LAYOUT-LGRID-01).
    fn column_tracks(&self) -> Vec<TrackSize> {
        if self.columns.is_empty() {
            vec![TrackSize::Flex(1.0)]
        } else {
            self.columns.clone()
        }
    }

    fn column_gap(&self) -> f32 {
        clean(self.column_gap)
    }

    fn row_gap(&self) -> f32 {
        clean(self.row_gap)
    }

    /// Places every item (LAYOUT-LGRID-02); returns the items and the row tracks, implicit
    /// auto rows included.
    fn place(
        &self,
        columns: usize,
        data: &[Option<GridPlacementData>],
    ) -> (Vec<Item>, Vec<TrackSize>) {
        let mut occupancy = Occupancy {
            columns,
            cells: Vec::new(),
        };
        let shape = |d: GridPlacementData| {
            let spanned = d.column_span.clamp(1, columns);
            (spanned, d.row_span.max(1))
        };
        let mut items = vec![
            Item {
                column: 0,
                row: 0,
                columns: 1,
                rows: 1,
            };
            data.len()
        ];
        let mut placed = vec![false; data.len()];
        for (i, d) in data.iter().enumerate() {
            let d = d.unwrap_or_default();
            if let (Some(column), Some(row)) = (d.column, d.row) {
                let (spanned, rows) = shape(d);
                let item = Item {
                    column: column.min(columns - spanned),
                    row,
                    columns: spanned,
                    rows,
                };
                occupancy.take(item);
                items[i] = item;
                placed[i] = true;
            }
        }
        for (i, d) in data.iter().enumerate() {
            if placed[i] {
                continue;
            }
            let d = d.unwrap_or_default();
            let (spanned, rows) = shape(d);
            let last = columns - spanned;
            let fixed_column = d.column.map(|c| c.min(last));
            let mut row = d.row.unwrap_or(0);
            let item = loop {
                let candidates = match fixed_column {
                    Some(column) => column..=column,
                    None => 0..=last,
                };
                let found = candidates
                    .map(|column| Item {
                        column,
                        row,
                        columns: spanned,
                        rows,
                    })
                    .find(|item| occupancy.fits(*item));
                match found {
                    Some(item) => break item,
                    // A fixed row with no room: the first column, overlapping.
                    None if d.row.is_some() => {
                        break Item {
                            column: fixed_column.unwrap_or(0),
                            row,
                            columns: spanned,
                            rows,
                        };
                    }
                    None => row += 1,
                }
            };
            occupancy.take(item);
            items[i] = item;
        }
        let used = items.iter().map(|i| i.row + i.rows).max().unwrap_or(0);
        let mut rows = self.rows.clone();
        if rows.len() < used {
            rows.resize(used, TrackSize::Auto);
        }
        (items, rows)
    }

    /// Column widths for `available` width, measuring single-column items in auto (and
    /// unbounded flexible) columns with `measure` (LAYOUT-LGRID-03).
    fn column_widths(
        &self,
        tracks: &[TrackSize],
        items: &[Item],
        available: f32,
        mut measure: impl FnMut(usize) -> f32,
    ) -> Vec<f32> {
        let mut contributions = vec![0.0f32; tracks.len()];
        for (i, item) in items.iter().enumerate() {
            if item.columns == 1 && Self::content_sized(tracks[item.column], available) {
                contributions[item.column] = contributions[item.column].max(measure(i));
            }
        }
        size_tracks(tracks, available, self.column_gap(), &contributions)
    }

    /// Whether a track is sized by its items for an axis with `available` space.
    fn content_sized(track: TrackSize, available: f32) -> bool {
        match track {
            TrackSize::Auto => true,
            TrackSize::Flex(_) => !available.is_finite(),
            TrackSize::Fixed(_) => false,
        }
    }

    /// The intrinsic width: columns sized to their items' intrinsic widths (LAYOUT-LGRID-06).
    fn intrinsic_width(
        &self,
        children: &mut IntrinsicChildren<'_>,
        measure: fn(&mut IntrinsicChildren<'_>, usize) -> f32,
    ) -> f32 {
        let tracks = self.column_tracks();
        let data: Vec<_> = (0..children.len())
            .map(|i| children.parent_data::<GridPlacementData>(i).copied())
            .collect();
        let (items, _) = self.place(tracks.len(), &data);
        let widths = self.column_widths(&tracks, &items, f32::INFINITY, |i| measure(children, i));
        starts(&widths, self.column_gap()).1
    }

    /// The intrinsic height for `width`: rows sized to their items' intrinsic heights at
    /// their columns' widths (LAYOUT-LGRID-06).
    fn intrinsic_height(
        &self,
        width: f32,
        children: &mut IntrinsicChildren<'_>,
        measure: fn(&mut IntrinsicChildren<'_>, usize, f32) -> f32,
    ) -> f32 {
        let tracks = self.column_tracks();
        let data: Vec<_> = (0..children.len())
            .map(|i| children.parent_data::<GridPlacementData>(i).copied())
            .collect();
        let (items, rows) = self.place(tracks.len(), &data);
        let widths = self.column_widths(&tracks, &items, width, |i| {
            children.max_intrinsic_width(i, f32::INFINITY)
        });
        let gap = self.column_gap();
        let mut contributions = vec![0.0f32; rows.len()];
        for (i, item) in items.iter().enumerate() {
            if item.rows == 1 && Self::content_sized(rows[item.row], f32::INFINITY) {
                let w = span(&widths, gap, item.column, item.columns);
                contributions[item.row] = contributions[item.row].max(measure(children, i, w));
            }
        }
        let heights = size_tracks(&rows, f32::INFINITY, self.row_gap(), &contributions);
        starts(&heights, self.row_gap()).1
    }
}

impl RenderBox for RenderLayoutGrid {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let tracks = self.column_tracks();
        let data: Vec<_> = (0..children.len())
            .map(|i| children.parent_data::<GridPlacementData>(i).copied())
            .collect();
        let (items, rows) = self.place(tracks.len(), &data);
        let (column_gap, row_gap) = (self.column_gap(), self.row_gap());
        let widths = self.column_widths(&tracks, &items, constraints.max_width, |i| {
            children.max_intrinsic_width(i, f32::INFINITY)
        });
        let width_of = |item: &Item| span(&widths, column_gap, item.column, item.columns);
        // Items in content-sized rows first, with their own heights (LAYOUT-LGRID-04).
        let max_height = constraints.max_height;
        let sized_by_rows = |item: &Item| {
            rows[item.row..item.row + item.rows]
                .iter()
                .all(|track| !Self::content_sized(*track, max_height))
        };
        let mut contributions = vec![0.0f32; rows.len()];
        for (i, item) in items.iter().enumerate() {
            if sized_by_rows(item) {
                continue;
            }
            let w = width_of(item);
            let size = children.layout(i, BoxConstraints::new(w, w, 0.0, f32::INFINITY));
            if item.rows == 1 {
                contributions[item.row] = contributions[item.row].max(size.height);
            }
        }
        let heights = size_tracks(&rows, max_height, row_gap, &contributions);
        // Every item fills its cells, the measured ones included.
        for (i, item) in items.iter().enumerate() {
            let (w, h) = (width_of(item), span(&heights, row_gap, item.row, item.rows));
            children.layout(i, BoxConstraints::tight(Size::new(w, h)));
        }
        // Size and offsets (LAYOUT-LGRID-05).
        let (xs, width) = starts(&widths, column_gap);
        let (ys, height) = starts(&heights, row_gap);
        let size = constraints.constrain(Size::new(width, height));
        for (i, item) in items.iter().enumerate() {
            let offset = Vec2::new(xs[item.column], ys[item.row]);
            let child_width = children.size(i).width;
            children.set_offset(
                i,
                crate::flex::mirrored(self.text_direction, offset, size.width, child_width),
            );
        }
        size
    }

    fn min_intrinsic_width(&self, _height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.intrinsic_width(children, |c, i| c.min_intrinsic_width(i, f32::INFINITY))
    }

    fn max_intrinsic_width(&self, _height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.intrinsic_width(children, |c, i| c.max_intrinsic_width(i, f32::INFINITY))
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.intrinsic_height(width, children, |c, i, w| c.min_intrinsic_height(i, w))
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.intrinsic_height(width, children, |c, i, w| c.max_intrinsic_height(i, w))
    }
}
