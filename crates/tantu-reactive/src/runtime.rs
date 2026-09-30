//! [`Runtime`] and the reactive graph.
//!
//! The graph is push-pull with three node states, as in Reactively and Leptos. A write marks
//! the written signal's subscribers dirty and everything further down "check", and queues the
//! effects it reaches. Queued effects are then brought up to date by pulling: a "check" node
//! first updates its memo sources and recomputes only if one of them actually changed. Every
//! affected node runs at most once and after its sources, which makes updates glitch-free.
//!
//! No borrow of the graph is held while user code runs (computations, cleanups, value drops),
//! so user code can call back into the runtime freely.

use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use tantu_core::{Arena, Id};

/// Identifies a runtime, so a handle never resolves in another runtime's arena.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct RuntimeId(u64);

/// Source of runtime ids. Only ever incremented; 2^64 runtimes won't be created.
static NEXT_RUNTIME_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// The entered runtimes on this thread, innermost last.
    static CURRENT: RefCell<Vec<Rc<Inner>>> = const { RefCell::new(Vec::new()) };
}

/// The reactive graph: every signal, memo, effect and scope of one app or window.
///
/// Not `Send`. Handles resolve against the runtime that is current on this thread, set by
/// [`Runtime::enter`]. Dropping the runtime disposes every node it still holds.
pub struct Runtime {
    inner: Rc<Inner>,
}

impl Runtime {
    /// An empty runtime.
    pub fn new() -> Runtime {
        let mut nodes = Arena::new();
        let root = nodes.insert(Node::new(Kind::Scope, State::Clean, None));
        Runtime {
            inner: Rc::new(Inner {
                id: RuntimeId(NEXT_RUNTIME_ID.fetch_add(1, Ordering::Relaxed)),
                graph: RefCell::new(Graph {
                    nodes,
                    root,
                    owner: root,
                    observer: None,
                    depth: 0,
                    flushing: false,
                    queue: VecDeque::new(),
                    mark_stack: Vec::new(),
                    stamp: 0,
                }),
            }),
        }
    }

    /// Runs `f` with this runtime as the current one on this thread, then restores the previous
    /// current runtime (also if `f` panics). Calls may nest.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        /// Pops the runtime pushed by `enter`, also during unwinding.
        struct Exit;
        impl Drop for Exit {
            fn drop(&mut self) {
                CURRENT.with(|c| c.borrow_mut().pop());
            }
        }
        CURRENT.with(|c| c.borrow_mut().push(self.inner.clone()));
        let _exit = Exit;
        f()
    }

    /// Number of live nodes (signals, memos, effects and scopes). For leak tests.
    pub fn node_count(&self) -> usize {
        let graph = self.inner.graph.borrow();
        // The root owner is an internal node, not counted.
        graph.nodes.len() - usize::from(graph.nodes.contains(graph.root))
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let root = self.inner.graph.borrow().root;
        // Entered, so cleanups and value drops can still use their handles.
        self.enter(|| self.inner.dispose(root));
    }
}

/// The current runtime, or `None` outside [`Runtime::enter`].
pub(crate) fn current() -> Option<Rc<Inner>> {
    CURRENT.with(|c| c.borrow().last().cloned())
}

/// The current runtime; panics outside [`Runtime::enter`].
pub(crate) fn expect_current(what: &str) -> Rc<Inner> {
    match current() {
        Some(inner) => inner,
        None => panic!("{what}: no current Runtime; call it inside `Runtime::enter`"),
    }
}

/// The current runtime if it is the one with id `rt`.
pub(crate) fn lookup(rt: RuntimeId) -> Option<Rc<Inner>> {
    current().filter(|inner| inner.id == rt)
}

/// Panic for reads of a disposed handle (ADR 0007).
pub(crate) fn disposed(what: &str) -> ! {
    panic!("{what} is disposed, or used while its Runtime is not current")
}

/// Panic for a value borrowed against its own write.
pub(crate) fn borrowed(what: &str) -> ! {
    panic!("{what}: the value is already borrowed (a write inside its own `with` or `update`)")
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum State {
    /// Up to date.
    Clean,
    /// A source further up changed; memo sources may or may not have changed.
    Check,
    /// A direct source changed; must re-run.
    Dirty,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    Signal,
    Memo,
    Effect,
    Scope,
}

/// A computation: returns true if the value changed (always true for effects).
type Compute = Rc<RefCell<dyn FnMut() -> bool>>;

pub(crate) struct Node {
    kind: Kind,
    state: State,
    /// True while the computation runs, to detect cycles.
    running: bool,
    /// `Rc<RefCell<T>>` for signals, `Rc<RefCell<Option<T>>>` for memos.
    value: Option<Rc<dyn Any>>,
    compute: Option<Compute>,
    sources: Vec<Id>,
    subscribers: Vec<Id>,
    /// While running: how many of `sources` the run has read again, in the same order.
    cursor: usize,
    /// While running: sources read after the first read that differed from `sources`.
    new_sources: Vec<Id>,
    /// Stamp of the current run, while running.
    run: u64,
    /// Scratch stamp: the last run that tracked this node, or a reconcile mark.
    mark: u64,
    owner: Option<Id>,
    owned: Vec<Id>,
    cleanups: Vec<Box<dyn FnOnce()>>,
}

impl Node {
    /// Whether this node, while running, has already read `source` in the current run. Until
    /// then an older subscription to `source` must not mark it: the run will read the new value.
    fn has_read(&self, source: Id) -> bool {
        if self.new_sources.is_empty() && self.sources.get(self.cursor) == Some(&source) {
            return false; // The next read expected; sources hold no duplicates.
        }
        let read = &self.sources[..self.cursor.min(self.sources.len())];
        read.contains(&source) || self.new_sources.contains(&source)
    }

    fn new(kind: Kind, state: State, owner: Option<Id>) -> Node {
        Node {
            kind,
            state,
            running: false,
            value: None,
            compute: None,
            sources: Vec::new(),
            subscribers: Vec::new(),
            cursor: 0,
            new_sources: Vec::new(),
            run: 0,
            mark: 0,
            owner,
            owned: Vec::new(),
            cleanups: Vec::new(),
        }
    }
}

struct Graph {
    nodes: Arena<Node>,
    /// Owner of nodes created outside any scope or computation.
    root: Id,
    /// The current owner.
    owner: Id,
    /// The running computation, which reads subscribe.
    observer: Option<Id>,
    /// Nesting of batches and running computations; effects wait while it is non-zero.
    depth: u32,
    /// True while the effect queue is being drained.
    flushing: bool,
    /// Effects to bring up to date.
    queue: VecDeque<Id>,
    /// Reused by `mark_subscribers`: (node, state to raise it to, the source that marks it).
    mark_stack: Vec<(Id, State, Id)>,
    /// Last stamp handed out by `next_stamp`.
    stamp: u64,
}

impl Graph {
    /// Raises the state of `id`'s subscribers to `state` and of everything below them to
    /// `Check`, queuing effects that were clean.
    fn mark_subscribers(&mut self, id: Id, state: State) {
        let mut stack = mem::take(&mut self.mark_stack);
        if let Some(node) = self.nodes.get(id) {
            stack.extend(node.subscribers.iter().map(|&s| (s, state, id)));
        }
        while let Some((id, state, from)) = stack.pop() {
            let Some(node) = self.nodes.get_mut(id) else {
                continue;
            };
            if node.state >= state || (node.running && !node.has_read(from)) {
                continue;
            }
            if node.state == State::Clean && node.kind == Kind::Effect {
                self.queue.push_back(id);
            }
            node.state = state;
            stack.extend(node.subscribers.iter().map(|&s| (s, State::Check, id)));
        }
        self.mark_stack = stack;
    }

    /// A stamp no node carries yet.
    fn next_stamp(&mut self) -> u64 {
        self.stamp += 1;
        self.stamp
    }

    /// Makes the running computation (if any) depend on `id`.
    ///
    /// Reads that repeat the previous run's sources in order only advance a cursor. Other reads
    /// subscribe at once (so writes later in the run are seen) and are reconciled with the old
    /// sources by [`Graph::end_tracking`]. Re-runs that read the same sources therefore don't
    /// touch any subscriber list.
    fn track(&mut self, id: Id) {
        let Some(observer) = self.observer else {
            return;
        };
        let Some(run) = self.nodes.get(observer).map(|n| n.run) else {
            return;
        };
        let Some(source) = self.nodes.get_mut(id) else {
            return;
        };
        if source.mark == run {
            return; // Already read in this run.
        }
        source.mark = run;
        let Some(node) = self.nodes.get_mut(observer) else {
            return;
        };
        if node.new_sources.is_empty() && node.sources.get(node.cursor) == Some(&id) {
            node.cursor += 1;
            return;
        }
        node.new_sources.push(id);
        if let Some(source) = self.nodes.get_mut(id) {
            source.subscribers.push(observer);
        }
    }

    /// Prepares `id` to record the sources of a new run.
    fn begin_tracking(&mut self, id: Id) {
        let run = self.next_stamp();
        if let Some(node) = self.nodes.get_mut(id) {
            node.cursor = 0;
            node.new_sources.clear();
            node.run = run;
        }
    }

    /// Makes the sources read by the run of `id` its sources: drops the extra subscription of
    /// reads that turned out to be duplicates or old sources, and unsubscribes from old sources
    /// it no longer read.
    fn end_tracking(&mut self, id: Id) {
        let Some(node) = self.nodes.get_mut(id) else {
            return;
        };
        let cursor = node.cursor.min(node.sources.len());
        if node.new_sources.is_empty() && cursor == node.sources.len() {
            return; // Same sources as last time.
        }
        let mut sources = mem::take(&mut node.sources);
        let mut new = mem::take(&mut node.new_sources);
        let stale = if cursor < sources.len() {
            sources.split_off(cursor)
        } else {
            Vec::new()
        };

        let (old, keep) = (self.next_stamp(), self.next_stamp());
        for (list, mark) in [(&stale, old), (&sources, keep)] {
            for &s in list {
                if let Some(s) = self.nodes.get_mut(s) {
                    s.mark = mark;
                }
            }
        }
        let nodes = &mut self.nodes;
        new.retain(|&s| {
            let Some(source) = nodes.get_mut(s) else {
                return false;
            };
            if source.mark == keep || source.mark == old {
                // Already subscribed before this read subscribed again.
                if let Some(i) = source.subscribers.iter().rposition(|&x| x == id) {
                    source.subscribers.remove(i);
                }
            }
            let first = source.mark != keep;
            source.mark = keep;
            first
        });
        for &s in &stale {
            if let Some(source) = self.nodes.get_mut(s) {
                if source.mark != keep {
                    source.subscribers.retain(|&x| x != id);
                }
            }
        }
        if sources.is_empty() {
            // Typically the first run: the new reads become the sources without copying.
            mem::swap(&mut sources, &mut new);
        } else {
            sources.append(&mut new);
        }
        if let Some(node) = self.nodes.get_mut(id) {
            node.sources = sources;
            node.new_sources = new;
            node.cursor = 0;
        }
    }

    /// Ids of `id` and everything it owns, owned nodes before their owner.
    fn subtree(&self, id: Id, out: &mut Vec<Id>) {
        let mut stack = vec![(id, false)];
        while let Some((id, expanded)) = stack.pop() {
            if expanded {
                out.push(id);
                continue;
            }
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            stack.push((id, true));
            stack.extend(node.owned.iter().rev().map(|&c| (c, false)));
        }
    }

    /// Removes `id` from the arena and from its sources' and subscribers' lists.
    fn unlink(&mut self, id: Id) -> Option<Node> {
        let node = self.nodes.remove(id)?;
        for &source in &node.sources {
            if let Some(s) = self.nodes.get_mut(source) {
                s.subscribers.retain(|&x| x != id);
            }
        }
        for &sub in &node.subscribers {
            if let Some(s) = self.nodes.get_mut(sub) {
                if let Some(i) = s.sources.iter().position(|&x| x == id) {
                    s.sources.remove(i);
                    if i < s.cursor {
                        s.cursor -= 1;
                    }
                }
            }
        }
        Some(node)
    }
}

/// The shared state behind a [`Runtime`].
pub(crate) struct Inner {
    pub(crate) id: RuntimeId,
    graph: RefCell<Graph>,
}

impl Inner {
    /// Adds a node owned by the current owner.
    pub(crate) fn create(
        &self,
        kind: Kind,
        value: Option<Rc<dyn Any>>,
        compute: Option<Compute>,
    ) -> Id {
        let mut graph = self.graph.borrow_mut();
        let owner = if graph.nodes.contains(graph.owner) {
            graph.owner
        } else {
            graph.root
        };
        // Memos and effects start dirty: they have never run.
        let state = if compute.is_some() {
            State::Dirty
        } else {
            State::Clean
        };
        let mut node = Node::new(kind, state, Some(owner));
        node.value = value;
        node.compute = compute;
        let id = graph.nodes.insert(node);
        if let Some(owner) = graph.nodes.get_mut(owner) {
            owner.owned.push(id);
        }
        id
    }

    /// True if `id` is a live node of kind `kind`.
    pub(crate) fn is_alive(&self, id: Id, kind: Kind) -> bool {
        self.graph
            .borrow()
            .nodes
            .get(id)
            .is_some_and(|n| n.kind == kind)
    }

    /// The value cell of a live node, subscribing the running computation if `track`.
    pub(crate) fn value(&self, id: Id, track: bool) -> Option<Rc<dyn Any>> {
        let mut graph = self.graph.borrow_mut();
        let value = graph.nodes.get(id)?.value.clone()?;
        if track {
            graph.track(id);
        }
        Some(value)
    }

    /// Panics if `id`'s computation is running: it would read its own value.
    pub(crate) fn check_cycle(&self, id: Id) {
        let running = self.graph.borrow().nodes.get(id).is_some_and(|n| n.running);
        if running {
            panic!("cycle detected: a memo read itself, directly or through other memos");
        }
    }

    /// Notifies the subscribers of signal `id` that it changed.
    pub(crate) fn notify(&self, id: Id) {
        self.graph.borrow_mut().mark_subscribers(id, State::Dirty);
        self.flush_if_idle();
    }

    /// Brings `id` up to date: recomputes it if a source changed.
    pub(crate) fn update_if_necessary(&self, id: Id) {
        let state = match self.graph.borrow().nodes.get(id) {
            Some(node) => node.state,
            None => return,
        };
        if state == State::Check {
            // Update memo sources in order until one of them changes (which marks `id` dirty).
            let mut i = 0;
            loop {
                let source = {
                    let graph = self.graph.borrow();
                    let Some(node) = graph.nodes.get(id) else {
                        return;
                    };
                    if node.state != State::Check {
                        break;
                    }
                    let Some(&source) = node.sources.get(i) else {
                        break;
                    };
                    graph
                        .nodes
                        .get(source)
                        .is_some_and(|s| s.kind == Kind::Memo)
                        .then_some(source)
                };
                if let Some(source) = source {
                    self.update_if_necessary(source);
                }
                i += 1;
            }
        }
        let state = {
            let mut graph = self.graph.borrow_mut();
            let Some(node) = graph.nodes.get_mut(id) else {
                return;
            };
            if node.state == State::Check {
                node.state = State::Clean;
            }
            node.state
        };
        if state == State::Dirty {
            self.recompute(id);
        }
    }

    /// Re-runs the computation of `id`: disposes what its previous run created, re-tracks its
    /// sources, and marks its subscribers dirty if a memo's value changed.
    fn recompute(&self, id: Id) {
        self.check_cycle(id);
        // As a batch: effects triggered by the run (or by cleanups) wait until it is over.
        self.batch(|| self.run_computation(id));
    }

    /// The body of [`Inner::recompute`].
    fn run_computation(&self, id: Id) {
        self.dispose_owned(id);

        let (compute, previous_observer, previous_owner) = {
            let mut graph = self.graph.borrow_mut();
            let Some(node) = graph.nodes.get_mut(id) else {
                return;
            };
            let Some(compute) = node.compute.clone() else {
                return;
            };
            // Clean before the run, so writes during the run can mark it dirty again.
            node.state = State::Clean;
            node.running = true;
            graph.begin_tracking(id);
            let observer = graph.observer.replace(id);
            let owner = mem::replace(&mut graph.owner, id);
            (compute, observer, owner)
        };

        /// Restores the graph after the run, also during unwinding.
        struct Restore<'a> {
            inner: &'a Inner,
            id: Id,
            observer: Option<Id>,
            owner: Id,
        }
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                let mut graph = self.inner.graph.borrow_mut();
                graph.end_tracking(self.id);
                graph.observer = self.observer;
                graph.owner = self.owner;
                if let Some(node) = graph.nodes.get_mut(self.id) {
                    node.running = false;
                }
            }
        }
        let restore = Restore {
            inner: self,
            id,
            observer: previous_observer,
            owner: previous_owner,
        };
        let changed = match compute.try_borrow_mut() {
            Ok(mut f) => f(),
            Err(_) => panic!("cycle detected: a computation re-entered itself"),
        };
        drop(restore);

        let mut graph = self.graph.borrow_mut();
        if changed && graph.nodes.get(id).is_some_and(|n| n.kind == Kind::Memo) {
            graph.mark_subscribers(id, State::Dirty);
        }
    }

    /// Runs queued effects, unless a batch or computation is in progress or a flush is already
    /// running further up the stack.
    pub(crate) fn flush_if_idle(&self) {
        {
            let mut graph = self.graph.borrow_mut();
            if graph.depth > 0 || graph.flushing {
                return;
            }
            graph.flushing = true;
        }

        /// Ends the flush, also during unwinding.
        struct EndFlush<'a>(&'a Inner);
        impl Drop for EndFlush<'_> {
            fn drop(&mut self) {
                self.0.graph.borrow_mut().flushing = false;
            }
        }
        let _end = EndFlush(self);
        loop {
            let next = self.graph.borrow_mut().queue.pop_front();
            let Some(effect) = next else {
                break;
            };
            self.update_if_necessary(effect);
        }
    }

    /// Runs `f` as a batch: effects wait until the outermost batch ends.
    pub(crate) fn batch<R>(&self, f: impl FnOnce() -> R) -> R {
        /// Ends the batch, also during unwinding (without flushing then).
        struct EndBatch<'a>(&'a Inner);
        impl Drop for EndBatch<'_> {
            fn drop(&mut self) {
                self.0.graph.borrow_mut().depth -= 1;
                if !std::thread::panicking() {
                    self.0.flush_if_idle();
                }
            }
        }
        self.graph.borrow_mut().depth += 1;
        let _end = EndBatch(self);
        f()
    }

    /// Runs `f` with no running computation, so reads don't subscribe.
    pub(crate) fn untrack<R>(&self, f: impl FnOnce() -> R) -> R {
        /// Restores the observer, also during unwinding.
        struct Restore<'a>(&'a Inner, Option<Id>);
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                self.0.graph.borrow_mut().observer = self.1;
            }
        }
        let observer = self.graph.borrow_mut().observer.take();
        let _restore = Restore(self, observer);
        f()
    }

    /// Runs `f` with scope `id` as the current owner. Panics if the scope is disposed.
    pub(crate) fn run_in_scope<R>(&self, id: Id, f: impl FnOnce() -> R) -> R {
        /// Restores the owner, also during unwinding.
        struct Restore<'a>(&'a Inner, Id);
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                self.0.graph.borrow_mut().owner = self.1;
            }
        }
        let owner = {
            let mut graph = self.graph.borrow_mut();
            if !graph.nodes.get(id).is_some_and(|n| n.kind == Kind::Scope) {
                drop(graph);
                disposed("Scope::run: the scope");
            }
            mem::replace(&mut graph.owner, id)
        };
        let _restore = Restore(self, owner);
        f()
    }

    /// Registers a cleanup with the current owner (runs it now if the owner is gone).
    pub(crate) fn on_cleanup(&self, f: Box<dyn FnOnce()>) {
        let rejected = {
            let mut graph = self.graph.borrow_mut();
            let owner = graph.owner;
            match graph.nodes.get_mut(owner) {
                Some(node) => {
                    node.cleanups.push(f);
                    None
                }
                None => Some(f),
            }
        };
        if let Some(f) = rejected {
            f();
        }
    }

    /// Disposes `id` and everything it owns.
    pub(crate) fn dispose(&self, id: Id) {
        self.batch(|| {
            let mut order = Vec::new();
            {
                let mut graph = self.graph.borrow_mut();
                graph.subtree(id, &mut order);
                // Detach from the owner, which lives on.
                let owner = graph.nodes.get(id).and_then(|n| n.owner);
                if let Some(owner) = owner.and_then(|o| graph.nodes.get_mut(o)) {
                    owner.owned.retain(|&x| x != id);
                }
            }
            self.dispose_nodes(&order);
        });
    }

    /// Disposes what `id` owns and runs its cleanups; `id` itself stays (before a re-run).
    /// Called inside a batch.
    fn dispose_owned(&self, id: Id) {
        let order = {
            let mut graph = self.graph.borrow_mut();
            let mut order = Vec::new();
            graph.subtree(id, &mut order);
            // The last entry is `id` itself.
            order.pop();
            if let Some(node) = graph.nodes.get_mut(id) {
                node.owned.clear();
            }
            order
        };
        self.dispose_nodes(&order);
        self.run_cleanups(id);
    }

    /// Runs the cleanups of each node in `order`, then removes them all. Owned nodes come
    /// before their owner in `order`.
    fn dispose_nodes(&self, order: &[Id]) {
        for &id in order {
            self.run_cleanups(id);
        }
        let removed: Vec<Node> = {
            let mut graph = self.graph.borrow_mut();
            order.iter().filter_map(|&id| graph.unlink(id)).collect()
        };
        // Values and closures are dropped here, with no borrow of the graph held.
        drop(removed);
    }

    /// Runs and removes the cleanups registered on `id`, in registration order.
    fn run_cleanups(&self, id: Id) {
        let cleanups = match self.graph.borrow_mut().nodes.get_mut(id) {
            Some(node) => mem::take(&mut node.cleanups),
            None => return,
        };
        for f in cleanups {
            f();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{effect, signal};

    /// Subscriber lists hold each subscriber once, however often dependencies change.
    #[test]
    fn reactive_sig_14_subscriber_lists_do_not_grow() {
        let rt = Runtime::new();
        rt.enter(|| {
            let flip = signal(false);
            let a = signal(0);
            let b = signal(0);
            effect(move || {
                if flip.get() {
                    b.get();
                    a.get();
                    a.get();
                } else {
                    a.get();
                    b.get();
                    b.get();
                }
            });
            for i in 0..50 {
                flip.set(i % 2 == 0);
                a.set(i);
            }
        });
        let graph = rt.inner.graph.borrow();
        // flip, a and b have one subscriber each (the effect); the effect and root have none.
        let mut lens: Vec<usize> = graph
            .nodes
            .iter()
            .map(|(_, n)| n.subscribers.len())
            .collect();
        lens.sort_unstable();
        assert_eq!(lens, [0, 0, 1, 1, 1]);
    }
}
