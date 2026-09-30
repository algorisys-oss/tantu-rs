//! Tests for `docs/specs/core/arena.md`, one or more per rule CORE-ARENA-NN.
//! CORE-ARENA-14 and -15 need the arena's internals and live in `src/arena.rs`.

use std::cell::Cell;
use std::collections::{BTreeSet, HashSet};
use std::mem::size_of;
use std::rc::Rc;

use tantu_core::{Arena, Id};

/// Counts its drops in a shared counter.
struct DropCounter(Rc<Cell<usize>>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

// ---- Ids --------------------------------------------------------------------------------

#[test]
fn core_arena_01_id_is_8_bytes() {
    assert_eq!(size_of::<Id>(), 8);
    assert_eq!(size_of::<Option<Id>>(), 8);
}

#[test]
fn core_arena_02_bits_round_trip() {
    let mut arena = Arena::new();
    let ids: Vec<Id> = (0..10).map(|i| arena.insert(i)).collect();
    for &id in &ids {
        let bits = id.to_bits();
        assert_ne!(bits, 0);
        assert_eq!(bits >> 32, u64::from(id.generation()));
        assert_eq!(bits & 0xFFFF_FFFF, u64::from(id.index()));
        assert_eq!(Id::from_bits(bits), Some(id));
    }

    assert_eq!(Id::from_bits(0), None);
    assert_eq!(Id::from_bits(0xFFFF_FFFF), None);
    let id = Id::from_bits((7 << 32) | 3).expect("generation 7 is valid");
    assert_eq!((id.index(), id.generation()), (3, 7));
    let id = Id::from_bits(u64::MAX).expect("generation u32::MAX is valid");
    assert_eq!((id.index(), id.generation()), (u32::MAX, u32::MAX));
}

#[test]
fn core_arena_03_equality_order_hash() {
    let a = Id::from_bits((1 << 32) | 5).expect("valid");
    let a2 = Id::from_bits((1 << 32) | 5).expect("valid");
    let b = Id::from_bits((2 << 32) | 5).expect("valid"); // same index, newer generation
    let c = Id::from_bits((1 << 32) | 6).expect("valid"); // next index
    assert_eq!(a, a2);
    assert_ne!(a, b);
    assert_ne!(a, c);
    // by index, then generation
    assert!(a < b);
    assert!(b < c);

    let set: HashSet<Id> = [a, a2, b, c].into_iter().collect();
    assert_eq!(set.len(), 3);
    let ordered: Vec<Id> = [c, b, a]
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(ordered, vec![a, b, c]);
}

// ---- Insert, lookup, remove -------------------------------------------------------------

#[test]
fn core_arena_04_insert_then_get() {
    let mut arena = Arena::new();
    let ids: Vec<Id> = (0..100).map(|i| arena.insert(i * 10)).collect();
    for (i, &id) in ids.iter().enumerate() {
        assert_eq!(arena.get(id), Some(&(i * 10)));
        assert!(arena.contains(id));
    }
    let unique: HashSet<Id> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len());

    *arena.get_mut(ids[3]).expect("live") += 1;
    assert_eq!(arena.get(ids[3]), Some(&31));
}

#[test]
fn core_arena_05_generations() {
    let mut arena = Arena::new();
    let a = arena.insert("a");
    assert_eq!(a.generation(), 1);
    assert_eq!(arena.remove(a), Some("a"));
    assert!(!arena.contains(a));
    let b = arena.insert("b");
    assert_eq!(b.index(), a.index());
    assert_eq!(b.generation(), 2);
}

#[test]
fn core_arena_06_stale_ids() {
    let mut arena = Arena::new();
    let a = arena.insert(1);
    let other = arena.insert(99);
    arena.remove(a);

    assert_eq!(arena.get(a), None);
    assert_eq!(arena.get_mut(a), None);
    assert!(!arena.contains(a));
    assert_eq!(arena.get_pair_mut(a, other), None);
    assert_eq!(arena.remove(a), None);
    assert_eq!(arena.len(), 1);

    // after the slot is reused, the old id still doesn't resolve
    let b = arena.insert(2);
    assert_eq!(b.index(), a.index());
    assert_eq!(arena.get(a), None);
    assert!(!arena.contains(a));
    assert_eq!(arena.remove(a), None);
    assert_eq!(arena.get(b), Some(&2));
    assert_eq!(arena.len(), 2);
}

#[test]
fn core_arena_07_foreign_ids() {
    let mut arena = Arena::new();
    let live = arena.insert(1);
    let far = Id::from_bits((1 << 32) | 1000).expect("valid");
    let max = Id::from_bits(u64::MAX).expect("valid");
    for id in [far, max] {
        assert_eq!(arena.get(id), None);
        assert_eq!(arena.get_mut(id), None);
        assert!(!arena.contains(id));
        assert_eq!(arena.remove(id), None);
        assert_eq!(arena.get_pair_mut(live, id), None);
        assert_eq!(arena.get_pair_mut(id, live), None);
    }
    assert_eq!(arena.len(), 1);

    let empty: Arena<i32> = Arena::new();
    assert_eq!(empty.get(live), None);

    // an id from another arena that matches a live slot resolves to it (documented limit)
    let mut other = Arena::new();
    let o = other.insert(7);
    assert_eq!(o, live);
    assert_eq!(arena.get(o), Some(&1));
}

#[test]
fn core_arena_08_lifo_slot_reuse() {
    let mut arena = Arena::new();
    let ids: Vec<Id> = (0..5).map(|i| arena.insert(i)).collect();
    arena.remove(ids[1]);
    arena.remove(ids[3]);
    // most recently freed first
    let x = arena.insert(10);
    let y = arena.insert(11);
    assert_eq!(x.index(), ids[3].index());
    assert_eq!(y.index(), ids[1].index());
    assert_ne!(x, ids[3]);
    assert_ne!(y, ids[1]);
    // then the arena grows
    let z = arena.insert(12);
    assert_eq!(z.index(), 5);
}

#[test]
fn core_arena_09_len_and_is_empty() {
    let mut arena = Arena::new();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    let a = arena.insert(1);
    let b = arena.insert(2);
    assert_eq!(arena.len(), 2);
    assert!(!arena.is_empty());
    arena.remove(a);
    arena.remove(a); // stale: no change
    assert_eq!(arena.len(), 1);
    arena.remove(b);
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

// ---- Pair access ------------------------------------------------------------------------

#[test]
fn core_arena_10_get_pair_mut() {
    let mut arena = Arena::new();
    let a = arena.insert(String::from("a"));
    let b = arena.insert(String::from("b"));
    let c = arena.insert(String::from("c"));

    {
        let (x, y) = arena
            .get_pair_mut(a, c)
            .expect("both live, different slots");
        x.push('1');
        y.push('3');
    }
    {
        // argument order, also when the second slot comes first
        let (x, y) = arena
            .get_pair_mut(c, b)
            .expect("both live, different slots");
        assert_eq!((x.as_str(), y.as_str()), ("c3", "b"));
    }
    assert_eq!(arena.get(a).map(String::as_str), Some("a1"));

    assert!(arena.get_pair_mut(a, a).is_none());
    // same slot, different generations
    arena.remove(b);
    let b2 = arena.insert(String::from("b2"));
    assert_eq!(b2.index(), b.index());
    assert!(arena.get_pair_mut(b, b2).is_none());
    assert!(arena.get_pair_mut(b2, b).is_none());
    assert!(arena.get_pair_mut(a, b).is_none());
}

// ---- Iteration and bulk operations ------------------------------------------------------

#[test]
fn core_arena_11_iteration_order() {
    let mut arena = Arena::new();
    let ids: Vec<Id> = (0..6).map(|i| arena.insert(i)).collect();
    arena.remove(ids[0]);
    arena.remove(ids[4]);
    let reused = arena.insert(40); // takes slot 4, after slots 1..3 in index order

    let seen: Vec<(Id, i32)> = arena.iter().map(|(id, v)| (id, *v)).collect();
    assert_eq!(
        seen,
        vec![
            (ids[1], 1),
            (ids[2], 2),
            (ids[3], 3),
            (reused, 40),
            (ids[5], 5)
        ]
    );

    for (_, v) in arena.iter_mut() {
        *v *= 2;
    }
    let values: Vec<i32> = arena.iter().map(|(_, v)| *v).collect();
    assert_eq!(values, vec![2, 4, 6, 80, 10]);
    let ids_mut: Vec<Id> = arena.iter_mut().map(|(id, _)| id).collect();
    let ids_ref: Vec<Id> = arena.iter().map(|(id, _)| id).collect();
    assert_eq!(ids_mut, ids_ref);

    assert_eq!(Arena::<i32>::new().iter().count(), 0);
}

#[test]
fn core_arena_12_retain() {
    let mut arena = Arena::new();
    let ids: Vec<Id> = (0..6).map(|i| arena.insert(i)).collect();
    arena.remove(ids[2]);

    let mut visited = Vec::new();
    arena.retain(|id, v| {
        visited.push(id);
        *v += 100;
        *v % 2 == 0
    });
    assert_eq!(visited, vec![ids[0], ids[1], ids[3], ids[4], ids[5]]);
    assert_eq!(arena.len(), 2);
    assert_eq!(arena.get(ids[0]), Some(&100));
    assert_eq!(arena.get(ids[4]), Some(&104));
    for i in [1, 3, 5] {
        assert!(!arena.contains(ids[i]));
    }
    // removed slots got a new generation
    let reused = arena.insert(7);
    assert_eq!(reused.generation(), 2);
}

#[test]
fn core_arena_13_clear() {
    let drops = Rc::new(Cell::new(0));
    let mut arena = Arena::new();
    let ids: Vec<Id> = (0..10)
        .map(|_| arena.insert(DropCounter(drops.clone())))
        .collect();
    let capacity = arena.capacity();

    arena.clear();
    assert_eq!(drops.get(), 10);
    assert!(arena.is_empty());
    assert_eq!(arena.capacity(), capacity);
    for &id in &ids {
        assert!(!arena.contains(id));
    }

    // old slots are reused with new generations
    let fresh = arena.insert(DropCounter(drops.clone()));
    assert!(fresh.index() < 10);
    assert!(fresh.generation() > 1);
    assert!(!ids.contains(&fresh));
    assert_eq!(arena.capacity(), capacity);
}

// ---- Ownership --------------------------------------------------------------------------

#[test]
fn core_arena_16_values_dropped_exactly_once() {
    let drops = Rc::new(Cell::new(0));
    let item = || DropCounter(drops.clone());

    let mut arena = Arena::new();
    let a = arena.insert(item());
    let b = arena.insert(item());
    let _c = arena.insert(item());
    let _d = arena.insert(item());

    // remove hands the value over without dropping it
    let taken = arena.remove(a).expect("live");
    assert_eq!(drops.get(), 0);
    drop(taken);
    assert_eq!(drops.get(), 1);
    assert!(arena.remove(a).is_none());
    assert_eq!(drops.get(), 1);

    // retain drops what it removes
    arena.retain(|id, _| id != b);
    assert_eq!(drops.get(), 2);

    // dropping the arena drops the rest
    drop(arena);
    assert_eq!(drops.get(), 4);
}

// ---- Performance and allocation ---------------------------------------------------------

#[test]
fn core_arena_17_no_growth_in_steady_state() {
    // The arena's only heap memory is its slot Vec, so an unchanged capacity means no
    // allocation happened.
    let arena: Arena<u64> = Arena::new();
    assert_eq!(arena.capacity(), 0);

    let mut arena = Arena::with_capacity(64);
    let capacity = arena.capacity();
    assert!(capacity >= 64);

    let mut live: Vec<Id> = (0..32).map(|i| arena.insert(i)).collect();
    for round in 0..10_000u64 {
        let k = (round as usize * 7) % live.len();
        let id = live[k];
        assert!(arena.contains(id));
        assert!(arena.get(id).is_some());
        assert!(arena.get_mut(id).is_some());
        let other = live[(k + 1) % live.len()];
        assert!(arena.get_pair_mut(id, other).is_some());
        arena.remove(id);
        live[k] = arena.insert(round);
    }
    assert_eq!(arena.capacity(), capacity);
    assert_eq!(arena.len(), 32);
}

#[test]
fn core_arena_17_debug_lists_live_values() {
    let mut arena = Arena::new();
    let a = arena.insert("x");
    arena.insert("y");
    arena.remove(a);
    let text = format!("{arena:?}");
    assert!(text.contains("\"y\""), "{text}");
    assert!(!text.contains("\"x\""), "{text}");
}
