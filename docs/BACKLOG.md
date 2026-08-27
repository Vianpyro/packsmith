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
