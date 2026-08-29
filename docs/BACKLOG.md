# Backlog

Ideas and adjacent work discovered while doing something else. Adding here is how you avoid
widening the scope of the task in front of you. Nothing here is committed to.

- `spec/` carries `LICENSE-APACHE` but no `LICENSE-MIT`, though ADR-0011 licenses it
  `MIT OR Apache-2.0`. Same check for `conformance/` and `sdk/`.
- The declarative-block template language is referenced by `block-manifest.schema.json`
  (`implementation.kind: "declarative"`) but is not specified anywhere. It needs its own
  spec document before Phase 1 writes a template expander.
- `block-manifest.schema.json` has no `implementation.kind` for a block that lowers in native
  compiler code. The built-ins do, and `BlockDescriptor::to_manifest` (task 11) fills the
  required `implementation` with a `declarative` template path that does not exist. Decide
  whether the built-ins get real manifests + templates, or the schema grows a `native`/
  `builtin` kind, before out-of-tree blocks make the manifest a hard contract (Phase 3).
- `BlockDescriptor::to_manifest` hardcodes `targets.min` (`BUILTIN_MIN_TARGET = "26.2"`) and
  `block_version` (the crate version). Task 15 (cross-block constraint resolution) is where
  the built-ins' real supported range has to be decided; revisit then.
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
- The recipe/loot-table body shapes emitted by `packsmith-blocks` are hand-written best
  guesses (`minecraft:crafting_shapeless` type string, bare-string ingredients, `{id,count}`
  result, `{type,name}` loot entry). The `recipe` case README wants the recipe type
  discriminator to come from target data, not a literal. Neither the recipe type table nor a
  per-category body schema is extracted yet; add it to `xtask sync-target` when the schema
  validator lands and move the literal out of the block.
- `packsmith/loot-table` drops the `count` on an `item_stack` drop entirely (the one case has
  no count). Carry it through — as `minecraft:set_count` or the entry's own field, whichever
  the target schema wants — once that schema is available.
- The `xtask` conformance runner does not execute `expected-diagnostics.json` cases: it
  builds `expected/` trees and leaves diagnostics cases to the structural check.
  `crates/packsmith-compiler/tests/conformance_diagnostics.rs` runs them Rust-side for now.
  The language-agnostic runner should too, which needs a machine-readable diagnostics mode on
  the `packsmith` CLI (JSON to a file or stdout) so every SDK and host is held to the same
  assertion.
- The semantic data-edge checks are deferred until a value block exists to exercise them:
  the source of an edge must be a value node, the endpoint types must be assignable
  (`spec/types.md` section 5), a slot-scoped output is invisible from outside its slot
  (section 2.4), and a port holds a literal or an edge but not both. Codes are reserved in
  `spec/diagnostics.md`; `packsmith-compiler::validate` checks only edge structure today
  (unknown node, forward reference, cycle).
- Generate `test_instance` and `test_environment` assets for a user's own pack, so Packsmith
  can emit tests for the pack it just built. Unscheduled, out of v1 scope. Under ADR-0010 the
  pack model is open: `test_instance` and `test_environment` are registry categories like any
  other, so supporting them is target data plus conformance cases, not a change to
  `packsmith-ir` or `packsmith-compiler`. Noted in ADR-0017.
