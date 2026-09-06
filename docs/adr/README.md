# Architecture Decision Records

This directory holds the architectural decisions that shape
html-generator — the choices that would be expensive to reverse and that
contributors should understand before proposing structural changes.

## Format

Every ADR uses [Michael Nygard's format](https://github.com/joelparkerhenderson/architecture-decision-record/tree/main/locales/en/templates/decision-record-template-by-michael-nygard)
— see [`TEMPLATE.md`](./TEMPLATE.md). Sections are:

- **Status:** proposed / accepted / superseded / deprecated
- **Context:** what forces are at play
- **Decision:** what we're doing
- **Consequences:** what becomes easier and harder

ADRs are **immutable** once accepted — if a decision changes, the old
ADR moves to "superseded" and a new ADR is added with a reference back.
Nothing is silently rewritten.

## Index

| # | Title | Status |
|---|---|---|
| [0001](./0001-zero-unsafe-policy.md) | `#![forbid(unsafe_code)]`, pure Rust, no FFI | accepted |
| [0002](./0002-non-fatal-pipeline-steps.md) | Only the Markdown parse is fatal; every later step degrades | accepted |
| [0003](./0003-offset-rewriting-in-accessibility.md) | ARIA enrichment rewrites by byte offset, not by rebuilding the DOM | accepted |
| [0004](./0004-native-minifier.md) | A native single-pass minifier instead of `minify-html` | accepted |
| [0005](./0005-first-party-exact-pins.md) | First-party crates are pinned exactly (`=0.0.X`) | accepted |

## When to add an ADR

Add one when you're about to make a decision that:

- Is hard to reverse (changes the data model, public API surface,
  dependency floor, or core invariants like the unsafe policy)
- Will surprise a future contributor reading the code
- Has plausible alternatives that someone might propose later

Don't add ADRs for routine implementation choices — those go in commit
messages and code comments. The bar is "would I want a new contributor
to read this before proposing the opposite?"
