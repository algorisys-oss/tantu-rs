//! Tests for `docs/specs/widgets/dock-layout.md`, rules WIDGETS-DOCK-01..12.

use std::collections::HashSet;

use tantu_view::layout::Axis;
use tantu_widgets::{
    DockError, DockGroupId, DockLayout, DockLoadError, DockNode, DockSide, DockTarget, DockView,
    PanelId,
};

fn p(id: &str) -> PanelId {
    PanelId::new(id)
}

fn tabs(panels: &[&str]) -> DockNode {
    DockNode::tabs(panels.iter().copied())
}

fn tabs_at(panels: &[&str], active: usize) -> DockNode {
    DockNode::Tabs {
        panels: panels.iter().map(|s| p(s)).collect(),
        active,
    }
}

/// explorer+search | (editor / terminal+problems, problems active).
fn ide() -> DockLayout {
    DockLayout::new(DockNode::row([
        (0.2, tabs(&["explorer", "search"])),
        (
            0.8,
            DockNode::column([
                (0.7, tabs(&["editor"])),
                (0.3, tabs_at(&["terminal", "problems"], 1)),
            ]),
        ),
    ]))
}

fn group(layout: &DockLayout, panel: &str) -> DockGroupId {
    layout.group_of(&p(panel)).expect("panel in layout")
}

fn names(panels: &[PanelId]) -> Vec<&str> {
    panels.iter().map(PanelId::as_str).collect()
}

/// The root split's id and fractions.
fn root_split(layout: &DockLayout) -> (tantu_widgets::DockSplitId, Vec<f32>) {
    match layout.root() {
        Some(DockView::Split { id, fractions, .. }) => (id, fractions.to_vec()),
        other => panic!("root is not a split: {other:?}"),
    }
}

fn assert_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() < 1e-5, "{actual:?} vs {expected:?}");
    }
}

/// Asserts WIDGETS-DOCK-02's normal form.
fn assert_normal(layout: &DockLayout) {
    fn walk(view: &DockView<'_>, parent: Option<Axis>, seen: &mut HashSet<PanelId>) {
        match view {
            DockView::Tabs { panels, active, .. } => {
                assert!(!panels.is_empty(), "empty group");
                assert!(*active < panels.len(), "active out of range");
                for panel in *panels {
                    assert!(seen.insert(panel.clone()), "{panel:?} twice");
                }
            }
            DockView::Split {
                axis,
                fractions,
                children,
                ..
            } => {
                assert!(
                    children.len() >= 2,
                    "split with {} children",
                    children.len()
                );
                assert_eq!(fractions.len(), children.len());
                assert_ne!(Some(*axis), parent, "same-axis child split");
                assert!(fractions.iter().all(|f| *f >= 0.0 && f.is_finite()));
                let sum: f32 = fractions.iter().sum();
                assert!((sum - 1.0).abs() < 1e-5, "fractions sum to {sum}");
                for child in children {
                    walk(child, Some(*axis), seen);
                }
            }
        }
    }
    if let Some(root) = layout.root() {
        walk(&root, None, &mut HashSet::new());
        assert!(!layout.is_empty());
    } else {
        assert!(layout.is_empty());
        assert!(layout.panels().is_empty());
    }
}

#[test]
fn widgets_dock_01_construction_normalizes() {
    // Duplicates keep their first position in tree order.
    let l = DockLayout::new(DockNode::row([
        (0.5, tabs(&["a"])),
        (0.5, tabs(&["a", "b", "b"])),
    ]));
    assert_eq!(names(&l.panels()), ["a", "b"]);
    assert_eq!(
        l,
        DockLayout::new(DockNode::row([(0.5, tabs(&["a"])), (0.5, tabs(&["b"]))]))
    );
    // Empty groups are removed, with their fractions; a one-child split becomes its child.
    let l = DockLayout::new(DockNode::row([(0.5, tabs(&[])), (0.5, tabs(&["a"]))]));
    assert_eq!(l, DockLayout::new(tabs(&["a"])));
    let l = DockLayout::new(DockNode::row([
        (0.2, tabs(&[])),
        (0.3, tabs(&["a"])),
        (0.5, tabs(&["b"])),
    ]));
    assert_close(&root_split(&l).1, &[0.375, 0.625]);
    // Active is clamped.
    let l = DockLayout::new(tabs_at(&["a", "b"], 5));
    assert_eq!(l.active(group(&l, "a")), Some(&p("b")));
    // Fractions are cleaned as LAYOUT-SPLIT-01.
    let l = DockLayout::new(DockNode::row([
        (-1.0, tabs(&["a"])),
        (f32::NAN, tabs(&["b"])),
        (2.0, tabs(&["c"])),
    ]));
    assert_close(&root_split(&l).1, &[0.0, 0.0, 1.0]);
    let l = DockLayout::new(DockNode::row([(0.0, tabs(&["a"])), (0.0, tabs(&["b"]))]));
    assert_close(&root_split(&l).1, &[0.5, 0.5]);
    // No panels: the empty layout.
    let l = DockLayout::new(DockNode::column([(1.0, tabs(&[])), (1.0, tabs(&[]))]));
    assert!(l.is_empty());
    assert_eq!(l, DockLayout::empty());
    assert_eq!(DockLayout::default(), DockLayout::empty());
    assert_eq!(l.root(), None);
}

#[test]
fn widgets_dock_02_normal_form() {
    // A same-axis child split is flattened, its fractions scaled by its own.
    let l = DockLayout::new(DockNode::row([
        (0.5, tabs(&["a"])),
        (
            0.5,
            DockNode::row([(0.5, tabs(&["b"])), (0.5, tabs(&["c"]))]),
        ),
    ]));
    assert_normal(&l);
    assert_close(&root_split(&l).1, &[0.5, 0.25, 0.25]);
    assert_eq!(l.groups().len(), 3);
    // A failing operation changes nothing.
    let mut l = ide();
    let before = l.clone();
    let ids = (l.groups(), root_split(&l).0);
    assert!(l.close(&p("nope")).is_err());
    assert!(
        l.move_panel(&p("nope"), DockTarget::Edge(DockSide::Left))
            .is_err()
    );
    assert_eq!(l, before);
    assert_eq!((l.groups(), root_split(&l).0), ids);

    // Every operation keeps the normal form, over a deterministic random sequence.
    let mut l = ide();
    let mut seed = 0x2545_f491_u32;
    let mut next = |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed as usize % n.max(1)
    };
    let sides = [
        DockSide::Left,
        DockSide::Right,
        DockSide::Top,
        DockSide::Bottom,
    ];
    for step in 0..400 {
        let panels = l.panels();
        let groups = l.groups();
        let panel = if panels.is_empty() || next(5) == 0 {
            p(&format!("new{step}"))
        } else {
            panels[next(panels.len())].clone()
        };
        let target = match (groups.is_empty(), next(3)) {
            (true, _) | (false, 0) => DockTarget::Edge(sides[next(4)]),
            (false, 1) => DockTarget::Tabs {
                group: groups[next(groups.len())],
                index: next(4),
            },
            _ => DockTarget::Beside {
                group: groups[next(groups.len())],
                side: sides[next(4)],
            },
        };
        let _ = match next(4) {
            0 => l.close(&panel),
            1 => l.activate(&panel),
            _ => l.add(panel, target),
        };
        assert_normal(&l);
    }
}

#[test]
fn widgets_dock_03_ids() {
    let mut l = DockLayout::new(DockNode::row([
        (0.5, tabs(&["a"])),
        (
            0.5,
            DockNode::column([(0.5, tabs(&["b"])), (0.5, tabs(&["c"]))]),
        ),
    ]));
    let (ga, gc) = (group(&l, "a"), group(&l, "c"));
    let mut seen: HashSet<DockGroupId> = l.groups().into_iter().collect();
    // c's group survives its parent split being replaced by it.
    l.close(&p("b")).expect("b is there");
    assert_eq!(group(&l, "c"), gc);
    assert_eq!(group(&l, "a"), ga);
    // Panels moving in keep the group's id; a new group gets a fresh one.
    l.add(
        "d",
        DockTarget::Tabs {
            group: ga,
            index: 9,
        },
    )
    .expect("ga exists");
    assert_eq!(group(&l, "d"), ga);
    l.add(
        "e",
        DockTarget::Beside {
            group: gc,
            side: DockSide::Bottom,
        },
    )
    .expect("gc exists");
    let ge = group(&l, "e");
    assert!(seen.insert(ge), "a fresh id");
    // A clone keeps ids.
    let copy = l.clone();
    assert_eq!(copy.groups(), l.groups());
    // Equality ignores ids: re-creating the same shape with a new group compares equal.
    let mut l = DockLayout::new(DockNode::row([(0.5, tabs(&["a"])), (0.5, tabs(&["b"]))]));
    let ga = group(&l, "a");
    let gb = group(&l, "b");
    l.move_panel(
        &p("a"),
        DockTarget::Beside {
            group: gb,
            side: DockSide::Left,
        },
    )
    .expect("moves");
    assert_ne!(group(&l, "a"), ga);
    assert_eq!(
        l,
        DockLayout::new(DockNode::row([(0.5, tabs(&["a"])), (0.5, tabs(&["b"]))]))
    );
}

#[test]
fn widgets_dock_04_queries() {
    let mut l = ide();
    assert_eq!(
        names(&l.panels()),
        ["explorer", "search", "editor", "terminal", "problems"]
    );
    let groups = l.groups();
    assert_eq!(groups.len(), 3);
    assert_eq!(group(&l, "problems"), groups[2]);
    let (panels, active) = l.group(groups[2]).expect("group");
    assert_eq!((names(panels), active), (vec!["terminal", "problems"], 1));
    assert_eq!(l.active(groups[2]), Some(&p("problems")));
    assert!(l.contains(&p("editor")));
    assert!(!l.contains(&p("nope")));
    assert_eq!(l.group_of(&p("nope")), None);
    match l.root() {
        Some(DockView::Split {
            axis: Axis::Horizontal,
            fractions,
            children,
            ..
        }) => {
            assert_close(fractions, &[0.2, 0.8]);
            assert!(matches!(children[0], DockView::Tabs { active: 0, .. }));
            assert!(matches!(
                children[1],
                DockView::Split {
                    axis: Axis::Vertical,
                    ..
                }
            ));
        }
        other => panic!("{other:?}"),
    }
    // A removed group's id is unknown.
    let editor = group(&l, "editor");
    l.close(&p("editor")).expect("closes");
    assert_eq!(l.group(editor), None);
    assert_eq!(l.active(editor), None);
}

#[test]
fn widgets_dock_05_activate() {
    let mut l = ide();
    let g = group(&l, "search");
    l.activate(&p("search")).expect("known");
    assert_eq!(l.active(g), Some(&p("search")));
    let before = l.clone();
    assert_eq!(l.activate(&p("search")), Ok(()));
    assert_eq!(l, before);
    assert_eq!(
        l.activate(&p("nope")),
        Err(DockError::UnknownPanel(p("nope")))
    );
}

#[test]
fn widgets_dock_06_close() {
    // Closing the active tab activates the next one, or the previous when it was last.
    let mut l = DockLayout::new(tabs_at(&["a", "b", "c"], 1));
    let g = group(&l, "a");
    l.close(&p("b")).expect("closes");
    assert_eq!(l.active(g), Some(&p("c")));
    l.close(&p("c")).expect("closes");
    assert_eq!(l.active(g), Some(&p("a")));
    // Closing an inactive tab keeps the active one.
    let mut l = DockLayout::new(tabs_at(&["a", "b", "c"], 2));
    l.close(&p("a")).expect("closes");
    assert_eq!(l.active(group(&l, "b")), Some(&p("c")));
    // An emptied group's share goes to its siblings in proportion.
    let mut l = DockLayout::new(DockNode::row([
        (0.2, tabs(&["a"])),
        (0.3, tabs(&["b"])),
        (0.5, tabs(&["c"])),
    ]));
    l.close(&p("a")).expect("closes");
    assert_close(&root_split(&l).1, &[0.375, 0.625]);
    // Down to one group, then empty.
    l.close(&p("b")).expect("closes");
    assert_eq!(l, DockLayout::new(tabs(&["c"])));
    l.close(&p("c")).expect("closes");
    assert!(l.is_empty());
    assert_eq!(l.close(&p("c")), Err(DockError::UnknownPanel(p("c"))));
}

#[test]
fn widgets_dock_07_tabs_target() {
    let mut l = ide();
    let editor = group(&l, "editor");
    l.move_panel(
        &p("terminal"),
        DockTarget::Tabs {
            group: editor,
            index: 0,
        },
    )
    .expect("moves");
    assert_eq!(
        l.group(editor).map(|(p, a)| (names(p), a)),
        Some((vec!["terminal", "editor"], 0))
    );
    assert_eq!(l.active(group(&l, "problems")), Some(&p("problems")));
    // Within a group: the index counts the other tabs.
    let mut l = DockLayout::new(tabs(&["a", "b", "c"]));
    let g = group(&l, "a");
    l.move_panel(&p("a"), DockTarget::Tabs { group: g, index: 2 })
        .expect("moves");
    assert_eq!(
        l.group(g).map(|(p, a)| (names(p), a)),
        Some((vec!["b", "c", "a"], 2))
    );
    l.move_panel(
        &p("b"),
        DockTarget::Tabs {
            group: g,
            index: 99,
        },
    )
    .expect("moves");
    assert_eq!(names(l.group(g).expect("g").0), ["c", "a", "b"]);
    // A new panel goes in at the index and becomes active.
    l.add("d", DockTarget::Tabs { group: g, index: 1 })
        .expect("adds");
    assert_eq!(
        l.group(g).map(|(p, a)| (names(p), a)),
        Some((vec!["c", "d", "a", "b"], 1))
    );
    // An emptied source group is removed.
    let mut l = DockLayout::new(DockNode::row([(0.5, tabs(&["a"])), (0.5, tabs(&["b"]))]));
    let gb = group(&l, "b");
    l.move_panel(
        &p("a"),
        DockTarget::Tabs {
            group: gb,
            index: 0,
        },
    )
    .expect("moves");
    assert_eq!(l, DockLayout::new(tabs(&["a", "b"])));
    assert_eq!(l.groups(), [gb]);
}

#[test]
fn widgets_dock_08_beside_target() {
    // Along the parent split's axis: inserted next to the group, halving its fraction.
    let mut l = DockLayout::new(DockNode::row([(0.4, tabs(&["a"])), (0.6, tabs(&["b"]))]));
    let gb = group(&l, "b");
    l.add(
        "c",
        DockTarget::Beside {
            group: gb,
            side: DockSide::Right,
        },
    )
    .expect("adds");
    assert_eq!(
        l,
        DockLayout::new(DockNode::row([
            (0.4, tabs(&["a"])),
            (0.3, tabs(&["b"])),
            (0.3, tabs(&["c"])),
        ]))
    );
    // Across it: the group becomes a split of the two, the new group first for Top.
    l.add(
        "d",
        DockTarget::Beside {
            group: gb,
            side: DockSide::Top,
        },
    )
    .expect("adds");
    assert_eq!(
        l,
        DockLayout::new(DockNode::row([
            (0.4, tabs(&["a"])),
            (
                0.3,
                DockNode::column([(0.5, tabs(&["d"])), (0.5, tabs(&["b"]))]),
            ),
            (0.3, tabs(&["c"])),
        ]))
    );
    // A root group becomes a split; the old group keeps its id.
    let mut l = DockLayout::new(tabs(&["a", "b"]));
    let g = group(&l, "a");
    l.move_panel(
        &p("a"),
        DockTarget::Beside {
            group: g,
            side: DockSide::Bottom,
        },
    )
    .expect("moves");
    assert_eq!(
        l,
        DockLayout::new(DockNode::column([(0.5, tabs(&["b"])), (0.5, tabs(&["a"]))]))
    );
    assert_eq!(group(&l, "b"), g);
    // A group's only panel beside its own group: nothing changes.
    let mut l = DockLayout::new(DockNode::row([(0.5, tabs(&["a"])), (0.5, tabs(&["b"]))]));
    let before = l.clone();
    let ga = group(&l, "a");
    assert_eq!(
        l.move_panel(
            &p("a"),
            DockTarget::Beside {
                group: ga,
                side: DockSide::Right
            }
        ),
        Err(DockError::NoOp)
    );
    assert_eq!(l, before);
}

#[test]
fn widgets_dock_09_edge_target_and_errors() {
    let mut l = DockLayout::new(DockNode::row([(0.5, tabs(&["a"])), (0.5, tabs(&["b"]))]));
    l.add("c", DockTarget::Edge(DockSide::Left)).expect("adds");
    assert_eq!(
        l,
        DockLayout::new(DockNode::row([
            (0.25, tabs(&["c"])),
            (0.375, tabs(&["a"])),
            (0.375, tabs(&["b"])),
        ]))
    );
    l.add("d", DockTarget::Edge(DockSide::Bottom))
        .expect("adds");
    assert_eq!(
        l,
        DockLayout::new(DockNode::column([
            (
                0.75,
                DockNode::row([
                    (0.25, tabs(&["c"])),
                    (0.375, tabs(&["a"])),
                    (0.375, tabs(&["b"])),
                ]),
            ),
            (0.25, tabs(&["d"])),
        ]))
    );
    // An empty layout: an edge creates the root group; a group target is unknown.
    let mut empty = DockLayout::empty();
    let gone = group(&l, "d");
    assert_eq!(
        empty.add(
            "x",
            DockTarget::Tabs {
                group: gone,
                index: 0
            }
        ),
        Err(DockError::UnknownGroup(gone))
    );
    empty
        .add("x", DockTarget::Edge(DockSide::Right))
        .expect("adds");
    assert_eq!(empty, DockLayout::new(tabs(&["x"])));
    // A removed group is unknown; moving an unknown panel fails; adding a known one moves it.
    l.close(&p("d")).expect("closes");
    assert_eq!(
        l.add(
            "y",
            DockTarget::Beside {
                group: gone,
                side: DockSide::Left
            }
        ),
        Err(DockError::UnknownGroup(gone))
    );
    assert_eq!(
        l.move_panel(&p("nope"), DockTarget::Edge(DockSide::Top)),
        Err(DockError::UnknownPanel(p("nope")))
    );
    let ga = group(&l, "a");
    l.add(
        "c",
        DockTarget::Tabs {
            group: ga,
            index: 0,
        },
    )
    .expect("moves");
    assert_eq!(names(l.group(ga).expect("ga").0), ["c", "a"]);
    assert_eq!(l.panels().len(), 3);
}

#[test]
fn widgets_dock_10_set_fractions() {
    let mut l = DockLayout::new(DockNode::row([
        (0.2, tabs(&["a"])),
        (0.3, tabs(&["b"])),
        (0.5, tabs(&["c"])),
    ]));
    let (split, _) = root_split(&l);
    l.set_fractions(split, &[1.0, 1.0, 2.0]).expect("known");
    assert_close(&root_split(&l).1, &[0.25, 0.25, 0.5]);
    // Missing entries count as an equal share; bad ones as 0.
    l.set_fractions(split, &[1.0]).expect("known");
    assert_close(&root_split(&l).1, &[0.6, 0.2, 0.2]);
    l.set_fractions(split, &[f32::NAN, -2.0, 1.0])
        .expect("known");
    assert_close(&root_split(&l).1, &[0.0, 0.0, 1.0]);
    // A split removed by an operation is unknown.
    l.close(&p("a")).expect("closes");
    l.close(&p("b")).expect("closes");
    assert_eq!(
        l.set_fractions(split, &[1.0]),
        Err(DockError::UnknownSplit(split))
    );
}

const IDE_TEXT: &str = "row(0.2 tabs(0 \"explorer\" \"search\") 0.8 column(0.7 tabs(0 \"editor\") 0.3 tabs(1 \"terminal\" \"problems\")))";

#[test]
fn widgets_dock_11_save() {
    assert_eq!(ide().save(), format!("tantu-dock 1\n{IDE_TEXT}\n"));
    assert_eq!(DockLayout::empty().save(), "tantu-dock 1\nempty\n");
    // Quotes and backslashes are escaped.
    let l = DockLayout::new(tabs(&["a\"b\\c", "é ✓"]));
    assert_eq!(l.save(), "tantu-dock 1\ntabs(0 \"a\\\"b\\\\c\" \"é ✓\")\n");
    // Round trips, including fractions produced by operations.
    let mut l = ide();
    l.add("x", DockTarget::Edge(DockSide::Left)).expect("adds");
    l.move_panel(
        &p("search"),
        DockTarget::Beside {
            group: group(&l, "editor"),
            side: DockSide::Right,
        },
    )
    .expect("moves");
    for layout in [
        ide(),
        l,
        DockLayout::new(tabs(&["a\"b\\c", "é ✓"])),
        DockLayout::empty(),
    ] {
        let text = layout.save();
        assert_eq!(DockLayout::load(&text, |_| true), Ok(layout), "{text}");
    }
}

#[test]
fn widgets_dock_12_load() {
    // Whitespace between items and a missing final newline are fine.
    let text = "tantu-dock 1\n  row( 0.5\n tabs(0 \"a\")  0.5 tabs( 0 \"b\" ) )";
    assert_eq!(
        DockLayout::load(text, |_| true),
        Ok(DockLayout::new(DockNode::row([
            (0.5, tabs(&["a"])),
            (0.5, tabs(&["b"]))
        ])))
    );
    // The version line.
    assert_eq!(DockLayout::load("", |_| true), Err(DockLoadError::Version));
    assert_eq!(
        DockLayout::load("tantu-dock 2\nempty\n", |_| true),
        Err(DockLoadError::Version)
    );
    // Syntax errors report the offset and what was expected.
    let syntax = |text: &str| match DockLayout::load(text, |_| true) {
        Err(DockLoadError::Syntax { offset, expected }) => (offset, expected),
        other => panic!("{text:?}: {other:?}"),
    };
    assert_eq!(syntax("tantu-dock 1\nfoo"), (13, "node"));
    assert_eq!(syntax("tantu-dock 1\nrow(x"), (17, "fraction"));
    assert_eq!(syntax("tantu-dock 1\ntabs(0 a)"), (20, "panel"));
    assert_eq!(syntax("tantu-dock 1\nempty x"), (19, "end of input"));
    let open = "tantu-dock 1\nrow(0.5 tabs(0 \"a\")";
    assert_eq!(syntax(open), (open.len(), ")"));
    // Panels the app no longer has are dropped, then the layout is normalized.
    let text = format!("tantu-dock 1\n{IDE_TEXT}\n");
    let kept = DockLayout::load(&text, |panel| {
        !["search", "editor"].contains(&panel.as_str())
    });
    assert_eq!(
        kept,
        Ok(DockLayout::new(DockNode::row([
            (0.2, tabs(&["explorer"])),
            (0.8, tabs_at(&["terminal", "problems"], 1)),
        ])))
    );
    // Repairs rather than errors: active, duplicates, one-child splits, empty groups.
    assert_eq!(
        DockLayout::load(
            "tantu-dock 1\nrow(1 tabs(7 \"a\" \"b\" \"a\") 1 tabs(0))",
            |_| true
        ),
        Ok(DockLayout::new(tabs_at(&["a", "b"], 1)))
    );
    // No input panics: deep nesting, truncated text, stray escapes, multi-byte characters.
    let deep = format!("tantu-dock 1\n{}", "row(1 ".repeat(1000));
    assert!(DockLayout::load(&deep, |_| true).is_err());
    let full = DockLayout::new(tabs(&["é\\\"", "b"])).save();
    for end in 0..=full.len() {
        if full.is_char_boundary(end) {
            let _ = DockLayout::load(&full[..end], |_| true);
        }
    }
    for text in [
        "tantu-dock 1\ntabs(0 \"a\\",
        "tantu-dock 1\ntabs(99999999999999999999 \"a\")",
        "tantu-dock 1\nrow(1e40 tabs(0 \"a\") -inf tabs(0 \"b\"))",
        "tantu-dock 1\nrow()",
        "tantu-dock 1\n)",
        "tantu-dock 1",
    ] {
        let _ = DockLayout::load(text, |_| true);
    }
}
