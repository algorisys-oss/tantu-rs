//! The Tantu widget gallery (spec `docs/specs/examples/gallery.md`): a header, a sidebar of
//! sections and the selected section's samples.

use tantu::prelude::*;

/// The section names, in sidebar order.
pub const SECTIONS: [&str; 6] = [
    "Buttons",
    "Text",
    "Selection",
    "Text field",
    "Lists",
    "Media",
];

/// The gallery's view.
pub fn gallery() -> impl View {
    #[allow(unreachable_code)]
    {
        todo!("GALLERY-01");
        SizedBox::shrink()
    }
}
