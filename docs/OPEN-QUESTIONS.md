# Open questions

Unresolved decisions. Each carries a recommendation, but a recommendation is not an answer.

**If a task depends on a question still marked `OPEN`, stop and ask. Do not pick an answer.**

When a question is settled, mark it `RESOLVED`, record the answer, and if it is structural,
write the ADR.

---

## A. Product scope

### A1. Who is the primary user? `OPEN` — BLOCKING Phase 4

A total beginner who has never coded, or a developer who wants to move fast? The two produce
incompatible editors: heavily constrained puzzle-piece blocks with few types, versus a typed
node graph with rich connections. Serving both means two editors over one compiler, which is
possible but doubles Phase 4.

**Recommendation:** the total beginner, one editor, no second mode. That is the stated reason
the project exists, and "accessible to beginners" is the only positioning not already occupied
by existing tooling. Developers are served by the CLI.

### A2. Does Packsmith also produce resource packs? `OPEN` — BLOCKING Phase 0

Many modern data packs are useless without a paired resource pack (custom item models, sounds,
fonts). Shipping only data packs caps what users can build.

**Recommendation:** not in v1, but the IR must model **two pack outputs** from day one. The
retrofit cost later is high; the cost now is a field in a schema.

### A3. Which registries are in scope for v1? `OPEN` — BLOCKING Phase 0

**Recommendation:** `function`, tags, `recipe`, `loot_table`, `advancement`, `predicate`,
`item_modifier`. Explicitly out: worldgen, dimensions, biomes, structures, enchantments.
Worldgen is a separate product with its own complexity and its own existing tools.

### A4. Is there a raw-mcfunction escape hatch? `OPEN` — BLOCKING Phase 0

**Recommendation:** yes, a `raw` block in Phase 1. Every low-code platform without an escape
hatch gets abandoned at the first thing it cannot express. It should be validated against the
target's command syntax, and flagged in the UI as version-fragile.

### A5. Can Packsmith import an existing data pack? `OPEN`

**Recommendation:** no. Lossless round-tripping from files back to a graph is a decompiler,
which is a larger project than the compiler. Offer a one-way "wrap this pack as an asset"
path instead if the need proves real.

---

## B. Technical scope

### B1. Single-user or multi-tenant instances? `OPEN` — BLOCKING Phase 3

Single-user means the sandbox is a convenience. Multi-tenant means it is a security boundary
with a much higher bar, plus quotas, isolation, and abuse handling.

**Recommendation:** single-user is the supported deployment. Multi-tenant is possible but
unsupported, and documented as such.

### B2. Where does the compiler run? `OPEN` — BLOCKING Phase 0 (highest impact)

Server-side, or compiled to WASM and run in the user's browser?

**Recommendation: browser-side, with the CLI as the second host.** Consequences, which are
large and mostly good:

- No untrusted code executes on the server, so B1 mostly dissolves.
- Self-hosting becomes static files plus a small registry service. Near-zero operations,
  which is what makes "self-hostable" a real promise rather than a slogan.
- The user's project never leaves their machine unless they publish it.
- The cost: two hosts for computed blocks (wasmtime natively, jco in the browser). This is
  exactly what ADR-0005 buys with the component model, but it is still two things to keep
  in sync, and the browser host must enforce the same limits.

This one decides the shape of Phases 3, 4, and 5. Settle it before writing schemas.

### B3. How are projects persisted? `OPEN`

**Recommendation:** plain files in a project directory, git-friendly, no database. A database
appears only in Phase 5, for the registry, and only there.

### B4. What language are the code and docs written in? `ASSUMED: English`

The chat and planning happen in French; the repository is in English because the audience is
the international Minecraft community and the block SDKs target four language ecosystems.
Say so if you would rather have French, or French docs with English code.

### B5. Licence? `OPEN` — BLOCKING repository creation

**Recommendation:** a split. `AGPL-3.0` for the editor, the server, and the registry, so that
a hosted commercial fork has to give back. `MIT OR Apache-2.0` for `spec/`, `conformance/`,
and every SDK, so that nobody has to think about licensing before writing a block. A single
AGPL over the SDKs would suppress the ecosystem; a single MIT gives the platform away.

### B6. Name, GitHub org, domain? `OPEN` — BLOCKING repository creation

**Recommendation:** `Packsmith`. No collision found in the data pack tooling space, no Mojang
trademark. Alternatives: `Mortar`, `Tessera`. Crates are prefixed `packsmith-`, the binary is
`packsmith`.

---

## C. Blocks and registry

### C1. Can a block depend on another block? `OPEN` — BLOCKING Phase 5

**Recommendation:** yes, but with exact-version resolution and a committed lockfile. No
version ranges in block dependencies. Ranges are for target compatibility only.

### C2. Federated or centralised registry? `OPEN` — BLOCKING Phase 5

**Recommendation:** federated, with one official index configured by default. Blocks are
addressed by `namespace/name@version` plus a content hash, so an artifact fetched from any
mirror is verifiably the same artifact.

### C3. Are published blocks signed? Is there moderation? `OPEN`

**Recommendation:** signatures required for the official index, optional for self-hosted ones.
Moderation is a policy problem, not a technical one; do not build tooling for it before there
is a community to moderate.

### C4. Can a block emit arbitrary files? `RESOLVED: no`

Blocks return IR nodes only (ADR-0005). Arbitrary file output would defeat validation,
version retargeting, and the determinism guarantee.

---

## D. Security

### D1. What are the default sandbox limits? `OPEN` — BLOCKING Phase 3

Fuel, memory ceiling, wall-clock timeout, maximum output size, maximum number of emitted
files. Needs numbers, not adjectives.

**Recommendation:** start deliberately tight (64 MB, 2 s, 10 MB of output, 5 000 files),
make them configurable, and raise them only in response to a real block that needs more.

### D2. Does a self-hosted instance have authentication? `OPEN`

**Recommendation:** none by default in the single-user deployment, with a loud warning in the
docs against exposing it to the internet. Optional OIDC later if anyone asks.

---

## E. Process

### E1. How do we verify a generated pack actually works? `OPEN` — BLOCKING Phase 2

Conformance tests prove the compiler produces the tree we expect. They do not prove the game
accepts it.

**Recommendation:** a Docker-based harness running a headless server, loading the pack,
running `/reload` and a suite of test functions, and asserting on the output. Expensive to
build, and the only test that answers the actual question.

### E2. CI platform and gates? `OPEN`

**Recommendation:** GitHub Actions. Gates: fmt, clippy with warnings denied, tests,
conformance, `cargo deny`, and a reproducibility check that builds each conformance case
twice and compares hashes. No coverage percentage target; it optimises for the wrong thing.
