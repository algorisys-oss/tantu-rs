//! [`ViewTree`], [`View`], [`AnyView`] and [`BuildCx`]: views that build elements once, and
//! the per-window tree of elements (ADR 0011). Spec: `docs/specs/view/tree.md`.

use std::any::Any;
use std::rc::Rc;

use tantu_core::{Arena, Size};
use tantu_layout::{BoxConstraints, LayoutId, LayoutTree, RenderBox, TextMeasure};
use tantu_reactive::{Runtime, Scope};
use tantu_scene::ElementId;

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
        let _ = view;
        todo!()
    }
}

impl View for AnyView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.0);
        todo!()
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
    tree: &'a mut ViewTree,
    parent: ElementId,
}

impl BuildCx<'_> {
    /// Creates a render element under the current parent, owning a new layout node with
    /// `render`, then builds `children` under it, in order, inside its scope.
    pub fn render(
        &mut self,
        render: impl RenderBox,
        children: impl IntoIterator<Item = AnyView>,
    ) -> ElementId {
        let _ = (render, children.into_iter().count(), &self.tree);
        todo!()
    }

    /// Creates a region element under the current parent, then runs `build` with the region as
    /// the parent (inside its scope) to build its children.
    pub fn region(&mut self, build: impl FnOnce(&mut BuildCx<'_>)) -> ElementId {
        let _ = build;
        todo!()
    }

    /// Sets the parent data of a render element's layout node (read by its layout parent, e.g.
    /// `FlexParentData` for `Expanded`). Returns false (and does nothing) for a region or an
    /// unknown id.
    pub fn set_parent_data<T: Any>(&mut self, element: ElementId, data: T) -> bool {
        let _ = (element, data);
        todo!()
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
struct Element {
    parent: Option<ElementId>,
    children: Vec<ElementId>,
    scope: Scope,
    kind: Kind,
}

/// The element kind, with a render element's layout node.
enum Kind {
    Render(LayoutId),
    Region,
}

/// The views, elements and layout of one window, with its reactive runtime.
pub struct ViewTree {
    elements: Arena<Element>,
    layout: LayoutTree,
    root: Option<ElementId>,
    runtime: Rc<Runtime>,
}

impl ViewTree {
    /// A tree whose content is the view returned by `app`. `app` runs once, with the tree's
    /// runtime current and inside the root element's scope, so it can create signals.
    pub fn new<V: View>(app: impl FnOnce() -> V) -> Self {
        let _ = app;
        todo!()
    }

    /// The implicit root render element; the app's view is built under it.
    pub fn root(&self) -> ElementId {
        todo!()
    }

    /// Number of elements, the root included.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True if the tree has no elements (never, while it exists: the root is always there).
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// True if `id` is an element of this tree.
    pub fn contains(&self, id: ElementId) -> bool {
        let _ = id;
        todo!()
    }

    /// The element's kind.
    pub fn kind(&self, id: ElementId) -> Option<ElementKind> {
        let _ = id;
        todo!()
    }

    /// The element's parent (`None` for the root or an unknown id).
    pub fn parent(&self, id: ElementId) -> Option<ElementId> {
        let _ = id;
        todo!()
    }

    /// The element's children, in order (empty for an unknown id).
    pub fn children(&self, id: ElementId) -> &[ElementId] {
        let _ = id;
        todo!()
    }

    /// A render element's layout node.
    pub fn layout_id(&self, id: ElementId) -> Option<LayoutId> {
        let _ = id;
        todo!()
    }

    /// The element's reactive scope.
    pub fn scope(&self, id: ElementId) -> Option<Scope> {
        let _ = id;
        todo!()
    }

    /// The layout tree (read-only; it changes only through the view tree).
    pub fn layout_tree(&self) -> &LayoutTree {
        &self.layout
    }

    /// Removes `id` and its descendants: disposes their scopes (effects stop, cleanups run),
    /// removes their layout nodes, and updates the parent. Returns the number of elements
    /// removed (0 for the root or an unknown id).
    pub fn remove(&mut self, id: ElementId) -> usize {
        let _ = id;
        todo!()
    }

    /// Runs a layout pass over the whole tree with `constraints` for the root (the window) and
    /// `text` as the text measurer; returns the root's size.
    pub fn layout(&mut self, constraints: BoxConstraints, text: &mut dyn TextMeasure) -> Size {
        let _ = (constraints, text);
        todo!()
    }

    /// Runs `f` with the tree's runtime current (to read or write signals from outside).
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _ = (f, &self.elements, &self.root, &self.runtime);
        todo!()
    }
}
