//! # Tantu: development tasks
//!
//! Tooling for working on the Tantu workspace, run as `cargo xtask <command>`. Not published.
//!
//! Commands:
//!
//! - `spec-coverage`: checks that every rule of an agreed spec in `docs/specs/` has a test named
//!   after it. Spec: `docs/specs/xtask/spec-coverage.md`.
//!
//! ```text
//! cargo xtask spec-coverage
//! ```

#![forbid(unsafe_code)]

use std::io::Write;
use std::path::Path;

/// Status of a spec, from its `- **Status:** <value>` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Being written or reviewed; its rules are not checked.
    Draft,
    /// Reviewed and agreed; every active rule needs a test.
    Agreed,
    /// Code and tests match the spec; every active rule needs a test.
    Implemented,
}

/// One rule found in a spec.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// The rule id, e.g. `LAYOUT-FLEX-03`.
    pub id: String,
    /// True if the rule is struck through (removed); removed rules need no test.
    pub removed: bool,
    /// 1-based line number in the spec file.
    pub line: usize,
}

/// A parsed spec file.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    /// The status, or `None` if the status line is missing or has an unknown value.
    pub status: Option<Status>,
    /// Rules in file order.
    pub rules: Vec<Rule>,
}

/// One problem found by the check.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    /// Path of the spec, relative to the workspace root, with `/` separators.
    pub path: String,
    /// Human-readable description of the problem.
    pub message: String,
}

/// Result of a full check.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Specs with status Agreed or Implemented.
    pub checked: usize,
    /// Specs with status Draft.
    pub skipped: usize,
    /// Problems, sorted by path, then by line.
    pub problems: Vec<Problem>,
}

/// Parses the text of one spec file. Never fails; problems show up as `status: None`.
pub fn parse_spec(text: &str) -> Spec {
    let _ = text;
    todo!()
}

/// The test-name prefix for a rule id: lowercase, `-` replaced by `_` (`LAYOUT-FLEX-03` →
/// `layout_flex_03`).
pub fn test_name(rule_id: &str) -> String {
    let _ = rule_id;
    todo!()
}

/// Names of all functions defined in one Rust source file.
pub fn fn_names(source: &str) -> Vec<String> {
    let _ = source;
    todo!()
}

/// Runs the whole check against the workspace at `root`.
pub fn check(root: &Path) -> Report {
    let _ = root;
    todo!()
}

/// Runs the command line with `args` (without the program name) against the workspace at
/// `root`, writing normal output to `out` and errors to `err`. Returns the exit status.
pub fn run(args: &[String], root: &Path, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let _ = (args, root, out, err);
    todo!()
}
