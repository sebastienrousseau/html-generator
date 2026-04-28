# Changelog

All notable changes to **html-generator** are recorded in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.5] — Unreleased

### Added (capability)

- **Server-side LaTeX → MathML.** New `enable_math` config flag and
  `html_generator::math::convert_math` post-processor convert `$..$`
  and `$$..$$` spans to native `<math>` elements via
  `pulldown-latex`. Pure server-side; browsers render MathML without
  JS. Parse errors are encoded inline as `<merror>` markers rather
  than failing the build. Gated behind the `math` feature (on by
  default; turn off with `--no-default-features` for a smaller binary).
- **Mermaid diagram passthrough.** New `enable_diagrams` config flag
  and `html_generator::math::rewrite_mermaid_blocks` post-processor
  rewrite `\u{60}\u{60}\u{60}mermaid` fenced blocks from
  `<pre><code class="language-mermaid">` to `<pre class="mermaid">`
  so the standard client-side mermaid.js bundle picks them up. No
  server-side rendering — diagram source flows through verbatim.
- **WebAssembly target.** New `wasm` feature exposes three
  JS-friendly entry points via `wasm-bindgen`: `generateHtml`,
  `generateHtmlFullDocument`, `generateHtmlWithOptions`. Build with
  `cargo build --target wasm32-unknown-unknown --features wasm
  --no-default-features` or `wasm-pack build --target web --features
  wasm --no-default-features`. The WASM build path delegates to
  `comrak` directly because `mdx-gen` pulls in `tokio` (which pulls
  in `mio`) unconditionally and `tokio` doesn't compile to
  `wasm32-unknown-unknown`. Trade-off: WASM loses `mdx-gen`'s
  `:::class` blocks, image-class syntax, and `syntect` syntax
  highlighting; everything else (CommonMark + GFM, ARIA, TOC,
  JSON-LD, math, mermaid) renders identically. Smoke tests live in
  `tests/wasm_smoke.rs` and run under `wasm-pack test --node
  --features wasm`.
- **Comparative benchmarks.** New `benches/competitors.rs` runs the
  same realistic 8 KB blog-post payload through `html-generator`,
  `comrak`, and `pulldown-cmark` so the README can cite measured
  numbers, not estimates. Numbers (Apple M-series, `cargo bench
  --bench competitors --quick`):
    * `pulldown-cmark` parse only: **45 µs**
    * `comrak` parse only: **172 µs**
    * `html-generator` full pipeline: **2.09 ms**
  Pure parsers are faster but produce raw HTML you still need to
  post-process; the html-generator number includes ARIA, TOC,
  JSON-LD, and minification in one pass.

### Examples

- New `examples/math_and_diagrams.rs` covers four cases: inline +
  display LaTeX, currency-style `$5` left untouched, mermaid block
  rewrite, and both at once.



The first release after the public `0.0.4` ship on crates.io. Everything
below is hardening on top of the published `0.0.4` and is **not** yet
on the registry — `cargo install html-generator` continues to resolve
to `0.0.4` until this version is published.

### Performance

- Bench profile gets explicit `opt-level = 3` + fat LTO. The release
  profile had been `opt-level = "z"` (size-optimised) and
  `cargo bench` inherited it; benchmarks were measuring a size-tuned
  binary, not realistic throughput. This single change took
  `generate_html` on a trivial input from **421 µs → 117 µs** before
  any code changes.
- Per-handler fast-paths in `add_aria_attributes`. Every
  `add_aria_to_*` now short-circuits via `str::contains` (SIMD-backed
  via stdlib `memchr`) before any html5ever parse. A realistic 8 KB
  blog-post markdown skips 7 of the 9 ARIA parses; pipeline drops
  from **6.16 ms → 2.58 ms** (−58%).
- `escape_html` gains a SIMD short-circuit before entering the regex
  engine — zero-alloc on the typical no-escape input.
- mdx-gen `Options` is now memoised in a `Lazy<Options<'static>>` and
  cloned per call; only the runtime-varying fields (unsafe HTML,
  syntax theme) mutate. `add_custom_classes` and
  `process_images_with_classes` return `Cow<'_, str>` so a markdown
  document with no `:::class` or image-class syntax allocates nothing.
- `generate_id` walks the input once instead of three times. Three
  intermediate allocations → one per heading.
- `remove_invalid_aria_attributes` is offset-based via `DomReplacer`
  with a no-op fast path when every attribute is already valid.
  `O(n²)` → `O(n)` on element-heavy documents.
- Nine hot-path `Html::parse_document` calls swapped for
  `Html::parse_fragment` (the input is a fragment; document parsing
  was synthesising an unnecessary `<html><head><body>` wrapper).
- Compact JSON-LD output (`serde_json::to_string` instead of
  `to_string_pretty`).

- `add_aria_attributes` skips the post-pipeline
  `remove_invalid_aria_attributes` parse when the input contains no
  `aria-*` substring (cheapest possible byte-scan). The slow path
  still runs when a user supplied raw HTML with `allow_unsafe_html:
  true` could carry invalid attributes. Realistic 8 KB markdown:
  **2.58 ms → 2.04 ms (−21%)**. Trivial input is now mdx-gen-bound
  (html5ever cost is negligible on a 30-byte string); breaking past
  the trivial floor would require a different markdown engine, not
  an ARIA refactor — out of scope for v0.0.5.

**Throughput on the trivial `generate_html` bench: 2 374 → 10 316
msg/sec (4.35×). Realistic 8 KB markdown: 162 → 490 msg/sec (3.0×).**
Scorecard methodology and per-bench measurements recorded in commit
`8fcfd7c`; final fast-path measured in `e522d38`.

### Security

- `validate_file_path` now rejects NUL bytes — closes a C-string
  smuggling vector on Unix where path-string handling silently
  truncates at the first `\0`.
- ARIA IDs are deterministic. The previous implementation generated
  `dialog-desc-{uuid::Uuid::new_v4()}` and `aria-{uuid}` per call,
  so the same Markdown produced different HTML on every run. Per
  call ID counters give byte-identical output for byte-identical
  input. The `uuid` dependency was dropped entirely.
- Dropped `RUSTSEC-2025-0068` from the `cargo-audit` and `cargo-deny`
  ignore lists. The advisory targets `serde_yml@0.0.12` (which links
  the unsafe `libyml` C library); after renaming our vendored copy
  from `serde_yml` to `yaml_safe` the name collision evaporated, so
  the ignore is no longer masking anything.
- All commits are signed (ED25519); every commit message carries the
  `Assisted-by:` trailer per the Linux kernel coding-assistants
  standard.

### Changed

- Vendored YAML implementation renamed from `serde_yml` to
  `yaml_safe` (re-imported from
  `/Users/seb/Code/Public/Rust/yaml_safe@0.1.0`) and then **inlined**
  into `src/yaml/` as a private module. `forbid(unsafe_code)`
  preserved; API surface unchanged at the parent-crate boundary; one
  call site (`src/utils.rs::parse_yaml_to_map`). The intermediate
  `crates/yaml_safe/` separate-crate path-dep is gone; `serde` and
  `indexmap` now appear as direct `[dependencies]` of html-generator.
  This unblocks `cargo publish --dry-run` (which rejects path deps
  without `version =`) without taking on any registry dependency on
  the unsound `serde_yml`. Upstream of record remains the standalone
  `yaml_safe` repo — fixes flow there first and are mirrored in by
  re-vendoring.
- `accessibility::Error` now converts cleanly to `HtmlError` via
  `From`. `?` composes across module boundaries without explicit
  `.map_err`. No public signatures changed.
- Static compile-time regex/selector patterns now panic with a clear
  message on parse failure instead of silently disabling features
  via `.ok()`. A typo in a static pattern fails loudly at first use.
- `generate_id` is single-pass; the now-unused
  `CONSECUTIVE_HYPHENS_REGEX` static is removed.
- `DomReplacer::apply` uses `sort_by_key(Reverse(_))` per
  `clippy::unnecessary-sort-by` (Rust 1.95+).

### Removed

- `uuid` runtime dependency (was only used for non-deterministic
  ARIA ids; replaced with deterministic counters).
- `tempfile` from `[dependencies]` — was only used in tests and
  benches; moved to `[dev-dependencies]`.
- Duplicate `performance::generate_html` (one-line wrapper around
  `generator::markdown_to_html_with_extensions`); use
  `generator::generate_html` instead.
- Unused `HtmlError::Minification { message, size, source }` variant
  (only the `MinificationError(String)` form was ever constructed).

### Fixed

- `validate_file_path`'s `#[cfg(not(test))]`-gated test was nested
  inside a `#[cfg(test)]` module — it was never compiled. Removed
  the dead gate; added live tests for absolute-path acceptance and
  NUL-byte rejection.
- `read_input` and `write_output` factored into testable
  `read_all_from_reader<R: Read>` and `write_all_to_writer<W: Write>`
  helpers so stdin and writer-failure paths are covered without
  subprocess tests.

### CI gates

- New `codecov.yml` enforces a coverage floor: project ≥95% (1%
  threshold), patch ≥90% (1% threshold). The `coverage-exclude`
  glob the workflow passes to tarpaulin is mirrored as the
  Codecov `ignore` list so the two sources agree on the
  denominator.

### Tests + coverage

- New `tests/unit_coverage.rs` — 58 tests covering every front-matter
  variant, builder option, full-document wrapping, fragment language
  injection, Writer destinations, every `From<accessibility::Error>`
  variant, `DomReplacer` direct cases (fast path, shorthand fallback,
  last-resort, duplicate snippets), checkbox vs option input labels,
  tooltip attachment, modal `aria-describedby` reuse, fast-path
  regressions.
- New `generate_html_realistic` criterion benchmark on an ~8 KB blog
  post payload — establishes a regression gate for realistic-document
  throughput.
- Determinism regression test in `tests/integration_tests.rs`
  asserts byte-identical output across two consecutive calls.
- **Line coverage: 91.10% → 98.04%** (`cargo llvm-cov`); Codecov
  project 95.78%, patch 93.90%.

### CI

- Fixed a five-day-long invisible CI failure: `ci.yml` was passing
  `all-features: true` to `pipelines/.github/workflows/rust-ci.yml@main`,
  which declares no such input. The workflow exited with
  `startup_failure` rather than a check failure on the PR, so the
  branch was passing review checks while no jobs were actually
  running. Removed the bad input.
- Tarpaulin coverage scoping. The reusable workflow's
  `coverage-exclude-packages` cannot reach a path-only dependency
  (vendored `crates/yaml_safe/` is not a workspace member). Switched
  to file-glob exclusion (`tests/*,benches/*,examples/*,build.rs,src/yaml/*`)
  which catches the vendored YAML tree regardless of how it's wired
  in (separate crate or inlined module). This took the
  codecov/project gate from `53.76% (-33.15%)` to `95.78% (+8.87%)`
  in one commit; the same glob still applies post-inline since
  `src/yaml/*` is the new home of the same code.
- New `[profile.bench]` section in `Cargo.toml` (see Performance).

### Internal — not user-visible

- `chore(fmt)` of the vendored YAML tree (was `crates/serde_yml/`,
  then `crates/yaml_safe/`, now inlined as `src/yaml/`) under
  workspace rustfmt rules. Isolated as its own commit per blame
  hygiene.
- `DEFAULT_SYNTAX_THEME` constant now actually used — three call
  sites that hardcoded `"github"` route through the constant.
- `Cargo.lock` transitive auto-bumps (e.g. `jiff` 0.2.23 → 0.2.24).

### Deferred to a later release

- **`mdx-gen` 0.0.4 upgrade.** Tracked as a separate effort. The
  release added `MarkdownOptions::validate()` (which rejects the
  existing default `syntax_theme = "github"`), native `:::warning`
  recognition that competes with `add_custom_classes`, and a hardened
  sanitiser that strips arbitrary `class` attributes. ~10 test
  rewrites required; out of scope for this release. See the close
  comment on PR #37.

## [0.0.4] — 2026-04-04

First public release on crates.io after a six-month iteration on
unreleased branches.

### Added

- HTML5 semantic element builders (`Article`, `Section`, `Nav`,
  `Aside`, `Template`) with built-in ARIA support (closes #25).
- Bundled emoji data via `include_str!` so emoji-to-ARIA-label
  mapping works without filesystem access. Override path via
  `HTML_GENERATOR_EMOJI_DATA` environment variable retained.
- `ammonia`-backed sanitisation when `allow_unsafe_html=true`
  combined with `sanitize_html=true`.
- Full HTML5 document wrapping (`generate_full_document`) — output
  is wrapped in `<!DOCTYPE html><html lang="…"><head>…</head><body>…</body></html>`
  with title extracted from the first `<h1>`.
- Diagnostics pipeline: `generate_html_with_diagnostics` returns
  `HtmlOutput { html, diagnostics }` so callers can inspect which
  post-processing steps degraded.
- Async generation (`async_generate_html`) behind the `async`
  feature flag.
- DOM-aware `replace_element` (`DomReplacer`) replacing brittle
  string-replacement code.
- Configurable emoji loader, relaxed input path validation,
  `[[TOC]]` placeholder, and unified pipeline.
- 13 branded examples and a complete README rewrite.

### Changed

- Manual front-matter parsers replaced with a single delimiter-aware
  extractor that supports YAML (`---`), TOML (`+++`), and JSON
  (`{ … }`).
- Migrated to `comrak` 0.50 and `minify-html` 0.18.
- Image CDN migrated from `kura.pro` to `cloudcdn.pro`.

## [0.0.3] — 2024-12-29

Iterative comrak compatibility track. Incremental upgrades of the
`comrak` dependency from 0.32 → 0.35 with associated minor fixes.

## [0.0.2] — 2024-12-01

Second pre-release iteration.

## [0.0.1] — 2024-10-07

Initial public release.

[0.0.5]: https://github.com/sebastienrousseau/html-generator/compare/v0.0.4...HEAD
[0.0.4]: https://github.com/sebastienrousseau/html-generator/releases/tag/v0.0.4
[0.0.3]: https://github.com/sebastienrousseau/html-generator/releases/tag/v0.0.3
[0.0.2]: https://github.com/sebastienrousseau/html-generator/releases/tag/v0.0.2
[0.0.1]: https://github.com/sebastienrousseau/html-generator/releases/tag/v0.0.1
