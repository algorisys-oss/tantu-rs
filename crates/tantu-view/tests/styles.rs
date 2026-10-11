//! Tests for `docs/specs/text/styles.md`, rules VIEW-STYLES-01..02.

use std::cell::Cell;
use std::rc::Rc;

use tantu_layout::{RenderConstrainedBox, TextStyleKey};
use tantu_text::{TextStyle, TextStyles};
use tantu_view::{BuildCx, ElementId, View, ViewTree};

struct Styled(Rc<Cell<Option<TextStyleKey>>>);

impl View for Styled {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.0.set(Some(cx.text_style(TextStyle::title())));
        cx.render(RenderConstrainedBox::sized(Some(1.0), Some(1.0)), [])
    }
}

#[test]
fn view_styles_01_build_time_keys() {
    let styles = TextStyles::new();
    let other = styles.key(TextStyle::body());
    let seen: Rc<Cell<Option<TextStyleKey>>> = Rc::default();
    let tree = {
        let seen = seen.clone();
        ViewTree::with_text_styles(styles.clone(), move || Styled(seen))
    };
    let key = seen.get().expect("built");
    assert_ne!(key, other);
    assert_eq!(styles.get(key), Some(TextStyle::title()));
    assert_eq!(tree.text_styles().len(), 2);
    // `new` has a table of its own.
    let seen2: Rc<Cell<Option<TextStyleKey>>> = Rc::default();
    let own = {
        let seen2 = seen2.clone();
        ViewTree::new(move || Styled(seen2))
    };
    assert_eq!(own.text_styles().len(), 1);
    assert_eq!(styles.len(), 2);
}

/// A view that brings a "font".
struct Fonted;

const FAKE_FONT: &[u8] = b"not really a font";

impl View for Fonted {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.ensure_font(FAKE_FONT);
        cx.ensure_font(FAKE_FONT);
        cx.render(RenderConstrainedBox::sized(Some(1.0), Some(1.0)), [])
    }
}

#[test]
fn view_styles_02_ensure_font() {
    let styles = TextStyles::new();
    let _tree = ViewTree::with_text_styles(styles.clone(), || Fonted);
    assert_eq!(styles.take_pending_fonts(), vec![FAKE_FONT]);
    assert!(styles.take_pending_fonts().is_empty());
}
