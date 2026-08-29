# missing-required-input

A `crafting-shapeless` block with a name and ingredients but no `result` is an
`input-missing` error at that node, and no pack is emitted. This pins that a
required port with neither a literal nor an incoming edge stops the build, with
the diagnostic anchored at the node that is missing the value.

Assumed built-in block: `packsmith/crafting-shapeless`, as in `recipe`.
