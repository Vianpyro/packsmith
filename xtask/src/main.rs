#![forbid(unsafe_code)]

//! Repository automation, run as `cargo xtask <task>`.
//!
//! `ci` is the gate that must be green before a change is called done: it shells
//! out to the same `cargo` invocations a contributor would type, then runs the
//! conformance suite. `sync-target` regenerates a target's derived data file
//! from a pinned mcmeta commit (ADR-0014).

mod conformance;
mod sync_target;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <task>

tasks:
  ci                            fmt check, clippy (warnings denied), tests, conformance
  sync-target --version <v>     regenerate crates/packsmith-mcversion/data/<v>.json from
                                a pinned misode/mcmeta commit (add --check to only verify)
";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("ci") => ci(),
        Some("sync-target") => match sync_target::run(args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("xtask sync-target: {e:#}");
                ExitCode::FAILURE
            }
        },
        Some("-h") | Some("--help") => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(task) => {
            eprintln!("xtask: unknown task '{task}'\n\n{USAGE}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn ci() -> ExitCode {
    let cargo_steps: [(&str, &[&str]); 3] = [
        ("fmt", &["fmt", "--all", "--check"]),
        (
            "clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        ),
        (
            "test",
            &["test", "--workspace", "--all-features", "--locked"],
        ),
    ];

    for (name, args) in cargo_steps {
        eprintln!("xtask ci: {name}");
        if !run_cargo(args) {
            eprintln!("xtask ci: {name} failed");
            return ExitCode::FAILURE;
        }
    }

    let cases_dir = repo_root().join("conformance/cases");

    eprintln!("xtask ci: conformance (structure)");
    match check_conformance(&cases_dir) {
        Ok(count) => eprintln!("xtask ci: conformance structure OK ({count} cases)"),
        Err(problems) => {
            eprintln!("xtask ci: conformance structure failed");
            for p in &problems {
                eprintln!("  {p}");
            }
            return ExitCode::FAILURE;
        }
    }

    eprintln!("xtask ci: conformance (build + reproducibility)");
    match conformance::run_verified_cases(&cases_dir) {
        Ok(o) => eprintln!(
            "xtask ci: conformance builds OK ({} verified, hashes match; {} built but tree still awaiting in-game verification)",
            o.ran, o.built_unchecked
        ),
        Err(problems) => {
            eprintln!("xtask ci: conformance builds failed");
            for p in &problems {
                eprintln!("  {p}");
            }
            return ExitCode::FAILURE;
        }
    }

    eprintln!("xtask ci: OK");
    ExitCode::SUCCESS
}

pub(crate) fn run_cargo(args: &[&str]) -> bool {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo)
        .args(args)
        .current_dir(repo_root())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub(crate) fn repo_root() -> PathBuf {
    // The xtask crate sits one level below the workspace root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

/// Structural check of the conformance suite: every case is a directory with
/// `input.json`, `target.json`, `README.md`, and exactly one expected result
/// (`expected/` for a successful build, `expected-diagnostics.json` for a
/// compile failure). Returns the case count on success.
///
/// Structure only. Building each case and diffing its tree is
/// [`conformance::run_verified_cases`], which needs the compiler and emitter;
/// this half runs even when those are broken.
fn check_conformance(cases_dir: &Path) -> Result<usize, Vec<String>> {
    let mut names: Vec<String> = match std::fs::read_dir(cases_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(e) => {
            return Err(vec![format!("cannot read {}: {e}", cases_dir.display())]);
        }
    };
    names.sort();

    if names.is_empty() {
        return Err(vec![format!("no cases in {}", cases_dir.display())]);
    }

    let mut problems = Vec::new();
    for name in &names {
        let dir = cases_dir.join(name);
        for required in ["input.json", "target.json", "README.md"] {
            if !dir.join(required).is_file() {
                problems.push(format!("{name}: missing {required}"));
            }
        }

        let has_tree = dir.join("expected").is_dir();
        let has_diagnostics = dir.join("expected-diagnostics.json").is_file();
        match (has_tree, has_diagnostics) {
            (false, false) => problems.push(format!(
                "{name}: needs expected/ or expected-diagnostics.json"
            )),
            (true, true) => problems.push(format!(
                "{name}: has both expected/ and expected-diagnostics.json; a case is one or the other"
            )),
            _ => {}
        }
    }

    if problems.is_empty() {
        Ok(names.len())
    } else {
        Err(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformance_suite_is_well_formed() {
        let cases = repo_root().join("conformance/cases");
        match check_conformance(&cases) {
            Ok(count) => assert!(count >= 5, "expected at least 5 cases, found {count}"),
            Err(problems) => panic!("conformance suite is malformed:\n{}", problems.join("\n")),
        }
    }

    #[test]
    fn missing_expected_result_is_a_problem() {
        let tmp = std::env::temp_dir().join("packsmith-xtask-test-cases");
        let case = tmp.join("broken");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&case).unwrap();
        for f in ["input.json", "target.json", "README.md"] {
            std::fs::write(case.join(f), "{}").unwrap();
        }

        let err = check_conformance(&tmp).expect_err("case has no expected result");
        assert!(err.iter().any(|p| p.contains("broken: needs expected/")));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
