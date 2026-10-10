//! Small layouts for the layout-tree tests. Each logs its name when `perform_layout` runs and
//! when an intrinsic method runs, so tests can check exactly what was laid out or queried.

#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::{Size, Vec2};
use tantu_layout::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// A shared log of `perform_layout` calls (`"name"`) and intrinsic queries (`"name:min_w"`, …).
#[derive(Clone, Default)]
pub struct Log(Rc<RefCell<Vec<String>>>);

impl Log {
    pub fn push(&self, entry: String) {
        self.0.borrow_mut().push(entry);
    }

    /// Takes the entries logged so far.
    pub fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.0.borrow_mut())
    }

    /// Takes the entries, sorted (for checks that don't depend on order).
    pub fn take_sorted(&self) -> Vec<String> {
        let mut v = self.take();
        v.sort();
        v
    }
}

/// A leaf that wants `size`, with intrinsics equal to its size.
#[derive(Clone)]
pub struct Leaf {
    pub name: &'static str,
    pub size: Size,
    pub log: Log,
}

impl RenderBox for Leaf {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        self.log.push(self.name.to_string());
        c.constrain(self.size)
    }

    fn min_intrinsic_width(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.log.push(format!("{}:min_w", self.name));
        self.size.width
    }

    fn max_intrinsic_width(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.log.push(format!("{}:max_w", self.name));
        self.size.width
    }

    fn min_intrinsic_height(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.log.push(format!("{}:min_h", self.name));
        self.size.height
    }

    fn max_intrinsic_height(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.log.push(format!("{}:max_h", self.name));
        self.size.height
    }
}

/// How a [`Column`] lays out its children.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChildMode {
    /// Width `0..=max_width`, height unbounded; uses the sizes.
    Loose,
    /// Tight at this node's own max width and the child's index-based height 10.
    Tight,
    /// As `Loose`, with `layout_ignoring_size`.
    IgnoringSize,
}

/// Stacks its children vertically from the top; as wide as the widest child.
#[derive(Clone)]
pub struct Column {
    pub name: &'static str,
    pub mode: ChildMode,
    pub log: Log,
}

impl RenderBox for Column {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        self.log.push(self.name.to_string());
        let mut y = 0.0f32;
        let mut width = 0.0f32;
        for i in 0..children.len() {
            let size = match self.mode {
                ChildMode::Loose => {
                    children.layout(i, BoxConstraints::new(0.0, c.max_width, 0.0, f32::INFINITY))
                }
                ChildMode::Tight => {
                    let size = Size::new(c.constrain_width(f32::INFINITY), 10.0);
                    children.layout(i, BoxConstraints::tight(size))
                }
                ChildMode::IgnoringSize => {
                    children.layout_ignoring_size(
                        i,
                        BoxConstraints::new(0.0, c.max_width, 0.0, f32::INFINITY),
                    );
                    Size::new(0.0, 10.0)
                }
            };
            children.set_offset(i, Vec2::new(0.0, y));
            y += size.height;
            width = width.max(size.width);
        }
        Size::new(width, y)
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.log.push(format!("{}:min_w", self.name));
        (0..children.len())
            .map(|i| children.min_intrinsic_width(i, height))
            .fold(0.0, f32::max)
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.log.push(format!("{}:max_h", self.name));
        (0..children.len())
            .map(|i| children.max_intrinsic_height(i, width))
            .sum()
    }
}

/// Takes all the space it is given (`sized_by_parent`), laying out children loosely inside.
#[derive(Clone)]
pub struct Fill {
    pub name: &'static str,
    pub log: Log,
}

impl RenderBox for Fill {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        self.log.push(self.name.to_string());
        let size = c.biggest();
        for i in 0..children.len() {
            children.layout(i, BoxConstraints::loose(size));
        }
        size
    }

    fn sized_by_parent(&self) -> bool {
        true
    }
}

/// A plain value layout for `set`: wants `size`, compares by value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fixed(pub Size);

impl RenderBox for Fixed {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        c.constrain(self.0)
    }
}

/// Returns whatever it is told to, ignoring the constraints (for LAYOUT-TREE-09).
pub struct Liar(pub Size);

impl RenderBox for Liar {
    fn perform_layout(&mut self, _: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        self.0
    }

    fn min_intrinsic_width(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.0.width
    }

    fn max_intrinsic_height(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.0.height
    }
}

/// A layout with only the defaults.
pub struct Plain;

impl RenderBox for Plain {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        c.smallest()
    }
}

pub fn leaf(log: &Log, name: &'static str, w: f32, h: f32) -> Leaf {
    Leaf {
        name,
        size: Size::new(w, h),
        log: log.clone(),
    }
}

pub fn column(log: &Log, name: &'static str, mode: ChildMode) -> Column {
    Column {
        name,
        mode,
        log: log.clone(),
    }
}

pub fn loose(w: f32, h: f32) -> BoxConstraints {
    BoxConstraints::loose(Size::new(w, h))
}
