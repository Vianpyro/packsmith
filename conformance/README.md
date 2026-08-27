# conformance/

Golden cases. Each one pins a `(graph, target)` pair to the exact file tree Packsmith must
produce. The suite is language-agnostic and normative: every SDK and every host passes it,
not just the Rust compiler. See `.claude/rules/spec.md` before editing.

## Case format

A case is a directory under `cases/` containing exactly four things:

```
cases/<case-name>/
  input.json     The graph.
  target.json    The Minecraft target to compile against.
  expected/      The exact file tree, as it appears inside the built pack.
  README.md      One paragraph: what this case pins down.
```

- `input.json` is a graph document conforming to `spec/graph.schema.json`. Semantics only —
  node positions live in a separate layout file and never reach a build (ADR-0013).
- `target.json` names the target version. The target is a compile parameter, never a
  constant, so it is always explicit and never a `LATEST` sentinel (ADR-0006).
- `expected/` mirrors the pack contents file for file, including `pack.mcmeta`. Builds are
  deterministic, so this is an exact match, not a fuzzy one (ADR-0007).
- `README.md` states the single behaviour the case exists to protect.

A case that expects a compile failure carries the expected diagnostics instead of
`expected/`; a validation failure is a result worth pinning down just as much as a
successful build.

## Rules

- **Cases are small and single-purpose.** One behaviour per case. A case that would need two
  sentences of `README.md` is two cases.
- **Never edit an existing case to make failing code pass.** Add a new case, or fix the code.
  If an expected tree was genuinely wrong, correcting it is a standalone commit with a
  written justification and no implementation change in the same diff.
- Expected trees are hand-verified against a real game instance before they are committed.

## Licence

`MIT OR Apache-2.0` — Zone 2, the contract and the tools. See `docs/LICENSING.md`.
