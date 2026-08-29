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
//! statement address it points at, and a suggested fix where one is knowable.
//! Codes are stable: `spec/diagnostics.md` is the normative list and conformance
//! cases assert on them.

use std::collections::{BTreeMap, HashSet};

use serde::Deserialize;
use serde_json::Value;

use packsmith_blocks::{Node, NodeKind, PortSpec, PortType, describe};
use packsmith_ir::{Diagnostic, Severity, StatementAddress, codes};

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
                    format!("Nothing here knows a block called \"{}\".", node.block),
                    Some(
                        "Check the name for a typo, or pick a block from the palette.".to_string(),
                    ),
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
        desc: &packsmith_blocks::BlockDescriptor,
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
                format!(
                    "\"{name}\" produces a value, so it can't be a step here. Steps happen in \
                     order; a value is wired into an input instead."
                ),
                Some(
                    "Connect this block into an input rather than placing it as a step."
                        .to_string(),
                ),
            );
            return;
        }

        if !accepts.is_empty() && !accepts.contains(&name) {
            self.push(
                codes::SLOT_REJECTS_BLOCK,
                at.clone(),
                format!(
                    "A \"{name}\" step can't go here; this slot only takes {}.",
                    join_or(accepts)
                ),
                Some(format!(
                    "Move the \"{name}\" step out, or replace it with {}.",
                    join_or(accepts)
                )),
            );
            return;
        }

        if let Some(required_parent) = desc.requires_parent
            && parent_block != Some(required_parent)
        {
            self.push(
                codes::SLOT_REJECTS_BLOCK,
                at.clone(),
                "A command can't sit on its own at the top level -- it has to be inside a function."
                    .to_string(),
                Some(
                    "Put this command inside a function's steps, or wrap it in a function."
                        .to_string(),
                ),
            );
        }
    }

    fn check_inputs(
        &mut self,
        desc: &packsmith_blocks::BlockDescriptor,
        node: &Node,
        at: &StatementAddress,
    ) {
        for port in desc.inputs {
            let literal = node.inputs.get(port.name);
            let fed = self.fed.contains(&(node.id.clone(), port.name.to_string()));

            match literal {
                Some(value) => {
                    if let Some(finding) = check_literal(port, value) {
                        self.push(finding.code, at.clone(), finding.message, finding.fix);
                    }
                }
                None if port.required && !fed => {
                    self.push(
                        codes::INPUT_MISSING,
                        at.clone(),
                        format!("This step has no {} set, and it needs one.", port.label),
                        Some(missing_fix(port)),
                    );
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
                    format!(
                        "A connection reads from a step called \"{}\" that isn't in this project.",
                        edge.from.node
                    ),
                    Some(
                        "Delete the dangling connection, or restore the missing step.".to_string(),
                    ),
                ),
                (Some(_), None) => self.push(
                    codes::EDGE_UNKNOWN_NODE,
                    anchor,
                    format!(
                        "A connection feeds a step called \"{}\" that isn't in this project.",
                        edge.to.node
                    ),
                    Some(
                        "Delete the dangling connection, or restore the missing step.".to_string(),
                    ),
                ),
                (Some(source), Some(target)) => {
                    if source.position > target.position {
                        self.push(
                            codes::EDGE_FORWARD_REFERENCE,
                            target.address.clone(),
                            format!(
                                "This connection reads a value from \"{0}\", but \"{0}\" comes \
                                 after the step that uses it. A value has to be produced before \
                                 it is read.",
                                edge.from.node
                            ),
                            Some(format!(
                                "Move \"{}\" earlier, or read a value that already exists at this \
                                 point.",
                                edge.from.node
                            )),
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
                    let cycle = path[start..].join(" -> ");
                    let anchor = self
                        .nodes
                        .get(next)
                        .map(|info| info.address.clone())
                        .unwrap_or_else(root_address);
                    self.push(
                        codes::EDGE_CYCLE,
                        anchor,
                        format!(
                            "These steps feed each other in a loop: {cycle} -> {next}. None of \
                             them can be worked out first."
                        ),
                        Some("Break the loop by removing one of the connections.".to_string()),
                    );
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

    fn push(
        &mut self,
        code: &str,
        address: StatementAddress,
        message: String,
        fix: Option<String>,
    ) {
        self.diagnostics.push(Diagnostic {
            code: Some(code.to_string()),
            severity: Severity::Error,
            address,
            message,
            fix,
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Unseen,
    InProgress,
    Done,
}

struct Finding {
    code: &'static str,
    message: String,
    fix: Option<String>,
}

fn check_literal(port: &PortSpec, value: &Value) -> Option<Finding> {
    let label = cap(port.label);
    match port.ty {
        PortType::Bool => require(value.is_boolean(), &label, value, "a yes/no value"),
        PortType::Float => require(value.is_number(), &label, value, "a number"),
        PortType::Int { min, max } => check_int(&label, value, min, max),
        PortType::Str => require(value.is_string(), &label, value, "text"),
        PortType::Text => require(
            value.is_string() || value.is_object() || value.is_array(),
            &label,
            value,
            "text",
        ),
        PortType::Id { allow_tag } => match value.as_str() {
            Some(text) => check_id(&label, text, allow_tag),
            None => Some(mismatch(&label, value, "a name like minecraft:stone")),
        },
        PortType::ItemStack => check_item_stack(value, &label),
        PortType::ListOfId { allow_tag } => {
            let Some(items) = value.as_array() else {
                return Some(mismatch(&label, value, "a list of names"));
            };
            for entry in items {
                let subject = format!("An entry in {}", port.label);
                let finding = match entry.as_str() {
                    Some(text) => check_id(&subject, text, allow_tag),
                    None => Some(mismatch(
                        &subject,
                        entry,
                        "a name like minecraft:oak_planks",
                    )),
                };
                if finding.is_some() {
                    return finding;
                }
            }
            None
        }
        PortType::ListOfItemStack => {
            let Some(items) = value.as_array() else {
                return Some(mismatch(&label, value, "a list of items"));
            };
            for entry in items {
                let finding = check_item_stack(entry, &format!("An item in {}", port.label));
                if finding.is_some() {
                    return finding;
                }
            }
            None
        }
        PortType::Enum(choices) => match value.as_str() {
            Some(text) if choices.contains(&text) => None,
            Some(text) => Some(Finding {
                code: codes::INPUT_CONSTRAINT,
                message: format!(
                    "{label} is \"{text}\", which isn't one of the choices: {}.",
                    choices.join(", ")
                ),
                fix: Some(format!("Use one of: {}.", choices.join(", "))),
            }),
            None => Some(mismatch(&label, value, "a choice")),
        },
    }
}

/// `input-type` unless `ok`: the literal is not a value of the port's type.
fn require(ok: bool, subject: &str, value: &Value, expected: &str) -> Option<Finding> {
    if ok {
        None
    } else {
        Some(mismatch(subject, value, expected))
    }
}

fn mismatch(subject: &str, value: &Value, expected: &str) -> Finding {
    Finding {
        code: codes::INPUT_TYPE,
        message: format!(
            "{subject} is {}, but this step needs {expected}.",
            describe_value(value)
        ),
        fix: Some(format!("Enter {expected} here.")),
    }
}

fn check_int(label: &str, value: &Value, min: Option<i64>, max: Option<i64>) -> Option<Finding> {
    let Some(number) = value.as_i64() else {
        return Some(mismatch(label, value, "a whole number"));
    };
    if let Some(low) = min
        && number < low
    {
        return Some(Finding {
            code: codes::INPUT_CONSTRAINT,
            message: format!("{label} is {number}, but it can't be less than {low}."),
            fix: Some(format!("Use {low} or more.")),
        });
    }
    if let Some(high) = max
        && number > high
    {
        return Some(Finding {
            code: codes::INPUT_CONSTRAINT,
            message: format!("{label} is {number}, but it can't be more than {high}."),
            fix: Some(format!("Use {high} or less.")),
        });
    }
    None
}

fn check_item_stack(value: &Value, subject: &str) -> Option<Finding> {
    let Some(object) = value.as_object() else {
        return Some(mismatch(subject, value, "an item, like minecraft:diamond"));
    };
    let Some(item) = object.get("item") else {
        return Some(Finding {
            code: codes::INPUT_TYPE,
            message: format!("{subject} has no item set. An item needs at least a name."),
            fix: Some("Give it an item name like \"minecraft:diamond\".".to_string()),
        });
    };
    let item_subject = format!("{subject}'s name");
    let id_finding = match item.as_str() {
        Some(text) => check_id(&item_subject, text, false),
        None => Some(mismatch(
            &item_subject,
            item,
            "a name like minecraft:diamond",
        )),
    };
    if id_finding.is_some() {
        return id_finding;
    }
    if let Some(count) = object.get("count") {
        match count.as_i64() {
            Some(number) if number >= 1 => {}
            Some(_) => {
                return Some(Finding {
                    code: codes::INPUT_CONSTRAINT,
                    message: format!(
                        "{subject} has a count below 1. You can't have less than one."
                    ),
                    fix: Some("Use 1 or more, or leave the count out.".to_string()),
                });
            }
            None => {
                return Some(Finding {
                    code: codes::INPUT_TYPE,
                    message: format!("{subject} has a count that isn't a whole number."),
                    fix: Some("Use a whole number like 1, or leave the count out.".to_string()),
                });
            }
        }
    }
    None
}

/// Syntax-only id check, matching `spec/types.md` section 4.5: a namespace is
/// required, a tag prefix is allowed only where the port allows it, and the
/// characters are the target's identifier set. Registry membership is not
/// checked here -- it needs target data that may not exist for the registry.
fn check_id(subject: &str, text: &str, allow_tag: bool) -> Option<Finding> {
    let (is_tag, core) = match text.strip_prefix('#') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    if is_tag && !allow_tag {
        return Some(Finding {
            code: codes::INPUT_CONSTRAINT,
            message: format!("{subject} is a tag, but this takes a single thing, not a tag."),
            fix: Some("Name one thing, without the leading #.".to_string()),
        });
    }
    match core.split_once(':') {
        None => Some(Finding {
            code: codes::INPUT_CONSTRAINT,
            message: format!(
                "\"{text}\" is missing a namespace. Minecraft names look like \"minecraft:stone\" \
                 -- a prefix, a colon, then the name."
            ),
            fix: Some(format!("Try \"minecraft:{core}\".")),
        }),
        Some((namespace, path))
            if !namespace.is_empty()
                && !path.is_empty()
                && namespace.chars().all(is_namespace_char)
                && path.chars().all(is_path_char) =>
        {
            None
        }
        Some(_) => Some(Finding {
            code: codes::INPUT_CONSTRAINT,
            message: format!("\"{text}\" isn't a valid name."),
            fix: Some(
                "Use lower-case letters, numbers, and _ . - only, like \"minecraft:stone\"."
                    .to_string(),
            ),
        }),
    }
}

fn missing_fix(port: &PortSpec) -> String {
    match port.ty {
        PortType::Id { .. } | PortType::ListOfId { .. } => format!(
            "Give this step a {}, written namespace:name (for example \"example:tick\").",
            port.label
        ),
        _ => format!("Give this step a {}.", port.label),
    }
}

fn describe_value(value: &Value) -> &'static str {
    match value {
        Value::Null => "nothing",
        Value::Bool(_) => "a yes/no value",
        Value::Number(_) => "a number",
        Value::String(_) => "text",
        Value::Array(_) => "a list",
        Value::Object(_) => "a set of fields",
    }
}

fn is_namespace_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-')
}

fn is_path_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/')
}

fn cap(label: &str) -> String {
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn join_or(names: &[&str]) -> String {
    match names {
        [] => "nothing".to_string(),
        [only] => format!("\"{only}\""),
        [head @ .., last] => format!(
            "{} or \"{last}\"",
            head.iter()
                .map(|n| format!("\"{n}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
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
