# id-missing-namespace

A `function` whose `name` is `tick` with no namespace is an `input-constraint`
error at that node, and no pack is emitted. This pins the rule from
`spec/types.md` section 4.5: a value without a namespace is rejected, never
defaulted to `minecraft:`, and the diagnostic is the newcomer's first lesson in
what a namespace is. The `command` child is well-formed and is not what fails.

Assumed built-in blocks: `packsmith/function` and `packsmith/command`, as in
`one-function`.
