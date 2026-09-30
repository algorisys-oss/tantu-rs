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

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
    let mut status_line: Option<&str> = None;
    let mut rules = Vec::new();
    let mut in_code_block = false;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("- **Status:**") {
            status_line.get_or_insert(value);
            continue;
        }
        let (rest, removed) = match trimmed.strip_prefix("- ~~**") {
            Some(rest) => (rest, true),
            None => match trimmed.strip_prefix("- **") {
                Some(rest) => (rest, false),
                None => continue,
            },
        };
        if let Some((id, _)) = rest.split_once(":**") {
            if is_rule_id(id) {
                rules.push(Rule {
                    id: id.to_owned(),
                    removed,
                    line: index + 1,
                });
            }
        }
    }
    let status = status_line.and_then(|value| match value.split_whitespace().next() {
        Some("Draft") => Some(Status::Draft),
        Some("Agreed") => Some(Status::Agreed),
        Some("Implemented") => Some(Status::Implemented),
        _ => None,
    });
    Spec { status, rules }
}

/// True if `id` has the form `AREA-FEATURE-NN` (XTASK-COV-08).
fn is_rule_id(id: &str) -> bool {
    let segments: Vec<&str> = id.split('-').collect();
    let Some((number, names)) = segments.split_last() else {
        return false;
    };
    let name_ok = |segment: &&str| {
        let mut chars = segment.chars();
        chars.next().is_some_and(|c| c.is_ascii_uppercase())
            && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    };
    !names.is_empty()
        && names.iter().all(name_ok)
        && number.len() >= 2
        && number.chars().all(|c| c.is_ascii_digit())
}

/// The test-name prefix for a rule id: lowercase, `-` replaced by `_` (`LAYOUT-FLEX-03` →
/// `layout_flex_03`).
pub fn test_name(rule_id: &str) -> String {
    rule_id.to_ascii_lowercase().replace('-', "_")
}

/// Names of all functions defined in one Rust source file.
pub fn fn_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for (at, _) in source.match_indices("fn") {
        let before_ok = source[..at]
            .chars()
            .next_back()
            .is_none_or(char::is_whitespace);
        let after = &source[at + 2..];
        if !before_ok || !after.starts_with(char::is_whitespace) {
            continue;
        }
        let name: String = after
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            names.push(name);
        }
    }
    names
}

/// Runs the whole check against the workspace at `root`.
pub fn check(root: &Path) -> Report {
    let mut report = Report {
        checked: 0,
        skipped: 0,
        problems: Vec::new(),
    };
    let specs_dir = root.join("docs").join("specs");
    if !specs_dir.is_dir() {
        report.problems.push(Problem {
            path: "docs/specs".to_owned(),
            message: "directory not found".to_owned(),
        });
        return report;
    }

    let mut spec_files = Vec::new();
    walk(&specs_dir, &mut |path| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.ends_with(".md") && name != "README.md" && name != "TEMPLATE.md" {
            spec_files.push((relative(root, path), path.to_path_buf()));
        }
    });
    spec_files.sort();

    let mut names = Vec::new();
    walk(root, &mut |path| {
        if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(source) = fs::read_to_string(path) {
                names.extend(fn_names(&source));
            }
        }
    });

    for (rel, path) in spec_files {
        let mut problem = |message: String| {
            report.problems.push(Problem {
                path: rel.clone(),
                message,
            });
        };
        let Ok(text) = fs::read_to_string(&path) else {
            problem("cannot be read as UTF-8 text".to_owned());
            continue;
        };
        let spec = parse_spec(&text);
        match spec.status {
            None => {
                problem("no valid status line".to_owned());
                continue;
            }
            Some(Status::Draft) => {
                report.skipped += 1;
                continue;
            }
            Some(Status::Agreed | Status::Implemented) => report.checked += 1,
        }
        let mut seen = HashSet::new();
        let mut active = 0;
        for rule in &spec.rules {
            if !seen.insert(rule.id.as_str()) {
                problem(format!(
                    "duplicate rule id {} (line {})",
                    rule.id, rule.line
                ));
            }
            if rule.removed {
                continue;
            }
            active += 1;
            let expected = test_name(&rule.id);
            let prefix = format!("{expected}_");
            if !names
                .iter()
                .any(|n| *n == expected || n.starts_with(&prefix))
            {
                problem(format!(
                    "{} (line {}) has no test named {expected} or {expected}_*",
                    rule.id, rule.line
                ));
            }
        }
        if active == 0 {
            problem("no active rules".to_owned());
        }
    }
    report
}

/// Calls `visit` for every file under `dir`, recursively, skipping `target` and hidden
/// directories (XTASK-COV-11). Unreadable directories are skipped.
fn walk(dir: &Path, visit: &mut dyn FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "target" && !name.starts_with('.') {
                walk(&path, visit);
            }
        } else {
            visit(&path);
        }
    }
}

/// `path` relative to `root`, with `/` separators on every platform.
fn relative(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Runs the command line with `args` (without the program name) against the workspace at
/// `root`, writing normal output to `out` and errors to `err`. Returns the exit status.
pub fn run(args: &[String], root: &Path, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match args.first().map(String::as_str) {
        Some("spec-coverage") => {
            let report = check(root);
            // Write errors (e.g. a closed pipe) can't be reported anywhere useful; the exit
            // status still reflects the result.
            for p in &report.problems {
                let _ = writeln!(out, "{}: {}", p.path, p.message);
            }
            let _ = writeln!(
                out,
                "spec-coverage: {} checked, {} draft, {} problem(s)",
                report.checked,
                report.skipped,
                report.problems.len()
            );
            i32::from(!report.problems.is_empty())
        }
        _ => {
            let _ = writeln!(
                err,
                "usage: cargo xtask <command>\n\ncommands:\n  spec-coverage  \
                 check that every rule of an agreed spec has a test"
            );
            2
        }
    }
}
