# Open questions

Unresolved decisions. Each carries a recommendation, but a recommendation is not an answer.

**If a task depends on a question still marked `OPEN`, stop and ask. Do not pick an answer.**

When a question is settled, mark it `RESOLVED`, record the answer, and if it is structural,
write the ADR.

---

## Resolved

| # | Question | Answer | ADR |
|---|---|---|---|
| A1 | Primary user | Newcomers and returning creators. One editor. Developers use the CLI. | 0009 |
| A2 | Resource packs | Not in v1. The IR models multiple pack outputs from schema v1. | 0010 |
| A3 | v1 registries | `function`, tags, `recipe`, `loot_table`, `advancement`, `predicate`, `item_modifier`. Worldgen later, by adding target data only. | 0010 |
| A4 | Raw mcfunction escape hatch | Yes, from Phase 1, validated against the target's command grammar. | 0012 |
| B1 | Multi-tenant | No. Single-user deployment; nothing untrusted runs server-side. | 0008 |
| B2 | Where the compiler runs | Client-side WASM. The server is a registry only. | 0008 |
| B4 | Language of code and docs | English. | - |
| B5 | Licence | AGPL-3.0-or-later for the platform, MIT OR Apache-2.0 for `spec/`, `conformance/`, `sdk/`. | 0011 |
| C4 | Can a block emit arbitrary files | No. IR nodes only. | 0005 |
| F1 | Where does target data come from | Extracted from a pinned `misode/mcmeta` commit by `xtask`, vendored as a derived artifact. Jar `--reports` as fallback. | 0014 |
| F2 | mcmeta as a git submodule | No. One commit cannot serve several targets, the browser cannot use a checkout, and it makes the network a build dependency. | 0014 |
| F3 | Minecraft-derived data in an AGPL repo | Keep AGPL. Scope the grant in `NOTICE`, mark files `LicenseRef-Minecraft-Derived`, keep the subset thin and functional, load rather than embed. | 0015 |

---

## Still open

### A5. Can Packsmith import an existing data pack? `OPEN`

**Recommendation:** no. Lossless round-tripping from files back to a graph is a decompiler,
which is a larger project than the compiler. Offer a one-way "wrap this pack as an asset"
path instead if the need proves real.

ADR-0009 makes this tempting: a returning creator with an old 1.16 pack is exactly the person
who would want to import it. Resist until the compiler is proven. A half-working importer
damages trust more than a missing one.

### B3. How are projects persisted in the browser? `OPEN` - BLOCKING Phase 4

ADR-0013 fixes the file layout inside a project. This question is only where projects live in
the browser deployment: File System Access API, explicit download and upload, or an optional
sync service.

**Recommendation:** File System Access API where available, with download and upload as the
fallback. No sync service. Projects stay plain files, git-friendly, on the user's disk.

### B6. Name, GitHub org, domain? `ASSUMED: Packsmith` - BLOCKING repository creation

No collision found in the data pack tooling space, no Mojang trademark. Crates prefixed
`packsmith-`, binary `packsmith`. Alternatives if you would rather change now than later:
`Mortar`, `Tessera`. Renaming after Phase 1 is cheap; after Phase 5 it is not.

### C1. Can a block depend on another block? `OPEN` - BLOCKING Phase 5

**Recommendation:** yes, with exact-version resolution and a committed lockfile. No version
ranges in block dependencies. Ranges are for target compatibility only.

### C2. Federated or centralised registry? `OPEN` - BLOCKING Phase 5

**Recommendation:** federated, with one official index configured by default. Blocks are
addressed by `namespace/name@version` plus a content hash, so an artifact fetched from any
mirror is verifiably the same artifact.

### C3. Are published blocks signed? Is there moderation? `OPEN`

**Recommendation:** signatures required for the official index, optional for self-hosted ones.
Moderation is a policy problem, not a technical one. Do not build tooling for it before there
is a community to moderate.

### D1. What are the default sandbox limits? `OPEN` - BLOCKING Phase 3

Fuel, memory ceiling, wall-clock timeout, maximum output size, maximum number of emitted
files. Needs numbers, not adjectives. ADR-0008 adds a constraint: the limits must be
survivable inside a browser tab, not just on a server.

**Recommendation:** start deliberately tight (64 MB, 2 s, 10 MB of output, 5000 files), make
them configurable, and raise them only in response to a real block that needs more.

### D2. Does a self-hosted instance have authentication? `OPEN`

Largely defused by ADR-0008: the server holds no projects and runs no user code. What remains
is publishing to a self-hosted registry.

**Recommendation:** token-based publishing, no user accounts, no login for reading.

### F4. Do we need a real legal opinion, and when? `OPEN`

ADR-0015 is a conservative engineering posture, not legal advice. The whole tooling ecosystem
redistributes generated Minecraft data, which is evidence of tolerated practice and nothing
stronger.

**Recommendation:** no opinion needed while this is an unfunded hobby project distributing
only functional data. Get one before any of: taking money, accepting sponsorship tied to the
project, distributing through a channel with its own IP review (a Linux distribution, an app
store), or vendoring anything beyond the thin functional subset.

### E1. How do we verify a generated pack actually works? `OPEN` - BLOCKING Phase 2

Conformance tests prove the compiler produces the tree we expect. They do not prove the game
accepts it.

**Recommendation:** a Docker-based harness running a headless server, loading the pack,
running `/reload` and a suite of test functions, and asserting on the output. Expensive to
build, and the only test that answers the actual question. It is also the natural place to
extract the command grammar that ADR-0012 depends on.

### E2. CI platform and gates? `OPEN`

**Recommendation:** GitHub Actions. Gates: fmt, clippy with warnings denied, tests,
conformance, `cargo deny`, and a reproducibility check that builds each conformance case twice
and compares hashes. No coverage percentage target; it optimises for the wrong thing.
