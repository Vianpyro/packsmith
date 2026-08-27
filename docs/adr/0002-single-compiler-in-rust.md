# ADR-0002: One compiler, written in Rust

- **Status:** accepted
- **Date:** 2026-08-27

## Context

The project goal includes block authoring in Rust, Python, TypeScript, and Java. Read
naively, that implies four compilers. Four implementations of the same semantics diverge:
each divergence is a data pack that builds on one instance and not another, which destroys
the shareability that is the whole point of the platform.

## Decision

We will implement exactly one compiler, in Rust. Other languages appear only as **block
SDKs**: thin libraries that help an author produce a conforming block artifact. They never
reimplement graph validation, version resolution, IR lowering, or emission.

## Consequences

Semantics have one home. Maintenance surface is one quarter of the naive design. Rust gives
us a native CLI, a WASM build for the browser, and a WASM host in the same language.

The cost: contributors who only know Python or TypeScript cannot work on the core. Mitigation
is that the core is small and stable by design, while the block ecosystem, where most
community energy will go, is open to all four languages.

## Alternatives considered

- Four compilers behind a conformance suite: the suite would have to be exhaustive to be
  meaningful, which is more work than the compilers.
- One compiler in TypeScript: better browser story, worse CLI and WASM host story, and no
  path to a fast native build.
