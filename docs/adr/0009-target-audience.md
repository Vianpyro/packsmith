# ADR-0009: Target audience is absolute beginners and returning creators

- **Status:** accepted
- **Date:** 2026-08-27
- **Resolves:** OPEN-QUESTIONS A1

## Context

Two audiences were candidates: people who have never coded, and developers who want to move
faster than writing `.mcfunction` by hand. They want incompatible editors.

## Decision

We will design for two personas and no others:

- **The newcomer.** Has never written code and does not know what a namespace is. Needs
  strongly constrained, discoverable blocks and errors phrased in game terms, not file terms.
- **The returning creator.** Made data packs years ago, on 1.16 or similar, and has lost the
  thread through the format churn. Knows Minecraft deeply; knows current syntax not at all.

Developers are served by the CLI, not by a second editor mode.

## Consequences

The returning creator is the persona that most existing tools ignore, and serving them shapes
several features that would otherwise look optional:

- Blocks are discoverable by **intent** ("when a player sleeps"), not by registry name.
- Raw `.mcfunction` input is validated against the target's command grammar, so pasting 1.16
  syntax produces a precise "this argument moved in 1.21" error rather than a silent failure
  in game. This makes ADR-0012 load-bearing rather than a nicety.
- Diagnostics name the game concept before the file path.
- Documentation for each block states what changed since the older versions, where relevant.

The cost is that power-user affordances in the editor are permanently out of scope. Anything
that needs them belongs in the CLI.

## Alternatives considered

- Two editor modes over one compiler: doubles Phase 4, and in practice the beginner mode is
  the one that gets neglected.
