#![forbid(unsafe_code)]

//! The built-in declarative blocks and how each one lowers to IR.
//!
//! A declarative block is native and trusted (ADR-0004). It turns one graph node
//! into IR resources, or into command lines inside a function body. This crate
//! is the lowering half of `packsmith-compiler`: the compiler walks the graph's
//! ordered slots (ADR-0016) and hands each node here.
//!
//! This crate depends only on `packsmith-ir`; it is a leaf like
//! `packsmith-mcversion`, and `packsmith-compiler` depends on it.
//!
//! No systematic validation happens here. An error this path naturally meets --
//! an unknown block, a missing required input, a node that cannot appear where
//! it was placed -- becomes a diagnostic. A full validation pass is a later task.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value, json};

use packsmith_ir::{Body, Command, Diagnostic, Resource, Severity, StatementAddress};

/// Minecraft version facts about the JSON a built-in block emits. These are shapes
/// written from memory, which `.claude/rules/minecraft.md` forbids; they are named
/// here so they are greppable when the schema validator of ADR-0019 lands and can
/// check emitted JSON against the target instead of trusting these literals.
mod target_shape {
    /// `type` of a shapeless crafting recipe (`recipe` category).
    pub const CRAFTING_SHAPELESS_RECIPE: &str = "minecraft:crafting_shapeless";
    /// `type` of a loot pool entry that yields a fixed item (`loot_table` category).
    pub const LOOT_ITEM_ENTRY: &str = "minecraft:item";
}

/// One instance of a block in the graph (`spec/graph.schema.json` `$defs/node`).
/// `inputs` and `slots` are captured loosely; the type each input must hold is
/// the block's business, checked (later) against the block manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct Node {
    pub id: String,
    pub block: String,
    #[serde(default)]
    pub inputs: Map<String, Value>,
    #[serde(default)]
    pub slots: BTreeMap<String, Vec<Node>>,
}

/// What lowering a list of statement nodes produced: the resources they emit and
/// every diagnostic met on the way. Diagnostics are collected, not raised on the
/// first failure, so the editor can show them all at once.
#[derive(Debug, Default)]
pub struct Lowered {
    pub resources: Vec<Resource>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Lower the graph's top-level `root` slot. A statement at index `i` here has the
/// address `(null, "root", i)` (`spec/ir.schema.json` `$defs/statement-address`).
pub fn lower_root(nodes: &[Node]) -> Lowered {
    let mut out = Lowered::default();
    for (i, node) in nodes.iter().enumerate() {
        lower_statement(node, address(None, "root", i), &mut out);
    }
    out
}

fn lower_statement(node: &Node, at: StatementAddress, out: &mut Lowered) {
    match block_name(&node.block) {
        "packsmith/function" => lower_function(node, at, out),
        "packsmith/function-tag" => lower_function_tag(node, at, out),
        "packsmith/crafting-shapeless" => lower_crafting_shapeless(node, at, out),
        "packsmith/loot-table" => lower_loot_table(node, at, out),
        "packsmith/command" => out.diagnostics.push(error(
            "command-outside-body",
            at,
            "move this command into a function's body slot",
        )),
        _ => out.diagnostics.push(error(
            "unknown-block",
            at,
            &format!("there is no built-in block '{}'", node.block),
        )),
    }
}

/// `packsmith/function`: a statement with a `name` id and a `body` slot. Its
/// children lower to an ordered `commands` body; the child at index `j` keeps
/// the address `(<this node>, "body", j)`.
fn lower_function(node: &Node, at: StatementAddress, out: &mut Lowered) {
    let Some(name) = require_str(node, "name", &at, out).map(str::to_string) else {
        return;
    };

    let mut statements = Vec::new();
    for (j, child) in node.slots.get("body").into_iter().flatten().enumerate() {
        lower_command_child(
            child,
            address(Some(&node.id), "body", j),
            &mut statements,
            out,
        );
    }

    out.resources.push(Resource {
        category: "function".to_string(),
        id: name,
        origin: at,
        body: Body::Commands { statements },
    });
}

fn lower_command_child(
    node: &Node,
    at: StatementAddress,
    into: &mut Vec<Command>,
    out: &mut Lowered,
) {
    if block_name(&node.block) != "packsmith/command" {
        out.diagnostics.push(error(
            "unsupported-in-body",
            at,
            "a function body holds command blocks",
        ));
        return;
    }
    let Some(command) = require_str(node, "command", &at, out).map(str::to_string) else {
        return;
    };
    into.push(Command::Text {
        command,
        origin: at,
    });
}

/// `packsmith/function-tag`: a `name` id and a `functions` list, optional
/// `replace` bool. Lowers to a `tags/function` JSON body. `replace` is written
/// only when it is `true`, so the default stays absent.
fn lower_function_tag(node: &Node, at: StatementAddress, out: &mut Lowered) {
    let Some(name) = require_str(node, "name", &at, out).map(str::to_string) else {
        return;
    };

    let functions: Vec<Value> = node
        .inputs
        .get("functions")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|s| json!(s))
                .collect()
        })
        .unwrap_or_default();

    let mut value = Map::new();
    value.insert("values".to_string(), Value::Array(functions));
    if node.inputs.get("replace").and_then(Value::as_bool) == Some(true) {
        value.insert("replace".to_string(), Value::Bool(true));
    }

    out.resources.push(Resource {
        category: "tags/function".to_string(),
        id: name,
        origin: at,
        body: Body::Json {
            value: Value::Object(value),
        },
    });
}

/// `packsmith/crafting-shapeless`: `name` id, `ingredients` list of item ids,
/// `result` item stack. Lowers to a `recipe` JSON body.
fn lower_crafting_shapeless(node: &Node, at: StatementAddress, out: &mut Lowered) {
    let Some(name) = require_str(node, "name", &at, out).map(str::to_string) else {
        return;
    };
    let Some(ingredients) = require_list(node, "ingredients", &at, out) else {
        return;
    };
    let Some(result) = node.inputs.get("result").cloned() else {
        out.diagnostics.push(error(
            "missing-input",
            at,
            "this recipe needs a 'result' item",
        ));
        return;
    };

    let value = json!({
        "type": target_shape::CRAFTING_SHAPELESS_RECIPE,
        "ingredients": ingredients,
        "result": item_stack_json(&result),
    });

    out.resources.push(Resource {
        category: "recipe".to_string(),
        id: name,
        origin: at,
        body: Body::Json { value },
    });
}

/// `packsmith/loot-table`: `name` id, `drops` list of item stacks. Lowers to a
/// `loot_table` JSON body of one pool with one entry per drop. Nothing the input
/// did not ask for -- no `count`, `functions`, or `conditions` -- is invented.
fn lower_loot_table(node: &Node, at: StatementAddress, out: &mut Lowered) {
    let Some(name) = require_str(node, "name", &at, out).map(str::to_string) else {
        return;
    };
    let Some(drops) = require_list(node, "drops", &at, out) else {
        return;
    };

    let entries: Vec<Value> = drops
        .iter()
        .filter_map(|d| d.get("item").and_then(Value::as_str))
        .map(|item| json!({ "type": target_shape::LOOT_ITEM_ENTRY, "name": item }))
        .collect();

    out.resources.push(Resource {
        category: "loot_table".to_string(),
        id: name,
        origin: at,
        body: Body::Json {
            value: json!({ "pools": [ { "rolls": 1, "entries": entries } ] }),
        },
    });
}

/// Convert an `item_stack` literal (`{item, count?, components?}`) to the shape
/// a recipe result / loot entry takes: `{id, count?}`. `count` is carried only
/// when the input gave one.
fn item_stack_json(v: &Value) -> Value {
    let mut m = Map::new();
    if let Some(item) = v.get("item").and_then(Value::as_str) {
        m.insert("id".to_string(), json!(item));
    }
    if let Some(count) = v.get("count").and_then(Value::as_i64) {
        m.insert("count".to_string(), json!(count));
    }
    if let Some(components) = v
        .get("components")
        .filter(|c| c.as_object().is_some_and(|o| !o.is_empty()))
    {
        m.insert("components".to_string(), components.clone());
    }
    Value::Object(m)
}

fn block_name(block_ref: &str) -> &str {
    block_ref.split_once('@').map_or(block_ref, |(n, _)| n)
}

fn address(node: Option<&str>, slot: &str, index: usize) -> StatementAddress {
    StatementAddress {
        node: node.map(str::to_string),
        slot: slot.to_string(),
        index: index as u32,
    }
}

fn error(code: &str, address: StatementAddress, fix: &str) -> Diagnostic {
    Diagnostic {
        code: Some(code.to_string()),
        severity: Severity::Error,
        address,
        fix: Some(fix.to_string()),
    }
}

fn require_str<'a>(
    node: &'a Node,
    port: &str,
    at: &StatementAddress,
    out: &mut Lowered,
) -> Option<&'a str> {
    match node.inputs.get(port).and_then(Value::as_str) {
        Some(s) => Some(s),
        None => {
            out.diagnostics.push(error(
                "missing-input",
                at.clone(),
                &format!("this block needs a '{port}' value"),
            ));
            None
        }
    }
}

fn require_list(
    node: &Node,
    port: &str,
    at: &StatementAddress,
    out: &mut Lowered,
) -> Option<Vec<Value>> {
    match node.inputs.get(port).and_then(Value::as_array) {
        Some(a) => Some(a.clone()),
        None => {
            out.diagnostics.push(error(
                "missing-input",
                at.clone(),
                &format!("this block needs a '{port}' list"),
            ));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(json: &str) -> Node {
        serde_json::from_str(json).expect("valid node fixture")
    }

    #[test]
    fn a_function_with_one_command_lowers_to_one_commands_resource() {
        let n = node(
            r#"{ "id": "fn-hello", "block": "packsmith/function@1.0.0",
                 "inputs": { "name": "example:hello" },
                 "slots": { "body": [
                   { "id": "cmd-say", "block": "packsmith/command@1.0.0",
                     "inputs": { "command": "say Hello, world!" } } ] } }"#,
        );
        let out = lower_root(&[n]);
        assert!(out.diagnostics.is_empty());
        assert_eq!(out.resources.len(), 1);
        let r = &out.resources[0];
        assert_eq!(r.category, "function");
        assert_eq!(r.id, "example:hello");
        assert_eq!(r.origin.slot, "root");
        match &r.body {
            Body::Commands { statements } => {
                let Command::Text { command, origin } = &statements[0];
                assert_eq!(command, "say Hello, world!");
                assert_eq!(origin.node.as_deref(), Some("fn-hello"));
                assert_eq!((origin.slot.as_str(), origin.index), ("body", 0));
            }
            Body::Json { .. } => panic!("a function is a commands body"),
        }
    }

    #[test]
    fn an_unknown_block_is_a_diagnostic_at_its_address() {
        let out = lower_root(&[node(r#"{ "id": "x", "block": "packsmith/nope@1.0.0" }"#)]);
        assert!(out.resources.is_empty());
        assert_eq!(out.diagnostics[0].code.as_deref(), Some("unknown-block"));
        assert_eq!(out.diagnostics[0].address.index, 0);
    }

    #[test]
    fn a_function_missing_its_name_is_a_missing_input_diagnostic() {
        let out = lower_root(&[node(
            r#"{ "id": "fn", "block": "packsmith/function@1.0.0" }"#,
        )]);
        assert!(out.resources.is_empty());
        assert_eq!(out.diagnostics[0].code.as_deref(), Some("missing-input"));
    }

    #[test]
    fn a_function_tag_writes_replace_only_when_true() {
        let out = lower_root(&[node(
            r#"{ "id": "t", "block": "packsmith/function-tag@1.0.0",
                 "inputs": { "name": "minecraft:load", "functions": ["example:hello"] } }"#,
        )]);
        let Body::Json { value } = &out.resources[0].body else {
            panic!("a tag is a json body");
        };
        assert_eq!(value, &json!({ "values": ["example:hello"] }));
    }

    #[test]
    fn a_shapeless_recipe_keeps_an_explicit_result_count() {
        let out = lower_root(&[node(
            r#"{ "id": "r", "block": "packsmith/crafting-shapeless@1.0.0",
                 "inputs": { "name": "example:d", "ingredients": ["minecraft:diamond_block"],
                             "result": { "item": "minecraft:diamond", "count": 9 } } }"#,
        )]);
        let Body::Json { value } = &out.resources[0].body else {
            panic!("a recipe is a json body");
        };
        assert_eq!(
            value["result"],
            json!({ "id": "minecraft:diamond", "count": 9 })
        );
    }

    #[test]
    fn a_loot_drop_with_no_count_invents_no_keys() {
        let out = lower_root(&[node(
            r#"{ "id": "l", "block": "packsmith/loot-table@1.0.0",
                 "inputs": { "name": "example:blocks/x", "drops": [ { "item": "minecraft:diamond" } ] } }"#,
        )]);
        let Body::Json { value } = &out.resources[0].body else {
            panic!("a loot table is a json body");
        };
        assert_eq!(
            value,
            &json!({ "pools": [ { "rolls": 1, "entries": [ { "type": "minecraft:item", "name": "minecraft:diamond" } ] } ] })
        );
    }
}
