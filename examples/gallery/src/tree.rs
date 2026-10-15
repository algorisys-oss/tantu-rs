//! Gallery and kitchen-sink TreeView examples (GALLERY-20..21).
use std::collections::BTreeSet;
use std::rc::Rc;
use tantu::prelude::*;
use tantu::view::Keyed;

struct Source {
    generated: bool,
}
impl TreeDataSource for Source {
    fn root_count(&self) -> usize {
        if self.generated { 3 } else { 1 }
    }
    fn root(&self, index: usize) -> Option<TreeNodeId> {
        [0, 9, 12]
            .get(index)
            .filter(|_| index < self.root_count())
            .map(|n| TreeNodeId(*n))
    }
    fn child_count(&self, node: TreeNodeId) -> usize {
        match node.0 {
            0 | 1 | 9 => 2,
            12 if self.generated => 5000,
            _ => 0,
        }
    }
    fn child(&self, node: TreeNodeId, index: usize) -> Option<TreeNodeId> {
        if index >= self.child_count(node) {
            return None;
        }
        match node.0 {
            0 => Some(TreeNodeId([1, 4][index])),
            1 => Some(TreeNodeId([2, 3][index])),
            9 => Some(TreeNodeId([10, 11][index])),
            12 => Some(TreeNodeId(100 + index as u64)),
            _ => None,
        }
    }
    fn label(&self, node: TreeNodeId) -> String {
        match node.0 {
            0 => "Workspace".into(),
            1 => "src".into(),
            2 => "main.rs".into(),
            3 => "lib.rs".into(),
            4 => "README.md".into(),
            9 => "Dependencies".into(),
            10 => "tantu".into(),
            11 => "tracing".into(),
            12 => "Generated".into(),
            n if n >= 100 => format!("Generated {}", n - 100),
            _ => String::new(),
        }
    }
}
fn selected(value: Option<TreeNodeId>) -> String {
    value.map_or_else(|| "none".into(), |n| n.0.to_string())
}

pub(super) fn section() -> impl View {
    let selection = signal(None);
    let expanded = signal(BTreeSet::from([TreeNodeId(0)]));
    let activated = signal(None);
    Column::new().cross_axis_alignment(CrossAxisAlignment::Start).spacing(8.0)
        .child(Text::new("Arrows navigate and expand; Space toggles; Enter activates. Scroll the Generated branch."))
        .child(SizedBox::new(680.0,320.0).child(Keyed::new("tree",TreeView::new(Rc::new(Source{generated:true})).selection(selection).expanded(expanded).on_activate(move |id|activated.set(Some(id))))))
        .child(Keyed::new("tree-selection",Text::new(move ||format!("Selected: {}",selected(selection.get())))))
        .child(Keyed::new("tree-expanded",Text::new(move ||format!("Expanded: {}",expanded.with(BTreeSet::len)))))
        .child(Keyed::new("tree-activated",Text::new(move ||format!("Activated: {}",selected(activated.get())))))
}
pub(super) fn sink() -> impl View {
    let selection = signal(None);
    let expanded = signal(BTreeSet::from([TreeNodeId(0)]));
    let activated = signal(None);
    let body = Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new(
            "Arrows navigate and expand; Space toggles; Enter activates.",
        ))
        .child(
            SizedBox::new(600.0, 240.0).child(Keyed::new(
                "sink-tree_view",
                TreeView::new(Rc::new(Source { generated: false }))
                    .selection(selection)
                    .expanded(expanded)
                    .on_activate(move |id| activated.set(Some(id))),
            )),
        )
        .child(Keyed::new(
            "sink-tree-activated",
            Text::new(move || format!("Activated: {}", selected(activated.get()))),
        ));
    super::card(
        "tree",
        "Tree view",
        body,
        super::echo(move || {
            format!(
                "Selected: {}, expanded: {}",
                selected(selection.get()),
                expanded.with(BTreeSet::len)
            )
        }),
    )
}
