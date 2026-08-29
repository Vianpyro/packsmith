# edge-to-missing-node

A data edge whose `from` endpoint names a node that is not in the graph is an
`edge-unknown-node` error, anchored at the node that would have received the
value, and no pack is emitted. This pins that the validation pass checks both
endpoints of every edge against the node set (`spec/graph.schema.json`
`$defs/edge`), so a dangling connection is caught here rather than dropped
silently.

Assumed built-in blocks: `packsmith/function` and `packsmith/command`, as in
`one-function`. v1 function bodies carry no data edges (ADR-0012); this case
exercises edge validation itself, not a wired body.
