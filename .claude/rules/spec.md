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

## Cases that expect a compile failure

A failure case replaces `expected/` with one file, `expected-diagnostics.json`:

```json
{
  "outcome": "compile-failure",
  "diagnostics": [
    { "code": null, "severity": "error",
      "address": { "node": "<id>", "slot": "<name>", "index": 0 },
      "message": "<human text, for readers only>" }
  ]
}
```

- **Asserted:** `outcome` (and that no pack is emitted), and per diagnostic `code`,
  `severity`, and `address` (node, slot, index) — matched as an unordered set, same count.
- **Not asserted:** `message`. It is recorded for readability; never pin wording.
- `code: null` marks a code the compiler does not emit yet. It stays satisfied until a real
  code replaces it, in its own commit. Do not invent the code.
- `address` is the `statement-address` shape from `spec/ir.schema.json`.

The full field-by-field rules live in `conformance/README.md`; keep the two in step.
