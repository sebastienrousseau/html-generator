<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/html-generator/images/logos/html-generator.svg"
alt="HTML Generator logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# HTML Generator (html-generator)

A Rust-based HTML generation and optimization library.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

The `html-generator` is a robust Rust library designed for transforming Markdown into SEO-optimized, accessible HTML. Featuring front matter extraction, custom header processing, table of contents generation, and performance optimization for web projects of any scale.

## Features

- **Markdown to HTML Conversion**: Convert Markdown content to HTML with support for custom extensions.
- **Front Matter Extraction**: Extract and process front matter from Markdown content.
- **Advanced Header Processing**: Automatically generate id and class attributes for headers.
- **Table of Contents Generation**: Create a table of contents from HTML content.
- **SEO Optimization**: Generate meta tags and structured data (JSON-LD) for improved search engine visibility.
- **Accessibility Enhancements**: Add ARIA attributes and validate against WCAG guidelines.
- **Performance Optimization**: Minify HTML output and support asynchronous generation for large sites.
- **Flexible Configuration**: Customize the HTML generation process through a comprehensive set of options.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
html-generator = "0.0.1"
```

## Usage

Here's a basic example of how to use `html-generator`:

```rust
use html_generator::utils::{extract_front_matter, format_header_with_id_class, generate_table_of_contents};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Extract front matter
    let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
    let content_without_front_matter = extract_front_matter(content)?;
    println!("Content without front matter:\n{}", content_without_front_matter);

    // Format header with ID and class
    let header = "<h2>Hello, World!</h2>";
    let formatted_header = format_header_with_id_class(header, None, None)?;
    println!("Formatted header:\n{}", formatted_header);

    // Generate table of contents
    let html = "<h1>Title</h1><p>Some content</p><h2>Subtitle</h2><p>More content</p>";
    let toc = generate_table_of_contents(html)?;
    println!("Table of contents:\n{}", toc);

    Ok(())
}
```

## Documentation

For full API documentation, please visit [docs.rs/html-generator][04].

## Examples

To run the examples, clone the repository and use the following command:

```shell
cargo run --example example_name
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- [Apache License, Version 2.0][10]
- [MIT license][11]

at your option.

## Acknowledgements

Special thanks to all contributors who have helped build the `html-generator` library.

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
[docs-badge]: https://img.shields.io/badge/docs.rs-metadata--gen-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/metadata--gen-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.1-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
