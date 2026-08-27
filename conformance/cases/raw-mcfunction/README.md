# raw-mcfunction

The escape hatch: a `mcfunction` block whose `source` is a whole function file
pasted as text lowers to one `function` resource whose body is that text passed
through unchanged — comment line and blank line preserved — while every non-blank,
non-comment line is still validated against the target's command grammar
(ADR-0012, OPEN-QUESTIONS A4). This pins that dropping to raw text when no block
fits does not also drop validation, and that the passthrough is byte-exact so the
build stays deterministic.

Assumed built-in block: `packsmith/mcfunction` (statement; input `name`: `id` of
registry `minecraft:function`; input `source`: `string` of format `mcfunction`).
The `mcfunction` string format — split on newlines, skip blank and `#`-comment
lines, validate the rest as command lines — is a target-data-supplied format, the
extension mechanism spec/types.md section 4.4 sanctions. It is not among the three
formats the v1 table currently lists; see `docs/BACKLOG.md`.
