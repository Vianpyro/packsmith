# expected/ not yet verified

This directory is intentionally empty. The expected file tree must be produced by
building `input.json` against a real Minecraft: Java Edition 26.2 instance and
confirming, by hand, that the game loads the pack and that running the function
prints `Hello, world!`. Only then does this case count as passing (ROADMAP
Phase 0 exit criteria).

To verify:

- Build the case and record the exact tree, byte for byte, including `pack.mcmeta`.
- Confirm the `pack.mcmeta` format number(s) against the released 26.2
  `version.json`, not the pre-release datum in ADR-0014.
- Confirm the `function` category directory name and file extension for 26.2 from
  target data, not from memory (these have churned: registry directories became
  singular in 1.21).

Expected contents (shape only, to be filled in once verified): `pack.mcmeta`, and
one function file for `example:hello` containing the single line
`say Hello, world!` followed by a trailing newline if the verified output has one.
