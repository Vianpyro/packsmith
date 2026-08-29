---
description: Execute a numbered task from docs/TASKS.md within its stated scope
argument-hint: <task number>
---

Execute task **$1** from `docs/TASKS.md`.

## Before doing anything

1. Read `CLAUDE.md` in full.
2. Read task $1 in `docs/TASKS.md` — its goal, governing ADRs, exact scope, out-of-scope list,
   and exit criterion.
3. Read every ADR the task lists, from `docs/adr/`.
4. If the task depends on an item still marked `OPEN` in `docs/OPEN-QUESTIONS.md`, stop and ask.
5. Set the task's `**Status.**` line in `docs/TASKS.md` to `In progress` before the first
   change.

## While working

- Do only what the **Scope** section names. Nothing from the **Out of scope** section, and
  nothing the task does not mention.
- Adjacent work you notice goes in `docs/BACKLOG.md` as one line. Do not do it.
- Follow the invariants and conventions in `CLAUDE.md` and `.claude/rules/`. Spec and
  conformance changes are their own commits (`.claude/rules/spec.md`).
- Every user-facing diagnostic carries a code, severity, source node address, and where
  possible a suggested fix, phrased in game terms first (ADR-0009, ADR-0012).

## Stop conditions — stop and report, do not push through

- **Two ADRs the task lists contradict each other**, or a listed ADR is still `proposed` and
  the task needs it accepted. Do not resolve it. Report the conflict; a new superseding ADR is
  the maintainer's call (`CLAUDE.md`).
- **The exit criterion is marked 🔒**, or otherwise needs a real game instance, a JDK plus
  server jar, an EULA acceptance, or a judgement no test in this repo can make. Do the code
  work up to that line, then stop and report exactly what the maintainer must check. Do not
  assume it passed. Leave the `**Status.**` line at `In progress`; never mark a 🔒 criterion
  `done` yourself.

## On finishing

Update the task's `**Status.**` line in `docs/TASKS.md`:

- `Done` only when the exit criterion is actually met. If it is 🔒, or otherwise needs the
  maintainer's confirmation, it is not done until the maintainer confirms — leave it
  `In progress` and report what they must check.
- `Blocked on <what>` if a stop condition was hit.

## When the exit criterion is fully machine-checkable

Run `cargo xtask ci`. Report it passing, the new test that fails without the change, and
confirm the diff contains nothing outside the task's scope.
