# 0004. A native single-pass minifier instead of `minify-html`

- **Status:** accepted
- **Date:** 2026-08-11

## Context

Minification was delegated to `minify-html`. It is a good general
minifier, and it brought roughly 1,300 lines of lockfile with it —
a dependency tree far larger than the one job it did here, on markup
this crate had itself just produced and therefore fully understood.

## Decision

`src/minifier.rs`: a single-pass minifier written for this crate's own
output. It collapses inter-tag whitespace, drops comments, and leaves
`<pre>`, `<code>`, `<textarea>` and `<script>` content untouched.

## Consequences

- The dependency graph loses a large subtree; builds and audits get
  smaller.
- The minifier only has to be correct on HTML this crate emits. Feeding
  it arbitrary third-party markup is out of scope, and that boundary is
  stated rather than assumed.
- Minification is the last pipeline step, so every earlier step still
  sees readable markup.
