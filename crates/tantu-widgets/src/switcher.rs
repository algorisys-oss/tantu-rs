//! [`AnimatedSwitcher`]: cross-fades between children keyed by a value. Spec:
//! `docs/specs/widgets/transitions.md` (WIDGETS-SWITCHER-01..03).

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use tantu_view::reactive::{Scope, Signal, effect, signal, untrack};
use tantu_view::{
    AnimationController, AnimationStatus, AnyView, BuildCx, Curve, ElementId, For, IntoProp, Prop,
    Tween, View,
};

use crate::{Opacity, Stack};

/// Shows the child built for the current key, cross-fading when the key changes (Flutter's
/// `AnimatedSwitcher`). With a `bool` key and an empty view for `false`, an animated `Show`.
pub struct AnimatedSwitcher<K> {
    key: Prop<K>,
    build: Rc<dyn Fn(K) -> AnyView>,
    duration: Duration,
    curve: Curve,
}

impl<K: Clone + PartialEq + 'static> AnimatedSwitcher<K> {
    /// `build(key)` for each new key; default 300 ms, `EaseInOut`.
    pub fn new(key: impl IntoProp<K>, build: impl Fn(K) -> AnyView + 'static) -> Self {
        AnimatedSwitcher {
            key: key.into_prop(),
            build: Rc::new(build),
            duration: Duration::from_millis(300),
            curve: Curve::EaseInOut,
        }
    }

    /// How long a cross-fade takes (default 300 ms).
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// The fades' curve (default `EaseInOut`).
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
}

/// One child of the switcher: its key, its fade and whether it is on its way out.
#[derive(Clone)]
struct Entry<K> {
    /// Unique, so a key that comes back while fading out keeps its element.
    id: u64,
    key: K,
    fade: AnimationController,
    leaving: Rc<Cell<bool>>,
}

impl<K: Clone + PartialEq + 'static> View for AnimatedSwitcher<K> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (key, build, duration, curve) = (self.key, self.build, self.duration, self.curve);
        cx.region(move |cx| {
            // Everything long-lived (controllers, watchers) is owned by this scope, which the
            // region's scope owns, so it all goes when the switcher is removed
            // (WIDGETS-SWITCHER-03).
            let owner = Scope::new();
            let next_id = Rc::new(Cell::new(0u64));
            let entries: Signal<Vec<Entry<K>>> = signal(Vec::new());
            let add = {
                let next_id = Rc::clone(&next_id);
                move |key: K, shown: bool| {
                    let entry = owner.run(|| {
                        let fade = AnimationController::new(duration);
                        if shown {
                            fade.set_value(1.0);
                        } else {
                            fade.forward();
                        }
                        let entry = Entry {
                            id: next_id.replace(next_id.get() + 1),
                            key,
                            fade,
                            leaving: Rc::new(Cell::new(false)),
                        };
                        // Removed once its fade out ends (WIDGETS-SWITCHER-01).
                        let (id, leaving) = (entry.id, Rc::clone(&entry.leaving));
                        effect(move || {
                            if fade.status() == AnimationStatus::Dismissed && leaving.get() {
                                entries.update(|list| list.retain(|e| e.id != id));
                            }
                        });
                        entry
                    });
                    entries.update(|list| list.push(entry));
                }
            };
            let first = untrack(|| key.get());
            add(first.clone(), true);
            let last = RefCell::new(first);
            owner.run(|| {
                effect(move || {
                    let key = key.get();
                    if *last.borrow() == key {
                        return;
                    }
                    *last.borrow_mut() = key.clone();
                    let list = untrack(|| entries.get());
                    let mut found = false;
                    for entry in &list {
                        if entry.key == key {
                            // Coming back: fade in from where it is (WIDGETS-SWITCHER-02).
                            found = true;
                            entry.leaving.set(false);
                            entry.fade.forward();
                        } else if !entry.leaving.get() {
                            entry.leaving.set(true);
                            entry.fade.reverse();
                        }
                    }
                    if !found {
                        add(key, false);
                    }
                });
            });
            let children = For::new(
                move || entries.get(),
                |entry: &Entry<K>| entry.id,
                move |entry: Entry<K>| {
                    Opacity::new(
                        entry.fade.drive(Tween::new(0.0f32, 1.0)).curve(curve),
                        build(entry.key),
                    )
                },
            );
            Stack::new().child(children).build(cx);
        })
    }
}
