//! [`Id`] and [`Arena`]: a generational arena for long-lived nodes (elements, render objects,
//! reactive nodes) that refer to each other by id instead of by pointer.
//!
//! Removing a value makes every id for it stale for good: lookups return `None`, even after its
//! slot is reused for a new value. Nothing panics on a stale or foreign id.
//!
//! Spec: `docs/specs/core/arena.md`.

use std::fmt;
use std::num::NonZeroU32;

/// A handle to a value in an [`Arena`]: a slot index plus the slot's generation.
///
/// Cheap to copy, compare and hash. `Option<Id>` is the same size as `Id`. Crates that want to
/// keep ids of different arenas apart wrap it in a newtype (`pub struct ElementId(Id)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    index: u32,
    generation: NonZeroU32,
}

impl Id {
    /// The slot index. Unique among live ids of one arena, reused after removal.
    #[inline]
    pub const fn index(self) -> u32 {
        todo!()
    }

    /// The slot's generation when the id was issued. Always ≥ 1.
    #[inline]
    pub const fn generation(self) -> u32 {
        todo!()
    }

    /// A stable 64-bit encoding: generation in the high 32 bits, index in the low 32 bits.
    /// Never 0. Used for Scene element ids and AccessKit node ids.
    #[inline]
    pub const fn to_bits(self) -> u64 {
        todo!()
    }

    /// The inverse of [`Id::to_bits`]. `None` if the generation part (high 32 bits) is 0.
    #[inline]
    pub const fn from_bits(bits: u64) -> Option<Id> {
        let _ = bits;
        todo!()
    }
}

/// One slot of an arena.
enum Slot<T> {
    /// Holds a live value, issued with `generation`.
    Occupied { generation: NonZeroU32, value: T },
    /// Free. The next value stored here gets `generation`; `next_free` links the free list.
    Vacant {
        generation: NonZeroU32,
        next_free: Option<u32>,
    },
    /// Its generation is used up: never handed out again.
    Retired,
}

/// A generational arena: a `Vec` of slots with a free list.
///
/// ```
/// use tantu_core::Arena;
///
/// let mut arena = Arena::new();
/// let a = arena.insert("a");
/// assert_eq!(arena.get(a), Some(&"a"));
/// assert_eq!(arena.remove(a), Some("a"));
/// let b = arena.insert("b"); // reuses a's slot
/// assert_eq!(arena.get(a), None); // the old id stays stale
/// assert_eq!(arena.get(b), Some(&"b"));
/// ```
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    /// Head of the free list (most recently freed slot).
    free_head: Option<u32>,
    /// Number of occupied slots.
    len: usize,
}

impl<T> Arena<T> {
    /// An empty arena. Does not allocate.
    #[inline]
    pub const fn new() -> Self {
        Arena {
            slots: Vec::new(),
            free_head: None,
            len: 0,
        }
    }

    /// An empty arena with room for `capacity` values before it reallocates.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let _ = capacity;
        todo!()
    }

    /// Stores `value` and returns its id.
    ///
    /// # Panics
    ///
    /// If the arena already has `u32::MAX` slots (live, free or retired), as `Vec` does on
    /// capacity overflow.
    pub fn insert(&mut self, value: T) -> Id {
        let _ = value;
        todo!()
    }

    /// Removes and returns the value, or `None` if `id` is stale or foreign.
    pub fn remove(&mut self, id: Id) -> Option<T> {
        let _ = id;
        todo!()
    }

    /// The value for `id`, or `None` if it is stale or foreign.
    #[inline]
    pub fn get(&self, id: Id) -> Option<&T> {
        let _ = id;
        todo!()
    }

    /// Mutable access to the value for `id`, or `None` if it is stale or foreign.
    #[inline]
    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let _ = id;
        todo!()
    }

    /// Mutable access to two different values at once (for example a parent and a child), in
    /// argument order. `None` if either id is stale or foreign, or both name the same slot.
    pub fn get_pair_mut(&mut self, a: Id, b: Id) -> Option<(&mut T, &mut T)> {
        let _ = (a, b);
        todo!()
    }

    /// True if `id` refers to a live value.
    #[inline]
    pub fn contains(&self, id: Id) -> bool {
        let _ = id;
        todo!()
    }

    /// Number of live values.
    #[inline]
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True if there are no live values.
    #[inline]
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Number of values the arena can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        todo!()
    }

    /// Live values with their ids, in ascending slot index order.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        todo!();
        #[allow(unreachable_code)]
        std::iter::empty()
    }

    /// Live values with their ids, mutably, in ascending slot index order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)> {
        todo!();
        #[allow(unreachable_code)]
        std::iter::empty()
    }

    /// Removes every value for which `keep` returns false, in ascending slot index order.
    /// Kept values keep their ids.
    pub fn retain(&mut self, keep: impl FnMut(Id, &mut T) -> bool) {
        let _ = keep;
        todo!()
    }

    /// Removes all values. Every id issued so far becomes stale. Keeps the capacity.
    pub fn clear(&mut self) {
        todo!()
    }
}

/// The index for a new slot when the arena has `slot_count` slots.
///
/// # Panics
///
/// If `slot_count` is `u32::MAX` or more: indices are `u32` and `u32::MAX` is not used.
fn new_slot_index(slot_count: usize) -> u32 {
    let _ = slot_count;
    todo!()
}

impl<T> Default for Arena<T> {
    #[inline]
    fn default() -> Self {
        Arena::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Puts the (only) occupied slot at generation `u32::MAX`, as if it had been reused that often.
    fn force_generation_to_max<T>(arena: &mut Arena<T>, id: Id) -> Id {
        let max = NonZeroU32::MAX;
        match &mut arena.slots[id.index as usize] {
            Slot::Occupied { generation, .. } => *generation = max,
            _ => panic!("slot must be occupied"),
        }
        Id {
            index: id.index,
            generation: max,
        }
    }

    #[test]
    fn core_arena_14_exhausted_generation_retires_slot() {
        let mut arena = Arena::new();
        let first = arena.insert('a');
        let old = force_generation_to_max(&mut arena, first);
        let other = arena.insert('b');

        assert_eq!(arena.remove(old), Some('a'));
        assert!(matches!(arena.slots[old.index as usize], Slot::Retired));
        assert_eq!(arena.len(), 1);
        assert_eq!(arena.get(other), Some(&'b'));

        // the retired slot is never handed out again
        let next = arena.insert('c');
        assert_ne!(next.index(), old.index());
        assert_eq!(arena.get(old), None);
        assert!(!arena.contains(old));
        assert_eq!(arena.remove(old), None);
        assert_eq!(arena.iter().count(), 2);

        // clear doesn't bring it back either
        arena.clear();
        for _ in 0..3 {
            assert_ne!(arena.insert('d').index(), old.index());
        }
    }

    #[test]
    fn core_arena_14_retire_via_retain_and_clear() {
        let mut arena = Arena::new();
        let a = arena.insert(1);
        let a = force_generation_to_max(&mut arena, a);
        arena.retain(|_, _| false);
        assert!(matches!(arena.slots[a.index as usize], Slot::Retired));

        let mut arena = Arena::new();
        let b = arena.insert(2);
        let b = force_generation_to_max(&mut arena, b);
        arena.clear();
        assert!(matches!(arena.slots[b.index as usize], Slot::Retired));
        assert_ne!(arena.insert(3).index(), b.index());
    }

    #[test]
    fn core_arena_15_slot_index_limit() {
        assert_eq!(new_slot_index(0), 0);
        assert_eq!(new_slot_index(u32::MAX as usize - 1), u32::MAX - 1);
    }

    #[test]
    #[should_panic(expected = "u32::MAX")]
    fn core_arena_15_insert_beyond_limit_panics() {
        new_slot_index(u32::MAX as usize);
    }
}
