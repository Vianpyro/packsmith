//! The diagnostic message table: code plus parameters in, wording out.
//!
//! Every user-facing sentence Packsmith shows for a diagnostic is produced here
//! and nowhere else. A [`Diagnostic`](crate::Diagnostic) upstream carries only a
//! [`code`](crate::codes) and typed [`params`](crate::Param); [`render`] turns
//! that into the sentence and the suggested fix the two personas read (ADR-0009).
//!
//! This function is the translation unit. A localisation replaces the bodies of
//! the arms below; it never has to reassemble fragments handed down from the
//! compiler, and capitalisation is the template's, never a substituted word's.
//!
//! The English here is deliberately plain: the audience includes someone who has
//! never written code and does not know what a namespace is. Conditions that an
//! ordered-slot editor makes unrepresentable (`edge-forward-reference`,
//! `edge-cycle`) are the exception -- only a hand-edited graph or the CLI reaches
//! them, so they are worded for a technical reader (ADR-0016).

use std::collections::BTreeMap;

use crate::{Diagnostic, Param, codes};

type Params = BTreeMap<String, Param>;

/// A diagnostic's wording: the sentence, and a concrete fix when one is knowable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    pub message: String,
    pub fix: Option<String>,
}

/// Render `diagnostic` to the wording shown to the user. An unknown or absent
/// code renders a neutral sentence rather than panicking: a diagnostic is still
/// useful with its address alone.
pub fn render(diagnostic: &Diagnostic) -> Rendered {
    let p = &diagnostic.params;
    match diagnostic.code.as_deref() {
        Some(codes::BLOCK_UNKNOWN) => block_unknown(p),
        Some(codes::INPUT_MISSING) => input_missing(p),
        Some(codes::INPUT_TYPE) => input_type(p),
        Some(codes::INPUT_CONSTRAINT) => input_constraint(p),
        Some(codes::SLOT_EXPECTS_STATEMENT) => slot_expects_statement(p),
        Some(codes::SLOT_REJECTS_BLOCK) => slot_rejects_block(p),
        Some(codes::EDGE_UNKNOWN_NODE) => edge_unknown_node(p),
        Some(codes::EDGE_FORWARD_REFERENCE) => edge_forward_reference(p),
        Some(codes::EDGE_CYCLE) => edge_cycle(p),
        _ => Rendered {
            message: "Something about this step isn't right.".to_string(),
            fix: None,
        },
    }
}

fn block_unknown(p: &Params) -> Rendered {
    let block = text(p, "block");
    Rendered {
        message: format!("There's no block called \"{block}\"."),
        fix: Some("Check the name for a typo, or pick a block from the palette.".to_string()),
    }
}

fn input_missing(p: &Params) -> Rendered {
    let block = text(p, "block");
    let label = text(p, "label");
    let fix = match p.get("example") {
        Some(example) => format!(
            "Give the {block} a {label}, written namespace:name (for example \"{}\").",
            example.text()
        ),
        None => format!("Give the {block} a {label}."),
    };
    Rendered {
        message: format!("The {block} has no {label} set, and it needs one."),
        fix: Some(fix),
    }
}

fn input_type(p: &Params) -> Rendered {
    let subject = subject(p);
    match text(p, "reason") {
        "no_item" => Rendered {
            message: format!("{subject} has no item set. An item needs at least a name."),
            fix: Some("Give it an item name like \"minecraft:diamond\".".to_string()),
        },
        "bad_count" => Rendered {
            message: format!("{subject} has a count that isn't a whole number."),
            fix: Some("Use a whole number like 1, or leave the count out.".to_string()),
        },
        _ => {
            let found = found_phrase(text(p, "found"));
            let expected = expected_phrase(text(p, "expected"));
            Rendered {
                message: format!("{subject} is {found}, but this step needs {expected}."),
                fix: Some(format!("Enter {expected} here.")),
            }
        }
    }
}

fn input_constraint(p: &Params) -> Rendered {
    let label = text(p, "label");
    let value = text(p, "value");
    match text(p, "reason") {
        "int_min" => Rendered {
            message: format!(
                "The {label} is {}, but it can't be less than {}.",
                int(p, "number"),
                int(p, "min")
            ),
            fix: Some(format!("Use {} or more.", int(p, "min"))),
        },
        "int_max" => Rendered {
            message: format!(
                "The {label} is {}, but it can't be more than {}.",
                int(p, "number"),
                int(p, "max")
            ),
            fix: Some(format!("Use {} or less.", int(p, "max"))),
        },
        "enum" => {
            let choices = comma_list(list(p, "choices"));
            Rendered {
                message: format!(
                    "The {label} is \"{value}\", which isn't one of the choices: {choices}."
                ),
                fix: Some(format!("Use one of: {choices}.")),
            }
        }
        "id_no_namespace" => Rendered {
            message: format!(
                "\"{value}\" is missing a namespace. Minecraft names look like \"minecraft:stone\" \
                 -- a prefix, a colon, then the name."
            ),
            fix: Some(format!("Try \"minecraft:{}\".", text(p, "suggestion"))),
        },
        "id_bad_chars" => Rendered {
            message: format!("\"{value}\" isn't a valid name."),
            fix: Some(
                "Use lower-case letters, numbers, and _ . - only, like \"minecraft:stone\"."
                    .to_string(),
            ),
        },
        "id_tag_not_allowed" => Rendered {
            message: format!(
                "{} is a tag, but this takes a single thing, not a tag.",
                subject(p)
            ),
            fix: Some("Name one thing, without the leading #.".to_string()),
        },
        "count_below_one" => Rendered {
            message: format!(
                "{} has a count below 1. You can't have less than one.",
                subject(p)
            ),
            fix: Some("Use 1 or more, or leave the count out.".to_string()),
        },
        _ => Rendered {
            message: format!("The {label} breaks one of its rules."),
            fix: None,
        },
    }
}

fn slot_expects_statement(p: &Params) -> Rendered {
    let block = text(p, "block");
    Rendered {
        message: format!(
            "\"{block}\" produces a value, so it can't be a step here. Steps happen in order; a \
             value is wired into an input instead."
        ),
        fix: Some("Connect this block into an input rather than placing it as a step.".to_string()),
    }
}

fn slot_rejects_block(p: &Params) -> Rendered {
    let block = text(p, "block");
    match text(p, "reason") {
        "needs_parent" => {
            let parent = text(p, "parent");
            Rendered {
                message: format!(
                    "A {block} can't sit on its own at the top level -- it has to be inside a \
                     {parent}."
                ),
                fix: Some(format!(
                    "Put this {block} inside a {parent}'s steps, or wrap it in a {parent}."
                )),
            }
        }
        _ => {
            let accepts = or_list(list(p, "accepts"));
            Rendered {
                message: format!(
                    "A \"{block}\" step can't go here; this slot only takes {accepts}."
                ),
                fix: Some(format!(
                    "Move the \"{block}\" step out, or replace it with {accepts}."
                )),
            }
        }
    }
}

fn edge_unknown_node(p: &Params) -> Rendered {
    let node = text(p, "node");
    let message = match text(p, "role") {
        "target" => {
            format!("A connection feeds a step called \"{node}\" that isn't in this project.")
        }
        _ => {
            format!("A connection reads from a step called \"{node}\" that isn't in this project.")
        }
    };
    Rendered {
        message,
        fix: Some("Delete the dangling connection, or restore the missing step.".to_string()),
    }
}

fn edge_forward_reference(p: &Params) -> Rendered {
    let from = text(p, "from");
    Rendered {
        message: format!(
            "The connection from \"{from}\" is a forward reference: \"{from}\" is ordered after the \
             step that reads it. Ordered slots make this unrepresentable in the editor, so it came \
             from hand-editing or the CLI."
        ),
        fix: Some(format!(
            "Reorder so \"{from}\" comes before the step that reads it."
        )),
    }
}

fn edge_cycle(p: &Params) -> Rendered {
    let cycle = arrow_list(list(p, "cycle"));
    Rendered {
        message: format!(
            "The connections form a cycle: {cycle}. Ordered slots make this unrepresentable in the \
             editor, so it came from hand-editing or the CLI."
        ),
        fix: Some("Remove one of the connections in the cycle.".to_string()),
    }
}

/// The noun phrase for the value a diagnostic points at, from its port `label`
/// and an optional `scope` that places it inside a collection.
fn subject(p: &Params) -> String {
    let label = text(p, "label");
    match text(p, "scope") {
        "entry" => format!("An entry in {label}"),
        "item" => format!("An item in {label}"),
        "name" => format!("The item name in {label}"),
        _ => format!("The {label}"),
    }
}

/// A JSON value's kind, as the reader sees it. Keyed off a tag the compiler
/// classified rather than the value itself, so no English crosses the crate
/// boundary.
fn found_phrase(tag: &str) -> &'static str {
    match tag {
        "nothing" => "nothing",
        "yes_no" => "a yes/no value",
        "number" => "a number",
        "list" => "a list",
        "group" => "a group of settings",
        _ => "text",
    }
}

/// What a port will accept, from the port's type tag.
fn expected_phrase(tag: &str) -> &'static str {
    match tag {
        "yes_no" => "a yes/no value",
        "number" => "a number",
        "whole_number" => "a whole number",
        "item" => "an item, like minecraft:diamond",
        "name" => "a name like minecraft:stone",
        "id_list" => "a list of names",
        "item_list" => "a list of items",
        "choice" => "one of the choices",
        _ => "text",
    }
}

fn text<'a>(p: &'a Params, key: &str) -> &'a str {
    p.get(key).map(Param::text).unwrap_or_default()
}

fn int(p: &Params, key: &str) -> i64 {
    p.get(key).map(Param::int).unwrap_or_default()
}

fn list<'a>(p: &'a Params, key: &str) -> &'a [String] {
    p.get(key).map(Param::list).unwrap_or_default()
}

fn comma_list(items: &[String]) -> String {
    items.join(", ")
}

fn arrow_list(items: &[String]) -> String {
    items.join(" -> ")
}

/// `"a"`, `"a" or "b"`, `"a", "b" or "c"` -- each name quoted.
fn or_list(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
    match quoted.split_last() {
        None => "nothing".to_string(),
        Some((last, [])) => last.clone(),
        Some((last, head)) => format!("{} or {last}", head.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Severity, StatementAddress, params};

    fn diag(code: &str, params: Params) -> Diagnostic {
        Diagnostic {
            code: Some(code.to_string()),
            severity: Severity::Error,
            address: StatementAddress {
                node: None,
                slot: "root".to_string(),
                index: 0,
            },
            params,
        }
    }

    #[test]
    fn block_unknown_names_the_block_and_is_not_jargon() {
        let r = render(&diag(
            codes::BLOCK_UNKNOWN,
            params! { "block" => "packsmith/nope@1.0.0" },
        ));
        assert_eq!(
            r.message,
            "There's no block called \"packsmith/nope@1.0.0\"."
        );
    }

    #[test]
    fn input_missing_names_the_block_so_the_cli_line_is_locatable() {
        let r = render(&diag(
            codes::INPUT_MISSING,
            params! { "block" => "function", "label" => "name", "example" => "example:tick" },
        ));
        assert_eq!(r.message, "The function has no name set, and it needs one.");
        assert!(r.fix.unwrap().contains("example:tick"));
    }

    #[test]
    fn slot_rejects_block_uses_display_names_not_ids() {
        let r = render(&diag(
            codes::SLOT_REJECTS_BLOCK,
            params! { "reason" => "needs_parent", "block" => "command", "parent" => "function" },
        ));
        assert!(r.message.contains("command"));
        assert!(r.message.contains("inside a function"));
        assert!(!r.message.contains("packsmith/"));
    }

    #[test]
    fn a_group_of_settings_replaces_a_set_of_fields() {
        let r = render(&diag(
            codes::INPUT_TYPE,
            params! { "label" => "result item", "found" => "group", "expected" => "item" },
        ));
        assert!(r.message.contains("a group of settings"));
        assert!(!r.message.contains("a set of fields"));
    }

    #[test]
    fn forward_reference_and_cycle_are_worded_for_a_technical_reader() {
        let fwd = render(&diag(
            codes::EDGE_FORWARD_REFERENCE,
            params! { "from" => "fn-late" },
        ));
        assert!(fwd.message.contains("forward reference"));
        assert!(fwd.message.contains("unrepresentable in the editor"));

        let cyc = render(&diag(
            codes::EDGE_CYCLE,
            params! { "cycle" => vec!["a".to_string(), "b".to_string(), "a".to_string()] },
        ));
        assert!(cyc.message.contains("a -> b -> a"));
    }

    #[test]
    fn an_unknown_code_still_renders_something() {
        let r = render(&diag("not-a-real-code", params! {}));
        assert!(!r.message.is_empty());
        assert!(r.fix.is_none());
    }
}
