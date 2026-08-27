---
paths:
  - 'crates/packsmith-mcversion/**'
  - 'crates/packsmith-emit/**'
  - 'xtask/**'
---

# Minecraft version rules

**Never write a Minecraft version fact from memory.** Format numbers, registry contents,
directory names, and command syntax change every release, and the model's training data is
stale by construction. All of it is generated data, extracted by `xtask`, and committed under
`crates/packsmith-mcversion/data/` with a provenance header. See ADR-0006 and ADR-0014.

## Sourcing

Primary source: `misode/mcmeta`, pinned to a specific commit.

- `summary` branch: `commands/data.json` (Brigadier command tree), `registries/data.json`,
  `blocks/data.json`, `versions/data.json`.
- `data` branch: root `version.json` with `data_pack_version` and `data_pack_version_minor`.

Fallback, documented and supported by `xtask`: the server jar's own data generator,
`java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports`.

mcmeta is **not** a git submodule and must not become one. See ADR-0014 for why.

Brigadier (`com.mojang:brigadier`, MIT) is the command *engine*. It contains no Minecraft
commands and is not a source of grammar. Do not reach for it expecting one.

## What to extract, and what never to extract

Extract: pack format major and minor, the category-to-directory-and-extension table, the
command tree pruned to what the validator uses, registry id lists (blocks, items, entity
types).

Never vendor: the vanilla data pack, assets, textures, sounds, language files, loot tables,
worldgen configurations. This is both a repository-size rule and a licensing rule (ADR-0015).

Every generated file carries `SPDX-License-Identifier: LicenseRef-Minecraft-Derived` and a
provenance header: source URL, upstream commit SHA, mcmeta version id, extraction date, input
hash. CI re-runs the extractor and requires byte-identical output.

Load target data at runtime from a file. Never `include_str!` derived data into a binary.

## Facts that shape the code

These are here because they determine control flow, not because they replace extraction.

- Since 25w31a / 1.21.9, `pack.mcmeta` uses `min_format` and `max_format`, and they are
  required. `pack_format` became optional and is only needed to also support data pack format
  below 82 (resource pack format below 65).
- `min_format` may be an integer or a `[major, minor]` pair. `max_format` given as a bare
  integer means "any minor version of that major".
- The emitter must therefore produce two shapes of `pack.mcmeta`: the legacy single-integer
  shape for older targets, the range shape from 1.21.9 on.
- Data pack and resource pack format numbers are independent and differ for the same game
  version. Never reuse one for the other.
- Registry directory names became singular in 1.21 (`function`, not `functions`). The
  directory name is per-target data, not a constant.
- Minecraft moved to year-based versions after 1.21.x. `26.2` is a release, not `1.26.2`. The
  version parser must accept both shapes.

## The 26.2 number

mcmeta at `26.2-pre-2` reports `data_pack_version: 107`, `data_pack_version_minor: 0`,
`resource_pack_version: 88`. **That is a pre-release. Do not commit it as fact.** Run the
extractor against released 26.2 and use what it returns. Until then the target table is a stub
and the compiler fails loudly rather than assuming.
