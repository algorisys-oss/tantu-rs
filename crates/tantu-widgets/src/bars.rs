//! [`Toolbar`] and [`StatusBar`]. Spec: `docs/specs/widgets/menu-bar.md`.

use tantu_view::{AnyView, BuildCx, ElementId, View};

/// One entry of a toolbar.
enum ToolbarEntry {
    Child(AnyView),
    Separator,
}

/// A row of tool controls with separators (WIDGETS-TOOLBAR-01..03).
pub struct Toolbar {
    entries: Vec<ToolbarEntry>,
}

impl Default for Toolbar {
    fn default() -> Self {
        Toolbar::new()
    }
}

impl Toolbar {
    /// An empty toolbar.
    pub fn new() -> Self {
        Toolbar {
            entries: Vec::new(),
        }
    }

    /// Adds a control.
    pub fn child(mut self, child: impl View) -> Self {
        self.entries.push(ToolbarEntry::Child(AnyView::new(child)));
        self
    }

    /// A vertical separator between groups of controls.
    pub fn separator(mut self) -> Self {
        self.entries.push(ToolbarEntry::Separator);
        self
    }
}

impl View for Toolbar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        for entry in self.entries {
            if let ToolbarEntry::Child(child) = entry {
                let _ = child;
            }
        }
        let _ = cx;
        todo!("WIDGETS-TOOLBAR-01..03")
    }
}

/// A strip along the window's bottom: leading items on the left, trailing on the right
/// (WIDGETS-STATUSBAR-01..02).
pub struct StatusBar {
    leading: Vec<AnyView>,
    trailing: Vec<AnyView>,
}

impl Default for StatusBar {
    fn default() -> Self {
        StatusBar::new()
    }
}

impl StatusBar {
    /// An empty status bar.
    pub fn new() -> Self {
        StatusBar {
            leading: Vec::new(),
            trailing: Vec::new(),
        }
    }

    /// Adds an item on the left.
    pub fn leading(mut self, item: impl View) -> Self {
        self.leading.push(AnyView::new(item));
        self
    }

    /// Adds an item on the right.
    pub fn trailing(mut self, item: impl View) -> Self {
        self.trailing.push(AnyView::new(item));
        self
    }
}

impl View for StatusBar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.leading, self.trailing);
        todo!("WIDGETS-STATUSBAR-01..02")
    }
}
