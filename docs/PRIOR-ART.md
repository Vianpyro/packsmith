# Prior art

Read this before proposing a feature. Most of the obvious ideas already exist somewhere, and
the ones that exist are usually better than a rushed reimplementation.

## Code-first data pack generators

- **beet** / **mecha** / **bolt** (Python). The most mature toolkit in the space. Study its
  pipeline and plugin model closely; a lot of the hard problems are already solved there.
- **Sandstone** (TypeScript). Typed DSL, strong autocomplete story.
- **Kore** (Kotlin DSL). Relevant because the Minecraft community is JVM-literate.
- **mcpack** (Rust CLI). Scaffolding and zipping, narrow scope.
- The predecessor projects, both archived: `Vianpyro/minecraft_with_python` (published as
  `mcwpy`) and `Vianpyro/minecraft_datapacks_generator`. Useful as a record of what broke:
  hardcoded game versions and a pack expressed as a Python object graph.

## Editors and generators

- **misode.github.io**. Schema-driven form generators for most registries, tracking current
  versions. The nearest thing to a reference for "what fields exist in this version".
- **MCDatapacker**, **JoachimCoenen/Datapack-Editor**. Desktop editors, file-oriented.
- **Scythe**. A purpose-built data pack IDE with visual editors and live preview.
- **MCreator**. No-code for mods rather than data packs, with a block-based procedure editor.
  The closest existing thing to this project's UX goal, in an adjacent domain.
- **Spyglass** (SpyglassMC, TypeScript). Language server and static analyzer for data pack
  files: parsing, validation, completion against a given version. The nearest thing to a
  reference implementation of the ADR-0012 structural validator, though the wrong language for
  our core. Static only: it cannot tell you the game loaded the pack (ADR-0017).

## In-game testing

- **Vanilla GameTest framework**. Since snapshot 25w03a (1.21.5), reachable from data packs in
  vanilla with no mod. Tests are `test_instance` registry assets; `test_environment` assets
  group them and supply `function`-type setup/teardown. The server jar exposes a headless entry
  point, `net.minecraft.gametest.Main`. Packsmith's Phase 2 verification harness runs on this
  (ADR-0017).
- **PackTest** (misode, Fabric mod). Predates the vanilla feature. Tests written as plain
  mcfunction, `-Dpacktest.auto` for CI, GitHub annotations on failure. Nicer to author than
  vanilla GameTest; rejected as a dependency because a mod loader sits between the harness and
  the runtime we claim to verify. Worth revisiting if authoring vanilla test instances proves
  painful (ADR-0017).

## Where Packsmith is different

Not "visual" and not "self-hosted" — both exist. The claim is **shareable, versioned,
multi-language blocks**: a package ecosystem for data pack logic, aimed at people who do not
write code. If a design choice does not serve that claim, it is not a priority.

## Data sources

- **misode/mcmeta**. Processed, version-controlled history of Minecraft's generated data and
  assets, produced by running Mojang's own data generator on each release. Our primary target
  data source (ADR-0014). `summary` branch for the command tree, registries, and blocks;
  `data` branch for the vanilla pack and the root `version.json`.
- **Mojang/brigadier** (MIT). The command parser and dispatcher itself. Useful to read for how
  the command tree is structured; contains no Minecraft commands and is not a grammar source.
- **PrismarineJS/minecraft-data**, **Arcensoth/mcdata**, the archived
  **SPGoding/vanilla-datapack**. Alternative or historical extractions. Relevant mostly as
  evidence of how the ecosystem handles redistribution of derived game data.
