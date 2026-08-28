# Roadmap

Phases are gates, not suggestions. Do not start a phase before the previous one meets its
exit criteria. Each phase exists to invalidate an assumption cheaply, before the next phase
builds on it.

The current phase is recorded in `CLAUDE.md`. Update it there when a gate is passed.

---

## Phase 0 - Specification

Nothing is implemented. The output is the contract.

- `spec/graph.schema.json` - node, port, input value, connection, project metadata.
- `spec/ir.schema.json` - the normalized pack description. Open category model (ADR-0010).
- `spec/block-manifest.schema.json` - id, version, ports, supported target range.
- `spec/wit/packsmith-block.wit` - the computed-block interface. Written now, unused until
  Phase 3, so that the IR is designed with it in mind.
- `spec/types.md` - the port type system.
- `spec/CHANGELOG.md`.
- 5 to 10 conformance cases: empty pack, one function, a function tag, a recipe, a loot table,
  the raw-mcfunction escape hatch, and one deliberate validation failure.
- Every blocking item in `OPEN-QUESTIONS.md` answered.

**Exit:** the schemas validate every conformance input, the expected trees are hand-verified
against a real game instance, and no blocking question remains open.

---

## Phase 1 - Compiler core and target data, CLI only

No UI. No registry. No sandbox. Built-in declarative blocks only.

The target data pipeline lands here rather than in Phase 2, because ADR-0014 made it cheap:
mcmeta removes the jar download, the JDK in CI, and the EULA step.

- `cargo xtask sync-target --version 26.2`: fetch from a pinned mcmeta commit, extract the
  thin functional subset, write `crates/packsmith-mcversion/data/26.2.json` with its
  provenance header and SPDX marker.
- Confirm the real 26.2 pack format from the released `version.json`. Do not carry the
  pre-release number forward.
- `packsmith-ir`, `packsmith-mcversion`, `packsmith-compiler`, `packsmith-emit`,
  `packsmith-blocks`, `packsmith-cli`, `xtask`.
- `packsmith build <project> --target 26.2` produces a `.zip`.
- Command validation against the extracted Brigadier tree (ADR-0012), with diagnostics that
  name the game concept before the file path.
- Diagnostics carry node ids and, where possible, a suggested fix.
- `cargo xtask ci` runs the conformance suite and the reproducibility check.

**Exit:** a generated pack loads in Java Edition 26.2 with no compatibility warning, its
functions run, building the same project twice produces identical bytes, and pasting 1.16-era
syntax into a raw block produces a precise error rather than a silent failure in game.

---

## Phase 2 - Multi-target and in-game verification

- Extract a second and third target. Constraint resolution across blocks, with diagnostics
  naming the offending block.
- Both `pack.mcmeta` shapes: legacy single format, and the modern range.
- Integration harness: vanilla headless GameTest (`net.minecraft.gametest.Main`), invoked from
  `xtask`, builds a conformance case and a generated test pack and runs the game against it.
  Assertions live in the test pack as `test_instance` assets with `function`-type environments,
  not in log scraping. No mod loader. This is the same jar invocation family as the `--reports`
  fallback for extracting target data when mcmeta is unavailable (ADR-0017, ADR-0014).

**Exit:** the same graph compiles for three targets, and the integration harness runs in CI.

---

## Phase 3 - Computed blocks, Rust SDK only

One language. Validate the ABI before multiplying it.

- `packsmith-host`: wasmtime host with fuel, memory, timeout, and output-size limits.
- `sdk/rust`.
- Content-addressed build cache.
- Conformance cases for computed blocks, including a block that attempts to escape the sandbox
  and must be rejected.

**Exit:** an out-of-tree computed block passes the conformance suite, and the sandbox tests
demonstrate that filesystem, network, and clock access all fail.

---

## Phase 4 - Editor

- Graph editor in the browser. Use an existing graph or block library; do not write one.
- The compiler runs client-side as WASM (ADR-0008). Computed blocks run in a browser host
  (jco) enforcing the same limits as the native host.
- Projects load and save as plain files (`graph.json` + `layout.json`, ADR-0013).
- Live diagnostics from the compiler, positioned on nodes.

**Exit:** someone who has never written an `.mcfunction` builds and downloads a working pack
without reading documentation.

---

## Phase 5 - Registry and sharing

- Block publishing, semantic versioning, lockfile, content-addressed storage, signatures.
- A default index, with federation so any instance can run its own.
- Self-host deployment: static assets plus one small service, no external dependencies.

**Exit:** a block published from one instance is installed and built on another.

---

## Phase 6 - Remaining SDKs

In this order, by descending toolchain maturity: TypeScript, then Python, then Java. Java may
land on the OCI runner rather than as a WASM component; that is acceptable and expected.

**Exit:** each SDK passes the same conformance suite as the Rust SDK.

---

## Explicitly after v1

Worldgen and resource packs. ADR-0010 makes both a matter of adding target data and
conformance cases, not of redesigning the IR. If either one ever requires a change to
`packsmith-ir` or `packsmith-compiler`, ADR-0010 was implemented wrong.
