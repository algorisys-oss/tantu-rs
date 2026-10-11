//! [`Slider`]. Spec: `docs/specs/widgets/slider.md`.

use std::rc::Rc;

use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

/// Picks a number in a range by dragging a thumb (Flutter's `Slider`, Material 3 look).
///
/// The slider is controlled: it shows `value` and reports changes through `on_change`, which
/// the app usually writes back to the signal it passed in:
///
/// ```
/// use tantu_view::reactive::signal;
/// use tantu_widgets::Slider;
///
/// let volume = signal(0.5f64);
/// let slider = Slider::new(volume).on_change(move |v| volume.set(v));
/// # let _ = slider;
/// ```
pub struct Slider {
    value: Prop<f64>,
    min: Prop<f64>,
    max: Prop<f64>,
    divisions: u32,
    enabled: Prop<bool>,
    on_change: Option<Rc<dyn Fn(f64)>>,
    on_change_start: Option<Rc<dyn Fn(f64)>>,
    on_change_end: Option<Rc<dyn Fn(f64)>>,
}

impl Slider {
    /// A continuous slider showing `value` in the range 0 to 1, enabled, reporting nothing
    /// (and so drawn disabled until `on_change` is set).
    pub fn new(value: impl IntoProp<f64>) -> Self {
        Slider {
            value: value.into_prop(),
            min: Prop::Value(0.0),
            max: Prop::Value(1.0),
            divisions: 0,
            enabled: Prop::Value(true),
            on_change: None,
            on_change_start: None,
            on_change_end: None,
        }
    }

    /// The range (default 0 to 1).
    pub fn range(mut self, min: impl IntoProp<f64>, max: impl IntoProp<f64>) -> Self {
        self.min = min.into_prop();
        self.max = max.into_prop();
        self
    }

    /// A discrete slider with `divisions` equal steps (0, the default, is continuous).
    pub fn divisions(mut self, divisions: u32) -> Self {
        self.divisions = divisions;
        self
    }

    /// A disabled slider ignores input and draws dimmed. A slider with no `on_change` is
    /// disabled too, as in Flutter.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }

    /// Called with the new value while the user drags or presses a key.
    pub fn on_change(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// Called with the value before a drag starts.
    pub fn on_change_start(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_change_start = Some(Rc::new(f));
        self
    }

    /// Called with the value when a drag ends.
    pub fn on_change_end(mut self, f: impl Fn(f64) + 'static) -> Self {
        self.on_change_end = Some(Rc::new(f));
        self
    }
}

impl View for Slider {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.value,
            self.min,
            self.max,
            self.divisions,
            self.enabled,
            self.on_change,
            self.on_change_start,
            self.on_change_end,
        );
        todo!("WIDGETS-SLIDER-01")
    }
}
