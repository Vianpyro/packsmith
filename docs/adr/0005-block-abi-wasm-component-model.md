# ADR-0005: Block ABI is the WebAssembly Component Model, with an OCI escape hatch

- **Status:** accepted
- **Date:** 2026-08-27

## Context

Computed blocks are untrusted code downloaded from a registry and executed on a user's
self-hosted instance or in their browser. This is a remote-code-execution surface by design.
We also need one ABI reachable from Rust, TypeScript, Python, and Java.

## Decision

We will define the computed-block contract in WIT (`spec/wit/`) and distribute computed
blocks as WebAssembly components. The contract is a **pure function**:

```
describe() -> block-manifest
emit(inputs, build-context) -> result<ir-patch, diagnostics>
```

The host grants no filesystem, no network, no clock, and no ambient randomness. Randomness,
if needed, is seeded by the host from the build inputs. The host enforces a fuel limit, a
memory limit, a wall-clock timeout, and an output-size limit.

Toolchain maturity differs by language: Rust (`wasm32-wasip2`) is excellent, TypeScript
(ComponentizeJS) is good, Python (`componentize-py`) works with large artifacts, and Java
(TeaVM WASM-GC) is the weak link. For languages whose WASM path is not viable, we will
provide an **OCI runner**: the same JSON contract over stdin/stdout in a container with
`--network=none`, a read-only rootfs, a non-root user, and a seccomp profile. One semantic
contract, two transports.

## Consequences

Determinism and content-addressed caching fall out of the pure-function shape. The same block
artifact runs in the native CLI host (wasmtime) and in the browser host (the browser's own
engine via jco), which is what makes a browser-side compiler possible at all.

The cost is real: two hosts to keep in sync, plus per-language componentization pain. The
conformance suite is the mitigation.

## Alternatives considered

- Subprocess or container for all languages: universal and easy, but heavy to self-host,
  slower per block, and weaker isolation than a WASM sandbox.
- A custom bytecode or expression language: avoids the toolchain problem and reintroduces the
  "invent a language" problem, which is worse.
