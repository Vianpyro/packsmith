# recipe

A `crafting-shapeless` block with one ingredient and a counted result lowers to a
single `recipe` resource whose JSON body is a shapeless crafting recipe turning
one `minecraft:diamond_block` into nine `minecraft:diamond`. This pins that the
`recipe` category emits a JSON body, that an `item_stack` literal with an explicit
`count` survives lowering, and that the recipe type discriminator the game
expects for 26.2 is written from target data rather than hardcoded.

Assumed built-in block: `packsmith/crafting-shapeless` (statement; input `name`:
`id` of the recipe registry; input `ingredients`: `list` of `id` of registry
`minecraft:item` with `allow_tag: true`; input `result`: `item_stack`).
