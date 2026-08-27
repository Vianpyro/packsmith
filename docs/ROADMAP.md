# Roadmap

Phases are gates, not suggestions. Do not start a phase before the previous one meets its
exit criteria. Each phase exists to invalidate an assumption cheaply, before the next phase
builds on it.

The current phase is recorded in `CLAUDE.md`. Update it there when a gate is passed.

---

## Phase 0 — Specification

Nothing is implemented. The output is the contract.

- `spec/graph.schema.json` — node, port, edge, literal, block reference, project metadata.
- `spec/ir.schema.json` — the normalized pack description.
- `spec/block-manifest.schema.json` — id, version, inputs, outputs, supported target range.
- `spec/wit/packsmith-block.wit` — the computed-block interface (written now, unused until
  Phase 3, so that the IR is designed with it in mind).
- `spec/CHANGELOG.md`.
- 5 to 10 conformance cases covering: an empty pack, one function, a function tag, a recipe,
  a loot table, a raw-mcfunction escape hatch, and one deliberate validation failure.
- Answers recorded for every blocking item in `OPEN-QUESTIONS.md`.

**Exit:** the schemas validate every conformance input, the expected trees are hand-verified
against a real game instance, and no open question is still blocking.

---

## Phase 1 — Compiler core, CLI only

No UI. No registry. No sandbox. Built-in declarative blocks only. Single target.

- `packsmith-ir`, `packsmith-mcversion` (stub table), `packsmith-compiler`, `packsmith-emit`,
  `packsmith-blocks`, `packsmith-cli`.
- `packsmith build <project> --target 26.2` produces a `.zip`.
- Diagnostics carry node ids and suggested fixes.
- `cargo xtask ci` runs the conformance suite.

**Exit:** a generated pack loads in Java Edition 26.2 with no compatibility warning, its
functions run, and building the same project twice produces identical bytes.

---

## Phase 2 — Version targeting for real

- Extractor that pulls `version.json` and the vanilla data pack from an official jar and
  emits the generated target data, with provenance.
- Constraint resolution across blocks, with diagnostics naming the offending block.
- Both `pack.mcmeta` shapes (legacy single format, modern range).
- Integration harness: a headless server in Docker loads the pack, runs `/reload` and a set
  of test functions, and asserts on the output.

**Exit:** the same graph compiles for three different targets, and the integration harness
runs in CI.

---

## Phase 3 — Computed blocks, Rust SDK only

One language. Validate the ABI before multiplying it.

- `packsmith-host`: wasmtime host with fuel, memory, timeout, and output-size limits.
- `sdk/rust`.
- Content-addressed build cache.
- Conformance cases for computed blocks, including a block that tries to escape the sandbox
  and must be rejected.

**Exit:** an out-of-tree computed block passes the conformance suite, and the sandbox tests
demonstrate that filesystem, network, and clock access all fail.

---

## Phase 4 — Editor

- Graph editor in the browser. Use an existing graph or block library; do not write one.
- The compiler runs client-side as WASM (see `OPEN-QUESTIONS.md` B2 if this is still open).
- Load and save projects as plain files.
- Live diagnostics from the compiler, positioned on nodes.

**Exit:** someone who has never written an `.mcfunction` builds and downloads a working pack
without reading documentation.

---

## Phase 5 — Registry and sharing

- Block publishing, semantic versioning, lockfile, content-addressed storage, signatures.
- A default index, with federation so that any instance can run its own.
- Self-host deployment: one container, no external dependencies for the single-user case.

**Exit:** a block published from one instance is installed and built on another.

---

## Phase 6 — Remaining SDKs

In this order, by descending toolchain maturity: TypeScript, then Python, then Java. Java may
land on the OCI runner rather than as a WASM component; that is acceptable and expected.

**Exit:** each SDK passes the same conformance suite as the Rust SDK.
