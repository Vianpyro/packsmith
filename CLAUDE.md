# Packsmith

Self-hosted, self-hostable low-code platform for building Minecraft: Java Edition data packs.
Users wire **blocks** together in a visual graph; Packsmith compiles the graph into a
ready-to-install `.zip`. Blocks are shareable, versioned, and authorable in several languages.

Primary target: **Java Edition 26.2**.

## Trademark constraint (hard rule)

Never use "Minecraft" or "Mojang" in package names, crate names, repository names, domain
names, or logos. Descriptive use in prose ("data packs for Minecraft: Java Edition") is fine.
The README and the web UI footer must carry:
`NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG.`

## Who this is for

Two personas, decided in ADR-0009. Every UX and diagnostic choice answers to them.

- **The newcomer.** Never written code. Does not know what a namespace is.
- **The returning creator.** Made data packs on 1.16 or thereabouts and lost the thread
  through the format churn. Knows Minecraft deeply, knows current syntax not at all.

Developers are served by the CLI, not by a second editor mode. Errors are phrased in game
terms first, file terms second.

## Licensing zones

`AGPL-3.0-or-later` for the platform (`crates/`, `web/`, registry, `xtask/`).
`MIT OR Apache-2.0` for the contract and the tools (`spec/`, `conformance/`, `sdk/`).
Moving code across that boundary is a deliberate, separate commit. See `docs/LICENSING.md`.

## Read these before writing code

- `docs/adr/` — accepted architecture decisions. **Do not contradict an accepted ADR.**
  If you think one is wrong, stop and write a new ADR that proposes superseding it.
- `docs/ROADMAP.md` — phases and exit criteria. Work only inside the current phase.
- `docs/GLOSSARY.md` — the project vocabulary is load-bearing. Use the exact terms.
- `docs/OPEN-QUESTIONS.md` — unresolved decisions. If a task depends on one of these,
  stop and ask instead of picking an answer.

**Current phase: Phase 0 (specification).** No implementation code yet. See ROADMAP.

## Invariants

These hold everywhere, in every phase. A change that breaks one of them is a bug, not a
trade-off.

1. **One compiler, written in Rust.** Other languages appear only as block SDKs. (ADR-0002)
2. **The graph and the IR are data, never code.** No embedded expressions, no eval, no
   language-specific constructs in a serialized graph. (ADR-0003)
3. **Builds are deterministic.** The same `(graph, target, resolved block versions)` must
   produce a byte-identical zip. No wall clock, no unseeded RNG, no environment reads, no
   iteration over `HashMap` when the order reaches the output, fixed zip entry timestamps,
   sorted entries. (ADR-0007)
4. **The Minecraft target is a compile parameter, never a constant.** No hardcoded format
   numbers, no hardcoded directory names, no `LATEST` sentinel in the compiler. (ADR-0006)
5. **Blocks are untrusted.** Every block runs sandboxed: no filesystem, no network, no clock,
   no ambient authority. It returns IR nodes, never files. (ADR-0005)
6. **Spec before implementation.** A schema and at least one conformance case exist before
   the code that satisfies them.
7. **The pack model is open.** Pack kinds and registry categories are target data, never
   a Rust `enum` and never a JSON Schema `enum`. Worldgen and resource packs are out of v1
   scope but must be reachable by adding data, not by redesigning. (ADR-0010)
8. **Semantics and layout are separate files.** Node positions never reach the build hash.
   (ADR-0013)

## Repository layout

```
spec/            Normative, language-agnostic, versioned. JSON Schemas + WIT.
conformance/     Golden cases: (graph, target) -> expected file tree. Language-agnostic.
crates/
  packsmith-ir/          IR types and (de)serialization. No I/O, no dependencies on other crates.
  packsmith-mcversion/   Version table, registry data, pack.mcmeta shaping.
  packsmith-compiler/    Graph -> IR. Validation, diagnostics, version resolution.
  packsmith-emit/        IR -> file tree -> deterministic zip.
  packsmith-blocks/      Built-in declarative blocks.
  packsmith-host/        Block runtime host (Phase 3+).
  packsmith-registry/    Block registry server and client (Phase 5+).
  packsmith-cli/         The `packsmith` binary.
sdk/{rust,ts,py,java}/   Block authoring SDKs (Phase 3+).
web/                     Editor (Phase 4+).
xtask/                   Repo automation (`cargo xtask ...`).
docs/, .claude/
```

Dependency direction is strictly downward: `ir` <- `mcversion` <- `compiler` <- `emit` <- `cli`.
A crate never imports from a crate to its right in that chain.

## Commands

```
cargo xtask ci        # fmt check + clippy + tests + conformance. Run before saying "done".
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
cargo run -p packsmith-cli -- build <project> --target 26.2
```

If a command above does not exist yet, that is a Phase 1 task, not a licence to skip it.

## Rust conventions

- Edition 2024. MSRV pinned in `rust-toolchain.toml`; do not raise it silently.
- `#![forbid(unsafe_code)]` in every crate.
- No `unwrap()` / `expect()` / `panic!()` outside tests, `build.rs`, and `main`.
- Libraries return typed errors (`thiserror`). Only `packsmith-cli` uses `anyhow`.
- Every user-facing error carries the source node id and, where possible, a suggested fix.
  A diagnostic that says only "invalid input" is unfinished work.
- No `async` in `ir`, `mcversion`, `compiler`, or `emit`. Those are pure and synchronous.
- Composition over inheritance-style trait towers. Do not introduce a trait until there are
  at least two real implementations, and do not introduce a generic parameter until there
  are two real instantiations.
- Comments document intent, invariants, assumptions, and non-obvious behaviour. Never restate
  the code. No section-banner comments, no `// TODO` without an issue number.
- ASCII in code, filenames, and identifiers. Unicode only where the data itself requires it
  (e.g. pack descriptions, translations).

## Testing

- Unit tests next to the code. Integration tests in `tests/`.
- Snapshot tests (`insta`) for emitted file trees.
- Property tests (`proptest`) for the version resolver and the graph validator: these are the
  two places where hand-written cases will miss the interesting failures.
- The conformance suite in `conformance/` is the contract. Every SDK and every host must pass it.

## Things not to do

- Do not modify `spec/` or `conformance/cases/*/expected/` in the same commit as an
  implementation change. If a golden file needs to change, that is its own commit with a
  written justification. Never "fix" a failing test by editing the expected output.
- Do not invent Minecraft version data from memory. Format numbers, registry contents, and
  directory names are derived from the official `version.json` and the vanilla data pack,
  extracted by tooling, and committed as generated data. See `.claude/rules/minecraft.md`.
- Do not add a database, a web framework, an auth system, or a job queue before Phase 5.
- Do not hand-roll a zip writer without pinning timestamps and entry order.
- Do not create types named `*Manager`, `*Helper`, `*Provider`, `*Service`, `Abstract*`, or
  `*Impl`. Name things after what they are.
- Do not add a dependency without stating in the commit message what it replaces and why
  the std or an existing dependency is not enough.
- Do not widen scope. If the task reveals adjacent work, write it in `docs/BACKLOG.md` and
  keep going.

## Definition of done

`cargo xtask ci` passes; the change is covered by a test that fails without it; public items
have doc comments; if the architecture moved, an ADR was added or amended; the diff contains
nothing unrelated to the task.

## Commits

Conventional Commits, scoped to a crate: `feat(compiler): resolve block version constraints`.
One logical change per commit.
