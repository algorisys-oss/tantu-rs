//! Tests for `docs/specs/view/text.md`, rules VIEW-TEXT-01..07.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_core::{Color, Point, Size};
use tantu_layout::{
    BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex, RenderParagraph, StyleSpan,
    TextMeasure, TextMetrics, TextStyleKey,
};
use tantu_scene::{Command, ImageData, ImageId, Resources, Scene, SceneBuilder};
use tantu_text::{FontFamily, TextStyle, TextSystem};
use tantu_view::{
    BuildCx, ElementId, Paint, PaintCx, ParagraphPaint, SystemText, TextContext, TextPainter, View,
    ViewTree,
};

const FONT: &[u8] = include_bytes!("../../tantu-text/tests/fonts/LiberationSans-Regular.ttf");

type Log = Rc<RefCell<Vec<String>>>;

/// A text context that records its calls; text is 10 per character, 20 per line.
struct Recorder(Log);

impl TextMeasure for Recorder {
    fn measure(
        &mut self,
        text: &str,
        _: TextStyleKey,
        max_width: f32,
        _: Option<u32>,
    ) -> TextMetrics {
        self.0
            .borrow_mut()
            .push(format!("measure {text} at {max_width}"));
        TextMetrics {
            size: Size::new(10.0 * text.chars().count() as f32, 20.0),
            line_count: 1,
            first_baseline: 16.0,
            last_baseline: 16.0,
        }
    }

    fn min_intrinsic_width(&mut self, _: &str, _: TextStyleKey) -> f32 {
        0.0
    }
}

impl TextPainter for Recorder {
    fn paint_text(
        &mut self,
        _: &mut SceneBuilder<'_>,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
        color: Color,
        origin: Point,
    ) {
        self.0.borrow_mut().push(format!(
            "paint {text} style {} width {max_width} lines {max_lines:?} color {:?} at {},{}",
            style.0,
            color == Color::BLACK,
            origin.x,
            origin.y
        ));
    }
}

/// A paragraph element with `ParagraphPaint`.
struct Para(RenderParagraph);

impl View for Para {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(self.0, []);
        cx.set_paint(
            id,
            ParagraphPaint {
                color: Color::BLACK,
            },
        );
        id
    }
}

fn takes_context(_: &mut dyn TextContext) {}

#[test]
fn view_text_01_no_text_measure_is_a_context() {
    takes_context(&mut NoTextMeasure);
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(10.0, 10.0));
    NoTextMeasure.paint_text(
        &mut b,
        "hello",
        TextStyleKey(0),
        100.0,
        None,
        Color::BLACK,
        Point::ZERO,
    );
    b.finish().expect("balanced");
    assert!(scene.entries().is_empty());
    takes_context(&mut Recorder(Log::default()));
}

#[test]
fn view_text_02_frame_passes_the_context_to_layout_and_paint() {
    let log = Log::default();
    let mut tree = ViewTree::new(|| Para(RenderParagraph::new("hi", TextStyleKey(3))));
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(300.0, 200.0)),
        &mut Recorder(log.clone()),
        &mut scene,
    );
    let log = log.borrow();
    assert!(log.iter().any(|e| e.starts_with("measure hi")), "{log:?}");
    assert!(log.iter().any(|e| e.starts_with("paint hi")), "{log:?}");
}

/// Records what `PaintCx` reports.
struct Inspect(Log);

impl Paint for Inspect {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let c = cx.constraints();
        let boxed = cx.render::<RenderConstrainedBox>().is_some();
        let flex = cx.render::<RenderFlex>().is_some();
        self.0
            .borrow_mut()
            .push(format!("max {} box {boxed} flex {flex}", c.max_width));
        cx.paint_text(
            "via cx",
            TextStyleKey(0),
            1.0,
            None,
            Color::BLACK,
            Point::ZERO,
        );
        let _: &mut dyn TextPainter = cx.text();
    }
}

struct Inspected(Log);

impl View for Inspected {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(10.0), Some(10.0)), []);
        cx.set_paint(id, Inspect(self.0));
        id
    }
}

#[test]
fn view_text_03_paint_cx_constraints_render_and_text() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || Inspected(log))
    };
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(300.0, 200.0)),
        &mut Recorder(log.clone()),
        &mut scene,
    );
    let log = log.borrow();
    assert!(
        log.contains(&"max 300 box true flex false".to_string()),
        "{log:?}"
    );
    assert!(log.iter().any(|e| e.starts_with("paint via cx")), "{log:?}");
}

#[test]
fn view_text_04_paragraph_paint() {
    let log = Log::default();
    let mut para = RenderParagraph::new("hello", TextStyleKey(3));
    para.max_lines = Some(2);
    let mut tree = ViewTree::new(move || Para(para));
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(300.0, 200.0)),
        &mut Recorder(log.clone()),
        &mut scene,
    );
    assert!(
        log.borrow()
            .contains(&"paint hello style 3 width 300 lines Some(2) color true at 0,0".to_string())
    );
    // Without soft wrap: unbounded.
    let log = Log::default();
    let mut para = RenderParagraph::new("hello", TextStyleKey(3));
    para.soft_wrap = false;
    let mut tree = ViewTree::new(move || Para(para));
    tree.frame(
        BoxConstraints::loose(Size::new(300.0, 200.0)),
        &mut Recorder(log.clone()),
        &mut scene,
    );
    assert!(
        log.borrow().iter().any(|e| e.contains("width inf")),
        "{:?}",
        log.borrow()
    );
    // On a non-paragraph element: nothing.
    let log = Log::default();
    let mut tree = ViewTree::new(|| NotAParagraph);
    tree.frame(
        BoxConstraints::loose(Size::new(300.0, 200.0)),
        &mut Recorder(log.clone()),
        &mut scene,
    );
    assert!(log.borrow().iter().all(|e| !e.starts_with("paint")));
}

struct NotAParagraph;

impl View for NotAParagraph {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(5.0), Some(5.0)), []);
        cx.set_paint(
            id,
            ParagraphPaint {
                color: Color::BLACK,
            },
        );
        id
    }
}

#[test]
fn view_text_05_system_text() {
    let mut system = TextSystem::without_system_fonts();
    system.register_font(FONT.to_vec());
    system.set_default_family(FontFamily::Named("Liberation Sans".into()));
    let style = system.style(TextStyle::default());
    let expected = system.measure("Hello", style, f32::INFINITY, None);
    let mut resources = Resources::new();
    let mut tree = ViewTree::new(move || Para(RenderParagraph::new("Hello", style)));
    let mut scene = Scene::new();
    {
        let mut text = SystemText {
            system: &mut system,
            resources: &mut resources,
        };
        assert_eq!(text.measure("Hello", style, f32::INFINITY, None), expected);
        tree.frame(
            BoxConstraints::loose(Size::new(300.0, 200.0)),
            &mut text,
            &mut scene,
        );
    }
    let runs: Vec<_> = scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(runs.len(), 1);
    assert_eq!(scene.glyphs(&runs[0]).len(), 5);
    assert!(resources.font(runs[0].font).is_some());
}

/// A paint that asks for an image's id.
struct ImageProbe {
    data: ImageData,
    seen: Rc<Cell<Option<Option<ImageId>>>>,
}

impl Paint for ImageProbe {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        self.seen.set(Some(cx.image_id(&self.data)));
    }
}

struct Probed(ImageData, Rc<Cell<Option<Option<ImageId>>>>);

impl View for Probed {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(10.0), Some(10.0)), []);
        cx.set_paint(
            id,
            ImageProbe {
                data: self.0,
                seen: self.1,
            },
        );
        id
    }
}

#[test]
fn view_text_06_image_id() {
    let data = ImageData::rgba8(2, 2, vec![0u8; 16]).expect("valid");
    assert_eq!(NoTextMeasure.image_id(&data), None);
    let mut system = TextSystem::without_system_fonts();
    let mut resources = Resources::new();
    let mut painter = SystemText {
        system: &mut system,
        resources: &mut resources,
    };
    let id = painter.image_id(&data).expect("interned");
    assert_eq!(painter.image_id(&data), Some(id));
    assert!(resources.image(id).is_some());
    assert_eq!(resources.revision(), 1);
    // Through a paint pass.
    let seen: Rc<Cell<Option<Option<ImageId>>>> = Rc::default();
    let mut tree = {
        let (data, seen) = (data.clone(), seen.clone());
        ViewTree::new(move || Probed(data, seen))
    };
    tree.layout(
        BoxConstraints::loose(Size::new(100.0, 100.0)),
        &mut NoTextMeasure,
    );
    let mut scene = Scene::new();
    let mut builder = scene.begin(Size::new(100.0, 100.0));
    tree.paint(&mut builder, &mut NoTextMeasure);
    builder.finish().expect("balanced");
    assert_eq!(seen.get(), Some(None));
    let mut builder = scene.begin(Size::new(100.0, 100.0));
    tree.paint(
        &mut builder,
        &mut SystemText {
            system: &mut system,
            resources: &mut resources,
        },
    );
    builder.finish().expect("balanced");
    assert_eq!(seen.get(), Some(Some(id)));
}

/// A paint calling `paint_rich`.
struct RichProbe(Vec<StyleSpan>);

impl Paint for RichProbe {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        cx.paint_rich(
            "rich",
            TextStyleKey(7),
            &self.0,
            50.0,
            None,
            Color::BLACK,
            Point::new(1.0, 2.0),
        );
    }
}

struct Probing(Vec<StyleSpan>);

impl View for Probing {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(10.0), Some(10.0)), []);
        cx.set_paint(id, RichProbe(self.0));
        id
    }
}

#[test]
fn view_text_07_paint_rich() {
    let red = Color::from_rgb8(200, 0, 0);
    let spans = vec![StyleSpan {
        start: 0,
        end: 2,
        style: TextStyleKey(0),
        color: Some(red),
    }];
    // The default `paint_rich` paints plain; `PaintCx::paint_rich` reaches the painter.
    let log: Log = Rc::default();
    let mut tree = {
        let spans = spans.clone();
        ViewTree::new(move || Probing(spans))
    };
    tree.layout(
        BoxConstraints::loose(Size::new(100.0, 100.0)),
        &mut NoTextMeasure,
    );
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(100.0, 100.0));
    tree.paint(&mut b, &mut Recorder(log.clone()));
    b.finish().expect("balanced");
    assert_eq!(
        log.borrow().clone(),
        ["paint rich style 7 width 50 lines None color true at 1,2"]
    );
    // `SystemText` paints the spans' colors, and so does a paragraph with spans.
    let mut system = TextSystem::without_system_fonts();
    system.register_font(FONT.to_vec());
    system.set_default_family(FontFamily::Named("Liberation Sans".into()));
    let style = system.style(TextStyle::body());
    let mut resources = Resources::new();
    let mut tree = ViewTree::with_text_styles(system.styles(), move || {
        Para(RenderParagraph {
            spans: spans.into(),
            ..RenderParagraph::new("Hello", style)
        })
    });
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(300.0, 100.0)),
        &mut SystemText {
            system: &mut system,
            resources: &mut resources,
        },
        &mut scene,
    );
    let runs: Vec<(usize, Color)> = scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some((scene.glyphs(run).len(), run.color)),
            _ => None,
        })
        .collect();
    assert_eq!(runs, [(2, red), (3, Color::BLACK)]);
}
