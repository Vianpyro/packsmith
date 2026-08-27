# expected/ not yet verified

This directory is intentionally empty. The expected file tree must be produced by
building `input.json` against a real Minecraft: Java Edition 26.2 instance and
confirming, by hand, that the game loads the pack and runs `example:hello` on
world load. Only then does this case count as passing (ROADMAP Phase 0 exit
criteria).

To verify:

- Build the case and record the exact tree, byte for byte, including `pack.mcmeta`.
- Confirm the `pack.mcmeta` format number(s) against the released 26.2
  `version.json`, not the pre-release datum in ADR-0014.
- Confirm the tag directory for the `function` registry for 26.2 from target data
  (`tags/function` vs `tags/functions` — singular since 1.21) and confirm the
  built-in tag id `minecraft:load` still resolves to that path.
- Confirm the exact JSON serialisation (key order, whitespace, trailing newline)
  the deterministic emitter is expected to produce.

Expected contents (shape only, to be filled in once verified): `pack.mcmeta`, one
function file for `example:hello`, and one tag JSON for `minecraft:load` whose
`values` array holds `example:hello`.
