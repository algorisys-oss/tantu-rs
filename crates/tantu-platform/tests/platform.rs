//! Tests for `docs/specs/platform/platform.md`: PLATFORM-TYPES-NN and PLATFORM-FAKE-NN.

use std::cell::{Cell, RefCell};
use std::error::Error;
use std::rc::Rc;
use std::time::{Duration, Instant};

use tantu_core::{Point, Size, Vec2};
use tantu_platform::accesskit::{
    Action, ActionRequest, Node, NodeId, Role, Tree, TreeId, TreeUpdate,
};
use tantu_platform::{
    AccessibilityEvent, ButtonState, CursorIcon, FakePlatform, FakeStep, ImeEvent, Key, KeyEvent,
    KeyLocation, Modifiers, NamedKey, PhysicalSize, Platform, PlatformContext, PlatformError,
    PlatformHandler, PointerButton, PointerId, ScrollDelta, WindowAttributes, WindowEvent,
    WindowId,
};

// ---- Value types -------------------------------------------------------------------------

#[test]
fn platform_types_01_window_id_round_trip() {
    for raw in [0, 1, 42, u64::MAX] {
        assert_eq!(WindowId::from_raw(raw).to_raw(), raw);
    }
}

#[test]
fn platform_types_02_physical_logical_conversion() {
    let size = PhysicalSize::new(1600, 900);
    assert_eq!(size.to_logical(2.0), Size::new(800.0, 450.0));
    assert_eq!(size.to_logical(1.0), Size::new(1600.0, 900.0));
    for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(
            size.to_logical(bad),
            Size::new(1600.0, 900.0),
            "scale {bad}"
        );
        assert_eq!(
            PhysicalSize::from_logical(Size::new(3.0, 4.0), bad),
            PhysicalSize::new(3, 4)
        );
    }
    assert_eq!(
        PhysicalSize::from_logical(Size::new(800.0, 450.0), 1.5),
        PhysicalSize::new(1200, 675)
    );
    assert_eq!(
        PhysicalSize::from_logical(Size::new(10.4, 10.6), 1.0),
        PhysicalSize::new(10, 11)
    );
    assert_eq!(
        PhysicalSize::from_logical(Size::new(-5.0, f32::NAN), 2.0),
        PhysicalSize::new(0, 0)
    );
    assert_eq!(
        PhysicalSize::from_logical(Size::new(1e20, f32::INFINITY), 1.0),
        PhysicalSize::new(u32::MAX, u32::MAX)
    );
}

#[test]
fn platform_types_03_window_attributes() {
    let a = WindowAttributes::new("Editor");
    assert_eq!(
        a,
        WindowAttributes {
            title: "Editor".to_owned(),
            size: Size::new(800.0, 600.0),
            min_size: None,
            resizable: true,
            visible: true,
        }
    );
    assert_eq!(WindowAttributes::default(), WindowAttributes::new("Tantu"));
    assert_eq!(
        a.clone().size(300.0, 200.0),
        WindowAttributes {
            size: Size::new(300.0, 200.0),
            ..a.clone()
        }
    );
    assert_eq!(
        a.clone().min_size(100.0, 50.0),
        WindowAttributes {
            min_size: Some(Size::new(100.0, 50.0)),
            ..a.clone()
        }
    );
    assert_eq!(
        a.clone().resizable(false),
        WindowAttributes {
            resizable: false,
            ..a.clone()
        }
    );
    assert_eq!(
        a.clone().visible(false),
        WindowAttributes {
            visible: false,
            ..a
        }
    );
}

#[test]
fn platform_types_04_modifiers() {
    assert_eq!(Modifiers::NONE, Modifiers::default());
    assert!(Modifiers::NONE.is_empty());
    for m in [
        Modifiers {
            shift: true,
            ..Modifiers::NONE
        },
        Modifiers {
            control: true,
            ..Modifiers::NONE
        },
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        },
        Modifiers {
            super_key: true,
            ..Modifiers::NONE
        },
    ] {
        assert!(!m.is_empty(), "{m:?}");
    }
}

#[test]
fn platform_types_05_platform_error() {
    let unsupported = PlatformError::Unsupported("no display".to_owned());
    assert!(!unsupported.to_string().is_empty());
    assert!(unsupported.source().is_none());
    let inner: Box<dyn Error + Send + Sync> = "boom".into();
    let os = PlatformError::Os(inner);
    assert!(!os.to_string().is_empty());
    assert_eq!(
        os.source().map(ToString::to_string),
        Some("boom".to_owned())
    );
}

/// A platform defined outside the crate.
struct NullPlatform;

impl Platform for NullPlatform {
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> {
        let _ = handler;
        Ok(())
    }
}

fn assert_send_sync<T: Send + Sync>() {}

/// Opens a window through `dyn PlatformContext`, as an app runner would.
fn open(cx: &mut dyn PlatformContext) -> Option<WindowId> {
    cx.create_window(&WindowAttributes::default()).ok()
}

#[test]
fn platform_types_06_traits_and_thread_safety() {
    let mut handler = Recorder::default();
    let dynamic: &mut dyn PlatformHandler = &mut handler;
    assert!(NullPlatform.run(dynamic).is_ok());

    let mut opener = Recorder::with(|cx, seen| {
        if *seen == Seen::Started {
            open(cx);
        }
    });
    let log = FakePlatform::new().run_logged(&mut opener);
    assert_eq!(log.windows.len(), 1);

    assert_send_sync::<WindowEvent>();
    assert_send_sync::<KeyEvent>();
    assert_send_sync::<WindowAttributes>();
    assert_send_sync::<PlatformError>();
    let _ = (
        WindowEvent::Wheel {
            pointer: PointerId::Mouse,
            delta: ScrollDelta::Pixels(Vec2::new(0.0, 3.0)),
            position: Point::ZERO,
        },
        WindowEvent::PointerButton {
            pointer: PointerId::Mouse,
            button: PointerButton::Other(9),
            state: ButtonState::Pressed,
            position: Point::ZERO,
        },
        Key::Named(NamedKey::F24),
        KeyLocation::Numpad,
    );
}

// ---- FakePlatform ------------------------------------------------------------------------

/// What a handler saw.
#[derive(Clone, Debug, PartialEq)]
enum Seen {
    Started,
    Event(WindowId, WindowEvent),
    Idle,
}

type Reaction = Box<dyn FnMut(&mut dyn PlatformContext, &Seen)>;

/// Records every callback and lets a test react to it.
#[derive(Default)]
struct Recorder {
    seen: Vec<Seen>,
    react: Option<Reaction>,
}

impl Recorder {
    fn with(react: impl FnMut(&mut dyn PlatformContext, &Seen) + 'static) -> Recorder {
        Recorder {
            seen: Vec::new(),
            react: Some(Box::new(react)),
        }
    }

    fn record(&mut self, cx: &mut dyn PlatformContext, seen: Seen) {
        if let Some(react) = &mut self.react {
            react(cx, &seen);
        }
        self.seen.push(seen);
    }
}

impl PlatformHandler for Recorder {
    fn started(&mut self, cx: &mut dyn PlatformContext) {
        self.record(cx, Seen::Started);
    }

    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent) {
        self.record(cx, Seen::Event(window, event));
    }

    fn idle(&mut self, cx: &mut dyn PlatformContext) {
        self.record(cx, Seen::Idle);
    }
}

/// A handler that opens `n` windows when started and otherwise just records.
fn opener(n: usize) -> Recorder {
    Recorder::with(move |cx, seen| {
        if *seen == Seen::Started {
            for i in 0..n {
                cx.create_window(&WindowAttributes::new(format!("w{i}")).size(100.0, 50.0))
                    .expect("fake windows always open");
            }
        }
    })
}

#[test]
fn platform_fake_01_started_first_then_steps_in_order() {
    let platform = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .event(0, WindowEvent::Focused(false));
    let mut handler = opener(1);
    let log = platform.clone().run_logged(&mut handler);
    let w = log.windows[0].id;
    assert_eq!(
        handler.seen,
        [
            Seen::Started,
            Seen::Event(w, WindowEvent::RedrawRequested),
            Seen::Event(w, WindowEvent::Focused(true)),
            Seen::Idle,
            Seen::Event(w, WindowEvent::Focused(false)),
            Seen::Idle,
        ]
    );
    let mut again = opener(1);
    assert!(platform.run(&mut again).is_ok());
    assert_eq!(again.seen, handler.seen);
}

#[test]
fn platform_fake_02_create_window() {
    let ids = Rc::new(RefCell::new(Vec::new()));
    let seen_ids = ids.clone();
    let mut handler = Recorder::with(move |cx, seen| {
        if *seen == Seen::Started {
            for title in ["a", "b"] {
                let w = cx
                    .create_window(&WindowAttributes::new(title).size(100.0, 50.0))
                    .unwrap();
                assert_eq!(cx.inner_size(w), Some(PhysicalSize::new(150, 75)));
                assert_eq!(cx.scale_factor(w), Some(1.5));
                seen_ids.borrow_mut().push(w);
            }
        }
    });
    let log = FakePlatform::new()
        .scale_factor(1.5)
        .run_logged(&mut handler);
    let ids = ids.borrow();
    assert_ne!(ids[0], ids[1]);
    assert_eq!(log.windows.len(), 2);
    assert_eq!(log.windows[0].id, ids[0]);
    assert_eq!(log.windows[1].attributes.title, "b");
    assert_eq!(log.windows[0].size, PhysicalSize::new(150, 75));
    assert_eq!(log.windows[0].scale_factor, 1.5);
    assert!(!log.windows[0].closed);
    // A new window gets one redraw.
    assert_eq!(log.windows[0].redraws, 1);
    assert_eq!(log.windows[1].redraws, 1);
}

#[test]
fn platform_fake_03_steps_target_windows_by_index() {
    let mut handler = opener(2);
    let log = FakePlatform::new()
        .event(1, WindowEvent::Focused(true))
        .event(2, WindowEvent::Focused(true)) // no third window: skipped
        .event(0, WindowEvent::CloseRequested)
        .run_logged(&mut handler);
    let (w0, w1) = (log.windows[0].id, log.windows[1].id);
    let events: Vec<_> = handler
        .seen
        .iter()
        .filter_map(|s| match s {
            Seen::Event(w, e) if *e != WindowEvent::RedrawRequested => Some((*w, e.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        events,
        [
            (w1, WindowEvent::Focused(true)),
            (w0, WindowEvent::CloseRequested)
        ]
    );
    assert_eq!(log.steps_delivered, 2);
}

#[test]
fn platform_fake_04_resize_and_scale_update_before_delivery() {
    let mut handler = Recorder::with(|cx, seen| match seen {
        Seen::Started => {
            cx.create_window(&WindowAttributes::default()).unwrap();
        }
        Seen::Event(w, WindowEvent::Resized(size)) => {
            assert_eq!(cx.inner_size(*w), Some(*size));
        }
        Seen::Event(w, WindowEvent::ScaleFactorChanged(s)) => {
            assert_eq!(cx.scale_factor(*w), Some(*s));
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Resized(PhysicalSize::new(321, 123)))
        .event(0, WindowEvent::ScaleFactorChanged(2.0))
        .run_logged(&mut handler);
    assert_eq!(log.windows[0].size, PhysicalSize::new(321, 123));
    assert_eq!(log.windows[0].scale_factor, 2.0);
}

#[test]
fn platform_fake_05_redraws_are_coalesced_and_deferred() {
    let windows = Rc::new(RefCell::new(Vec::<WindowId>::new()));
    let w = windows.clone();
    let mut handler = Recorder::with(move |cx, seen| match seen {
        Seen::Started => {
            for _ in 0..2 {
                let id = cx.create_window(&WindowAttributes::default()).unwrap();
                w.borrow_mut().push(id);
            }
        }
        Seen::Event(_, WindowEvent::Focused(true)) => {
            let ws = w.borrow();
            cx.request_redraw(ws[1]);
            cx.request_redraw(ws[0]);
            cx.request_redraw(ws[1]); // coalesced
            cx.request_redraw(WindowId::from_raw(999)); // unknown: ignored
        }
        Seen::Event(win, WindowEvent::RedrawRequested) if *win == w.borrow()[0] => {
            // A request during a redraw waits for the next round (no endless loop).
            cx.request_redraw(*win);
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .step(FakeStep::Idle)
        .run_logged(&mut handler);
    let w0 = log.windows[0].id;
    let trace: Vec<String> = handler
        .seen
        .iter()
        .map(|s| match s {
            Seen::Event(w, WindowEvent::RedrawRequested) => {
                format!("R{}", if *w == w0 { 0 } else { 1 })
            }
            Seen::Event(_, _) => "E".to_owned(),
            Seen::Idle => "I".to_owned(),
            Seen::Started => "S".to_owned(),
        })
        .collect();
    // Started: both new windows redraw (w0 asks again). Step 1: the event, then w0 (asked
    // first) and w1, then idle. Step 2 (Idle): w0, idle. After the script: w0 once more.
    assert_eq!(
        trace,
        ["S", "R0", "R1", "E", "R0", "R1", "I", "R0", "I", "R0"]
    );
    assert_eq!(log.windows[0].redraws, 4);
    assert_eq!(log.windows[1].redraws, 2);
}

#[test]
fn platform_fake_06_idle_after_each_step() {
    let mut handler = opener(1);
    FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .step(FakeStep::Idle)
        .step(FakeStep::Idle)
        .event(5, WindowEvent::Focused(true)) // skipped: no idle for it
        .run_logged(&mut handler);
    assert_eq!(handler.seen.iter().filter(|s| **s == Seen::Idle).count(), 3);
}

#[test]
fn platform_fake_07_close_window() {
    let mut handler = Recorder::with(|cx, seen| match seen {
        Seen::Started => {
            let w = cx.create_window(&WindowAttributes::default()).unwrap();
            cx.request_redraw(w);
        }
        Seen::Event(w, WindowEvent::Focused(false)) => {
            cx.request_redraw(*w);
            cx.close_window(*w);
            assert_eq!(cx.inner_size(*w), None);
            assert_eq!(cx.scale_factor(*w), None);
            cx.close_window(*w); // twice: ignored
            cx.request_redraw(*w); // closed: ignored
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::CloseRequested) // closes nothing by itself
        .event(0, WindowEvent::Focused(false))
        .event(0, WindowEvent::Focused(true)) // closed: skipped
        .run_logged(&mut handler);
    assert!(log.windows[0].closed);
    assert_eq!(log.windows[0].redraws, 1); // the pending one was dropped on close
    assert_eq!(log.steps_delivered, 2);
    assert!(
        !handler
            .seen
            .contains(&Seen::Event(log.windows[0].id, WindowEvent::Focused(true)))
    );
}

#[test]
fn platform_fake_08_exit_stops_everything() {
    let mut handler = Recorder::with(|cx, seen| match seen {
        Seen::Started => {
            cx.create_window(&WindowAttributes::default()).unwrap();
        }
        Seen::Event(w, WindowEvent::CloseRequested) => {
            cx.request_redraw(*w);
            cx.exit();
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::CloseRequested)
        .event(0, WindowEvent::Focused(true))
        .run_logged(&mut handler);
    assert!(log.exited);
    assert_eq!(
        handler.seen.last(),
        Some(&Seen::Event(log.windows[0].id, WindowEvent::CloseRequested))
    );
    assert_eq!(log.steps_delivered, 1);

    // Exit from `started`: nothing else is called.
    let mut early = Recorder::with(|cx, seen| {
        if *seen == Seen::Started {
            cx.create_window(&WindowAttributes::default()).unwrap();
            cx.exit();
        }
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .run_logged(&mut early);
    assert_eq!(early.seen, [Seen::Started]);
    assert!(log.exited);
    assert_eq!(log.windows[0].redraws, 0);
}

#[test]
fn platform_fake_09_titles_and_no_surface() {
    let mut handler = Recorder::with(|cx, seen| {
        if *seen == Seen::Started {
            cx.create_window(&WindowAttributes::new("Original"))
                .unwrap();
        }
        if let Seen::Event(w, WindowEvent::Focused(true)) = seen {
            cx.set_title(*w, "Renamed");
            cx.set_title(WindowId::from_raw(999), "ignored");
            assert!(cx.surface_target(*w).is_none());
        }
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .run_logged(&mut handler);
    assert_eq!(log.windows[0].attributes.title, "Renamed");
}

#[test]
fn platform_fake_10_steps_delivered() {
    let mut handler = opener(1);
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .step(FakeStep::Idle)
        .event(3, WindowEvent::Focused(true))
        .event(
            0,
            WindowEvent::ModifiersChanged(Modifiers {
                shift: true,
                ..Modifiers::NONE
            }),
        )
        .run_logged(&mut handler);
    assert_eq!(log.steps_delivered, 3);
    assert!(!log.exited);
    let _ = KeyEvent {
        key: Key::Character("a".to_owned()),
        location: KeyLocation::Standard,
        state: ButtonState::Pressed,
        repeat: false,
        text: Some("a".to_owned()),
    };
}

#[test]
fn platform_ime_01_events_and_context() {
    let preedit = ImeEvent::Preedit {
        text: "ab".into(),
        cursor: Some((0, 2)),
    };
    assert_eq!(preedit.clone(), preedit);
    assert_ne!(ImeEvent::Commit("ab".into()), ImeEvent::Disabled);
    assert_ne!(
        WindowEvent::Ime(ImeEvent::Enabled),
        WindowEvent::Ime(ImeEvent::Disabled)
    );
    // Delivered like any window event, and the context calls exist.
    let seen: Rc<RefCell<Vec<WindowEvent>>> = Rc::default();
    let log = seen.clone();
    let mut handler = Recorder::with(move |cx, seen| {
        if *seen == Seen::Started {
            cx.create_window(&WindowAttributes::new("A")).unwrap();
        }
        if let Seen::Event(w, event @ WindowEvent::Ime(_)) = seen {
            log.borrow_mut().push(event.clone());
            cx.set_ime_allowed(*w, true);
            cx.set_ime_cursor_area(*w, Point::new(1.0, 2.0), Size::new(3.0, 4.0));
        }
    });
    FakePlatform::new()
        .event(0, WindowEvent::Ime(ImeEvent::Enabled))
        .event(0, WindowEvent::Ime(preedit.clone()))
        .run_logged(&mut handler);
    assert_eq!(
        *seen.borrow(),
        [
            WindowEvent::Ime(ImeEvent::Enabled),
            WindowEvent::Ime(preedit)
        ]
    );
}

#[test]
fn platform_fake_11_records_ime_calls() {
    let mut handler = Recorder::with(|cx, seen| {
        if *seen == Seen::Started {
            cx.create_window(&WindowAttributes::new("A")).unwrap();
        }
        if let Seen::Event(w, WindowEvent::Focused(true)) = seen {
            cx.set_ime_allowed(*w, true);
            cx.set_ime_cursor_area(*w, Point::new(1.0, 2.0), Size::new(3.0, 4.0));
            cx.set_ime_allowed(WindowId::from_raw(999), true);
        }
        if let Seen::Event(w, WindowEvent::Focused(false)) = seen {
            cx.set_ime_allowed(*w, false);
        }
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .run_logged(&mut handler);
    assert!(log.windows[0].ime_allowed);
    assert_eq!(
        log.windows[0].ime_area,
        Some((Point::new(1.0, 2.0), Size::new(3.0, 4.0)))
    );
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .event(0, WindowEvent::Focused(false))
        .run_logged(&mut handler);
    assert!(!log.windows[0].ime_allowed);
    let log = FakePlatform::new().run_logged(&mut handler);
    assert!(!log.windows[0].ime_allowed);
    assert_eq!(log.windows[0].ime_area, None);
}

#[test]
fn platform_time_01_now_and_wake_at() {
    let clock: Rc<RefCell<Vec<Instant>>> = Rc::default();
    let idles = Rc::new(Cell::new(0));
    let c = clock.clone();
    let mut handler = Recorder::with(move |cx, seen| {
        c.borrow_mut().push(cx.now());
        match seen {
            Seen::Started => {
                cx.create_window(&WindowAttributes::default()).unwrap();
                let now = cx.now();
                cx.wake_at(Some(now + Duration::from_millis(30)));
            }
            Seen::Idle if idles.replace(idles.get() + 1) == 1 => {
                // The second idle (after the first advance): replace, then cancel.
                let now = cx.now();
                cx.wake_at(Some(now + Duration::from_millis(100)));
                cx.wake_at(None);
            }
            _ => {}
        }
    });
    let log = FakePlatform::new()
        .step(FakeStep::Idle)
        .step(FakeStep::Advance(Duration::from_millis(10)))
        .step(FakeStep::Advance(Duration::from_millis(25)))
        .run_logged(&mut handler);
    let clock = clock.borrow();
    assert!(clock.windows(2).all(|w| w[0] <= w[1]), "monotonic");
    assert_eq!(
        log.wakes,
        [
            Some(Duration::from_millis(30)),
            Some(Duration::from_millis(110)),
            None
        ]
    );
}

#[test]
fn platform_fake_12_clock_and_advance() {
    let start: Rc<Cell<Option<Instant>>> = Rc::default();
    let seen_now: Rc<RefCell<Vec<Duration>>> = Rc::default();
    let (st, sn) = (start.clone(), seen_now.clone());
    let mut handler = Recorder::with(move |cx, seen| match seen {
        Seen::Started => {
            st.set(Some(cx.now()));
            cx.create_window(&WindowAttributes::default()).unwrap();
        }
        Seen::Idle => {
            sn.borrow_mut()
                .push(cx.now() - st.get().expect("started first"));
            let w = WindowId::from_raw(1);
            cx.request_redraw(w);
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .step(FakeStep::Idle)
        .step(FakeStep::Advance(Duration::from_millis(10)))
        .event(0, WindowEvent::Focused(true))
        .step(FakeStep::Advance(Duration::from_millis(5)))
        .run_logged(&mut handler);
    // The clock only moves on `Advance`.
    assert_eq!(
        *seen_now.borrow(),
        [
            Duration::ZERO,
            Duration::from_millis(10),
            Duration::from_millis(10),
            Duration::from_millis(15)
        ]
    );
    assert_eq!(log.steps_delivered, 4);
    // Each advance ran a redraw round (the redraw requested at the previous idle) and idle.
    assert_eq!(log.windows[0].redraws, 1 + 4);
    assert_eq!(handler.seen.iter().filter(|s| **s == Seen::Idle).count(), 4);
    assert!(log.wakes.is_empty());
}

/// A one-node tree update with the window labelled `label`.
fn tree_update(label: &str) -> TreeUpdate {
    let mut window = Node::new(Role::Window);
    window.set_label(label);
    TreeUpdate {
        nodes: vec![(NodeId(0), window)],
        tree: Some(Tree::new(NodeId(0))),
        tree_id: TreeId::ROOT,
        focus: NodeId(0),
    }
}

#[test]
fn platform_a11y_01_events_and_update() {
    let request = ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: NodeId(7),
        data: None,
    };
    let mut handler = Recorder::with(|cx, seen| match seen {
        Seen::Started => {
            cx.create_window(&WindowAttributes::new("A")).unwrap();
        }
        Seen::Event(w, WindowEvent::Accessibility(AccessibilityEvent::Activated)) => {
            cx.update_accessibility(*w, tree_update("first"));
        }
        Seen::Event(w, WindowEvent::Accessibility(AccessibilityEvent::Action(_))) => {
            cx.update_accessibility(*w, tree_update("after action"));
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Accessibility(AccessibilityEvent::Activated))
        .event(
            0,
            WindowEvent::Accessibility(AccessibilityEvent::Action(request.clone())),
        )
        .event(
            0,
            WindowEvent::Accessibility(AccessibilityEvent::Deactivated),
        )
        .run_logged(&mut handler);
    // The events reach the handler like any other, with AccessKit's own request type.
    let events: Vec<&WindowEvent> = handler
        .seen
        .iter()
        .filter_map(|s| match s {
            Seen::Event(_, e) => Some(e),
            _ => None,
        })
        .filter(|e| matches!(e, WindowEvent::Accessibility(_)))
        .collect();
    assert_eq!(
        events,
        [
            &WindowEvent::Accessibility(AccessibilityEvent::Activated),
            &WindowEvent::Accessibility(AccessibilityEvent::Action(request)),
            &WindowEvent::Accessibility(AccessibilityEvent::Deactivated),
        ]
    );
    let labels: Vec<Option<&str>> = log.windows[0]
        .accessibility_updates
        .iter()
        .map(|u| u.nodes[0].1.label())
        .collect();
    assert_eq!(labels, [Some("first"), Some("after action")]);
}

#[test]
fn platform_fake_13_records_updates() {
    let mut handler = Recorder::with(|cx, seen| match seen {
        Seen::Started => {
            let a = cx.create_window(&WindowAttributes::new("A")).unwrap();
            let b = cx.create_window(&WindowAttributes::new("B")).unwrap();
            // Recorded without any activation, per window and in order.
            cx.update_accessibility(a, tree_update("a1"));
            cx.update_accessibility(b, tree_update("b1"));
            cx.update_accessibility(a, tree_update("a2"));
            // An unknown window is ignored.
            cx.update_accessibility(WindowId::from_raw(99), tree_update("nobody"));
        }
        Seen::Event(w, WindowEvent::Focused(true)) => {
            cx.update_accessibility(*w, tree_update("a3"));
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .run_logged(&mut handler);
    let labels = |w: usize| -> Vec<String> {
        log.windows[w]
            .accessibility_updates
            .iter()
            .map(|u| u.nodes[0].1.label().unwrap_or("").to_owned())
            .collect()
    };
    assert_eq!(labels(0), ["a1", "a2", "a3"]);
    assert_eq!(labels(1), ["b1"]);
    assert_eq!(log.windows.len(), 2);
}

#[test]
fn platform_cursor_01_set_cursor() {
    let mut handler = Recorder::with(|cx, seen| {
        if let Seen::Started = seen {
            let a = cx.create_window(&WindowAttributes::new("A")).unwrap();
            cx.set_cursor(a, CursorIcon::ResizeVertical);
            cx.close_window(a);
            // A closed window and an unknown one are ignored, without panicking.
            cx.set_cursor(a, CursorIcon::Text);
            cx.set_cursor(WindowId::from_raw(99), CursorIcon::Pointer);
        }
    });
    let log = FakePlatform::new().run_logged(&mut handler);
    assert_eq!(log.windows[0].cursors, [CursorIcon::ResizeVertical]);
    // A new window starts with the default shape and nothing recorded.
    assert_eq!(CursorIcon::default(), CursorIcon::Default);
}

#[test]
fn platform_fake_14_records_cursors() {
    let mut handler = Recorder::with(|cx, seen| match seen {
        Seen::Started => {
            let a = cx.create_window(&WindowAttributes::new("A")).unwrap();
            let b = cx.create_window(&WindowAttributes::new("B")).unwrap();
            cx.set_cursor(a, CursorIcon::Text);
            cx.set_cursor(b, CursorIcon::Pointer);
            // Repeats are recorded too.
            cx.set_cursor(a, CursorIcon::Text);
            cx.set_cursor(WindowId::from_raw(99), CursorIcon::Grab);
        }
        Seen::Event(w, WindowEvent::Focused(true)) => {
            cx.set_cursor(*w, CursorIcon::ResizeHorizontal);
        }
        _ => {}
    });
    let log = FakePlatform::new()
        .event(0, WindowEvent::Focused(true))
        .run_logged(&mut handler);
    assert_eq!(
        log.windows[0].cursors,
        [
            CursorIcon::Text,
            CursorIcon::Text,
            CursorIcon::ResizeHorizontal
        ]
    );
    assert_eq!(log.windows[1].cursors, [CursorIcon::Pointer]);
}

#[test]
fn platform_clip_01_clipboard() {
    let seen: Rc<RefCell<Vec<Option<String>>>> = Rc::default();
    let s = seen.clone();
    let mut handler = Recorder::with(move |cx, event| {
        if let Seen::Started = event {
            let clipboard = cx.clipboard().expect("the fake platform has a clipboard");
            s.borrow_mut().push(clipboard.read_text());
            assert!(clipboard.write_text("hello"));
            // The same handle for every call.
            let again = cx.clipboard().expect("a clipboard");
            s.borrow_mut().push(again.read_text());
            assert!(again.write_text(""));
            s.borrow_mut().push(clipboard.read_text());
        }
    });
    FakePlatform::new().run_logged(&mut handler);
    assert_eq!(*seen.borrow(), [None, Some("hello".to_owned()), None]);
}

#[test]
fn platform_fake_15_initial_and_final_clipboard() {
    let seen: Rc<RefCell<Option<String>>> = Rc::default();
    let s = seen.clone();
    let mut handler = Recorder::with(move |cx, event| {
        if let Seen::Started = event {
            let clipboard = cx.clipboard().expect("a clipboard");
            *s.borrow_mut() = clipboard.read_text();
            clipboard.write_text("after");
        }
    });
    let log = FakePlatform::new()
        .clipboard_text("before")
        .run_logged(&mut handler);
    assert_eq!(seen.borrow().as_deref(), Some("before"));
    assert_eq!(log.clipboard_text.as_deref(), Some("after"));
    // Without an initial text the log shows what was last written, or nothing.
    let mut quiet = Recorder::with(|_, _| {});
    assert_eq!(
        FakePlatform::new().run_logged(&mut quiet).clipboard_text,
        None
    );
}
