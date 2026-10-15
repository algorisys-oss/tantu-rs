//! [`ComboBox`]: a text field with a filtered list of options. Spec:
//! `docs/specs/widgets/combo-box.md`.

use std::rc::Rc;

use tantu_view::reactive::Signal;
use tantu_view::{BuildCx, ElementId, View};

/// A text field with a filtered list of options (Flutter's `DropdownMenu` with filtering;
/// WIDGETS-COMBO-01..07).
pub struct ComboBox {
    selected: Signal<Option<usize>>,
    options: Rc<[String]>,
    width: f32,
    placeholder: Option<String>,
    label: Option<String>,
    on_change: Option<Rc<dyn Fn(usize)>>,
}

impl ComboBox {
    /// Shows `options`; `selected` holds the chosen index (`None`: nothing chosen).
    pub fn new(
        selected: Signal<Option<usize>>,
        options: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        ComboBox {
            selected,
            options: options.into_iter().map(Into::into).collect(),
            width: 240.0,
            placeholder: None,
            label: None,
            on_change: None,
        }
    }

    /// The field's width and the popup's (default 240 px).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Shown in the empty field, and its accessible name.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// The label naming the field for assistive technology (default: the placeholder).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Called with the chosen index after the user picks an option.
    pub fn on_change(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl View for ComboBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.selected,
            self.options,
            self.width,
            self.placeholder,
            self.label,
            self.on_change,
        );
        todo!("WIDGETS-COMBO-01..07")
    }
}
