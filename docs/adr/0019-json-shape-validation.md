# ADR-0019: Emitted JSON is validated against the target's mcdoc schema

- **Status:** proposed
- **Date:** 2026-08-29
- **Refines:** ADR-0006
- **Related:** ADR-0012, ADR-0014, ADR-0017

## Context

ADR-0006 keeps every Minecraft version fact out of the compiler: format numbers, directory
names, and registry ids all live in `packsmith-mcversion` as extracted data. A declarative
block breaks the clean version of that rule. `packsmith/crafting-shapeless` cannot lower a
node to a recipe without knowing that a recipe has a `type`, that the type string is
`minecraft:crafting_shapeless`, that ingredients are a list and the result is a stack with an
`id` key. That shape *is* a version fact -- keys and type strings move between releases the
same way command arguments do -- and it is written from memory in `crates/packsmith-blocks`
today, which `.claude/rules/minecraft.md` forbids everywhere else.

Shape knowledge cannot be extracted out of `packsmith-blocks` the way format numbers were
extracted out of the compiler: a block's whole job is to produce a particular shape. What can
be moved elsewhere is the *check* that the shape is right.

## Decision

We will validate the JSON a build emits against the target's data pack schemas rather than
trusting the block that produced it. A block stays free to build whatever object it likes;
a compiler stage then checks that object against the schema for its registry category and the
resolved target, and reports a diagnostic (ADR-0012 style: code, node id, suggested fix) when
it does not conform.

The schema source is [SpyglassMC/vanilla-mcdoc](https://github.com/SpyglassMC/vanilla-mcdoc):
machine-readable, versioned schemas covering every data pack JSON file. This is the JSON
counterpart to the Brigadier command tree that ADR-0012 validates commands against, and it
fits the same extract-and-vendor pipeline as ADR-0014 -- fetched from a pinned commit,
reduced to the subset the validator needs, committed with a provenance header, loaded at
runtime.

## Consequences

The version rule holds for `packsmith-blocks` the way it holds everywhere else: a block may
still be wrong about a shape, but a wrong shape is caught by tooling against real schema data
instead of surviving on review. Adding a target becomes a data update. The check runs over
the artefact the user installs.

The cost is real. mcdoc is a custom schema language, not JSON Schema; parsing and evaluating
it in Rust is significant work -- a lexer, a resolver for its module and reference system, and
an evaluator that handles its dispatch-on-a-sibling-field construct. Spyglass's own
implementation is TypeScript. This is a Phase 2+ item, not a small one, and it must not pull
in a new dependency to get started.

## Until then

Built-in block shapes are verified against a running game (ADR-0017) before the block's
conformance case is marked passing. That in-game pass is the gate -- not a maintainer reading
the JSON and judging it plausible. The hardcoded recipe type string and loot entry shape in
`packsmith-blocks` are pulled into named constants with a comment pointing here, so the places
that need the validator are greppable when it lands.

## Alternatives considered

- **Trust the block, rely on review.** The status quo. Rejected: it is exactly the
  "version fact from memory" failure ADR-0006 exists to prevent, and review does not catch a
  key that was renamed two releases ago.
- **Hand-write JSON Schemas for the shapes we emit.** Smaller upfront cost, but it recreates
  the from-memory problem one level up and rots on every release, with no upstream to diff
  against.
- **Convert mcdoc to JSON Schema in `xtask` and validate with an existing JSON Schema crate.**
  Attractive -- it confines the hard part to a build tool and reuses a validator. Worth
  investigating when this ADR is taken up; the open question is whether mcdoc's dispatch
  construct survives the translation without losing the checks that matter.
- **Static validation with Spyglass itself.** Real and good, and the wrong language for our
  core (TypeScript). Same conclusion as ADR-0017: useful as a reference, not as a component.
