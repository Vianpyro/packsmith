# Diagnostic codes

- **Spec version:** 0 (unstable; Phase 0)
- **Status:** normative
- **Constrained by:** ADR-0009 (who reads these), ADR-0012 (command grammar is a
  separate stage, now landed), ADR-0016 (statement addresses).
- **Licence:** `MIT OR Apache-2.0`, as all of `spec/`.

A **diagnostic** is a structured, user-facing result the compiler produces when a
graph is wrong or questionable. It carries:

| Field | Meaning | Asserted by conformance? |
|---|---|---|
| `code` | a stable identifier for the *condition* | **yes** |
| `severity` | `error` or `warning` | **yes** |
| `address` | the statement address it points at, `(node, slot, index)`; `node` is `null` for the top-level `root` slot | **yes** |
| `params` | the facts of this occurrence, keyed by name (see below) | optional |

A diagnostic carries **no rendered sentence**. The wording the two personas of
ADR-0009 read — one sentence, game terms first, and a concrete `fix` when one is
knowable — is produced from `code` and `params` by a single per-code template
table (`packsmith-ir::message`). That table is the unit a translation replaces;
the compiler never assembles a sentence from fragments, and capitalisation is the
template's, never a substituted word's. The rendered wording is never asserted,
so it can be reworded or localised without touching a case.

### Parameters

`params` is a flat map from a name to a string, a whole number, or a list of
strings. The names are per condition and are not a stable contract in spec
version 0, but a case *may* assert an individual `param` when the fact it records
is the point of the case (a bound, a set of choices, the block a diagnostic
names). `message` is still recorded in `expected-diagnostics.json` for a reader,
and is still never asserted. The full `expected-diagnostics.json` rules are in
`conformance/README.md`.

## The code namespace

A code is one flat lower-case identifier, words joined by `-`. It names the
condition and nothing else: never a location, never a fix, never a severity.
Once a code has shipped it does not change; a reworded diagnostic keeps its code.

Codes are grouped by a prefix that names the part of the graph at fault:

| Prefix | The part at fault |
|---|---|
| `block-` | the block a node refers to |
| `input-` | the value on an input port |
| `slot-` | a node's place in a slot |
| `edge-` | a data edge |
| `command-` | a command string's grammar — **owned by the command-grammar stage (ADR-0012)**, emitted after lowering, not by the validation pass |

`code: null` in a conformance case marks a condition the compiler recognises but
has not assigned a code to yet. It stays satisfied until a real code replaces it,
in its own commit (`.claude/rules/spec.md`).

## Codes emitted by the validation pass

The pass runs once over the graph before lowering and collects every diagnostic
it finds. It does not consult target data (registry membership, block property
values) and it does not check command grammar; both need machinery it never
touches.

| Code | Severity | Condition | Fix knowable? |
|---|---|---|---|
| `block-unknown` | error | The node's `block` address matches no installed block. | sometimes — a near-miss name |
| `input-missing` | error | A required input port holds neither a literal nor an incoming edge. | yes — name the port |
| `input-type` | error | The literal on a port is not a value of the port's declared type (`spec/types.md` section 4): text where an item is wanted, a list where a number is wanted. | yes — name the wanted type |
| `input-constraint` | error | The literal is the right type but breaks a rule the type carries: an `id` with no namespace or bad characters, a tag where a tag is not allowed, an `int` outside its bounds, an `enum` value that is not a listed choice, an `item_stack` count below 1. | yes |
| `slot-expects-statement` | error | A value node sits in a slot. Slots hold statement nodes, which run in order; a value node is wired into an input instead (`spec/types.md` section 4.11). | yes |
| `slot-rejects-block` | error | A statement node sits where it cannot act: a `command` at the top level instead of inside a function, or a block a slot's own list does not accept. | yes |
| `edge-unknown-node` | error | A data edge names a node that is not in the graph. | yes — delete the edge or restore the node |
| `edge-forward-reference` | error | A data edge reads a value from a node that comes later in the document than the node that uses it (`spec/types.md` section 2.4). | yes — reorder |
| `edge-cycle` | error | The data edges form a cycle: every node in it waits for another (`spec/types.md` section 2.4). | yes — remove one edge |

Anchoring: an `input-*` and `slot-*` diagnostic points at the offending node's
own statement address. An `edge-*` diagnostic points at the address of the node
that *receives* the value, falling back to the source node, then to
`(null, "root", 0)`.

### Deliberately not checked here

- **Command and selector grammar.** `string` values of format `command`,
  `selector`, or `mcfunction` are handed to the extracted Brigadier tree for the
  target (ADR-0012). That is the command-grammar stage below; it runs after
  lowering, over the IR command lines, not in this pass.
- **Registry membership and block properties.** `spec/types.md` sections 4.5 and
  4.6 put these behind target data that may not exist for a given registry;
  syntax is checked here, membership is not.
- **A literal and an edge on the same port**, and **a data edge whose source is
  not a value node** or **whose types do not match** (`spec/graph.schema.json`
  `$defs/edge`). No built-in block is a value node, so v1 graphs have no edges
  that reach a port; these activate with the first value block.

## Codes emitted by the command-grammar stage

This stage runs after lowering, over the IR command lines (`Body.commands`),
walking the pruned Brigadier tree from target data (ADR-0012, ADR-0006). It never
touches the graph directly. Blank lines and `#`-comment lines are skipped, as the
game skips them. On any error the pack is not emitted, exactly as a validation
error stops the build.

Best-effort by design (ADR-0012): the stage checks a line against the shape of
the command tree — literal spelling, subcommand structure, whether the line is
complete — and reports only when a token can match no child at all. It does not
reimplement every argument parser (entity selectors, NBT, block states), so a
malformed *argument* inside an otherwise well-formed command can pass. It never
reports a valid command as invalid.

| Code | Severity | Condition | Fix knowable? |
|---|---|---|---|
| `command-invalid` | error | A command line does not parse against the target's Brigadier tree: an unknown command, a misspelled subcommand, a line that stops before it is complete. | sometimes — name the token the game rejected |
| `command-legacy-syntax` | error | A command line is a recognisable older form that the target no longer accepts — today, `execute` followed directly by a selector or a `~`/`^` position, the pre-1.13 form. This is the returning-creator failure mode of ADR-0009. | yes — show the modern shape |

Anchoring: a `command-*` diagnostic points at the statement address the command
line was lowered from — the `packsmith/command` node inside its function body,
or `(<mcfunction node>, "source", <line index>)` for a line inside a raw function
file.

Parameters: `command` (the offending line) is recorded on both. `command-invalid`
also records `token` (the word the walk rejected, absent when the whole line is
unknown). `command-legacy-syntax` records `selector` (the token that gave it
away). Only `code`, `severity`, and `address` are asserted by conformance;
`conformance/cases/legacy-syntax-rejected` pins `command-legacy-syntax`.
