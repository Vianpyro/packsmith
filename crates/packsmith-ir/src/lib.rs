#![forbid(unsafe_code)]

//! Packsmith IR: the normalized, target-resolved description of what a build
//! produces, one level above the file tree (`spec/ir.schema.json`).
//!
//! Data only, never code (ADR-0003). The emitter is the only component that
//! knows directory names and file extensions; nothing here names a path.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod codes;
pub mod message;

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
/// `category` is a free-form slash-separated string resolved through the target
/// data table, never an enum (ADR-0010). `origin` is the statement address the
/// resource was lowered from, so a diagnostic about it can name the block the
/// user placed (ADR-0009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub category: String,
    pub id: String,
    pub origin: StatementAddress,
    pub body: Body,
}

/// The content of a resource, as a tagged form (`spec/ir.schema.json`
/// `$defs/body`). `commands` carries an ordered statement list; `json` carries a
/// document the game reads as JSON. New forms are added as further variants
/// without a breaking change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "form", rename_all = "lowercase")]
pub enum Body {
    /// An ordered list of command lines. Order is significant and is exactly the
    /// order of the slot it was lowered from.
    Commands { statements: Vec<Command> },
    /// A JSON document. Object key order is not significant; the emitter writes
    /// keys sorted (`packsmith-emit`).
    Json { value: serde_json::Value },
}

/// One command line, as a tagged form (`spec/ir.schema.json` `$defs/command`).
/// `text` is the only form in v1: the command is text, carried through verbatim
/// (ADR-0012). Command grammar validation is a later task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "form", rename_all = "lowercase")]
pub enum Command {
    Text {
        /// One command line, no leading slash, no newline.
        command: String,
        /// The statement this line came from. Several lines may share one origin.
        origin: StatementAddress,
    },
}

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
///
/// A diagnostic carries no rendered sentence. It names a condition (`code`) and
/// the facts that describe this occurrence of it (`params`); the wording the two
/// personas read (ADR-0009) is produced from a per-code template table by
/// [`message::render`], which is the only place English lives and the unit a
/// translation replaces. Conformance asserts `code`, `severity`, and `address`;
/// `params` is available to a case that wants to pin a fact but the wording is
/// never asserted (`.claude/rules/spec.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable machine code, one of [`codes`]. `None` marks a condition the
    /// compiler recognises but has not assigned a code to yet; the conformance
    /// contract reads `null` the same way (`.claude/rules/spec.md`).
    pub code: Option<String>,
    pub severity: Severity,
    /// Where in the graph the diagnostic points.
    pub address: StatementAddress,
    /// The facts of this occurrence, keyed by name: a block's display name, a
    /// port label, a bound, a list of choices. The template for `code` reads
    /// them; a missing one renders as a neutral fallback rather than an error.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, Param>,
}

/// One fact attached to a [`Diagnostic`]: a string, a whole number, or a list of
/// strings. Deliberately small -- diagnostics describe graph shape, not
/// arbitrary values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Param {
    Text(String),
    Int(i64),
    List(Vec<String>),
}

impl Param {
    /// The string value, or `""` for a number or list.
    pub fn text(&self) -> &str {
        match self {
            Param::Text(s) => s,
            _ => "",
        }
    }

    /// The number value, or `0` for a string or list.
    pub fn int(&self) -> i64 {
        match self {
            Param::Int(n) => *n,
            _ => 0,
        }
    }

    /// The list value, or `&[]` for a string or number.
    pub fn list(&self) -> &[String] {
        match self {
            Param::List(items) => items,
            _ => &[],
        }
    }
}

impl From<&str> for Param {
    fn from(value: &str) -> Self {
        Param::Text(value.to_string())
    }
}

impl From<String> for Param {
    fn from(value: String) -> Self {
        Param::Text(value)
    }
}

impl From<i64> for Param {
    fn from(value: i64) -> Self {
        Param::Int(value)
    }
}

impl From<Vec<String>> for Param {
    fn from(value: Vec<String>) -> Self {
        Param::List(value)
    }
}

/// Build a `BTreeMap<String, Param>` for a [`Diagnostic`]:
/// `params!{ "block" => name, "min" => 1_i64 }`.
#[macro_export]
macro_rules! params {
    () => { ::std::collections::BTreeMap::<::std::string::String, $crate::Param>::new() };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::BTreeMap::<::std::string::String, $crate::Param>::new();
        $( map.insert(($key).to_string(), $crate::Param::from($value)); )+
        map
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_commands_body_and_a_json_body_carry_their_form_tag() {
        let addr = StatementAddress {
            node: Some("fn-hello".to_string()),
            slot: "body".to_string(),
            index: 0,
        };
        let commands = Body::Commands {
            statements: vec![Command::Text {
                command: "say hi".to_string(),
                origin: addr.clone(),
            }],
        };
        let json = serde_json::to_string(&commands).expect("serialises");
        assert!(json.contains(r#""form":"commands""#));
        assert!(json.contains(r#""form":"text""#));

        let doc = Body::Json {
            value: serde_json::json!({ "values": ["example:hello"] }),
        };
        assert_eq!(
            serde_json::to_string(&doc).expect("serialises"),
            r#"{"form":"json","value":{"values":["example:hello"]}}"#
        );
    }

    #[test]
    fn a_diagnostic_serialises_with_a_lowercase_severity_and_omits_empty_params() {
        let d = Diagnostic {
            code: None,
            severity: Severity::Warning,
            address: StatementAddress {
                node: None,
                slot: "root".to_string(),
                index: 0,
            },
            params: BTreeMap::new(),
        };
        let json = serde_json::to_string(&d).expect("serialises");
        assert!(json.contains(r#""severity":"warning""#));
        assert!(json.contains(r#""node":null"#));
        assert!(!json.contains("params"));
    }

    #[test]
    fn params_carry_their_json_type_and_read_back() {
        let p = crate::params! {
            "block" => "packsmith/function",
            "min" => 1_i64,
            "choices" => vec!["less_than".to_string(), "greater_than".to_string()],
        };
        assert_eq!(p["block"].text(), "packsmith/function");
        assert_eq!(p["min"].int(), 1);
        assert_eq!(p["choices"].list().len(), 2);

        let json = serde_json::to_string(&p).expect("serialises");
        assert!(json.contains(r#""block":"packsmith/function""#));
        assert!(json.contains(r#""min":1"#));
        assert!(json.contains(r#""choices":["less_than","greater_than"]"#));
    }
}
