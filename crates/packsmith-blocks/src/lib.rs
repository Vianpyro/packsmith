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
//! Systematic validation is `packsmith_compiler::validate`, which runs first and
//! stops the build before lowering when it finds an error. [`describe`] is this
//! crate's half of that: the manifest-shaped view of each built-in it checks a
//! graph against. The diagnostics still emitted here are a backstop for a direct
//! [`lower_root`] call and carry a message but no code-owned fix.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value, json};

use packsmith_ir::{
    Body, Command, Diagnostic, Param, Resource, Severity, StatementAddress, codes, params,
};

/// Minimum Minecraft target the built-ins declare support for, in each built-in
/// manifest's `targets.min`. 26.2 is the only extracted target today; task 14
/// adds older ones and task 15 makes the compiler intersect these ranges.
const BUILTIN_MIN_TARGET: &str = "26.2";

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

/// What a built-in block looks like from the outside: enough of its manifest
/// (ADR-0004, `spec/block-manifest.schema.json`) for the compiler's validation
/// pass to check a graph against it before lowering. The built-ins are a fixed
/// native set, so this is a table, not a parsed manifest; a parsed manifest
/// replaces it when out-of-tree blocks land (Phase 3+).
///
/// This is a second definition of a block's ports alongside
/// `block-manifest.schema.json`. [`BlockDescriptor::to_manifest`] serialises it
/// into that format, and a test validates every built-in against the schema: if
/// a built-in cannot be expressed in the form out-of-tree blocks must use, the
/// form is wrong and we want to know now, not in Phase 3.
#[derive(Debug, Clone, Copy)]
pub struct BlockDescriptor {
    /// The block's `namespace/name` id, without a version.
    pub id: &'static str,
    /// The block's name in game words, for the editor palette and for a
    /// diagnostic that must name it (ADR-0009). Never the `namespace/name` id.
    pub title: &'static str,
    pub node_kind: NodeKind,
    pub inputs: &'static [PortSpec],
    pub slots: &'static [SlotSpec],
    /// When set, the block is only meaningful inside a slot owned by the named
    /// block; anywhere else is `slot-rejects-block`. A `packsmith/command` needs
    /// a `packsmith/function` around it.
    pub requires_parent: Option<&'static str>,
}

impl BlockDescriptor {
    /// Serialise this descriptor into a `block-manifest.schema.json` document.
    ///
    /// `block_version`, `license`, and `targets` are the crate's: the built-ins
    /// ship and are versioned with `packsmith-blocks`. `implementation` names a
    /// declarative template that does not exist -- the built-ins lower in Rust,
    /// and the manifest format has no native-implementation kind (the template
    /// language itself is still unspecified, see `docs/BACKLOG.md`). The point of
    /// this projection is the port, slot, and type shape; that is what the
    /// schema test exercises.
    pub fn to_manifest(&self) -> serde_json::Value {
        let template = format!("{}.tmpl", self.id.rsplit('/').next().unwrap_or(self.id));
        let mut manifest = Map::new();
        manifest.insert("version".to_string(), json!(0));
        manifest.insert("id".to_string(), json!(self.id));
        manifest.insert(
            "block_version".to_string(),
            json!(env!("CARGO_PKG_VERSION")),
        );
        manifest.insert("license".to_string(), json!(env!("CARGO_PKG_LICENSE")));
        manifest.insert("title".to_string(), json!(self.title));
        manifest.insert(
            "node_kind".to_string(),
            json!(match self.node_kind {
                NodeKind::Statement => "statement",
                NodeKind::Value => "value",
            }),
        );
        manifest.insert("targets".to_string(), json!({ "min": BUILTIN_MIN_TARGET }));

        if !self.inputs.is_empty() {
            let inputs: Map<String, Value> = self
                .inputs
                .iter()
                .map(|port| {
                    let mut spec = Map::new();
                    spec.insert("type".to_string(), port.ty.to_type_ref());
                    spec.insert("label".to_string(), json!(port.label));
                    if !port.required {
                        spec.insert("optional".to_string(), json!(true));
                    }
                    (port.name.to_string(), Value::Object(spec))
                })
                .collect();
            manifest.insert("inputs".to_string(), Value::Object(inputs));
        }

        if !self.slots.is_empty() {
            let slots: Map<String, Value> = self
                .slots
                .iter()
                .map(|slot| {
                    (
                        slot.name.to_string(),
                        json!({ "type": { "type": "body" }, "label": slot.label }),
                    )
                })
                .collect();
            manifest.insert("slots".to_string(), Value::Object(slots));
        }

        manifest.insert(
            "implementation".to_string(),
            json!({ "kind": "declarative", "template": template }),
        );
        Value::Object(manifest)
    }
}

/// Whether instances of a block are statement nodes or value nodes
/// (`spec/types.md` section 1). A block is exactly one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Statement,
    Value,
}

/// One input port of a block.
#[derive(Debug, Clone, Copy)]
pub struct PortSpec {
    pub name: &'static str,
    /// What the editor calls this port: game words, not the field name
    /// (ADR-0009). Diagnostics use it.
    pub label: &'static str,
    pub ty: PortType,
    pub required: bool,
}

/// One slot of a block.
#[derive(Debug, Clone, Copy)]
pub struct SlotSpec {
    pub name: &'static str,
    pub label: &'static str,
    /// Block names this slot accepts. Empty means "any statement node": a slot
    /// cannot restrict by statement taxonomy in general (`spec/types.md` section
    /// 4.11), but a block may say which children its own slot expects.
    pub accepts: &'static [&'static str],
}

/// The port types of `spec/types.md`, flattened to what the built-ins use. List
/// element types are spelled out rather than nested: v1 needs only `list<id>`
/// and `list<item_stack>`, and a nested form would buy nothing.
#[derive(Debug, Clone, Copy)]
pub enum PortType {
    Bool,
    Int {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float,
    /// A string. Its `format` is validated by the command-grammar stage
    /// (ADR-0012), not by the validation pass.
    Str,
    Id {
        /// The namespaced registry the id belongs to (`minecraft:function`).
        /// Carried for the manifest and for the membership check that lands with
        /// target data; the validation pass checks syntax only.
        registry: &'static str,
        allow_tag: bool,
    },
    ItemStack,
    Text,
    ListOfId {
        registry: &'static str,
        allow_tag: bool,
    },
    ListOfItemStack,
    Enum(&'static [&'static str]),
}

impl PortType {
    /// This type as a `block-manifest.schema.json` type reference.
    fn to_type_ref(self) -> serde_json::Value {
        match self {
            PortType::Bool => json!({ "type": "bool" }),
            PortType::Int { min, max } => {
                let mut m = Map::new();
                m.insert("type".to_string(), json!("int"));
                if let Some(min) = min {
                    m.insert("min".to_string(), json!(min));
                }
                if let Some(max) = max {
                    m.insert("max".to_string(), json!(max));
                }
                Value::Object(m)
            }
            PortType::Float => json!({ "type": "float" }),
            PortType::Str => json!({ "type": "string" }),
            PortType::Id {
                registry,
                allow_tag,
            } => {
                json!({ "type": "id", "registry": registry, "allow_tag": allow_tag })
            }
            PortType::ItemStack => json!({ "type": "item_stack" }),
            PortType::Text => json!({ "type": "text" }),
            PortType::ListOfId {
                registry,
                allow_tag,
            } => json!({
                "type": "list",
                "of": { "type": "id", "registry": registry, "allow_tag": allow_tag },
            }),
            PortType::ListOfItemStack => {
                json!({ "type": "list", "of": { "type": "item_stack" } })
            }
            PortType::Enum(choices) => json!({ "type": "enum", "choices": choices }),
        }
    }
}

/// The manifest-shaped view of a built-in block, or `None` when no built-in
/// answers to `block_ref`. Version is ignored: the built-ins are versioned with
/// the crate.
pub fn describe(block_ref: &str) -> Option<BlockDescriptor> {
    Some(match block_name(block_ref) {
        "packsmith/function" => BlockDescriptor {
            id: "packsmith/function",
            title: "function",
            node_kind: NodeKind::Statement,
            inputs: &[PortSpec {
                name: "name",
                label: "name",
                ty: PortType::Id {
                    registry: "minecraft:function",
                    allow_tag: false,
                },
                required: true,
            }],
            slots: &[SlotSpec {
                name: "body",
                label: "steps",
                accepts: &["packsmith/command"],
            }],
            requires_parent: None,
        },
        "packsmith/command" => BlockDescriptor {
            id: "packsmith/command",
            title: "command",
            node_kind: NodeKind::Statement,
            inputs: &[PortSpec {
                name: "command",
                label: "command",
                ty: PortType::Str,
                required: true,
            }],
            slots: &[],
            requires_parent: Some("packsmith/function"),
        },
        "packsmith/function-tag" => BlockDescriptor {
            id: "packsmith/function-tag",
            title: "function tag",
            node_kind: NodeKind::Statement,
            inputs: &[
                PortSpec {
                    name: "name",
                    label: "tag name",
                    ty: PortType::Id {
                        registry: "minecraft:function",
                        allow_tag: false,
                    },
                    required: true,
                },
                PortSpec {
                    name: "functions",
                    label: "function list",
                    ty: PortType::ListOfId {
                        registry: "minecraft:function",
                        allow_tag: true,
                    },
                    required: true,
                },
                PortSpec {
                    name: "replace",
                    label: "replace",
                    ty: PortType::Bool,
                    required: false,
                },
            ],
            slots: &[],
            requires_parent: None,
        },
        "packsmith/crafting-shapeless" => BlockDescriptor {
            id: "packsmith/crafting-shapeless",
            title: "shapeless recipe",
            node_kind: NodeKind::Statement,
            inputs: &[
                PortSpec {
                    name: "name",
                    label: "recipe id",
                    ty: PortType::Id {
                        registry: "minecraft:recipe",
                        allow_tag: false,
                    },
                    required: true,
                },
                PortSpec {
                    name: "ingredients",
                    label: "ingredients",
                    ty: PortType::ListOfId {
                        registry: "minecraft:item",
                        allow_tag: true,
                    },
                    required: true,
                },
                PortSpec {
                    name: "result",
                    label: "result item",
                    ty: PortType::ItemStack,
                    required: true,
                },
            ],
            slots: &[],
            requires_parent: None,
        },
        "packsmith/loot-table" => BlockDescriptor {
            id: "packsmith/loot-table",
            title: "loot table",
            node_kind: NodeKind::Statement,
            inputs: &[
                PortSpec {
                    name: "name",
                    label: "loot table id",
                    ty: PortType::Id {
                        registry: "minecraft:loot_table",
                        allow_tag: false,
                    },
                    required: true,
                },
                PortSpec {
                    name: "drops",
                    label: "drops",
                    ty: PortType::ListOfItemStack,
                    required: true,
                },
            ],
            slots: &[],
            requires_parent: None,
        },
        _ => return None,
    })
}

/// Every built-in block ref, for tests and for a future palette. Kept beside
/// [`describe`] so a new built-in is added in one place.
pub const BUILTIN_IDS: &[&str] = &[
    "packsmith/function",
    "packsmith/command",
    "packsmith/function-tag",
    "packsmith/crafting-shapeless",
    "packsmith/loot-table",
];

/// The block's title (game words) for `block_ref`, or its bare `namespace/name`
/// when it is not a built-in -- a diagnostic still needs something to call it.
pub fn display_name(block_ref: &str) -> String {
    describe(block_ref)
        .map(|d| d.title.to_string())
        .unwrap_or_else(|| block_name(block_ref).to_string())
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
        "packsmith/command" => out.diagnostics.push(diagnostic(
            codes::SLOT_REJECTS_BLOCK,
            at,
            params! {
                "reason" => "needs_parent",
                "block" => display_name("packsmith/command"),
                "parent" => display_name("packsmith/function"),
            },
        )),
        _ => out.diagnostics.push(diagnostic(
            codes::BLOCK_UNKNOWN,
            at,
            params! { "block" => node.block.clone() },
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
        out.diagnostics.push(diagnostic(
            codes::SLOT_REJECTS_BLOCK,
            at,
            params! {
                "reason" => "not_accepted",
                "block" => display_name(&node.block),
                "accepts" => vec![display_name("packsmith/command")],
            },
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
        out.diagnostics.push(diagnostic(
            codes::INPUT_MISSING,
            at,
            params! {
                "block" => display_name(&node.block),
                "label" => port_label(&node.block, "result"),
            },
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

/// A lowering-time diagnostic. The systematic pass in `packsmith-compiler` runs
/// first and owns every condition reachable here (`spec/diagnostics.md`); this
/// copy is a backstop for a direct `lower_root` call. It carries the same code
/// and parameters, so `packsmith_ir::message::render` words it the same way.
fn diagnostic(
    code: &str,
    address: StatementAddress,
    params: std::collections::BTreeMap<String, Param>,
) -> Diagnostic {
    Diagnostic {
        code: Some(code.to_string()),
        severity: Severity::Error,
        address,
        params,
    }
}

/// The `label` of `port` on `block_ref`, or the port name when it is not a
/// built-in port.
fn port_label(block_ref: &str, port: &str) -> String {
    describe(block_ref)
        .and_then(|d| d.inputs.iter().find(|p| p.name == port))
        .map(|p| p.label.to_string())
        .unwrap_or_else(|| port.to_string())
}

fn missing_input(node: &Node, port: &str, at: &StatementAddress, out: &mut Lowered) {
    out.diagnostics.push(diagnostic(
        codes::INPUT_MISSING,
        at.clone(),
        params! {
            "block" => display_name(&node.block),
            "label" => port_label(&node.block, port),
        },
    ));
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
            missing_input(node, port, at, out);
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
            missing_input(node, port, at, out);
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
    fn every_built_in_is_listed_and_describable() {
        for id in BUILTIN_IDS {
            let d = describe(id).unwrap_or_else(|| panic!("{id} is listed but not described"));
            assert_eq!(d.id, *id);
            assert!(!d.title.is_empty());
            assert!(
                !d.title.contains('/'),
                "{id} title is an id, not game words"
            );
        }
    }

    #[test]
    fn every_built_in_manifest_validates_against_the_schema() {
        let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../spec/block-manifest.schema.json");
        let schema: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&schema_path).expect("read schema"))
                .expect("schema is JSON");
        let loc = schema["$id"].as_str().expect("schema has $id").to_string();

        let mut compiler = boon::Compiler::new();
        compiler
            .add_resource(&loc, schema.clone())
            .expect("add schema resource");
        let mut schemas = boon::Schemas::new();
        let idx = compiler
            .compile(&loc, &mut schemas)
            .expect("compile schema");

        for id in BUILTIN_IDS {
            let manifest = describe(id).expect("built-in").to_manifest();
            if let Err(e) = schemas.validate(&manifest, idx) {
                panic!(
                    "{id} does not fit block-manifest.schema.json:\n{e}\n\nmanifest was:\n{}",
                    serde_json::to_string_pretty(&manifest).unwrap()
                );
            }
        }
    }

    #[test]
    fn describe_covers_the_built_ins_and_only_them() {
        assert!(describe("packsmith/function@1.0.0").is_some());
        assert!(describe("packsmith/command@9.9.9").is_some());
        assert!(describe("packsmith/nope@1.0.0").is_none());

        let function = describe("packsmith/function@1.0.0").expect("built-in");
        assert_eq!(function.node_kind, NodeKind::Statement);
        assert_eq!(function.slots.len(), 1);
        assert_eq!(function.slots[0].name, "body");
        assert!(
            function
                .inputs
                .iter()
                .any(|p| p.name == "name" && p.required)
        );

        let command = describe("packsmith/command@1.0.0").expect("built-in");
        assert_eq!(command.requires_parent, Some("packsmith/function"));
    }

    #[test]
    fn an_unknown_block_is_a_diagnostic_at_its_address() {
        let out = lower_root(&[node(r#"{ "id": "x", "block": "packsmith/nope@1.0.0" }"#)]);
        assert!(out.resources.is_empty());
        assert_eq!(
            out.diagnostics[0].code.as_deref(),
            Some(codes::BLOCK_UNKNOWN)
        );
        assert_eq!(out.diagnostics[0].address.index, 0);
    }

    #[test]
    fn a_function_missing_its_name_is_a_missing_input_diagnostic() {
        let out = lower_root(&[node(
            r#"{ "id": "fn", "block": "packsmith/function@1.0.0" }"#,
        )]);
        assert!(out.resources.is_empty());
        assert_eq!(
            out.diagnostics[0].code.as_deref(),
            Some(codes::INPUT_MISSING)
        );
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
