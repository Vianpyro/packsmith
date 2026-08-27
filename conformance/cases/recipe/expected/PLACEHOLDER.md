# expected/ not yet verified

This directory is intentionally empty. The expected file tree must be produced by
building `input.json` against a real Minecraft: Java Edition 26.2 instance and
confirming, by hand, that the game loads the pack and that the recipe appears and
crafts as described. Only then does this case count as passing (ROADMAP Phase 0
exit criteria).

To verify:

- Build the case and record the exact tree, byte for byte, including `pack.mcmeta`.
- Confirm the `pack.mcmeta` format number(s) against the released 26.2
  `version.json`, not the pre-release datum in ADR-0014.
- Confirm the `recipe` category directory for 26.2 from target data.
- Confirm the exact recipe JSON shape 26.2 expects: the `type` value, the key
  names for ingredients and result, and whether the result count is written when
  it is greater than one. Do not carry recipe JSON shape forward from an older
  version from memory.

Expected contents (shape only, to be filled in once verified): `pack.mcmeta`, and
one recipe JSON for `example:diamond_from_block`.
