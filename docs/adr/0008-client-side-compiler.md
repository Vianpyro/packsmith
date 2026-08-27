# ADR-0008: The compiler runs client-side; the server is a registry only

- **Status:** accepted
- **Date:** 2026-08-27
- **Resolves:** OPEN-QUESTIONS B1, B2

## Context

Blocks are third-party artifacts fetched from a registry and executed during a build. Running
them on a shared server makes every self-hosted instance an arbitrary-code-execution target
and forces multi-tenant isolation, quotas, and abuse handling onto a hobby deployment.

## Decision

We will compile `packsmith-compiler` to `wasm32` and run the entire build in the user's
browser. Computed blocks are executed by a browser-side host (jco) with the same limits as
the native host. The server's only responsibilities are serving static assets and hosting the
block registry. It never executes a block and never sees a user's project unless the user
publishes it.

The native CLI remains a first-class second host, using wasmtime.

## Consequences

Self-hosting reduces to static files plus a small registry service, which is what makes
"self-hostable" a real promise rather than a slogan. No untrusted execution server-side.
Projects stay on the user's machine by default. Builds are offline-capable once blocks are
cached.

The cost is two hosts to keep in sync. The WIT contract and the conformance suite are the
mitigation; if the two hosts ever disagree on a conformance case, that is a release blocker.
Large builds are also bounded by browser memory, which the sandbox limits must respect.

## Alternatives considered

- Server-side builds: simpler to implement, but turns every instance into an RCE surface and
  makes single-user self-hosting disproportionately heavy.
