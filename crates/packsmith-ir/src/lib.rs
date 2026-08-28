#![forbid(unsafe_code)]

//! Packsmith IR: the normalized, target-resolved description of what a build
//! produces, one level above the file tree (`spec/ir.schema.json`).
//!
//! Data only, never code (ADR-0003). The emitter is the only component that
//! knows directory names and file extensions; nothing here names a path.

use serde::{Deserialize, Serialize};

/// A text component: a bare string, an array of components, or an object
/// carrying one content field. Shape is not checked here (`spec/ir.schema.json`
/// `$defs/text`); it is carried through to the emitted document as-is.
pub type Text = serde_json::Value;

/// The whole IR document. `version` is the schema major version and is `0` for
/// every document this crate produces or accepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ir {
    pub version: u8,
    pub target: Target,
    pub packs: Vec<Pack>,
}

/// The resolved build target. Format numbers are not here: they are target data,
/// read by the emitter from the version table (ADR-0006, ADR-0014).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
}

/// One pack this build produces. v1 emits exactly one, of kind `data`
/// (`spec/ir.schema.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pack {
    /// Pack kind, looked up in target data for its root directory and format
    /// number. An open string, never an enum (ADR-0010).
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<Text>,
    /// Everything the pack contains. The emitter sorts by `(category, id)` before
    /// writing (ADR-0007).
    #[serde(default)]
    pub resources: Vec<Resource>,
}

/// One thing a pack contains: a function, a tag, a recipe.
///
/// Empty until the one-function conformance case forces the fields
/// (`category`, `id`, `origin`, `body`) to be modelled against a real
/// consumer (ROADMAP Phase 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {}

/// The stable address of a statement: the node owning the slot, the slot name,
/// and the zero-based index within it (`spec/ir.schema.json`
/// `$defs/statement-address`). `node` is `None` when the statement sits in the
/// graph's own top-level `root` slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementAddress {
    pub node: Option<String>,
    pub slot: String,
    pub index: u32,
}

/// How much a [`Diagnostic`] matters. `warning` never blocks a build; `error`
/// always does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

/// One compiler diagnostic, as a value. Diagnostics are collected and returned
/// as a set rather than raised on the first failure, so the editor can show
/// every problem at once (`.claude/rules/rust.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable machine code, e.g. `legacy-execute-syntax`. `None` marks a
    /// condition the compiler recognises but has not assigned a code to yet;
    /// the conformance contract reads `null` the same way
    /// (`.claude/rules/spec.md`).
    pub code: Option<String>,
    pub severity: Severity,
    /// Where in the graph the diagnostic points.
    pub address: StatementAddress,
    /// A concrete suggested edit, phrased in game terms first (ADR-0009).
    /// `None` when the compiler has no actionable suggestion.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fix: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diagnostic_serialises_with_a_lowercase_severity_and_omits_an_absent_fix() {
        let d = Diagnostic {
            code: None,
            severity: Severity::Warning,
            address: StatementAddress {
                node: None,
                slot: "root".to_string(),
                index: 0,
            },
            fix: None,
        };
        let json = serde_json::to_string(&d).expect("serialises");
        assert!(json.contains(r#""severity":"warning""#));
        assert!(json.contains(r#""node":null"#));
        assert!(!json.contains("fix"));
    }
}
