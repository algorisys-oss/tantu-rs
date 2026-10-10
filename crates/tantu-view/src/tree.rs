//! [`ViewTree`], [`View`], [`AnyView`] and [`BuildCx`]: views that build elements once, and
//! the per-window tree of elements (ADR 0011). Spec: `docs/specs/view/tree.md`.

use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use tantu_core::{Affine, Arena, Id, Rect, Size, Vec2};
use tantu_layout::{
    BoxConstraints, LayoutChildren, LayoutId, LayoutTree, RenderBox, TextMeasure, TextStyleKey,
};
use tantu_reactive::{Runtime, Scope};
use tantu_scene::{ElementId, SceneBuilder};
use tantu_text::{TextStyle, TextStyles};

use crate::events::EventState;
use crate::frame::Shared;
use crate::keys::Key;
use crate::layout_builder::LayoutBuilderRecord;
use crate::paint::Children;
use crate::{NoPaint, Paint, PaintCx, TextPainter};

/// A description of part of the UI that builds its element(s) once, when consumed.
pub trait View: 'static {
    /// Builds this view's element under `cx`'s current parent and returns it.
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId;
}

/// A type-erased view, for child lists.
pub struct AnyView(Box<dyn FnOnce(&mut BuildCx<'_>) -> ElementId>);

impl AnyView {
    /// Erases `view`'s type.
    pub fn new(view: impl View) -> Self {
        AnyView(Box::new(move |cx: &mut BuildCx<'_>| view.build(cx)))
    }
}

impl View for AnyView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        (self.0)(cx)
    }
}

/// What kind of element an id names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementKind {
    /// Owns a layout node (and, later, a paint behavior).
    Render,
    /// Owns no layout node; its children take its place in the nearest render ancestor's
    /// layout child list.
    Region,
}

/// The context a view builds in: the tree and the current parent element.
pub struct BuildCx<'a> {
    pub(crate) tree: &'a mut ViewTree,
    pub(crate) parent: ElementId,
}

impl BuildCx<'_> {
    /// Creates a render element under the current parent, owning a new layout node with
    /// `render`, then builds `children` under it, in order, inside its scope.
    pub fn render(
        &mut self,
        render: impl RenderBox,
        children: impl IntoIterator<Item = AnyView>,
    ) -> ElementId {
        let layout = self.tree.layout.insert(render);
        let id = self
            .tree
            .add_element(self.parent, Kind::Render(layout, Box::new(NoPaint)));
        let Some(scope) = self.tree.scope(id) else {
            return id;
        };
        scope.run(|| {
            let mut cx = BuildCx {
                tree: &mut *self.tree,
                parent: id,
            };
            for child in children {
                child.build(&mut cx);
            }
        });
        self.tree.sync_layout_children(id);
        id
    }

    /// Creates a region element under the current parent, then runs `build` with the region as
    /// the parent (inside its scope) to build its children.
    pub fn region(&mut self, build: impl FnOnce(&mut BuildCx<'_>)) -> ElementId {
        let id = self.tree.add_element(self.parent, Kind::Region);
        let Some(scope) = self.tree.scope(id) else {
            return id;
        };
        scope.run(|| {
            build(&mut BuildCx {
                tree: &mut *self.tree,
                parent: id,
            })
        });
        id
    }

    /// Sets the parent data of a render element's layout node (read by its layout parent, e.g.
    /// `FlexParentData` for `Expanded`). Returns false (and does nothing) for a region or an
    /// unknown id.
    pub fn set_parent_data<T: Any>(&mut self, element: ElementId, data: T) -> bool {
        match self.tree.layout_id(element) {
            Some(layout) => self.tree.layout.set_parent_data(layout, Some(data)),
            None => false,
        }
    }

    /// Gives a render element its paint behavior (replacing the previous one). Returns false
    /// for a region or an unknown id.
    pub fn set_paint(&mut self, element: ElementId, paint: impl Paint) -> bool {
        let Some(e) = arena_id(element).and_then(|a| self.tree.elements.get_mut(a)) else {
            return false;
        };
        match &mut e.kind {
            Kind::Render(_, slot) => {
                *slot = Box::new(paint);
                true
            }
            Kind::Region => false,
        }
    }

    /// The key for `style` in the tree's style table.
    pub fn text_style(&mut self, style: TextStyle) -> TextStyleKey {
        self.tree.text_styles.key(style)
    }

    /// The current parent element.
    pub fn parent(&self) -> ElementId {
        self.parent
    }

    /// The tree, read-only.
    pub fn tree(&self) -> &ViewTree {
        self.tree
    }
}

/// One element.
pub(crate) struct Element {
    parent: Option<ElementId>,
    children: Vec<ElementId>,
    scope: Scope,
    pub(crate) kind: Kind,
}

/// The element kind, with a render element's layout node and paint behavior.
pub(crate) enum Kind {
    Render(LayoutId, Box<dyn Paint>),
    Region,
}

/// The root element's layout: each child laid out with the window's constraints at the
/// origin; as large as the window when it is bounded (VIEW-TREE-08).
struct RootBox;

impl RenderBox for RootBox {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let mut largest = Size::ZERO;
        for i in 0..children.len() {
            largest = largest.max(children.layout(i, constraints));
            children.set_offset(i, Vec2::ZERO);
        }
        let biggest = constraints.biggest();
        if biggest.is_finite() {
            biggest
        } else {
            constraints.constrain(largest)
        }
    }
}

/// How many rebuild rounds one `ViewTree::layout` call allows for `LayoutBuilder`s.
const LAYOUT_BUILDER_ROUNDS: usize = 16;

/// The arena id behind an element id.
pub(crate) fn arena_id(id: ElementId) -> Option<Id> {
    Id::from_bits(id.to_raw())
}

/// The views, elements and layout of one window, with its reactive runtime.
pub struct ViewTree {
    pub(crate) elements: Arena<Element>,
    pub(crate) layout: LayoutTree,
    root: Option<ElementId>,
    pub(crate) runtime: Rc<Runtime>,
    pub(crate) shared: Rc<Shared>,
    pub(crate) layout_builders: Vec<LayoutBuilderRecord>,
    pub(crate) events: EventState,
    text_styles: TextStyles,
    pub(crate) keys: HashMap<ElementId, Key>,
}

impl ViewTree {
    /// A tree whose content is the view returned by `app`. `app` runs once, with the tree's
    /// runtime current and inside the root element's scope, so it can create signals.
    pub fn new<V: View>(app: impl FnOnce() -> V) -> Self {
        ViewTree::with_text_styles(TextStyles::new(), app)
    }

    /// Like [`new`](Self::new), with `styles` as the tree's style table (pass the text
    /// system's, so keys built here mean the same styles when measured and painted).
    pub fn with_text_styles<V: View>(styles: TextStyles, app: impl FnOnce() -> V) -> Self {
        let mut tree = ViewTree {
            elements: Arena::new(),
            layout: LayoutTree::new(),
            root: None,
            runtime: Rc::new(Runtime::new()),
            shared: Rc::default(),
            layout_builders: Vec::new(),
            events: EventState::default(),
            text_styles: styles,
            keys: HashMap::new(),
        };
        let runtime = tree.runtime.clone();
        runtime.enter(|| {
            let scope = Scope::new();
            let layout = tree.layout.insert(RootBox);
            let root = ElementId::from(tree.elements.insert(Element {
                parent: None,
                children: Vec::new(),
                scope,
                kind: Kind::Render(layout, Box::new(NoPaint)),
            }));
            tree.root = Some(root);
            scope.run(|| {
                let view = app();
                view.build(&mut BuildCx {
                    tree: &mut tree,
                    parent: root,
                });
            });
            tree.sync_layout_children(root);
        });
        tree
    }

    /// The tree's style table ([`new`](Self::new) creates an empty one of its own).
    pub fn text_styles(&self) -> &TextStyles {
        &self.text_styles
    }

    /// The implicit root render element; the app's view is built under it.
    pub fn root(&self) -> ElementId {
        self.root
            .expect("ViewTree::new sets the root before returning")
    }

    /// Number of elements, the root included.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// True if the tree has no elements (never, while it exists: the root is always there).
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// True if `id` is an element of this tree.
    pub fn contains(&self, id: ElementId) -> bool {
        self.element(id).is_some()
    }

    /// The element's kind.
    pub fn kind(&self, id: ElementId) -> Option<ElementKind> {
        self.element(id).map(|e| match e.kind {
            Kind::Render(..) => ElementKind::Render,
            Kind::Region => ElementKind::Region,
        })
    }

    /// The element's parent (`None` for the root or an unknown id).
    pub fn parent(&self, id: ElementId) -> Option<ElementId> {
        self.element(id)?.parent
    }

    /// The element's children, in order (empty for an unknown id).
    pub fn children(&self, id: ElementId) -> &[ElementId] {
        self.element(id).map_or(&[][..], |e| e.children.as_slice())
    }

    /// A render element's layout node.
    pub fn layout_id(&self, id: ElementId) -> Option<LayoutId> {
        match self.element(id)?.kind {
            Kind::Render(layout, _) => Some(layout),
            Kind::Region => None,
        }
    }

    /// The element's reactive scope.
    pub fn scope(&self, id: ElementId) -> Option<Scope> {
        self.element(id).map(|e| e.scope)
    }

    /// The layout tree (read-only; it changes only through the view tree).
    pub fn layout_tree(&self) -> &LayoutTree {
        &self.layout
    }

    /// Removes `id` and its descendants: disposes their scopes (effects stop, cleanups run),
    /// removes their layout nodes, and updates the parent. Returns the number of elements
    /// removed (0 for the root or an unknown id).
    pub fn remove(&mut self, id: ElementId) -> usize {
        let Some(element) = self.element(id) else {
            return 0;
        };
        let Some(parent) = element.parent else {
            return 0; // the root
        };
        let scope = element.scope;
        // The scope owns the descendants' scopes, so disposing it stops them all.
        let runtime = self.runtime.clone();
        runtime.enter(|| scope.dispose());
        self.remove_layout_nodes(id);
        let mut removed = 0;
        let mut stack = vec![id];
        while let Some(next) = stack.pop() {
            if let Some(e) = arena_id(next).and_then(|a| self.elements.remove(a)) {
                self.events.forget(next);
                self.keys.remove(&next);
                stack.extend(e.children);
                removed += 1;
            }
        }
        if let Some(p) = arena_id(parent).and_then(|a| self.elements.get_mut(a)) {
            p.children.retain(|c| *c != id);
        }
        self.sync_layout_children(parent);
        removed
    }

    /// Runs a layout pass over the whole tree with `constraints` for the root (the window) and
    /// `text` as the text measurer; returns the root's size.
    pub fn layout(&mut self, constraints: BoxConstraints, text: &mut dyn TextMeasure) -> Size {
        let mut size = self.layout_pass(constraints, &mut *text);
        // LayoutBuilders whose constraints changed rebuild, then lay out again (VIEW-LB-02, -05).
        for _ in 0..LAYOUT_BUILDER_ROUNDS {
            if !self.sync_layout_builders() {
                return size;
            }
            self.apply_internal_updates();
            size = self.layout_pass(constraints, &mut *text);
        }
        if self.layout_builders_pending() {
            tracing::warn!(
                "LayoutBuilder content still changing after {LAYOUT_BUILDER_ROUNDS} rounds; \
                 keeping the last round's content"
            );
        }
        size
    }

    /// One layout pass from the root.
    fn layout_pass(&mut self, constraints: BoxConstraints, text: &mut dyn TextMeasure) -> Size {
        match self.layout_id(self.root()) {
            Some(root) => self.layout.with_text(text).layout(root, constraints),
            None => Size::ZERO,
        }
    }

    /// Paints the whole tree into `scene` (an open builder for the window's Scene), using the
    /// geometry of the last layout pass.
    pub fn paint(&self, scene: &mut SceneBuilder<'_>, text: &mut dyn TextPainter) {
        self.paint_element(self.root(), scene, text);
    }

    /// Paints one element (VIEW-PAINT-02..05); regions paint their children in their place.
    pub(crate) fn paint_element(
        &self,
        id: ElementId,
        scene: &mut SceneBuilder<'_>,
        text: &mut dyn TextPainter,
    ) {
        let Some(element) = self.element(id) else {
            return;
        };
        let (layout, paint) = match &element.kind {
            Kind::Render(layout, paint) => (*layout, paint),
            Kind::Region => {
                for child in &element.children {
                    self.paint_element(*child, scene, &mut *text);
                }
                return;
            }
        };
        let (Some(size), Some(offset)) = (self.layout.size(layout), self.layout.offset(layout))
        else {
            return; // never laid out
        };
        let margin = paint.overflow().max(0.0);
        let bounds = Rect::from_ltwh(
            offset.x - margin,
            offset.y - margin,
            size.width + 2.0 * margin,
            size.height + 2.0 * margin,
        );
        if (element.children.is_empty() || paint.clips_children()) && scene.is_culled(bounds) {
            return;
        }
        scene.push_transform(Affine::translate(offset));
        scene.set_element(Some(id));
        let mut cx = PaintCx {
            tree: self,
            element: id,
            size,
            scene: &mut *scene,
            text,
            children: Children::Pending,
        };
        paint.paint(&mut cx);
        if cx.children == Children::Pending {
            cx.paint_children();
        }
        scene.pop();
    }

    /// Runs `f` with the tree's runtime current (to read or write signals from outside).
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        self.runtime.enter(f)
    }

    /// The element's kind, with its layout node and paint.
    pub(crate) fn element_kind(&self, id: ElementId) -> Option<&Kind> {
        self.element(id).map(|e| &e.kind)
    }

    fn element(&self, id: ElementId) -> Option<&Element> {
        self.elements.get(arena_id(id)?)
    }

    /// Adds an element of `kind` as the last child of `parent`, with a scope owned by the
    /// parent's scope.
    fn add_element(&mut self, parent: ElementId, kind: Kind) -> ElementId {
        let scope = match self.scope(parent) {
            Some(parent_scope) => parent_scope.run(Scope::new),
            None => Scope::new(),
        };
        let id = ElementId::from(self.elements.insert(Element {
            parent: Some(parent),
            children: Vec::new(),
            scope,
            kind,
        }));
        if let Some(p) = arena_id(parent).and_then(|a| self.elements.get_mut(a)) {
            p.children.push(id);
        }
        id
    }

    /// Replaces `id`'s child list with `order` (the same children, reordered) and updates the
    /// layout children.
    pub(crate) fn set_child_order(&mut self, id: ElementId, order: Vec<ElementId>) {
        if let Some(e) = arena_id(id).and_then(|a| self.elements.get_mut(a)) {
            e.children = order;
        }
        self.sync_layout_children(id);
    }

    /// The nearest render element at or above `id`.
    fn render_ancestor(&self, mut id: ElementId) -> Option<ElementId> {
        loop {
            let element = self.element(id)?;
            match element.kind {
                Kind::Render(..) => return Some(id),
                Kind::Region => id = element.parent?,
            }
        }
    }

    /// Recomputes the layout children of the render element at or above `id` (VIEW-TREE-04).
    pub(crate) fn sync_layout_children(&mut self, id: ElementId) {
        let Some(owner) = self.render_ancestor(id) else {
            return;
        };
        let Some(layout) = self.layout_id(owner) else {
            return;
        };
        let mut flat = Vec::new();
        self.collect_layout_children(owner, &mut flat);
        if let Err(error) = self.layout.set_children(layout, &flat) {
            tracing::error!(?error, "view tree out of step with its layout tree");
        }
    }

    /// The layout nodes of `id`'s children, with regions replaced by their own children.
    fn collect_layout_children(&self, id: ElementId, out: &mut Vec<LayoutId>) {
        for child in self.children(id) {
            match self.element(*child).map(|e| &e.kind) {
                Some(Kind::Render(layout, _)) => out.push(*layout),
                Some(Kind::Region) => self.collect_layout_children(*child, out),
                None => {}
            }
        }
    }

    /// Removes the layout nodes owned by `id` and its descendants: a render element's node
    /// takes its render descendants' nodes with it; a region's render children are removed
    /// one by one.
    fn remove_layout_nodes(&mut self, id: ElementId) {
        match self.element(id).map(|e| &e.kind) {
            Some(Kind::Render(layout, _)) => {
                let layout = *layout;
                self.layout.remove(layout);
            }
            Some(Kind::Region) => {
                let children = self.children(id).to_vec();
                for child in children {
                    self.remove_layout_nodes(child);
                }
            }
            None => {}
        }
    }
}

impl Drop for ViewTree {
    fn drop(&mut self) {
        if let Some(scope) = self.root.and_then(|root| self.scope(root)) {
            self.runtime.enter(|| scope.dispose());
        }
    }
}
