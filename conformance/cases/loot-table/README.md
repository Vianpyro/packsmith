# loot-table

A `loot-table` block with one guaranteed item drop lowers to a single
`loot_table` resource whose JSON body is one pool of one item entry for
`minecraft:diamond`. This pins that the `loot_table` category emits a JSON body
and that a single-item `item_stack` literal with no explicit count lowers without
inventing a `count`, `functions`, or `conditions` key the input did not ask for.

Assumed built-in block: `packsmith/loot-table` (statement; input `name`: `id` of
the loot-table registry; input `drops`: `list` of `item_stack`).
