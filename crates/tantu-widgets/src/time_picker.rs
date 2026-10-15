//! [`TimePickerField`]: a time typed as `HH:MM` or stepped with Up and Down. Spec:
//! `docs/specs/widgets/time-picker.md`.

use std::rc::Rc;

use tantu_view::reactive::{Signal, effect, signal, untrack};
use tantu_view::{BuildCx, ElementId, Handled, LogicalKey, Modifiers, NamedKey, Phase, Role, View};

use crate::formatter::MaskFormatter;
use crate::text_editing::{EditingValue, TextSelection};
use crate::text_field::FieldDecoration;
use crate::time::Time;
use crate::{SizedBox, TextEditingController, TextField};

/// The field's error description (WIDGETS-TIME-02).
const INVALID: &str = "Invalid time";
/// The colon's offset: a caret at or before it steps hours (WIDGETS-TIME-03).
const HOURS_END: usize = 2;

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

/// `time` moved one step: an hour when `hours`, else to the next or previous multiple of
/// `step` minutes (WIDGETS-TIME-03).
fn step(time: Time, hours: bool, up: bool, step: u32) -> Time {
    if hours {
        return time.add_minutes(if up { 60 } else { -60 });
    }
    let step = i64::from(step);
    let minutes = time.minutes_of_day();
    let target = if up {
        (minutes / step + 1) * step
    } else if minutes % step != 0 {
        minutes - minutes % step
    } else {
        minutes - step
    };
    // A day's minutes fit an `i32`.
    time.add_minutes((target - minutes) as i32)
}

fn text(time: Option<Time>) -> String {
    time.map(|t| t.to_string()).unwrap_or_default()
}

impl View for TimePickerField {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let selected = self.selected;
        let on_change = self.on_change;
        let minute_step = self.minute_step;
        let controller = TextEditingController::new(text(untrack(|| selected.get())));
        let (invalid, focused) = (signal(false), signal(false));
        // The text follows the selection while the field doesn't have the focus.
        effect(move || {
            let time = selected.get();
            if !untrack(|| focused.get()) {
                let text = text(time);
                untrack(|| {
                    if controller.text() != text {
                        controller.set_text(text);
                    }
                });
            }
        });
        let mask = MaskFormatter::new("##:##");
        let report = {
            let on_change = on_change.clone();
            move |time: Time| {
                if let Some(f) = &on_change {
                    f(time);
                }
            }
        };
        let field = TextField::new(controller)
            .placeholder("HH:MM")
            .input_formatter(mask.clone())
            .decoration(FieldDecoration {
                error: Rc::new(move || invalid.get().then(|| INVALID.to_owned())),
                role: Some(Role::SpinButton),
                ..FieldDecoration::named(Some(self.label))
            })
            .on_change({
                let (mask, report) = (mask.clone(), report.clone());
                move |typed| {
                    if !mask.is_complete(&typed) {
                        invalid.set(false);
                        return;
                    }
                    let time = Time::parse(&typed);
                    invalid.set(time.is_none());
                    if let Some(time) = time {
                        if selected.get_untracked() != Some(time) {
                            selected.set(Some(time));
                            report(time);
                        }
                    }
                }
            });
        let id = SizedBox::width(self.width).child(field).build(cx);
        // Up and Down step the segment under the caret (WIDGETS-TIME-03).
        cx.on_key(id, Phase::Capture, move |k| {
            let event = k.event;
            if !event.pressed || event.modifiers != Modifiers::default() {
                return Handled::Continue;
            }
            let up = match &event.key {
                LogicalKey::Named(NamedKey::ArrowUp) => true,
                LogicalKey::Named(NamedKey::ArrowDown) => false,
                _ => return Handled::Continue,
            };
            let value = controller.value();
            let typed = mask.is_complete(&value.text) && Time::parse(&value.text).is_some();
            let current = if typed {
                selected.get_untracked()
            } else {
                None
            };
            let caret = value.selection.extent;
            let next = match current {
                Some(time) => step(time, caret <= HOURS_END, up, minute_step),
                None => Time::from_minutes(0),
            };
            selected.set(Some(next));
            invalid.set(false);
            let shown = next.to_string();
            let caret = caret.min(shown.len());
            controller.set_value(
                EditingValue::new(shown).with_selection(TextSelection::collapsed(caret)),
            );
            report(next);
            Handled::Stop
        });
        cx.on_focus_within(id, move |inside| {
            focused.set(inside);
            if !inside {
                let restored = text(selected.get_untracked());
                if controller.text() != restored {
                    controller.set_text(restored);
                }
                if invalid.get_untracked() {
                    invalid.set(false);
                }
            }
        });
        id
    }
}
