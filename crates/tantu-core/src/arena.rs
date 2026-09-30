//! [`Id`] and [`Arena`]: a generational arena for long-lived nodes (elements, render objects,
//! reactive nodes) that refer to each other by id instead of by pointer.
//!
//! Removing a value makes every id for it stale for good: lookups return `None`, even after its
//! slot is reused for a new value. Nothing panics on a stale or foreign id.
//!
//! Spec: `docs/specs/core/arena.md`.

use std::fmt;
use std::mem;
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
        self.index
    }

    /// The slot's generation when the id was issued. Always ≥ 1.
    #[inline]
    pub const fn generation(self) -> u32 {
        self.generation.get()
    }

    /// A stable 64-bit encoding: generation in the high 32 bits, index in the low 32 bits.
    /// Never 0. Used for Scene element ids and AccessKit node ids.
    #[inline]
    pub const fn to_bits(self) -> u64 {
        ((self.generation.get() as u64) << 32) | self.index as u64
    }

    /// The inverse of [`Id::to_bits`]. `None` if the generation part (high 32 bits) is 0.
    #[inline]
    pub const fn from_bits(bits: u64) -> Option<Id> {
        match NonZeroU32::new((bits >> 32) as u32) {
            Some(generation) => Some(Id {
                index: bits as u32,
                generation,
            }),
            None => None,
        }
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

impl<T> Slot<T> {
    /// The value, if the slot is occupied by the generation `id` was issued with.
    #[inline]
    fn get(&self, id: Id) -> Option<&T> {
        match self {
            Slot::Occupied { generation, value } if *generation == id.generation => Some(value),
            _ => None,
        }
    }

    /// Mutable form of [`Slot::get`].
    #[inline]
    fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        match self {
            Slot::Occupied { generation, value } if *generation == id.generation => Some(value),
            _ => None,
        }
    }
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
        Arena {
            slots: Vec::with_capacity(capacity),
            free_head: None,
            len: 0,
        }
    }

    /// Stores `value` and returns its id.
    ///
    /// # Panics
    ///
    /// If the arena already has `u32::MAX` slots (live, free or retired), as `Vec` does on
    /// capacity overflow.
    pub fn insert(&mut self, value: T) -> Id {
        if let Some(index) = self.free_head {
            let slot = &mut self.slots[index as usize];
            let Slot::Vacant {
                generation,
                next_free,
            } = *slot
            else {
                unreachable!("the free list only links vacant slots");
            };
            self.free_head = next_free;
            *slot = Slot::Occupied { generation, value };
            self.len += 1;
            return Id { index, generation };
        }

        let index = new_slot_index(self.slots.len());
        let generation = NonZeroU32::MIN;
        self.slots.push(Slot::Occupied { generation, value });
        self.len += 1;
        Id { index, generation }
    }

    /// Removes and returns the value, or `None` if `id` is stale or foreign.
    pub fn remove(&mut self, id: Id) -> Option<T> {
        if self.contains(id) {
            self.take(id.index)
        } else {
            None
        }
    }

    /// The value for `id`, or `None` if it is stale or foreign.
    #[inline]
    pub fn get(&self, id: Id) -> Option<&T> {
        self.slots.get(id.index as usize)?.get(id)
    }

    /// Mutable access to the value for `id`, or `None` if it is stale or foreign.
    #[inline]
    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        self.slots.get_mut(id.index as usize)?.get_mut(id)
    }

    /// Mutable access to two different values at once (for example a parent and a child), in
    /// argument order. `None` if either id is stale or foreign, or both name the same slot.
    pub fn get_pair_mut(&mut self, a: Id, b: Id) -> Option<(&mut T, &mut T)> {
        let (i, j) = (a.index as usize, b.index as usize);
        // Checking liveness first also keeps both indices in bounds for the split.
        if i == j || !self.contains(a) || !self.contains(b) {
            return None;
        }
        // Split between the two slots so both halves can be borrowed mutably.
        let (lo, hi) = (i.min(j), i.max(j));
        let (head, tail) = self.slots.split_at_mut(hi);
        let (lo_slot, hi_slot) = (head.get_mut(lo)?, tail.first_mut()?);
        if i < j {
            Some((lo_slot.get_mut(a)?, hi_slot.get_mut(b)?))
        } else {
            Some((hi_slot.get_mut(a)?, lo_slot.get_mut(b)?))
        }
    }

    /// True if `id` refers to a live value.
    #[inline]
    pub fn contains(&self, id: Id) -> bool {
        self.get(id).is_some()
    }

    /// Number of live values.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// True if there are no live values.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Number of values the arena can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Live values with their ids, in ascending slot index order.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| match slot {
                Slot::Occupied { generation, value } => Some((
                    Id {
                        index: index as u32,
                        generation: *generation,
                    },
                    value,
                )),
                _ => None,
            })
    }

    /// Live values with their ids, mutably, in ascending slot index order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| match slot {
                Slot::Occupied { generation, value } => Some((
                    Id {
                        index: index as u32,
                        generation: *generation,
                    },
                    value,
                )),
                _ => None,
            })
    }

    /// Removes every value for which `keep` returns false, in ascending slot index order.
    /// Kept values keep their ids.
    pub fn retain(&mut self, mut keep: impl FnMut(Id, &mut T) -> bool) {
        for index in 0..self.slots.len() {
            let remove = match &mut self.slots[index] {
                Slot::Occupied { generation, value } => {
                    let id = Id {
                        index: index as u32,
                        generation: *generation,
                    };
                    !keep(id, value)
                }
                _ => false,
            };
            if remove {
                self.take(index as u32);
            }
        }
    }

    /// Removes all values. Every id issued so far becomes stale. Keeps the capacity.
    pub fn clear(&mut self) {
        for index in 0..self.slots.len() {
            self.take(index as u32);
        }
    }

    /// Empties slot `index` if it is occupied and returns its value. The slot moves to the next
    /// generation and onto the free list, or is retired when its generation is used up.
    fn take(&mut self, index: u32) -> Option<T> {
        let slot = self.slots.get_mut(index as usize)?;
        let Slot::Occupied { generation, .. } = *slot else {
            return None;
        };
        let emptied = match generation.checked_add(1) {
            Some(generation) => {
                let vacant = Slot::Vacant {
                    generation,
                    next_free: self.free_head,
                };
                self.free_head = Some(index);
                vacant
            }
            None => Slot::Retired,
        };
        self.len -= 1;
        match mem::replace(slot, emptied) {
            Slot::Occupied { value, .. } => Some(value),
            _ => None,
        }
    }
}

/// The index for a new slot when the arena has `slot_count` slots.
///
/// # Panics
///
/// If `slot_count` is `u32::MAX` or more: indices are `u32` and `u32::MAX` is not used.
fn new_slot_index(slot_count: usize) -> u32 {
    match u32::try_from(slot_count) {
        Ok(index) if index < u32::MAX => index,
        _ => panic!("Arena is full: it can hold at most u32::MAX slots"),
    }
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
