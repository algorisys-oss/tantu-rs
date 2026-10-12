//! # Tantu: Widgets
//!
//! The standard Tantu widget set. Layout widgets follow Flutter's names, defaults and
//! semantics in `snake_case` (AGENTS.md); every widget is a [`View`](tantu_view::View) built
//! once, with reactive props that update their element at the next frame. Specs:
//! `docs/specs/widgets/`.
//!
//! ```
//! use tantu_view::ViewTree;
//! use tantu_view::layout::{BoxConstraints, NoTextMeasure};
//! use tantu_view::core::Size;
//! use tantu_widgets::{Column, Padding, SizedBox};
//!
//! let mut tree = ViewTree::new(|| {
//!     Padding::all(16.0).child(
//!         Column::new()
//!             .spacing(8.0)
//!             .child(SizedBox::new(50.0, 50.0))
//!             .child(SizedBox::new(50.0, 50.0)),
//!     )
//! });
//! let size = tree.layout(BoxConstraints::loose(Size::new(200.0, 200.0)), &mut NoTextMeasure);
//! assert_eq!(size, Size::new(200.0, 200.0));
//! ```

#![forbid(unsafe_code)]

mod basic;
mod data_grid;
mod divider;
mod focus_scope;
mod grid_selection;
mod icon;
#[cfg(feature = "material-icons")]
mod icons;
mod image;
mod layout;
mod list_view;
mod opacity;
mod rich_text;
mod scroll;
mod slider;
mod switcher;
mod text_editing;
mod text_field;
mod theme;
mod toggles;
mod tooltip;
mod util;

pub use basic::{Button, Text};
pub use data_grid::{
    CellValue, DataGrid, DataSource, FnSource, GridColumn, GridEditError, RowFilter, SortState,
};
pub use divider::{Divider, VerticalDivider};
pub use focus_scope::FocusScope;
pub use grid_selection::{GridCell, GridSelection, GridSelectionController, GridSelectionMode};
pub use icon::{Icon, IconButton, IconData};
#[cfg(feature = "material-icons")]
pub use icon::{MATERIAL_ICONS_FAMILY, MATERIAL_ICONS_FONT};
#[cfg(feature = "material-icons")]
pub use icons::Icons;
pub use image::{BoxFit, Image, ImageError, decode_png};
pub use list_view::ListView;
pub use opacity::Opacity;
pub use rich_text::{RichText, TextSpan};
pub use scroll::{ScrollController, ScrollView};
pub use slider::Slider;
pub use switcher::AnimatedSwitcher;
pub use text_editing::{
    Direction, EditingValue, Movement, TextEditingController, TextRange, TextSelection,
};
pub use text_field::{RedoIntent, SelectAllIntent, TextField, UndoIntent};
pub use toggles::{Checkbox, Radio, Switch};
pub use tooltip::Tooltip;

pub use layout::{
    Align, Center, Column, ConstrainedBox, Expanded, Flexible, LimitedBox, Padding, Positioned,
    Row, SizedBox, Spacer, Stack,
};
