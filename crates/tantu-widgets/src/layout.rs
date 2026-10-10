//! Layout widgets: thin views over `tantu-layout`'s render objects, with Flutter's names and
//! defaults. Spec: `docs/specs/widgets/layout.md`.

use tantu_view::core::EdgeInsets;
use tantu_view::layout::{
    Alignment, BoxConstraints, CrossAxisAlignment, FlexFit, MainAxisAlignment, MainAxisSize,
    StackFit,
};
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Prop, View};

/// Insets its child (Flutter's `Padding`).
pub struct Padding {
    padding: Prop<EdgeInsets>,
    child: Option<AnyView>,
}

impl Padding {
    /// Padding of `padding` (a value, a closure or a signal).
    pub fn new(padding: impl IntoProp<EdgeInsets>) -> Self {
        Padding {
            padding: padding.into_prop(),
            child: None,
        }
    }

    /// `Padding::new(EdgeInsets::all(value))`.
    pub fn all(value: f32) -> Self {
        Padding::new(EdgeInsets::all(value))
    }

    /// Sets the child (replacing a previous one).
    pub fn child(mut self, child: impl View) -> Self {
        self.child = Some(AnyView::new(child));
        self
    }
}

impl View for Padding {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.padding, self.child, cx);
        todo!()
    }
}

/// Positions its child by `alignment` (Flutter's `Align`).
pub struct Align {
    alignment: Prop<Alignment>,
    width_factor: Option<f32>,
    height_factor: Option<f32>,
    child: Option<AnyView>,
}

impl Align {
    /// Aligns the child at `alignment`, filling the available space when bounded.
    pub fn new(alignment: impl IntoProp<Alignment>) -> Self {
        Align {
            alignment: alignment.into_prop(),
            width_factor: None,
            height_factor: None,
            child: None,
        }
    }

    /// Width as a multiple of the child's width, instead of filling.
    pub fn width_factor(mut self, factor: f32) -> Self {
        self.width_factor = Some(factor);
        self
    }

    /// Height as a multiple of the child's height, instead of filling.
    pub fn height_factor(mut self, factor: f32) -> Self {
        self.height_factor = Some(factor);
        self
    }

    /// Sets the child.
    pub fn child(mut self, child: impl View) -> Self {
        self.child = Some(AnyView::new(child));
        self
    }
}

impl View for Align {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.alignment,
            self.width_factor,
            self.height_factor,
            self.child,
            cx,
        );
        todo!()
    }
}

/// Centers its child (Flutter's `Center`, an [`Align`] at `Alignment::CENTER`).
pub struct Center(Align);

impl Center {
    /// A centering box.
    pub fn new() -> Self {
        Center(Align::new(Alignment::CENTER))
    }

    /// Sets the child.
    pub fn child(self, child: impl View) -> Self {
        Center(self.0.child(child))
    }
}

impl Default for Center {
    fn default() -> Self {
        Center::new()
    }
}

impl View for Center {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.0.build(cx)
    }
}

/// A box of a given size (Flutter's `SizedBox`).
pub struct SizedBox {
    width: Option<Prop<f32>>,
    height: Option<Prop<f32>>,
    child: Option<AnyView>,
}

impl SizedBox {
    /// Exactly `width` × `height`.
    pub fn new(width: impl IntoProp<f32>, height: impl IntoProp<f32>) -> Self {
        SizedBox {
            width: Some(width.into_prop()),
            height: Some(height.into_prop()),
            child: None,
        }
    }

    /// Exactly `width` wide; the height follows the child.
    pub fn width(width: impl IntoProp<f32>) -> Self {
        SizedBox {
            width: Some(width.into_prop()),
            height: None,
            child: None,
        }
    }

    /// Exactly `height` tall; the width follows the child.
    pub fn height(height: impl IntoProp<f32>) -> Self {
        SizedBox {
            width: None,
            height: Some(height.into_prop()),
            child: None,
        }
    }

    /// As large as the constraints allow (`SizedBox.expand`).
    pub fn expand() -> Self {
        SizedBox::new(f32::INFINITY, f32::INFINITY)
    }

    /// 0 × 0 (`SizedBox.shrink`).
    pub fn shrink() -> Self {
        SizedBox::new(0.0, 0.0)
    }

    /// Sets the child.
    pub fn child(mut self, child: impl View) -> Self {
        self.child = Some(AnyView::new(child));
        self
    }
}

impl View for SizedBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.width, self.height, self.child, cx);
        todo!()
    }
}

/// Adds constraints to its child's (Flutter's `ConstrainedBox`).
pub struct ConstrainedBox {
    constraints: Prop<BoxConstraints>,
    child: Option<AnyView>,
}

impl ConstrainedBox {
    /// Adds `constraints` (through `BoxConstraints::enforce`).
    pub fn new(constraints: impl IntoProp<BoxConstraints>) -> Self {
        ConstrainedBox {
            constraints: constraints.into_prop(),
            child: None,
        }
    }

    /// Sets the child.
    pub fn child(mut self, child: impl View) -> Self {
        self.child = Some(AnyView::new(child));
        self
    }
}

impl View for ConstrainedBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.constraints, self.child, cx);
        todo!()
    }
}

/// Properties shared by [`Row`] and [`Column`].
struct Flex {
    main_axis_alignment: Prop<MainAxisAlignment>,
    main_axis_size: Prop<MainAxisSize>,
    cross_axis_alignment: Prop<CrossAxisAlignment>,
    spacing: Prop<f32>,
    children: Vec<AnyView>,
}

impl Flex {
    fn new() -> Self {
        Flex {
            main_axis_alignment: Prop::Value(MainAxisAlignment::Start),
            main_axis_size: Prop::Value(MainAxisSize::Max),
            cross_axis_alignment: Prop::Value(CrossAxisAlignment::Center),
            spacing: Prop::Value(0.0),
            children: Vec::new(),
        }
    }

    fn build(self, vertical: bool, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.main_axis_alignment,
            self.main_axis_size,
            self.cross_axis_alignment,
            self.spacing,
            self.children,
            vertical,
            cx,
        );
        todo!()
    }
}

macro_rules! flex_widget {
    ($(#[$doc:meta])* $name:ident, $vertical:expr) => {
        $(#[$doc])*
        pub struct $name(Flex);

        impl $name {
            /// Flutter's defaults: `Start`, `Max`, `Center`, spacing 0, no children.
            pub fn new() -> Self {
                $name(Flex::new())
            }

            /// How free main-axis space is distributed.
            pub fn main_axis_alignment(mut self, value: impl IntoProp<MainAxisAlignment>) -> Self {
                self.0.main_axis_alignment = value.into_prop();
                self
            }

            /// How much main-axis space it takes.
            pub fn main_axis_size(mut self, value: impl IntoProp<MainAxisSize>) -> Self {
                self.0.main_axis_size = value.into_prop();
                self
            }

            /// How children are placed on the cross axis.
            pub fn cross_axis_alignment(
                mut self,
                value: impl IntoProp<CrossAxisAlignment>,
            ) -> Self {
                self.0.cross_axis_alignment = value.into_prop();
                self
            }

            /// Space between adjacent children.
            pub fn spacing(mut self, value: impl IntoProp<f32>) -> Self {
                self.0.spacing = value.into_prop();
                self
            }

            /// Appends a child.
            pub fn child(mut self, child: impl View) -> Self {
                self.0.children.push(AnyView::new(child));
                self
            }

            /// Appends children.
            pub fn children<V: View>(mut self, children: impl IntoIterator<Item = V>) -> Self {
                self.0.children.extend(children.into_iter().map(AnyView::new));
                self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name::new()
            }
        }

        impl View for $name {
            fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
                self.0.build($vertical, cx)
            }
        }
    };
}

flex_widget!(
    /// Lays its children out horizontally (Flutter's `Row`).
    Row,
    false
);
flex_widget!(
    /// Lays its children out vertically (Flutter's `Column`).
    Column,
    true
);

/// Gives its child a share of a [`Row`]'s or [`Column`]'s free space.
struct FlexChild {
    child: AnyView,
    flex: u32,
    fit: FlexFit,
}

impl FlexChild {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.child, self.flex, self.fit, cx);
        todo!()
    }
}

/// Makes its child fill its share of a [`Row`]'s or [`Column`]'s free space (Flutter's
/// `Expanded`: flex 1, tight fit).
pub struct Expanded(FlexChild);

impl Expanded {
    /// Wraps `child` with flex 1.
    pub fn new(child: impl View) -> Self {
        Expanded(FlexChild {
            child: AnyView::new(child),
            flex: 1,
            fit: FlexFit::Tight,
        })
    }

    /// The child's share relative to the other flexible children.
    pub fn flex(mut self, flex: u32) -> Self {
        self.0.flex = flex;
        self
    }
}

impl View for Expanded {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.0.build(cx)
    }
}

/// Lets its child take up to its share of the free space (Flutter's `Flexible`: flex 1, loose
/// fit).
pub struct Flexible(FlexChild);

impl Flexible {
    /// Wraps `child` with flex 1 and a loose fit.
    pub fn new(child: impl View) -> Self {
        Flexible(FlexChild {
            child: AnyView::new(child),
            flex: 1,
            fit: FlexFit::Loose,
        })
    }

    /// The child's share relative to the other flexible children.
    pub fn flex(mut self, flex: u32) -> Self {
        self.0.flex = flex;
        self
    }

    /// Whether the child must fill its share (`Tight`) or may be smaller (`Loose`).
    pub fn fit(mut self, fit: FlexFit) -> Self {
        self.0.fit = fit;
        self
    }
}

impl View for Flexible {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.0.build(cx)
    }
}

/// Empty space taking a share of a [`Row`]'s or [`Column`]'s free space (Flutter's `Spacer`).
pub struct Spacer {
    flex: u32,
}

impl Spacer {
    /// A spacer with flex 1.
    pub fn new() -> Self {
        Spacer { flex: 1 }
    }

    /// The spacer's share relative to the other flexible children.
    pub fn flex(mut self, flex: u32) -> Self {
        self.flex = flex;
        self
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Spacer::new()
    }
}

impl View for Spacer {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        Expanded::new(SizedBox::shrink()).flex(self.flex).build(cx)
    }
}

/// Layers its children, later ones on top (Flutter's `Stack`).
pub struct Stack {
    alignment: Prop<Alignment>,
    fit: Prop<StackFit>,
    children: Vec<AnyView>,
}

impl Stack {
    /// `alignment: TOP_LEFT`, `fit: Loose`, no children.
    pub fn new() -> Self {
        Stack {
            alignment: Prop::Value(Alignment::TOP_LEFT),
            fit: Prop::Value(StackFit::Loose),
            children: Vec::new(),
        }
    }

    /// Where non-positioned children go.
    pub fn alignment(mut self, alignment: impl IntoProp<Alignment>) -> Self {
        self.alignment = alignment.into_prop();
        self
    }

    /// How non-positioned children are constrained.
    pub fn fit(mut self, fit: impl IntoProp<StackFit>) -> Self {
        self.fit = fit.into_prop();
        self
    }

    /// Appends a child.
    pub fn child(mut self, child: impl View) -> Self {
        self.children.push(AnyView::new(child));
        self
    }

    /// Appends children.
    pub fn children<V: View>(mut self, children: impl IntoIterator<Item = V>) -> Self {
        self.children.extend(children.into_iter().map(AnyView::new));
        self
    }
}

impl Default for Stack {
    fn default() -> Self {
        Stack::new()
    }
}

impl View for Stack {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.alignment, self.fit, self.children, cx);
        todo!()
    }
}

/// Places its child in a [`Stack`] by edges and size (Flutter's `Positioned`).
pub struct Positioned {
    child: AnyView,
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
}

impl Positioned {
    /// Wraps `child` with no edges set (set some with the builders).
    pub fn new(child: impl View) -> Self {
        Positioned {
            child: AnyView::new(child),
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
        }
    }

    /// Fills the stack: all four edges at 0 (`Positioned.fill`).
    pub fn fill(child: impl View) -> Self {
        Positioned::new(child)
            .left(0.0)
            .top(0.0)
            .right(0.0)
            .bottom(0.0)
    }

    /// Distance from the stack's left edge.
    pub fn left(mut self, value: f32) -> Self {
        self.left = Some(value);
        self
    }

    /// Distance from the stack's top edge.
    pub fn top(mut self, value: f32) -> Self {
        self.top = Some(value);
        self
    }

    /// Distance from the stack's right edge.
    pub fn right(mut self, value: f32) -> Self {
        self.right = Some(value);
        self
    }

    /// Distance from the stack's bottom edge.
    pub fn bottom(mut self, value: f32) -> Self {
        self.bottom = Some(value);
        self
    }

    /// The child's width.
    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(value);
        self
    }

    /// The child's height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(value);
        self
    }
}

impl View for Positioned {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.child,
            self.left,
            self.top,
            self.right,
            self.bottom,
            self.width,
            self.height,
            cx,
        );
        todo!()
    }
}
