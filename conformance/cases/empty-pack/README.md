# empty-pack

A project with no nodes still produces a valid, installable data pack: a single
`pack.mcmeta` at the pack root carrying the description from `project.description`
and the format number the target data supplies for kind `data`, and nothing else.
This pins the floor of the emitter — an empty `root` slot is a valid, silent
program (ADR-0016), not an error — and it is the case that first exercises
`pack.mcmeta` shaping without any resources in the way.
