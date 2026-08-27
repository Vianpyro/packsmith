# ADR-0011: Split licensing

- **Status:** accepted
- **Date:** 2026-08-27
- **Resolves:** OPEN-QUESTIONS B5

## Context

A single licence cannot serve both goals: keeping a hosted commercial fork from taking the
platform without giving back, and letting anyone write and ship a block without thinking
about licensing first.

## Decision

- `AGPL-3.0-or-later` for the editor, the server, and the registry.
- `MIT OR Apache-2.0` for `spec/`, `conformance/`, and every SDK under `sdk/`.

Each licensed area carries its own `LICENSE` file and an SPDX identifier in its manifest.
See `docs/LICENSING.md` for the boundary and the reasoning.

## Consequences

The platform stays open even if someone hosts it commercially, while the block ecosystem has
no licensing friction. Blocks written against the SDKs may be licensed however their authors
want, including proprietarily.

The cost is a boundary that must be maintained: any code moving between an AGPL crate and an
SDK crate needs a deliberate decision, not a copy-paste.

## Alternatives considered

- MIT everywhere (as in the predecessor projects): gives the platform away.
- AGPL everywhere: an AGPL SDK makes people hesitate before writing their first block, which
  suppresses the ecosystem the project depends on.
