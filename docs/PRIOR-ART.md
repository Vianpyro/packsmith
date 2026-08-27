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

## Where Packsmith is different

Not "visual" and not "self-hosted" — both exist. The claim is **shareable, versioned,
multi-language blocks**: a package ecosystem for data pack logic, aimed at people who do not
write code. If a design choice does not serve that claim, it is not a priority.
