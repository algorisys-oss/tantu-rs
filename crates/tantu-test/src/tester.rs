//! [`WidgetTester`] and [`Finder`].

use std::path::Path;

use tantu_render_headless::{HeadlessRenderer, RecordedFrame};
use tantu_render_soft::{SoftRenderer, decode_png, diff_images, encode_png};
use tantu_view::core::{Point, Rect, Size, Vec2};
use tantu_view::layout::{BoxConstraints, RenderBox, RenderParagraph};
use tantu_view::scene::{Renderer, Resources, Scene};
use tantu_view::text::{FontFamily, TextSystem};
use tantu_view::{
    ElementId, ElementKind, Key, PointerButton, PointerEvent, PointerKind, SystemText, View,
    ViewTree,
};

/// Liberation Sans Regular (SIL Open Font License 1.1, see `fonts/LICENSE-OFL.txt`).
const TEST_FONT: &[u8] = include_bytes!("../fonts/LiberationSans-Regular.ttf");

/// Channel tolerance of golden comparisons (as the renderer goldens).
const GOLDEN_TOLERANCE: u8 = 2;

/// Whether an element of a tree matches a finder.
type Matcher = Box<dyn Fn(&ViewTree, ElementId) -> bool>;

/// Which elements to find.
pub struct Finder {
    description: String,
    matches: Matcher,
}

impl Finder {
    /// Elements with this key ([`Keyed`](tantu_view::Keyed)).
    pub fn key(key: impl Into<Key>) -> Self {
        let key = key.into();
        Finder {
            description: format!("Finder::key({:?})", &*key.0),
            matches: Box::new(move |tree, id| tree.find_key(&key).contains(&id)),
        }
    }

    /// `Text` elements showing exactly `text`.
    pub fn text(text: impl Into<String>) -> Self {
        let text = text.into();
        Finder {
            description: format!("Finder::text({text:?})"),
            matches: Box::new(move |tree, id| paragraph_text(tree, id).as_deref() == Some(&text)),
        }
    }

    /// Render elements whose render object is an `R`.
    pub fn render<R: RenderBox>() -> Self {
        Finder {
            description: format!("Finder::render::<{}>()", std::any::type_name::<R>()),
            matches: Box::new(|tree, id| {
                tree.layout_id(id)
                    .is_some_and(|layout| tree.layout_tree().get::<R>(layout).is_some())
            }),
        }
    }
}

impl std::fmt::Debug for Finder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)
    }
}

/// The text of a `Text` element (its `RenderParagraph`).
fn paragraph_text(tree: &ViewTree, id: ElementId) -> Option<String> {
    let layout = tree.layout_id(id)?;
    let paragraph = tree.layout_tree().get::<RenderParagraph>(layout)?;
    Some(paragraph.text.to_string())
}

/// Builds a view in a test window and drives it: frames, pointer input, finders, goldens.
pub struct WidgetTester {
    tree: ViewTree,
    text: TextSystem,
    resources: Resources,
    recorder: HeadlessRenderer,
    scene: Scene,
    size: Size,
}

impl WidgetTester {
    /// Builds `app` in an 800 × 600 window (logical, scale 1) with the test font as default
    /// family, and pumps the first frame.
    pub fn new<V: View>(app: impl FnOnce() -> V) -> Self {
        WidgetTester::with_size(800.0, 600.0, app)
    }

    /// Same, in a `width` × `height` window.
    pub fn with_size<V: View>(width: f32, height: f32, app: impl FnOnce() -> V) -> Self {
        let mut text = TextSystem::without_system_fonts();
        text.register_font(TEST_FONT.to_vec());
        text.set_default_family(FontFamily::Named("Liberation Sans".into()));
        let tree = ViewTree::with_text_styles(text.styles(), app);
        let size = Size::new(width, height);
        let mut recorder = HeadlessRenderer::new(1, 1);
        recorder.resize(pixels(width), pixels(height), 1.0);
        let mut tester = WidgetTester {
            tree,
            text,
            resources: Resources::new(),
            recorder,
            scene: Scene::new(),
            size,
        };
        tester.pump();
        tester
    }

    /// Runs one frame (apply updates, layout, paint, record); returns whether anything had
    /// changed.
    pub fn pump(&mut self) -> bool {
        let changed = self.tree.needs_frame();
        let mut text = SystemText {
            system: &mut self.text,
            resources: &mut self.resources,
        };
        self.tree
            .frame(BoxConstraints::tight(self.size), &mut text, &mut self.scene);
        if let Err(error) = self.recorder.render(&self.scene, &self.resources) {
            panic!("the headless recorder failed: {error}");
        }
        changed
    }

    /// The last recorded frame.
    pub fn frame(&self) -> &RecordedFrame {
        self.recorder
            .last_frame()
            .expect("WidgetTester pumps a frame when it is created")
    }

    /// Primary press and release at the center of the single element `finder` matches, then a
    /// pump. Panics if it matches no element or several.
    pub fn tap(&mut self, finder: &Finder) {
        let center = self.rect(self.find(finder)).center();
        for kind in [
            PointerKind::Down(PointerButton::Primary),
            PointerKind::Up(PointerButton::Primary),
        ] {
            self.pointer(PointerEvent {
                kind,
                position: center,
            });
        }
        self.pump();
    }

    /// A pointer move to the center of the single element `finder` matches, then a pump.
    pub fn hover(&mut self, finder: &Finder) {
        let center = self.rect(self.find(finder)).center();
        self.pointer(PointerEvent {
            kind: PointerKind::Move,
            position: center,
        });
        self.pump();
    }

    /// Dispatches a raw pointer event (no pump).
    pub fn pointer(&mut self, event: PointerEvent) -> bool {
        self.tree.dispatch_pointer(event)
    }

    /// The single element `finder` matches (panics otherwise, naming the finder).
    pub fn find(&self, finder: &Finder) -> ElementId {
        match self.find_all(finder).as_slice() {
            [one] => *one,
            [] => panic!("{finder:?} matches no element"),
            many => panic!("{finder:?} matches {} elements, expected one", many.len()),
        }
    }

    /// Every element `finder` matches, in tree order.
    pub fn find_all(&self, finder: &Finder) -> Vec<ElementId> {
        let mut found = Vec::new();
        let mut stack = vec![self.tree.root()];
        while let Some(id) = stack.pop() {
            if (finder.matches)(&self.tree, id) {
                found.push(id);
            }
            stack.extend(self.tree.children(id).iter().rev());
        }
        found
    }

    /// The element's bounds in window coordinates (from the last layout). A region's bounds
    /// are the union of its render children's (empty at the origin when there are none).
    pub fn rect(&self, element: ElementId) -> Rect {
        if self.tree.kind(element) == Some(ElementKind::Region) {
            return self
                .tree
                .children(element)
                .iter()
                .map(|child| self.rect(*child))
                .reduce(Rect::union)
                .unwrap_or(Rect::ZERO);
        }
        let layout = self.tree.layout_tree();
        let size = self
            .tree
            .layout_id(element)
            .and_then(|id| layout.size(id))
            .unwrap_or(Size::ZERO);
        let mut origin = Vec2::ZERO;
        let mut next = Some(element);
        while let Some(id) = next {
            if let Some(offset) = self.tree.layout_id(id).and_then(|l| layout.offset(l)) {
                origin += offset;
            }
            next = self.tree.parent(id);
        }
        Rect::from_origin_size(Point::new(origin.x, origin.y), size)
    }

    /// The text a `Text` element shows (`None` for other elements).
    pub fn text(&self, element: ElementId) -> Option<String> {
        paragraph_text(&self.tree, element)
    }

    /// Runs `f` with the tree's runtime current.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        self.tree.enter(f)
    }

    /// Renders the last frame with the software renderer and compares it with the PNG at
    /// `path` (tolerance 2); with `TANTU_UPDATE_GOLDENS=1` it writes the PNG instead.
    pub fn matches_golden(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let mut soft = SoftRenderer::new(pixels(self.size.width), pixels(self.size.height));
        if let Err(error) = soft.render(&self.scene, &self.resources) {
            panic!("rendering the golden frame failed: {error}");
        }
        let image = soft
            .snapshot()
            .expect("the software renderer has a target after rendering");
        if std::env::var_os("TANTU_UPDATE_GOLDENS").is_some_and(|v| v == "1") {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)
                    .unwrap_or_else(|e| panic!("creating {}: {e}", dir.display()));
            }
            let png = encode_png(&image).unwrap_or_else(|e| panic!("encoding PNG: {e}"));
            std::fs::write(path, png).unwrap_or_else(|e| panic!("writing {}: {e}", path.display()));
            return;
        }
        let bytes = std::fs::read(path).unwrap_or_else(|e| {
            panic!(
                "golden {} can't be read ({e}); run with TANTU_UPDATE_GOLDENS=1 to create it",
                path.display()
            )
        });
        let expected =
            decode_png(&bytes).unwrap_or_else(|e| panic!("golden {}: {e}", path.display()));
        match diff_images(&image, &expected, GOLDEN_TOLERANCE) {
            None => panic!(
                "golden {}: size {}x{}, frame {}x{}",
                path.display(),
                expected.width(),
                expected.height(),
                image.width(),
                image.height()
            ),
            Some(diff) if diff.differing_pixels > 0 => panic!(
                "golden {}: {} pixels differ (largest channel difference {})",
                path.display(),
                diff.differing_pixels,
                diff.max_channel_delta
            ),
            Some(_) => {}
        }
    }
}

/// A logical length as a pixel count at scale 1 (at least 1).
fn pixels(length: f32) -> u32 {
    if length.is_finite() && length >= 1.0 {
        length.round() as u32
    } else {
        1
    }
}
