//! [`LayoutBuilder`]: content built from the constraints its parent gives it. Spec:
//! `docs/specs/view/layout-builder.md`.

use std::cell::Cell;
use std::rc::Rc;

use tantu_core::{Size, Vec2};
use tantu_layout::{BoxConstraints, LayoutChildren, RenderBox};
use tantu_reactive::{Signal, signal};
use tantu_scene::ElementId;

use crate::{AnyView, BuildCx, Dyn, View, ViewTree};

/// Content built from the constraints its parent gives it.
pub struct LayoutBuilder {
    builder: Box<dyn Fn(BoxConstraints) -> AnyView>,
}

impl LayoutBuilder {
    /// Content from `builder`, called with the element's constraints.
    pub fn new<V: View>(builder: impl Fn(BoxConstraints) -> V + 'static) -> Self {
        LayoutBuilder {
            builder: Box::new(move |c| AnyView::new(builder(c))),
        }
    }
}

/// Lays out the content with its own constraints and records them (VIEW-LB-01).
struct RenderLayoutBuilder {
    seen: Rc<Cell<Option<BoxConstraints>>>,
}

impl RenderBox for RenderLayoutBuilder {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        self.seen.set(Some(constraints));
        if children.is_empty() {
            return constraints.smallest();
        }
        let mut largest = Size::ZERO;
        for i in 0..children.len() {
            largest = largest.max(children.layout(i, constraints));
            children.set_offset(i, Vec2::ZERO);
        }
        constraints.constrain(largest)
    }
}

/// Builds nothing (an empty region), before the first constraints are known.
struct Nothing;

impl View for Nothing {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}

/// What the view tree tracks for one `LayoutBuilder`.
pub(crate) struct LayoutBuilderRecord {
    element: ElementId,
    /// The constraints of the element's latest layout.
    seen: Rc<Cell<Option<BoxConstraints>>>,
    /// The constraints the content follows (read by its `Dyn`).
    constraints: Signal<Option<BoxConstraints>>,
    /// The bits of the constraints last written to `constraints`.
    last: Cell<Option<[u32; 4]>>,
}

impl View for LayoutBuilder {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let seen = Rc::new(Cell::new(None));
        let id = cx.render(
            RenderLayoutBuilder {
                seen: Rc::clone(&seen),
            },
            [],
        );
        let Some(scope) = cx.tree.scope(id) else {
            return id;
        };
        let constraints = scope.run(|| signal(None::<BoxConstraints>));
        let builder = self.builder;
        scope.run(|| {
            Dyn::new(move || match constraints.get() {
                Some(c) => builder(c),
                None => AnyView::new(Nothing),
            })
            .build(&mut BuildCx {
                tree: &mut *cx.tree,
                parent: id,
            });
        });
        cx.tree.sync_layout_children(id);
        cx.tree.layout_builders.push(LayoutBuilderRecord {
            element: id,
            seen,
            constraints,
            last: Cell::new(None),
        });
        id
    }
}

/// The bits of `c`, so NaN compares equal to itself (VIEW-LB-03).
fn bits(c: BoxConstraints) -> [u32; 4] {
    [
        c.min_width.to_bits(),
        c.max_width.to_bits(),
        c.min_height.to_bits(),
        c.max_height.to_bits(),
    ]
}

impl ViewTree {
    /// True if some builder's latest constraints differ from the ones its content follows
    /// (read-only: unlike `sync_layout_builders`, it runs no builder).
    pub(crate) fn layout_builders_pending(&self) -> bool {
        self.layout_builders.iter().any(|record| {
            self.contains(record.element)
                && record
                    .seen
                    .get()
                    .is_some_and(|c| record.last.get() != Some(bits(c)))
        })
    }

    /// Writes changed constraints into the builders' signals (their `Dyn`s queue rebuilds).
    /// Returns whether any changed. Records of removed elements are dropped.
    pub(crate) fn sync_layout_builders(&mut self) -> bool {
        let records = std::mem::take(&mut self.layout_builders);
        let mut kept = Vec::with_capacity(records.len());
        let mut changed = false;
        for record in records {
            if !self.contains(record.element) {
                continue;
            }
            if let Some(c) = record.seen.get() {
                let b = bits(c);
                if record.last.get() != Some(b) {
                    record.last.set(Some(b));
                    let signal = record.constraints;
                    self.enter(|| signal.set(Some(c)));
                    changed = true;
                }
            }
            kept.push(record);
        }
        kept.append(&mut self.layout_builders);
        self.layout_builders = kept;
        changed
    }
}
