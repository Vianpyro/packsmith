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
                 A failure case replaces this with expected-diagnostics.json
                 (see "Cases that expect a compile failure" below).
  README.md      One paragraph: what this case pins down.
```

- `input.json` is a graph document conforming to `spec/graph.schema.json`. Semantics only —
  node positions live in a separate layout file and never reach a build (ADR-0013).
- `target.json` names the target version. The target is a compile parameter, never a
  constant, so it is always explicit and never a `LATEST` sentinel (ADR-0006).
- `expected/` mirrors the pack contents file for file, including `pack.mcmeta`. Builds are
  deterministic, so this is an exact match, not a fuzzy one (ADR-0007).
- `README.md` states the single behaviour the case exists to protect.

### Cases that expect a compile failure

A validation failure is a result worth pinning down just as much as a successful build. Such a
case replaces `expected/` with a single file, `expected-diagnostics.json`:

```json
{
  "outcome": "compile-failure",
  "diagnostics": [
    {
      "code": null,
      "severity": "error",
      "address": { "node": "fn-legacy", "slot": "body", "index": 0 },
      "message": "execute no longer takes a bare selector and position; write `execute as <targets> at @s run <command>`"
    }
  ]
}
```

**Compared fields — the assertion:**

- `outcome` — must be `"compile-failure"`, and the build must emit no pack.
- For every diagnostic: `code`, `severity`, and `address` (all three of `node`, `slot`,
  `index`), matched exactly.

The runner matches the produced diagnostics to the expected ones as sets: same count, each
expected diagnostic paired with a distinct produced one on the compared fields. Order is not
significant.

**Not compared:**

- `message` — recorded in the file so a reader can see what the diagnostic says, never
  asserted. Pinning wording turns every rephrasing into a test failure, and the message is the
  part most likely to improve.

**Pending code:** `code: null` means the diagnostic code is not yet pinned because the
compiler does not emit it yet. The runner still checks `outcome`, `severity`, and `address`;
it treats the `code` field as satisfied until a real code replaces `null`. Filling in a code
is its own commit, like correcting an expected tree.

`severity` is `"error"` or `"warning"`. A `compile-failure` case carries at least one `error`.
`address` uses the statement-address shape from `spec/ir.schema.json`: `node` is the id of the
node owning the slot, or `null` for the graph's top-level `root` slot.

## Rules

- **Cases are small and single-purpose.** One behaviour per case. A case that would need two
  sentences of `README.md` is two cases.
- **Never edit an existing case to make failing code pass.** Add a new case, or fix the code.
  If an expected tree was genuinely wrong, correcting it is a standalone commit with a
  written justification and no implementation change in the same diff.
- Expected trees are hand-verified against a real game instance before they are committed.

## Licence

`MIT OR Apache-2.0` — Zone 2, the contract and the tools. See `docs/LICENSING.md`.
