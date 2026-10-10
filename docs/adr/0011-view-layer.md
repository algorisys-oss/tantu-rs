# 0011. The view layer: run-once components over a retained element arena

- **Status:** Accepted. The reactivity model (point 1) is the user's choice (2026-10-10). The
  rest was decided by the agent while the user was away, at their request ("decide and mark for
  review"); review it.
- **Date:** 2026-10-10
- **Related:** [ADR 0001](0001-retained-tree-and-fine-grained-reactivity.md) (retained tree and
  signals), [ADR 0002](0002-flutter-structure-and-layout-protocol.md),
  [ADR 0003](0003-scene-as-the-renderer-contract.md), [ADR 0007](0007-using-disposed-reactive-handles.md),
  [ADR 0009](0009-layout-tree-in-tantu-layout.md) (layout tree), [ADR 0010](0010-text-measurement-in-layout.md)

## Context

ADR 0001 chose a retained element tree driven by fine-grained signals; AGENTS.md says a signal
change marks only its dependent elements dirty, with no whole-subtree `setState` rebuild by
default, and that reactive props accept a value or a closure. ADR 0009 put the layout half of
the render tree in `tantu-layout`. What remains is the shape of `tantu-view`: what a view is,
what an element is, how signals reach elements, how dynamic content (conditionals, lists) works,
and how painting produces a Scene.

Two facts from `tantu-reactive` constrain it: effects run synchronously, in the middle of the
signal write that triggers them (REACTIVE-SIG-13), and every node belongs to the owner current
when it was created and is disposed with it (REACTIVE-SIG-18, -19).

## Decision

1. **Components run once** (the user's choice). A component is a plain function returning
   `impl View`; it runs once, when its views are built. Reactive props are closures; each
   becomes an effect that updates one property of one element when the signals it reads
   change. Nothing re-runs a component.
2. **Views are one-shot builders.** `View` is a trait with `fn build(self, cx: &mut BuildCx) ->
   ElementId`; a view is consumed when it builds its element. `AnyView` erases the type for
   child lists. Views are cheap values, as in Flutter, but they don't live on: the element tree
   is the retained state.
3. **Elements live in an arena in a `ViewTree`, one per window.** The tree owns the window's
   reactive `Runtime`, its element arena and its `LayoutTree`. Element ids are Scene element
   ids (`tantu_scene::ElementId`, built from the arena id's bits), so Scene commands, AccessKit
   nodes and elements share one id.
4. **Two kinds of elements.** A *render element* owns exactly one layout node (a `RenderBox`)
   and a paint behavior. A *region element* owns no layout node: it holds dynamic content, and
   its children take its place in the nearest render ancestor's layout child list (like
   Solid's fragments). That is how a reactive list inside a `Column` produces children of the
   column's flex, not a nested box.
5. **Each element has a reactive scope.** Effects for an element's reactive props, and
   everything built inside it, belong to its scope; removing an element disposes its scope,
   so its effects stop and its cleanups run.
6. **Effects don't touch the tree; they queue updates.** A reactive prop's effect computes the
   new value and pushes an update (element, change) onto the tree's queue and flags that a
   frame is needed. `ViewTree::frame` applies the queue (each change updates a layout object
   through `LayoutTree::set`, so layout is marked only on real change, or marks paint), runs
   the layout pass with the window's `TextMeasure`, and repaints into the Scene. This avoids
   re-entrancy (an effect firing while the tree is borrowed) and batches changes per frame.
7. **Dynamic content is explicit.** `Dyn` (re-run a view closure when its signals change),
   `Show` (a condition) and `For` (a keyed list) are region elements whose effect queues a
   rebuild; the rebuild disposes the content that went away and builds the new content in a
   fresh child scope. `For` keeps the elements of keys that remain (reordering their layout
   nodes) so focus, scroll position and animation state survive.
8. **Painting is a traversal of render elements.** Each render element's `Paint` draws itself
   and decides where its children are painted (so it can wrap them in a clip, transform or
   layer, as Flutter's `paintChild` allows). The traversal pushes each element's offset as a
   transform, sets the element id on the Scene builder, and skips subtrees that are culled
   (Clay's technique) where it is safe.

## Consequences

- A signal write costs one effect run and one queued update per affected prop; no view code
  re-runs. Large static trees cost nothing after they are built.
- Component state is ordinary signals created in the component function, owned by the scope
  that built it.
- Layout widgets (`Padding`, `Row`, …) are thin views that create a render element with the
  matching `tantu-layout` object, in `tantu-widgets`; app-defined widgets do the same.
- Because effects only queue, everything a frame changes is applied together, in a known order
  (updates, rebuilds, layout, paint), on the thread that owns the tree.
- Conditionals and lists must use `Dyn`/`Show`/`For`; writing `if signal.get() { a } else { b }`
  in a component reads the signal once, at build time (the same rule as Solid and Leptos).
  The docs and examples have to teach this.
- `tantu-view` holds one more piece of shared state per tree: the update queue, shared with
  effects through an `Rc` (the tree is single-threaded, like the runtime).

## Alternatives considered

- **Rebuild subtrees when signals change (Flutter, Xilem):** rejected by the user; it re-runs view
  code per change, against AGENTS.md's fine-grained design.
- **Effects mutating the tree directly** through an `Rc<RefCell<ViewTree>>`: re-entrancy panics
  when an effect fires while the tree is borrowed (during event dispatch or build), and no
  per-frame batching.
- **Dynamic content as ordinary boxes** (a `For` that is its own layout node): a list inside a
  `Column` would be one child of the flex instead of many, breaking `Expanded`, spacing and
  alignment across list items.
- **One element type with an optional layout node** instead of two kinds: equivalent at runtime;
  two kinds make the "no layout node" case explicit in the API and the specs.
