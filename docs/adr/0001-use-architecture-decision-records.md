# ADR-0001: Record architecture decisions

- **Status:** accepted
- **Date:** 2026-08-27

## Context

Most of this project will be written by an AI agent across many sessions with no shared
memory between them. The failure mode is not bad code; it is silent architectural drift,
where each session makes a locally reasonable choice that contradicts an earlier one.

## Decision

We will record every structural decision as a numbered ADR in `docs/adr/`, using
`0000-template.md`. An accepted ADR is binding. It is never edited to reflect a change of
mind; it is superseded by a new ADR that states what changed and why.

## Consequences

Sessions start with the "why" already written down, which is the part that is normally lost.
The cost is a few minutes per decision and the discipline to write the ADR before the code.

## Alternatives considered

- Comments in code: invisible until you already opened the right file.
- A single design document: becomes stale silently, with no record of what changed.
