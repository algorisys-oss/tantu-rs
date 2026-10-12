//! Element semantics for assistive technology (spec `docs/specs/a11y/semantics.md`,
//! VIEW-SEM-01..03): what an element is, its name, value and state, set by widgets and read
//! by `tantu-a11y`, which builds AccessKit trees from them.

use std::rc::Rc;

use tantu_core::{Point, Rect, Size, Vec2};

use crate::events::ActionHandler;
use crate::{
    BuildCx, ElementId, ElementKind, ElementMut, KeyEvent, LogicalKey, Modifiers, NamedKey,
    PointerButton, PointerEvent, PointerKind, ViewTree,
};

/// What an element is, for assistive technology (a backend-neutral subset of AccessKit's
/// roles).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    /// The window (the tree's root).
    Window,
    /// A container with no meaning of its own.
    #[default]
    Group,
    /// Static text.
    Label,
    /// A pressable control.
    Button,
    /// A checkable box.
    CheckBox,
    /// One choice of a group.
    RadioButton,
    /// An on/off switch.
    Switch,
    /// A numeric range control.
    Slider,
    /// A single-line text input.
    TextInput,
    /// A multi-line text input.
    MultilineTextInput,
    /// A picture or icon.
    Image,
    /// A list of items.
    List,
    /// One item of a list.
    ListItem,
    /// A scrolling container.
    ScrollView,
    /// A tooltip.
    Tooltip,
    /// A table of cells (WIDGETS-GRID-07).
    Grid,
    /// A column's header cell in a grid.
    ColumnHeader,
    /// A cell in a grid.
    Cell,
    /// A hierarchical tree.
    Tree,
    /// A tree row.
    TreeItem,
}

/// A checkable element's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggled {
    /// Off.
    False,
    /// On.
    True,
    /// Partly on (an indeterminate checkbox).
    Mixed,
}

/// A numeric value with its range and step, for sliders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericValue {
    /// The current value.
    pub value: f64,
    /// The smallest value.
    pub min: f64,
    /// The largest value.
    pub max: f64,
    /// One increment or decrement.
    pub step: f64,
}

/// The actions an element supports, as assistive technology may request them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SemanticsActions {
    /// Expanding a branch.
    pub expand: bool,
    /// Collapsing a branch.
    pub collapse: bool,
    /// A press (a button, a toggle).
    pub click: bool,
    /// Taking keyboard focus.
    pub focus: bool,
    /// One step up (a slider).
    pub increment: bool,
    /// One step down.
    pub decrement: bool,
    /// Replacing the value (a text field).
    pub set_value: bool,
    /// Scrolling into view (a child of a scroll view).
    pub scroll_into_view: bool,
}

/// What assistive technology is told about an element (VIEW-SEM-01).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantics {
    /// Open or closed; None for leaves.
    pub expanded: Option<bool>,
    /// One-based tree depth.
    pub level: Option<usize>,
    /// One-based sibling position.
    pub position_in_set: Option<usize>,
    /// Declared sibling count.
    pub size_of_set: Option<usize>,
    /// Total displayed body rows, excluding headers.
    pub row_count: Option<usize>,
    /// Total displayed columns.
    pub column_count: Option<usize>,
    /// Zero-based body row index.
    pub row_index: Option<usize>,
    /// Zero-based display column index.
    pub column_index: Option<usize>,
    /// A column header's sort direction.
    pub sort_direction: Option<SortDirection>,
    /// Whether multiple items may be selected.
    pub multiselectable: bool,
    /// A composite widget's current semantic descendant.
    pub active_descendant: Option<ElementId>,
    /// What the element is.
    pub role: Role,
    /// The accessible name (a label, a button's text).
    pub name: Option<String>,
    /// The current value as text (a field's text, a slider's number).
    pub value: Option<String>,
    /// A numeric value with its range and step, for sliders.
    pub numeric: Option<NumericValue>,
    /// A checkable element's state.
    pub toggled: Option<Toggled>,
    /// Whether a selectable element is selected.
    pub selected: Option<bool>,
    /// The element ignores input.
    pub disabled: bool,
    /// The element takes keyboard focus.
    pub focusable: bool,
    /// The supported actions.
    pub actions: SemanticsActions,
}

/// A column header's sorting direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

/// What assistive technology asked for (VIEW-SEM-03).
#[derive(Clone, Debug, PartialEq)]
pub enum SemanticsAction {
    /// Expand a branch.
    Expand,
    /// Collapse a branch.
    Collapse,
    /// Give the element keyboard focus.
    Focus,
    /// Press it.
    Click,
    /// One step up.
    Increment,
    /// One step down.
    Decrement,
    /// Replace its value.
    SetValue(String),
    /// Scroll it into view.
    ScrollIntoView,
}

/// What an action handler sees: the tree (to read bounds), its own element, the element the
/// request named, and the action (VIEW-SEM-03).
pub struct ActionCx<'a> {
    /// The tree, for bounds and structure.
    pub tree: &'a ViewTree,
    /// The element the handler was registered on.
    pub element: ElementId,
    /// The element the request named (the handler's element or a descendant).
    pub target: ElementId,
    /// The action.
    pub action: &'a SemanticsAction,
}

impl BuildCx<'_> {
    /// Sets the element's semantics (a render element only; false for a region)
    /// (VIEW-SEM-01).
    pub fn set_semantics(&mut self, element: ElementId, semantics: Semantics) -> bool {
        self.tree.set_semantics(element, semantics)
    }

    /// Handles `SetValue` and `ScrollIntoView` requests for `element` and its descendants;
    /// true when handled (VIEW-SEM-03).
    pub fn on_action(
        &mut self,
        element: ElementId,
        handler: impl Fn(&ActionCx<'_>) -> bool + 'static,
    ) {
        if self.tree.contains(element) {
            self.tree
                .events
                .actions
                .entry(element)
                .or_default()
                .push(Rc::new(handler));
        }
    }
}

impl ElementMut<'_> {
    /// Changes the semantics in place (for reactive props); false without semantics
    /// (VIEW-SEM-01).
    pub fn update_semantics(&mut self, f: impl FnOnce(&mut Semantics)) -> bool {
        match self
            .tree
            .element_mut(self.id)
            .and_then(|e| e.semantics.as_deref_mut())
        {
            Some(semantics) => {
                f(semantics);
                true
            }
            None => false,
        }
    }
}

impl ViewTree {
    /// Stores `semantics` on a render element (VIEW-SEM-01).
    pub(crate) fn set_semantics(&mut self, element: ElementId, semantics: Semantics) -> bool {
        if self.kind(element) != Some(ElementKind::Render) {
            return false;
        }
        match self.element_mut(element) {
            Some(e) => {
                e.semantics = Some(Box::new(semantics));
                true
            }
            None => false,
        }
    }

    /// The element's semantics, if set (VIEW-SEM-01).
    pub fn semantics(&self, element: ElementId) -> Option<&Semantics> {
        self.element(element)?.semantics.as_deref()
    }

    /// The element's bounds in window coordinates after the last layout (a region: the
    /// union of its children; `None` for an unknown element) (VIEW-SEM-02).
    pub fn bounds(&self, element: ElementId) -> Option<Rect> {
        match self.kind(element)? {
            ElementKind::Region => Some(
                self.children(element)
                    .iter()
                    .filter_map(|child| self.bounds(*child))
                    .reduce(Rect::union)
                    .unwrap_or(Rect::ZERO),
            ),
            ElementKind::Render => {
                let layout = self.layout_tree();
                let size = self
                    .layout_id(element)
                    .and_then(|id| layout.size(id))
                    .unwrap_or(Size::ZERO);
                let mut origin = Vec2::ZERO;
                let mut next = Some(element);
                while let Some(id) = next {
                    if let Some(offset) = self.layout_id(id).and_then(|l| layout.offset(l)) {
                        origin += offset;
                    }
                    next = self.parent(id);
                }
                Some(Rect::from_origin_size(Point::new(origin.x, origin.y), size))
            }
        }
    }

    /// Performs `action` on `element` as a user would (VIEW-SEM-03); false if nothing
    /// happened.
    pub fn perform_action(&mut self, element: ElementId, action: SemanticsAction) -> bool {
        let Some(semantics) = self.semantics(element) else {
            return false;
        };
        let (role, actions) = (semantics.role, semantics.actions);
        let supported = match &action {
            SemanticsAction::Expand | SemanticsAction::Collapse => false,
            SemanticsAction::Focus => actions.focus,
            SemanticsAction::Click => actions.click,
            SemanticsAction::Increment => actions.increment,
            SemanticsAction::Decrement => actions.decrement,
            SemanticsAction::SetValue(_) => actions.set_value,
            // Any node may be scrolled into view; the handlers decide.
            SemanticsAction::ScrollIntoView => true,
        };
        if !supported {
            return false;
        }
        match action {
            SemanticsAction::Expand | SemanticsAction::Collapse => false,
            SemanticsAction::Focus => self.focus(element),
            SemanticsAction::Click => {
                let Some(center) = self.bounds(element).map(|b| b.center()) else {
                    return false;
                };
                let down = self.dispatch_pointer(PointerEvent {
                    kind: PointerKind::Down(PointerButton::Primary),
                    position: center,
                    modifiers: Default::default(),
                });
                let up = self.dispatch_pointer(PointerEvent {
                    kind: PointerKind::Up(PointerButton::Primary),
                    position: center,
                    modifiers: Default::default(),
                });
                down || up
            }
            SemanticsAction::Increment | SemanticsAction::Decrement => {
                let forward = action == SemanticsAction::Increment;
                let key = match (role, forward) {
                    (Role::Slider, true) => NamedKey::ArrowRight,
                    (Role::Slider, false) => NamedKey::ArrowLeft,
                    (_, true) => NamedKey::ArrowDown,
                    (_, false) => NamedKey::ArrowUp,
                };
                if actions.focus {
                    self.focus(element);
                }
                self.dispatch_key(KeyEvent {
                    key: LogicalKey::Named(key),
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::default(),
                    text: None,
                })
            }
            // Every enclosing container reveals (VIEW-REVEAL-03).
            SemanticsAction::ScrollIntoView => self.ensure_visible(element),
            SemanticsAction::SetValue(_) => {
                // Handlers on the element, then its ancestors; the first `true` wins.
                let mut next = Some(element);
                while let Some(id) = next {
                    let handlers: Vec<ActionHandler> =
                        self.events.actions.get(&id).cloned().unwrap_or_default();
                    let handled = self.enter(|| {
                        handlers.iter().any(|handler| {
                            handler(&ActionCx {
                                tree: self,
                                element: id,
                                target: element,
                                action: &action,
                            })
                        })
                    });
                    if handled {
                        return true;
                    }
                    next = self.parent(id);
                }
                false
            }
        }
    }
}
