# Glossary

These terms are used with exactly these meanings, in code, docs, and UI. If you need a
concept that is not here, add it here first.

**Project** — the unit a user works on. A directory containing a graph, assets, and a
lockfile. Plain files, git-friendly.

**Graph** — the serialized, schema-validated description of what the user wired together.
Data only, never code. Nodes and edges.

**Node** — one instance of a block inside a graph, with its own id, input values, and edges.

**Port** — a typed connection point on a node. Inputs and outputs.

**Block** — a reusable, versioned unit of pack-generating behaviour. Identified as
`namespace/name@version`. Declares a supported target range.

**Declarative block** — a block that is a manifest plus a template. No code, no runtime.

**Computed block** — a block that is code, executed in a sandbox, returning an IR patch.

**Block manifest** — a block's self-description: id, version, ports, target range, metadata.
Both block tiers have one; it is the same schema.

**IR** — the intermediate representation. A normalized, target-resolved description of pack
contents, one level above the file tree. Produced by the compiler, consumed by the emitter.

**IR patch** — what a computed block returns: a set of IR nodes to merge, never files.

**Emitter** — the stage that turns IR into an actual file tree and then a zip. The only
component that knows about directory names and file formats.

**Target** — a specific Minecraft: Java Edition release the build is aimed at, for example
`26.2`. Always explicit.

**Pack format** — the number Minecraft uses in `pack.mcmeta` to decide compatibility. Data
pack and resource pack formats are different numbers for the same release.

**Host** — the component that executes computed blocks under a sandbox. There are two: the
native host (wasmtime) and the browser host.

**Registry** — where blocks are published and fetched. Federated; an instance can run its own.

**Conformance case** — a directory in `conformance/cases/` pinning one behaviour:
`input.json`, `target.json`, `expected/`, `README.md`.

**Diagnostic** — a structured, user-facing message with a code, a severity, a node id, and
where possible a suggested fix. Never a bare string.
