---
paths:
  - 'crates/**/*.rs'
  - 'xtask/**/*.rs'
---

# Rust rules

- `#![forbid(unsafe_code)]` at the top of every crate root.
- No `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` outside `#[cfg(test)]`,
  `build.rs`, and `fn main`. Return a typed error instead.
- Libraries define their own error enum with `thiserror`. `anyhow` is allowed only in
  `packsmith-cli` and `xtask`.
- Diagnostics are values, not strings: a diagnostic carries a code, a severity, a node id,
  and an optional fix suggestion. Collect diagnostics rather than bailing on the first error,
  so the editor can show them all at once.
- Keep cyclomatic complexity low: prefer early returns over nested conditionals, and extract
  a named function rather than adding a fourth level of indentation.
- Do not add a trait for a single implementation. Do not add a lifetime parameter to avoid a
  clone in code that is not on a measured hot path.
- Anything that reaches the emitted output must have a deterministic order. Use `BTreeMap`
  and `BTreeSet`, or sort explicitly before emitting. `HashMap` is fine for internal lookups
  that never influence output ordering.
