# Files dropped from the OS

- **Status:** Agreed (2026-10-16; the user said "continue" on the five proposals)
- **Crates:** `tantu-platform` (events, fake platform), `tantu-platform-winit`,
  `tantu-view` (delivery), `tantu-widgets` (`FileDropTarget`), `tantu` (runner),
  `tantu-test`, `examples/gallery`
- **Plan item:** Phase 4, "Multi-window, drag & drop, clipboard", part "drag and drop with
  the OS"
- **Related:** [drag and drop inside the app](../widgets/drag-and-drop.md),
  [platform](platform.md), [winit shell](../platform-winit/shell.md)

## Purpose

Apps take files dragged from the desktop or a file manager: an image viewer opening a
picture, an upload area, an editor opening a document. winit reports these drops, and
Tantu currently ignores them. This spec carries them from the platform to a
`FileDropTarget` widget.

## The position problem

winit 0.30 (the version Tantu uses) reports `HoveredFile(path)`, `DroppedFile(path)` and
`HoveredFileCancelled`, one event per file and **without a position**. While the OS drags,
most platforms don't send pointer moves either. winit 0.31 adds `DragEntered`,
`DragMoved` and `DragDropped` with positions, but it is still a beta (0.31.0-beta.3).

So this part delivers drops with an *optional* position: winit 0.30 gives the window's
last known pointer position when the pointer is inside the window (often the point where
the drag entered), otherwise none. Targets are chosen by that position when there is
one, else the drop goes to the window's outermost target. When winit 0.31 is released,
the shell fills in exact positions and moves, and the rules below still hold.

## Scope

In scope: hovered, cancelled and dropped file paths through the platform trait, the fake
platform and the winit shell; delivery in the view tree; a `FileDropTarget` widget with a
hover signal; tester support; a gallery sample.

Out of scope: text, URL and image drops (winit has no API for them); dragging files *out*
of the app; exact drop positions before winit 0.31; reading file contents (apps use
`std::fs`).

## Public API

```rust
// tantu-platform
pub enum WindowEvent {
    // ...
    /// Files are dragged over the window (one event with every path, PLATFORM-DROP-01).
    FilesHovered { paths: Vec<PathBuf>, position: Option<Point> },
    /// The drag left the window or was cancelled.
    FilesHoverCancelled,
    /// Files were dropped on the window.
    FilesDropped { paths: Vec<PathBuf>, position: Option<Point> },
}

// tantu-view
/// What a file-drop handler receives.
pub enum FileDropEvent<'a> {
    Hovered(&'a [PathBuf]),
    Cancelled,
    Dropped(&'a [PathBuf]),
}
impl BuildCx<'_> {
    /// Registers `element` as a file-drop target.
    pub fn on_file_drop(&mut self, element: ElementId, handler: impl Fn(&FileDropEvent<'_>) + 'static);
}
impl ViewTree {
    /// Delivers a platform file drop (see VIEW-FILEDROP-01).
    pub fn dispatch_file_drop(&mut self, event: FileDrop);
}

// tantu-widgets
/// Accepts files dropped from the OS onto `child` (Flutter has no core equivalent; the
/// `desktop_drop` package's `DropTarget` is the model).
pub struct FileDropTarget { /* … */ }
impl FileDropTarget {
    pub fn new(child: impl View) -> Self;
    /// Called with the dropped paths.
    pub fn on_drop(self, f: impl Fn(&[PathBuf]) + 'static) -> Self;
    /// True while files hover over this target (for highlighting).
    pub fn hovering(self, signal: Signal<bool>) -> Self;
}

// tantu-test
impl WidgetTester {
    /// Hovers then drops `paths` at `at` (or with no position).
    pub fn drop_files(&mut self, paths: &[PathBuf], at: Option<Point>);
}
```

## Behavior

- **PLATFORM-DROP-01:** A drop is reported as one `FilesHovered` with every path while the
  files are over the window, then `FilesDropped` with the same paths, or
  `FilesHoverCancelled`. Positions are logical pixels, or `None` when the platform gives
  none.
- **PLATFORM-WINIT-18:** The winit shell collects the per-file `HoveredFile` and
  `DroppedFile` events of one gesture (consecutive events of the same kind) into one
  `FilesHovered`/`FilesDropped`, sent before the next other event or at the end of the
  event batch. The position is the window's last pointer position when the pointer was
  inside the window, else `None`. `HoveredFileCancelled` becomes `FilesHoverCancelled`.
- **PLATFORM-FAKE-16:** The fake platform delivers scripted file events like any window
  event.
- **VIEW-FILEDROP-01:** The target is the innermost registered element containing the
  position when there is one; otherwise, or when no target contains it, the outermost
  target (the first in tree order whose ancestors aren't targets). Offstage targets are
  skipped. No target: the event is ignored.
- **VIEW-FILEDROP-02:** `Hovered` goes to the current target (a target change sends
  `Cancelled` to the old one first); `Dropped` and `Cancelled` go to the hovered target
  and end the gesture. A drop without a preceding hover picks its target the same way.
- **FACADE-APP-19:** The runner forwards file events to the window's tree and redraws as
  for other events.
- **WIDGETS-FILEDROP-01:** `FileDropTarget` sets `hovering` true on `Hovered` and false on
  `Cancelled` or `Dropped`, and calls `on_drop` with the paths. It paints nothing itself
  (apps highlight with the signal).
- **TEST-WT-20:** `WidgetTester::drop_files` sends a hover and a drop, pumping after each.
- **GALLERY-54:** A "files" kitchen-sink card holds a 120 px tall drop area that turns
  `primary_container` while files hover and lists the dropped file names in
  `sink-files-state` ("Dropped: a.txt, b.png").

## Performance and allocation

A drop allocates its path list once; nothing per frame.

## Open questions

Proposals from the agent, for review:

1. **Stay on winit 0.30 with optional positions** (last known pointer, else the outermost
   target), rather than moving to the 0.31 beta now. Most apps have one drop area per
   window, which works without positions. Upgrade when 0.31 is released.
2. **One event per gesture with all paths** (the shell batches winit's per-file events),
   so a target sees "3 files dropped", not three drops.
3. **Files only.** winit can't deliver text or URL drops; those wait for the platform.
4. **`FileDropTarget` paints nothing**; a hover signal lets apps choose the highlight,
   as `DragTarget`'s `candidate` does.
5. **No contents reading or filtering by type** in the widget: apps check extensions in
   `on_drop` (a later `accept` filter can come with positions, since without them the
   platform's cursor can't show "not allowed" per target anyway).
