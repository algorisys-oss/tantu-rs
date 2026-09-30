# Id and generational arena

- **Status:** Agreed
- **Crate:** `tantu-core`
- **Plan item:** Phase 0, "`tantu-core` → `Id` and generational arena"
- **Related:** [ADR 0001](../../adr/0001-retained-tree-and-fine-grained-reactivity.md) (elements
  and render objects live in a generational arena and refer to each other by id),
  [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md) (Scene commands carry element ids)

## Purpose

Long-lived nodes (elements, render objects, reactive nodes) are stored in an arena and refer to
each other by `Id`, not by pointer or `Rc`. That gives stable identity for focus, IME, a11y and
animations, keeps Rust ownership simple (no reference cycles), and lets a Scene command or an
AccessKit node carry a plain 64-bit id.

A **generational** arena makes stale ids safe: once a value is removed, every `Id` that pointed
at it stops resolving, even after its slot is reused for a new value. Lookups on a stale id
return `None`; they never return the new occupant.

Users are other Tantu crates (`tantu-reactive`, `tantu-view`, `tantu-scene`); app developers
don't use the arena directly.

## Scope

In scope:

- `Id`: a small `Copy` handle (slot index + generation) with a stable 64-bit encoding.
- `Arena<T>`: insert, remove, lookup, mutable pair lookup, iteration, retain, clear.
- Behavior for stale ids, slot reuse, generation overflow and capacity limits.

Out of scope:

- Typed ids per arena (`ElementId`, `RenderId`, …). Crates wrap `Id` in their own newtypes; see
  open question 1.
- Thread safety beyond what `Send`/`Sync` of `T` gives (no internal locking).
- Tree structure (parent/child links). That belongs to `tantu-view`.
- Serialization format (serde). `to_bits`/`from_bits` are enough for Scene and AccessKit.
- Shrinking the arena's memory; it only grows.

## Public API

Module `tantu_core::arena`, re-exported at the crate root as `tantu_core::{Arena, Id}`.

```rust
/// A handle to a value in an [`Arena`]: a slot index plus the slot's generation.
///
/// Cheap to copy, compare and hash. `Option<Id>` is the same size as `Id`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id { /* index: u32, generation: NonZeroU32 */ }

impl Id {
    /// The slot index. Unique among live ids of one arena, reused after removal.
    pub const fn index(self) -> u32;
    /// The slot's generation when the id was issued. Always ≥ 1.
    pub const fn generation(self) -> u32;
    /// A stable 64-bit encoding: generation in the high 32 bits, index in the low 32 bits.
    /// Never 0. Used for Scene element ids and AccessKit node ids.
    pub const fn to_bits(self) -> u64;
    /// The inverse of `to_bits`. `None` if the generation part is 0.
    pub const fn from_bits(bits: u64) -> Option<Id>;
}

/// A generational arena: a `Vec` of slots with a free list.
pub struct Arena<T> { /* private */ }

impl<T> Arena<T> {
    /// An empty arena. Does not allocate.
    pub const fn new() -> Self;
    /// An empty arena with room for `capacity` values before it reallocates.
    pub fn with_capacity(capacity: usize) -> Self;

    /// Stores `value` and returns its id.
    pub fn insert(&mut self, value: T) -> Id;
    /// Removes and returns the value, or `None` if `id` is stale or foreign.
    pub fn remove(&mut self, id: Id) -> Option<T>;

    /// The value for `id`, or `None` if it is stale or foreign.
    pub fn get(&self, id: Id) -> Option<&T>;
    /// Mutable access to the value for `id`, or `None` if it is stale or foreign.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut T>;
    /// Mutable access to two different values at once (for example a parent and a child).
    /// `None` if either id is stale or foreign, or both name the same slot.
    pub fn get_pair_mut(&mut self, a: Id, b: Id) -> Option<(&mut T, &mut T)>;
    /// True if `id` refers to a live value.
    pub fn contains(&self, id: Id) -> bool;

    /// Number of live values.
    pub fn len(&self) -> usize;
    /// True if there are no live values.
    pub fn is_empty(&self) -> bool;
    /// Number of values the arena can hold without reallocating.
    pub fn capacity(&self) -> usize;

    /// Live values with their ids, in ascending slot index order.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)>;
    /// Live values with their ids, mutably, in ascending slot index order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut T)>;
    /// Removes every value for which `keep` returns false, in ascending slot index order.
    pub fn retain(&mut self, keep: impl FnMut(Id, &mut T) -> bool);
    /// Removes all values. Every id issued so far becomes stale. Keeps the capacity.
    pub fn clear(&mut self);
}

impl<T> Default for Arena<T> { /* Arena::new() */ }
impl<T: Debug> Debug for Arena<T> { /* the live (Id, value) pairs, as a map */ }
```

`Arena<T>` is `Send`/`Sync` when `T` is. No `Index`/`IndexMut`: they would have to panic on a
stale id.

## Behavior

Ids

- **CORE-ARENA-01:** `size_of::<Id>()` and `size_of::<Option<Id>>()` are both 8 bytes.
- **CORE-ARENA-02:** `Id::from_bits(id.to_bits()) == Some(id)` for every id. `to_bits` puts the
  generation in the high 32 bits and the index in the low 32 bits, so it is never 0.
  `from_bits(b)` is `None` exactly when `b >> 32 == 0`.
- **CORE-ARENA-03:** Two ids are equal (and hash equally) exactly when both index and generation
  are equal. Ids order by index, then generation. (Ordering exists so ids can be `BTreeMap` keys;
  it has no meaning beyond that.)

Insert, lookup, remove

- **CORE-ARENA-04:** `insert` returns an id for which `get` returns the inserted value and
  `contains` is true, until that id is removed. Ids of live values are all different.
- **CORE-ARENA-05:** A new slot starts at generation 1. `remove(id)` on a live id returns the
  value, makes `id` stale and increments the slot's generation.
- **CORE-ARENA-06:** On a stale id, `get`, `get_mut` and `get_pair_mut` return `None`,
  `contains` returns false, and `remove` returns `None` and changes nothing. This holds after the
  slot has been reused: the old id never resolves to the new occupant.
- **CORE-ARENA-07:** An id whose index is past the end of the arena ("foreign" id, for example
  from another arena or from `from_bits`) behaves like a stale id: `None`/false, never a panic.
  An id from another arena whose index and generation happen to match a live slot resolves to
  that slot; the arena cannot tell (see open question 1).
- **CORE-ARENA-08:** Freed slots are reused before the arena grows, most recently freed first.
  A reused slot keeps its incremented generation, so the new id differs from the old one.
- **CORE-ARENA-09:** `len` is the number of live values: +1 per `insert`, −1 per successful
  `remove`. `is_empty` is `len() == 0`.

Pair access

- **CORE-ARENA-10:** `get_pair_mut(a, b)` returns mutable references to both values when both
  ids are live and name different slots, in argument order. It returns `None` when `a` and `b`
  name the same slot (even with different generations) or either is stale or foreign.

Iteration and bulk operations

- **CORE-ARENA-11:** `iter` and `iter_mut` yield each live value exactly once with its current id,
  in ascending slot index order (not insertion order). They skip free slots.
- **CORE-ARENA-12:** `retain(keep)` calls `keep` once per live value, in ascending slot index
  order, and removes (as `remove` does, generation incremented) exactly those for which it
  returns false. Values it keeps keep their ids.
- **CORE-ARENA-13:** `clear` removes every value (dropping it), makes every id issued so far
  stale, and keeps the capacity. Later inserts reuse the old slots with new generations.

Limits

- **CORE-ARENA-14:** When a slot's generation would pass `u32::MAX`, the slot is retired
  instead: it is never handed out again, so no old id can ever match a new value. The arena keeps
  working; `len` and lookups are unaffected.
- **CORE-ARENA-15:** An arena holds at most `u32::MAX` slots (indices `0..u32::MAX`). `insert`
  beyond that panics with a message saying so, as `Vec` does on capacity overflow. It can only
  happen with more than four billion values alive or retired at once.

Ownership

- **CORE-ARENA-16:** Values are dropped exactly once: by `remove` or `retain` (as they are
  removed), by `clear`, or when the arena is dropped. `remove` hands the value to the caller
  without dropping it.

## Performance and allocation

- **CORE-ARENA-17:** `insert` (when a free slot exists or capacity remains), `remove`, `get`,
  `get_mut`, `get_pair_mut` and `contains` are O(1) and don't allocate. After warm-up, a
  steady insert/remove cycle does not allocate at all. `new` does not allocate.
- Iteration, `retain` and `clear` are O(slots), including free ones.
- A slot costs the size of `T` plus a generation and a free-list link (8 bytes when `T` has a
  niche the enum can use, more otherwise). No per-value heap allocation.
- No `unsafe`: slots are a `Vec` of an enum (`Occupied { generation, value }` /
  `Vacant { generation, next_free }`), and `get_pair_mut` uses `split_at_mut`.

## Open questions

Resolved (2026-09-30, the proposals were accepted):

1. **Untyped or typed ids?** One untyped `Id`. The Scene and AccessKit need a plain id that
   crosses crate boundaries, and a generic `Id<T>` would need manual `Copy`/`Eq`/`Hash` impls and
   turbofish-heavy code. Crates that want type safety wrap it (`pub struct ElementId(Id)`), which
   catches using an id with the wrong arena at compile time where it matters.
2. **`unsafe` in `tantu-core`.** None. `tantu-core` gets `#![forbid(unsafe_code)]` like the other
   non-backend crates, and the "`tantu-core` arena internals" exception is removed from
   `AGENTS.md`.
3. **Write it or use `slotmap`?** Write it. `tantu-core` has no dependencies by design, and we
   need exact control over generation overflow and the id encoding for Scene/AccessKit. `slotmap`
   (or `thunderdome`) is the fallback if ours grows complicated.
