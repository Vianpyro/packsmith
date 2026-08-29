//! The validation pass: walk the graph once, before lowering, and collect every
//! diagnostic the shape of the graph can carry -- an unknown block, a missing
//! required input, a literal that is the wrong type or breaks its type's rules,
//! a node placed where it cannot act, a data edge to nowhere.
//!
//! It does not check command grammar: that is the Brigadier stage (ADR-0012),
//! the next task, and it needs the extracted command tree this pass never
//! touches. It does not check anything against target data either -- registry
//! membership, block property values -- because `spec/types.md` puts those
//! behind extracted data that may not exist for a given registry.
//!
//! Every diagnostic carries a code from [`packsmith_ir::codes`], a severity, the
//! statement address it points at, and typed `params` -- the facts of this
//! occurrence. No sentence is built here: `packsmith_ir::message::render` turns
//! code plus params into the wording and the fix, in one place (ADR-0009). Codes
//! are stable: `spec/diagnostics.md` is the normative list and conformance cases
//! assert on them.

use std::collections::{BTreeMap, HashSet};

use serde::Deserialize;
use serde_json::Value;

use packsmith_blocks::{
    BlockDescriptor, Node, NodeKind, PortSpec, PortType, describe, display_name,
};
use packsmith_ir::{Diagnostic, Param, Severity, StatementAddress, codes, params};

use crate::Graph;

/// Validate `graph` and return every diagnostic found, in document order for the
/// node walk followed by the edge checks. An empty result means the graph is
/// well-formed enough to lower; the caller still refuses to emit if any
/// diagnostic is an error.
pub fn validate(graph: &Graph) -> Vec<Diagnostic> {
    let edges = parse_edges(&graph.edges);
    let fed: HashSet<(String, String)> = edges
        .iter()
        .map(|e| (e.to.node.clone(), e.to.port.clone()))
        .collect();

    let mut walk = Walk {
        diagnostics: Vec::new(),
        nodes: BTreeMap::new(),
        position: 0,
        fed: &fed,
    };
    walk.slot(&graph.root, None, "root", None, &[]);
    walk.check_edges(&edges);
    walk.diagnostics
}

struct Walk<'a> {
    diagnostics: Vec<Diagnostic>,
    /// Every node seen so far, by id: its document position and its address.
    nodes: BTreeMap<String, NodeInfo>,
    position: u32,
    /// `(node id, port name)` pairs that an edge delivers a value to. A required
    /// port fed by an edge is not "missing".
    fed: &'a HashSet<(String, String)>,
}

struct NodeInfo {
    position: u32,
    address: StatementAddress,
}

impl Walk<'_> {
    /// Walk one slot's children. `owner` and `slot` name the slot; `parent_block`
    /// is the block of the node that owns it (`None` for `root`); `accepts` is
    /// the block names the slot restricts to, empty for "any statement".
    fn slot(
        &mut self,
        children: &[Node],
        owner: Option<&str>,
        slot: &str,
        parent_block: Option<&str>,
        accepts: &[&str],
    ) {
        for (index, node) in children.iter().enumerate() {
            let at = StatementAddress {
                node: owner.map(str::to_string),
                slot: slot.to_string(),
                index: index as u32,
            };
            self.nodes.insert(
                node.id.clone(),
                NodeInfo {
                    position: self.position,
                    address: at.clone(),
                },
            );
            self.position += 1;

            let Some(desc) = describe(&node.block) else {
                self.push(
                    codes::BLOCK_UNKNOWN,
                    at,
                    params! { "block" => node.block.clone() },
                );
                continue;
            };

            self.check_placement(&desc, &node.block, parent_block, accepts, &at);
            self.check_inputs(&desc, node, &at);

            for slot_spec in desc.slots {
                let grandchildren = node
                    .slots
                    .get(slot_spec.name)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                self.slot(
                    grandchildren,
                    Some(&node.id),
                    slot_spec.name,
                    Some(block_name(&node.block)),
                    slot_spec.accepts,
                );
            }
        }
    }

    fn check_placement(
        &mut self,
        desc: &BlockDescriptor,
        block_ref: &str,
        parent_block: Option<&str>,
        accepts: &[&str],
        at: &StatementAddress,
    ) {
        let name = block_name(block_ref);

        if desc.node_kind == NodeKind::Value {
            self.push(
                codes::SLOT_EXPECTS_STATEMENT,
                at.clone(),
                params! { "block" => desc.title },
            );
            return;
        }

        if !accepts.is_empty() && !accepts.contains(&name) {
            let allowed: Vec<String> = accepts.iter().map(|n| display_name(n)).collect();
            self.push(
                codes::SLOT_REJECTS_BLOCK,
                at.clone(),
                params! {
                    "reason" => "not_accepted",
                    "block" => desc.title,
                    "accepts" => allowed,
                },
            );
            return;
        }

        if let Some(required_parent) = desc.requires_parent
            && parent_block != Some(required_parent)
        {
            self.push(
                codes::SLOT_REJECTS_BLOCK,
                at.clone(),
                params! {
                    "reason" => "needs_parent",
                    "block" => desc.title,
                    "parent" => display_name(required_parent),
                },
            );
        }
    }

    fn check_inputs(&mut self, desc: &BlockDescriptor, node: &Node, at: &StatementAddress) {
        for port in desc.inputs {
            let literal = node.inputs.get(port.name);
            let fed = self.fed.contains(&(node.id.clone(), port.name.to_string()));

            match literal {
                Some(value) => {
                    if let Some(finding) = check_literal(port, value) {
                        self.push(finding.code, at.clone(), finding.params);
                    }
                }
                None if port.required && !fed => {
                    let mut p = params! { "block" => desc.title, "label" => port.label };
                    if matches!(port.ty, PortType::Id { .. } | PortType::ListOfId { .. }) {
                        p.insert("example".to_string(), Param::from("example:tick"));
                    }
                    self.push(codes::INPUT_MISSING, at.clone(), p);
                }
                None => {}
            }
        }
    }

    fn check_edges(&mut self, edges: &[Edge]) {
        for edge in edges {
            let from = self.nodes.get(&edge.from.node);
            let to = self.nodes.get(&edge.to.node);
            let anchor = to
                .or(from)
                .map(|info| info.address.clone())
                .unwrap_or_else(root_address);

            match (from, to) {
                (None, _) => self.push(
                    codes::EDGE_UNKNOWN_NODE,
                    anchor,
                    params! { "node" => edge.from.node.clone(), "role" => "source" },
                ),
                (Some(_), None) => self.push(
                    codes::EDGE_UNKNOWN_NODE,
                    anchor,
                    params! { "node" => edge.to.node.clone(), "role" => "target" },
                ),
                (Some(source), Some(target)) => {
                    if source.position > target.position {
                        self.push(
                            codes::EDGE_FORWARD_REFERENCE,
                            target.address.clone(),
                            params! { "from" => edge.from.node.clone() },
                        );
                    }
                }
            }
        }

        self.detect_cycle(edges);
    }

    fn detect_cycle(&mut self, edges: &[Edge]) {
        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in edges {
            if self.nodes.contains_key(&edge.from.node) && self.nodes.contains_key(&edge.to.node) {
                adjacency
                    .entry(edge.from.node.as_str())
                    .or_default()
                    .push(edge.to.node.as_str());
            }
        }

        let mut state: BTreeMap<&str, Mark> = BTreeMap::new();
        let mut path: Vec<&str> = Vec::new();
        let starts: Vec<&str> = adjacency.keys().copied().collect();
        for start in starts {
            if self.visit(start, &adjacency, &mut state, &mut path) {
                return;
            }
        }
    }

    fn visit<'g>(
        &mut self,
        node: &'g str,
        adjacency: &BTreeMap<&'g str, Vec<&'g str>>,
        state: &mut BTreeMap<&'g str, Mark>,
        path: &mut Vec<&'g str>,
    ) -> bool {
        state.insert(node, Mark::InProgress);
        path.push(node);

        for &next in adjacency.get(node).map(Vec::as_slice).unwrap_or(&[]) {
            match state.get(next).copied().unwrap_or(Mark::Unseen) {
                Mark::InProgress => {
                    let start = path.iter().position(|n| *n == next).unwrap_or(0);
                    let cycle: Vec<String> = path[start..]
                        .iter()
                        .map(|n| n.to_string())
                        .chain(std::iter::once(next.to_string()))
                        .collect();
                    let anchor = self
                        .nodes
                        .get(next)
                        .map(|info| info.address.clone())
                        .unwrap_or_else(root_address);
                    self.push(codes::EDGE_CYCLE, anchor, params! { "cycle" => cycle });
                    path.pop();
                    return true;
                }
                Mark::Unseen => {
                    if self.visit(next, adjacency, state, path) {
                        path.pop();
                        return true;
                    }
                }
                Mark::Done => {}
            }
        }

        path.pop();
        state.insert(node, Mark::Done);
        false
    }

    fn push(&mut self, code: &str, address: StatementAddress, params: BTreeMap<String, Param>) {
        self.diagnostics.push(Diagnostic {
            code: Some(code.to_string()),
            severity: Severity::Error,
            address,
            params,
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Unseen,
    InProgress,
    Done,
}

/// A validation finding: the stable code and the facts its message template
/// reads (`packsmith_ir::message`). No rendered sentence is built here -- the
/// wording is `render`'s, so it can be reworded or translated in one place
/// (ADR-0009).
struct Finding {
    code: &'static str,
    params: BTreeMap<String, Param>,
}

fn finding(code: &'static str, params: BTreeMap<String, Param>) -> Option<Finding> {
    Some(Finding { code, params })
}

fn check_literal(port: &PortSpec, value: &Value) -> Option<Finding> {
    let label = port.label;
    match port.ty {
        PortType::Bool => require(value.is_boolean(), label, value, "yes_no"),
        PortType::Float => require(value.is_number(), label, value, "number"),
        PortType::Int { min, max } => check_int(label, value, min, max),
        PortType::Str => require(value.is_string(), label, value, "text"),
        PortType::Text => require(
            value.is_string() || value.is_object() || value.is_array(),
            label,
            value,
            "text",
        ),
        PortType::Id { allow_tag, .. } => match value.as_str() {
            Some(text) => check_id(label, None, text, allow_tag),
            None => mismatch(label, None, value, "name"),
        },
        PortType::ItemStack => check_item_stack(value, label, None),
        PortType::ListOfId { allow_tag, .. } => {
            let Some(items) = value.as_array() else {
                return mismatch(label, None, value, "id_list");
            };
            items.iter().find_map(|entry| match entry.as_str() {
                Some(text) => check_id(label, Some("entry"), text, allow_tag),
                None => mismatch(label, Some("entry"), entry, "name"),
            })
        }
        PortType::ListOfItemStack => {
            let Some(items) = value.as_array() else {
                return mismatch(label, None, value, "item_list");
            };
            items
                .iter()
                .find_map(|entry| check_item_stack(entry, label, Some("item")))
        }
        PortType::Enum(choices) => match value.as_str() {
            Some(text) if choices.contains(&text) => None,
            Some(text) => finding(
                codes::INPUT_CONSTRAINT,
                params! {
                    "reason" => "enum",
                    "label" => label,
                    "value" => text,
                    "choices" => choices.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                },
            ),
            None => mismatch(label, None, value, "choice"),
        },
    }
}

/// `input-type` unless `ok`: the literal is not a value of the port's type.
fn require(ok: bool, label: &str, value: &Value, expected: &str) -> Option<Finding> {
    if ok {
        None
    } else {
        mismatch(label, None, value, expected)
    }
}

/// An `input-type` finding: the value on `label` is not of the wanted type.
/// `scope` places it inside a list or an item when that is where it sits;
/// `expected` is a type tag the message table turns into a phrase.
fn mismatch(label: &str, scope: Option<&str>, value: &Value, expected: &str) -> Option<Finding> {
    let mut p = params! {
        "label" => label,
        "found" => value_tag(value),
        "expected" => expected,
    };
    with_scope(&mut p, scope);
    finding(codes::INPUT_TYPE, p)
}

fn check_int(label: &str, value: &Value, min: Option<i64>, max: Option<i64>) -> Option<Finding> {
    let Some(number) = value.as_i64() else {
        return mismatch(label, None, value, "whole_number");
    };
    if let Some(low) = min
        && number < low
    {
        return finding(
            codes::INPUT_CONSTRAINT,
            params! { "reason" => "int_min", "label" => label, "number" => number, "min" => low },
        );
    }
    if let Some(high) = max
        && number > high
    {
        return finding(
            codes::INPUT_CONSTRAINT,
            params! { "reason" => "int_max", "label" => label, "number" => number, "max" => high },
        );
    }
    None
}

fn check_item_stack(value: &Value, label: &str, scope: Option<&str>) -> Option<Finding> {
    let Some(object) = value.as_object() else {
        return mismatch(label, scope, value, "item");
    };
    let Some(item) = object.get("item") else {
        let mut p = params! { "reason" => "no_item", "label" => label };
        with_scope(&mut p, scope);
        return finding(codes::INPUT_TYPE, p);
    };
    let id_finding = match item.as_str() {
        Some(text) => check_id(label, Some("name"), text, false),
        None => mismatch(label, Some("name"), item, "name"),
    };
    if id_finding.is_some() {
        return id_finding;
    }
    match object.get("count").map(Value::as_i64) {
        None | Some(Some(1..)) => None,
        Some(Some(_)) => {
            let mut p = params! { "reason" => "count_below_one", "label" => label };
            with_scope(&mut p, scope);
            finding(codes::INPUT_CONSTRAINT, p)
        }
        Some(None) => {
            let mut p = params! { "reason" => "bad_count", "label" => label };
            with_scope(&mut p, scope);
            finding(codes::INPUT_TYPE, p)
        }
    }
}

/// Syntax-only id check, matching `spec/types.md` section 4.5: a namespace is
/// required, a tag prefix is allowed only where the port allows it, and the
/// characters are the target's identifier set. Registry membership is not
/// checked here -- it needs target data that may not exist for the registry.
fn check_id(label: &str, scope: Option<&str>, text: &str, allow_tag: bool) -> Option<Finding> {
    let (is_tag, core) = match text.strip_prefix('#') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    if is_tag && !allow_tag {
        let mut p = params! { "reason" => "id_tag_not_allowed", "label" => label };
        with_scope(&mut p, scope);
        return finding(codes::INPUT_CONSTRAINT, p);
    }
    match core.split_once(':') {
        None => finding(
            codes::INPUT_CONSTRAINT,
            params! { "reason" => "id_no_namespace", "value" => text, "suggestion" => core },
        ),
        Some((namespace, path))
            if !namespace.is_empty()
                && !path.is_empty()
                && namespace.chars().all(is_namespace_char)
                && path.chars().all(is_path_char) =>
        {
            None
        }
        Some(_) => finding(
            codes::INPUT_CONSTRAINT,
            params! { "reason" => "id_bad_chars", "value" => text },
        ),
    }
}

fn with_scope(p: &mut BTreeMap<String, Param>, scope: Option<&str>) {
    if let Some(s) = scope {
        p.insert("scope".to_string(), Param::from(s));
    }
}

/// A JSON value's kind as a tag the message table turns into a phrase. Keeps the
/// English on the far side of the crate boundary.
fn value_tag(value: &Value) -> &'static str {
    match value {
        Value::Null => "nothing",
        Value::Bool(_) => "yes_no",
        Value::Number(_) => "number",
        Value::String(_) => "text",
        Value::Array(_) => "list",
        Value::Object(_) => "group",
    }
}

fn is_namespace_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-')
}

fn is_path_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/')
}

fn block_name(block_ref: &str) -> &str {
    block_ref
        .split_once('@')
        .map_or(block_ref, |(name, _)| name)
}

fn root_address() -> StatementAddress {
    StatementAddress {
        node: None,
        slot: "root".to_string(),
        index: 0,
    }
}

#[derive(Debug, Deserialize)]
struct Edge {
    from: Endpoint,
    to: Endpoint,
}

#[derive(Debug, Deserialize)]
struct Endpoint {
    node: String,
    port: String,
}

/// Parse the graph's `edges` list. An edge that does not deserialise is left to
/// schema validation (`spec/graph.schema.json`), which is a separate pass
/// (`docs/BACKLOG.md`); this one only reports on structurally valid edges.
fn parse_edges(raw: &[Value]) -> Vec<Edge> {
    raw.iter()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(json: &str) -> Graph {
        serde_json::from_str(json).expect("valid graph fixture")
    }

    fn codes_of(diagnostics: &[Diagnostic]) -> Vec<&str> {
        diagnostics
            .iter()
            .filter_map(|d| d.code.as_deref())
            .collect()
    }

    #[test]
    fn a_clean_function_graph_has_no_diagnostics() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [
                  { "id": "fn", "block": "packsmith/function@1.0.0",
                    "inputs": { "name": "example:hello" },
                    "slots": { "body": [
                      { "id": "c", "block": "packsmith/command@1.0.0",
                        "inputs": { "command": "say hi" } } ] } } ] }"#,
        ));
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn an_unknown_block_is_reported_once_at_its_address() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "x", "block": "packsmith/mystery@1.0.0" } ] }"#,
        ));
        assert_eq!(codes_of(&d), [codes::BLOCK_UNKNOWN]);
        assert_eq!(d[0].address.index, 0);
        assert_eq!(d[0].address.node, None);
    }

    #[test]
    fn a_missing_required_input_is_reported() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "r", "block": "packsmith/crafting-shapeless@1.0.0",
                            "inputs": { "name": "example:x", "ingredients": ["minecraft:stone"] } } ] }"#,
        ));
        assert_eq!(codes_of(&d), [codes::INPUT_MISSING]);
    }

    #[test]
    fn a_wrong_typed_literal_is_a_type_diagnostic() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "r", "block": "packsmith/crafting-shapeless@1.0.0",
                            "inputs": { "name": "example:x", "ingredients": ["minecraft:stone"],
                                        "result": "minecraft:diamond" } } ] }"#,
        ));
        assert_eq!(codes_of(&d), [codes::INPUT_TYPE]);
    }

    #[test]
    fn an_id_without_a_namespace_violates_a_constraint() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "fn", "block": "packsmith/function@1.0.0",
                            "inputs": { "name": "tick" } } ] }"#,
        ));
        assert_eq!(codes_of(&d), [codes::INPUT_CONSTRAINT]);
    }

    #[test]
    fn an_int_below_its_bound_violates_a_constraint() {
        let port = PortSpec {
            name: "n",
            label: "count",
            ty: PortType::Int {
                min: Some(1),
                max: None,
            },
            required: true,
        };
        let finding = check_literal(&port, &serde_json::json!(0)).expect("out of bounds");
        assert_eq!(finding.code, codes::INPUT_CONSTRAINT);
    }

    #[test]
    fn an_enum_non_member_violates_a_constraint() {
        let port = PortSpec {
            name: "op",
            label: "comparison",
            ty: PortType::Enum(&["less_than", "greater_than"]),
            required: true,
        };
        let finding = check_literal(&port, &serde_json::json!("equal_to")).expect("not a choice");
        assert_eq!(finding.code, codes::INPUT_CONSTRAINT);
    }

    #[test]
    fn a_command_at_the_top_level_is_rejected() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "c", "block": "packsmith/command@1.0.0",
                            "inputs": { "command": "say hi" } } ] }"#,
        ));
        assert_eq!(codes_of(&d), [codes::SLOT_REJECTS_BLOCK]);
    }

    #[test]
    fn a_non_command_in_a_function_body_is_rejected() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "fn", "block": "packsmith/function@1.0.0",
                            "inputs": { "name": "example:x" },
                            "slots": { "body": [
                              { "id": "inner", "block": "packsmith/function@1.0.0",
                                "inputs": { "name": "example:y" } } ] } } ] }"#,
        ));
        assert_eq!(codes_of(&d), [codes::SLOT_REJECTS_BLOCK]);
    }

    #[test]
    fn an_edge_to_a_missing_node_is_reported() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [ { "id": "fn", "block": "packsmith/function@1.0.0",
                            "inputs": { "name": "example:x" } } ],
                "edges": [ { "from": { "node": "ghost", "port": "out" },
                             "to": { "node": "fn", "port": "name" } } ] }"#,
        ));
        assert_eq!(codes_of(&d), [codes::EDGE_UNKNOWN_NODE]);
        assert_eq!(d[0].address.node, None);
    }

    #[test]
    fn an_edge_that_reads_a_later_node_is_a_forward_reference() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [
                  { "id": "a", "block": "packsmith/function@1.0.0", "inputs": {} },
                  { "id": "b", "block": "packsmith/function@1.0.0", "inputs": { "name": "example:b" } } ],
                "edges": [ { "from": { "node": "b", "port": "out" },
                             "to": { "node": "a", "port": "name" } } ] }"#,
        ));
        assert!(codes_of(&d).contains(&codes::EDGE_FORWARD_REFERENCE));
    }

    #[test]
    fn a_cycle_among_edges_is_reported_once() {
        let d = validate(&graph(
            r#"{ "version": 0, "project": { "name": "P", "namespace": "example" },
                "root": [
                  { "id": "a", "block": "packsmith/function@1.0.0", "inputs": { "name": "example:a" } },
                  { "id": "b", "block": "packsmith/function@1.0.0", "inputs": { "name": "example:b" } } ],
                "edges": [ { "from": { "node": "a", "port": "o" }, "to": { "node": "b", "port": "name" } },
                           { "from": { "node": "b", "port": "o" }, "to": { "node": "a", "port": "name" } } ] }"#,
        ));
        assert_eq!(
            d.iter()
                .filter(|x| x.code.as_deref() == Some(codes::EDGE_CYCLE))
                .count(),
            1
        );
    }
}
