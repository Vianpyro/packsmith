#![forbid(unsafe_code)]

//! Graph to IR.
//!
//! The compiler walks the graph's ordered slots (ADR-0016) and hands each node
//! to `packsmith-blocks`, which lowers it to IR resources or command lines. The
//! result is one `data` pack carrying the project description and every resource
//! the graph produced.
//!
//! Before lowering, [`validate`] walks the graph once and collects every
//! diagnostic its shape can carry -- unknown blocks, missing or ill-typed
//! inputs, misplaced nodes, broken data edges (`spec/diagnostics.md`). After
//! lowering, [`command::validate`] walks each emitted command line against the
//! target's Brigadier tree (ADR-0012). An error from either stops the build:
//! the graph is not lowered, or the lowered resources are dropped, and the
//! diagnostics are reported from one place.

mod command;
mod validate;

use serde::Deserialize;

use packsmith_blocks::{Node, lower_root};
use packsmith_ir::{Ir, Pack, Target, Text};
use packsmith_mcversion::TargetData;

pub use packsmith_ir::{Diagnostic, Param, Severity, StatementAddress, message};
pub use validate::validate;

/// A graph document (`spec/graph.schema.json`). `root` is the top-level ordered
/// slot; `edges` are captured but not interpreted yet (v1 function bodies are
/// wire-free, ADR-0012).
#[derive(Debug, Clone, Deserialize)]
pub struct Graph {
    pub version: u8,
    pub project: Project,
    #[serde(default)]
    pub root: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<serde_json::Value>,
}

/// Project metadata. The build target is not here: it is a compile parameter
/// (ADR-0006).
#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name: String,
    pub namespace: String,
    #[serde(default)]
    pub description: Option<Text>,
}

/// The outcome of compiling a graph: the IR, plus every diagnostic found.
///
/// A graph that fails validation is not lowered: `ir` then holds the one `data`
/// pack with its description and no resources, and `diagnostics` holds the
/// validation errors. A graph that passes is lowered node by node and any
/// diagnostic met there is collected too. Either way the CLI refuses to emit
/// when a diagnostic is an error.
#[derive(Debug, Clone)]
pub struct Compilation {
    pub ir: Ir,
    pub diagnostics: Vec<Diagnostic>,
}

/// Compile a graph to IR for `target`: validate the graph, lower it if clean,
/// then check its command lines against the target's grammar (ADR-0012). Any
/// error leaves the pack with no resources; the CLI refuses to emit either way.
pub fn compile(graph: &Graph, target: &TargetData) -> Compilation {
    let mut diagnostics = validate(graph);

    let mut resources = if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        Vec::new()
    } else {
        let lowered = lower_root(&graph.root);
        diagnostics.extend(lowered.diagnostics);
        diagnostics.extend(command::validate(&lowered.resources, target));
        lowered.resources
    };

    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        resources.clear();
    }

    let ir = Ir {
        version: 0,
        target: Target {
            id: target.target().to_string(),
        },
        // v1 emits exactly one pack, of kind "data" (spec/ir.schema.json). The
        // kind is a string the emitter looks up in target data, not a hardcoded
        // directory name (ADR-0006, ADR-0010).
        packs: vec![Pack {
            kind: "data".to_string(),
            // The game's pack list shows this, and a data pack whose pack.mcmeta
            // carries no `description` is silently rejected. Fall back to the
            // project name so a project that never set one still loads.
            description: Some(
                graph
                    .project
                    .description
                    .clone()
                    .unwrap_or_else(|| Text::from(graph.project.name.clone())),
            ),
            resources,
        }],
    };
    Compilation { ir, diagnostics }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Graph {
        serde_json::from_str(json).expect("valid graph fixture")
    }

    fn target() -> TargetData {
        TargetData::load(&packsmith_mcversion::bundled_data_dir(), "26.2")
            .expect("26.2 data ships with packsmith-mcversion")
    }

    #[test]
    fn empty_root_lowers_to_one_data_pack_with_the_description() {
        let g = parse(
            r#"{ "version": 0,
                "project": { "name": "Empty Pack", "namespace": "example",
                             "description": "An empty Packsmith project." },
                "root": [] }"#,
        );
        let Compilation { ir, diagnostics } = compile(&g, &target());
        assert!(diagnostics.is_empty());
        assert_eq!(ir.target.id, "26.2");
        assert_eq!(ir.packs.len(), 1);
        assert_eq!(ir.packs[0].kind, "data");
        assert_eq!(
            ir.packs[0].description.as_ref().and_then(|d| d.as_str()),
            Some("An empty Packsmith project.")
        );
        assert!(ir.packs[0].resources.is_empty());
    }

    #[test]
    fn a_function_graph_lowers_to_one_function_resource() {
        let g = parse(
            r#"{ "version": 0,
                "project": { "name": "One Function", "namespace": "example" },
                "root": [
                  { "id": "fn-hello", "block": "packsmith/function@1.0.0",
                    "inputs": { "name": "example:hello" },
                    "slots": { "body": [
                      { "id": "cmd-say", "block": "packsmith/command@1.0.0",
                        "inputs": { "command": "say Hello, world!" } } ] } } ] }"#,
        );
        let Compilation { ir, diagnostics } = compile(&g, &target());
        assert!(diagnostics.is_empty());
        assert_eq!(ir.packs[0].resources.len(), 1);
        assert_eq!(ir.packs[0].resources[0].id, "example:hello");
    }

    #[test]
    fn a_legacy_command_stops_the_build_with_a_command_diagnostic() {
        let g = parse(
            r#"{ "version": 0,
                "project": { "name": "Legacy", "namespace": "example" },
                "root": [
                  { "id": "fn-legacy", "block": "packsmith/function@1.0.0",
                    "inputs": { "name": "example:legacy" },
                    "slots": { "body": [
                      { "id": "c", "block": "packsmith/command@1.0.0",
                        "inputs": { "command": "execute @e[type=zombie] ~ ~ ~ say boo" } } ] } } ] }"#,
        );
        let out = compile(&g, &target());
        assert_eq!(
            out.diagnostics
                .iter()
                .filter_map(|d| d.code.as_deref())
                .collect::<Vec<_>>(),
            [packsmith_ir::codes::COMMAND_LEGACY_SYNTAX]
        );
        assert_eq!(
            out.diagnostics[0].address.node.as_deref(),
            Some("fn-legacy")
        );
        assert!(
            out.ir.packs[0].resources.is_empty(),
            "a failed command check must emit no resources"
        );
    }

    #[test]
    fn a_raw_mcfunction_file_lowers_with_comments_and_blanks_kept() {
        let g = parse(
            r##"{ "version": 0,
                "project": { "name": "Raw", "namespace": "example" },
                "root": [
                  { "id": "fn-raw", "block": "packsmith/mcfunction@1.0.0",
                    "inputs": { "name": "example:raw",
                                "source": "# a note\nsay first line\n\nsay second line\n" } } ] }"##,
        );
        let out = compile(&g, &target());
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);
        let body = &out.ir.packs[0].resources[0].body;
        let packsmith_ir::Body::Commands { statements } = body else {
            panic!("a function is a commands body");
        };
        assert_eq!(statements.len(), 4, "comment and blank line are kept");
    }

    #[test]
    fn an_unknown_block_surfaces_as_a_diagnostic() {
        let g = parse(
            r#"{ "version": 0,
                "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "x", "block": "packsmith/mystery@1.0.0" } ] }"#,
        );
        let out = compile(&g, &target());
        assert_eq!(out.diagnostics.len(), 1);
        assert_eq!(out.diagnostics[0].severity, Severity::Error);
    }
}
