# ADR-0013: Editor layout is stored separately from graph semantics

- **Status:** accepted
- **Date:** 2026-08-27

## Context

ADR-0007 guarantees that the same graph produces byte-identical output. If node coordinates
live in the graph, dragging a node changes the project file, which changes the build hash,
which invalidates the cache and produces a misleading diff for a purely cosmetic edit.

## Decision

A project directory separates the two:

```
graph.json     semantic: nodes, block references, input values, connections
layout.json    cosmetic: node positions, collapsed state, colours, comments, viewport
```

Only `graph.json` and the lockfile feed the build hash. `layout.json` is never read by the
compiler and may be absent.

## Consequences

Diffs are meaningful, the cache survives cosmetic edits, and a project built by the CLI needs
no UI data at all. Adds one file to the project format and requires the editor to keep two
documents consistent, which is a small, contained cost.

## Alternatives considered

- Layout inside the graph with hashing that skips a `ui` field: works, and quietly breaks the
  moment someone adds a field in the wrong place.
