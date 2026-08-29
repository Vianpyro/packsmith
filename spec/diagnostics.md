# Diagnostic codes

- **Spec version:** 0 (unstable; Phase 0)
- **Status:** normative
- **Constrained by:** ADR-0009 (who reads these), ADR-0012 (command grammar is a
  separate stage), ADR-0016 (statement addresses).
- **Licence:** `MIT OR Apache-2.0`, as all of `spec/`.

A **diagnostic** is a structured, user-facing result the compiler produces when a
graph is wrong or questionable. It carries:

| Field | Meaning | Asserted by conformance? |
|---|---|---|
| `code` | a stable identifier for the *condition* | **yes** |
| `severity` | `error` or `warning` | **yes** |
| `address` | the statement address it points at, `(node, slot, index)`; `node` is `null` for the top-level `root` slot | **yes** |
| `message` | one sentence, game terms first, file terms second | no |
| `fix` | a concrete suggested edit, or absent when none is knowable | no |

`message` and `fix` are the wording the two personas of ADR-0009 read. They are
recorded but never asserted, so they can be reworded without breaking a case.
The full `expected-diagnostics.json` rules are in `conformance/README.md`.

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
| `command-` | a command string's grammar — **owned by the Brigadier stage (ADR-0012)**, listed below but not emitted by the validation pass |

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

- **Command and selector grammar.** `string` values of format `command` or
  `selector` are handed to the extracted Brigadier tree for the target
  (ADR-0012). That stage owns the `command-` codes and is a separate task.
- **Registry membership and block properties.** `spec/types.md` sections 4.5 and
  4.6 put these behind target data that may not exist for a given registry;
  syntax is checked here, membership is not.
- **A literal and an edge on the same port**, and **a data edge whose source is
  not a value node** or **whose types do not match** (`spec/graph.schema.json`
  `$defs/edge`). No built-in block is a value node, so v1 graphs have no edges
  that reach a port; these activate with the first value block.

## Reserved: `command-` codes

Declared here so a conformance case can reference one and so the validation pass
does not reuse the prefix. Emitted by the command-grammar stage, not by this
pass.

| Code | Severity | Condition |
|---|---|---|
| `command-invalid` | error | A `command` or `selector` string does not parse against the target's Brigadier tree. |
| `command-legacy-syntax` | error | A `command` string is valid for an older release but not the target: `execute` with a bare selector and position, a removed argument, a renamed subcommand. This is the returning-creator failure mode of ADR-0009. |

Until that stage lands, a case that means to assert one of these uses `code: null`
(see `conformance/cases/legacy-syntax-rejected`).
