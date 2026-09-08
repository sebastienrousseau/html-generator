<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/html-generator/v1/logos/html-generator.svg" alt="HTML Generator logo" width="128" />
</p>

<h1 align="center">html-generator</h1>

<p align="center">
  <strong>Pure Rust library for transforming Markdown into SEO-optimized, accessible HTML. Zero unsafe code.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/html-generator/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/html-generator/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/html-generator"><img src="https://img.shields.io/crates/v/html-generator.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/html-generator"><img src="https://img.shields.io/badge/docs.rs-html--generator-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/html-generator"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/html-generator?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://www.bestpractices.dev/projects/14537"><img src="https://img.shields.io/cii/level/14537?style=for-the-badge&label=OpenSSF%20Best%20Practices&logo=openssf" alt="OpenSSF Best Practices" /></a>
  <a href="https://lib.rs/crates/html-generator"><img src="https://img.shields.io/badge/lib.rs-html--generator-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) — Cargo, source, optional features
- [Requirements](#requirements) — toolchain floor, platforms
- [Quick Start](#quick-start) — Markdown to accessible HTML in five lines

**Library reference**

- [Overview](#overview) — what the pipeline does, step by step
- [Features](#features) — capability matrix
- [Library Usage](#library-usage) — pipeline, front matter, TOC, SEO, accessibility
- [Configuration](#configuration) — every `HtmlConfig` option
- [Math and diagrams](#math-and-diagrams) — LaTeX to MathML, Mermaid passthrough
- [WebAssembly](#webassembly) — browser, Workers and Edge bindings
- [Benchmarks](#benchmarks) — measured numbers with the host stated
- [Examples](#examples) — runnable example index

**Operational**

- [When not to use html-generator](#when-not-to-use-html-generator) — limitations
- [Development](#development) — make targets, fuzzing, Miri, CI
- [Security](#security) — hardening, fuzzing, supply chain
- [Documentation](#documentation) — all reference docs
- [Stability guarantees](#stability-guarantees) — SemVer axis, output stability
- [Minimum-toolchain policy](#minimum-toolchain-policy)
- [License](#license)

---

## Install

```toml
[dependencies]
html-generator = "0.0.11"
```

### Optional async support

```toml
[dependencies]
html-generator = { version = "0.0.11", features = ["async"] }
```

### Build from source

```bash
git clone https://github.com/sebastienrousseau/html-generator.git
cd html-generator
make          # check + clippy + test
```

Requires **Rust 1.80.0+**. Tested on Linux, macOS, and Windows.

---

## Requirements

- **Rust 1.80.0 or newer.** `rust-version` in `Cargo.toml` is the floor
  and Cargo enforces it; CI builds on stable across Linux, macOS and
  Windows. See the [minimum-toolchain policy](#minimum-toolchain-policy)
  for when and why it may move.
- **A `std` platform**, or `wasm32` with the `wasm` feature. The crate
  uses `std` unconditionally; there is no `no_std` build.
- **No async runtime is required.** Every entry point is synchronous
  unless you enable the `async` feature, which adds a Tokio-based
  wrapper for callers who already run one.

---

## Quick Start

```rust
use html_generator::{generate_html, HtmlConfig};

fn main() -> Result<(), html_generator::error::HtmlError> {
    let markdown = "# Hello\n\nThis is **bold** text.";
    let config = HtmlConfig::default();
    let html = generate_html(markdown, &config)?;
    println!("{html}");
    Ok(())
}
```

---

## Overview

html-generator converts Markdown into production-ready HTML with a configurable pipeline that applies accessibility, SEO, table of contents, math, diagrams, and minification in a single pass. No raw HTML passthrough by default — safe for untrusted input. Runs natively or as WebAssembly in browsers, Cloudflare Workers, and edge runtimes.

- **Full CommonMark** with extensions (tables, strikethrough, task lists, superscript)
- **Front matter extraction** from YAML (`---`), TOML (`+++`), and JSON (`{...}`)
- **WCAG-compliant output** with automatic ARIA attribute injection
- **JSON-LD structured data** appended for rich search results
- **Table of contents** injected at `[[TOC]]` placeholder
- **Server-side LaTeX → MathML** for `$..$` and `$$..$$` (no client-side JS needed)
- **Mermaid diagram passthrough** for `\u{60}\u{60}\u{60}mermaid` fenced blocks
- **In-memory minification** without disk I/O
- **WebAssembly bindings** via `wasm-bindgen` (browsers, Workers, Edge)
- **Optional async** via tokio `spawn_blocking` (behind `async` feature)
- **Zero unsafe code** via `#![forbid(unsafe_code)]` at crate root

| Metric | Value |
| :--- | :--- |
| **Source** | ~12,900 lines across 11 modules (`src/yaml/` is a vendored snapshot, see FAQ) |
| **Test suite** | 533 unit/integration tests + 163 doctests + 4 WASM smoke tests = **700 total** |
| **Coverage** | 98.18% line coverage (`cargo llvm-cov`); Codecov project ≥95%, patch ≥90% gates |
| **Examples** | 14 runnable examples, all executed in CI |
| **Dependencies** | 13 native runtime + 1 optional async (`tokio`) + 2 optional WASM (`wasm-bindgen`, `js-sys`) |
| **MSRV** | Rust 1.80.0 |
| **WASM bundle** | 5.8 MB raw / **2.0 MB gzipped** (after `wasm-opt -Os`) |
| **CI gates** | 10 distinct checks including end-to-end `wasm-pack test --node` against Node 20 |

---

## Features

| | |
| :--- | :--- |
| **Markdown to HTML** | Full CommonMark via mdx-gen with extensions: tables, strikethrough, task lists, autolinks, superscript. Custom class blocks via `:::class` syntax. Image class attributes via `![alt](url).class="cls"`. |
| **Accessibility** | Automatic ARIA attribute injection for buttons, navs, forms, inputs, tabs, modals, accordions, tooltips. WCAG 2.1 validation (Levels A, AA, AAA). Heading structure checks. Language attribute validation. |
| **Front matter** | YAML (`---`), TOML (`+++`), JSON (`{...}`) delimiters. `extract_front_matter` strips metadata and returns body. `extract_front_matter_data` parses metadata into `serde_json::Value`. |
| **Table of contents** | `generate_table_of_contents` builds `<ul>` from headings. Pipeline injects at `[[TOC]]` placeholder when `generate_toc` is enabled. |
| **SEO** | `MetaTagsBuilder` for meta tag generation. `generate_structured_data` for JSON-LD `<script>` output with configurable `@type` and additional properties. HTML entity escaping via `escape_html`. |
| **Math (MathML)** | `enable_math` flag converts `$..$` and `$$..$$` LaTeX spans to native `<math>` MathML via `pulldown-latex`. Server-side, no JS bundle. Conservative regex matchers leave `$5` currency literals alone. Behind the `math` feature (default-on). |
| **Diagrams (Mermaid)** | `enable_diagrams` flag rewrites `\u{60}\u{60}\u{60}mermaid` fenced blocks to `<pre class="mermaid">` for the standard client-side mermaid.js bundle. Diagram source flows through verbatim. |
| **Minification** | File-based `minify_html(path)` and in-memory `minify_html_string(html)`. Preserves HTML semantics, strips comments, minifies CSS/JS. Configurable via `MinifyConfig`. |
| **WebAssembly** | `wasm` feature exposes `generateHtml`, `generateHtmlFullDocument`, `generateHtmlWithOptions` to JavaScript via `wasm-bindgen`. Build with `wasm-pack build --target web --features wasm --no-default-features`. |
| **Performance** | Regexes and CSS selectors compiled once into `static Lazy`. SIMD-backed `str::contains` short-circuits before any html5ever parse. DOM-aware element replacement handles attribute reordering. **2.09 ms** full pipeline on an 8 KB blog payload (`comrak` parse alone is 172 µs). |
| **Async** | Optional `async` feature enables `async_generate_html` via tokio `spawn_blocking`. Synchronous users pay zero cost — tokio not compiled without the feature. |
| **Security** | `#![forbid(unsafe_code)]`. Raw HTML stripped by default (`allow_unsafe_html: false`). All user-controlled attributes escaped. NUL-byte rejection on file paths. Directory traversal blocked. Input size limits enforced. |

---

## Library Usage

<details>
<summary><b>Full pipeline</b></summary>

```rust
use html_generator::{generate_html, HtmlConfig};

let config = HtmlConfig {
    add_aria_attributes: true,
    generate_toc: true,
    generate_structured_data: true,
    minify_output: true,
    ..HtmlConfig::default()
};

let markdown = "[[TOC]]\n\n# Introduction\n\nWelcome to the guide.\n\n## Getting Started\n\nFollow these steps.";
let html = generate_html(markdown, &config)?;
// Output includes: ARIA attributes, TOC at [[TOC]], JSON-LD, minified
# Ok::<(), html_generator::error::HtmlError>(())
```

The pipeline applies steps in order:
1. Markdown → HTML (with extensions)
2. Accessibility (ARIA attributes)
3. Table of contents (inject at `[[TOC]]`)
4. Structured data (append JSON-LD)
5. Minification (compress)

</details>

<details>
<summary><b>Front matter</b></summary>

```rust
use html_generator::utils::extract_front_matter_data;

// YAML front matter
let content = "---\ntitle: My Page\nauthor: Jane Doe\n---\n# Hello";
let (metadata, body) = extract_front_matter_data(content)?;
assert_eq!(metadata["title"], "My Page");
assert_eq!(body, "# Hello");

// TOML front matter
let content = "+++\ntitle = \"My Page\"\nauthor = \"Jane Doe\"\n+++\n# Hello";
let (metadata, body) = extract_front_matter_data(content)?;
assert_eq!(metadata["title"], "My Page");

// JSON front matter
let content = "{\"title\": \"My Page\"}\n# Hello";
let (metadata, body) = extract_front_matter_data(content)?;
assert_eq!(metadata["title"], "My Page");
# Ok::<(), html_generator::error::HtmlError>(())
```

</details>

<details>
<summary><b>Table of contents</b></summary>

```rust
use html_generator::{generate_html, HtmlConfig};

let markdown = "[[TOC]]\n\n# Chapter 1\n\n## Section 1.1\n\n# Chapter 2";
let config = HtmlConfig {
    generate_toc: true,
    ..HtmlConfig::default()
};
let html = generate_html(markdown, &config)?;
assert!(html.contains(r#"<ul>"#));
assert!(html.contains(r#"<a href="\#chapter-1">"#));
# Ok::<(), html_generator::error::HtmlError>(())
```

</details>

<details>
<summary><b>SEO and structured data</b></summary>

```rust
use html_generator::seo::{MetaTagsBuilder, generate_structured_data, StructuredDataConfig};
use std::collections::HashMap;

// Meta tags
let meta = MetaTagsBuilder::new()
    .with_title("My Page")
    .with_description("A great page")
    .add_meta_tag("author", "Jane Doe")
    .build()?;

// JSON-LD structured data
let html = r#"<html><head><title>My Page</title></head><body><p>Content</p></body></html>"#;
let config = StructuredDataConfig {
    page_type: "Article".to_string(),
    additional_data: Some(HashMap::from([("author".to_string(), "Jane".to_string())])),
    ..Default::default()
};
let json_ld = generate_structured_data(html, Some(config))?;
assert!(json_ld.contains("application/ld+json"));
# Ok::<(), html_generator::error::HtmlError>(())
```

</details>

<details>
<summary><b>Accessibility</b></summary>

```rust
use html_generator::accessibility::{add_aria_attributes, validate_wcag, AccessibilityConfig};

let html = r#"<button>Submit</button><nav><ul><li>Home</li></ul></nav>"#;

// Enhance with ARIA attributes
let enhanced = add_aria_attributes(html, None)?;
assert!(enhanced.contains("aria-label"));

// Validate WCAG compliance
let config = AccessibilityConfig::default();
let report = validate_wcag(&enhanced, &config, None)?;
println!("Issues found: {}", report.issue_count);
# Ok::<(), html_generator::accessibility::Error>(())
```

</details>

<details>
<summary><b>Minification</b></summary>

```rust
use html_generator::performance::minify_html_string;

let html = "<html>  <body>  <p>Hello</p>  </body>  </html>";
let minified = minify_html_string(html)?;
assert_eq!(minified, "<html><body><p>Hello</p></body></html>");
# Ok::<(), html_generator::error::HtmlError>(())
```

</details>

<details>
<summary><b>Diagnostics</b></summary>

The default `generate_html` silently degrades when optional steps fail.
Use `generate_html_with_diagnostics` to inspect which steps succeeded:

```rust
use html_generator::{generate_html_with_diagnostics, HtmlConfig};

let config = HtmlConfig {
    add_aria_attributes: true,
    generate_toc: true,
    generate_structured_data: true,
    minify_output: true,
    ..HtmlConfig::default()
};

let output = generate_html_with_diagnostics("# Hello", &config)?;
println!("HTML: {} bytes", output.html.len());
for d in &output.diagnostics {
    eprintln!("warning: {d}");
}
# Ok::<(), html_generator::error::HtmlError>(())
```

</details>

<details>
<summary><b>Async (optional)</b></summary>

Enable with `features = ["async"]`:

```rust,ignore
use html_generator::performance::async_generate_html;

#[tokio::main]
async fn main() -> Result<(), html_generator::error::HtmlError> {
    let html = async_generate_html("# Hello\n\nWorld").await?;
    println!("{html}");
    Ok(())
}
```

</details>

---

## Configuration

```rust
use html_generator::HtmlConfig;

let config = HtmlConfig {
    enable_syntax_highlighting: true,       // Syntax-highlighted code blocks
    syntax_theme: Some("github".into()),    // Highlighting theme
    minify_output: false,                   // Compress output HTML
    add_aria_attributes: true,              // Inject ARIA attributes
    generate_structured_data: false,        // Append JSON-LD
    generate_toc: false,                    // Inject TOC at [[TOC]]
    allow_unsafe_html: false,               // Strip raw HTML (XSS-safe default)
    sanitize_html: false,                   // Sanitize via ammonia (when unsafe is on)
    generate_full_document: false,          // Wrap in HTML5 boilerplate
    max_input_size: 5 * 1024 * 1024,        // 5MB input limit
    max_buffer_size: 16 * 1024 * 1024,      // 16MB I/O buffer
    language: "en-GB".into(),               // Content language (used in html lang attr)
    encoding: "utf-8".into(),               // File I/O encoding
    enable_math: false,                     // LaTeX → MathML for $..$ / $$..$$
    enable_diagrams: false,                 // Mermaid passthrough for ```mermaid blocks
};
```

Use the builder for validated configuration:

```rust
use html_generator::HtmlConfig;

let config = HtmlConfig::builder()
    .with_syntax_highlighting(true, Some("monokai".into()))
    .with_language("en-US")
    .build()?;
# Ok::<(), html_generator::error::HtmlError>(())
```

---

## Examples

| Example | Description |
| :--- | :--- |
| `hello` | Heading, lists, code blocks, links — basic Markdown to HTML |
| `pipeline` | Full pipeline: ARIA + TOC + JSON-LD + minification in one pass |
| `frontmatter` | YAML, TOML, JSON front matter extraction and parsing |
| `accessibility` | ARIA injection for buttons, navs, forms; WCAG validation |
| `seo` | Meta tags, JSON-LD structured data, HTML entity escaping |
| `toc` | Table of contents from headings, `[[TOC]]` placeholder |
| `minify` | In-memory HTML minification with size savings |
| `errors` | Error variants, type matching, graceful recovery patterns |
| `config` | HtmlConfig builder, validation, field inspection |
| `headers` | Custom ID and class generators for heading elements |
| `custom_syntax` | Triple-colon blocks (`:::warning`) and image classes |
| `emojis` | Bundled emoji data, emoji-to-ARIA-label mapping |
| `math_and_diagrams` | LaTeX → MathML and `\u{60}\u{60}\u{60}mermaid` passthrough |
| `async` | Asynchronous generation via tokio (requires `--features async`) |

Run any of them with `cargo run --example <name>`, or all fourteen with
`make examples`. CI does the same on every push, so an example that
stops working fails the build.

```bash
cargo run --example hello
cargo run --example async --features async
```

---

## Benchmarks

Comparative throughput on the same realistic 8 KB blog payload,
measured with Criterion (`--quick`) on an Apple M-series CPU with
`[profile.bench]` at `opt-level = 3` and fat LTO. Numbers are worth
nothing without the host they came from; reproduce them on yours with
`cargo bench --bench competitors`.

| Engine | Time / iter | What it does |
| :--- | ---: | :--- |
| `pulldown_cmark` (parse only) | **45 µs** | Pull-parser, no post-processing. Fastest plain CommonMark in Rust. |
| `comrak` (parse only) | **172 µs** | The CommonMark/GFM parser this crate wraps. |
| `html_generator` (full pipeline) | **2.09 ms** | Parse + ARIA injection + TOC + JSON-LD + minification. |

Pure parsers will always be faster — they don't do ARIA, JSON-LD, TOC, or
minification. `html-generator` does all four in one pass; the ~2 ms
overhead is what buys WCAG-compliant output without a downstream
post-processing layer. Reproduce with:

```bash
cargo bench --bench competitors
```

---

## Math and diagrams

Two opt-in post-processors turn ordinary Markdown into rich technical
documentation without client-side JavaScript for math:

```rust
use html_generator::{generate_html, HtmlConfig};

let md = r"
# Pythagoras

In a right triangle, $a^2 + b^2 = c^2$.

```mermaid
graph LR
    A --> B
```";
let cfg = HtmlConfig {
    enable_math: true,        // $..$ and $$..$$ → <math> MathML
    enable_diagrams: true,    // ```mermaid → <pre class="mermaid">
    ..HtmlConfig::default()
};
let html = generate_html(md, &cfg)?;
# Ok::<(), html_generator::error::HtmlError>(())
```

* **Math** — server-side LaTeX → MathML via `pulldown-latex` (gated
  behind the `math` feature, on by default). Browsers render MathML
  natively, so no client-side bundle is required. Parse errors are
  encoded inline as `<merror>` markers rather than crashing the build.
* **Diagrams** — `\u{60}\u{60}\u{60}mermaid` fenced blocks become
  `<pre class="mermaid">…</pre>` so the standard mermaid.js loader picks
  them up. Drop a single
  `<script type="module">import mermaid from "https://…/mermaid.esm.mjs"; mermaid.initialize({startOnLoad:true});</script>`
  in your page and you're done.

---

## WebAssembly

The same pipeline runs in Cloudflare Workers, Vercel Edge, browsers, and
Node — without changing API:

```bash
cargo build --release --target wasm32-unknown-unknown \
  --features wasm --no-default-features
# or, to publish an npm bundle:
wasm-pack build --target web --features wasm --no-default-features
```

Three JS-friendly entry points are exposed via `wasm-bindgen`:

| JS name | Description |
| :--- | :--- |
| `generateHtml(markdown)` | Render Markdown to an accessible HTML fragment with default config. |
| `generateHtmlFullDocument(markdown)` | Same but wrapped in `<!DOCTYPE html><html>…</html>`. |
| `generateHtmlWithOptions(markdown, optionsJson)` | Pass a JSON object configuring `add_aria_attributes`, `generate_toc`, `enable_math`, `enable_diagrams`, etc. |

WASM builds drop `mdx-gen`'s `:::class`, image-class, and `syntect`
syntax highlighting (the underlying `tokio`/`onig` C dependencies do not
compile to `wasm32-unknown-unknown`); CommonMark + GFM (tables,
strikethrough, autolinks, tasklists, superscript) plus the full ARIA /
TOC / JSON-LD / math / mermaid post-processing layer renders identically.

Use it from JavaScript:

```javascript
// pkg/ generated by `wasm-pack build --target web ...`
import init, {
  generateHtml,
  generateHtmlWithOptions,
} from "./pkg/html_generator.js";

await init();

// Simple render with defaults (ARIA on):
const fragment = generateHtml("# Hello, **world**!");

// Render with custom options:
const article = generateHtmlWithOptions(
  "# Math\n\n$$E = mc^2$$",
  JSON.stringify({
    enable_math: true,
    generate_full_document: true,
    language: "en-GB",
  }),
);
```

From Cloudflare Workers / Vercel Edge: use `wasm-pack build --target
bundler` and import the generated module from your worker entry point.
The JS-side API is identical to the browser case.

### Bundle size

Measured `wasm-pack build --release --target web` output, post
`wasm-opt -Os`:

| Feature set | `.wasm` raw | `.wasm` gzipped |
| :--- | ---: | ---: |
| `--features wasm,math` | 5.8 MB | **2.0 MB** |
| `--features wasm` (no math) | 5.7 MB | **1.96 MB** |

The `math` feature adds ~40 KB gzipped. Both bundles fit comfortably
in Cloudflare Workers' paid plan (10 MB compressed); the free plan
(1 MB compressed) requires further trimming — the `ammonia`,
`minify-html`, and `scraper`-on-`html5ever` deps account for the bulk
of the binary.

Smoke tests live in [`tests/wasm_smoke.rs`](tests/wasm_smoke.rs) and run
under `wasm-pack test --node --no-default-features --features wasm,math`.
The CI's `wasm-build` job exercises this exact command on every push.

---

## When not to use html-generator

Cases where something else fits better, listed because the honest answer
is "not yet" or "by design" rather than a disagreement about priorities.

- **You only need Markdown to HTML.** Use `comrak` or
  `pulldown-cmark` directly. They are the parsers underneath, they are
  an order of magnitude faster, and everything this crate adds on top is
  overhead you would not be using.
- **You need `no_std`.** The crate uses `std` unconditionally. The
  `wasm32` build is the only non-native target it supports, and it still
  needs `std`.
- **You need a full HTML parser's error recovery.** The accessibility
  and SEO passes read the generated document with `scraper`; they are
  built for markup this crate produced, not for arbitrary broken HTML
  from the wild.
- **You need WCAG conformance as a legal guarantee.** `validate_wcag`
  checks the rules it implements: heading order, image alt text, form
  labels, landmark structure, link text. Conformance is a property of a
  whole site and its content, and no library can certify it for you.
- **You want raw HTML in Markdown to pass through untouched by
  default.** It does not, and that is deliberate. See
  [Security](#security).

---

## Development

```bash
make              # check + clippy + test
make test         # all tests, all features
make clippy       # lints, warnings denied
make fmt          # formatting check
make lint         # markdownlint + codespell + REUSE
make doc          # rustdoc with warnings denied
make coverage     # line coverage gate (98%, excluding src/wasm.rs)
make miri         # lib tests under Miri
make fuzz         # build every target, replay corpus and regressions
make examples     # run all fourteen examples
make bench-smoke  # compile and run each bench once
make versions     # every version-bearing file agrees
make deny / vet / audit   # supply chain
```

[`DEVELOPMENT.md`](DEVELOPMENT.md) maps each CI job to its local
equivalent and explains the gotchas.

### Fuzzing

Three `cargo-fuzz` targets live under `fuzz/fuzz_targets/`:

```bash
cargo +nightly fuzz run fuzz_markdown       # the whole pipeline, every step on
cargo +nightly fuzz run fuzz_front_matter   # extraction
cargo +nightly fuzz run fuzz_accessibility  # ARIA enrichment + WCAG validation
```

`fuzz_accessibility` is the one that matters most: the enrichment pass
rewrites markup by byte offset, which is where this crate's
slice-boundary bugs have lived. `fuzz/corpus/<target>` holds the
committed seeds and `fuzz/regressions/<target>` every fixed-bug input;
both replay on each push, so a fixed crash cannot silently return.

cargo-fuzz must be **installed from source** (`cargo install --locked
cargo-fuzz`): the prebuilt binary is a musl build and infers its own
build triple as the fuzz target.

### Miri

The crate is `#![forbid(unsafe_code)]`, so Miri does not police its own
code. The job exists to check the interaction with dependencies that do
use `unsafe` internally.

```bash
make miri     # cargo +nightly miri test --lib
```

### CI

| Workflow | Trigger | Purpose |
| :--- | :--- | :--- |
| `ci.yml` | push, PR | clippy, fmt, tests across three OSes, coverage, audit |
| `quality.yml` | push, PR | coverage gate, Miri, fuzz replay, docs lint, cargo-vet ratchet, release hygiene |
| `docs.yml` | push to main | build and deploy API docs to GitHub Pages |
| `release.yml` | tag `v*` | validate, build, GitHub Release |

See [CONTRIBUTING.md](CONTRIBUTING.md) for signed commits and PR
guidelines.

---

## Security

**Reporting:** never open a public issue for a vulnerability. See
[`SECURITY.md`](SECURITY.md) for the private channel and disclosure
policy.

This crate turns untrusted Markdown into HTML a site will serve, so
injection is the first-order risk and the reason for most of what
follows.

### Injection

Raw HTML in Markdown is **off by default**: `allow_unsafe_html` is
`false`, so embedded markup is escaped and nothing a document author
writes reaches the page as live HTML. `sanitize_html` is a **separate
switch, also off by default**, and has no effect on its own: it runs the
output through `ammonia`, an allow-list sanitiser, only when
`allow_unsafe_html` is `true`. If you enable one, enable both. Every
user-controlled attribute value is escaped on the way out.

### Architectural posture

- `#![forbid(unsafe_code)]` — the compiler proves the absence of unsafe
  blocks.
- No C dependencies, no FFI, no network I/O. File access happens only
  where the caller names a path, and `..` traversal is rejected.
- Input size limits at every boundary: a per-call `max_input_size`
  (5 MiB by default), a 1 MB cap on the HTML the accessibility and SEO
  passes will rewrite, and a 16 MiB reader buffer.

### Supply chain

- `cargo-deny` and `cargo-audit` in CI; documented advisory exemptions
  live in `.cargo/audit.toml` with the upstream reason for each.
- `cargo-vet` provenance in `supply-chain/`, with an exemption baseline
  the CI ratchet cannot exceed.
- `noyalib` pinned exactly (`=0.0.X`); a bump is a deliberate release.
- `Cargo.lock` committed; CI builds `--locked`. Actions pinned by SHA.
- REUSE 3.3 compliant, linted in CI.
- Commits on `main` are signed; releases are signed tags
  ([`KEYS.asc`](KEYS.asc)).

---

## Documentation

The four entry points, identical across every repo in the family:

- **[API reference](https://docs.rs/html-generator)** — rustdoc on docs.rs
- **[Developer docs](DEVELOPMENT.md)** — toolchain, task map, reproducing
  every CI gate locally
- **[Architecture](docs/ARCHITECTURE.md)** — module map, pipeline, design
  decisions
- **[Decision records](docs/adr/README.md)** — the choices that would be
  expensive to reverse

| Document | Covers |
|---|---|
| [`CHANGELOG.md`](CHANGELOG.md) | per-release notes, Keep a Changelog format |
| [`SECURITY.md`](SECURITY.md) | disclosure policy, injection posture, resource limits, supply chain |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | branch and commit conventions, PR expectations, code standards |
| [`GOVERNANCE.md`](GOVERNANCE.md) | who decides what, how changes land |
| [`SUPPORT.md`](SUPPORT.md) | where to ask, what to expect |
| [`AGENTS.md`](AGENTS.md) | invariants for AI-assisted contributions |

---

## Stability guarantees

- **Versioning.** [SemVer](https://semver.org), with the pre-1.0 posture
  that the patch number is the breaking axis during `0.0.x`. Releases
  increment by `+0.0.1` and every breaking change is called out in
  [`CHANGELOG.md`](CHANGELOG.md).
- **Output stability.** For a generator, output *is* API: a change to
  the HTML produced for a given Markdown input — the ARIA attributes
  added, the heading ids emitted, the JSON-LD shape, what the minifier
  collapses — is treated as breaking even when no Rust signature moves.
- **Determinism.** The same Markdown and the same `HtmlConfig` produce
  byte-identical HTML on every run and every platform. Anything else is
  a bug, not a tolerance.
- **Deprecations** live for at least two releases with a `#[deprecated]`
  note naming the replacement before removal.
- **Version-bearing files** are checked against the manifest by
  `scripts/verify-release-versions.sh` before a tag exists, so an
  install snippet cannot go stale.

---

## Minimum-toolchain policy

The floor is **Rust 1.80.0**, declared as `rust-version` in
`Cargo.toml` so Cargo refuses older toolchains with a clear message.

- **When it may rise:** only on a release, never silently, and always
  with the reason in the changelog entry.
- **Why it is where it is:** the floor follows the highest requirement
  in the dependency graph, not an aspiration. It moves when a dependency
  the crate needs moves it.
- **What is verified:** CI builds and tests on stable. The floor is the
  version Cargo enforces from the manifest.

No claim is made about distro-LTS toolchains. Making one would require a
table mapping current distro versions to this floor, and an aspirational
claim there is worse than none.

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#contents">Back to Top</a></p>
