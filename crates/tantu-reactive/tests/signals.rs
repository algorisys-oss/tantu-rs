//! Tests for `docs/specs/reactive/signals.md`, one or more per rule.

use std::cell::{Cell, RefCell};
use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use tantu_reactive::{
    Effect, Memo, Runtime, Scope, Signal, batch, effect, memo, on_cleanup, signal, untrack,
};

/// A shared counter for "how many times did this run".
#[derive(Clone, Default)]
struct Count(Rc<Cell<usize>>);

impl Count {
    fn inc(&self) {
        self.0.set(self.0.get() + 1);
    }
    fn get(&self) -> usize {
        self.0.get()
    }
}

/// Increments its counter when dropped.
struct DropCounter(Count);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.inc();
    }
}

/// A shared log of events, to check order.
#[derive(Clone, Default)]
struct Log(Rc<RefCell<Vec<String>>>);

impl Log {
    fn push(&self, s: impl Into<String>) {
        self.0.borrow_mut().push(s.into());
    }
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut self.0.borrow_mut())
    }
}

fn assert_copy_static<T: Copy + 'static>() {}

// Runtime and handles

#[test]
fn reactive_sig_01_handles_are_small_copy_values() {
    assert_copy_static::<Signal<String>>();
    assert_copy_static::<Memo<Vec<u8>>>();
    assert_copy_static::<Effect>();
    assert_copy_static::<Scope>();
    assert!(size_of::<Signal<String>>() <= 16);
    assert!(size_of::<Memo<Vec<u8>>>() <= 16);
    assert!(size_of::<Effect>() <= 16);
    assert!(size_of::<Scope>() <= 16);
}

#[test]
#[should_panic(expected = "no current Runtime")]
fn reactive_sig_02_signal_outside_runtime_panics() {
    let _ = signal(0);
}

#[test]
#[should_panic(expected = "no current Runtime")]
fn reactive_sig_02_memo_outside_runtime_panics() {
    let _ = memo(|| 0);
}

#[test]
#[should_panic(expected = "no current Runtime")]
fn reactive_sig_02_effect_outside_runtime_panics() {
    let _ = effect(|| {});
}

#[test]
#[should_panic(expected = "no current Runtime")]
fn reactive_sig_02_scope_outside_runtime_panics() {
    let _ = Scope::new();
}

#[test]
#[should_panic(expected = "no current Runtime")]
fn reactive_sig_02_on_cleanup_outside_runtime_panics() {
    on_cleanup(|| {});
}

#[test]
fn reactive_sig_02_batch_and_untrack_outside_runtime_just_run() {
    assert_eq!(batch(|| 3), 3);
    assert_eq!(untrack(|| 4), 4);
}

#[test]
fn reactive_sig_03_enter_nests_and_restores() {
    let a = Runtime::new();
    let b = Runtime::new();
    a.enter(|| {
        let sa = signal(1);
        b.enter(|| {
            let sb = signal(2);
            assert_eq!(sb.get(), 2);
            assert!(sa.is_disposed(), "a's handle doesn't resolve inside b");
            a.enter(|| assert_eq!(sa.get(), 1));
            assert_eq!(sb.get(), 2);
        });
        assert_eq!(sa.get(), 1);
    });
    assert!(
        catch_unwind(|| signal(0)).is_err(),
        "no runtime current after enter"
    );
}

#[test]
fn reactive_sig_03_enter_restores_after_panic() {
    let a = Runtime::new();
    let result = catch_unwind(AssertUnwindSafe(|| a.enter(|| panic!("boom"))));
    assert!(result.is_err());
    assert!(
        catch_unwind(|| signal(0)).is_err(),
        "no runtime current after the panic"
    );
    a.enter(|| assert_eq!(signal(5).get(), 5));
}

#[test]
fn reactive_sig_04_handle_outside_its_runtime_acts_disposed() {
    let a = Runtime::new();
    let s = a.enter(|| signal(1));

    // No runtime current.
    assert!(s.is_disposed());
    assert_eq!(s.try_get(), None);
    s.set(3);

    // Another runtime current, which has a node at the same index and generation.
    let b = Runtime::new();
    b.enter(|| {
        let t = signal(9);
        assert!(s.is_disposed());
        assert_eq!(s.try_get(), None);
        assert_eq!(s.try_with(|v| *v), None);
        s.set(5);
        s.update(|v| *v = 6);
        assert_eq!(t.get(), 9, "writes never reach the other runtime's node");
    });

    a.enter(|| assert_eq!(s.get(), 1, "the ignored writes changed nothing"));
}

#[test]
#[should_panic(expected = "disposed")]
fn reactive_sig_04_read_outside_its_runtime_panics() {
    let a = Runtime::new();
    let s = a.enter(|| signal(1));
    let _ = s.get();
}

#[test]
fn reactive_sig_05_node_count_and_drop_disposes_everything() {
    let rt = Runtime::new();
    assert_eq!(rt.node_count(), 0);
    let dropped = Count::default();
    let cleaned = Count::default();
    rt.enter(|| {
        let s = signal(DropCounter(dropped.clone()));
        let m = memo(move || s.with(|_| 1));
        effect(move || {
            m.get();
        });
        let _ = Scope::new();
        let cleaned = cleaned.clone();
        on_cleanup(move || cleaned.inc());
    });
    assert_eq!(rt.node_count(), 4);
    assert_eq!(dropped.get(), 0);
    drop(rt);
    assert_eq!(dropped.get(), 1, "the signal's value was dropped");
    assert_eq!(cleaned.get(), 1, "the top-level cleanup ran");
}

// Signals

#[test]
fn reactive_sig_06_get_set_update_with() {
    Runtime::new().enter(|| {
        let s = signal(String::from("a"));
        assert_eq!(s.get(), "a");
        s.set(String::from("b"));
        assert_eq!(s.get(), "b");
        s.update(|v| v.push('c'));
        assert_eq!(s.with(|v| v.len()), 2);
        assert_eq!(s.get_untracked(), "bc");
        assert_eq!(s.with_untracked(|v| v.clone()), "bc");
        assert_eq!(s.try_get().as_deref(), Some("bc"));
        assert_eq!(Signal::new(7).get(), 7);
    });
}

#[test]
fn reactive_sig_07_equal_writes_still_notify() {
    Runtime::new().enter(|| {
        let s = signal(1);
        let runs = Count::default();
        let r = runs.clone();
        effect(move || {
            s.get();
            r.inc();
        });
        assert_eq!(runs.get(), 1);
        s.set(1);
        s.update(|_| {});
        assert_eq!(runs.get(), 3);
    });
}

#[test]
fn reactive_sig_08_tracked_and_untracked_reads() {
    Runtime::new().enter(|| {
        let tracked = [signal(0), signal(0), signal(0), signal(0)];
        let untracked = [signal(0), signal(0), signal(0)];
        let runs = Count::default();
        let r = runs.clone();
        effect(move || {
            tracked[0].get();
            tracked[1].with(|_| ());
            tracked[2].try_get();
            tracked[3].try_with(|_| ());
            untracked[0].get_untracked();
            untracked[1].with_untracked(|_| ());
            untrack(|| untracked[2].get());
            r.inc();
        });
        for s in untracked {
            s.set(1);
        }
        assert_eq!(runs.get(), 1, "untracked reads don't subscribe");
        for (i, s) in tracked.into_iter().enumerate() {
            s.set(1);
            assert_eq!(runs.get(), 2 + i, "tracked read {i} subscribes");
        }
    });
}

// Memos

#[test]
fn reactive_sig_09_memos_are_lazy() {
    Runtime::new().enter(|| {
        let s = signal(2);
        let computed = Count::default();
        let c = computed.clone();
        let doubled = memo(move || {
            c.inc();
            s.get() * 2
        });
        assert_eq!(computed.get(), 0, "not computed at creation");
        assert_eq!(doubled.get(), 4);
        assert_eq!(doubled.get(), 4);
        assert_eq!(computed.get(), 1, "computed once for two reads");
        s.set(3);
        s.set(4);
        assert_eq!(computed.get(), 1, "not recomputed until read");
        assert_eq!(doubled.get(), 8);
        assert_eq!(computed.get(), 2);
        assert_eq!(Memo::new(move || s.get() + 1).get(), 5);
    });
}

#[test]
fn reactive_sig_10_equal_memo_value_stops_propagation() {
    Runtime::new().enter(|| {
        let s = signal(1);
        let parity = memo(move || s.get() % 2);
        let runs = Count::default();
        let r = runs.clone();
        effect(move || {
            parity.get();
            r.inc();
        });
        s.set(3);
        assert_eq!(runs.get(), 1, "parity unchanged, effect not re-run");
        s.set(4);
        assert_eq!(runs.get(), 2);
    });
}

#[test]
fn reactive_sig_10_equal_memo_value_keeps_the_previous_value() {
    /// Equal when `key` is equal, whatever `tag` is.
    #[derive(Clone, Debug)]
    struct Keyed {
        key: i32,
        tag: i32,
    }
    impl PartialEq for Keyed {
        fn eq(&self, other: &Self) -> bool {
            self.key == other.key
        }
    }
    Runtime::new().enter(|| {
        let s = signal(10);
        let m = memo(move || Keyed {
            key: s.get() / 10,
            tag: s.get(),
        });
        assert_eq!(m.get().tag, 10);
        s.set(11);
        assert_eq!(m.get().tag, 10, "equal value: the old one is kept");
        s.set(20);
        assert_eq!(m.get().tag, 20);
    });
}

#[test]
#[should_panic(expected = "cycle")]
fn reactive_sig_11_memo_reading_itself_panics() {
    Runtime::new().enter(|| {
        let slot: Rc<Cell<Option<Memo<i32>>>> = Rc::default();
        let s = slot.clone();
        let m = memo(move || s.get().map_or(0, |m| m.get() + 1));
        slot.set(Some(m));
        m.get();
    });
}

#[test]
#[should_panic(expected = "cycle")]
fn reactive_sig_11_memo_cycle_through_another_memo_panics() {
    Runtime::new().enter(|| {
        let slot: Rc<Cell<Option<Memo<i32>>>> = Rc::default();
        let s = slot.clone();
        let a = memo(move || s.get().map_or(0, |b| b.get() + 1));
        let b = memo(move || a.get() + 1);
        slot.set(Some(b));
        a.get();
    });
}

// Effects

#[test]
fn reactive_sig_12_effect_runs_at_creation_even_in_batch() {
    Runtime::new().enter(|| {
        let runs = Count::default();
        let r = runs.clone();
        effect(move || r.inc());
        assert_eq!(runs.get(), 1);
        batch(|| {
            let r = runs.clone();
            Effect::new(move || r.inc());
            assert_eq!(
                runs.get(),
                2,
                "ran before `effect` returned, inside the batch"
            );
        });
    });
}

#[test]
fn reactive_sig_13_effect_reruns_synchronously_only_for_its_sources() {
    Runtime::new().enter(|| {
        let a = signal(1);
        let b = signal(1);
        let seen = Rc::new(Cell::new(0));
        let runs = Count::default();
        let (seen2, r) = (seen.clone(), runs.clone());
        effect(move || {
            seen2.set(a.get());
            r.inc();
        });
        b.set(2);
        assert_eq!(runs.get(), 1, "b is not a dependency");
        a.set(7);
        assert_eq!(runs.get(), 2);
        assert_eq!(seen.get(), 7, "re-ran before `set` returned");
    });
}

#[test]
fn reactive_sig_14_effect_dependencies_are_dynamic() {
    Runtime::new().enter(|| {
        let use_a = signal(true);
        let a = signal(0);
        let b = signal(0);
        let runs = Count::default();
        let r = runs.clone();
        effect(move || {
            if use_a.get() {
                a.get();
            } else {
                b.get();
            }
            r.inc();
        });
        b.set(1);
        assert_eq!(runs.get(), 1, "b not read yet");
        use_a.set(false);
        assert_eq!(runs.get(), 2);
        a.set(1);
        assert_eq!(runs.get(), 2, "a no longer read");
        b.set(2);
        assert_eq!(runs.get(), 3);
    });
}

#[test]
fn reactive_sig_14_memo_dependencies_are_dynamic() {
    Runtime::new().enter(|| {
        let use_a = signal(true);
        let a = signal(1);
        let b = signal(2);
        let computed = Count::default();
        let c = computed.clone();
        let m = memo(move || {
            c.inc();
            if use_a.get() { a.get() } else { b.get() }
        });
        assert_eq!(m.get(), 1);
        use_a.set(false);
        assert_eq!(m.get(), 2);
        let before = computed.get();
        a.set(10);
        assert_eq!(m.get(), 2);
        assert_eq!(computed.get(), before, "a no longer read");
    });
}

#[test]
fn reactive_sig_15_diamond_is_glitch_free() {
    Runtime::new().enter(|| {
        let s = signal(1);
        let a = memo(move || s.get() + 1);
        let b = memo(move || s.get() * 2);
        let seen: Rc<RefCell<Vec<(i32, i32)>>> = Rc::default();
        let log = seen.clone();
        effect(move || log.borrow_mut().push((a.get(), b.get())));
        s.set(5);
        s.set(10);
        assert_eq!(*seen.borrow(), vec![(2, 2), (6, 10), (11, 20)]);
    });
}

#[test]
fn reactive_sig_15_every_node_runs_once_and_sees_consistent_values() {
    Runtime::new().enter(|| {
        let s = signal(1);
        let a = memo(move || s.get() + 1);
        let b = memo(move || s.get() * 2);
        let sum_runs = Count::default();
        let sr = sum_runs.clone();
        let sum = memo(move || {
            sr.inc();
            a.get() + b.get()
        });
        let runs = Count::default();
        let r = runs.clone();
        effect(move || {
            // Reads the signal directly and through the memos: always consistent.
            let s = s.get();
            assert_eq!(sum.get(), (s + 1) + s * 2);
            assert_eq!(a.get(), s + 1);
            r.inc();
        });
        for v in 2..6 {
            s.set(v);
        }
        assert_eq!(runs.get(), 5, "one run per write");
        assert_eq!(sum_runs.get(), 5, "one recompute per write");
    });
}

#[test]
fn reactive_sig_16_effects_triggered_inside_an_effect_run_after_it() {
    Runtime::new().enter(|| {
        let s = signal(0);
        let t = signal(0);
        let log = Log::default();
        let l1 = log.clone();
        effect(move || {
            let v = s.get();
            l1.push("e1 start");
            t.set(v * 10);
            l1.push("e1 end");
        });
        let l2 = log.clone();
        effect(move || l2.push(format!("e2 {}", t.get())));
        log.take();

        s.set(1);
        assert_eq!(log.take(), ["e1 start", "e1 end", "e2 10"]);
    });
}

#[test]
fn reactive_sig_16_effect_writing_its_own_source_reruns() {
    Runtime::new().enter(|| {
        let n = signal(0);
        let runs = Count::default();
        let r = runs.clone();
        effect(move || {
            r.inc();
            let v = n.get();
            if v < 3 {
                n.set(v + 1);
            }
        });
        assert_eq!(n.get_untracked(), 3);
        assert_eq!(runs.get(), 4);
    });
}

// Batching

#[test]
fn reactive_sig_17_batch_defers_effects_to_the_outermost_batch() {
    Runtime::new().enter(|| {
        let a = signal(1);
        let b = signal(1);
        let sum = memo(move || a.get() + b.get());
        let seen: Rc<RefCell<Vec<i32>>> = Rc::default();
        let log = seen.clone();
        effect(move || log.borrow_mut().push(a.get() + b.get()));

        let result = batch(|| {
            a.set(10);
            batch(|| b.set(20));
            assert_eq!(
                seen.borrow().len(),
                1,
                "inner batch end doesn't run effects"
            );
            assert_eq!(a.get(), 10, "reads see new values");
            assert_eq!(sum.get(), 30, "memos too");
            "done"
        });
        assert_eq!(result, "done");
        assert_eq!(*seen.borrow(), vec![2, 30], "one run after the batch");
    });
}

// Ownership and disposal

#[test]
fn reactive_sig_18_disposing_a_scope_disposes_what_it_owns() {
    let rt = Runtime::new();
    rt.enter(|| {
        let scope = Scope::new();
        let (s, m, e, inner, inner_s) = scope.run(|| {
            let s = signal(1);
            let m = memo(move || s.get());
            let e = effect(move || {
                m.get();
            });
            let inner = Scope::new();
            let inner_s = inner.run(|| signal(2));
            (s, m, e, inner, inner_s)
        });
        assert_eq!(rt.node_count(), 6);
        scope.dispose();
        assert!(scope.is_disposed());
        assert!(s.is_disposed() && m.is_disposed() && e.is_disposed());
        assert!(inner.is_disposed() && inner_s.is_disposed());
        assert_eq!(rt.node_count(), 0);
    });
}

#[test]
fn reactive_sig_18_effects_and_memos_own_what_they_create() {
    let rt = Runtime::new();
    rt.enter(|| {
        let made: Rc<Cell<Option<Signal<i32>>>> = Rc::default();
        let m2 = made.clone();
        let e = effect(move || m2.set(Some(signal(1))));
        let from_memo: Rc<Cell<Option<Signal<i32>>>> = Rc::default();
        let f2 = from_memo.clone();
        let m = memo(move || {
            f2.set(Some(signal(1)));
            0
        });
        m.get();
        let (inner_e, inner_m) = (made.get().unwrap(), from_memo.get().unwrap());
        e.dispose();
        assert!(inner_e.is_disposed());
        m.dispose();
        assert!(inner_m.is_disposed());
        assert_eq!(rt.node_count(), 0);
    });
}

#[test]
fn reactive_sig_19_rerun_disposes_what_the_previous_run_created() {
    let rt = Runtime::new();
    rt.enter(|| {
        let s = signal(0);
        let created: Rc<RefCell<Vec<Signal<i32>>>> = Rc::default();
        let cleaned = Count::default();
        let (c2, cl) = (created.clone(), cleaned.clone());
        effect(move || {
            s.get();
            c2.borrow_mut().push(signal(0));
            let cl = cl.clone();
            on_cleanup(move || cl.inc());
        });
        let count = rt.node_count();
        s.set(1);
        s.set(2);
        let created = created.borrow();
        assert!(created[0].is_disposed() && created[1].is_disposed());
        assert!(!created[2].is_disposed());
        assert_eq!(cleaned.get(), 2, "one cleanup per re-run");
        assert_eq!(rt.node_count(), count, "no growth across re-runs");
    });
}

#[test]
fn reactive_sig_19_memo_recompute_disposes_what_it_created() {
    Runtime::new().enter(|| {
        let s = signal(0);
        let created: Rc<RefCell<Vec<Signal<i32>>>> = Rc::default();
        let c2 = created.clone();
        let m = memo(move || {
            c2.borrow_mut().push(signal(0));
            s.get()
        });
        m.get();
        s.set(1);
        m.get();
        let created = created.borrow();
        assert!(created[0].is_disposed());
        assert!(!created[1].is_disposed());
    });
}

#[test]
fn reactive_sig_20_cleanups_run_once_children_first_in_order() {
    let rt = Runtime::new();
    let log = Log::default();
    rt.enter(|| {
        let scope = Scope::new();
        scope.run(|| {
            let l = log.clone();
            on_cleanup(move || l.push("outer 1"));
            let inner = Scope::new();
            let l = log.clone();
            inner.run(|| on_cleanup(move || l.push("inner")));
            let l = log.clone();
            on_cleanup(move || l.push("outer 2"));
        });
        assert!(log.take().is_empty());
        scope.dispose();
        assert_eq!(log.take(), ["inner", "outer 1", "outer 2"]);
        scope.dispose();
        assert!(log.take().is_empty(), "exactly once");

        let l = log.clone();
        on_cleanup(move || l.push("top level"));
    });
    assert!(log.take().is_empty());
    drop(rt);
    assert_eq!(log.take(), ["top level"]);
}

#[test]
fn reactive_sig_20_effect_cleanup_runs_before_rerun_and_on_dispose() {
    Runtime::new().enter(|| {
        let s = signal(0);
        let log = Log::default();
        let l = log.clone();
        let e = effect(move || {
            let v = s.get();
            l.push(format!("run {v}"));
            let l = l.clone();
            on_cleanup(move || l.push(format!("cleanup {v}")));
        });
        s.set(1);
        e.dispose();
        s.set(2);
        assert_eq!(log.take(), ["run 0", "cleanup 0", "run 1", "cleanup 1"]);
    });
}

#[test]
fn reactive_sig_21_disposed_handles() {
    Runtime::new().enter(|| {
        let dropped = Count::default();
        let s = signal(DropCounter(dropped.clone()));
        s.dispose();
        assert_eq!(dropped.get(), 1, "value dropped at disposal");
        assert!(s.is_disposed());
        assert!(s.try_with(|_| ()).is_none());
        s.set(DropCounter(dropped.clone()));
        assert_eq!(dropped.get(), 2, "the value passed to set is dropped");
        s.update(|_| panic!("not called"));
        s.dispose();

        let n = signal(1);
        n.dispose();
        assert_eq!(n.try_get(), None);

        let src = signal(1);
        let m = memo(move || src.get());
        m.dispose();
        assert!(m.is_disposed());
        assert_eq!(m.try_get(), None);
        assert_eq!(m.try_with(|v| *v), None);
        m.dispose();

        let runs = Count::default();
        let r = runs.clone();
        let e = effect(move || {
            src.get();
            r.inc();
        });
        e.dispose();
        assert!(e.is_disposed());
        src.set(2);
        assert_eq!(runs.get(), 1, "a disposed effect never runs again");
        e.dispose();
    });
}

#[test]
#[should_panic(expected = "disposed")]
fn reactive_sig_21_get_on_disposed_signal_panics() {
    Runtime::new().enter(|| {
        let s = signal(1);
        s.dispose();
        s.get();
    });
}

#[test]
#[should_panic(expected = "disposed")]
fn reactive_sig_21_with_untracked_on_disposed_signal_panics() {
    Runtime::new().enter(|| {
        let s = signal(1);
        s.dispose();
        s.with_untracked(|_| ());
    });
}

#[test]
#[should_panic(expected = "disposed")]
fn reactive_sig_21_get_on_disposed_memo_panics() {
    Runtime::new().enter(|| {
        let m = memo(|| 1);
        m.dispose();
        m.get();
    });
}

#[test]
#[should_panic(expected = "disposed")]
fn reactive_sig_21_run_on_disposed_scope_panics() {
    Runtime::new().enter(|| {
        let scope = Scope::new();
        scope.dispose();
        scope.run(|| ());
    });
}

#[test]
fn reactive_sig_22_no_leaks() {
    let rt = Runtime::new();
    let dropped = Count::default();
    rt.enter(|| {
        let base = rt.node_count();
        for _ in 0..100 {
            let scope = Scope::new();
            scope.run(|| {
                let s = signal(DropCounter(dropped.clone()));
                let in_memo = DropCounter(dropped.clone());
                let m = memo(move || {
                    let _ = &in_memo;
                    s.with(|_| 1)
                });
                let in_effect = DropCounter(dropped.clone());
                effect(move || {
                    let _ = &in_effect;
                    m.get();
                });
                let in_cleanup = DropCounter(dropped.clone());
                on_cleanup(move || drop(in_cleanup));
            });
            scope.dispose();
        }
        assert_eq!(rt.node_count(), base);
        assert_eq!(dropped.get(), 400, "every value and closure dropped once");
    });
    drop(rt);
    assert_eq!(dropped.get(), 400, "nothing left to drop");
}

// Misuse

#[test]
#[should_panic(expected = "borrowed")]
fn reactive_sig_23_set_inside_own_with_panics() {
    Runtime::new().enter(|| {
        let s = signal(1);
        s.with(|_| s.set(2));
    });
}

#[test]
#[should_panic(expected = "borrowed")]
fn reactive_sig_23_set_inside_own_update_panics() {
    Runtime::new().enter(|| {
        let s = signal(1);
        s.update(|_| s.set(2));
    });
}
