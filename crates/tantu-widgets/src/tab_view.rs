//! Tab bars and kept tab panels. Spec: `docs/specs/widgets/tab-view.md`.
use crate::IconData;
use std::rc::Rc;
use tantu_view::reactive::Signal;
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Prop, View};

/// One tab's header: a label, an optional leading icon, enabled by default.
#[derive(Clone)]
pub struct Tab {
    label: String,
    icon: Option<IconData>,
    enabled: Rc<dyn Fn() -> bool>,
}
impl Tab {
    /// A tab labelled `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            enabled: Rc::new(|| true),
        }
    }
    /// An icon above the label (the bar becomes 64 px tall instead of 48 px).
    pub fn icon(mut self, icon: IconData) -> Self {
        self.icon = Some(icon);
        self
    }
    /// A disabled tab is shown dimmed, can't be selected by the user and is skipped by keys.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = match enabled.into_prop() {
            Prop::Value(value) => Rc::new(move || value),
            Prop::Dynamic(f) => Rc::from(f),
        };
        self
    }
}

/// The row of tab headers. The selected index is a shared signal.
pub struct TabBar {
    selected: Signal<usize>,
    tabs: Vec<Tab>,
    scrollable: bool,
}
impl TabBar {
    /// A bar showing `selected` (clamped to the tabs).
    pub fn new(selected: Signal<usize>) -> Self {
        Self {
            selected,
            tabs: Vec::new(),
            scrollable: false,
        }
    }
    /// Adds the next tab.
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }
    /// Adds several tabs.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = Tab>) -> Self {
        self.tabs.extend(tabs);
        self
    }
    /// Natural-width tabs in a horizontal scroll view instead of equal widths; default false.
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
}
impl View for TabBar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.selected, self.scrollable, cx);
        let _ = self.tabs.iter().map(|t| (&t.label, t.icon, &t.enabled));
        todo!("WIDGETS-TAB-01")
    }
}

type PageBuilder = Rc<dyn Fn() -> AnyView>;

/// The pages; only the selected one is shown. Pages build on first selection.
pub struct TabBarView {
    selected: Signal<usize>,
    pages: Vec<PageBuilder>,
    keep_alive: bool,
}
impl TabBarView {
    /// Pages following `selected` (clamped to the pages).
    pub fn new(selected: Signal<usize>) -> Self {
        Self {
            selected,
            pages: Vec::new(),
            keep_alive: true,
        }
    }
    /// Adds the next page; `build` runs when the page is first selected.
    pub fn page<V: View>(mut self, build: impl Fn() -> V + 'static) -> Self {
        self.pages.push(Rc::new(move || AnyView::new(build())));
        self
    }
    /// Keep visited pages alive offstage (default true); false disposes a page on leaving.
    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }
}
impl View for TabBarView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.selected, self.pages, self.keep_alive, cx);
        todo!("WIDGETS-TAB-08")
    }
}

/// A `TabBar` above an expanded `TabBarView`, with Control+Tab cycling across both.
pub struct TabView {
    selected: Option<Signal<usize>>,
    tabs: Vec<(Tab, PageBuilder)>,
    scrollable: bool,
    keep_alive: bool,
}
impl Default for TabView {
    fn default() -> Self {
        Self::new()
    }
}
impl TabView {
    /// Uses an owner-scoped selected signal starting at 0 unless `selected` shares one.
    pub fn new() -> Self {
        Self {
            selected: None,
            tabs: Vec::new(),
            scrollable: false,
            keep_alive: true,
        }
    }
    /// Shares the selected index.
    pub fn selected(mut self, selected: Signal<usize>) -> Self {
        self.selected = Some(selected);
        self
    }
    /// Adds a tab and its page builder.
    pub fn tab<V: View>(mut self, tab: Tab, build: impl Fn() -> V + 'static) -> Self {
        self.tabs
            .push((tab, Rc::new(move || AnyView::new(build()))));
        self
    }
    /// As [`TabBar::scrollable`].
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
    /// As [`TabBarView::keep_alive`].
    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }
}
impl View for TabView {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.selected,
            self.tabs,
            self.scrollable,
            self.keep_alive,
            cx,
        );
        todo!("WIDGETS-TAB-07")
    }
}
