# ADR-0012: Commands are strings, validated against the target's command grammar

- **Status:** accepted
- **Date:** 2026-08-27

## Context

A function body is a list of commands. Two options: model commands as a typed AST, or keep
them as text. A full AST for the Minecraft command grammar is an enormous, permanently moving
target. Plain unchecked text, on the other hand, means errors surface in game rather than in
the editor, which is exactly the failure the returning-creator persona (ADR-0009) suffers from.

## Decision

We will represent a command as text plus provenance, and **validate it against the target's
command grammar** extracted from the game (the Brigadier command tree that the server can
report). Validation is a compiler stage with real diagnostics, not a lint.

The IR command form is tagged, so a structured form can be added later as a second variant
without a schema break.

## Consequences

We get most of the value of an AST — errors caught before the pack is installed, and precise
messages like "this argument moved in 1.21" — for a fraction of the cost, and the grammar
updates itself with each release because it is extracted, not written.

The limit is honest and must be documented: retargeting a function across game versions is
**best-effort**. Packsmith can tell you a command is no longer valid for the new target; it
cannot rewrite it for you. Blocks that emit commands should emit them per-target rather than
relying on retargeting.

## Alternatives considered

- Full command AST: correct, and a multi-year project on its own that would have to be
  rewritten every release.
- Unvalidated text: cheapest, and it fails the primary persona.
