//! Tests for `docs/specs/view/inherited.md`, rules VIEW-INHERIT-01..03.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::Size;
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_view::reactive::signal;
use tantu_view::{AnyView, BuildCx, Dyn, ElementId, Key, Keyed, Provide, View, ViewTree};

type Log = Rc<RefCell<Vec<String>>>;

#[derive(Clone, Debug, PartialEq)]
struct Theme(&'static str);

#[derive(Clone, Debug, PartialEq)]
struct Density(u8);

/// A leaf logging what it inherits as `name: theme/density`.
struct Reader {
    name: &'static str,
    log: Log,
}

impl View for Reader {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let theme = cx.inherit::<Theme>().map_or("none", |t| t.0);
        let density = cx.inherit::<Density>().map_or(0, |d| d.0);
        self.log
            .borrow_mut()
            .push(format!("{}: {theme}/{density}", self.name));
        cx.render(RenderConstrainedBox::sized(Some(10.0), Some(10.0)), [])
    }
}

/// A build-time action on a context (a `provide` call).
type Provider = Box<dyn FnOnce(&mut BuildCx<'_>)>;

/// A column providing values through `BuildCx::provide` on itself.
struct Column {
    provides: Vec<Provider>,
    children: Vec<AnyView>,
}

impl View for Column {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let provides = self.provides;
        let children = self.children;
        cx.region(move |cx| {
            for provide in provides {
                provide(cx);
            }
            let _ = cx.render(RenderFlex::column(), children);
        })
    }
}

fn reader(name: &'static str, log: &Log) -> AnyView {
    AnyView::new(Reader {
        name,
        log: log.clone(),
    })
}

#[test]
fn view_inherit_01_nearest_value() {
    let log: Log = Rc::default();
    let tree = {
        let log = log.clone();
        ViewTree::new(move || Column {
            provides: vec![
                Box::new(|cx| cx.provide(Theme("light"))),
                Box::new(|cx| cx.provide(Density(2))),
                Box::new(|cx| cx.provide(Theme("dark"))),
            ],
            children: vec![
                reader("a", &log),
                AnyView::new(Column {
                    provides: vec![Box::new(|cx| cx.provide(Theme("inner")))],
                    children: vec![reader("b", &log)],
                }),
            ],
        })
    };
    assert_eq!(log.borrow().clone(), ["a: dark/2", "b: inner/2"]);
    let _ = tree;
    // Nothing provided: `None`.
    let log: Log = Rc::default();
    let _tree = {
        let log = log.clone();
        ViewTree::new(move || Reader { name: "c", log })
    };
    assert_eq!(log.borrow().clone(), ["c: none/0"]);
}

#[test]
fn view_inherit_02_provide_view() {
    let log: Log = Rc::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            let tick = signal(0u32);
            let dyn_log = log.clone();
            Column {
                provides: vec![],
                children: vec![
                    AnyView::new(Provide::new(
                        Theme("provided"),
                        Column {
                            provides: vec![],
                            children: vec![
                                reader("inside", &log),
                                AnyView::new(Keyed::new(
                                    "dyn",
                                    Dyn::new(move || {
                                        let _ = tick.get();
                                        Reader {
                                            name: "rebuilt",
                                            log: dyn_log.clone(),
                                        }
                                    }),
                                )),
                            ],
                        },
                    )),
                    reader("sibling", &log),
                    AnyView::new(Keyed::new("tick", Ticker(tick))),
                ],
            }
        })
    };
    assert_eq!(
        log.borrow().clone(),
        [
            "inside: provided/0",
            "rebuilt: provided/0",
            "sibling: none/0"
        ]
    );
    log.borrow_mut().clear();
    // Content rebuilt later under the provider still inherits.
    let ticker = tree.find_key(&Key::from("tick"))[0];
    let tick = tree.ticker_signal(ticker);
    tree.enter(|| tick.set(1));
    tree.layout(
        BoxConstraints::loose(Size::new(100.0, 100.0)),
        &mut NoTextMeasure,
    );
    let mut scene = tantu_view::scene::Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(100.0, 100.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    assert_eq!(log.borrow().clone(), ["rebuilt: provided/0"]);
}

/// Holds a signal so a test can reach it through the tree.
struct Ticker(tantu_view::reactive::Signal<u32>);

impl View for Ticker {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.provide(self.0);
        cx.render(RenderConstrainedBox::sized(Some(1.0), Some(1.0)), [])
    }
}

trait TickerSignal {
    fn ticker_signal(&self, id: ElementId) -> tantu_view::reactive::Signal<u32>;
}

impl TickerSignal for ViewTree {
    fn ticker_signal(&self, id: ElementId) -> tantu_view::reactive::Signal<u32> {
        // A leaf provides on the element being built: its parent.
        let parent = self.parent(id).expect("the ticker has a parent");
        self.inherited_on::<tantu_view::reactive::Signal<u32>>(parent)
            .expect("the ticker provided its signal")
    }
}

#[test]
fn view_inherit_03_removed_values() {
    let log: Log = Rc::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || Column {
            provides: vec![],
            children: vec![AnyView::new(Keyed::new(
                "p",
                Provide::new(Theme("gone"), reader("x", &log)),
            ))],
        })
    };
    let provider = tree.find_key(&Key::from("p"))[0];
    assert_eq!(tree.inherited_on::<Theme>(provider), Some(Theme("gone")));
    tree.remove(provider);
    assert_eq!(tree.inherited_on::<Theme>(provider), None);
}
