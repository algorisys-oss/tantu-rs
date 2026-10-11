//! [`WidgetTester`] and [`Finder`].

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use tantu_render_headless::{HeadlessRenderer, RecordedFrame};
use tantu_render_soft::{SoftRenderer, decode_png, diff_images, encode_png};
use tantu_theme::{Theme, ThemeProvider};
use tantu_view::core::{Color, Point, Rect, Size, Vec2};
use tantu_view::layout::{BoxConstraints, RenderBox, RenderParagraph};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{ImageData, Renderer, Resources, Scene};
use tantu_view::text::{FontFamily, TextSystem};
use tantu_view::{
    ElementId, ElementKind, ImeEvent, Key, KeyEvent, LogicalKey, Modifiers, PointerButton,
    PointerEvent, PointerKind, SemanticsAction, SystemText, View, ViewTree,
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
    theme: Signal<Theme>,
    /// The fake clock (TEST-WT-13).
    clock: std::time::Instant,
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
        // The light theme is provided around the app (TEST-WT-12).
        let theme = Rc::new(Cell::new(None));
        let tree = {
            let theme = Rc::clone(&theme);
            ViewTree::with_text_styles(text.styles(), move || {
                let signal = signal(Theme::light());
                theme.set(Some(signal));
                ThemeProvider::new(signal, app())
            })
        };
        let theme = theme
            .get()
            .expect("the tester's app closure runs once, when the tree is built");
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
            theme,
            clock: std::time::Instant::now(),
        };
        tester.pump();
        tester
    }

    /// Runs one frame (apply updates, layout, paint, record); returns whether anything had
    /// changed.
    pub fn pump(&mut self) -> bool {
        // Each pump is a frame at the current clock (TEST-WT-13).
        self.tree.tick(self.clock);
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

    /// Moves the tester's clock by `by`, ticks the tree and pumps one frame (TEST-WT-13).
    pub fn advance(&mut self, by: std::time::Duration) {
        self.clock += by;
        self.pump();
    }

    /// The tester's clock, starting at the tester's creation (TEST-WT-13).
    pub fn now(&self) -> std::time::Instant {
        self.clock
    }

    /// The AccessKit tree of the last frame (TEST-WT-14).
    pub fn a11y_tree(&mut self) -> tantu_a11y::accesskit::TreeUpdate {
        tantu_a11y::AccessibilityTree::new("test").update(&self.tree)
    }

    /// Performs an accessibility action on the element `finder` matches, then pumps
    /// (TEST-WT-14).
    pub fn perform_action(&mut self, finder: &Finder, action: SemanticsAction) -> bool {
        let element = self.find(finder);
        let handled = self.tree.perform_action(element, action);
        self.pump();
        handled
    }

    /// The theme signal provided at the root, initially `Theme::light()` (TEST-WT-12).
    pub fn theme(&self) -> Signal<Theme> {
        self.theme
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

    /// A primary press at the center of the single element `finder` matches, a move by
    /// `delta`, and a release there, pumping after each. Panics if it matches no element or
    /// several.
    pub fn drag(&mut self, finder: &Finder, delta: Vec2) {
        let start = self.rect(self.find(finder)).center();
        let end = start + delta;
        for (kind, position) in [
            (PointerKind::Down(PointerButton::Primary), start),
            (PointerKind::Move, end),
            (PointerKind::Up(PointerButton::Primary), end),
        ] {
            self.pointer(PointerEvent { kind, position });
            self.pump();
        }
    }

    /// Taps the single element `finder` matches (focusing a field), then presses and releases,
    /// for each character of `text`, a key carrying it as text, pumping after each
    /// (TEST-WT-09).
    pub fn enter_text(&mut self, finder: &Finder, text: &str) {
        self.tap(finder);
        for ch in text.chars() {
            let character: Arc<str> = ch.to_string().into();
            let key = LogicalKey::Character(character.clone());
            let mut press = KeyEvent::press(key.clone(), Modifiers::default());
            press.text = Some(character);
            self.tree.dispatch_key(press);
            self.tree
                .dispatch_key(KeyEvent::release(key, Modifiers::default()));
            self.pump();
        }
    }

    /// A wheel scroll of `delta` at the center of the single element `finder` matches, then
    /// a pump (TEST-WT-11).
    pub fn scroll(&mut self, finder: &Finder, delta: Vec2) {
        let center = self.rect(self.find(finder)).center();
        self.pointer(PointerEvent {
            kind: PointerKind::Scroll(delta),
            position: center,
        });
        self.pump();
    }

    /// Dispatches an IME event to the focused element, then pumps (TEST-WT-10).
    pub fn ime(&mut self, event: ImeEvent) {
        self.tree.dispatch_ime(event);
        self.pump();
    }

    /// Presses and releases `key` (with `modifiers`) on the focused element, then pumps.
    pub fn press_key(&mut self, key: LogicalKey, modifiers: Modifiers) {
        self.tree
            .dispatch_key(KeyEvent::press(key.clone(), modifiers));
        self.tree.dispatch_key(KeyEvent::release(key, modifiers));
        self.pump();
    }

    /// Focuses the single element `finder` matches, then pumps (panics if it isn't
    /// focusable).
    pub fn focus(&mut self, finder: &Finder) {
        let element = self.find(finder);
        if !self.tree.focus(element) {
            panic!("{finder:?} matches an element that isn't focusable");
        }
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

    /// Renders the last frame with the software renderer over the theme's `surface` and
    /// compares it with the PNG at `path` (tolerance 2); with `TANTU_UPDATE_GOLDENS=1` it
    /// writes the PNG instead.
    pub fn matches_golden(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let mut soft = SoftRenderer::new(pixels(self.size.width), pixels(self.size.height));
        if let Err(error) = soft.render(&self.scene, &self.resources) {
            panic!("rendering the golden frame failed: {error}");
        }
        let image = soft
            .snapshot()
            .expect("the software renderer has a target after rendering");
        // Over the theme's surface, as a window shows it (TEST-WT-12).
        let surface = self.tree.enter(|| self.theme.with(|t| t.colors.surface));
        let image = over(&image, surface);
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

/// `image` composited over `background` (source-over in 8-bit sRGB with straight alpha), so
/// the result is opaque.
fn over(image: &ImageData, background: Color) -> ImageData {
    let [br, bg, bb, _] = background.to_rgba8();
    let pixels: Vec<u8> = image
        .pixels()
        .chunks_exact(4)
        .flat_map(|px| {
            let a = u32::from(px[3]);
            let mix =
                |s: u8, b: u8| ((u32::from(s) * a + u32::from(b) * (255 - a) + 127) / 255) as u8;
            [mix(px[0], br), mix(px[1], bg), mix(px[2], bb), 255]
        })
        .collect();
    ImageData::rgba8(image.width(), image.height(), pixels)
        .expect("the composited image has the source's dimensions and pixel count")
}
