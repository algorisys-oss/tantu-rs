//! Dynamic content (ADR 0011, point 7): [`Dyn`], [`Show`] and [`For`], region elements
//! rebuilt at frames. Spec: `docs/specs/view/dynamic.md`.

use std::hash::Hash;
use std::marker::PhantomData;

use tantu_scene::ElementId;

use crate::{AnyView, BuildCx, View};

/// Content rebuilt from `f` whenever the signals `f` reads change.
pub struct Dyn<V> {
    f: Box<dyn Fn() -> V>,
}

impl<V: View> Dyn<V> {
    /// Content from `f`.
    pub fn new(f: impl Fn() -> V + 'static) -> Self {
        let _ = f;
        todo!()
    }
}

impl<V: View> View for Dyn<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.f);
        todo!()
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
        let _ = (when, then);
        todo!()
    }

    /// Shows `fallback()` while the condition is false.
    pub fn fallback<V: View>(self, fallback: impl Fn() -> V + 'static) -> Self {
        let _ = (fallback, &self.when, &self.then, &self.fallback);
        todo!()
    }
}

impl View for Show {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = cx;
        todo!()
    }
}

/// One child per item of `each`, matched to existing children by `key`.
pub struct For<T, K, V> {
    each: Box<dyn Fn() -> Vec<T>>,
    key: Box<dyn Fn(&T) -> K>,
    view: Box<dyn Fn(T) -> V>,
    _marker: PhantomData<fn() -> K>,
}

impl<T: 'static, K: Eq + Hash + Clone + 'static, V: View> For<T, K, V> {
    /// A list built from `each`, keyed by `key`, with `view` for each new key.
    pub fn new(
        each: impl Fn() -> Vec<T> + 'static,
        key: impl Fn(&T) -> K + 'static,
        view: impl Fn(T) -> V + 'static,
    ) -> Self {
        let _ = (each, key, view);
        todo!()
    }
}

impl<T: 'static, K: Eq + Hash + Clone + 'static, V: View> View for For<T, K, V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.each, self.key, self.view);
        todo!()
    }
}
