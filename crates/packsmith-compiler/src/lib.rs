#![forbid(unsafe_code)]

//! Graph to IR.
//!
//! The compiler walks the graph's ordered slots (ADR-0016) and hands each node
//! to `packsmith-blocks`, which lowers it to IR resources or command lines. The
//! result is one `data` pack carrying the project description and every resource
//! the graph produced.
//!
//! Lowering emits the diagnostics this path naturally meets -- an unknown block,
//! a missing required input. A systematic validation pass (types, cross
//! references, command grammar) is the next task and is not done here.

use serde::Deserialize;

use packsmith_blocks::{Node, lower_root};
use packsmith_ir::{Ir, Pack, Target, Text};

pub use packsmith_ir::{Diagnostic, Severity, StatementAddress};

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

/// The outcome of lowering a graph: the IR, plus every diagnostic found. A graph
/// with mistakes still lowers to a best-effort IR so the editor can show the
/// whole picture at once; the CLI refuses to emit when any diagnostic is an
/// error.
#[derive(Debug, Clone)]
pub struct Compilation {
    pub ir: Ir,
    pub diagnostics: Vec<Diagnostic>,
}

/// Lower a graph to IR for `target_id`.
pub fn compile(graph: &Graph, target_id: &str) -> Compilation {
    let _ = &graph.edges;
    let lowered = lower_root(&graph.root);
    let ir = Ir {
        version: 0,
        target: Target {
            id: target_id.to_string(),
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
            resources: lowered.resources,
        }],
    };
    Compilation {
        ir,
        diagnostics: lowered.diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Graph {
        serde_json::from_str(json).expect("valid graph fixture")
    }

    #[test]
    fn empty_root_lowers_to_one_data_pack_with_the_description() {
        let g = parse(
            r#"{ "version": 0,
                "project": { "name": "Empty Pack", "namespace": "example",
                             "description": "An empty Packsmith project." },
                "root": [] }"#,
        );
        let Compilation { ir, diagnostics } = compile(&g, "26.2");
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
        let Compilation { ir, diagnostics } = compile(&g, "26.2");
        assert!(diagnostics.is_empty());
        assert_eq!(ir.packs[0].resources.len(), 1);
        assert_eq!(ir.packs[0].resources[0].id, "example:hello");
    }

    #[test]
    fn an_unknown_block_surfaces_as_a_diagnostic() {
        let g = parse(
            r#"{ "version": 0,
                "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "x", "block": "packsmith/mystery@1.0.0" } ] }"#,
        );
        let out = compile(&g, "26.2");
        assert_eq!(out.diagnostics.len(), 1);
        assert_eq!(out.diagnostics[0].severity, Severity::Error);
    }
}
