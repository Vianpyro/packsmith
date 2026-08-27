# ADR-0014: Target data is extracted from mcmeta and vendored as a derived artifact

- **Status:** accepted
- **Date:** 2026-08-27
- **Refines:** ADR-0006

## Context

ADR-0006 requires that every Minecraft version fact be generated data rather than something
written from memory, and named the server jar's data generator as the source
(`java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports`). That path works
but needs a jar download, a JDK in CI, and EULA acceptance, which is why the extractor was
scheduled for Phase 2.

`misode/mcmeta` already runs that generator for every release and publishes the results in a
version-controlled repository: the `summary` branch carries `commands/data.json` (the Brigadier
command tree), `registries/data.json`, `blocks/data.json`, and `versions/data.json`; the `data`
branch carries the vanilla data pack and a root `version.json` with the pack format numbers.

Brigadier itself (MIT, `com.mojang:brigadier`) is the command *engine* and contains no
Minecraft commands. It is not a source of grammar.

## Decision

We will source target data from mcmeta, extract a thin functional subset, and commit that
subset as a derived artifact. mcmeta is **not** a git submodule.

- `cargo xtask sync-target --version <v>` fetches from a pinned mcmeta commit, extracts, and
  writes `crates/packsmith-mcversion/data/<v>.json` with a provenance header: source URL,
  upstream commit SHA, mcmeta version `id`, extraction date, and the hash of the input.
- **Extracted:** pack format major and minor, the category-to-directory-and-extension table,
  the command tree pruned to what the validator uses, and registry id lists (blocks, items,
  entity types) for validation and completion.
- **Not extracted:** the vanilla data pack itself, assets, textures, sounds, language files,
  loot tables, worldgen configurations. If a feature ever needs those, it fetches them at
  runtime rather than vendoring them.
- CI re-runs the extractor and requires byte-identical output. That is the same integrity
  guarantee a pinned submodule would give, without its costs.
- Target data is **loaded at runtime** from a data file, never embedded with `include_str!`.
  See ADR-0015 for why this matters beyond ergonomics.
- The jar `--reports` path stays documented as the fallback, and `xtask` supports it.

## Consequences

The version pipeline becomes cheap enough to move into Phase 1, which in turn makes command
validation (ADR-0012) realistic immediately rather than a Phase 2 bet. No JDK in CI, no jar
download, offline builds.

The dependency is real and named: a repository maintained by one person. Automation gives us
correctness (what mcmeta publishes is what Mojang's generator produced) but not continuity
(the pipeline can stop for mundane reasons). Vendoring converts that from an outage into a
scheduling problem: we stay pinned to the last extracted version until someone fixes the
extractor. A `mcmeta-backup` repository and community forks exist, which is evidence that the
ecosystem already treats this bus factor as real.

## Why not a submodule

- A submodule points at one commit. ADR-0006 requires several targets to coexist in one build,
  and blocks declare compatibility ranges. That would mean one submodule per supported version,
  each on a different orphan branch. It does not scale.
- The compiler runs in the browser (ADR-0008). A git checkout is useless to the client; the
  data has to become a compact indexed artifact loaded per target regardless.
- The `data` and `assets` branches are large, and we need a few percent of them. Shallow and
  partial clones interact badly with submodules.
- A submodule makes the network a dependency of `cargo build`.

## Provisional datum

mcmeta at `26.2-pre-2` reports `data_pack_version: 107`, `data_pack_version_minor: 0`,
`resource_pack_version: 88`. **This is a pre-release and must not be committed as fact.**
Confirm against the released 26.2 `version.json` before the target table leaves stub status.

## Amendment 2026-08-27: pipeline implemented, 26.2 extracted

`cargo xtask sync-target --version <v>` is implemented in `xtask/src/sync_target.rs`. It
does a shallow blobless sparse `git` clone of a pinned mcmeta tag, verifies `HEAD` against a
commit SHA held in the task, extracts the subset below, and writes
`crates/packsmith-mcversion/data/<v>.json`. `packsmith-mcversion` loads that file at runtime
(`TargetData::load`); a request for a target with no file is `LoadError::NotFound`.

**26.2 pin.** Source `https://github.com/misode/mcmeta`, tag `26.2-summary`, commit
`711a353b47d84e6cb592a1b72f682e5f44759284` (mcmeta version id `26.2`, upstream date
2026-06-16). Extracted 2026-08-27.

**Released 26.2 pack formats.** `data` **107.1** (major 107, minor 1), `resource` **88.0**.
The pre-release datum above (107.0) is superseded and must not be carried forward.

**Sourcing notes for the extracted subset.**

- Pack formats and the `block` / `item` / `entity_type` id lists come straight from the
  summary branch `version.json` and `registries/data.json`.
- The command tree is the summary branch `commands/data.json` with every `permissions` key
  removed: it is command-level authorisation, not grammar. No other pruning yet; "pruned to
  what the validator uses" (above) waits on the validator existing (ADR-0012).
- The category-to-directory-and-extension table is the seven v1 categories (ADR-0010,
  OPEN-QUESTIONS A3). `recipe`, `loot_table`, and `advancement` are registry-backed and the
  extractor fails if mcmeta stops listing them. `function`, `tags/function`, `predicate`, and
  `item_modifier` are not datapack registries in any Minecraft-generated report — the game
  special-cases them (`ServerFunctionLibrary`, `LootDataType`) — so their directory and
  extension are held in the extractor's `V1_CATEGORIES` table. Likewise the pack-root names
  `data` and `assets` are structural constants, not generated data. When the jar `--reports`
  fallback lands (Phase 2) it will not change this: those directories are hardcoded upstream.
- `provenance.extracted` is a wall-clock date and is the only field `--check` ignores when it
  compares a re-extraction against the committed file.
