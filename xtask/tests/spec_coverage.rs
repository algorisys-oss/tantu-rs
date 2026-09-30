//! Tests for `cargo xtask spec-coverage`, one or more per rule in
//! `docs/specs/xtask/spec-coverage.md`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xtask::{Status, check, fn_names, parse_spec, run, test_name};

/// A throwaway workspace root under the system temp directory, removed on drop.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("tantu-xtask-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create fixture root");
        Self { root }
    }

    fn write(&self, rel: &str, contents: impl AsRef<[u8]>) -> &Self {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().expect("fixture paths have a parent"))
            .expect("create fixture dir");
        fs::write(path, contents).expect("write fixture file");
        self
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A minimal spec with the given status line value and behavior section.
fn spec(status: &str, rules: &str) -> String {
    format!("# Test spec\n\n- **Status:** {status}\n\n## Behavior\n\n{rules}\n")
}

fn messages(report: &xtask::Report) -> Vec<String> {
    report
        .problems
        .iter()
        .map(|p| format!("{}: {}", p.path, p.message))
        .collect()
}

fn run_capture(args: &[&str], root: &Path) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let status = run(&args, root, &mut out, &mut err);
    (
        status,
        String::from_utf8(out).expect("stdout is UTF-8"),
        String::from_utf8(err).expect("stderr is UTF-8"),
    )
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives inside the workspace root")
        .to_path_buf()
}

// Finding specs

#[test]
fn xtask_cov_01_finds_specs_recursively_except_readme_and_template() {
    let f = Fixture::new("cov01");
    f.write("docs/specs/a/x.md", spec("Agreed", "- **A-B-01:** rule"))
        .write("docs/specs/b/c/y.md", spec("Draft", "- **A-B-02:** rule"))
        .write("docs/specs/README.md", spec("Agreed", "- **A-B-03:** rule"))
        .write(
            "docs/specs/TEMPLATE.md",
            spec("Agreed", "- **A-B-04:** rule"),
        )
        .write(
            "docs/specs/a/notes.txt",
            spec("Agreed", "- **A-B-05:** rule"),
        );
    let report = check(&f.root);
    assert_eq!(report.checked, 1);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.problems.len(), 1, "{:?}", messages(&report));
    assert_eq!(report.problems[0].path, "docs/specs/a/x.md");
}

#[test]
fn xtask_cov_02_missing_specs_directory_is_one_problem() {
    let f = Fixture::new("cov02");
    let report = check(&f.root);
    assert_eq!(report.checked, 0);
    assert_eq!(report.skipped, 0);
    assert_eq!(report.problems.len(), 1);
    assert_eq!(report.problems[0].path, "docs/specs");
    assert!(report.problems[0].message.contains("directory not found"));
}

#[test]
fn xtask_cov_03_unreadable_spec_is_a_problem_and_check_continues() {
    let f = Fixture::new("cov03");
    f.write("docs/specs/bad.md", [0xff_u8, 0xfe, 0xfd])
        .write("docs/specs/good.md", spec("Draft", ""));
    let report = check(&f.root);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.problems.len(), 1, "{:?}", messages(&report));
    assert_eq!(report.problems[0].path, "docs/specs/bad.md");
}

// Reading a spec

#[test]
fn xtask_cov_04_status_is_first_word_of_first_status_line() {
    assert_eq!(
        parse_spec("- **Status:** Agreed <!-- Draft | Agreed -->").status,
        Some(Status::Agreed)
    );
    assert_eq!(
        parse_spec("- **Status:** Implemented").status,
        Some(Status::Implemented)
    );
    assert_eq!(
        parse_spec("- **Status:** Draft").status,
        Some(Status::Draft)
    );
    assert_eq!(
        parse_spec("- **Status:** Draft\n- **Status:** Agreed").status,
        Some(Status::Draft)
    );
}

#[test]
fn xtask_cov_04_status_is_case_sensitive() {
    assert_eq!(parse_spec("- **Status:** agreed").status, None);
}

#[test]
fn xtask_cov_05_missing_or_unknown_status_is_a_problem() {
    let f = Fixture::new("cov05");
    f.write("docs/specs/none.md", "# No status\n\n- **A-B-01:** rule\n")
        .write("docs/specs/unknown.md", spec("Done", "- **A-B-01:** rule"));
    let report = check(&f.root);
    assert_eq!(report.checked, 0);
    assert_eq!(report.skipped, 0);
    let msgs = messages(&report);
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(
        msgs.iter().all(|m| m.contains("no valid status line")),
        "{msgs:?}"
    );
}

#[test]
fn xtask_cov_06_draft_specs_are_skipped() {
    let f = Fixture::new("cov06");
    f.write(
        "docs/specs/d.md",
        spec("Draft", "- **A-B-01:** uncovered rule"),
    );
    let report = check(&f.root);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.checked, 0);
    assert!(report.problems.is_empty(), "{:?}", messages(&report));
}

#[test]
fn xtask_cov_07_rules_are_list_items_only() {
    let text = "\
- **Status:** Agreed
- **A-B-01:** active
  - **A-B-02:** nested active
- ~~**A-B-03:** removed~~ Removed: no longer needed.
See **A-B-04:** in prose.
| **A-B-05:** | table |
```
- **A-B-06:** inside a code block
```
- **A-B-07:** after the code block
";
    let rules = parse_spec(text).rules;
    let found: Vec<(&str, bool, usize)> = rules
        .iter()
        .map(|r| (r.id.as_str(), r.removed, r.line))
        .collect();
    assert_eq!(
        found,
        vec![
            ("A-B-01", false, 2),
            ("A-B-02", false, 3),
            ("A-B-03", true, 4),
            ("A-B-07", false, 10),
        ]
    );
}

#[test]
fn xtask_cov_08_rule_id_format() {
    let text = "\
- **LAYOUT-FLEX-03:** valid
- **A11Y-TREE-12:** valid, digits inside a segment
- **CORE-GEOM-100:** valid, three digits
- **FLEX-3:** one digit
- **layout-flex-03:** lowercase
- **03:** number only
- **1AB-CD-01:** segment starts with a digit
- **LAYOUT-FLEX:** no number
- **Status:** not an id
";
    let ids: Vec<String> = parse_spec(text).rules.into_iter().map(|r| r.id).collect();
    assert_eq!(ids, vec!["LAYOUT-FLEX-03", "A11Y-TREE-12", "CORE-GEOM-100"]);
}

#[test]
fn xtask_cov_09_duplicate_rule_ids_are_problems() {
    let f = Fixture::new("cov09");
    f.write(
        "docs/specs/dup.md",
        spec(
            "Agreed",
            "- **A-B-01:** one\n- **A-B-01:** again\n- ~~**A-B-01:** removed~~",
        ),
    )
    .write("src/lib.rs", "#[test]\nfn a_b_01() {}\n");
    let report = check(&f.root);
    let msgs = messages(&report);
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(
        msgs.iter().all(|m| m.contains("duplicate rule id")),
        "{msgs:?}"
    );
}

#[test]
fn xtask_cov_10_agreed_spec_without_active_rules_is_a_problem() {
    let f = Fixture::new("cov10");
    f.write(
        "docs/specs/empty.md",
        spec("Agreed", "- ~~**A-B-01:** removed~~"),
    )
    .write("docs/specs/draft.md", spec("Draft", ""));
    let report = check(&f.root);
    let msgs = messages(&report);
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    assert!(msgs[0].starts_with("docs/specs/empty.md:"), "{msgs:?}");
    assert!(msgs[0].contains("no active rules"), "{msgs:?}");
}

// Matching tests

#[test]
fn xtask_cov_11_collects_rust_sources_skipping_target_and_hidden_dirs() {
    let f = Fixture::new("cov11");
    f.write(
        "docs/specs/s.md",
        spec(
            "Agreed",
            "- **A-B-01:** r\n- **A-B-02:** r\n- **A-B-03:** r\n- **A-B-04:** r",
        ),
    )
    .write("src/lib.rs", "fn a_b_01() {}")
    .write("target/debug/gen.rs", "fn a_b_02() {}")
    .write(".hidden/x.rs", "fn a_b_03() {}")
    .write("deep/nested/dir/w.rs", "fn a_b_04() {}");
    let report = check(&f.root);
    let msgs = messages(&report);
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(msgs[0].contains("A-B-02"), "{msgs:?}");
    assert!(msgs[1].contains("A-B-03"), "{msgs:?}");
}

#[test]
fn xtask_cov_12_function_names() {
    let source = "\
fn a() {}
    pub fn b() {}
pub(crate) fn c() {}
async fn d() {}
pub async fn e<T>(t: T) {}
fnord x
my_fn f
let fnx = 1;
";
    let names = fn_names(source);
    assert_eq!(names, vec!["a", "b", "c", "d", "e"]);
}

#[test]
fn xtask_cov_13_test_name_mapping() {
    assert_eq!(test_name("LAYOUT-FLEX-03"), "layout_flex_03");
    assert_eq!(test_name("A11Y-TREE-12"), "a11y_tree_12");
}

#[test]
fn xtask_cov_13_exact_or_underscore_prefix_covers_a_rule() {
    let f = Fixture::new("cov13");
    f.write(
        "docs/specs/s.md",
        spec(
            "Agreed",
            "- **LAYOUT-FLEX-03:** r\n- **LAYOUT-FLEX-04:** r\n\
             - **LAYOUT-FLEX-05:** r\n- **LAYOUT-FLEX-06:** r",
        ),
    )
    .write(
        "tests/t.rs",
        "fn layout_flex_03() {}\nfn layout_flex_04_shares_space() {}\n\
         fn layout_flex_050() {}\nfn layout_flex_6() {}\n",
    );
    let report = check(&f.root);
    let msgs = messages(&report);
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(msgs[0].contains("LAYOUT-FLEX-05"), "{msgs:?}");
    assert!(msgs[1].contains("LAYOUT-FLEX-06"), "{msgs:?}");
}

#[test]
fn xtask_cov_14_uncovered_rule_message_names_id_line_and_expected_test() {
    let f = Fixture::new("cov14");
    f.write(
        "docs/specs/s.md",
        spec(
            "Implemented",
            "- **A-B-01:** active\n- ~~**A-B-02:** removed~~",
        ),
    );
    let report = check(&f.root);
    let msgs = messages(&report);
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    // spec() puts the behavior section on line 7.
    assert_eq!(
        msgs[0],
        "docs/specs/s.md: A-B-01 (line 7) has no test named a_b_01 or a_b_01_*"
    );
}

// Command line

#[test]
fn xtask_cov_15_binary_checks_the_workspace_from_any_directory() {
    let f = Fixture::new("cov15");
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("spec-coverage")
        .current_dir(&f.root)
        .output()
        .expect("run xtask binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        matches!(output.status.code(), Some(0 | 1)),
        "status {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    // The real workspace has at least this spec, which is Agreed or Implemented.
    assert!(stdout.contains("spec-coverage: "), "{stdout}");
    assert!(!stdout.contains(" 0 checked"), "{stdout}");
}

#[test]
fn xtask_cov_16_prints_problems_and_summary_and_sets_exit_status() {
    let f = Fixture::new("cov16");
    f.write("docs/specs/s.md", spec("Agreed", "- **A-B-01:** r"))
        .write("docs/specs/d.md", spec("Draft", ""));
    let (status, out, _) = run_capture(&["spec-coverage"], &f.root);
    assert_eq!(status, 1);
    assert_eq!(
        out,
        "docs/specs/s.md: A-B-01 (line 7) has no test named a_b_01 or a_b_01_*\n\
         spec-coverage: 1 checked, 1 draft, 1 problem(s)\n"
    );

    f.write("src/lib.rs", "fn a_b_01_covered() {}");
    let (status, out, _) = run_capture(&["spec-coverage"], &f.root);
    assert_eq!(status, 0);
    assert_eq!(out, "spec-coverage: 1 checked, 1 draft, 0 problem(s)\n");
}

#[test]
fn xtask_cov_17_missing_or_unknown_subcommand_prints_usage() {
    let f = Fixture::new("cov17");
    for args in [&[][..], &["bogus"][..]] {
        let (status, out, err) = run_capture(args, &f.root);
        assert_eq!(status, 2, "args {args:?}");
        assert!(out.is_empty(), "args {args:?}: {out}");
        assert!(err.contains("spec-coverage"), "args {args:?}: {err}");
    }
}

// CI

#[test]
fn xtask_cov_18_ci_runs_spec_coverage() {
    let ci = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("read CI workflow");
    assert!(ci.contains("run: cargo xtask spec-coverage"), "{ci}");
}
