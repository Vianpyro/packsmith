#![forbid(unsafe_code)]

//! Graph to IR.
//!
//! Phase 1 slice: lower a project with an empty `root` slot to the IR for a
//! single `data` pack carrying the project description. An empty `root` is a
//! valid, silent program, not an error (ADR-0016).

use serde::Deserialize;

use packsmith_ir::{Ir, Pack, Target, Text};

/// A graph document (`spec/graph.schema.json`), parsed only as far as this slice
/// needs. `root` and `edges` are captured untyped so a malformed one still
/// round-trips; they are not interpreted yet.
#[derive(Debug, Clone, Deserialize)]
pub struct Graph {
    pub version: u8,
    pub project: Project,
    #[serde(default)]
    pub root: Vec<serde_json::Value>,
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

/// Lower a graph to IR for `target_id`.
///
/// Infallible for this slice: an empty `root` cannot fail to compile. The first
/// failure mode arrives with graph validation, and returns a `Result` then.
pub fn compile(graph: &Graph, target_id: &str) -> Ir {
    let _ = (&graph.root, &graph.edges);
    Ir {
        version: 0,
        target: Target {
            id: target_id.to_string(),
        },
        // v1 emits exactly one pack, of kind "data" (spec/ir.schema.json). The
        // kind is a string looked up in target data by the emitter, not a
        // hardcoded directory name (ADR-0006, ADR-0010).
        packs: vec![Pack {
            kind: "data".to_string(),
            description: graph.project.description.clone(),
            resources: Vec::new(),
        }],
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
        let ir = compile(&g, "26.2");
        assert_eq!(ir.version, 0);
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
    fn a_missing_description_stays_absent() {
        let g = parse(
            r#"{ "version": 0,
                "project": { "name": "P", "namespace": "example" },
                "root": [] }"#,
        );
        let ir = compile(&g, "26.2");
        assert!(ir.packs[0].description.is_none());
    }
}
