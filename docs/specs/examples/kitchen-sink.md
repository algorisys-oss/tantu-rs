# Kitchen sink: every widget on one scrolling page

- **Status:** Implemented (the user said "continue" on the draft, 2026-10-12, taking its six
  proposals)
- **Crate:** `examples/gallery` (a new section; no library changes)
- **Plan item:** Phase 4, "Kitchen sink"
- **Related:** [gallery](gallery.md) (the sections and keys this adds to),
  [DataGrid](../widgets/data-grid.md), [DataGrid sorting](../widgets/data-grid-sort.md),
  [cell editors](../widgets/data-grid-editing.md), [stress example](data-grid-stress.md),
  [scroll](../widgets/scroll.md), [focus scopes](../view/focus-scope.md)

## Purpose

The gallery shows widgets a section at a time; a reader (or a screenshot) still can't see
everything Tantu has in one place, and a few widgets have no sample at all (`VerticalDivider`,
`Opacity`, `FocusScope`, `Stack`/`Positioned`, `Flexible`, `LimitedBox`, `Center`, a bare
`ScrollView`). The kitchen sink is one scrolling page with a labelled card per widget family,
every exported widget in it at least once, and each card's state echoed as text, so
`cargo run -p gallery` is also the complete demo and a test can check nothing is missing.

## Scope

In scope: a "Kitchen sink" section (the last in the sidebar) built from the existing widgets;
cards for buttons, toggles, slider, text, fields, media, lists, grid, layout, scrolling,
animation and focus; keys on every card and sample; goldens; a README line.

Out of scope: new widgets or new widget features; a separate example crate (the gallery is
the demo, so it gains the page rather than a sibling duplicating its shell); per-platform
screenshots.

## Public API

No library API. In `examples/gallery`:

```rust
/// The section names, in sidebar order; "Kitchen sink" is the last.
pub const SECTIONS: [&str; 9];

/// The kitchen-sink cards, in page order: (card key, the widgets it shows).
pub const SINK_CARDS: [(&str, &[&str]); 12];
```

The page: a `ScrollView` (key `sink`) over a `Column` of cards, each a `Column` keyed
`sink-<card>` with a title `Text` in the title style, the samples, and (where the card has
state) an echo `Text` keyed `sink-<card>-state`. The samples are keyed `sink-<widget>` in
snake case (`sink-button`, `sink-icon_button`, …) where a test needs them; card keys never
equal a sample key (`sliders`, `typography`).

| Card key | Shows | Echo |
|---|---|---|
| `buttons` | `Button` (enabled, disabled), `IconButton`, `Tooltip` around a button | "Pressed N" |
| `toggles` | `Checkbox` (on, indeterminate, disabled), three `Radio`s, `Switch` | "Checkbox on, Radio b, Switch off" |
| `sliders` | `Slider` continuous and with 4 divisions | "Slider 50 / 25" |
| `typography` | `Text` body/title/label, `RichText`, `Divider`, a `Row` with `VerticalDivider` | |
| `fields` | `TextField` single-line, multi-line, with a placeholder, disabled | "Typed: …" |
| `media` | `Icon`s, `Image` in `BoxFit::Contain`, `Cover` and `Fill` at 96 × 64 | |
| `lists` | `ListView::builder` (1 000 rows) and `separated` with a selection | "Selected N" |
| `grid` | `DataGrid` (1 000 rows, frozen Id, sortable, `client_sort`), resize/reorder headers, range selection, editable Name/Score and a "Even rows" filter | Sort/rows, selection, widths/order and edit errors |
| `layout` | `Row`/`Column` with `Expanded`, `Flexible` and `Spacer`; `Stack` with `Positioned`; `Center`, `Align`, `Padding`, `SizedBox`, `ConstrainedBox`, `LimitedBox` as colored boxes | |
| `scrolling` | a 240 px tall `ScrollView` of 30 rows and a horizontal `ScrollView` of 20 boxes | "Offset N" |
| `animation` | `Opacity` driven by a `Slider`, `AnimatedSwitcher` toggled by a `Switch` | |
| `focus` | a `FocusScope` with three `Button`s and a button outside it | "Focus: name" (from `on_focus_change`) |

## Behavior

- **GALLERY-12:** The "Kitchen sink" section (key `kitchen-sink`, nav key `nav-kitchen-sink`)
  is a `ScrollView` keyed `sink` over the cards of `SINK_CARDS`, in order, each present by its
  `sink-<card>` key, with the first card at the top; every widget named in `SINK_CARDS` is
  built by that card (the test checks each `sink-<widget>` key exists after scrolling the
  card into view with `ensure_visible`); the section's content is taller than the window and
  scrolls with the wheel (the `sink` controller's maximum extent is above 0).
- **GALLERY-13:** The echoes follow their widgets: pressing the button updates "Pressed 1";
  the checkbox, radio and switch echo "Checkbox off, Radio c, Switch on" after taps; typing
  into the single-line field updates "Typed: …"; the grid's "Even rows" switch halves the row
  count in "rows N" and a tap on its "Id" header sets "Sort: Id ascending" and reorders the
  first visible row; focusing a button in the focus card (`WidgetTester::focus`; buttons don't take focus
  on press) echoes "Focus: Two", and Tab from the scope's last button wraps to its first
  (the outside button is never reached).
- **GALLERY-14:** Goldens at 960 × 640: `tests/goldens/gallery-sink-top.png` (the page as
  opened) and `tests/goldens/gallery-sink-end.png` (after a tap on the page focuses its
  scroll view and End scrolls it to the maximum extent).

- **GALLERY-18:** Amendment on the user's request to update the kitchen sink (2026-10-12;
  routine composition of agreed grid APIs, implementation details marked review): the grid
  keeps its frozen Id, sorting and Even rows filter; Name and Score are editable, Id is
  read-only, successful writes are stored as sparse overrides and Score outside 0..=100
  reports "Score must be between 0 and 100" in `sink-grid-edit-error` while retaining the
  draft. Double-click/F2, Enter and Escape follow the cell-editor spec. Range selection
  echoes anchor/active source coordinates in `sink-grid-selection`. Resizing and reordering
  share app-visible width/order signals echoed as `sink-grid-name-width` and
  `sink-grid-order`; keyboard header navigation can resize/reorder them. Instructions
  explain editing and selection; `sink-grid-stress-command` shows
  `cargo run --release -p data-grid-stress` for the separate million-row workload. A golden
  at 960 × 640 with the grid card scrolled into view records these controls and feedback.
  Existing GALLERY-12..14 remain valid.

- **GALLERY-19:** The Buttons card demonstrates inherited overlay themes. A `Local dark`
  switch (`sink-popup-dark`, initially on) controls a local ThemeProvider around an
  OverlayPortal; `Open popup` (`sink-overlay_portal`) opens a `Close themed popup` button
  (`sink-popup-close`) beside the compact target controls (12 px gap, centers aligned, usual flip/clamp),
  using the local theme's primary color. The switch
  updates an open popup's colors without changing its element identity, independently of
  the gallery's global dark switch. Close or a primary press outside target/popup hides it.
  A `sink-popup-state` text shows visible/hidden and light/dark state. A software golden
  `tests/goldens/gallery-sink-popup.png` at 960 × 640 records the open local-dark popup.
  `SINK_CARDS` includes `overlay_portal` in the Buttons inventory; existing grid and focus
  samples remain unchanged. This composition amendment follows the user's request to
  continue overlay inheritance; presentation details are marked review.

## Performance and allocation

The page builds every card at once (about a dozen widgets per card, the lists and the grid
virtualized), so its first frame costs more than a single section; nothing new per frame.
If the section's build time becomes noticeable, cards can be wrapped in `LayoutBuilder`s
that build lazily; not needed for a demo.

## Open questions

Resolved (2026-10-12; the user said "continue" on the draft, taking these proposals):

1. **A section of the gallery, not a new example crate.** The gallery already has the shell,
   the theme switch, the tests and the README entry; a `kitchen-sink` crate would duplicate
   them and drift. `cargo run -p gallery` stays the one command to see everything.
2. **Every card builds eagerly inside one `ScrollView`,** so the page is one scroll and one
   screenshot; the virtualized widgets keep it cheap. Lazy cards (`LayoutBuilder`) are the
   fallback if the first frame gets slow.
3. **`SINK_CARDS` is a public constant** the test iterates, so adding a widget to the crate
   without adding it to the page is a one-line change to keep in step, and a reviewer sees
   the inventory in one place. A check against the crate's exports would need reflection
   the language doesn't have; the constant is the honest substitute.
4. **State echoes as text everywhere,** as the rest of the gallery does, so the tests and a
   reader see what a sample did without reading pixels.
5. **Two goldens (top and end)** rather than one per card: the page's purpose is the whole,
   and the per-widget goldens already live in the widget tests.
6. **The grid demo turns `client_sort` on with 1 000 rows** (not a million): small enough to
   sort in a debug test, large enough to scroll.

7. **Kitchen-sink grid refresh (review, 2026-10-12):** the user asked to update the demo
   alongside the stress example. Compose the completed resize/reorder, range selection
   and editor APIs; keep the source small for client sorting and provide the standalone
   stress command as text. No process launch or library API is added.
