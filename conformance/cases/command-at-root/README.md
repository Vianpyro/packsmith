# command-at-root

A `command` block placed directly in the top-level `root` slot, not inside a
function, is a `slot-rejects-block` error at `(null, "root", 0)`, and no pack is
emitted. This pins the placement rule the BACKLOG deferred: a command produces a
line in a function body and nothing else, so it cannot stand alone as a
top-level statement.

Assumed built-in block: `packsmith/command`, as in `one-function`.
