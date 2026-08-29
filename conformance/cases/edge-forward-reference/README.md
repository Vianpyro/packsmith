# edge-forward-reference

A data edge that reads a value from `fn-late` into `fn-early`, where `fn-late`
comes after `fn-early` in document order, is an `edge-forward-reference` error at
the receiving node, and no pack is emitted. This pins the direction rule of
`spec/types.md` section 2.4: a value has to be produced before it is read, so an
edge from a later node to an earlier one is rejected here rather than lowering to
an order the game cannot honour.

Assumed built-in block: `packsmith/function`, as in `one-function`. v1 bodies
are wire-free (ADR-0012); this case exercises edge ordering itself.
