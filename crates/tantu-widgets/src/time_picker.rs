//! [`TimePickerField`]: a time typed as `HH:MM` or stepped with Up and Down. Spec:
//! `docs/specs/widgets/time-picker.md`.

use std::rc::Rc;

use tantu_view::reactive::Signal;
use tantu_view::{BuildCx, ElementId, View};

use crate::time::Time;

/// A time typed as `HH:MM` or stepped with Up and Down (WIDGETS-TIME-02..04).
pub struct TimePickerField {
    selected: Signal<Option<Time>>,
    label: String,
    width: f32,
    minute_step: u32,
    on_change: Option<Rc<dyn Fn(Time)>>,
}

impl TimePickerField {
    /// A field choosing into `selected`.
    pub fn new(selected: Signal<Option<Time>>) -> Self {
        TimePickerField {
            selected,
            label: "Time".to_owned(),
            width: 120.0,
            minute_step: 1,
            on_change: None,
        }
    }

    /// The accessible name (default "Time").
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// The field's width (default 120 px).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// How far Up/Down move the minutes (default 1; clamped to 1..=60).
    pub fn minute_step(mut self, minutes: u32) -> Self {
        self.minute_step = minutes.clamp(1, 60);
        self
    }

    /// Called with the time after the user types or steps one that differs.
    pub fn on_change(mut self, f: impl Fn(Time) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl View for TimePickerField {
    fn build(self, _cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.selected,
            self.label,
            self.width,
            self.minute_step,
            self.on_change,
        );
        todo!("WIDGETS-TIME-02")
    }
}
