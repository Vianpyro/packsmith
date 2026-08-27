# Backlog

Ideas and adjacent work discovered while doing something else. Adding here is how you avoid
widening the scope of the task in front of you. Nothing here is committed to.

- `spec/` carries `LICENSE-APACHE` but no `LICENSE-MIT`, though ADR-0011 licenses it
  `MIT OR Apache-2.0`. Same check for `conformance/` and `sdk/`.
- The declarative-block template language is referenced by `block-manifest.schema.json`
  (`implementation.kind: "declarative"`) but is not specified anywhere. It needs its own
  spec document before Phase 1 writes a template expander.
- Schema validation in CI: `cargo xtask ci` should validate every conformance `input.json`
  against `spec/graph.schema.json`, and the schemas themselves against draft 2020-12.
- The raw-mcfunction escape hatch (conformance case `raw-mcfunction`) needs a string format
  for a whole function file — split on newlines, skip blank and `#`-comment lines, validate
  the rest as commands. `spec/types.md` section 4.4 already makes formats open and
  target-data-supplied, but the section 7 coverage row for `function` lists only
  `string(command)` and `string(selector)`. Either add the file-level format to that row or
  decide the escape hatch is `list<string(command)>` and adjust the conformance case.
- `conformance/` has no schema for `target.json`. The cases currently use `{ "id": "26.2" }`,
  matching the `target` object in `ir.schema.json`. Worth a one-object schema so the runner
  can reject a malformed target.
- CI runs `reuse lint-file` scoped to `crates/packsmith-mcversion/data/` only. Promote it to a
  whole-repo `reuse lint` once every file carries SPDX info (via headers or `REUSE.toml`
  aggregate annotations for the two licence zones) and `LICENSES/` holds the canonical texts
  (`AGPL-3.0-or-later`, `MIT`, `Apache-2.0`, `LicenseRef-Minecraft-Derived`).
- `conformance/` has no schema for `expected-diagnostics.json` (the compile-failure result
  format defined in `conformance/README.md`). A one-object schema would let the runner reject
  a malformed expectation instead of misreading it.
