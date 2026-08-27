# conformance/

Golden cases that every implementation must satisfy: the compiler, each block SDK, and each host. A case is a directory under `cases/`:

```
cases/<name>/
  README.md      one paragraph: what behaviour this case pins down
  input.json     the graph
  target.json    the Minecraft target
  expected/      the exact expected file tree
```

Rules: one behaviour per case, keep cases small, and never edit an expected tree to make failing code pass.
