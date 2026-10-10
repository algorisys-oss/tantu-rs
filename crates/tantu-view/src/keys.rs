//! [`Key`] and [`Keyed`]: identities for finding elements. Spec:
//! `docs/specs/test/widget-tester.md` (VIEW-KEY-01).

use std::sync::Arc;

use tantu_scene::ElementId;

use crate::{BuildCx, View, ViewTree};

/// An identity for an element, for finding it (tests, inspector).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(pub Arc<str>);

impl From<&str> for Key {
    fn from(key: &str) -> Self {
        Key(Arc::from(key))
    }
}

impl From<String> for Key {
    fn from(key: String) -> Self {
        Key(Arc::from(key))
    }
}

/// Gives the element `child` builds the key `key`. A region child (`Dyn`, ...) is keyed as the
/// region element.
pub struct Keyed<V> {
    key: Key,
    child: V,
}

impl<V: View> Keyed<V> {
    /// Keys `child` with `key`.
    pub fn new(key: impl Into<Key>, child: V) -> Self {
        Keyed {
            key: key.into(),
            child,
        }
    }
}

impl<V: View> View for Keyed<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.key, self.child, cx);
        todo!()
    }
}

impl ViewTree {
    /// The elements with `key`, in tree order (removed elements drop their key).
    pub fn find_key(&self, key: &Key) -> Vec<ElementId> {
        let _ = key;
        todo!()
    }
}
