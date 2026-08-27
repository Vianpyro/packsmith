# expected/ not yet verified

This case expects a **compile failure**, not a file tree. This directory is
intentionally empty; the expected diagnostic is described in `../README.md` and
must be pinned down exactly once the command-grammar stage exists.

To verify:

- Confirm against a real Minecraft: Java Edition 26.2 instance that
  `execute @e[type=zombie] ~ ~ ~ say boo` is in fact rejected by 26.2 (paste it
  into a function or the chat and observe the parse error), so that this case is
  testing a real incompatibility and not a Packsmith invention.
- Once the compiler can produce it, record the exact diagnostic — code, severity,
  node id, statement address, message, and suggested fix — as the golden result
  for this case, in whatever form the conformance runner adopts for failures.
- A passing run means the build stops with that diagnostic and emits no pack.
