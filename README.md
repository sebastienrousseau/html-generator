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
  <a href="https://lib.rs/crates/html-generator"><img src="https://img.shields.io/badge/lib.rs-v0.0.6-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Contents

- [Install](#install) -- Cargo, source
- [Quick Start](#quick-start) -- convert Markdown to HTML in 5 lines
- [Overview](#overview) -- what html-generator does
- [Features](#features) -- capability matrix
- [Library Usage](#library-usage) -- pipeline, front matter, TOC, SEO, accessibility
- [Configuration](#configuration) -- HtmlConfig options
- [Examples](#examples) -- 14 branded examples
- [Performance](#performance) -- comparative benchmarks vs comrak / pulldown-cmark
- [Math and diagrams](#math-and-diagrams) -- LaTeX → MathML, Mermaid passthrough
- [WebAssembly](#webassembly) -- browser, Workers, Edge bindings
- [FAQ](#faq) -- common questions and design decisions
- [Development](#development) -- make targets, CI
- [Security](#security) -- safety guarantees
- [License](#license)

---

## Install

```toml
[dependencies]
html-generator = "0.0.6"
```

### Optional async support

```toml
[dependencies]
html-generator = { version = "0.0.6", features = ["async"] }
```

### Build from source

```bash
git clone https://github.com/sebastienrousseau/html-generator.git
cd html-generator
make          # check + clippy + test
```

Requires **Rust 1.80.0+**. Tested on Linux, macOS, and Windows.

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
| **Examples** | 14 branded examples covering every public surface |
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

Run any example:

```bash
cargo run --example hello
cargo run --example pipeline
cargo run --example accessibility
cargo run --example math_and_diagrams
cargo run --example async --features async
```

---

## Performance

Comparative throughput on the same realistic 8 KB blog payload (Apple
M-series, criterion `--quick`, `[profile.bench]` with `opt-level = 3`
+ fat LTO):

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

## FAQ

<details>
<summary><b>Why this crate over `comrak`, `pulldown-cmark`, or `markdown-it`?</b></summary>

Those are pure CommonMark parsers — they hand you raw HTML. html-generator
is the layer above: parse + ARIA injection + JSON-LD structured data +
table of contents + math + mermaid + minification, all in one call.
The benchmarks in [Performance](#performance) show the trade-off
explicitly. If you only need parse-to-HTML, prefer `pulldown-cmark` (45 µs
on the same payload) and write your own post-processing. If you want a
2026-grade content pipeline that ships WCAG 2.1 + SEO out of the box,
this is it.

</details>

<details>
<summary><b>Is the output really WCAG-compliant?</b></summary>

Yes for the structural conformance items: ARIA labels, roles, landmarks,
heading hierarchy, language declarations. `validate_wcag(html, &config,
None)` returns an `AccessibilityReport` with any remaining issues
(missing alt text, color contrast — which html-generator can't infer
from Markdown). Output passes WCAG 2.1 Levels A and AA out of the box;
Level AAA requires opt-in via `WcagLevel::AAA` in the config because some
AAA criteria (heading-jump strictness, contrast ratio 7.0:1) reject
otherwise-valid documents.

</details>

<details>
<summary><b>How do I render math without a JavaScript bundle?</b></summary>

Set `enable_math: true` (it's behind the `math` feature, on by default).
`$..$` and `$$..$$` LaTeX spans become `<math>...</math>` MathML, which
modern browsers render natively — no MathJax, no KaTeX, no client-side
script tag. Parse errors are encoded inline as `<merror>` markers so
broken LaTeX is visible in the page rather than crashing the build.
Currency-style `$5` is left literal (the matcher requires a non-digit
after the closing `$`).

</details>

<details>
<summary><b>Do I have to manage Mermaid rendering myself?</b></summary>

For Mermaid, yes — html-generator only rewrites the markup so the standard
`mermaid.js` bundle finds it. Set `enable_diagrams: true` and the
pipeline emits `<pre class="mermaid">` instead of
`<pre><code class="language-mermaid">`. Then drop a single
`<script type="module">import mermaid from "https://…/mermaid.esm.mjs";
mermaid.initialize({startOnLoad:true});</script>` in your page.
Server-side mermaid rendering would require running a headless browser
or porting the diagram engine to Rust — out of scope for this crate.

</details>

<details>
<summary><b>Can I run this in Cloudflare Workers / Vercel Edge / a browser?</b></summary>

Yes — `wasm-pack build --release --target web --no-default-features
--features wasm,math` produces a 5.8 MB raw / 2.0 MB gzipped bundle plus
~13 KB of JS bindings. The exposed JS surface is `generateHtml`,
`generateHtmlFullDocument`, and `generateHtmlWithOptions(markdown,
optionsJson)`. Workers' paid plan allows 10 MB compressed scripts,
fitting comfortably; the free tier (1 MB compressed) requires further
trimming and is not currently a supported configuration.

</details>

<details>
<summary><b>What's missing on the WASM target compared to native?</b></summary>

Three things, all from `mdx-gen`'s extension layer (which doesn't compile
to `wasm32-unknown-unknown` because of an unconditional `tokio` dep):
`:::class` custom blocks, image-class syntax (`![alt](url).class="…"`),
and `syntect` syntax highlighting. CommonMark + GFM (tables,
strikethrough, autolinks, tasklists, superscript) plus the full ARIA /
TOC / JSON-LD / math / mermaid post-processing renders identically.

</details>

<details>
<summary><b>Why is raw HTML in Markdown stripped by default?</b></summary>

Untrusted Markdown that contains raw `<script>` tags is an XSS vector.
`HtmlConfig::default()` sets `allow_unsafe_html = false` so `<script>` and
friends never make it to the output. If you control the Markdown source
(e.g. site authors you trust), set `allow_unsafe_html = true`. For
user-submitted Markdown, set both `allow_unsafe_html = true` and
`sanitize_html = true` — the pipeline runs `ammonia` over the final HTML
to strip dangerous elements while keeping safe ones.

</details>

<details>
<summary><b>Why does the same Markdown produce identical HTML on every run now?</b></summary>

Earlier versions (≤ 0.0.4) used `uuid::Uuid::new_v4()` for
auto-generated ARIA IDs, so two runs over the same input produced
different HTML — bad for content-addressable caching, deterministic
builds, and snapshot testing. v0.0.5 replaced UUIDs with per-call
counters so byte-identical input produces byte-identical output. The
`uuid` runtime dependency was dropped in the same commit.

</details>

<details>
<summary><b>How does the pipeline handle errors gracefully?</b></summary>

Use `generate_html_with_diagnostics` instead of `generate_html`. It
returns an `HtmlOutput` with `html: String` and `diagnostics:
Vec<Diagnostic>`. Each diagnostic records which pipeline step
(`accessibility`, `toc`, `structured_data`, `minification`, etc.)
emitted it and at what severity. Non-fatal failures degrade rather than
abort — e.g., if ARIA injection fails on malformed HTML the unenhanced
HTML is returned with an `Error`-level diagnostic, and the rest of the
pipeline continues.

</details>

<details>
<summary><b>What's `src/yaml/`? It's massive.</b></summary>

A vendored, pure-Rust YAML parser kept verbatim from upstream
(`yaml_safe@0.1.0`, in turn a fork-and-rename of `serde_yml` away from
the unsound `libyml` C dependency). It exists as a private `mod yaml`
inside the crate (~2 700 lines) so the crate compiles without taking
on the unsound `serde_yml` registry dependency or its
`RUSTSEC-2025-0068` advisory. Excluded from coverage and clippy in CI;
not part of the public API surface. Will be replaced with the
crates.io-published `yaml_safe = "0.1"` registry dependency once that
ships.

</details>

<details>
<summary><b>Is `cargo publish` supported?</b></summary>

Yes — `cargo publish --dry-run` succeeds as of v0.0.5. The earlier
blocker (path-only `crates/yaml_safe/` without a `version =` field) was
closed by inlining the YAML implementation into `src/yaml/`.

</details>

<details>
<summary><b>What's the MSRV policy?</b></summary>

Rust 1.80.0 is the floor. Bumps require a minor-version increment and
a CHANGELOG entry. Linting and formatting follow the latest stable
(`cargo fmt --all -- --check` and `cargo clippy -- -D warnings` are
expected to pass on the toolchain pinned in `mise.toml`/the CI config).

</details>

---

## Development

```bash
make              # check + clippy + test
make build        # cargo build
make test         # run all tests
make lint         # clippy with strict flags
make format       # rustfmt
make deny         # supply-chain audit
make outdated     # dependency freshness check
make help         # list all targets
```

### CI

| Workflow | Trigger | Purpose |
| :--- | :--- | :--- |
| `ci.yml` | push, PR | Clippy, fmt, test (all features), coverage, audit |
| `docs.yml` | push to main | Build and deploy API docs to GitHub Pages |
| `security.yml` | push, PR | Dependency review, CodeQL, cargo-audit, cargo-deny |

See [CONTRIBUTING.md](CONTRIBUTING.md) for signed commits and PR guidelines.

---

## Security

- `#![forbid(unsafe_code)]` at crate root and in `Cargo.toml` lints
- Raw HTML stripped by default — opt-in via `allow_unsafe_html: true`
- All user-controlled attributes escaped via `escape_html`
- Directory traversal (`..`) blocked in file path validation
- Input size limits enforced at all boundaries
- `cargo audit` clean (transitive advisory ignores documented in `.cargo/audit.toml`)
- `cargo deny` -- license, advisory, and ban checks
- SPDX license headers on all source files
- Signed commits enforced via CI

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#contents">Back to Top</a></p>
