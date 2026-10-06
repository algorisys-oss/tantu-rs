//! Conversions from winit's types to Tantu's (spec rules PLATFORM-WINIT-05..12).

use tantu_core::Point;
use tantu_platform::{
    ButtonState, Key, KeyEvent, KeyLocation, Modifiers, NamedKey, PhysicalSize, PointerButton,
    ScrollDelta, WindowAttributes, WindowEvent,
};
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::ModifiersState;

/// A winit logical key as a Tantu key.
pub(crate) fn key(key: &winit::keyboard::Key) -> Key {
    todo!()
}

/// A winit key location as a Tantu one.
pub(crate) fn location(location: winit::keyboard::KeyLocation) -> KeyLocation {
    todo!()
}

/// A key press or release, from the parts of a winit `KeyEvent` (its constructor is private).
pub(crate) fn key_event(
    logical_key: &winit::keyboard::Key,
    key_location: winit::keyboard::KeyLocation,
    state: ElementState,
    repeat: bool,
    text: Option<&str>,
) -> KeyEvent {
    todo!()
}

/// Held modifiers.
pub(crate) fn modifiers(state: ModifiersState) -> Modifiers {
    todo!()
}

/// A mouse button.
pub(crate) fn button(button: MouseButton) -> PointerButton {
    todo!()
}

/// Pressed or released.
pub(crate) fn button_state(state: ElementState) -> ButtonState {
    todo!()
}

/// A wheel delta in Tantu's convention (positive `y` scrolls down), pixels made logical.
pub(crate) fn scroll(delta: MouseScrollDelta, scale_factor: f32) -> ScrollDelta {
    todo!()
}

/// A physical position as a logical point.
pub(crate) fn position(position: winit::dpi::PhysicalPosition<f64>, scale_factor: f32) -> Point {
    todo!()
}

/// A physical size.
pub(crate) fn size(size: winit::dpi::PhysicalSize<u32>) -> PhysicalSize {
    todo!()
}

/// winit window attributes for Tantu's.
pub(crate) fn attributes(attributes: &WindowAttributes) -> winit::window::WindowAttributes {
    todo!()
}

/// A winit window event as a Tantu one, or `None` for events Tantu doesn't handle yet.
/// `pointer` is the window's last cursor position: updated by cursor moves, used for button and
/// wheel events.
pub(crate) fn window_event(
    event: winit::event::WindowEvent,
    scale_factor: f32,
    pointer: &mut Point,
) -> Option<WindowEvent> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantu_core::{Size, Vec2};
    use tantu_platform::PointerId;
    use winit::dpi::{LogicalSize, PhysicalPosition, Size as WinitSize};
    use winit::event::{DeviceId, TouchPhase, WindowEvent as W};
    use winit::keyboard::{Key as WKey, KeyLocation as WLoc, NamedKey as WNamed, SmolStr};

    const NAMED: [(WNamed, NamedKey); 48] = [
        (WNamed::Enter, NamedKey::Enter),
        (WNamed::Tab, NamedKey::Tab),
        (WNamed::Backspace, NamedKey::Backspace),
        (WNamed::Delete, NamedKey::Delete),
        (WNamed::Escape, NamedKey::Escape),
        (WNamed::Insert, NamedKey::Insert),
        (WNamed::ArrowLeft, NamedKey::ArrowLeft),
        (WNamed::ArrowRight, NamedKey::ArrowRight),
        (WNamed::ArrowUp, NamedKey::ArrowUp),
        (WNamed::ArrowDown, NamedKey::ArrowDown),
        (WNamed::Home, NamedKey::Home),
        (WNamed::End, NamedKey::End),
        (WNamed::PageUp, NamedKey::PageUp),
        (WNamed::PageDown, NamedKey::PageDown),
        (WNamed::Shift, NamedKey::Shift),
        (WNamed::Control, NamedKey::Control),
        (WNamed::Alt, NamedKey::Alt),
        (WNamed::Super, NamedKey::Super),
        (WNamed::CapsLock, NamedKey::CapsLock),
        (WNamed::ContextMenu, NamedKey::ContextMenu),
        (WNamed::PrintScreen, NamedKey::PrintScreen),
        (WNamed::Pause, NamedKey::Pause),
        (WNamed::NumLock, NamedKey::NumLock),
        (WNamed::ScrollLock, NamedKey::ScrollLock),
        (WNamed::F1, NamedKey::F1),
        (WNamed::F2, NamedKey::F2),
        (WNamed::F3, NamedKey::F3),
        (WNamed::F4, NamedKey::F4),
        (WNamed::F5, NamedKey::F5),
        (WNamed::F6, NamedKey::F6),
        (WNamed::F7, NamedKey::F7),
        (WNamed::F8, NamedKey::F8),
        (WNamed::F9, NamedKey::F9),
        (WNamed::F10, NamedKey::F10),
        (WNamed::F11, NamedKey::F11),
        (WNamed::F12, NamedKey::F12),
        (WNamed::F13, NamedKey::F13),
        (WNamed::F14, NamedKey::F14),
        (WNamed::F15, NamedKey::F15),
        (WNamed::F16, NamedKey::F16),
        (WNamed::F17, NamedKey::F17),
        (WNamed::F18, NamedKey::F18),
        (WNamed::F19, NamedKey::F19),
        (WNamed::F20, NamedKey::F20),
        (WNamed::F21, NamedKey::F21),
        (WNamed::F22, NamedKey::F22),
        (WNamed::F23, NamedKey::F23),
        (WNamed::F24, NamedKey::F24),
    ];

    #[test]
    fn platform_winit_05_keys() {
        for (winit_key, tantu_key) in NAMED {
            assert_eq!(
                key(&WKey::Named(winit_key)),
                Key::Named(tantu_key),
                "{winit_key:?}"
            );
        }
        assert_eq!(
            key(&WKey::Named(WNamed::Space)),
            Key::Character(" ".to_owned())
        );
        assert_eq!(key(&WKey::Named(WNamed::F35)), Key::Unidentified);
        assert_eq!(key(&WKey::Named(WNamed::MediaPlay)), Key::Unidentified);
        assert_eq!(key(&WKey::Dead(Some('`'))), Key::Unidentified);
        assert_eq!(
            key(&WKey::Unidentified(
                winit::keyboard::NativeKey::Unidentified
            )),
            Key::Unidentified
        );
        assert_eq!(
            key(&WKey::Character(SmolStr::new("é"))),
            Key::Character("é".to_owned())
        );
    }

    #[test]
    fn platform_winit_06_locations_and_key_events() {
        assert_eq!(location(WLoc::Standard), KeyLocation::Standard);
        assert_eq!(location(WLoc::Left), KeyLocation::Left);
        assert_eq!(location(WLoc::Right), KeyLocation::Right);
        assert_eq!(location(WLoc::Numpad), KeyLocation::Numpad);
        assert_eq!(
            key_event(
                &WKey::Character(SmolStr::new("a")),
                WLoc::Standard,
                ElementState::Pressed,
                true,
                Some("a")
            ),
            KeyEvent {
                key: Key::Character("a".to_owned()),
                location: KeyLocation::Standard,
                state: ButtonState::Pressed,
                repeat: true,
                text: Some("a".to_owned()),
            }
        );
        assert_eq!(
            key_event(
                &WKey::Named(WNamed::Shift),
                WLoc::Right,
                ElementState::Released,
                false,
                None
            ),
            KeyEvent {
                key: Key::Named(NamedKey::Shift),
                location: KeyLocation::Right,
                state: ButtonState::Released,
                repeat: false,
                text: None,
            }
        );
    }

    #[test]
    fn platform_winit_07_modifiers() {
        assert_eq!(modifiers(ModifiersState::empty()), Modifiers::NONE);
        assert_eq!(
            modifiers(ModifiersState::SHIFT | ModifiersState::SUPER),
            Modifiers {
                shift: true,
                super_key: true,
                ..Modifiers::NONE
            }
        );
        assert_eq!(
            modifiers(ModifiersState::CONTROL | ModifiersState::ALT),
            Modifiers {
                control: true,
                alt: true,
                ..Modifiers::NONE
            }
        );
    }

    #[test]
    fn platform_winit_08_mouse_buttons() {
        assert_eq!(button(MouseButton::Left), PointerButton::Primary);
        assert_eq!(button(MouseButton::Right), PointerButton::Secondary);
        assert_eq!(button(MouseButton::Middle), PointerButton::Middle);
        assert_eq!(button(MouseButton::Back), PointerButton::Back);
        assert_eq!(button(MouseButton::Forward), PointerButton::Forward);
        assert_eq!(button(MouseButton::Other(7)), PointerButton::Other(7));
        assert_eq!(button_state(ElementState::Pressed), ButtonState::Pressed);
        assert_eq!(button_state(ElementState::Released), ButtonState::Released);
    }

    #[test]
    fn platform_winit_09_positions_and_pointer_events() {
        assert_eq!(
            position(PhysicalPosition::new(300.0, 150.0), 1.5),
            Point::new(200.0, 100.0)
        );
        let device_id = DeviceId::dummy();
        let mut last = Point::ZERO;

        // Before any move, buttons happen at the origin.
        let press = W::MouseInput {
            device_id,
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };
        assert_eq!(
            window_event(press.clone(), 2.0, &mut last),
            Some(WindowEvent::PointerButton {
                pointer: PointerId::Mouse,
                button: PointerButton::Primary,
                state: ButtonState::Pressed,
                position: Point::ZERO,
            })
        );
        let moved = W::CursorMoved {
            device_id,
            position: PhysicalPosition::new(40.0, 20.0),
        };
        assert_eq!(
            window_event(moved, 2.0, &mut last),
            Some(WindowEvent::PointerMoved {
                pointer: PointerId::Mouse,
                position: Point::new(20.0, 10.0)
            })
        );
        assert_eq!(last, Point::new(20.0, 10.0));
        assert_eq!(
            window_event(press, 2.0, &mut last),
            Some(WindowEvent::PointerButton {
                pointer: PointerId::Mouse,
                button: PointerButton::Primary,
                state: ButtonState::Pressed,
                position: Point::new(20.0, 10.0),
            })
        );
        let wheel = W::MouseWheel {
            device_id,
            delta: MouseScrollDelta::LineDelta(0.0, 1.0),
            phase: TouchPhase::Moved,
        };
        assert_eq!(
            window_event(wheel, 2.0, &mut last),
            Some(WindowEvent::Wheel {
                pointer: PointerId::Mouse,
                delta: ScrollDelta::Lines { x: -0.0, y: -1.0 },
                position: Point::new(20.0, 10.0),
            })
        );
        assert_eq!(
            window_event(W::CursorEntered { device_id }, 2.0, &mut last),
            Some(WindowEvent::PointerEntered {
                pointer: PointerId::Mouse
            })
        );
        assert_eq!(
            window_event(W::CursorLeft { device_id }, 2.0, &mut last),
            Some(WindowEvent::PointerLeft {
                pointer: PointerId::Mouse
            })
        );
    }

    #[test]
    fn platform_winit_10_wheel_deltas() {
        assert_eq!(
            scroll(MouseScrollDelta::LineDelta(2.0, -3.0), 1.0),
            ScrollDelta::Lines { x: -2.0, y: 3.0 }
        );
        assert_eq!(
            scroll(
                MouseScrollDelta::PixelDelta(PhysicalPosition::new(10.0, -40.0)),
                2.0
            ),
            ScrollDelta::Pixels(Vec2::new(-5.0, 20.0))
        );
    }

    #[test]
    fn platform_winit_11_window_events() {
        let mut last = Point::ZERO;
        let mut convert = |e: W| window_event(e, 1.0, &mut last);
        assert_eq!(
            convert(W::CloseRequested),
            Some(WindowEvent::CloseRequested)
        );
        assert_eq!(
            convert(W::Resized(winit::dpi::PhysicalSize::new(640, 480))),
            Some(WindowEvent::Resized(PhysicalSize::new(640, 480)))
        );
        assert_eq!(
            convert(W::RedrawRequested),
            Some(WindowEvent::RedrawRequested)
        );
        assert_eq!(convert(W::Focused(true)), Some(WindowEvent::Focused(true)));
        assert_eq!(
            convert(W::ModifiersChanged(ModifiersState::ALT.into())),
            Some(WindowEvent::ModifiersChanged(Modifiers {
                alt: true,
                ..Modifiers::NONE
            }))
        );
        assert_eq!(convert(W::Destroyed), None);
        assert_eq!(convert(W::Occluded(true)), None);
        assert_eq!(convert(W::HoveredFileCancelled), None);
        assert_eq!(
            size(winit::dpi::PhysicalSize::new(3, 4)),
            PhysicalSize::new(3, 4)
        );
    }

    #[test]
    fn platform_winit_12_window_attributes() {
        let a = attributes(
            &WindowAttributes::new("Editor")
                .size(300.0, 200.0)
                .min_size(100.0, 50.0)
                .resizable(false)
                .visible(false),
        );
        assert_eq!(a.title, "Editor");
        assert_eq!(
            a.inner_size,
            Some(WinitSize::Logical(LogicalSize::new(300.0, 200.0)))
        );
        assert_eq!(
            a.min_inner_size,
            Some(WinitSize::Logical(LogicalSize::new(100.0, 50.0)))
        );
        assert!(!a.resizable);
        assert!(!a.visible);
        let b = attributes(&WindowAttributes::default());
        assert_eq!(b.min_inner_size, None);
        assert!(b.resizable && b.visible);
        let _ = Size::ZERO;
    }
}
