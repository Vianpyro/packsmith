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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {}
