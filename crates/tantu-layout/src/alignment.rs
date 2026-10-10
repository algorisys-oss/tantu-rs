//! [`Alignment`]: a point in a rectangle, for positioning a child in its parent.

use tantu_core::Vec2;

/// A point in a rectangle: `x` and `y` run from -1 (left/top) through 0 (center) to 1
/// (right/bottom). Values outside -1..=1 are allowed and point outside. Flutter's `Alignment`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alignment {
    /// Horizontal position: -1 left, 0 center, 1 right.
    pub x: f32,
    /// Vertical position: -1 top, 0 center, 1 bottom.
    pub y: f32,
}

impl Alignment {
    /// The top-left corner, (-1, -1).
    pub const TOP_LEFT: Alignment = Alignment::new(0.0, 0.0);
    /// The middle of the top edge, (0, -1).
    pub const TOP_CENTER: Alignment = Alignment::new(0.0, 0.0);
    /// The top-right corner, (1, -1).
    pub const TOP_RIGHT: Alignment = Alignment::new(0.0, 0.0);
    /// The middle of the left edge, (-1, 0).
    pub const CENTER_LEFT: Alignment = Alignment::new(0.0, 0.0);
    /// The center, (0, 0).
    pub const CENTER: Alignment = Alignment::new(0.0, 0.0);
    /// The middle of the right edge, (1, 0).
    pub const CENTER_RIGHT: Alignment = Alignment::new(0.0, 0.0);
    /// The bottom-left corner, (-1, 1).
    pub const BOTTOM_LEFT: Alignment = Alignment::new(0.0, 0.0);
    /// The middle of the bottom edge, (0, 1).
    pub const BOTTOM_CENTER: Alignment = Alignment::new(0.0, 0.0);
    /// The bottom-right corner, (1, 1).
    pub const BOTTOM_RIGHT: Alignment = Alignment::new(0.0, 0.0);

    /// An alignment at `(x, y)`, stored as given.
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Alignment { x, y }
    }

    /// The offset of this point in a free space of `free` (the parent's size minus the
    /// child's): `((x + 1) / 2 · free.x, (y + 1) / 2 · free.y)`. Negative free space gives a
    /// negative offset (the child overflows evenly for `CENTER`).
    pub fn along_offset(self, free: Vec2) -> Vec2 {
        let _ = free;
        todo!()
    }
}

impl Default for Alignment {
    /// [`Alignment::CENTER`].
    fn default() -> Self {
        todo!()
    }
}
