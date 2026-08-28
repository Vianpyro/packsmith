# ADR-0018: The emitter packs with STORE, not DEFLATE

- **Status:** accepted
- **Date:** 2026-08-28

## Context

ADR-0007 requires that the same `(graph, target, resolved block versions)` produce a
byte-identical zip on any machine, at any time. A `.zip` is a compression container, and the
compressed bytes DEFLATE produces are not standardised: zlib, zlib-ng, and libdeflate all emit
different output for the same input, the output changes with the compression level, and it can
change between versions of one library. A build that shells out to whatever zlib is installed,
or links one and later upgrades it, would produce a different zip from the same inputs. That
is exactly the nondeterminism ADR-0007 exists to forbid, reintroduced one layer down.

## Decision

We will store every entry uncompressed (method `0`, STORE). The emitter writes the zip
structure by hand with pinned entry order and fixed timestamps (ADR-0007); STORE keeps the
entry payloads identical to the file tree, so the only bytes in the archive are ones we chose.
No compressor, no compression level, no library version reaches the output.

## Consequences

The build hash depends only on our own code and the inputs, and independent verification stays
a plain byte comparison. The conformance reader is a few lines because it never inflates.

The cost is real and paid by users: a data pack is shipped uncompressed. A pack with hundreds
of small functions and JSON files, which compress well, is meaningfully larger to upload and
share than a DEFLATE `.zip` of the same content. Minecraft itself loads either form, so there
is no runtime penalty, only a distribution one.

We revisit this if and only if a compressor exists whose output we can pin exactly: a vendored,
version-locked implementation (a pure-Rust DEFLATE at a fixed level, built from source in our
tree) whose bytes are covered by the conformance suite's twice-and-compare check. Reaching for
the system zlib, or any dependency we do not build ourselves, does not meet that bar.

## Alternatives considered

- DEFLATE via a well-known crate at a fixed level: still not reproducible across that crate's
  own versions, and a routine dependency bump would silently change every build hash.
- Compress outside the emitter, as a post-processing step: moves the nondeterminism, does not
  remove it, and breaks "the emitter's output is the artifact".
