//! An order dashboard, the Phase 4 milestone: a dockable layout with an order grid, a
//! details form, KPI tiles and a revenue chart, an activity log, a menu bar, a toolbar, a
//! status bar and English or German. Run `cargo run -p enterprise-dashboard`.
//! Spec: `docs/specs/examples/enterprise-dashboard.md`.
//!
//! ```no_run
//! use tantu::prelude::*;
//! App::new()
//!     .window(Window::new("Orders").size(1280.0, 800.0), enterprise_dashboard::dashboard)
//!     .run()?;
//! # Ok::<(), tantu::Error>(())
//! ```

#![forbid(unsafe_code)]

use tantu::prelude::*;

/// A sales region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// North.
    North,
    /// South.
    South,
    /// East.
    East,
    /// West.
    West,
}

/// Where an order is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Not shipped yet.
    Open,
    /// On its way.
    Shipped,
    /// Settled.
    Paid,
}

/// One order (generated).
#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    /// The order number, from 1.
    pub id: u32,
    /// Who ordered.
    pub customer: String,
    /// Where.
    pub region: Region,
    /// How much, in currency units with cents.
    pub amount: f64,
    /// When.
    pub date: Date,
    /// Where it is.
    pub status: Status,
}

/// The `count` orders the app shows, the same on every run (DASH-02).
pub fn orders(count: usize) -> Vec<Order> {
    let _ = count;
    todo!("DASH-02")
}

/// The dashboard's view (DASH-01).
pub fn dashboard() -> impl View {
    todo!("DASH-01");
    #[allow(unreachable_code)]
    Text::new("")
}
