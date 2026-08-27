# legacy-syntax-rejected

The one deliberate failure. `execute @e[type=zombie] ~ ~ ~ say boo` is the
pre-1.13 `execute` form — target, position, command with no `as` / `at` / `run`.
It is valid graph JSON and a syntactically well-formed `string`, so it passes
schema validation, but it must be rejected by the compiler's command-grammar
stage for target 26.2 with a diagnostic that names the statement address
`(fn-legacy, "body", 0)`, says `execute` no longer takes a bare selector and
position, and points at the modern `execute as <targets> at @s run <command>`
shape. This is the returning-creator failure mode from ADR-0009: the build stops
here instead of the pack installing and silently doing nothing in game.

Assumed built-in blocks: `packsmith/function` and `packsmith/command`, as in
`one-function`.
