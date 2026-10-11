//! Tests for `docs/specs/reactive/timers.md`, rules REACTIVE-TIME-01..05.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use tantu_reactive::{Runtime, Scope, Wake, effect, on_cleanup, signal, ticker, timer};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
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

/// Increments its counter when dropped.
struct DropCounter(Rc<Cell<usize>>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn reactive_time_01_tick_runs_due_timers_then_tickers() {
    let rt = Runtime::new();
    let t0 = Instant::now();
    let log = Log::default();
    rt.enter(|| {
        assert_eq!(rt.now(), None);
        let l = log.clone();
        timer(ms(20), move || l.push("t20"));
        let l = log.clone();
        timer(ms(10), move || l.push("t10"));
        let l = log.clone();
        timer(ms(10), move || l.push("t10b"));
        let l = log.clone();
        let tk = ticker(move |elapsed| l.push(format!("tick {}", elapsed.as_millis())));
        tk.start();
        // Timers and the ticker count from the first tick.
        assert_eq!(rt.tick(t0), 1);
        assert_eq!(rt.now(), Some(t0));
        assert_eq!(log.take(), ["tick 0"]);
        assert_eq!(rt.tick(t0 + ms(10)), 3);
        assert_eq!(log.take(), ["t10", "t10b", "tick 10"]);
        assert_eq!(rt.tick(t0 + ms(15)), 1);
        assert_eq!(log.take(), ["tick 15"]);
        assert_eq!(rt.tick(t0 + ms(25)), 2);
        assert_eq!(log.take(), ["t20", "tick 25"]);
        assert_eq!(rt.now(), Some(t0 + ms(25)));
    });
}

#[test]
fn reactive_time_02_timer_fires_once_cancel_and_context() {
    let rt = Runtime::new();
    let t0 = Instant::now();
    let log = Log::default();
    rt.enter(|| {
        rt.tick(t0);
        // Zero delay: the next tick, once.
        let l = log.clone();
        let zero = timer(Duration::ZERO, move || l.push("zero"));
        assert!(zero.is_pending());
        assert_eq!(rt.tick(t0), 1);
        assert!(!zero.is_pending());
        assert_eq!(rt.tick(t0 + ms(1)), 0);
        assert_eq!(log.take(), ["zero"]);
        // Cancel drops the callback without running it.
        let dropped = Rc::new(Cell::new(0));
        let guard = DropCounter(dropped.clone());
        let l = log.clone();
        let cancelled = timer(ms(5), move || {
            let _ = &guard;
            l.push("cancelled");
        });
        cancelled.cancel();
        assert!(!cancelled.is_pending());
        assert_eq!(dropped.get(), 1);
        cancelled.cancel();
        assert_eq!(rt.tick(t0 + ms(10)), 0);
        assert!(log.take().is_empty());
        // The callback runs with the runtime current and the timer's owner current: a write
        // notifies, and a cleanup registered inside belongs to that owner.
        let hits = signal(0);
        let l = log.clone();
        effect(move || l.push(format!("hits {}", hits.get())));
        let scope = Scope::new();
        let l = log.clone();
        scope.run(|| {
            timer(ms(5), move || {
                hits.set(1);
                let l = l.clone();
                on_cleanup(move || l.push("cleanup"));
            });
        });
        assert_eq!(log.take(), ["hits 0"]);
        assert_eq!(rt.tick(t0 + ms(15)), 1);
        assert_eq!(log.take(), ["hits 1"]);
        scope.dispose();
        assert_eq!(log.take(), ["cleanup"]);
    });
}

#[test]
fn reactive_time_03_ticker_lifecycle() {
    let rt = Runtime::new();
    let t0 = Instant::now();
    let log = Log::default();
    rt.enter(|| {
        rt.tick(t0);
        let l = log.clone();
        let tk = ticker(move |elapsed| l.push(format!("{}", elapsed.as_millis())));
        assert!(!tk.is_active());
        assert_eq!(rt.tick(t0 + ms(1)), 0);
        tk.start();
        assert!(tk.is_active());
        assert_eq!(rt.tick(t0 + ms(1)), 1);
        assert_eq!(rt.tick(t0 + ms(4)), 1);
        assert_eq!(log.take(), ["0", "3"]);
        tk.stop();
        assert!(!tk.is_active());
        assert_eq!(rt.tick(t0 + ms(10)), 0);
        // Restarting counts from zero again.
        tk.start();
        assert_eq!(rt.tick(t0 + ms(12)), 1);
        assert_eq!(log.take(), ["2"]);
        // A ticker stopping itself isn't called again; a timer created inside a callback
        // first runs at a later tick.
        let holder: Rc<Cell<Option<tantu_reactive::Ticker>>> = Rc::default();
        let (h, l) = (holder.clone(), log.clone());
        let once = ticker(move |_| {
            l.push("once");
            if let Some(me) = h.get() {
                me.stop();
            }
            let l = l.clone();
            timer(Duration::ZERO, move || l.push("inner"));
        });
        holder.set(Some(once));
        once.start();
        tk.stop();
        assert_eq!(rt.tick(t0 + ms(13)), 1);
        assert_eq!(log.take(), ["once"]);
        assert!(!once.is_active());
        assert_eq!(rt.tick(t0 + ms(14)), 1);
        assert_eq!(log.take(), ["inner"]);
    });
}

#[test]
fn reactive_time_04_ownership_and_disposal() {
    let rt = Runtime::new();
    let t0 = Instant::now();
    let dropped = Rc::new(Cell::new(0));
    rt.enter(|| {
        rt.tick(t0);
        let base = rt.node_count();
        let scope = Scope::new();
        let guard_a = DropCounter(dropped.clone());
        let guard_b = DropCounter(dropped.clone());
        let (t, tk) = scope.run(|| {
            let t = timer(ms(5), move || {
                let _ = &guard_a;
                panic!("cancelled by disposal");
            });
            let tk = ticker(move |_| {
                let _ = &guard_b;
                panic!("stopped by disposal");
            });
            tk.start();
            (t, tk)
        });
        assert!(t.is_pending());
        assert!(tk.is_active());
        scope.dispose();
        assert_eq!(dropped.get(), 2, "each callback dropped once");
        assert!(!t.is_pending());
        assert!(!tk.is_active());
        assert_eq!(rt.tick(t0 + ms(10)), 0);
        // Methods on disposed handles are ignored.
        t.cancel();
        tk.start();
        tk.stop();
        assert!(!tk.is_active());
        assert_eq!(rt.node_count(), base);
        assert_eq!(rt.next_wake(), None);
    });
}

#[test]
fn reactive_time_05_next_wake() {
    let rt = Runtime::new();
    let t0 = Instant::now();
    rt.enter(|| {
        assert_eq!(rt.next_wake(), None);
        let late = timer(ms(10), || {});
        // Before the first tick the deadline is unresolved.
        assert_eq!(rt.next_wake(), Some(Wake::Now));
        rt.tick(t0);
        assert_eq!(rt.next_wake(), Some(Wake::At(t0 + ms(10))));
        let soon = timer(ms(5), || {});
        assert_eq!(rt.next_wake(), Some(Wake::At(t0 + ms(5))));
        soon.cancel();
        assert_eq!(rt.next_wake(), Some(Wake::At(t0 + ms(10))));
        let tk = ticker(|_| {});
        assert_eq!(rt.next_wake(), Some(Wake::At(t0 + ms(10))));
        tk.start();
        assert_eq!(rt.next_wake(), Some(Wake::Now));
        tk.stop();
        assert_eq!(rt.next_wake(), Some(Wake::At(t0 + ms(10))));
        rt.tick(t0 + ms(10));
        assert!(!late.is_pending());
        assert_eq!(rt.next_wake(), None);
        let scope = Scope::new();
        scope.run(|| {
            timer(ms(1), || {});
        });
        assert_eq!(rt.next_wake(), Some(Wake::At(t0 + ms(11))));
        scope.dispose();
        assert_eq!(rt.next_wake(), None);
    });
}
