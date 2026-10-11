//! Tests for `docs/specs/widgets/image.md`, rules WIDGETS-IMAGE-01..05.

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Rect, Size};
use tantu_view::layout::{Alignment, BoxConstraints, NoTextMeasure, RenderImage};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{Command, ImageData, ImageDraw, ImageSampling, Scene};
use tantu_view::{Keyed, View, ViewTree};
use tantu_widgets::{BoxFit, Column, Image, Row, decode_png};

/// A 60 × 40 image of one color.
fn image() -> ImageData {
    ImageData::rgba8(60, 40, vec![0x80u8; 60 * 40 * 4]).expect("valid")
}

/// A 4 × 2 image of eight distinct colors (red, green, blue, yellow / cyan, magenta, white,
/// black).
fn colorful() -> ImageData {
    let px = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 0, 255],
        [0, 255, 255, 255],
        [255, 0, 255, 255],
        [255, 255, 255, 255],
        [0, 0, 0, 255],
    ];
    ImageData::rgba8(4, 2, px.concat()).expect("valid")
}

fn encode(
    width: u32,
    height: u32,
    color: png::ColorType,
    depth: png::BitDepth,
    data: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(color);
        encoder.set_depth(depth);
        let mut writer = encoder.write_header().expect("header");
        writer.write_image_data(data).expect("data");
    }
    out
}

/// The image command the element keyed `key` painted.
fn draw(tester: &WidgetTester, key: &str) -> Option<ImageDraw> {
    let element = tester.find(&Finder::key(key));
    tester
        .frame()
        .entries_for(element)
        .find_map(|e| match e.command {
            Command::Image(draw) => Some(draw),
            _ => None,
        })
}

fn window(view: impl View + 'static) -> WidgetTester {
    let view = Cell::new(Some(view));
    WidgetTester::with_size(300.0, 200.0, move || {
        Column::new().child(Keyed::new("i", view.take().expect("built once")))
    })
}

#[test]
fn widgets_image_01_decode_png() {
    let rgba = [
        255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 10, 20, 30, 40, 1, 2, 3, 4, 5, 6, 7, 8,
    ];
    let decoded = decode_png(&encode(
        3,
        2,
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &rgba,
    ))
    .expect("decodes");
    assert_eq!((decoded.width(), decoded.height()), (3, 2));
    assert_eq!(decoded.pixels(), &rgba);
    let rgb = decode_png(&encode(
        2,
        1,
        png::ColorType::Rgb,
        png::BitDepth::Eight,
        &[1, 2, 3, 4, 5, 6],
    ))
    .expect("decodes");
    assert_eq!(rgb.pixels(), &[1, 2, 3, 255, 4, 5, 6, 255]);
    let gray = decode_png(&encode(
        2,
        1,
        png::ColorType::Grayscale,
        png::BitDepth::Eight,
        &[9, 200],
    ))
    .expect("decodes");
    assert_eq!(gray.pixels(), &[9, 9, 9, 255, 200, 200, 200, 255]);
    let gray_alpha = decode_png(&encode(
        1,
        1,
        png::ColorType::GrayscaleAlpha,
        png::BitDepth::Eight,
        &[7, 77],
    ))
    .expect("decodes");
    assert_eq!(gray_alpha.pixels(), &[7, 7, 7, 77]);
    let deep = decode_png(&encode(
        1,
        1,
        png::ColorType::Rgba,
        png::BitDepth::Sixteen,
        &[0xFF, 0xFF, 0, 0, 0x80, 0, 0xFF, 0xFF],
    ))
    .expect("decodes");
    assert_eq!(deep.pixels(), &[255, 0, 128, 255]);
    let error = decode_png(b"not a png").expect_err("fails");
    assert!(!error.to_string().is_empty());
}

#[test]
fn widgets_image_02_sizing() {
    let size = |tester: &WidgetTester| tester.rect(tester.find(&Finder::key("i"))).size();
    let tester = window(Image::new(image()));
    assert_eq!(size(&tester), Size::new(60.0, 40.0));
    tester.find(&Finder::render::<RenderImage>());
    assert_eq!(
        size(&window(Image::new(image()).width(120.0))),
        Size::new(120.0, 80.0)
    );
    assert_eq!(
        size(&window(Image::new(image()).height(10.0))),
        Size::new(15.0, 10.0)
    );
    assert_eq!(
        size(&window(Image::new(image()).size(50.0, 50.0))),
        Size::new(50.0, 50.0)
    );
}

#[test]
fn widgets_image_03_fit() {
    let whole = Rect::from_ltrb(0.0, 0.0, 120.0, 120.0);
    let cases = [
        (
            BoxFit::Contain,
            None,
            Rect::from_ltrb(0.0, 20.0, 120.0, 100.0),
        ),
        (
            BoxFit::Cover,
            Some(Rect::from_ltrb(10.0, 0.0, 50.0, 40.0)),
            whole,
        ),
        (BoxFit::Fill, None, whole),
        (
            BoxFit::FitWidth,
            None,
            Rect::from_ltrb(0.0, 20.0, 120.0, 100.0),
        ),
        (
            BoxFit::FitHeight,
            Some(Rect::from_ltrb(10.0, 0.0, 50.0, 40.0)),
            whole,
        ),
        (BoxFit::None, None, Rect::from_ltrb(30.0, 40.0, 90.0, 80.0)),
        (
            BoxFit::ScaleDown,
            None,
            Rect::from_ltrb(30.0, 40.0, 90.0, 80.0),
        ),
    ];
    for (fit, src, dest) in cases {
        let tester = window(Image::new(image()).size(120.0, 120.0).fit(fit));
        let draw = draw(&tester, "i").expect("an image command");
        assert_eq!((draw.src, draw.dest), (src, dest), "{fit:?}");
        assert_eq!(
            (draw.sampling, draw.opacity),
            (ImageSampling::Linear, 1.0),
            "{fit:?}"
        );
    }
    // `None` with a box larger than the image shows the whole image, centered.
    let tester = window(Image::new(image()).size(120.0, 120.0).fit(BoxFit::None));
    assert_eq!(
        draw(&tester, "i").map(|d| d.dest),
        Some(Rect::from_ltrb(30.0, 40.0, 90.0, 80.0))
    );
    // `ScaleDown` of an image smaller than its box keeps its size; `None` crops a bigger one.
    let tester = window(Image::new(image()).size(30.0, 30.0).fit(BoxFit::None));
    let d = draw(&tester, "i").expect("an image command");
    assert_eq!(
        (d.src, d.dest),
        (
            Some(Rect::from_ltrb(15.0, 5.0, 45.0, 35.0)),
            Rect::from_ltrb(0.0, 0.0, 30.0, 30.0)
        )
    );
    let mut tester = WidgetTester::with_size(160.0, 60.0, || {
        Row::new()
            .child(Image::new(colorful()).size(50.0, 50.0).fit(BoxFit::Contain))
            .child(Image::new(colorful()).size(50.0, 50.0).fit(BoxFit::Cover))
            .child(Image::new(colorful()).size(50.0, 50.0).fit(BoxFit::Fill))
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/image.png"
    ));
}

#[test]
fn widgets_image_04_alignment_and_opacity() {
    let tester = window(
        Image::new(image())
            .size(120.0, 120.0)
            .fit(BoxFit::Contain)
            .alignment(Alignment::TOP_LEFT),
    );
    assert_eq!(
        draw(&tester, "i").map(|d| d.dest),
        Some(Rect::from_ltrb(0.0, 0.0, 120.0, 80.0))
    );
    let tester = window(
        Image::new(image())
            .size(120.0, 120.0)
            .fit(BoxFit::Cover)
            .alignment(Alignment::BOTTOM_RIGHT),
    );
    assert_eq!(
        draw(&tester, "i").map(|d| d.src),
        Some(Some(Rect::from_ltrb(20.0, 0.0, 60.0, 40.0)))
    );
    let out: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let opacity = signal(1.0f32);
            out.set(Some(opacity));
            Column::new().child(Keyed::new("i", Image::new(image()).opacity(opacity)))
        })
    };
    let opacity = out.get().expect("built");
    assert_eq!(draw(&tester, "i").map(|d| d.opacity), Some(1.0));
    tester.enter(|| opacity.set(0.5));
    assert!(tester.pump());
    assert_eq!(draw(&tester, "i").map(|d| d.opacity), Some(0.5));
}

#[test]
fn widgets_image_05_registration() {
    let data = image();
    let tester = {
        let (a, b) = (data.clone(), data.clone());
        let views = Cell::new(Some((a, b)));
        WidgetTester::with_size(300.0, 200.0, move || {
            let (a, b) = views.take().expect("built once");
            Column::new()
                .child(Keyed::new("a", Image::new(a)))
                .child(Keyed::new("b", Image::new(b)))
        })
    };
    let (a, b) = (
        draw(&tester, "a").expect("drawn"),
        draw(&tester, "b").expect("drawn"),
    );
    assert_eq!(a.image, b.image);
    // Without resources (no text system), nothing is painted.
    let views = Cell::new(Some(data));
    let mut tree = ViewTree::new(move || Image::new(views.take().expect("built once")));
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::tight(Size::new(300.0, 200.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    assert!(
        scene
            .entries()
            .iter()
            .all(|e| !matches!(e.command, Command::Image(_)))
    );
}
