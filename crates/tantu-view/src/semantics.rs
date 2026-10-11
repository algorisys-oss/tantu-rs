//! Element semantics for assistive technology (spec `docs/specs/a11y/semantics.md`,
//! VIEW-SEM-01..03): what an element is, its name, value and state, set by widgets and read
//! by `tantu-a11y`, which builds AccessKit trees from them.

use tantu_core::Rect;

use crate::{BuildCx, ElementId, ElementMut, ViewTree};

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

/// What assistive technology asked for (VIEW-SEM-03).
#[derive(Clone, Debug, PartialEq)]
pub enum SemanticsAction {
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

impl BuildCx<'_> {
    /// Sets the element's semantics (a render element only; false for a region)
    /// (VIEW-SEM-01).
    pub fn set_semantics(&mut self, element: ElementId, semantics: Semantics) -> bool {
        let _ = (element, semantics);
        todo!()
    }
}

impl BuildCx<'_> {
    /// Handles `SetValue` and `ScrollIntoView` requests for `element` and its descendants
    /// (`target` is the element the request named); true when handled (VIEW-SEM-03).
    pub fn on_action(
        &mut self,
        element: ElementId,
        handler: impl Fn(ElementId, &SemanticsAction) -> bool + 'static,
    ) {
        let _ = (
            element,
            Box::new(handler) as Box<dyn Fn(ElementId, &SemanticsAction) -> bool>,
        );
        todo!()
    }
}

impl ElementMut<'_> {
    /// Changes the semantics in place (for reactive props); false without semantics
    /// (VIEW-SEM-01).
    pub fn update_semantics(&mut self, f: impl FnOnce(&mut Semantics)) -> bool {
        let _ = f;
        todo!()
    }
}

impl ViewTree {
    /// The element's semantics, if set (VIEW-SEM-01).
    pub fn semantics(&self, element: ElementId) -> Option<&Semantics> {
        let _ = element;
        todo!()
    }

    /// The element's bounds in window coordinates after the last layout (a region: the
    /// union of its children; `None` for an unknown element) (VIEW-SEM-02).
    pub fn bounds(&self, element: ElementId) -> Option<Rect> {
        let _ = element;
        todo!()
    }

    /// Performs `action` on `element` as a user would (VIEW-SEM-03); false if nothing
    /// happened.
    pub fn perform_action(&mut self, element: ElementId, action: SemanticsAction) -> bool {
        let _ = (element, action);
        todo!()
    }
}
