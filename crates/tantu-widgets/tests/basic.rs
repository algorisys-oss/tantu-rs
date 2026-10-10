//! Tests for `docs/specs/widgets/basic.md`, rules WIDGETS-TEXT-01..02 and WIDGETS-BUTTON-01..04.

use std::cell::Cell;
use std::rc::Rc;

use tantu_view::core::{Color, Point, Size};
use tantu_view::layout::{BoxConstraints, RenderParagraph, TextMeasure, TextMetrics, TextStyleKey};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{Command, ElementId, Resources, Scene, SceneBuilder};
use tantu_view::text::{FontFamily, TextStyle, TextSystem};
use tantu_view::{
    PointerButton, PointerEvent, PointerKind, SystemText, TextPainter, View, ViewTree,
};
use tantu_widgets::{Button, Text};

const FONT: &[u8] = include_bytes!("../../tantu-text/tests/fonts/LiberationSans-Regular.ttf");

/// A text system with the test font, measuring calls counted.
struct Fonts {
    system: TextSystem,
    resources: Resources,
    measures: usize,
}

impl Fonts {
    fn new() -> Fonts {
        let mut system = TextSystem::without_system_fonts();
        system.register_font(FONT.to_vec());
        system.set_default_family(FontFamily::Named("Liberation Sans".into()));
        Fonts {
            system,
            resources: Resources::new(),
            measures: 0,
        }
    }

    fn tree<V: View>(&self, app: impl FnOnce() -> V) -> ViewTree {
        ViewTree::with_text_styles(self.system.styles(), app)
    }

    fn frame(&mut self, tree: &mut ViewTree) -> Scene {
        let mut scene = Scene::new();
        let constraints = BoxConstraints::loose(Size::new(400.0, 300.0));
        tree.frame(constraints, self, &mut scene);
        scene
    }

    fn label_size(&mut self, text: &str) -> Size {
        let style = self.system.style(TextStyle::label());
        self.system.measure(text, style, f32::INFINITY, None).size
    }
}

impl TextMeasure for Fonts {
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        self.measures += 1;
        self.system.measure(text, style, max_width, max_lines)
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        self.system.min_intrinsic_width(text, style)
    }
}

impl TextPainter for Fonts {
    fn paint_text(
        &mut self,
        scene: &mut SceneBuilder<'_>,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
        color: Color,
        origin: Point,
    ) {
        SystemText {
            system: &mut self.system,
            resources: &mut self.resources,
        }
        .paint_text(scene, text, style, max_width, max_lines, color, origin);
    }
}

fn top(tree: &ViewTree) -> ElementId {
    tree.children(tree.root())[0]
}

/// (glyph count, color) of each glyph run.
fn runs(scene: &Scene) -> Vec<(usize, Color)> {
    scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some((scene.glyphs(run).len(), run.color)),
            _ => None,
        })
        .collect()
}

fn paragraph(tree: &ViewTree, id: ElementId) -> RenderParagraph {
    let layout = tree.layout_id(id).expect("render element");
    tree.layout_tree()
        .get::<RenderParagraph>(layout)
        .expect("a paragraph")
        .clone()
}

const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

#[test]
fn widgets_text_01_paragraph_and_glyphs() {
    let mut fonts = Fonts::new();
    let mut tree = fonts.tree(|| Text::new("Hello").color(RED));
    let p = paragraph(&tree, top(&tree));
    assert_eq!(&*p.text, "Hello");
    assert_eq!(p.style, fonts.system.style(TextStyle::body()));
    assert!(p.soft_wrap);
    assert_eq!(p.max_lines, None);
    let scene = fonts.frame(&mut tree);
    assert_eq!(runs(&scene), [(5, RED)]);

    let tree = fonts.tree(|| {
        Text::new("x")
            .style(TextStyle::title())
            .max_lines(2)
            .soft_wrap(false)
    });
    let p = paragraph(&tree, top(&tree));
    assert_eq!(p.style, fonts.system.style(TextStyle::title()));
    assert_eq!((p.max_lines, p.soft_wrap), (Some(2), false));
    // The default color.
    let mut tree = fonts.tree(|| Text::new("ab"));
    let scene = fonts.frame(&mut tree);
    assert_eq!(runs(&scene), [(2, Color::from_argb32(0xFF1C_1B1F))]);
}

#[test]
fn widgets_text_02_dynamic_text_and_color() {
    let mut fonts = Fonts::new();
    let handles: Rc<Cell<Option<(Signal<String>, Signal<Color>)>>> = Rc::default();
    let mut tree = {
        let handles = handles.clone();
        fonts.tree(move || {
            let (text, color) = (signal("Hi".to_owned()), signal(RED));
            handles.set(Some((text, color)));
            Text::new(text).color(color)
        })
    };
    let (text, color) = handles.get().expect("built");
    assert_eq!(runs(&fonts.frame(&mut tree)), [(2, RED)]);
    tree.enter(|| text.set("Hello".to_owned()));
    assert_eq!(runs(&fonts.frame(&mut tree)), [(5, RED)]);
    // A new color repaints without measuring again.
    let measured = fonts.measures;
    tree.enter(|| color.set(Color::BLACK));
    assert_eq!(runs(&fonts.frame(&mut tree)), [(5, Color::BLACK)]);
    assert_eq!(fonts.measures, measured);
}

fn press(tree: &mut ViewTree, kind: PointerKind, x: f32, y: f32) {
    tree.dispatch_pointer(PointerEvent {
        kind,
        position: Point::new(x, y),
    });
}

fn click(tree: &mut ViewTree, x: f32, y: f32) {
    press(tree, PointerKind::Down(PointerButton::Primary), x, y);
    press(tree, PointerKind::Up(PointerButton::Primary), x, y);
}

/// A button (alone in the window, at the origin) counting presses into a shared cell.
fn counting_button(fonts: &Fonts, enabled: Option<Signal<bool>>) -> (ViewTree, Rc<Cell<u32>>) {
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let tree = fonts.tree(move || {
        let button = Button::new("Increment").on_press(move || c.set(c.get() + 1));
        match enabled {
            Some(enabled) => button.enabled(enabled),
            None => button,
        }
    });
    (tree, count)
}

#[test]
fn widgets_button_01_press_and_release_inside() {
    let mut fonts = Fonts::new();
    let (mut tree, count) = counting_button(&fonts, None);
    fonts.frame(&mut tree);
    click(&mut tree, 10.0, 10.0);
    assert_eq!(count.get(), 1);
    // Released outside.
    press(
        &mut tree,
        PointerKind::Down(PointerButton::Primary),
        10.0,
        10.0,
    );
    press(
        &mut tree,
        PointerKind::Up(PointerButton::Primary),
        300.0,
        200.0,
    );
    assert_eq!(count.get(), 1);
    // Another button.
    press(
        &mut tree,
        PointerKind::Down(PointerButton::Secondary),
        10.0,
        10.0,
    );
    press(
        &mut tree,
        PointerKind::Up(PointerButton::Secondary),
        10.0,
        10.0,
    );
    assert_eq!(count.get(), 1);
    // Pressed elsewhere, released over the button.
    press(
        &mut tree,
        PointerKind::Down(PointerButton::Primary),
        300.0,
        200.0,
    );
    press(
        &mut tree,
        PointerKind::Up(PointerButton::Primary),
        10.0,
        10.0,
    );
    assert_eq!(count.get(), 1);
    click(&mut tree, 20.0, 20.0);
    assert_eq!(count.get(), 2);

    // Disabled: nothing.
    let mut fonts = Fonts::new();
    let mut tree = {
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        let tree = fonts.tree(move || {
            Button::new("Off")
                .enabled(false)
                .on_press(move || c.set(c.get() + 1))
        });
        (tree, count)
    };
    fonts.frame(&mut tree.0);
    click(&mut tree.0, 10.0, 10.0);
    assert_eq!(tree.1.get(), 0);

    // The callback can write signals.
    let mut fonts = Fonts::new();
    let sig: Rc<Cell<Option<Signal<u32>>>> = Rc::default();
    let mut tree = {
        let sig = sig.clone();
        fonts.tree(move || {
            let n = signal(0u32);
            sig.set(Some(n));
            Button::new("+").on_press(move || n.update(|v| *v += 1))
        })
    };
    fonts.frame(&mut tree);
    click(&mut tree, 5.0, 5.0);
    let n = sig.get().expect("built");
    assert_eq!(tree.enter(|| n.get()), 1);
}

fn size_of(tree: &ViewTree, id: ElementId) -> Size {
    tree.layout_tree()
        .size(tree.layout_id(id).expect("render element"))
        .expect("laid out")
}

#[test]
fn widgets_button_02_metrics() {
    let mut fonts = Fonts::new();
    let label = fonts.label_size("Increment");
    let (mut tree, _) = counting_button(&fonts, None);
    fonts.frame(&mut tree);
    let button = top(&tree);
    let size = size_of(&tree, button);
    assert_eq!(size.width, (label.width + 48.0).max(64.0));
    assert_eq!(size.height, (label.height + 20.0).max(40.0));

    // A short label: the minimum width, the label centered.
    let mut tree = fonts.tree(|| Button::new("A"));
    fonts.frame(&mut tree);
    let button = top(&tree);
    assert_eq!(size_of(&tree, button), Size::new(64.0, 40.0));
    let padding = tree.children(button)[0];
    let align = tree.children(padding)[0];
    let text = tree.children(align)[0];
    let a = fonts.label_size("A");
    assert_eq!(size_of(&tree, text), a);
    let offset = tree
        .layout_tree()
        .offset(tree.layout_id(text).expect("render element"))
        .expect("laid out");
    assert!(
        (offset.x - (16.0 - a.width) / 2.0).abs() < 1e-4,
        "{offset:?}"
    );
    assert!(
        (offset.y - (20.0 - a.height) / 2.0).abs() < 1e-4,
        "{offset:?}"
    );
}

/// `idle` with `a` white over it (source-over in sRGB).
fn over_white(color: Color, a: f32) -> Color {
    let mix = |c: f32| c * (1.0 - a) + a;
    Color {
        r: mix(color.r),
        g: mix(color.g),
        b: mix(color.b),
        a: color.a,
    }
}

fn close(a: Color, b: Color) -> bool {
    (a.r - b.r).abs() < 1e-5
        && (a.g - b.g).abs() < 1e-5
        && (a.b - b.b).abs() < 1e-5
        && (a.a - b.a).abs() < 1e-5
}

/// The button's fill color in a frame.
fn fill(scene: &Scene, button: ElementId) -> Color {
    scene
        .entries()
        .iter()
        .find_map(|e| match e.command {
            Command::Fill { color, .. } if e.element == Some(button) => Some(color),
            _ => None,
        })
        .expect("the button fills its background")
}

#[test]
fn widgets_button_03_visual_states() {
    let idle = Color::from_argb32(0xFF67_50A4);
    let mut fonts = Fonts::new();
    let enabled: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let (mut tree, _) = {
        let enabled = enabled.clone();
        let count = Rc::new(Cell::new(0u32));
        let c = count.clone();
        let tree = fonts.tree(move || {
            let on = signal(true);
            enabled.set(Some(on));
            Button::new("Go")
                .enabled(on)
                .on_press(move || c.set(c.get() + 1))
        });
        (tree, count)
    };
    let on = enabled.get().expect("built");
    let scene = fonts.frame(&mut tree);
    let button = top(&tree);
    assert!(close(fill(&scene, button), idle));
    assert!(runs(&scene).iter().all(|(_, c)| *c == Color::WHITE));

    press(&mut tree, PointerKind::Move, 10.0, 10.0);
    assert!(close(
        fill(&fonts.frame(&mut tree), button),
        over_white(idle, 0.08)
    ));
    press(
        &mut tree,
        PointerKind::Down(PointerButton::Primary),
        10.0,
        10.0,
    );
    assert!(close(
        fill(&fonts.frame(&mut tree), button),
        over_white(idle, 0.12)
    ));
    press(
        &mut tree,
        PointerKind::Up(PointerButton::Primary),
        10.0,
        10.0,
    );
    assert!(close(
        fill(&fonts.frame(&mut tree), button),
        over_white(idle, 0.08)
    ));
    press(&mut tree, PointerKind::Move, 300.0, 200.0);
    assert!(close(fill(&fonts.frame(&mut tree), button), idle));

    // Disabled while hovered: disabled colors, hover cleared.
    press(&mut tree, PointerKind::Move, 10.0, 10.0);
    fonts.frame(&mut tree);
    tree.enter(|| on.set(false));
    let scene = fonts.frame(&mut tree);
    assert!(close(fill(&scene, button), Color::from_argb32(0x1F1C_1B1F)));
    assert!(
        runs(&scene)
            .iter()
            .all(|(_, c)| close(*c, Color::from_argb32(0x611C_1B1F)))
    );
    // Disabled buttons ignore the pointer.
    press(
        &mut tree,
        PointerKind::Down(PointerButton::Primary),
        10.0,
        10.0,
    );
    press(
        &mut tree,
        PointerKind::Up(PointerButton::Primary),
        10.0,
        10.0,
    );
    // Enabled again: idle, not hovered or pressed.
    tree.enter(|| on.set(true));
    assert!(close(fill(&fonts.frame(&mut tree), button), idle));
}

#[test]
fn widgets_button_04_reactive_label() {
    let mut fonts = Fonts::new();
    let label: Rc<Cell<Option<Signal<String>>>> = Rc::default();
    let mut tree = {
        let label = label.clone();
        fonts.tree(move || {
            let text = signal("Go".to_owned());
            label.set(Some(text));
            Button::new(text)
        })
    };
    let text = label.get().expect("built");
    assert_eq!(runs(&fonts.frame(&mut tree)).len(), 1);
    assert_eq!(runs(&fonts.frame(&mut tree))[0].0, 2);
    tree.enter(|| text.set("Stop!".to_owned()));
    assert_eq!(runs(&fonts.frame(&mut tree))[0].0, 5);
}
