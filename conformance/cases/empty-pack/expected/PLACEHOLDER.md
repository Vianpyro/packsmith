# expected/ not yet verified

This directory is intentionally empty. The expected file tree must be produced by
building `input.json` against a real Minecraft: Java Edition 26.2 instance and
confirming, by hand, that the game loads the result with no compatibility
warning. Only then does this case count as passing (ROADMAP Phase 0 exit
criteria).

To verify:

- Build the case and record the exact tree, byte for byte, including `pack.mcmeta`.
- Confirm the `pack.mcmeta` format number(s) against the released 26.2
  `version.json`. Do **not** use the pre-release `data_pack_version: 107` from
  ADR-0014 — that is a provisional datum from `26.2-pre-2` and must not be
  committed as fact.
- Confirm the pack root directory name from target data, not from memory.

Expected contents (shape only, to be filled in once verified): `pack.mcmeta`
alone, with the project description and the 26.2 data pack format.
