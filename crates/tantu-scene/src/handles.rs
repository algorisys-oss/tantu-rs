//! Opaque handles carried by Scene commands: element ids, image and font handles, custom kinds.

use std::num::NonZeroU64;

use tantu_core::Id;

/// The element a command was painted for. Encodes an arena [`Id`] with [`Id::to_bits`].
///
/// `Option<ElementId>` is 8 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(NonZeroU64);

impl ElementId {
    /// From a raw value; `None` for 0.
    #[inline]
    pub const fn from_raw(raw: u64) -> Option<ElementId> {
        todo!()
    }

    /// The raw value, never 0.
    #[inline]
    pub const fn to_raw(self) -> u64 {
        todo!()
    }
}

impl From<Id> for ElementId {
    #[inline]
    fn from(id: Id) -> Self {
        todo!()
    }
}

/// Handle to an image registered with the renderer's resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(NonZeroU64);

impl ImageId {
    /// From a raw value; `None` for 0.
    #[inline]
    pub const fn from_raw(raw: u64) -> Option<ImageId> {
        todo!()
    }

    /// The raw value, never 0.
    #[inline]
    pub const fn to_raw(self) -> u64 {
        todo!()
    }
}

/// Handle to a font face registered with the renderer's resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId(NonZeroU64);

impl FontId {
    /// From a raw value; `None` for 0.
    #[inline]
    pub const fn from_raw(raw: u64) -> Option<FontId> {
        todo!()
    }

    /// The raw value, never 0.
    #[inline]
    pub const fn to_raw(self) -> u64 {
        todo!()
    }
}

/// Identifies what a custom command draws, so a renderer can find its handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CustomKind(pub u32);
