//! Tests for `docs/specs/a11y/semantics.md`, rules A11Y-TREE-01..04.

use tantu_a11y::AccessibilityTree;
use tantu_a11y::accesskit::{
    Action, ActionData, ActionRequest, NodeId, Role as AkRole, Toggled as AkToggled, TreeId,
};
use tantu_view::core::{Rect, Size};
use tantu_view::layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_view::scene::Scene;
use tantu_view::{
    AnyView, BuildCx, Dyn, ElementId, FocusOptions, NumericValue, Role, Semantics, SemanticsAction,
    SemanticsActions, Toggled, View, ViewTree,
};

/// A box, or a column of children, with optional semantics.
struct Node {
    size: Size,
    semantics: Option<Semantics>,
    focusable: bool,
    children: Vec<AnyView>,
}

fn leaf(semantics: Option<Semantics>) -> Node {
    Node {
        size: Size::new(100.0, 20.0),
        semantics,
        focusable: false,
        children: Vec::new(),
    }
}

fn column(semantics: Option<Semantics>, children: Vec<AnyView>) -> Node {
    Node {
        size: Size::ZERO,
        semantics,
        focusable: false,
        children,
    }
}

impl View for Node {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = if self.children.is_empty() {
            cx.render(
                RenderConstrainedBox::sized(Some(self.size.width), Some(self.size.height)),
                [],
            )
        } else {
            cx.render(RenderFlex::column(), self.children)
        };
        if let Some(semantics) = self.semantics {
            cx.set_semantics(id, semantics);
        }
        if self.focusable {
            cx.focusable(id, FocusOptions::default());
        }
        id
    }
}

fn sem(role: Role, name: &str) -> Semantics {
    Semantics {
        role,
        name: Some(name.to_owned()),
        ..Default::default()
    }
}

fn frame(tree: &mut ViewTree) {
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::tight(Size::new(300.0, 200.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
}

fn child(tree: &ViewTree, parent: ElementId, index: usize) -> ElementId {
    tree.children(parent)[index]
}

#[test]
fn a11y_tree_01_structure() {
    let mut tree = ViewTree::new(|| {
        column(
            None,
            vec![
                AnyView::new(leaf(Some(sem(Role::Label, "Title")))),
                // An unsemantic column: its semantic descendants re-parent to the window.
                AnyView::new(column(
                    None,
                    vec![
                        AnyView::new(leaf(Some(sem(Role::Button, "A")))),
                        AnyView::new(leaf(None)),
                        AnyView::new(Dyn::new(|| leaf(Some(sem(Role::Button, "B"))))),
                    ],
                )),
                // A semantic group keeps its children.
                AnyView::new(column(
                    Some(sem(Role::List, "L")),
                    vec![
                        AnyView::new(leaf(Some(sem(Role::ListItem, "1")))),
                        AnyView::new(column(
                            None,
                            vec![AnyView::new(leaf(Some(sem(Role::ListItem, "2"))))],
                        )),
                    ],
                )),
            ],
        )
    });
    frame(&mut tree);
    let update = AccessibilityTree::new("app").update(&tree);
    let root_id = update.tree.as_ref().expect("a tree").root;
    let node = |id: NodeId| {
        &update
            .nodes
            .iter()
            .find(|(n, _)| *n == id)
            .expect("present")
            .1
    };
    let window = node(root_id);
    assert_eq!(window.role(), AkRole::Window);
    assert_eq!(window.label(), Some("app"));
    let labels = |ids: &[NodeId]| -> Vec<String> {
        ids.iter()
            .map(|id| node(*id).label().unwrap_or("").to_owned())
            .collect()
    };
    assert_eq!(labels(window.children()), ["Title", "A", "B", "L"]);
    let list = window.children()[3];
    assert_eq!(labels(node(list).children()), ["1", "2"]);
    assert_eq!(update.nodes.len(), 7);
    // Node ids are the elements' ids.
    let col = child(&tree, tree.root(), 0);
    let title = child(&tree, col, 0);
    assert_eq!(window.children()[0], AccessibilityTree::node_id(title));
    assert_eq!(update.focus, root_id, "nothing focused: the window");
}

#[test]
fn a11y_tree_02_properties() {
    let mut tree = ViewTree::new(|| {
        column(
            None,
            vec![
                AnyView::new(leaf(Some(Semantics {
                    role: Role::Button,
                    name: Some("Go".into()),
                    disabled: true,
                    focusable: true,
                    actions: SemanticsActions {
                        click: true,
                        focus: true,
                        ..Default::default()
                    },
                    ..Default::default()
                }))),
                AnyView::new(leaf(Some(Semantics {
                    role: Role::CheckBox,
                    toggled: Some(Toggled::Mixed),
                    ..Default::default()
                }))),
                AnyView::new(leaf(Some(Semantics {
                    role: Role::Slider,
                    value: Some("40".into()),
                    numeric: Some(NumericValue {
                        value: 40.0,
                        min: 0.0,
                        max: 100.0,
                        step: 5.0,
                    }),
                    actions: SemanticsActions {
                        increment: true,
                        decrement: true,
                        ..Default::default()
                    },
                    ..Default::default()
                }))),
                AnyView::new(leaf(Some(Semantics {
                    role: Role::TextInput,
                    value: Some("hello".into()),
                    actions: SemanticsActions {
                        set_value: true,
                        ..Default::default()
                    },
                    ..Default::default()
                }))),
                AnyView::new(leaf(Some(Semantics {
                    role: Role::ListItem,
                    selected: Some(true),
                    actions: SemanticsActions {
                        scroll_into_view: true,
                        ..Default::default()
                    },
                    ..Default::default()
                }))),
            ],
        )
    });
    frame(&mut tree);
    let update = AccessibilityTree::new("app").update(&tree);
    let root_id = update.tree.as_ref().expect("a tree").root;
    let node = |id: NodeId| {
        &update
            .nodes
            .iter()
            .find(|(n, _)| *n == id)
            .expect("present")
            .1
    };
    let kids = node(root_id).children().to_vec();
    let button = node(kids[0]);
    assert_eq!(button.role(), AkRole::Button);
    assert_eq!(button.label(), Some("Go"));
    assert!(button.is_disabled());
    assert!(button.supports_action(Action::Click));
    assert!(button.supports_action(Action::Focus));
    assert!(!button.supports_action(Action::Increment));
    let check = node(kids[1]);
    assert_eq!(check.role(), AkRole::CheckBox);
    assert_eq!(check.toggled(), Some(AkToggled::Mixed));
    assert!(!check.is_disabled());
    let slider = node(kids[2]);
    assert_eq!(slider.role(), AkRole::Slider);
    assert_eq!(slider.value(), Some("40"));
    assert_eq!(slider.numeric_value(), Some(40.0));
    assert_eq!(slider.min_numeric_value(), Some(0.0));
    assert_eq!(slider.max_numeric_value(), Some(100.0));
    assert_eq!(slider.numeric_value_step(), Some(5.0));
    assert!(slider.supports_action(Action::Increment) && slider.supports_action(Action::Decrement));
    let field = node(kids[3]);
    assert_eq!(field.role(), AkRole::TextInput);
    assert_eq!(field.value(), Some("hello"));
    assert!(field.supports_action(Action::SetValue));
    let item = node(kids[4]);
    assert_eq!(item.role(), AkRole::ListItem);
    assert_eq!(item.is_selected(), Some(true));
    assert!(item.supports_action(Action::ScrollIntoView));
}

#[test]
fn a11y_tree_03_bounds_and_focus() {
    let mut tree = ViewTree::new(|| {
        column(
            None,
            vec![
                AnyView::new(leaf(Some(sem(Role::Label, "one")))),
                AnyView::new(Node {
                    focusable: true,
                    ..leaf(Some(Semantics {
                        focusable: true,
                        ..sem(Role::Button, "two")
                    }))
                }),
                AnyView::new(Node {
                    size: Size::ZERO,
                    ..leaf(Some(sem(Role::Label, "empty")))
                }),
            ],
        )
    });
    frame(&mut tree);
    let col = child(&tree, tree.root(), 0);
    let two = child(&tree, col, 1);
    let mut builder = AccessibilityTree::new("app");
    let update = builder.update(&tree);
    let root_id = update.tree.as_ref().expect("a tree").root;
    let node = |id: NodeId| {
        &update
            .nodes
            .iter()
            .find(|(n, _)| *n == id)
            .expect("present")
            .1
    };
    let kids = node(root_id).children().to_vec();
    // The column centers 100 px children in 300 px.
    let b = node(kids[0]).bounds().expect("bounds");
    assert_eq!((b.x0, b.y0, b.x1, b.y1), (100.0, 0.0, 200.0, 20.0));
    let b = node(kids[1]).bounds().expect("bounds");
    assert_eq!((b.y0, b.y1), (20.0, 40.0));
    let b = node(kids[2])
        .bounds()
        .expect("a zero-sized node still has bounds");
    assert_eq!((b.x1 - b.x0, b.y1 - b.y0), (0.0, 0.0));
    assert!(node(kids[0]).transform().is_none());
    let window = node(root_id).bounds().expect("bounds");
    assert_eq!((window.x1, window.y1), (300.0, 200.0));
    assert_eq!(update.focus, root_id);
    assert!(tree.focus(two));
    let update = builder.update(&tree);
    assert_eq!(update.focus, AccessibilityTree::node_id(two));
    let _ = Rect::ZERO;
}

#[test]
fn a11y_tree_04_map_action() {
    let mut tree = ViewTree::new(|| leaf(Some(sem(Role::Button, "Go"))));
    frame(&mut tree);
    let button = child(&tree, tree.root(), 0);
    let request = |action: Action, data: Option<ActionData>| ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: AccessibilityTree::node_id(button),
        data,
    };
    assert_eq!(
        AccessibilityTree::map_action(&request(Action::Click, None)),
        Some((button, SemanticsAction::Click))
    );
    assert_eq!(
        AccessibilityTree::map_action(&request(Action::Focus, None)),
        Some((button, SemanticsAction::Focus))
    );
    assert_eq!(
        AccessibilityTree::map_action(&request(Action::Increment, None)),
        Some((button, SemanticsAction::Increment))
    );
    assert_eq!(
        AccessibilityTree::map_action(&request(Action::Decrement, None)),
        Some((button, SemanticsAction::Decrement))
    );
    assert_eq!(
        AccessibilityTree::map_action(&request(
            Action::SetValue,
            Some(ActionData::Value("x".into()))
        )),
        Some((button, SemanticsAction::SetValue("x".into())))
    );
    assert_eq!(
        AccessibilityTree::map_action(&request(Action::ScrollIntoView, None)),
        Some((button, SemanticsAction::ScrollIntoView))
    );
    assert_eq!(
        AccessibilityTree::map_action(&request(Action::SetValue, None)),
        None
    );
    assert_eq!(
        AccessibilityTree::map_action(&request(Action::Blur, None)),
        None
    );
    // The window's node is not an element.
    let update = AccessibilityTree::new("app").update(&tree);
    let root_id = update.tree.as_ref().expect("a tree").root;
    assert_eq!(
        AccessibilityTree::map_action(&ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: root_id,
            data: None,
        }),
        None
    );
}

#[test]
fn a11y_tree_05_roles_properties_and_expand_collapse_requests() {
    let tree = ViewTree::new(|| {
        leaf(Some(Semantics {
            role: tantu_view::Role::TreeItem,
            expanded: Some(false),
            level: Some(3),
            position_in_set: Some(2),
            size_of_set: Some(5),
            actions: SemanticsActions {
                expand: true,
                ..Default::default()
            },
            ..Default::default()
        }))
    });
    let update = AccessibilityTree::new("tree").update(&tree);
    let item = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == AkRole::TreeItem)
        .expect("tree item role");
    assert_eq!(
        (
            item.1.is_expanded(),
            item.1.level(),
            item.1.position_in_set(),
            item.1.size_of_set()
        ),
        (Some(false), Some(3), Some(2), Some(5))
    );
    assert!(item.1.supports_action(Action::Expand));
    for (action, expected) in [
        (Action::Expand, SemanticsAction::Expand),
        (Action::Collapse, SemanticsAction::Collapse),
    ] {
        let request = ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: item.0,
            data: None,
        };
        assert_eq!(
            AccessibilityTree::map_action(&request),
            Some((ElementId::from_raw(item.0.0).unwrap(), expected))
        );
    }
}

#[test]
fn a11y_tree_07_tab_roles() {
    let semantics = |role| {
        Some(Semantics {
            role,
            ..Default::default()
        })
    };
    let tree = ViewTree::new(move || {
        column(
            semantics(tantu_view::Role::TabList),
            vec![
                AnyView::new(leaf(semantics(tantu_view::Role::Tab))),
                AnyView::new(leaf(semantics(tantu_view::Role::TabPanel))),
            ],
        )
    });
    let update = AccessibilityTree::new("tabs").update(&tree);
    for role in [AkRole::TabList, AkRole::Tab, AkRole::TabPanel] {
        assert!(
            update.nodes.iter().any(|(_, n)| n.role() == role),
            "{role:?} is mapped"
        );
    }
}

#[test]
fn a11y_tree_08_splitter_role() {
    let tree = ViewTree::new(move || {
        leaf(Some(Semantics {
            role: tantu_view::Role::Splitter,
            ..Default::default()
        }))
    });
    let update = AccessibilityTree::new("split").update(&tree);
    assert!(
        update
            .nodes
            .iter()
            .any(|(_, n)| n.role() == AkRole::Splitter),
        "Splitter is mapped"
    );
}
