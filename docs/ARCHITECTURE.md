# `html-generator` architecture

How the crate is put together, for contributors. The user-facing story
is in the [README](../README.md); this page is about the shape of the
code and the decisions behind it.

## Layout

```text
html-generator/
├── src/
│   ├── lib.rs           # public surface, HtmlConfig, the builder, IO helpers
│   ├── generator.rs     # the pipeline: Markdown in, HtmlOutput out
│   ├── accessibility.rs # ARIA enrichment and WCAG validation
│   ├── seo.rs           # meta tags and JSON-LD structured data
│   ├── minifier.rs      # single-pass HTML minifier (native since 0.0.10)
│   ├── math.rs          # LaTeX to MathML, Mermaid block rewriting
│   ├── elements.rs      # heading ids, class generators
│   ├── emojis.rs        # bundled emoji data and ARIA labels
│   ├── performance.rs   # async wrappers behind the `async` feature
│   ├── utils.rs         # front-matter extraction, escaping, path checks
│   ├── error.rs         # HtmlError
│   └── wasm.rs          # #[wasm_bindgen] glue, `wasm` feature only
├── tests/               # integration, fixtures, targeted coverage, wasm smoke
├── examples/            # fourteen runnable examples + support.rs helper
├── benches/             # Criterion: this crate, and against other engines
├── fuzz/                # libFuzzer targets, seed corpus, regression inputs
├── docs/adr/            # architecture decision records
└── supply-chain/        # cargo-vet state
```

## The pipeline

`generate_html_with_diagnostics(markdown, config)` is the whole crate in
one call. `generate_html` is the same thing with the diagnostics
discarded. The steps run in a fixed order, and each one after the first
is non-fatal: a step that fails records an `Error`-level `Diagnostic`
and the pipeline continues, because a missing table of contents is not a
reason to return no HTML at all.

1. **Markdown to HTML** (`comrak`). The only fatal step. Raw HTML in the
   source is escaped unless `allow_unsafe_html` is set.
2. **Sanitisation** (`ammonia`), only when the caller has opted into raw
   HTML *and* `sanitize_html`. Both default to false; see the README's
   Security section for why that pair is deliberate.
3. **Syntax highlighting**, when enabled.
4. **Math** (`pulldown-latex`, behind the `math` feature) and
   **diagrams** (Mermaid fenced blocks rewritten for client-side
   rendering). Each reports a diagnostic only when it actually changed
   the document.
5. **DOM parse once** (`scraper`) for the read-only steps: SEO metadata
   and heading extraction. Parsing once and sharing the tree is the
   reason these are grouped.
6. **Table of contents**, built from the extracted headings.
7. **Accessibility**: ARIA enrichment, then WCAG validation.
8. **Minification**, last, so every earlier step sees readable markup.

## The accessibility pass

`add_aria_attributes` is the most delicate code in the crate. It
computes replacements against the source string and applies them by byte
offset, because rebuilding the document from a DOM would lose the
formatting the caller's other tools depend on.

Offsets are resolved in three tiers: an exact anchored match, an
approximate offset carried forward from the previous replacement, and a
last-resort first-occurrence search. Each tier exists because the one
above it fails on real input, and the fallbacks are where this crate's
slice-boundary bugs have lived. `fuzz_accessibility` exists for exactly
this function.

`validate_wcag` walks the result and reports issues by type and level.
It checks the rules it implements — heading order, image alt text, form
labels, landmark structure, link text — and conformance of a whole site
is not something it can certify.

## Configuration

`HtmlConfig` is a plain struct with public fields plus a builder
(`HtmlConfig::builder()`). Both are supported: the struct-update form
reads well when a test needs two flags, the builder when a caller is
assembling configuration from elsewhere. `validate()` is the single
place where cross-field rules live.

## Errors

`HtmlError` is one enum with a variant per failure class. The pipeline
distinguishes fatal from non-fatal by construction: only step 1 returns
`Err`, and everything else surfaces through `HtmlOutput::diagnostics`,
so a caller can log degradation without losing output.

## The wasm surface

`src/wasm.rs` is `#[wasm_bindgen]` glue behind `#[cfg(feature =
"wasm")]`. It exposes three functions to JavaScript and holds no logic
of its own beyond JSON option decoding. Native coverage cannot execute
it, so it is excluded from the coverage gate and verified by
`tests/wasm_smoke.rs` under `wasm-pack`. That trade is argued in
[`DEVELOPMENT.md`](../DEVELOPMENT.md).

## Tests

- `src/**` `#[cfg(test)]` — unit tests next to the code.
- `tests/unit_coverage.rs` — targeted tests for paths the module suites
  miss, grouped by why they were missed rather than by module.
- `tests/integration_tests.rs` — the pipeline end to end.
- `tests/fixture_tests.rs` with `tests/fixtures/` — input and expected
  output pairs.
- `fuzz/` — three targets; the seed corpus and every fixed-bug input
  replay per push.

## Where to read next

- [`docs/adr/`](adr/README.md) for the decisions that shape the above.
- [`DEVELOPMENT.md`](../DEVELOPMENT.md) for reproducing every CI gate.
