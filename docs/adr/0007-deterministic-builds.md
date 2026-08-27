# ADR-0007: Builds are deterministic and reproducible

- **Status:** accepted
- **Date:** 2026-08-27

## Context

Blocks are third-party artifacts that will be executed repeatedly. Caching them is a
performance requirement. Verifying that a published pack matches its source graph is a trust
requirement. Both need bit-for-bit reproducibility.

## Decision

We will guarantee that the same `(graph, target, resolved block versions)` produces a
byte-identical zip on any machine, at any time. Concretely: no wall-clock reads, no
environment reads, no unseeded randomness, no host locale or path leaking into output, sorted
directory entries, `BTreeMap` / `BTreeSet` or explicit sorting wherever ordering reaches the
output, and fixed zip entry timestamps.

The build cache is content-addressed on `hash(block artifact) + hash(inputs) + hash(target)`.

## Consequences

Caching, incremental builds, and independent verification all become straightforward. Any
nondeterminism is a test failure, not a mystery: the conformance suite runs each case twice
and compares hashes.

The cost is that a few conveniences (embedding a build timestamp, a generator version string)
are forbidden in the output unless supplied explicitly as a graph input.

## Alternatives considered

- Best-effort determinism: in practice means no determinism, because the one nondeterministic
  path is the one that matters.
