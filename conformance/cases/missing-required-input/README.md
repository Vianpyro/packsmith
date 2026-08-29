# missing-required-input

A `crafting-shapeless` block with a name and ingredients but no `result` is an
`input-missing` error at that node, and no pack is emitted. This pins that a
required port with neither a literal nor an incoming edge stops the build, with
the diagnostic anchored at the node that is missing the value. It also pins the
diagnostic's `params` — the block and the port it names — since naming both is
what makes the message locatable (ADR-0009).

Assumed built-in block: `packsmith/crafting-shapeless`, as in `recipe`.
