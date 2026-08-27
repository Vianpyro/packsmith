# Packsmith

Packsmith is a self-hosted, self-hostable low-code platform for building **Minecraft: Java
Edition** data packs. You wire *blocks* together in a visual graph; Packsmith compiles the
graph into a ready-to-install `.zip`. Blocks are shareable, versioned, and authorable in
several languages.

Primary target: **Java Edition 26.2**. The target version is a compile parameter, not a
constant, so the same graph can be built for several versions.

## Who it is for

Packsmith is designed for two people, and no others (ADR-0009):

- **The newcomer.** Has never written code and does not know what a namespace is. Needs
  constrained, discoverable blocks and errors phrased in game terms, not file terms.
- **The returning creator.** Made data packs years ago, on 1.16 or thereabouts, and lost the
  thread through the format churn. Knows Minecraft deeply; knows current syntax not at all.
  Pasting 1.16-era command syntax gets a precise "this argument moved" error, not a silent
  failure in game.

Developers are served by the command line, not by a second editor mode.

## Current status

**Phase 0 — specification.** Nothing is implemented yet. The output of this phase is the
contract: the JSON Schemas and WIT interface in `spec/`, and the golden cases in
`conformance/`. See `docs/ROADMAP.md` for the phases and their exit criteria, and
`docs/adr/` for the accepted architecture decisions.

## Building

Requires the Rust toolchain pinned in `rust-toolchain.toml`.

```sh
cargo xtask ci        # fmt check + clippy + tests + conformance
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
```

Once the compiler lands in Phase 1:

```sh
cargo run -p packsmith-cli -- build <project> --target 26.2
```

## Repository layout

| Path            | Contents                                                        |
| --------------- | --------------------------------------------------------------- |
| `spec/`         | Normative, language-agnostic, versioned JSON Schemas and WIT.     |
| `conformance/`  | Golden cases: `(graph, target)` -> expected file tree.            |
| `crates/`       | The compiler, the target data, the emitter, and the CLI.          |
| `sdk/`          | Block authoring SDKs (Phase 3+).                                  |
| `web/`          | The graph editor (Phase 4+).                                      |
| `xtask/`        | Repository automation (`cargo xtask ...`).                        |
| `docs/`         | ADRs, roadmap, glossary, licensing.                               |

## Licensing

Three zones. The boundary is deliberate; see `docs/LICENSING.md` and ADR-0011/ADR-0015.

- **`AGPL-3.0-or-later`** — the platform: `crates/`, `web/`, the registry service, `xtask/`.
  Text in `LICENSE`.
- **`MIT OR Apache-2.0`** — the contract and the tools: `spec/`, `conformance/`, `sdk/`.
  Nobody should have to think about licensing before writing their first block.
- **Neither** — `crates/packsmith-mcversion/data/**` is data derived from Minecraft: Java
  Edition. We redistribute it for interoperability but do not own it. It carries
  `LicenseRef-Minecraft-Derived` and is excluded from our grants in `NOTICE`.

Blocks written by users are licensed however their author chooses, including proprietary.
The SDK licence does not reach the blocks built with it.

---

NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG.
