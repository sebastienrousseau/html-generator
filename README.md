<p align="center">
  <img src="https://kura.pro/html-generator/images/logos/html-generator.svg" alt="HTML Generator logo" width="128" />
</p>

<h1 align="center">HTML Generator</h1>

<p align="center">
  <strong>A Rust library for transforming Markdown into SEO-optimized, accessible HTML.</strong>
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/html-generator/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/html-generator/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/html-generator"><img src="https://img.shields.io/crates/v/html-generator.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/html-generator"><img src="https://img.shields.io/badge/docs.rs-html-generator-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://codecov.io/gh/sebastienrousseau/html-generator"><img src="https://img.shields.io/codecov/c/github/sebastienrousseau/html-generator?style=for-the-badge&logo=codecov" alt="Coverage" /></a>
  <a href="https://lib.rs/crates/html-generator"><img src="https://img.shields.io/badge/lib.rs-v0.0.4-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
</p>

---

## Install

```bash
cargo add html-generator
```

Or add to `Cargo.toml`:

```toml
[dependencies]
html-generator = "0.0.4"
```

You need [Rust](https://rustup.rs/) 1.56.0 or later. Works on macOS, Linux, and Windows.

---

## Overview

HTML Generator transforms Markdown into production-ready, SEO-optimized HTML with accessibility compliance built in.

- **Markdown to HTML** with full CommonMark support
- **Front matter extraction** from YAML, TOML, and JSON
- **Automatic table of contents** from document headings
- **WCAG-compliant output** with ARIA attributes

---

## Features

| | |
| :--- | :--- |
| **Markdown to HTML** | Convert Markdown to SEO-optimized HTML |
| **Accessibility** | WCAG-compliant output with ARIA attributes |
| **Front matter** | Extract and process YAML/TOML/JSON front matter |
| **Table of contents** | Automatic TOC generation from headings |
| **Custom headers** | Configurable header ID generation and processing |
| **Performance** | Optimized for large-scale web projects |

---

## Usage

```rust
use html_generator::{generate_html, HtmlConfig};

fn main() {
    let markdown = "# Hello\n\nThis is **bold** text.";
    let config = HtmlConfig::default();
    let html = generate_html(markdown, &config).unwrap();
    println!("{}", html);
}
```

---

## Development

```bash
cargo build        # Build the project
cargo test         # Run all tests
cargo clippy       # Lint with Clippy
cargo fmt          # Format with rustfmt
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, signed commits, and PR guidelines.

---

**THE ARCHITECT** \u1d2b [Sebastien Rousseau](https://sebastienrousseau.com)
**THE ENGINE** \u1d5e [EUXIS](https://euxis.co) \u1d2b Enterprise Unified Execution Intelligence System

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#html-generator">Back to Top</a></p>