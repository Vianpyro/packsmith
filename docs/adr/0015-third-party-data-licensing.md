# ADR-0015: Minecraft-derived data is a separate work, excluded from our licence grants

- **Status:** accepted
- **Date:** 2026-08-27
- **Refines:** ADR-0011, ADR-0014

## Context

ADR-0014 puts data derived from Minecraft: Java Edition into an AGPL-3.0 repository. Two
distinct questions get conflated here, and only one of them is a real problem.

**Licence compatibility is not the problem.** The AGPL is a grant *we* make over *our* code.
It does not require every file in the repository to be AGPL.

**Overclaiming is the problem.** A bare `LICENSE` file at the root, read as "everything in
this repository is AGPL-3.0", asserts a grant we have no authority to make over content we do
not own. That is the actual defect, and it is fixed by scoping the claim, not by changing
licence.

A second axis is how much of Mojang's work we carry. Format numbers, directory names, and
registry identifiers like `minecraft:stone` are functional facts and names, thin on copyright
in most jurisdictions. Loot tables, advancements, worldgen configurations, language strings,
and textures are Mojang's creative work and are not thin at all.

## Decision

We will keep AGPL-3.0-or-later for the platform and MIT OR Apache-2.0 for the contract and
SDKs, unchanged, and add a third category that is licensed by neither.

1. **Scope the grant.** A root `NOTICE` file states that our licences cover Packsmith's own
   source, and that `crates/packsmith-mcversion/data/**` is derived from Minecraft: Java
   Edition, copyright Mojang AB, included for interoperability and not covered by our grants.
   The `LICENSE` file references `NOTICE`.
2. **Mark every derived file.** Each generated data file carries
   `SPDX-License-Identifier: LicenseRef-Minecraft-Derived` alongside its ADR-0014 provenance
   header. Follow the REUSE specification so the marking is machine-checkable.
3. **Keep the subset thin and functional.** Only what ADR-0014 lists as extracted. This is the
   same rule that keeps the repository small, which is convenient: the technically correct
   subset and the legally conservative subset are the same subset.
4. **Never vendor the vanilla data pack or any asset.** Fetch at runtime if ever needed.
5. **Load derived data at runtime, do not embed it.** No `include_str!` of derived data into a
   Packsmith binary. Keeping it a separate file the program reads keeps it an aggregate
   alongside our work rather than a component compiled into it. The browser fetches it per
   target anyway, so this costs nothing.
6. **Keep the trademark disclaimer** already required by `CLAUDE.md`.

## Consequences

No licence change, no relicensing, no dual-repository split. The repository states accurately
what it grants and what it merely redistributes, which is what a downstream packager or a
distribution maintainer needs in order to ship it.

This is a conservative engineering posture, not legal advice. The whole tooling ecosystem
redistributes generated Minecraft data (mcmeta, minecraft-data, MCDatapacker, the archived
vanilla-datapack), which is evidence of tolerated practice and nothing stronger. If Packsmith
ever takes money or reaches meaningful scale, get an actual opinion from an actual lawyer.

## Alternatives considered

- Relicensing the project to something permissive: solves nothing. The overclaiming problem is
  identical under MIT, and we would lose the hosted-fork protection ADR-0011 was chosen for.
- A separate repository for derived data: cleaner boundary, and it reintroduces the multi-repo
  coordination cost that ADR-0014 rejected. Revisit only if a packager objects.
- Fetching all target data at build time and vendoring nothing: strongest posture, and it
  breaks offline builds and reproducibility. Rejected on ADR-0007 grounds.
