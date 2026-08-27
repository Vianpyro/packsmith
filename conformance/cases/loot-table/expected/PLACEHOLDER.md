# expected/ not yet verified

This directory is intentionally empty. The expected file tree must be produced by
building `input.json` against a real Minecraft: Java Edition 26.2 instance and
confirming, by hand, that the game loads the pack and that the loot table yields
one diamond when rolled. Only then does this case count as passing (ROADMAP
Phase 0 exit criteria).

To verify:

- Build the case and record the exact tree, byte for byte, including `pack.mcmeta`.
- Confirm the `pack.mcmeta` format number(s) against the released 26.2
  `version.json`, not the pre-release datum in ADR-0014.
- Confirm the `loot_table` category directory for 26.2 from target data
  (`loot_table` vs `loot_tables` — singular since 1.21).
- Confirm the loot table JSON shape 26.2 expects: pool/entry/`type` key names and
  whether a `type` on the table itself is required.

Expected contents (shape only, to be filled in once verified): `pack.mcmeta`, and
one loot table JSON for `example:blocks/sparkstone`.
