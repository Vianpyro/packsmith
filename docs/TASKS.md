# Tasks

One numbered task per unit of work. Each task states its goal, the ADRs that govern it, its
exact scope, what is explicitly out of scope, and the one condition that closes it.

Tasks 1-10 are backfilled from git history: they record what was actually done, not a plan.
Task 11 onward are planned; from task 12 they come straight from `docs/ROADMAP.md`.

Every task carries a `**Status.**` line: `done`, `in progress`, `not started`, or
`blocked on <what>`. A task is executed with `/task <n>`. That command reads this file, reads
every ADR the task lists, works only inside the stated scope, and keeps the `**Status.**` line
current.

**🔒 marks an exit criterion only the maintainer can check** — it needs a real game instance,
a JDK, or a judgement call that no test in this repo can make. `/task` stops and reports when
it reaches one of these rather than assuming success, and never marks the task `done`.

---

## 1. Repository bootstrap and the decision record

**Status.** Done.

**Goal.** Stand up the repository as a specification project: the working agreement, the
vocabulary, the phase gates, and the architecture decisions everything else answers to.

**Governing ADRs.** 0001 (use ADRs), 0002 (single compiler in Rust), 0003 (graph and IR are
data), 0004 (declarative blocks first), 0005 (block ABI is the component model), 0006 (target
is a compile parameter), 0007 (deterministic builds), 0008 (client-side compiler), 0009
(target audience), 0010 (open pack model), 0011 (licensing), 0012 (commands are validated
strings), 0013 (semantics and layout are separate).

**Scope.**
- `CLAUDE.md`, `README.md` with the trademark notice, `docs/GLOSSARY.md`, `docs/ROADMAP.md`,
  `docs/OPEN-QUESTIONS.md`, `docs/PRIOR-ART.md`, `docs/LICENSING.md`, `NOTICE`.
- ADRs 0001 through 0013 written and marked accepted.
- Cargo workspace skeleton: the seven `crates/` members plus `xtask`, each compiling empty
  with `#![forbid(unsafe_code)]`.
- `rust-toolchain.toml`, `deny.toml`, editor and devcontainer config, CI workflow files.

**Out of scope.** Any compiler, emitter, or schema logic. Any Minecraft version data.

**Exit criterion.** The workspace builds; every accepted ADR from 0001-0013 exists; the phase
gates in `ROADMAP.md` are written.

---

## 2. Port type system, sequencing model, and CI gate

**Status.** Done.

**Goal.** Fix how the graph expresses order and what a port may carry, before any schema
depends on it.

**Governing ADRs.** 0016 (ordered child slots express sequencing; small closed set of port
types), 0003 (graph and IR are data), 0007 (deterministic builds).

**Scope.**
- `docs/adr/0016-port-type-system-and-sequencing.md`.
- `spec/types.md`: the port type system, the ordered-slot sequencing rule, the coverage table
  per registry category.
- `cargo xtask ci` wired to fmt check, clippy with warnings denied, and workspace tests.
- `cargo-deny` configuration for licences and advisories.

**Out of scope.** The schemas themselves (task 3). Command grammar (task 12).

**Exit criterion.** `spec/types.md` describes every port type and the sequencing rule; ADR-0016
is accepted; `cargo xtask ci` runs the three cargo steps.

---

## 3. Normative schemas

**Status.** Done.

**Goal.** Publish the contract between the compiler, every SDK, and every host as versioned,
language-agnostic schemas.

**Governing ADRs.** 0003 (graph and IR are data), 0010 (open pack model — no category or
pack-kind enum), 0005 (block ABI), 0006 (target is a parameter), 0016 (port types).

**Scope.**
- `spec/graph.schema.json`: node, port, input value, connection, project metadata.
- `spec/ir.schema.json`: normalized pack description, open category model, `statement-address`
  shape.
- `spec/block-manifest.schema.json`: id, version, ports, supported target range.
- `spec/wit/packsmith-block.wit`: the computed-block interface, written now and unused until
  Phase 3 so the IR is designed against it.
- `spec/CHANGELOG.md`, each schema carrying an explicit `version` field.

**Out of scope.** Conformance cases (task 4). Any Rust type mirroring a schema.

**Exit criterion.** All four spec artifacts exist, each with a `version` field, no category or
pack-kind expressed as a JSON Schema `enum`.

---

## 4. Initial conformance cases

**Status.** Done.

**Goal.** Give the contract teeth: golden `(graph, target) -> file tree` cases covering the
Phase 0 surface.

**Governing ADRs.** 0007 (deterministic builds), 0010 (open pack model), 0012 (raw mcfunction
escape hatch is validated), 0006 (target is a parameter).

**Scope.**
- `conformance/cases/`: `empty-pack`, `one-function`, `function-tag`, `recipe`, `loot-table`,
  `raw-mcfunction`, `legacy-syntax-rejected`.
- Each case: `input.json`, `target.json`, `README.md`, and either `expected/` or
  `expected-diagnostics.json`.
- `expected/` trees stubbed with `PLACEHOLDER.md` until verified in game (task 9).
- `conformance/README.md` describing the case format.

**Out of scope.** The runner that builds and diffs cases (task 7). Verifying the trees (task 9).

**Exit criterion.** Seven cases, each well-formed per `conformance/README.md`; the compile
success cases carry a placeholder, the failure case carries `expected-diagnostics.json`.

---

## 5. Compile-failure cases and the structural checker

**Status.** Done.

**Goal.** Let a conformance case assert a precise compile failure, and check every case is
well-formed without needing a working compiler.

**Governing ADRs.** 0012 (diagnostics carry a code, severity, and address), 0007
(deterministic builds).

**Scope.**
- `expected-diagnostics.json` format in `conformance/README.md`: asserted fields are `outcome`,
  per-diagnostic `code`, `severity`, `address`; `message` is never pinned; `code: null` is
  allowed for a not-yet-emitted code.
- `.claude/rules/spec.md`.
- `xtask` structural check: every case is a directory with `input.json`, `target.json`,
  `README.md`, and exactly one of `expected/` or `expected-diagnostics.json`. Runs in
  `cargo xtask ci` and as a unit test.

**Out of scope.** Actually building cases or checking diagnostics against a real compiler.

**Exit criterion.** `cargo xtask ci` fails on a malformed case; the diagnostics format is
documented in `conformance/README.md` and mirrored in `.claude/rules/spec.md`.

---

## 6. Target data pipeline

**Status.** Done.

**Goal.** Extract the thin functional subset of Minecraft 26.2 data and load it at runtime,
with no version fact anywhere in the compiler.

**Governing ADRs.** 0014 (extract from a pinned mcmeta commit, vendor as a derived artifact,
`--reports` fallback, never a submodule), 0015 (derived data is a separate work, SPDX-marked,
excluded from our grants), 0006 (target is a parameter), 0010 (open category and pack-kind
model).

**Scope.**
- `cargo xtask sync-target --version 26.2` (`--check` to verify only): fetch from the pinned
  `misode/mcmeta` commit, extract pack formats, the category-to-directory-and-extension table,
  the pruned command tree, and registry id lists.
- `crates/packsmith-mcversion/data/26.2.json` with its provenance header and
  `SPDX-License-Identifier: LicenseRef-Minecraft-Derived`.
- `packsmith-mcversion`: `TargetData`, runtime file loader, `LoadError` split so an unknown
  target becomes a diagnostic while a corrupt file is a hard error.
- `LICENSES/LicenseRef-Minecraft-Derived.txt`, `REUSE.toml`, `deny.toml` and `.gitattributes`
  updates, CI `reuse lint-file` scoped to the data directory.

**Out of scope.** Using the command tree for validation (task 12). A second target (task 14).

**Exit criterion.** `cargo xtask sync-target --version 26.2 --check` passes on CI with
byte-identical output; `26.2.json` carries the provenance header and SPDX marker;
`TargetData::load` reads it from a file, never `include_str!`.

---

## 7. Compiler / emitter / CLI vertical slice for the empty pack

**Status.** Done.

**Goal.** Take one conformance case end to end — graph in, deterministic `.zip` out — and
build the runner that proves it.

**Governing ADRs.** 0007 (deterministic builds), 0018 (STORE, not DEFLATE), 0006 (target is a
parameter), 0010 (pack kind and directory come from target data), 0017 (in-game verification
is the gate for a tree), 0012 (diagnostics carry node id and, where possible, a fix).

**Scope.**
- `packsmith-ir`: `Ir`, `Pack`, `Target`, `Text`, `Diagnostic`, `Severity`, `StatementAddress`.
- `packsmith-compiler`: `compile(graph, target_id) -> Compilation`; empty root lowers to one
  `data` pack with a description fallback.
- `packsmith-emit`: `file_tree` from IR and target data, `pack.mcmeta` shaped from the target's
  pack format, deterministic STORE `zip` with pinned timestamps and sorted entries.
- `packsmith-cli`: `packsmith build <project> --target <v> [--output <f>]`, diagnostic
  rendering, refuse to emit when any diagnostic is an error.
- `xtask` conformance runner: build each verified case twice, diff bytes for reproducibility
  (ADR-0007), compare the built tree against `expected/`, skip cases still on `PLACEHOLDER.md`,
  fail if every buildable case is a placeholder.
- `docs/adr/0017-in-game-verification.md`, `docs/adr/0018-store-only-zip.md`.

**Out of scope.** Functions, tags, recipes, loot tables (task 8). Command validation (task 12).

**Exit criterion.** `cargo xtask ci` builds `empty-pack` twice to identical bytes and matches
its verified `expected/` tree.

---

## 8. Lower functions, tags, recipes, and loot tables

**Status.** Done.

**Goal.** Turn the remaining Phase 0 case graphs into IR and emitted files through built-in
declarative blocks.

**Governing ADRs.** 0004 (declarative blocks first), 0003 (blocks return IR, not files), 0016
(ordered `body` slot is the command sequence), 0010 (category directory and extension from
target data), 0019 (emitted JSON shape is a version fact; until the validator lands, verify
in game and mark the shape with greppable constants).

**Scope.**
- `packsmith-blocks`: `packsmith/function`, `packsmith/command`, `packsmith/function-tag`,
  `packsmith/crafting-shapeless`, `packsmith/loot-table`; `lower_root`.
- `packsmith-ir`: resource and command-line IR nodes.
- `packsmith-compiler`: walk ordered slots, hand nodes to `packsmith-blocks`, collect
  lowering diagnostics (unknown block, missing required input).
- `packsmith-emit`: write each resource under its category directory with its extension.
- Named constants for the recipe type string and loot entry shape, commented to ADR-0019.
- `docs/adr/0019-json-shape-validation.md` (proposed), `.claude/rules/minecraft.md` update on
  block output shapes.

**Out of scope.** A systematic type / cross-reference / root-placement validation pass (see
`docs/BACKLOG.md`). The mcdoc schema validator (task 17). Command grammar (task 12).

**Exit criterion.** `one-function`, `function-tag`, `recipe`, `loot-table` lower to IR and emit
the expected files; hand-guessed JSON shapes sit behind constants pointing at ADR-0019.

---

## 9. In-game verification of the Phase 0 trees

**Status.** Done — the 🔒 in-game verification was carried out and confirmed by the maintainer.

**Goal.** Replace the placeholder expected trees with ones a real 26.2 client has accepted.

**Governing ADRs.** 0017 (the in-game pass is the gate, not a maintainer reading the JSON and
judging it plausible), 0019 (same gate for block output shapes until the validator lands),
0007 (the verified tree is what reproducibility is checked against).

**Scope.**
- Build `empty-pack`, `one-function`, `function-tag`, `recipe`, `loot-table` for 26.2, load
  each in game, confirm no compatibility warning and that functions run and recipes and loot
  behave.
- Write the confirmed file trees into each `expected/`, delete the `PLACEHOLDER.md` files.
- `.vscode/tasks.json` entries to build the test datapacks.

**Out of scope.** `raw-mcfunction` (waits on command grammar, task 12) and
`legacy-syntax-rejected` (a diagnostics case, no tree).

**Exit criterion. 🔒** Each of the five trees has been loaded in Java Edition 26.2 with no
compatibility warning and its behaviour confirmed in game; only then are the placeholders
removed. No test in this repo can stand in for this.

---

## 10. The graph validation pass

**Status.** Done.

**Goal.** Check a graph's shape before lowering — unknown blocks, missing or ill-typed inputs,
misplaced nodes, broken data edges — and give every failure a stable code an SDK or host can
assert on.

**Governing ADRs.** 0009 (the diagnostic names the game concept before the file path), 0012
(command and selector grammar is a separate stage, not this pass), 0016 (statement addresses;
ordered slots hold statements, not values).

**Scope.**
- `spec/diagnostics.md`: the diagnostic code namespace (`block-`, `input-`, `slot-`, `edge-`),
  the fields a diagnostic carries, which are asserted by conformance (`code`, `severity`,
  `address`) and which are recorded only (`message`, `fix`), anchoring rules, and the
  `command-` codes reserved for the Brigadier stage.
- `packsmith-ir::codes`: the Rust mirror of the codes the pass emits, kept in step with
  `spec/diagnostics.md`.
- `packsmith-ir`: `Diagnostic` carries an optional `code`, a `severity`, an `address`, and a
  recorded `message` and `fix`.
- `packsmith-compiler::validate`: one pass over the graph, collecting every diagnostic rather
  than bailing on the first; wired into `compile` so a graph with an error is not lowered and
  a diagnostic is reported from one place.
- `packsmith-blocks::describe`: the port and slot shape of each built-in, so the pass knows
  what a node requires.
- Seven diagnostics conformance cases — `unknown-block`, `missing-required-input`,
  `input-type-mismatch`, `id-missing-namespace`, `command-at-root`, `edge-to-missing-node`,
  `edge-forward-reference` — each with an `expected-diagnostics.json`, run Rust-side by
  `crates/packsmith-compiler/tests/conformance_diagnostics.rs`.

**Out of scope.** Command and selector grammar (task 12). Registry membership and block
property values — behind target data that may not exist for a given registry. The semantic
data-edge checks (edge source must be a value node, endpoint types must be assignable,
slot-scoped output visibility, a port holds a literal or an edge but not both), deferred until
a value block exists to exercise them (`docs/BACKLOG.md`).

**Exit criterion.** `cargo xtask ci` green; every diagnostics conformance case produces the
`code`, `severity`, and `address` its `expected-diagnostics.json` asserts; `spec/diagnostics.md`
and `packsmith-ir::codes` list the same codes.

---

## 11. Close the gaps the validation pass left open

**Status.** Done.

**Goal.** Close three gaps opened by the validation pass before the diagnostic set grows.

**Governing ADRs.** 0009 (the diagnostic names the game concept before the file path), 0016 (an
invalid ordering is unrepresentable in the editor). Also bound by `spec/diagnostics.md` and
`spec/block-manifest.schema.json`.

**Scope.**
- `packsmith-blocks::describe` returns `BlockDescriptor`, a second definition of what a block
  port is alongside `block-manifest.schema.json`. Add a test that serialises every built-in
  descriptor and validates it against that schema. If a built-in cannot be expressed in the
  format out-of-tree blocks will have to use, the format is wrong and we want to know now, not
  in Phase 3.
- Diagnostic messages are assembled from English fragments, with capitalisation carried by
  substitution. That does not translate, and the audience is a mostly non-English community
  with a total-beginner persona. Change `Diagnostic` to carry its code plus typed structured
  parameters instead of a rendered sentence; render messages from a template table keyed by
  code. Keep the English templates in the repo; change nothing else about the wording yet.
  Conformance cases may then assert on parameters as well as `code`, `severity`, and `address`.
- `slot-rejects-block` names `packsmith/command` in user-facing text. Give `BlockDescriptor` a
  display name and use it, so the diagnostic names the game concept first (ADR-0009).
- Reword `block-unknown` to "There's no block called ...", replace "a set of fields" with
  wording that is not jargon in disguise, and name the block in `input-missing` so the CLI
  message is locatable.
- Rewrite the `edge-forward-reference` and `edge-cycle` messages for a technical reader rather
  than a beginner: per ADR-0016 an invalid ordering is unrepresentable in the editor, so those
  conditions are only reachable by hand-editing or the CLI. A beginner who sees one has hit an
  editor bug.

**Out of scope.** Command grammar validation (task 12). Any actual translation. Any new
diagnostic code.

**Exit criterion.** `cargo xtask ci` green; no rendered sentence is stored on a `Diagnostic`.

---

## 12. Command grammar validation

**Status.** In progress.

**Goal.** Validate every command string against the target's extracted Brigadier tree, as a
compiler stage with real diagnostics — the last Phase 1 code work.

**Governing ADRs.** 0012 (commands are text plus provenance, validated against the extracted
grammar; validation is a stage, not a lint; retargeting is best-effort), 0006 (grammar is
target data, never in the compiler), 0009 (the diagnostic names the game concept before the
file path).

**Scope.**
- A validator in `packsmith-compiler` (or a crate it owns) that walks the pruned command tree
  from `TargetData` and checks each `command` IR node.
- Diagnostics with a code, severity, the source node address, and where possible a suggested
  fix; phrased in game terms first.
- Wire it into `compile` so `packsmith build` refuses on an invalid command.
- Make the `raw-mcfunction` escape hatch real: split a function-file string on newlines, skip
  blank and `#`-comment lines, validate the rest (resolve the open item in `docs/BACKLOG.md`
  about the `function` format row in `spec/types.md`).
- Verify `raw-mcfunction` and `legacy-syntax-rejected`: the first emits a file, the second
  produces the diagnostics its `expected-diagnostics.json` asserts.

**Out of scope.** A structured (AST) command form — the IR form is tagged so it can be added
later without a schema break. Rewriting commands across targets. JSON shape validation (task 17).

**Exit criterion.** `legacy-syntax-rejected` produces the asserted diagnostics and
`raw-mcfunction` builds, both under `cargo xtask ci`; pasting 1.16-era syntax into a raw block
yields a precise, game-worded error.
**🔒** The Phase 1 gate also requires a full generated pack to load in Java Edition 26.2 with
no compatibility warning and its functions to run — `/task` stops and reports here.

---

## 13. In-game verification harness (GameTest)

**Status.** Not started.

**Goal.** Prove the game accepts a generated pack, from `xtask`, using vanilla headless
GameTest — no mod loader.

**Governing ADRs.** 0017 (vanilla headless `net.minecraft.gametest.Main`; assertions are
`test_instance` assets with `function`-type environments, not log scraping; no Fabric, no
PackTest), 0014 (same jar-handling path as the `--reports` fallback; respects the EULA step
that target-data extraction avoids).

**Scope.**
- An `xtask` task that builds a conformance case, writes the resulting pack plus a generated
  test pack into a packs directory, and runs `net.minecraft.gametest.Main --packs <dir>`.
- The generated test pack: `test_instance` assets with `function`-type `test_environment`
  setup and teardown, one trivial structure per functional assertion.
- JDK and server-jar handling shared with the data-generator fallback path; EULA acceptance
  surfaced explicitly.
- A CI job separate from `cargo test` (needs a JDK and the jar).

**Out of scope.** Generating tests for a user's own pack (`docs/BACKLOG.md`, ADR-0017,
unscheduled). Any mod loader or version matrix.

**Exit criterion.** The harness builds a conformance case and runs the game against it, green,
in a dedicated CI job.
**🔒** First run needs the maintainer to accept the EULA and supply the 26.2 server jar;
`/task` stops and reports when it reaches that step.

---

## 14. Second and third targets

**Status.** Not started.

**Goal.** Extract two more releases and prove the same graph compiles for all three.

**Governing ADRs.** 0006 (each target is data; the version parser accepts both `1.x.y` and
year-based `NN.N` shapes), 0014 (each target is its own pinned mcmeta commit — one commit
cannot serve several), 0015 (each data file SPDX-marked and provenance-headed), 0010 (category
and pack-kind differences between targets are absorbed by data).

**Scope.**
- `cargo xtask sync-target` for two further releases, one of them old enough to need the
  legacy single-integer `pack_format` (task 16).
- `crates/packsmith-mcversion/data/<v>.json` for each, with headers and SPDX markers.
- Version parser handling both version-string shapes.
- A conformance case, or a runner mode, that builds one graph for all three targets.

**Out of scope.** Cross-block constraint resolution (task 15). Retargeting a command string
between versions (best-effort per ADR-0012, not automated here).

**Exit criterion.** One graph compiles for all three targets under `cargo xtask ci`, each
producing a pack valid for that target's format.
**🔒** Confirming each target's pack actually loads in that game version needs the maintainer;
the CI check only proves the bytes are well-formed.

---

## 15. Cross-block constraint resolution

**Status.** Not started.

**Goal.** Resolve target-compatibility constraints across all blocks in a graph and name the
block that breaks a build.

**Governing ADRs.** 0006 (target compatibility is expressed as ranges over target data), 0004
(built-in declarative blocks carry a supported target range in their manifest), 0012
(diagnostics carry an address and a suggested fix).

**Scope.**
- Read each block's supported target range from its manifest
  (`spec/block-manifest.schema.json`).
- Intersect the ranges against the resolved target; on an empty intersection, emit a
  diagnostic naming the offending block and its declared range.
- Property tests for the resolver (per `CLAUDE.md` — this is one of the two places hand-written
  cases miss the interesting failures).

**Out of scope.** Block-to-block *dependency* resolution and lockfiles — that is OPEN-QUESTION
C1, BLOCKING Phase 5. Do not pick an answer.

**Exit criterion.** A graph mixing a block that supports only an older target with a newer
build target fails with a diagnostic naming that block; the resolver has property-test
coverage.

---

## 16. Both `pack.mcmeta` shapes

**Status.** Not started.

**Goal.** Emit the legacy single-integer `pack_format` for old targets and the modern
`min_format` / `max_format` range from 1.21.9 on.

**Governing ADRs.** 0006 (which shape to emit is decided from target data, not a constant),
0010 (the emitter looks the pack kind up in target data), 0007 (byte-identical output).

**Scope.**
- `packsmith-emit`: choose the `pack.mcmeta` shape from the target's pack-format data.
- Legacy shape: bare-integer `pack_format`.
- Modern shape: `min_format` as an integer or `[major, minor]` pair, `max_format` as a bare
  major meaning any minor of that major.
- A conformance case per shape, tied to targets from task 14.

**Out of scope.** Resource-pack format numbers beyond what the data carries (resource packs
are post-v1, ADR-0010).

**Exit criterion.** An old target emits the legacy shape and a 1.21.9+ target emits the range
shape, each pinned by a conformance case under `cargo xtask ci`.
**🔒** Both shapes confirmed to load without warning in their respective game versions.

---

## 17. Emitted-JSON shape validation (mcdoc)

**Status.** Blocked on ADR-0019 (proposed; the task implements it only once accepted).

**Goal.** Stop trusting the block that produced a JSON object; check the object against the
target's data-pack schemas.

**Governing ADRs.** 0019 (validate against `SpyglassMC/vanilla-mcdoc`, extracted and vendored
like ADR-0014, loaded at runtime; a compiler stage checks each object against its category's
schema and reports an ADR-0012-style diagnostic; no new dependency to get started; this is a
Phase 2+ item and not a small one), 0006, 0014, 0012.

**Scope (when taken up).**
- Extend `xtask sync-target` to fetch and reduce mcdoc schemas from a pinned commit.
- An mcdoc evaluator in Rust: lexer, module and reference resolver, dispatch-on-sibling-field
  handling — or, if it survives translation, an mcdoc-to-JSON-Schema conversion in `xtask`
  validated with an existing crate.
- A compiler stage validating each emitted object; diagnostics with code, address, fix.
- Move the hardcoded recipe type string and loot entry shape out of `packsmith-blocks` (the
  constants from task 8) once the check covers them.

**Out of scope.** Everything until ADR-0019 moves from proposed to accepted. `/task` stops on
that contradiction rather than implementing a proposed ADR.

**Exit criterion.** A block that emits a recipe with a key renamed two releases ago is caught
by the validator against real schema data, with a diagnostic naming the node.
