//! Dynamic content (ADR 0011, point 7): [`Dyn`], [`Show`] and [`For`], region elements
//! rebuilt at frames. Spec: `docs/specs/view/dynamic.md`.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

use tantu_reactive::{Scope, effect, memo, untrack};
use tantu_scene::ElementId;

use crate::{AnyView, BuildCx, View, ViewTree};

/// A version of a `Dyn`'s content: the scope owning what its closure created, and its view.
type Version<V> = Option<(Scope, V)>;

/// Content rebuilt from `f` whenever the signals `f` reads change.
pub struct Dyn<V> {
    f: Box<dyn Fn() -> V>,
}

impl<V: View> Dyn<V> {
    /// Content from `f`.
    pub fn new(f: impl Fn() -> V + 'static) -> Self {
        Dyn { f: Box::new(f) }
    }
}

impl<V: View> View for Dyn<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let region = cx.region(|_| {});
        let Some(region_scope) = cx.tree.scope(region) else {
            return region;
        };
        let f = self.f;
        let initial: Rc<RefCell<Version<V>>> = Rc::default();
        let pending: Rc<RefCell<Version<V>>> = Rc::default();
        let current: Rc<Cell<Option<Scope>>> = Rc::default();
        let first = Cell::new(true);
        let shared = Rc::clone(&cx.tree.shared);
        {
            let (initial, pending, current) = (
                Rc::clone(&initial),
                Rc::clone(&pending),
                Rc::clone(&current),
            );
            region_scope.run(|| {
                effect(move || {
                    // Owned by the region, not the effect, so it lives until it is replaced
                    // at a frame (VIEW-DYN-02); the effect still tracks what `f` reads.
                    let content = region_scope.run(Scope::new);
                    let view = content.run(&f);
                    if first.replace(false) {
                        *initial.borrow_mut() = Some((content, view));
                        return;
                    }
                    let replaced = pending.borrow_mut().replace((content, view));
                    match replaced {
                        Some((never_built, _)) => never_built.dispose(),
                        None => {
                            let (pending, current) = (Rc::clone(&pending), Rc::clone(&current));
                            shared.push(Box::new(move |tree: &mut ViewTree| {
                                let Some((content, view)) = pending.borrow_mut().take() else {
                                    return false;
                                };
                                if !tree.contains(region) {
                                    content.dispose();
                                    return false;
                                }
                                build_version(tree, region, content, view, &current);
                                true
                            }));
                        }
                    }
                });
            });
        }
        let first_version = initial.borrow_mut().take();
        if let Some((content, view)) = first_version {
            build_version(cx.tree, region, content, view, &current);
        }
        region
    }
}

/// Replaces `region`'s content: removes the old elements (their bindings stop), disposes the
/// old version's scope, then builds `view` inside `content`.
fn build_version<V: View>(
    tree: &mut ViewTree,
    region: ElementId,
    content: Scope,
    view: V,
    current: &Cell<Option<Scope>>,
) {
    for child in tree.children(region).to_vec() {
        tree.remove(child);
    }
    if let Some(old) = current.replace(Some(content)) {
        old.dispose();
    }
    content.run(|| {
        view.build(&mut BuildCx {
            tree: &mut *tree,
            parent: region,
        });
    });
    tree.sync_layout_children(region);
}

/// Builds nothing (an empty region).
struct Nothing;

impl View for Nothing {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}

/// `then` while `when` is true, else `fallback` (or nothing).
pub struct Show {
    when: Box<dyn Fn() -> bool>,
    then: Box<dyn Fn() -> AnyView>,
    fallback: Option<Box<dyn Fn() -> AnyView>>,
}

impl Show {
    /// Shows `then()` while `when()` is true.
    pub fn new<V: View>(when: impl Fn() -> bool + 'static, then: impl Fn() -> V + 'static) -> Self {
        Show {
            when: Box::new(when),
            then: Box::new(move || AnyView::new(then())),
            fallback: None,
        }
    }

    /// Shows `fallback()` while the condition is false.
    pub fn fallback<V: View>(self, fallback: impl Fn() -> V + 'static) -> Self {
        Show {
            fallback: Some(Box::new(move || AnyView::new(fallback()))),
            ..self
        }
    }
}

impl View for Show {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let when = self.when;
        // A memo, so only a change of the condition's value re-runs the content (VIEW-DYN-04).
        let condition = memo(when);
        let (then, fallback) = (self.then, self.fallback);
        Dyn::new(move || {
            if condition.get() {
                untrack(&then)
            } else {
                match &fallback {
                    Some(fallback) => untrack(fallback),
                    None => AnyView::new(Nothing),
                }
            }
        })
        .build(cx)
    }
}

/// One child per item of `each`, matched to existing children by `key`.
pub struct For<T, K, V> {
    each: Box<dyn Fn() -> Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    view: Rc<dyn Fn(T) -> V>,
}

impl<T: 'static, K: Eq + Hash + Clone + 'static, V: View> For<T, K, V> {
    /// A list built from `each`, keyed by `key`, with `view` for each new key.
    pub fn new(
        each: impl Fn() -> Vec<T> + 'static,
        key: impl Fn(&T) -> K + 'static,
        view: impl Fn(T) -> V + 'static,
    ) -> Self {
        For {
            each: Box::new(each),
            key: Rc::new(key),
            view: Rc::new(view),
        }
    }
}

/// The items a `For` has built: key, element, and the scope owning what `view` created.
type Built<K> = Vec<(K, ElementId, Scope)>;

impl<T: 'static, K: Eq + Hash + Clone + 'static, V: View> View for For<T, K, V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let region = cx.region(|_| {});
        let Some(region_scope) = cx.tree.scope(region) else {
            return region;
        };
        let (each, key, view) = (self.each, self.key, self.view);
        let built: Rc<RefCell<Built<K>>> = Rc::default();
        let initial: Rc<RefCell<Option<Vec<T>>>> = Rc::default();
        let pending: Rc<RefCell<Option<Vec<T>>>> = Rc::default();
        let first = Cell::new(true);
        let shared = Rc::clone(&cx.tree.shared);
        {
            let (initial, pending, built) =
                (Rc::clone(&initial), Rc::clone(&pending), Rc::clone(&built));
            let (key, view) = (Rc::clone(&key), Rc::clone(&view));
            region_scope.run(|| {
                effect(move || {
                    let items = each();
                    if first.replace(false) {
                        *initial.borrow_mut() = Some(items);
                        return;
                    }
                    let was_empty = pending.borrow_mut().replace(items).is_none();
                    if was_empty {
                        let (pending, built) = (Rc::clone(&pending), Rc::clone(&built));
                        let (key, view) = (Rc::clone(&key), Rc::clone(&view));
                        shared.push(Box::new(move |tree: &mut ViewTree| {
                            let Some(items) = pending.borrow_mut().take() else {
                                return false;
                            };
                            if !tree.contains(region) {
                                return false;
                            }
                            reconcile(tree, region, region_scope, items, &built, &*key, &*view);
                            true
                        }));
                    }
                });
            });
        }
        let first_items = initial.borrow_mut().take();
        if let Some(items) = first_items {
            reconcile(cx.tree, region, region_scope, items, &built, &*key, &*view);
        }
        region
    }
}

/// A position in the new list: an item kept from before, or a new item still to build.
type Slot<K, T> = Result<(K, ElementId, Scope), (K, T)>;

/// Brings `region`'s children in step with `items` by key (VIEW-DYN-06, -07).
fn reconcile<T, K: Eq + Hash + Clone, V: View>(
    tree: &mut ViewTree,
    region: ElementId,
    region_scope: Scope,
    items: Vec<T>,
    built: &RefCell<Built<K>>,
    key: &dyn Fn(&T) -> K,
    view: &dyn Fn(T) -> V,
) {
    let mut old: HashMap<K, (ElementId, Scope)> = built
        .borrow_mut()
        .drain(..)
        .map(|(k, element, scope)| (k, (element, scope)))
        .collect();
    let mut seen = HashSet::new();
    // In the new order: kept items, or new items still to build.
    let mut slots: Vec<Slot<K, T>> = Vec::with_capacity(items.len());
    for item in items {
        let k = key(&item);
        if !seen.insert(k.clone()) {
            continue; // duplicate key: the first one wins
        }
        match old.remove(&k) {
            Some((element, scope)) => slots.push(Ok((k, element, scope))),
            None => slots.push(Err((k, item))),
        }
    }
    for (element, scope) in old.into_values() {
        tree.remove(element);
        scope.dispose();
    }
    let mut order = Vec::with_capacity(slots.len());
    let mut now_built = Vec::with_capacity(slots.len());
    for slot in slots {
        let (k, element, scope) = match slot {
            Ok(kept) => kept,
            Err((k, item)) => {
                let scope = region_scope.run(Scope::new);
                let element = scope.run(|| {
                    let v = untrack(|| view(item));
                    v.build(&mut BuildCx {
                        tree: &mut *tree,
                        parent: region,
                    })
                });
                (k, element, scope)
            }
        };
        order.push(element);
        now_built.push((k, element, scope));
    }
    tree.set_child_order(region, order);
    *built.borrow_mut() = now_built;
}
