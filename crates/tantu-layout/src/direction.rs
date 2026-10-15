//! [`TextDirection`] and the directional alignment and insets resolved against it. Spec:
//! `docs/specs/widgets/directionality.md` (LAYOUT-DIR-01).

use tantu_core::EdgeInsets;

use crate::Alignment;

/// The direction text and horizontal layout run in (Flutter's `TextDirection`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextDirection {
    /// Left to right.
    #[default]
    Ltr,
    /// Right to left (Arabic, Hebrew, Persian, Urdu).
    Rtl,
}

/// An alignment whose x runs from the start edge (-1) to the end edge (1) (Flutter's
/// `AlignmentDirectional`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlignmentDirectional {
    /// -1 at the start edge, 1 at the end edge.
    pub start: f32,
    /// -1 at the top, 1 at the bottom.
    pub y: f32,
}

impl AlignmentDirectional {
    /// The top start corner.
    pub const TOP_START: Self = Self::new(-1.0, -1.0);
    /// The top center.
    pub const TOP_CENTER: Self = Self::new(0.0, -1.0);
    /// The top end corner.
    pub const TOP_END: Self = Self::new(1.0, -1.0);
    /// The start edge's center.
    pub const CENTER_START: Self = Self::new(-1.0, 0.0);
    /// The center.
    pub const CENTER: Self = Self::new(0.0, 0.0);
    /// The end edge's center.
    pub const CENTER_END: Self = Self::new(1.0, 0.0);
    /// The bottom start corner.
    pub const BOTTOM_START: Self = Self::new(-1.0, 1.0);
    /// The bottom center.
    pub const BOTTOM_CENTER: Self = Self::new(0.0, 1.0);
    /// The bottom end corner.
    pub const BOTTOM_END: Self = Self::new(1.0, 1.0);

    /// An alignment `start` along the text direction and `y` down.
    pub const fn new(start: f32, y: f32) -> Self {
        AlignmentDirectional { start, y }
    }

    /// The alignment for `direction`: start is left for `Ltr`, right for `Rtl`.
    pub fn resolve(self, direction: TextDirection) -> Alignment {
        match direction {
            TextDirection::Ltr => Alignment::new(self.start, self.y),
            TextDirection::Rtl => Alignment::new(-self.start, self.y),
        }
    }
}

/// Insets with start and end sides instead of left and right (Flutter's
/// `EdgeInsetsDirectional`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeInsetsDirectional {
    /// The inset at the start edge.
    pub start: f32,
    /// The top inset.
    pub top: f32,
    /// The inset at the end edge.
    pub end: f32,
    /// The bottom inset.
    pub bottom: f32,
}

impl EdgeInsetsDirectional {
    /// Insets from the four sides.
    pub const fn new(start: f32, top: f32, end: f32, bottom: f32) -> Self {
        EdgeInsetsDirectional {
            start,
            top,
            end,
            bottom,
        }
    }

    /// The insets for `direction`: start on the left for `Ltr`, on the right for `Rtl`.
    pub fn resolve(self, direction: TextDirection) -> EdgeInsets {
        match direction {
            TextDirection::Ltr => {
                EdgeInsets::from_ltrb(self.start, self.top, self.end, self.bottom)
            }
            TextDirection::Rtl => {
                EdgeInsets::from_ltrb(self.end, self.top, self.start, self.bottom)
            }
        }
    }
}
