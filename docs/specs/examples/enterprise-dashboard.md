# Enterprise dashboard example (the Phase 4 milestone)

- **Status:** Agreed (2026-10-16; the user, leaving for five hours, asked the agent to
  complete pending items; decided by the agent, review)
- **Crate:** `examples/enterprise-dashboard`
- **Plan item:** Phase 4, "**Milestone:** `examples/enterprise-dashboard` (docking + grid +
  forms + charts area)"
- **Related:** [dock area](../widgets/dock-area.md), [data grid](../widgets/data-grid.md),
  [forms](../widgets/form.md), [layout grid](../widgets/layout-grid.md),
  [localization](../i18n/localization.md), [menu bar](../widgets/menu-bar.md),
  [gallery](gallery.md)

## Purpose

Phase 4's goal is a toolkit that can carry an enterprise app. The gallery shows widgets one
at a time; this example puts them together the way a line-of-business screen does: an
order list in a data grid, a form editing the selected order, KPI tiles and a chart that
follow the edits, an activity log, all in a dockable layout with a menu bar, a toolbar, a
status bar and a language switch. It is the integration test of the phase and a starting
point for app authors.

## Scope

In scope: a workspace package (library + binary) depending only on `tantu`
(`tantu-test` for tests); 10 000 generated orders; the four panels and window chrome
below; English and German; behavior tests and a software golden.

Out of scope: persistence, networking, real data, printing, multiple windows (the dock's
floating panels wait for multi-window), and a general charting library (the chart is a
small custom-painted view in the example; a charts library is a later item in PLAN.md).

## Public API

```rust
// examples/enterprise-dashboard (library)
/// One order (generated).
#[derive(Clone, Debug, PartialEq)]
pub struct Order { pub id: u32, pub customer: String, pub region: Region, pub amount: f64,
                   pub date: Date, pub status: Status }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region { North, South, East, West }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status { Open, Shipped, Paid }
/// The `count` orders the app shows, the same on every run.
pub fn orders(count: usize) -> Vec<Order>;
/// The dashboard's view.
pub fn dashboard() -> impl View;
```

## Behavior

- **DASH-01:** Window: a `MenuBar` (File → Reset layout; View → English / Deutsch; Help →
  About), a `Toolbar` (Reset edits, the selected order's number), a `DockArea` and a
  `StatusBar`. The dock starts with the Orders panel on the left (55 %) and, on the right,
  Details above Revenue, with Activity as a second tab beside Orders. Panels are keyed
  `orders`, `details`, `revenue`, `activity`.
- **DASH-02:** `orders(n)` is deterministic (a seeded generator): ids `1..=n`, customers
  from a fixed list, regions and statuses cycling, amounts between 10.00 and 5 000.00 with
  cents, dates in 2026. The Orders panel is a `DataGrid` (No, Customer, Region, Amount,
  Date, Status; sortable on the client; row selection), amounts and dates written with
  the localization's formats.
- **DASH-03:** Details: a `Form` for the selected order: Customer (`TextFormField`,
  required), Amount (digits and one decimal separator, must be above 0), Date
  (`DatePickerField`), Region (`ComboBox`) and a Save button. Save validates; when valid
  it stores the edit (the grid shows it), and the activity log gains "Order 42 updated".
  With no order selected the panel says "Select an order".
- **DASH-04:** Revenue: three KPI tiles in a `LayoutGrid` (orders, total revenue, average
  order) and a bar chart of revenue per region (four bars, heights proportional to the
  largest, labelled with region and amount), all following the edits.
- **DASH-05:** View → Deutsch switches every message, number and date to German (for
  example the total revenue tile and the grid's amounts), and back with View → English.
  File → Reset layout restores the starting dock layout; Reset edits drops every edit.
- **DASH-06:** The status bar shows "10,000 orders" and, with a selection, "Order 42
  selected". A software golden of the 1280 × 800 window with order 1 selected is
  `tests/goldens/dashboard.png`.

## Performance and allocation

The grid virtualizes rows; totals are memos over the orders and the edits, recomputed per
edit (10 000 additions), not per frame.

## Open questions

Decided by the agent (2026-10-16), for review:

1. **A painted bar chart in the example**, not a chart widget in `tantu-widgets`; charts
   remain the "later" library item.
2. **Edits as overrides** on generated data (as the gallery's editable grid), so the data
   stays deterministic and Reset edits is trivial.
3. **English and German** only.
