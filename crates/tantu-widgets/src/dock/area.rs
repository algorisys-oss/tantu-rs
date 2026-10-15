//! [`DockArea`]: shows a [`DockLayout`] and lets the user change it. Spec:
//! `docs/specs/widgets/dock-area.md`.

use std::rc::Rc;

use tantu_view::reactive::Signal;
use tantu_view::{AnyView, BuildCx, ElementId, View};

use super::{DockGroupId, DockLayout, PanelId};

type PanelFn<T> = Rc<dyn Fn(&PanelId) -> T>;

/// Shows a dock layout and lets the user change it (WIDGETS-DOCKAREA-01..09).
pub struct DockArea {
    layout: Signal<DockLayout>,
    content: PanelFn<AnyView>,
    title: PanelFn<String>,
    closable: PanelFn<bool>,
    on_close_request: PanelFn<bool>,
    active_group: Option<Signal<Option<DockGroupId>>>,
    placeholder: Option<Rc<dyn Fn() -> AnyView>>,
}

impl DockArea {
    /// A dock area showing `layout`; `content` builds a panel's view the first time it
    /// appears in the layout.
    pub fn new<V: View>(
        layout: Signal<DockLayout>,
        content: impl Fn(&PanelId) -> V + 'static,
    ) -> Self {
        DockArea {
            layout,
            content: Rc::new(move |id| AnyView::new(content(id))),
            title: Rc::new(|id| id.as_str().to_owned()),
            closable: Rc::new(|_| true),
            on_close_request: Rc::new(|_| true),
            active_group: None,
            placeholder: None,
        }
    }

    /// A tab's title (default: the panel id). Signals read inside are tracked.
    pub fn title(mut self, title: impl Fn(&PanelId) -> String + 'static) -> Self {
        self.title = Rc::new(title);
        self
    }

    /// Whether a panel shows a close button and can be closed (default: all).
    pub fn closable(mut self, closable: impl Fn(&PanelId) -> bool + 'static) -> Self {
        self.closable = Rc::new(closable);
        self
    }

    /// Asked before the user closes a panel; false keeps it. Default: close.
    pub fn on_close_request(mut self, f: impl Fn(&PanelId) -> bool + 'static) -> Self {
        self.on_close_request = Rc::new(f);
        self
    }

    /// Shares the active group (the one the user last worked in) with the app.
    pub fn active_group(mut self, group: Signal<Option<DockGroupId>>) -> Self {
        self.active_group = Some(group);
        self
    }

    /// What to show when the layout is empty (default: nothing).
    pub fn placeholder<V: View>(mut self, view: impl Fn() -> V + 'static) -> Self {
        self.placeholder = Some(Rc::new(move || AnyView::new(view())));
        self
    }
}

impl View for DockArea {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.layout,
            self.content,
            self.title,
            self.closable,
            self.on_close_request,
            self.active_group,
            self.placeholder,
        );
        todo!("WIDGETS-DOCKAREA-01..09")
    }
}
