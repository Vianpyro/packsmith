# ADR-0003: The graph and the IR are data, not code

- **Status:** accepted
- **Date:** 2026-08-27

## Context

The predecessor projects (`minecraft_with_python`, `minecraft_datapacks_generator`) expressed
a data pack as a Python object graph. That makes the pack a program: it cannot be inspected,
diffed, validated, migrated, or rendered in a visual editor without executing Python.

## Decision

We will define two serialized, schema-validated formats in `spec/`:

- **Graph** — what the user edits. Nodes, ports, edges, literal values, block references.
- **IR** — what the compiler produces. A normalized, target-resolved description of the pack
  contents, one level above raw files.

Neither format may contain executable expressions, embedded source, or anything requiring a
language runtime to interpret. Computation happens in blocks, whose *output* is IR.

## Consequences

The graph is diffable and git-friendly, the editor needs no runtime to load a project, and
migrations between schema versions are ordinary data transforms. Validation and good
diagnostics become possible because the compiler can see the whole program before running
anything.

The cost is expressive power: things a user could once write inline as Python now require a
block. This is deliberate, and it is what makes the platform safe and shareable.

## Alternatives considered

- An embedded scripting language in the graph (Rhai, Lua): reintroduces a runtime dependency
  in the editor, an evaluation-order problem in the compiler, and a sandbox surface in a place
  we would rather keep inert.
