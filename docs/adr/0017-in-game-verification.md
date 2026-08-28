# ADR-0017: In-game verification uses vanilla headless GameTest

- **Status:** accepted
- **Date:** 2026-08-28
- **Resolves:** OPEN-QUESTIONS E1
- **Refines:** ADR-0014

## Context

Conformance cases prove the compiler produces the file tree we expect. They do not prove the
game accepts it. E1 has been open since Phase 0 and blocks Phase 2, and the assumed answer was
a bespoke Docker harness driving a server through `/reload` and a set of hand-written probe
functions, with results scraped from the log.

That assumption was based on a wrong premise: that Mojang ships no headless validation path.
It does. Since snapshot 25w03a, shipped in 1.21.5, the GameTest framework is reachable from
data packs in vanilla, with no mod, and the server jar exposes a headless entry point:

```
java -DbundlerMainClass="net.minecraft.gametest.Main" -jar server.jar --packs <dir>
```

Tests are assets in the `test_instance` registry. `test_environment` assets group them and
supply preconditions; an environment of type `function` runs mcfunction files for setup and
teardown. The framework runs tests in a separate superflat world.

## Decision

We will verify generated packs with vanilla headless GameTest, invoked from `xtask`.

- The harness builds a conformance case, writes the resulting pack plus a generated test pack
  into a packs directory, and runs `net.minecraft.gametest.Main` against it.
- Assertions live in the test pack as `test_instance` assets with `function`-type environments,
  not in log scraping.
- No mod loader, no Fabric, no mod version matrix. The harness validates that a pack works in
  vanilla, which a mod-dependent harness cannot honestly claim.
- This is the same invocation family as the `--reports` data generator that ADR-0014 keeps as
  its fallback source, so `xtask` gains one jar-handling path rather than two.

## Consequences

E1 closes, and the Phase 2 harness becomes ordinary work instead of the phase's largest
unknown. Verification runs on the same artefact users install, in the same runtime, with no
intermediary that could mask a defect.

Second-order, and deliberately out of v1 scope: `test_instance` and `test_environment` are
registry categories like any other, so under ADR-0010 supporting them is target data plus
conformance cases. That would let Packsmith generate tests *for a user's own pack*, which no
low-code tool in this space offers. Recorded in `docs/BACKLOG.md`; not scheduled.

The costs are real and bounded. The harness needs a JDK and a server jar, so it cannot run in
the same lightweight CI job as `cargo test`, and it must respect the EULA acceptance step that
ADR-0014 avoids for target data. GameTest is structure-oriented, so a purely functional
assertion still needs a trivial structure to hang off. Version coupling is inherent: the
harness only proves things about the target it runs.

## Alternatives considered

- **A bespoke `/reload`-and-scrape-the-log harness.** The original assumption. Rejected: more
  code, more fragile, and it reimplements a framework the game already ships.
- **PackTest** (misode). A Fabric mod predating the vanilla feature, with tests written as
  plain mcfunction, `-Dpacktest.auto` for CI, and GitHub annotations on failure. Nicer to write
  than vanilla GameTest, and rejected as a dependency because it puts a mod loader between us
  and the thing we claim to verify. Worth revisiting if authoring vanilla test instances proves
  painful enough to matter. Recorded in `docs/PRIOR-ART.md`.
- **Static validation only** (Spyglass, beet/mecha). Both are real and good, both are the wrong
  language for our core (TypeScript and Python), and neither answers the question this ADR is
  about: static analysis cannot tell you the game loaded your pack. Spyglass remains useful as
  a structural reference for the ADR-0012 validator.
