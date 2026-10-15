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

mod bars;
mod basic;
mod combo_box;
mod data_grid;
mod date;
mod date_picker;
mod dialog;
mod directionality;
mod divider;
mod dock;
mod drag_and_drop;
mod focus_scope;
mod form;
mod formatter;
mod grid_selection;
mod icon;
#[cfg(feature = "material-icons")]
mod icons;
mod image;
mod layout;
mod layout_grid;
mod list_view;
mod menu;
mod offstage;
mod opacity;
mod rich_text;
mod scroll;
mod slider;
mod split_view;
mod switcher;
mod tab_view;
mod text_editing;
mod text_field;
mod theme;
mod time;
mod time_picker;
mod toggles;
mod tooltip;
mod tree_view;
mod util;

pub use bars::{StatusBar, Toolbar};
pub use basic::{Button, Text};
pub use combo_box::ComboBox;
pub use data_grid::{
    CellValue, DataGrid, DataSource, FnSource, GridColumn, GridEditError, RowFilter, SortState,
};
pub use date::{Date, Weekday};
pub use date_picker::{CalendarDatePicker, DatePickerField};
pub use dialog::{AlertDialog, Dialog, DialogController};
pub use directionality::{Directionality, PositionedDirectional};
pub use divider::{Divider, VerticalDivider};
pub use dock::{
    DockArea, DockError, DockGroupId, DockLayout, DockLoadError, DockNode, DockSide, DockSplitId,
    DockTarget, DockView, PanelId,
};
pub use drag_and_drop::{DragTarget, Draggable};
pub use focus_scope::FocusScope;
pub use form::{AutovalidateMode, Form, FormController, TextFormField, Validator, validators};
pub use formatter::{
    FilteringFormatter, LengthLimitingFormatter, MaskFormatter, TextInputFormatter,
};
pub use grid_selection::{GridCell, GridSelection, GridSelectionController, GridSelectionMode};
pub use icon::{Icon, IconButton, IconData};
#[cfg(feature = "material-icons")]
pub use icon::{MATERIAL_ICONS_FAMILY, MATERIAL_ICONS_FONT};
#[cfg(feature = "material-icons")]
pub use icons::Icons;
pub use image::{BoxFit, Image, ImageError, decode_png};
pub use layout_grid::{GridPlacement, LayoutGrid};
pub use list_view::ListView;
pub use menu::{ContextMenuRegion, MenuAnchor, MenuBar, MenuController, MenuItem};
pub use offstage::Offstage;
pub use opacity::Opacity;
pub use rich_text::{RichText, TextSpan};
pub use scroll::{ScrollController, ScrollView};
pub use slider::Slider;
pub use split_view::{SplitPane, SplitView};
pub use switcher::AnimatedSwitcher;
pub use tab_view::{Tab, TabBar, TabBarView, TabView};
pub use text_editing::{
    Direction, EditingValue, Movement, TextEditingController, TextRange, TextSelection,
};
pub use text_field::{
    CopyIntent, CutIntent, PasteIntent, RedoIntent, SelectAllIntent, TextField, UndoIntent,
};
pub use time::Time;
pub use time_picker::TimePickerField;
pub use toggles::{Checkbox, Radio, Switch};
pub use tooltip::Tooltip;
pub use tree_view::{TreeDataSource, TreeNodeId, TreeRowState, TreeView};

pub use layout::{
    Align, Center, Column, ConstrainedBox, Expanded, Flexible, LimitedBox, Padding, Positioned,
    Row, SizedBox, Spacer, Stack,
};
