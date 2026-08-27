# spec changelog

Normative changes to `spec/`. Each schema carries its own `version` field, holding the major
version of that format. Changes are additive within a major version; a breaking change bumps
the number and gets a migration note here.

The format is loosely [Keep a Changelog]. Dates are the date the change landed.

[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/

## Unreleased

### Added

- `graph.schema.json` (format version 0). Recursive: a graph is a forest of nodes under a
  top-level `root` slot, and order is the index of a node inside an ordered slot (ADR-0016).
  Data edges are a separate top-level list addressing nodes by id and carry no ordering
  meaning. Ten literal forms, one per value type; `body` has no literal because it appears
  only on a slot. Covers `graph.json` only: `layout.json` is out of scope and never reaches
  the compiler or the build hash (ADR-0013).
- `ir.schema.json` (format version 0). A list of packs, each with an open `kind`, each holding
  resources whose `category` is a free-form slash-separated string looked up in target data
  (ADR-0010). Resource bodies and command lines are tagged forms, so a structured command
  variant can join them without a break (ADR-0012). Every resource carries the statement
  address it came from.
- `block-manifest.schema.json` (format version 0). One manifest for both block tiers
  (ADR-0004), with a tagged `implementation`, a required SPDX `license`, a supported target
  range (ADR-0006), and the eleven type references of `spec/types.md`.
- `wit/packsmith-block.wit` (package `packsmith:block@0.1.0`). The computed-block contract:
  `describe()` and a pure `emit()` returning an IR patch or diagnostics. Written now so the IR
  is designed against it; not implemented until Phase 3 (ADR-0005).
- `types.md` (spec version 0). The port type system and sequencing rules (ADR-0016).

### Notes

No format here is stable. Version 0 means Phase 0: the schemas may change in any direction
until the first conformance run against a real game instance passes.
