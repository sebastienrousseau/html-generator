
<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/html-generator/images/logos/html-generator.svg"
alt="HTML Generator logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# HTML Generator (html-generator)

A comprehensive Rust library for transforming Markdown into optimised, accessible HTML.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview 🎯

The `html-generator` library simplifies the process of transforming Markdown into SEO-optimised, accessible HTML. This library provides tools for processing front matter, generating semantic headers, validating accessibility, and optimising performance for modern web applications.

## Features ✨

### Markdown to HTML Conversion

- **Standard and Custom Extensions**: Supports GFM and extensible custom syntax.
- **Front Matter Parsing**: Processes YAML/TOML/JSON front matter seamlessly.
- **Header Customisation**: Generates semantic headers with custom IDs and classes.

### SEO and Accessibility

- **SEO Utilities**: Automatically generates meta tags and JSON-LD structured data.
- **Accessibility Enhancements**: Validates against WCAG standards and supports ARIA attributes.
- **Semantic HTML**: Ensures well-structured, readable markup.

### Performance Optimisations

- **Asynchronous Processing**: Handles large documents efficiently with async support.
- **HTML Minification**: Reduces file sizes while maintaining functionality.
- **Lightweight**: Optimised for minimal memory usage and fast execution.

### Developer-Friendly

- **Configurable API**: Extensively configurable options for flexible use cases.
- **Detailed Errors**: Comprehensive error types for easier debugging.
- **Rich Documentation**: Includes examples and detailed usage guides.

## Installation 🚀

Add the following to your `Cargo.toml`:

```toml
[dependencies]
html-generator = "0.0.3"
```

## Usage 💻

### Basic Example

```rust
use html_generator::{generate_html, HtmlConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = HtmlConfig::default();

    let markdown = "# Welcome to HTML Generator

This library makes HTML creation effortless.";
    let html = generate_html(markdown, &config)?;

    println!("Generated HTML:
{}", html);
    Ok(())
}
```

### Advanced Example

```rust
use html_generator::{
    accessibility::validate_wcag,
    seo::{generate_meta_tags, generate_structured_data},
    HtmlConfig,
};

async fn advanced_example() -> Result<String, Box<dyn std::error::Error>> {
    let config = HtmlConfig::builder()
        .with_language("en-GB")
        .with_syntax_highlighting(true, Some("dracula".to_string()))
        .build()?;

    let markdown = "# Advanced Example

Features include syntax highlighting and WCAG validation.";
    let html = generate_html(markdown, &config)?;

    validate_wcag(&html, &config, None)?;
    let meta_tags = generate_meta_tags(&html)?;
    let structured_data = generate_structured_data(&html, None)?;

    Ok(format!("{}
{}
{}", meta_tags, structured_data, html))
}
```

## Examples 💡

Run examples from the repository:

```bash
git clone https://github.com/sebastienrousseau/html-generator.git
cd html-generator
cargo run --example basic
```

## Documentation 📚

- [API Documentation][04]: Detailed function and struct definitions.
- [Example Code](https://github.com/sebastienrousseau/html-generator/tree/main/examples): Practical, real-world use cases.

## Contributing 🤝

We welcome contributions of all kinds! Please read our [Contributing Guidelines][05] for instructions on:

- Reporting issues
- Requesting features
- Submitting code

## License 📜

This project is licensed under either of the following at your choice:

- [Apache License, Version 2.0][10]
- [MIT license][11]

## Acknowledgements 🙏

Heartfelt thanks to all contributors who have supported the development of `html-generator`.

[00]: https://html-generator.co
[01]: https://lib.rs/crates/html-generator
[02]: https://github.com/sebastienrousseau/html-generator/issues
[03]: https://crates.io/crates/html-generator
[04]: https://docs.rs/html-generator
[05]: https://github.com/sebastienrousseau/html-generator/blob/main/CONTRIBUTING.md
[06]: https://codecov.io/gh/sebastienrousseau/html-generator
[07]: https://github.com/sebastienrousseau/html-generator/actions?query=branch%3Amain
[08]: https://www.rust-lang.org/
[09]: https://github.com/sebastienrousseau/html-generator
[10]: https://www.apache.org/licenses/LICENSE-2.0
[11]: https://opensource.org/licenses/MIT

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/html-generator/release.yml?branch=main&style=for-the-badge&logo=github

[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/html-generator?style=for-the-badge&token=Q9KJ6XXL67&logo=codecov
[crates-badge]: https://img.shields.io/crates/v/html-generator.svg?style=for-the-badge&color=fc8d62&logo=rust
[docs-badge]: https://img.shields.io/badge/docs.rs-html--generator-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/html--generator-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.3-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
