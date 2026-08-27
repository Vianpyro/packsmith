# ADR-0004: Declarative blocks first, computed blocks later

- **Status:** accepted
- **Date:** 2026-08-27

## Context

A "block" can mean two very different things: a parameterized template that expands to IR,
or arbitrary code that computes IR. The second implies a runtime, a sandbox, a per-language
toolchain, and a distribution story. The first implies none of those.

## Decision

We will define two tiers with a shared manifest format:

- **Declarative block** — a manifest plus a template. No code, no runtime, no sandbox.
  Anyone can write and publish one. This is the default and the majority case.
- **Computed block** — real code, run in a sandbox, for logic that templates cannot express
  (for example, generating a binary-search fan-out of functions for a raycast).

Phases 1 and 2 ship declarative blocks only. Computed blocks arrive in Phase 3.

## Consequences

The riskiest and most expensive subsystem (multi-language sandboxed execution) is deferred
until the compiler, the IR, and the version model have been validated by real use. If it
turns out that most useful blocks are declarative, the multi-language question shrinks from
a load-bearing requirement to a nice-to-have.

The cost is that some early blocks will be built in rather than community-authored.

## Alternatives considered

- Computed blocks only, from day one: makes the simplest possible block require a toolchain,
  a compile step, and a sandbox. Kills community contribution at the low end.
