//! Command-grammar validation (ADR-0012): the last compiler stage.
//!
//! After lowering, every IR command line is walked against the target's pruned
//! Brigadier tree ([`TargetData::commands`]). This is not a parser for the
//! command language -- ADR-0012 is explicit that a full command AST is a
//! multi-year target we are not building. It checks the *shape*: is the first
//! word a command, are the subcommands spelled and nested right, does the line
//! finish. It reports only when a token can match no child of the current tree
//! node, so a valid command is never flagged; a malformed argument -- a broken
//! selector, bad NBT -- inside an otherwise well-formed command passes here and
//! fails in game.
//!
//! Blank lines and `#`-comment lines are skipped, exactly as the game skips them
//! in a function file. That covers both the `packsmith/command` block (one line)
//! and `packsmith/mcfunction` (a whole file, `spec/types.md` section 4.4).
//!
//! Diagnostics anchor at the statement address the line was lowered from and are
//! worded game-concept-first by `packsmith_ir::message` (ADR-0009).

use serde_json::Value;

use packsmith_ir::{Body, Command, Diagnostic, Param, Resource, Severity, StatementAddress, codes};
use packsmith_mcversion::TargetData;

/// Validate every command line in `resources` against `target`'s command tree.
/// Collects, never bails: the caller shows them all and refuses to emit.
pub fn validate(resources: &[Resource], target: &TargetData) -> Vec<Diagnostic> {
    let tree = Tree {
        root: target.commands(),
    };
    let mut out = Vec::new();
    for resource in resources {
        let Body::Commands { statements } = &resource.body else {
            continue;
        };
        for Command::Text { command, origin } in statements {
            if let Some(reject) = tree.check(command) {
                out.push(reject.into_diagnostic(command, origin));
            }
        }
    }
    out
}

/// Why a command line did not parse. `token` is the word the walk could match
/// nowhere (`None` when the line simply stopped early); `legacy_selector` is set
/// when the line is a recognisable pre-1.13 `execute` form.
struct Reject {
    token: Option<String>,
    legacy_selector: Option<String>,
}

impl Reject {
    fn into_diagnostic(self, line: &str, origin: &StatementAddress) -> Diagnostic {
        let mut params = std::collections::BTreeMap::new();
        params.insert("command".to_string(), Param::from(line.to_string()));

        let code = match self.legacy_selector {
            Some(selector) => {
                params.insert("selector".to_string(), Param::from(selector));
                codes::COMMAND_LEGACY_SYNTAX
            }
            None => {
                if let Some(token) = self.token {
                    params.insert("token".to_string(), Param::from(token));
                }
                codes::COMMAND_INVALID
            }
        };

        Diagnostic {
            code: Some(code.to_string()),
            severity: Severity::Error,
            address: origin.clone(),
            params,
        }
    }
}

struct Tree<'a> {
    root: &'a Value,
}

impl Tree<'_> {
    /// `None` if `line` parses (or is a blank / comment line), `Some(reject)`
    /// otherwise.
    fn check(&self, line: &str) -> Option<Reject> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        self.walk(self.root, &parts, 0).err()
    }

    fn walk(&self, node: &Value, parts: &[&str], i: usize) -> Result<(), Reject> {
        let node = self.effective(node);

        if i >= parts.len() {
            return if executable(node) {
                Ok(())
            } else {
                Err(Reject {
                    token: None,
                    legacy_selector: None,
                })
            };
        }

        let token = parts[i];
        let Some(children) = children(node) else {
            return Err(self.reject(node, parts, i));
        };

        // Literals win over arguments, and a matched literal is committed to:
        // Brigadier does the same, and it keeps the walk linear.
        if let Some(child) = children.get(token)
            && node_type(child) == "literal"
        {
            return self.walk(child, parts, i + 1);
        }

        let mut first: Option<Reject> = None;
        for child in children.values() {
            if node_type(child) != "argument" {
                continue;
            }
            // Parsers whose syntax can hold spaces or braces cannot be tokenised
            // on whitespace. Consume the rest of the line and accept: we lose
            // whatever follows, which only ever costs a missed error, never a
            // false one (ADR-0012 is best-effort).
            if greedy_accept(child) {
                return Ok(());
            }
            let span = arg_span(child, parts.len() - i);
            match self.walk(child, parts, i + span) {
                Ok(()) => return Ok(()),
                Err(reject) => {
                    first.get_or_insert(reject);
                }
            }
        }

        Err(first.unwrap_or_else(|| self.reject(node, parts, i)))
    }

    fn reject(&self, node: &Value, parts: &[&str], i: usize) -> Reject {
        let token = parts[i];
        let legacy_selector =
            (looks_like_execute(node) && is_positional(token)).then(|| token.to_string());
        Reject {
            token: Some(token.to_string()),
            legacy_selector,
        }
    }

    /// The node whose `children` and `executable` actually govern the next step:
    /// a `redirect` target, or the root for `execute`/`return`'s `run` (a
    /// childless non-executable literal, which mcmeta emits without the
    /// redirect-to-root the game reports).
    fn effective<'a>(&'a self, node: &'a Value) -> &'a Value {
        if let Some(path) = node.get("redirect").and_then(Value::as_array) {
            let segments: Vec<&str> = path.iter().filter_map(Value::as_str).collect();
            if let Some(target) = self.resolve(&segments) {
                return target;
            }
        }
        if node_type(node) == "literal"
            && !executable(node)
            && children(node).is_none_or(serde_json::Map::is_empty)
        {
            return self.root;
        }
        node
    }

    fn resolve(&self, path: &[&str]) -> Option<&Value> {
        let mut node = self.root;
        for segment in path {
            node = node.get("children")?.get(segment)?;
        }
        Some(node)
    }
}

fn children(node: &Value) -> Option<&serde_json::Map<String, Value>> {
    node.get("children").and_then(Value::as_object)
}

fn node_type(node: &Value) -> &str {
    node.get("type").and_then(Value::as_str).unwrap_or("")
}

fn executable(node: &Value) -> bool {
    node.get("executable")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn looks_like_execute(node: &Value) -> bool {
    children(node).is_some_and(|c| c.contains_key("as") && c.contains_key("run"))
}

fn is_positional(token: &str) -> bool {
    token.starts_with(['@', '~', '^'])
}

/// How many whitespace tokens an argument consumes. One, unless it is a
/// coordinate tuple; clamped to what is left and to at least one.
fn arg_span(child: &Value, remaining: usize) -> usize {
    let parser = child.get("parser").and_then(Value::as_str).unwrap_or("");
    let want = match parser {
        "minecraft:vec3" | "minecraft:block_pos" => 3,
        "minecraft:vec2" | "minecraft:rotation" | "minecraft:column_pos" => 2,
        _ => 1,
    };
    want.min(remaining).max(1)
}

/// Parsers whose value can contain spaces or nested brackets (NBT, block/item
/// states, chat components, a greedy string). Not `minecraft:entity`: a selector
/// with a quoted space is rare, and treating it as greedy would break the common
/// `execute as @e run ...`.
fn greedy_accept(child: &Value) -> bool {
    let parser = child.get("parser").and_then(Value::as_str).unwrap_or("");
    if matches!(
        parser,
        "minecraft:message"
            | "minecraft:component"
            | "minecraft:style"
            | "minecraft:nbt_compound_tag"
            | "minecraft:nbt_tag"
            | "minecraft:nbt_path"
            | "minecraft:particle"
            | "minecraft:item_stack"
            | "minecraft:item_predicate"
            | "minecraft:block_state"
            | "minecraft:block_predicate"
            | "minecraft:dialog"
    ) {
        return true;
    }
    parser == "brigadier:string"
        && child
            .get("properties")
            .and_then(|p| p.get("type"))
            .and_then(Value::as_str)
            == Some("greedy")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> TargetData {
        TargetData::load(&packsmith_mcversion::bundled_data_dir(), "26.2")
            .expect("26.2 data ships with packsmith-mcversion")
    }

    fn check(line: &str) -> Option<Reject> {
        let target = tree();
        Tree {
            root: target.commands(),
        }
        .check(line)
    }

    #[test]
    fn a_plain_say_is_valid() {
        assert!(check("say Hello, world!").is_none());
        assert!(check("say boo").is_none());
    }

    #[test]
    fn blank_and_comment_lines_are_skipped() {
        assert!(check("").is_none());
        assert!(check("   ").is_none());
        assert!(check("# a hand-written note").is_none());
        assert!(check("   # indented note").is_none());
    }

    #[test]
    fn an_unknown_command_is_rejected_with_the_token() {
        let r = check("frobnicate the widget").expect("not a command");
        assert_eq!(r.token.as_deref(), Some("frobnicate"));
        assert!(r.legacy_selector.is_none());
    }

    #[test]
    fn pre_1_13_execute_is_flagged_as_legacy_with_its_selector() {
        let r = check("execute @e[type=zombie] ~ ~ ~ say boo").expect("legacy form");
        assert_eq!(r.legacy_selector.as_deref(), Some("@e[type=zombie]"));
    }

    #[test]
    fn modern_execute_with_a_run_body_is_valid() {
        assert!(check("execute as @e[type=zombie] at @s run say boo").is_none());
        assert!(check("execute if entity @s run say hi").is_none());
    }

    #[test]
    fn an_incomplete_command_is_rejected_without_a_token() {
        let r = check("execute as @s").expect("stops before run");
        assert!(r.token.is_none());
        assert!(r.legacy_selector.is_none());
    }

    #[test]
    fn a_misspelled_subcommand_is_rejected() {
        let r = check("execute af @s run say hi").expect("no such subcommand");
        assert_eq!(r.token.as_deref(), Some("af"));
    }

    #[test]
    fn a_coordinate_tuple_is_consumed_as_one_argument() {
        // setblock <pos:block_pos> <block:block_state>
        assert!(check("setblock ~ ~1 ~ minecraft:stone").is_none());
    }
}
