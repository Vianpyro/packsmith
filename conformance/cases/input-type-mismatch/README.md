# input-type-mismatch

A `crafting-shapeless` whose `result` port holds a bare string where the port's
type is `item_stack` is an `input-type` error at that node, and no pack is
emitted. This pins that the validation pass checks a literal's shape against the
port's declared type (`spec/types.md` section 4) before lowering, rather than
letting the wrong shape reach the emitted JSON.

Assumed built-in block: `packsmith/crafting-shapeless`, as in `recipe`.
