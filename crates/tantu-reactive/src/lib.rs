//! # Tantu: Reactivity
//!
//! Signals, memos and effects with automatic dependency tracking. State lives in [`Signal`]s;
//! [`Memo`]s derive cached values from them; [`Effect`]s run side effects. Writing a signal
//! re-runs only what read it, once, glitch-free.
//!
//! Everything lives in a [`Runtime`], entered with [`Runtime::enter`]. Handles are `Copy` ids,
//! so closures capture them freely. [`Scope`]s own nodes and dispose them as a unit.
//!
//! ```
//! use std::cell::Cell;
//! use std::rc::Rc;
//! use tantu_reactive::{Runtime, effect, memo, signal};
//!
//! Runtime::new().enter(|| {
//!     let count = signal(1);
//!     let doubled = memo(move || count.get() * 2);
//!     let seen = Rc::new(Cell::new(0));
//!     let s = seen.clone();
//!     effect(move || s.set(doubled.get()));
//!     assert_eq!(seen.get(), 2);
//!     count.set(5);
//!     assert_eq!(seen.get(), 10);
//! });
//! ```
//!
//! Spec: `docs/specs/reactive/signals.md`.

#![forbid(unsafe_code)]

mod effect;
mod memo;
mod runtime;
mod scope;
mod signal;

pub use effect::{Effect, effect};
pub use memo::{Memo, memo};
pub use runtime::Runtime;
pub use scope::{Scope, batch, on_cleanup, untrack};
pub use signal::{Signal, signal};
