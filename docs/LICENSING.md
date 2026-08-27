# Licensing

Decided in ADR-0011. Two zones, one boundary.

## Zone 1 — the platform: `AGPL-3.0-or-later`

`crates/` (except SDK-facing crates), `web/`, the registry service, `xtask/`.

Rationale: if someone runs a modified Packsmith as a hosted service, the modifications come
back. This is the only licence that covers the network case, and a hosted fork is the realistic
way this project gets taken.

## Zone 2 — the contract and the tools: `MIT OR Apache-2.0`

`spec/`, `conformance/`, `sdk/rust`, `sdk/ts`, `sdk/py`, `sdk/java`.

Rationale: nobody should have to think about licensing before writing their first block. The
dual MIT/Apache form is the Rust ecosystem default and removes the patent-grant question for
corporate contributors.

## Blocks written by users

Whatever their author chooses, including proprietary. The SDK licence does not reach the
blocks built with it. Block manifests carry an SPDX `license` field; the registry displays it
and does not enforce a policy.

## Practical requirements

- A `LICENSE` file at the repository root (AGPL-3.0) and one in each Zone 2 directory.
- An SPDX identifier in every `Cargo.toml`, `package.json`, `pyproject.toml`, and `pom.xml`.
- `cargo deny` configured to reject dependencies incompatible with the zone that uses them.
- Fetch the canonical licence texts from `https://www.gnu.org/licenses/agpl-3.0.txt` and
  `https://www.apache.org/licenses/LICENSE-2.0.txt`. Do not reproduce a licence text from
  memory: a paraphrased licence is not a licence.

## Boundary maintenance

Moving code between zones is a deliberate act, never a copy-paste. If a Zone 1 crate grows
something an SDK needs, extract it into a new Zone 2 crate in its own commit, with the
relicensing noted in the message.

---

## Zone 3 — Minecraft-derived data: covered by neither

`crates/packsmith-mcversion/data/**`. Decided in ADR-0015.

This is data extracted from Minecraft: Java Edition (via `misode/mcmeta`, itself produced by
Mojang's own data generator). We do not own it and cannot sublicense it. The repository's
`NOTICE` file says so explicitly, and every file in that directory carries
`SPDX-License-Identifier: LicenseRef-Minecraft-Derived` plus its provenance header.

Two rules keep this contained:

- **Thin and functional only.** Format numbers, directory and extension tables, registry id
  lists, pruned command grammar. Never loot tables, advancements, worldgen configurations,
  language strings, or assets. If a feature needs those, it fetches them at runtime.
- **Loaded, never embedded.** No `include_str!` of derived data into a Packsmith binary. It
  stays a separate file the program reads, which keeps it an aggregate alongside our work
  rather than a component compiled into it.

Follow the REUSE specification (`https://reuse.software`) so per-file licensing is
machine-checkable, and wire `reuse lint` into CI next to `cargo deny`.

This is a conservative engineering posture, not legal advice.
