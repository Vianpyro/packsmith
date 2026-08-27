---
paths:
  - 'crates/packsmith-mcversion/**'
  - 'crates/packsmith-emit/**'
---

# Minecraft version rules

**Never write a Minecraft version fact from memory.** Format numbers, registry contents,
directory names, and command syntax change every release and the model's training data is
stale by construction. All of it is generated data, extracted by tooling from official
sources, and committed under `crates/packsmith-mcversion/data/` with a provenance header
recording the source, the version, and the extraction date.

Known facts, kept here because they shape the code, not because they replace extraction:

- Since 25w31a / 1.21.9, `pack.mcmeta` uses `min_format` and `max_format`, and they are
  required. `pack_format` became optional and is only needed to also support data pack
  format below 82 (resource pack format below 65).
- `min_format` may be an integer or a `[major, minor]` pair. `max_format` given as a bare
  integer means "any minor version of that major".
- The emitter must therefore produce two different shapes of `pack.mcmeta` depending on the
  target: the legacy single-integer shape for older targets, the range shape from 1.21.9 on.
- Data pack and resource pack format numbers are independent and differ for the same game
  version. Do not reuse one for the other.
- Registry directory names became singular in 1.21 (`function`, not `functions`). The
  directory name is per-target data, not a constant.
- Minecraft moved to year-based versions after 1.21.x. `26.2` is a release, not `1.26.2`.
  The version parser must accept both shapes.

The exact data pack format for 26.2 is **not** recorded in this repo yet and must not be
guessed. Extract it from `version.json` inside the official jar. That extractor is a Phase 2
deliverable; until it exists, the target table is a stub and the compiler must fail loudly
rather than assume.
