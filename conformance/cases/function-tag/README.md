# function-tag

A `function-tag` block bound to `minecraft:load` with one project-defined function
in its list lowers to a `tags/function` resource whose JSON body is
`{"values":["example:hello"]}`. This pins two things: the `tags/function` category
emits a JSON body (ir.schema `body-json`) at the path target data gives for tags,
and a bare `id` in the `functions` list resolves against a function defined
elsewhere in the same project (the compiler's cross-reference pass, spec/types.md
section 4.5) rather than being flagged as unknown.

Assumed built-in blocks: `packsmith/function` (as in `one-function`),
`packsmith/function-tag` (statement; input `name`: `id` of the function-tag
registry; input `functions`: `list` of `id` of registry `minecraft:function`
with `allow_tag: true`; optional `replace`: `bool`, default `false`).
