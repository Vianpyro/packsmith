# ADR-0006: The Minecraft target is a compile parameter; initial target is 26.2

- **Status:** accepted
- **Date:** 2026-08-27

## Context

Data pack formats churn hard. Recent examples: registry directories became singular in 1.21;
`pack_format` was replaced by required `min_format` / `max_format` in 25w31a / 1.21.9, with
`min_format` optionally a `[major, minor]` pair; data pack and resource pack format numbers
are independent; and versions moved to a year-based scheme after 1.21.x. Both predecessor
projects hardcoded a version and broke.

## Decision

We will make the target an explicit input to every compilation: `packsmith build --target 26.2`.
No format number, directory name, or registry name is a constant in the compiler. All of it
lives in `packsmith-mcversion` as **generated data**, extracted from the official
`version.json` and the vanilla data pack, committed with a provenance header.

Every block declares a supported target range. The compiler refuses to build when the
requested target falls outside the intersection of the ranges of the blocks in use, and says
which block is responsible.

The initial and only supported target is **Java Edition 26.2**. The exact format number for
26.2 is not recorded in this repository and must not be guessed; the extractor is a Phase 2
deliverable and the target table is a stub until then.

## Consequences

Supporting a new game release becomes a data update plus new conformance cases, rather than
a code archaeology exercise. This is the component with the longest useful life in the
project, and it is deliberately the most boring.

The cost is that a build always needs a target, including in tests and fixtures.

## Alternatives considered

- Target latest only: guarantees breakage on every release and orphans every existing project.
- A `LATEST` sentinel: makes builds non-reproducible, since the same graph produces different
  output over time.
