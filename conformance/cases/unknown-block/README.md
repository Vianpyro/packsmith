# unknown-block

A node whose `block` address matches no installed block is a `block-unknown`
error at that node's statement address, and no pack is emitted. This pins the
first thing the validation pass checks: a graph that names a block Packsmith
does not have fails before lowering, rather than lowering to a silent gap.
