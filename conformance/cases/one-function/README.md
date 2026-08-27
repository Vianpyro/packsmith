# one-function

A `function` block with one `command` child statement lowers to exactly one
function resource, whose id is `example:hello` and whose body is the single line
`say Hello, world!` in slot order. This pins the `function` category end to end:
the slot becomes a `commands` body (ir.schema `body-commands`), the child
statement keeps its address `(fn-hello, "body", 0)`, and the emitted file lands
where target data says the `function` category lives for 26.2, with the target's
file extension.

Assumed built-in blocks: `packsmith/function` (statement; input `name`:
`id` of registry `minecraft:function`; slot `body`), `packsmith/command`
(statement; input `command`: `string` of format `command`).
