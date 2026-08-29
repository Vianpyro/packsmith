//! Stable identifiers for compiler diagnostics.
//!
//! A code names a *condition*, never a place and never a fix. With the
//! [`Diagnostic`](crate::Diagnostic)'s `params` it is what a machine reads:
//! conformance cases assert on it (`.claude/rules/spec.md`), the message table
//! ([`crate::message`]) keys off it, and it never changes once shipped.
//! `spec/diagnostics.md` is the normative list; this module is its Rust mirror
//! and the two are kept in step.
//!
//! Codes are lower-case, `-` separated, and grouped by a prefix that names the
//! part of the graph at fault: `block-`, `input-`, `slot-`, `edge-`. The
//! `command-` family is emitted by the command-grammar stage (ADR-0012), which
//! runs after lowering over the IR command lines rather than in the graph
//! validation pass.

/// The node names a block that is not installed.
pub const BLOCK_UNKNOWN: &str = "block-unknown";

/// A required input port holds neither a literal nor an incoming edge.
pub const INPUT_MISSING: &str = "input-missing";

/// The literal on an input port is not a value of the port's declared type
/// (`spec/types.md` section 4) -- text where an item is wanted, a list where a
/// number is wanted.
pub const INPUT_TYPE: &str = "input-type";

/// The literal is the right type but breaks a rule the type carries
/// (`spec/types.md` section 4): an `id` with no namespace, an `int` outside its
/// bounds, an `enum` value that is not one of the choices, a `list` that is too
/// short, too long, or not unique.
pub const INPUT_CONSTRAINT: &str = "input-constraint";

/// A value node sits in a slot. Slots hold statement nodes, which happen in
/// order; a value node produces a value and is wired into an input instead
/// (`spec/types.md` section 4.11).
pub const SLOT_EXPECTS_STATEMENT: &str = "slot-expects-statement";

/// A statement node sits where it cannot do anything: a `command` at the top
/// level instead of inside a function, or a block a slot does not accept.
pub const SLOT_REJECTS_BLOCK: &str = "slot-rejects-block";

/// A data edge names a node that is not in the graph.
pub const EDGE_UNKNOWN_NODE: &str = "edge-unknown-node";

/// A data edge takes a value from a node that comes later in the document than
/// the node that uses it. A value must be produced before it is read
/// (`spec/types.md` section 2.4).
pub const EDGE_FORWARD_REFERENCE: &str = "edge-forward-reference";

/// The data edges form a cycle: every node in it waits for another
/// (`spec/types.md` section 2.4).
pub const EDGE_CYCLE: &str = "edge-cycle";

/// A command line does not parse against the target's Brigadier tree: an unknown
/// command, a misspelled subcommand, or a line that stops before it is complete
/// (ADR-0012). Emitted after lowering, over the IR command lines.
pub const COMMAND_INVALID: &str = "command-invalid";

/// A command line is a recognisable older form the target no longer accepts --
/// today, `execute` followed straight by a selector or a `~`/`^` position, the
/// pre-1.13 form. The returning-creator failure mode of ADR-0009.
pub const COMMAND_LEGACY_SYNTAX: &str = "command-legacy-syntax";
