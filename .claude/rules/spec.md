---
paths:
  - 'spec/**'
  - 'conformance/**'
---

# Spec and conformance rules

`spec/` and `conformance/` are the contract between the compiler, every SDK, and every host.
They are normative. Treat a change here the way you would treat a change to a wire protocol.

- Every schema carries an explicit `version` field. Schema changes are additive within a major
  version. A breaking change requires a major bump and a migration note in `spec/CHANGELOG.md`.
- Never edit an existing conformance case to make failing code pass. Add a new case instead,
  or fix the code. If the expected output was genuinely wrong, that is a standalone commit
  explaining why, with no implementation change in the same diff.
- A conformance case is a directory: `input.json` (graph), `target.json` (Minecraft target),
  `expected/` (the exact file tree), and `README.md` (one paragraph: what this case pins down).
- Cases must be small and single-purpose. One case per behaviour.
- Schemas are language-agnostic. No Rust type names, no Rust semantics leaking into the spec.
