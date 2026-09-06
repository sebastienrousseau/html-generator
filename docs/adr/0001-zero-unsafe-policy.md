# 0001. `#![forbid(unsafe_code)]`, pure Rust, no FFI

- **Status:** accepted
- **Date:** 2024-10-07 (recorded 2026-09-06)

## Context

The crate turns untrusted Markdown into HTML that a site will serve. A
memory-safety bug here is a bug in every page built with it. Nothing in
the problem needs raw pointers, and the parsers it builds on (`comrak`,
`scraper`, `ammonia`) are pure Rust.

## Decision

`#![forbid(unsafe_code)]` at the crate root; no C dependencies, no FFI.
A dependency that requires `unsafe` in this crate's own code, or that
pulls in a C build, is a reason to pick a different dependency.

## Consequences

- The compiler proves the absence of unsafe blocks; there is nothing to
  audit by hand.
- Miri's job here is to check the *interaction* with dependencies that
  use `unsafe` internally, not this crate's own code.
- If a hot path ever needs `unsafe`, that is a new ADR superseding this
  one, not a local `#[allow]`.
