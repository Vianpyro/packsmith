#![forbid(unsafe_code)]

//! The `packsmith` command line interface.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use packsmith_compiler::Graph;
use packsmith_mcversion::TargetData;

const USAGE: &str = "\
packsmith - compile a block graph into a Minecraft: Java Edition data pack

usage:
  packsmith build <project> --target <version> [--output <file>]
  packsmith --help
  packsmith --version

commands:
  build    compile a project directory into an installable .zip

build:
  <project>          a project directory (containing graph.json) or a graph .json file
  --target <version> the Minecraft: Java Edition release to build for, e.g. 26.2
  --output <file>    where to write the .zip (default: <project>.zip in the working directory)
";

const BUILD_USAGE: &str = "usage: packsmith build <project> --target <version> [--output <file>]";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("build") => build(&args[1..]),
        Some("-h" | "--help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some("-V" | "--version") => {
            println!("packsmith {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(other) => Err(format!("packsmith: unknown command '{other}'\n\n{USAGE}")),
    }
}

struct BuildArgs {
    project: String,
    target: String,
    output: Option<String>,
}

fn build(args: &[String]) -> Result<(), String> {
    let BuildArgs {
        project,
        target,
        output,
    } = parse_build_args(args)?;

    let graph_path = resolve_graph_path(Path::new(&project))?;
    let graph_bytes = std::fs::read(&graph_path)
        .map_err(|e| format!("packsmith build: reading {}: {e}", graph_path.display()))?;
    let graph: Graph = serde_json::from_slice(&graph_bytes).map_err(|e| {
        format!(
            "packsmith build: {} is not a valid graph: {e}",
            graph_path.display()
        )
    })?;

    let target_data = TargetData::load(&packsmith_mcversion::bundled_data_dir(), &target)
        .map_err(|e| format!("packsmith build: {e}"))?;

    let ir = packsmith_compiler::compile(&graph, &target);
    let tree = packsmith_emit::file_tree(&ir, &target_data)
        .map_err(|e| format!("packsmith build: {e}"))?;
    let zip = packsmith_emit::zip(&tree);

    let output = output
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output(&graph_path));
    std::fs::write(&output, &zip)
        .map_err(|e| format!("packsmith build: writing {}: {e}", output.display()))?;
    eprintln!(
        "packsmith build: wrote {} ({} bytes)",
        output.display(),
        zip.len()
    );
    Ok(())
}

/// A `<project>` is either a graph file or a directory holding `graph.json`.
fn resolve_graph_path(project: &Path) -> Result<PathBuf, String> {
    if project.is_dir() {
        let candidate = project.join("graph.json");
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!(
            "packsmith build: {} has no graph.json",
            project.display()
        ));
    }
    if project.is_file() {
        return Ok(project.to_path_buf());
    }
    Err(format!(
        "packsmith build: {} does not exist",
        project.display()
    ))
}

fn default_output(graph_path: &Path) -> PathBuf {
    let stem = graph_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("pack");
    PathBuf::from(format!("{stem}.zip"))
}

fn parse_build_args(args: &[String]) -> Result<BuildArgs, String> {
    let mut project: Option<String> = None;
    let mut target: Option<String> = None;
    let mut output: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if let Some(value) = arg.strip_prefix("--target=") {
            target = Some(value.to_string());
        } else if arg == "--target" {
            i += 1;
            let value = args.get(i).ok_or_else(|| {
                format!("packsmith build: --target needs a value\n\n{BUILD_USAGE}")
            })?;
            target = Some(value.clone());
        } else if let Some(value) = arg.strip_prefix("--output=") {
            output = Some(value.to_string());
        } else if arg == "--output" {
            i += 1;
            let value = args.get(i).ok_or_else(|| {
                format!("packsmith build: --output needs a value\n\n{BUILD_USAGE}")
            })?;
            output = Some(value.clone());
        } else if arg.starts_with('-') {
            return Err(format!(
                "packsmith build: unknown option '{arg}'\n\n{BUILD_USAGE}"
            ));
        } else if project.is_none() {
            project = Some(arg.to_string());
        } else {
            return Err(format!(
                "packsmith build: unexpected extra argument '{arg}'\n\n{BUILD_USAGE}"
            ));
        }
        i += 1;
    }

    Ok(BuildArgs {
        project: project
            .ok_or_else(|| format!("packsmith build: missing <project>\n\n{BUILD_USAGE}"))?,
        target: target.ok_or_else(|| {
            format!("packsmith build: missing --target <version>\n\n{BUILD_USAGE}")
        })?,
        output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn build_requires_project_and_target() {
        assert!(parse_build_args(&argv(&[])).is_err());
        assert!(parse_build_args(&argv(&["proj"])).is_err());
        assert!(parse_build_args(&argv(&["--target", "26.2"])).is_err());
        assert!(parse_build_args(&argv(&["--target"])).is_err());
    }

    #[test]
    fn build_accepts_options_in_any_order() {
        let a = parse_build_args(&argv(&["proj", "--target", "26.2"])).unwrap();
        assert_eq!((a.project.as_str(), a.target.as_str()), ("proj", "26.2"));
        assert!(a.output.is_none());

        let a = parse_build_args(&argv(&["--target=26.2", "--output=out.zip", "proj"])).unwrap();
        assert_eq!(
            (a.project.as_str(), a.target.as_str(), a.output.as_deref()),
            ("proj", "26.2", Some("out.zip"))
        );
    }

    #[test]
    fn unknown_command_is_rejected() {
        assert!(run(argv(&["frobnicate"])).is_err());
    }

    #[test]
    fn help_and_version_succeed() {
        assert!(run(argv(&["--help"])).is_ok());
        assert!(run(argv(&["--version"])).is_ok());
        assert!(run(argv(&[])).is_ok());
    }

    #[test]
    fn build_reports_a_missing_project_before_touching_target_data() {
        let err = build(&argv(&["no-such-dir", "--target", "26.2"])).unwrap_err();
        assert!(err.contains("does not exist"));
    }
}
