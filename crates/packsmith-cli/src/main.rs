#![forbid(unsafe_code)]

//! The `packsmith` command line interface.
//!
//! Phase 0 ships the argument parser and nothing behind it: `build` parses its
//! arguments and then reports that the compiler does not exist yet, rather than
//! emitting an empty or fake pack.

use std::process::ExitCode;

const USAGE: &str = "\
packsmith - compile a block graph into a Minecraft: Java Edition data pack

usage:
  packsmith build <project> --target <version>
  packsmith --help
  packsmith --version

commands:
  build    compile a project directory into an installable .zip
";

const BUILD_USAGE: &str = "usage: packsmith build <project> --target <version>";

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

fn build(args: &[String]) -> Result<(), String> {
    let (project, target) = parse_build_args(args)?;
    Err(format!(
        "packsmith build: not implemented until Phase 1.\n\
         The compiler and emitter are not built yet (see docs/ROADMAP.md).\n\
         Parsed: project '{project}', target '{target}'."
    ))
}

fn parse_build_args(args: &[String]) -> Result<(String, String), String> {
    let mut project: Option<String> = None;
    let mut target: Option<String> = None;

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

    let project =
        project.ok_or_else(|| format!("packsmith build: missing <project>\n\n{BUILD_USAGE}"))?;
    let target = target
        .ok_or_else(|| format!("packsmith build: missing --target <version>\n\n{BUILD_USAGE}"))?;
    Ok((project, target))
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
    fn build_accepts_both_target_spellings_in_any_order() {
        let (p, t) = parse_build_args(&argv(&["proj", "--target", "26.2"])).unwrap();
        assert_eq!((p.as_str(), t.as_str()), ("proj", "26.2"));

        let (p, t) = parse_build_args(&argv(&["--target=26.2", "proj"])).unwrap();
        assert_eq!((p.as_str(), t.as_str()), ("proj", "26.2"));
    }

    #[test]
    fn build_validates_before_reporting_not_implemented() {
        let err = build(&argv(&["proj", "--target", "26.2"])).unwrap_err();
        assert!(err.contains("not implemented until Phase 1"));

        let err = build(&argv(&["proj"])).unwrap_err();
        assert!(!err.contains("not implemented"));
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
}
