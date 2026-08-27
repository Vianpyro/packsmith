# ADR-0010: The pack model is open; categories and pack kinds are data

- **Status:** accepted
- **Date:** 2026-08-27
- **Resolves:** OPEN-QUESTIONS A2, A3

## Context

v1 covers `function`, tags, `recipe`, `loot_table`, `advancement`, `predicate`, and
`item_modifier`. Worldgen and resource packs are explicitly out of v1, but neither is a
separate product: both must be reachable later without redesigning the IR.

## Decision

We will make the set of pack kinds and the set of registry categories **open**, and describe
them as target data rather than as compiler code.

- An IR document holds a list of packs, each with a `kind` (`data`, `resource`, ...). v1
  emits exactly one, of kind `data`. The list shape exists from the first schema version.
- A resource carries a `category` string that may contain slashes: `function`, `tags/function`,
  `worldgen/biome`. The compiler does not enumerate categories. It looks each one up in the
  target data table, which supplies the directory path, the file extension, and the schema to
  validate the body against.
- v1 ships target data for the seven categories above, and nothing else. Adding worldgen is
  then a data update plus conformance cases, with no change to `packsmith-ir` or
  `packsmith-compiler`.

There must be no `enum Category` in Rust and no `enum` of categories in the JSON Schemas.

## Consequences

Worldgen and resource packs become scope decisions rather than architecture decisions, which
is the whole point. Unknown categories fail with a clear "not supported for this target"
diagnostic instead of a parse error.

The cost is that we lose exhaustive matching in Rust: the compiler cannot be statically sure
it handled every category. Validation moves to the target data table, which must therefore be
covered by its own tests.

## Alternatives considered

- A closed enum with a v2 migration: cheaper now, and every closed enum in this domain has
  had to be reopened within a year of Minecraft's release cadence.
