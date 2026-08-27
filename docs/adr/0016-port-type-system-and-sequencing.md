# ADR-0016: Ordered child slots express sequencing; a small closed set of port types

- **Status:** accepted
- **Date:** 2026-08-27

## Context

Phase 0 owes `spec/types.md`, and no schema can be written before two questions are settled.

The first is sequencing. A data pack function is an ordered list of commands. Order is
semantic: `kill @e` before `summon` is a different pack from `summon` before `kill @e`. The graph must express that order, and it must express it as data (ADR-0003), without node coordinates (ADR-0013), and in a form the two personas of ADR-0009 can read.

The second is the port type system itself. Ports are typed (GLOSSARY: *Port*), and every type is a concept a newcomer must learn, a case every SDK must implement (ADR-0002, Phase 6), and a validator the compiler must carry. The v1 surface is seven registries (ADR-0010): `function`, tags, `recipe`, `loot_table`, `advancement`, `predicate`, `item_modifier`. ADR-0012 removes the largest possible source of type-system bloat by keeping commands as validated text, so the type system does not have to model the command grammar.

## Decision

We will express order by **containment**: a node may declare named, ordered **slots**, each holding a list of child nodes. Order inside a slot is the order of the list, and that list is the only thing that means order. Data flow stays as typed edges between ports and carries no ordering meaning at all, because blocks are pure (ADR-0005).

We will define eleven port types and no more, specified in `spec/types.md`. Entity selectors and commands are *not* separate types: they are `string` values with a `format` tag, validated by the machinery ADR-0012 already requires.

### Sequencing: the three candidates

**A. Scratch-style vertical stacking.** Statements are nodes on a canvas; a node's order is its next-sibling relationship, drawn as physical snapping.

**B. Explicit execution-flow ports, Blueprint-style.** Each statement node carries an `exec` input pin and one or more `exec` output pins. An edge from one node's `exec-out` to another's `exec-in` means "then".

**C. Ordered child slots on a container node.** A container node (a function, a conditional, an `execute` wrapper) owns one or more named slots; each slot is an array of child nodes, and the array index is the order.

### Judged against the criteria

**Discoverability for someone who has never coded.** A wins on sight, and C ties it, because C is what Scratch actually *is* underneath: a Scratch C-block holds a substack array, not a pointer to a floating neighbour. The stacking metaphor is a rendering of an ordered child list, so choosing C costs nothing in discoverability and buys a data model. B loses badly. The newcomer must first learn that a wire can mean "then" as well as "this value", must tell the execution wires from the data wires, and must understand that a node with no incoming execution wire is dead code that looks identical to live code.

**Whether an invalid order is representable at all.** This is where the candidates separate.
C makes an invalid order *unrepresentable*: an array is a total order by construction, a statement is in exactly one slot at exactly one index, and there is no dangling statement, no statement with two predecessors, and no cycle. B can represent every one of those errors, so each becomes a validation rule, a diagnostic, an editor guard rail, and a conformance case.
A sits in between: the data would be fine, but the ordering lives in coordinates, which ADR-0013 forbids from reaching the build hash. Making A work means either breaking ADR-0013 or storing a sibling-pointer chain in `graph.json` -- a hand-rolled linked list, which can dangle, fork, and cycle exactly like B, while looking like a list.

**How a diagnostic points at the offending place.** Under C, a statement's address is
`(node id, slot name, index)`, and every part of it is stable and pronounceable: "step 3 of the *Otherwise* branch of *When a player sleeps*". Under B, the offending thing is often an edge, and an edge is not something the newcomer perceives as an object with an identity; "the execution wire into node 7 is missing" names a thing they never knowingly created. Under A, the address is a coordinate, which cannot appear in a CLI diagnostic at all.

**Conditionals and execute-modifier chains.** C fits Minecraft's own shape. `execute` is literally a wrapper around one command: `execute <modifiers> run <command>`. Under C, an execute node carries its modifiers as ordinary typed inputs and its target as a `run` slot; wrapping three existing commands in "as each zombie, at itself" is a structural move of three children into a slot, which is a gesture, not a rewiring. Nesting is nesting. A conditional is the same shape with two slots, `then` and `else`. Under B, the same edit means re-routing four execution edges by hand, and a half-finished re-route is a representable graph. Under A, conditionals require C-block-shaped nodes anyway, which is C with the ordering left in the coordinates.

**Editor cost in Phase 4.** C needs list rendering, drag-to-reorder, and drop-into-slot. It needs no execution-edge routing, no cycle detection on control flow, no orphan-subgraph analysis, and no auto-layout for execution order, because the order *is* the layout. B needs all of those, and the editor must additionally teach the difference between two kinds of wire.
A needs snapping, collision, and a coordinate-to-order reconciliation pass that must stay correct while nodes are being dragged.

### What we choose, and what it gives up

We choose **C**, rendered in the editor as **A**. The user sees stacked, snapping blocks; the file holds ordered arrays.

What this gives up, plainly:

- **Arbitrary control flow.** No reconvergence, no jumping into the middle of a sequence, no
  node with two execution outputs feeding one shared continuation. Minecraft has no `goto`; it
  has `run` and it has function calls, which are containment and a named reference
  respectively. If a future feature genuinely needs a control-flow graph, this ADR is what has
  to be superseded.
- **A flat node set.** `graph.json` becomes a forest, not a bag of nodes plus edges. Node ids
  stay globally unique and data edges still address nodes by id, but move, copy, and delete
  become tree operations, and the schema is recursive.
- **One visual idiom for order, a second for data.** Order is nesting; data is wires. This
  hybrid is a real cost, and it is the honest reason B is tempting: B has one idiom for both.
  We accept it because Scratch demonstrates the hybrid is learnable by children, and because
  ADR-0012 keeps most function bodies free of data wires entirely.
- **Deep nesting is a rendering problem.** A long chain of nested `execute` wrappers indents
  forever. That is a Phase 4 presentation concern, not a model concern.

## Consequences

The graph schema gains recursion and loses an entire class of validation rules. Determinism (ADR-0007) gets easier: statement order is already an array, so nothing has to be sorted into a canonical shape on the way to the IR, and no ordering depends on map iteration.

The compiler's statement address -- node id, slot name, index -- becomes the anchor for every diagnostic about a function body, which is what ADR-0009 requires: it names the block the user placed before it names the file the result landed in.

Flattening is a lowering concern, not a graph concern. A wrapper node with several children in its `run` slot lowers to several `execute ... run` lines, or to a generated sub-function where the wrapper's semantics need one; that is the wrapper block's business, decided per block.

Eleven types is a ceiling we are deliberately setting low. Adding a twelfth means changing four SDKs, the conformance suite, and the editor, and teaching one more thing to someone who does not yet know what a namespace is. The cost is that some values are modelled loosely -- an item stack's components are a pass-through object with key-existence validation only.

## Alternatives considered

- **Scratch-style stacking as the data model (A).** Rejected because ordering by position
  contradicts ADR-0013, and the sibling-pointer version of it is a hand-rolled linked list with
  all of B's failure modes and none of B's honesty about being a graph. We keep its appearance,
  which is the part that serves the newcomer.
- **Execution-flow ports (B).** Rejected because it makes invalid programs representable by
  default, addresses errors at edges rather than at things the user placed, turns "wrap these in
  an execute" into a four-edge rewiring, and is the most expensive option in Phase 4. What it
  buys is arbitrary control flow, which Minecraft does not have.
- **Implicit ordering by data dependency.** Rejected outright: commands are side effects, most
  of them share no data, and any tie-break would be either arbitrary or positional.
- **An `int` order field on each statement, siblings sorted by it.** Rejected: it admits
  duplicates and gaps, requires renumbering on every insert, and produces diffs unrelated to the
  edit.
- **Separate `selector` and `command` port types.** Rejected in favour of `string` with a
  `format` tag. Both validate by handing text to the target grammar (ADR-0012); making them
  distinct types adds two concepts and two SDK cases to buy one validator call each.
- **A `range` port type** for the `{min, max}` shapes that predicates and advancement criteria
  use. Rejected: a block exposes two optional `int` inputs instead. No new type, no new SDK case.
- **A generic `object` or `struct` type** so users can wire arbitrary JSON. Rejected: it is an
  escape hatch that would swallow the type system, and it serves neither persona. Recipes and
  loot tables are produced by blocks, not hand-assembled from key-value pairs.
