//! Tests for `docs/specs/layout/text.md`, rules LAYOUT-TEXT-01..10.

use tantu_core::Size;
use tantu_layout::{
    BoxConstraints, IntrinsicChildren, LayoutChildren, LayoutTree, MeasureCache, NoTextMeasure,
    RenderBox, RenderParagraph, TextMeasure, TextMetrics, TextStyleKey, TextWidthBasis,
};

const INF: f32 = f32::INFINITY;
const NAN: f32 = f32::NAN;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

const S1: TextStyleKey = TextStyleKey(1);
const S2: TextStyleKey = TextStyleKey(2);

/// The fake measurer of the spec: each character is `10 · style` wide, lines are 20 tall with
/// the baseline at 16, words break at spaces (a space is one character wide), `\n` is a hard
/// break. Counts its calls.
#[derive(Default)]
struct Fake {
    measures: u32,
    mins: u32,
}

impl Fake {
    fn char_width(style: TextStyleKey) -> f32 {
        10.0 * style.0.max(1) as f32
    }
}

impl TextMeasure for Fake {
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        self.measures += 1;
        let cw = Fake::char_width(style);
        let mut lines: Vec<f32> = Vec::new();
        if !text.is_empty() {
            for hard in text.split('\n') {
                let mut line: Option<f32> = None;
                for word in hard.split(' ') {
                    let w = word.chars().count() as f32 * cw;
                    line = Some(match line {
                        None => w,
                        Some(l) if l + cw + w > max_width => {
                            lines.push(l);
                            w
                        }
                        Some(l) => l + cw + w,
                    });
                }
                lines.push(line.unwrap_or(0.0));
            }
        }
        if let Some(max) = max_lines {
            lines.truncate(max as usize);
        }
        let n = lines.len() as u32;
        TextMetrics {
            size: s(lines.iter().copied().fold(0.0, f32::max), 20.0 * n as f32),
            line_count: n,
            first_baseline: if n > 0 { 16.0 } else { 0.0 },
            last_baseline: if n > 0 {
                20.0 * (n - 1) as f32 + 16.0
            } else {
                0.0
            },
        }
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        self.mins += 1;
        text.split([' ', '\n'])
            .map(|w| w.chars().count() as f32 * Fake::char_width(style))
            .fold(0.0, f32::max)
    }
}

#[test]
fn layout_text_01_plain_types() {
    let zero = TextMetrics::default();
    assert_eq!(
        (
            zero.size,
            zero.line_count,
            zero.first_baseline,
            zero.last_baseline
        ),
        (Size::ZERO, 0, 0.0, 0.0)
    );
    let mut none = NoTextMeasure;
    for (text, width) in [("", 0.0), ("hello world", 30.0), ("x", INF), ("y", NAN)] {
        assert_eq!(none.measure(text, S1, width, None), zero);
        assert_eq!(none.measure(text, S2, width, Some(1)), zero);
        assert_eq!(none.min_intrinsic_width(text, S1), 0.0);
    }
    assert_eq!(TextStyleKey(3), TextStyleKey(3));
    assert!(TextStyleKey(1) < TextStyleKey(2));
}

#[test]
fn layout_text_02_cache_passthrough_and_key() {
    let mut cache = MeasureCache::new(Fake::default(), 64);
    let mut fresh = Fake::default();
    let args: [(&str, TextStyleKey, f32, Option<u32>); 6] = [
        ("hello world", S1, 60.0, None),
        ("hello world", S2, 60.0, None),
        ("hello world", S1, 70.0, None),
        ("hello world", S1, 60.0, Some(1)),
        ("hello there", S1, 60.0, None),
        ("", S1, 60.0, None),
    ];
    for (text, style, width, lines) in args {
        let expected = fresh.measure(text, style, width, lines);
        assert_eq!(
            cache.measure(text, style, width, lines),
            expected,
            "{text:?}"
        );
    }
    assert_eq!((cache.misses(), cache.hits()), (6, 0));
    assert_eq!(cache.inner().measures, 6);
    for (text, style, width, lines) in args {
        let expected = fresh.measure(text, style, width, lines);
        assert_eq!(
            cache.measure(text, style, width, lines),
            expected,
            "{text:?}"
        );
    }
    assert_eq!((cache.misses(), cache.hits()), (6, 6));
    assert_eq!(cache.inner().measures, 6);
    assert_eq!(cache.len(), 6);
    assert!(!cache.is_empty());
}

#[test]
fn layout_text_03_one_line_shortcut() {
    let mut cache = MeasureCache::new(Fake::default(), 64);
    let single = cache.measure("hi there", S1, INF, None);
    assert_eq!(single.size, s(80.0, 20.0));
    // Wider than the single line: answered from the single-line result.
    assert_eq!(cache.measure("hi there", S1, 100.0, None), single);
    assert_eq!(cache.measure("hi there", S1, 80.0, Some(1)), single);
    assert_eq!(cache.inner().measures, 1);
    assert_eq!(cache.hits(), 2);
    // Narrower: measured.
    let wrapped = cache.measure("hi there", S1, 79.0, None);
    assert_eq!(wrapped.line_count, 2);
    assert_eq!(cache.inner().measures, 2);
    // Hard breaks: a line limit below the single-line count can't use the shortcut.
    let two = cache.measure("a\nb", S1, INF, None);
    assert_eq!(two.line_count, 2);
    let limited = cache.measure("a\nb", S1, 100.0, Some(1));
    assert_eq!(limited.line_count, 1);
    assert_eq!(cache.inner().measures, 4);
    // Without the single-line result cached, a wide call is an ordinary miss.
    let mut cache = MeasureCache::new(Fake::default(), 64);
    cache.measure("hi there", S1, 100.0, None);
    assert_eq!(cache.inner().measures, 1);
}

#[test]
fn layout_text_04_min_intrinsic_width_cached() {
    let mut cache = MeasureCache::new(Fake::default(), 64);
    assert_eq!(cache.min_intrinsic_width("hello wide world", S1), 50.0);
    assert_eq!(cache.min_intrinsic_width("hello wide world", S1), 50.0);
    assert_eq!(cache.inner().mins, 1);
    assert_eq!(cache.min_intrinsic_width("hello wide world", S2), 100.0);
    assert_eq!(cache.inner().mins, 2);
}

#[test]
fn layout_text_05_bounded_size() {
    let mut cache = MeasureCache::new(Fake::default(), 4);
    for i in 0..10 {
        cache.measure("text", S1, 100.0 + i as f32, None);
        assert!(cache.len() <= 4, "len {} after {i}", cache.len());
    }
    // The most recent entries survive.
    let before = cache.inner().measures;
    cache.measure("text", S1, 109.0, None);
    cache.measure("text", S1, 108.0, None);
    assert_eq!(cache.inner().measures, before);
    // An entry used again keeps surviving while others come and go.
    let mut cache = MeasureCache::new(Fake::default(), 4);
    cache.measure("keep", S1, 1.0, None);
    for i in 0..20 {
        cache.measure("other", S1, 100.0 + i as f32, None);
        cache.measure("keep", S1, 1.0, None);
    }
    assert_eq!(
        cache.inner().measures,
        21,
        "only the first `keep` and the 20 `other`s were measured"
    );
    // Capacity 0 caches nothing.
    let mut none = MeasureCache::new(Fake::default(), 0);
    none.measure("x", S1, 10.0, None);
    none.measure("x", S1, 10.0, None);
    assert_eq!((none.len(), none.inner().measures, none.hits()), (0, 2, 0));
    // Odd capacities round up.
    let mut one = MeasureCache::new(Fake::default(), 1);
    one.measure("a", S1, 10.0, None);
    one.measure("b", S1, 10.0, None);
    assert!(one.len() <= 2);
    // clear empties it; the counters stay.
    let mut cache = MeasureCache::new(Fake::default(), 8);
    cache.measure("x", S1, 10.0, None);
    cache.measure("x", S1, 10.0, None);
    cache.min_intrinsic_width("x", S1);
    cache.clear();
    assert!(cache.is_empty());
    assert_eq!((cache.hits(), cache.misses()), (1, 2));
    cache.measure("x", S1, 10.0, None);
    cache.min_intrinsic_width("x", S1);
    assert_eq!((cache.inner().measures, cache.inner().mins), (2, 2));
    cache.inner_mut().measures = 0;
    assert_eq!(cache.inner().measures, 0);
}

/// Sizes itself from the pass's measurer: "abc" in S1 at the incoming max width.
struct UsesText;

impl RenderBox for UsesText {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        children.text().measure("abc", S1, c.max_width, None).size
    }

    fn min_intrinsic_width(&self, _: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.text().min_intrinsic_width("abcd", S1)
    }
}

#[test]
fn layout_text_06_text_context() {
    let mut tree = LayoutTree::new();
    let id = tree.insert(UsesText);
    let c = BoxConstraints::loose(s(100.0, 100.0));
    // A plain pass measures with NoTextMeasure.
    assert_eq!(tree.layout(id, c), Size::ZERO);
    assert_eq!(tree.min_intrinsic_width(id, INF), 0.0);
    // A session uses its measurer.
    let mut fake = Fake::default();
    tree.mark_needs_layout(id);
    {
        let mut session = tree.with_text(&mut fake);
        assert_eq!(session.layout(id, c), s(30.0, 20.0));
        // Ordinary tree caching applies: same constraints, clean, no new measure.
        assert_eq!(session.layout(id, c), s(30.0, 20.0));
        assert_eq!(session.min_intrinsic_width(id, 50.0), 40.0);
        let _ = session.max_intrinsic_width(id, 50.0);
        let _ = session.min_intrinsic_height(id, 50.0);
        let _ = session.max_intrinsic_height(id, 50.0);
    }
    assert_eq!((fake.measures, fake.mins), (1, 1));
    assert_eq!(tree.size(id), Some(s(30.0, 20.0)));
}

fn paragraph(text: &str) -> RenderParagraph {
    RenderParagraph::new(text, S1)
}

/// Lays out `p` with the fake measurer and returns its size.
fn lay(p: RenderParagraph, c: BoxConstraints) -> Size {
    let mut tree = LayoutTree::new();
    let id = tree.insert(p);
    let mut fake = Fake::default();
    tree.with_text(&mut fake).layout(id, c)
}

#[test]
fn layout_text_07_paragraph_constructor() {
    let p = RenderParagraph::new("hello", S2);
    assert_eq!(&*p.text, "hello");
    assert_eq!(p.style, S2);
    assert!(p.soft_wrap);
    assert_eq!(p.max_lines, None);
    assert_eq!(p.text_width_basis, TextWidthBasis::Parent);
    assert_eq!(TextWidthBasis::default(), TextWidthBasis::Parent);
    assert_eq!(p, RenderParagraph::new(String::from("hello"), S2));
    assert_ne!(p, RenderParagraph::new("hullo", S2));
}

#[test]
fn layout_text_08_paragraph_layout() {
    // "hello world foo": one line is 150 wide; at 100 it wraps to "hello" / "world foo" (90).
    let text = "hello world foo";
    assert_eq!(
        lay(paragraph(text), BoxConstraints::loose(s(200.0, 100.0))),
        s(150.0, 20.0)
    );
    assert_eq!(
        lay(paragraph(text), BoxConstraints::loose(s(100.0, 100.0))),
        s(100.0, 40.0)
    );
    let longest = RenderParagraph {
        text_width_basis: TextWidthBasis::LongestLine,
        ..paragraph(text)
    };
    assert_eq!(
        lay(longest, BoxConstraints::loose(s(100.0, 100.0))),
        s(90.0, 40.0)
    );
    let no_wrap = RenderParagraph {
        soft_wrap: false,
        ..paragraph(text)
    };
    assert_eq!(
        lay(no_wrap, BoxConstraints::loose(s(100.0, 100.0))),
        s(100.0, 20.0)
    );
    let one_line = RenderParagraph {
        max_lines: Some(1),
        ..paragraph(text)
    };
    assert_eq!(
        lay(one_line, BoxConstraints::loose(s(100.0, 100.0))),
        s(100.0, 20.0)
    );
    // Unbounded width: one line.
    assert_eq!(
        lay(paragraph(text), BoxConstraints::new(0.0, INF, 0.0, 100.0)),
        s(150.0, 20.0)
    );
    // Constrained by the tree.
    assert_eq!(
        lay(paragraph(text), BoxConstraints::tight(s(40.0, 10.0))),
        s(40.0, 10.0)
    );
    // Children are not laid out.
    let mut tree = LayoutTree::new();
    let p = tree.insert(paragraph("x"));
    let kid = tree.insert(paragraph("y"));
    tree.set_children(p, &[kid]).expect("valid");
    let mut fake = Fake::default();
    tree.with_text(&mut fake)
        .layout(p, BoxConstraints::loose(s(100.0, 100.0)));
    assert_eq!(tree.size(kid), None);
}

#[test]
fn layout_text_09_paragraph_intrinsics() {
    let mut tree = LayoutTree::new();
    let wrap = tree.insert(paragraph("hello world foo"));
    let no_wrap = tree.insert(RenderParagraph {
        soft_wrap: false,
        ..paragraph("hello world foo")
    });
    let one_line = tree.insert(RenderParagraph {
        max_lines: Some(1),
        ..paragraph("hello world foo")
    });
    let mut fake = Fake::default();
    let mut session = tree.with_text(&mut fake);
    assert_eq!(session.min_intrinsic_width(wrap, INF), 50.0);
    assert_eq!(session.max_intrinsic_width(wrap, INF), 150.0);
    assert_eq!(session.min_intrinsic_height(wrap, 100.0), 40.0);
    assert_eq!(session.max_intrinsic_height(wrap, 100.0), 40.0);
    assert_eq!(session.max_intrinsic_height(wrap, INF), 20.0);
    assert_eq!(session.min_intrinsic_height(no_wrap, 100.0), 20.0);
    assert_eq!(session.max_intrinsic_height(one_line, 100.0), 20.0);
}

/// Returns whatever metrics it is told to.
struct Wild(TextMetrics, f32);

impl TextMeasure for Wild {
    fn measure(&mut self, _: &str, _: TextStyleKey, _: f32, _: Option<u32>) -> TextMetrics {
        self.0
    }

    fn min_intrinsic_width(&mut self, _: &str, _: TextStyleKey) -> f32 {
        self.1
    }
}

#[test]
fn layout_text_10_no_text_system_and_robustness() {
    let mut tree = LayoutTree::new();
    let id = tree.insert(paragraph("hello"));
    let c = BoxConstraints::new(5.0, 100.0, 7.0, 100.0);
    assert_eq!(tree.layout(id, c), s(5.0, 7.0));
    for metrics in [
        TextMetrics {
            size: s(NAN, -5.0),
            line_count: u32::MAX,
            first_baseline: NAN,
            last_baseline: INF,
        },
        TextMetrics {
            size: s(INF, INF),
            ..TextMetrics::default()
        },
    ] {
        for basis in [TextWidthBasis::Parent, TextWidthBasis::LongestLine] {
            let mut tree = LayoutTree::new();
            let id = tree.insert(RenderParagraph {
                text_width_basis: basis,
                ..paragraph("x")
            });
            let mut wild = Wild(metrics, NAN);
            let mut session = tree.with_text(&mut wild);
            for c in [
                c,
                BoxConstraints::UNCONSTRAINED,
                BoxConstraints::new(NAN, NAN, NAN, NAN),
            ] {
                let size = session.layout(id, c);
                if c.is_normalized() {
                    assert!(c.is_satisfied_by(size), "{size:?} in {c:?}");
                }
                let _ = session.min_intrinsic_width(id, NAN);
                assert!(session.max_intrinsic_height(id, 10.0) >= 0.0);
            }
        }
    }
}
