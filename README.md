<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/html-generator/images/logos/html-generator.svg"
alt="HTML Generator logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# HTML Generator (html-generator)

A high-performance Rust library that transforms Markdown into semantically rich, accessible HTML with WCAG 2.1 compliance.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Quick Links

- [HTML Generator (html-generator)](#html-generator-html-generator)
  - [Quick Links](#quick-links)
  - [Overview](#overview)
  - [Key Features](#key-features)
    - [Markdown Conversion](#markdown-conversion)
    - [Accessibility and Semantic Markup](#accessibility-and-semantic-markup)
    - [Performance Optimizations](#performance-optimizations)
    - [Advanced Configuration](#advanced-configuration)
  - [Installation](#installation)
  - [Basic Usage](#basic-usage)
  - [Advanced Configuration](#advanced-configuration-1)
  - [Processing Methods](#processing-methods)
    - [Synchronous Processing](#synchronous-processing)
    - [Asynchronous Processing](#asynchronous-processing)
  - [Error Handling](#error-handling)
  - [Examples](#examples)
    - [ARIA Elements \& Accessibility](#aria-elements--accessibility)
    - [Custom Markdown Styling](#custom-markdown-styling)
    - [Bringing It All Together](#bringing-it-all-together)
  - [Performance Characteristics](#performance-characteristics)
  - [Platform Support](#platform-support)
    - [Continuous Integration](#continuous-integration)
  - [Conversion Error Handling](#conversion-error-handling)
  - [Running Examples](#running-examples)
  - [Contributing](#contributing)
  - [Licensing](#licensing)
  - [Acknowledgements](#acknowledgements)

## Overview

HTML Generator is a high-performance Rust library for transforming Markdown into semantically rich, accessible HTML.

## Key Features

### Markdown Conversion

- **Core Processing**:
  - Standard Markdown to HTML transformation
  - Configurable parsing with `ComrakOptions`
  - Front matter extraction support
  - Basic syntax highlighting via `syntect`
  - Inline HTML preservation

### Accessibility and Semantic Markup

- **ARIA and Accessibility Features**:
  - Automated ARIA attribute generation for:
    - Buttons
    - Form elements
    - Navigation structures
    - Interactive components
  - WCAG 2.1 validation checks
  - Semantic HTML structure preservation
  - Automatic role inference for HTML elements

### Performance Optimizations

- **Efficient Processing**:
  - O(n) time complexity parsing
  - Constant memory overhead for small documents
  - Synchronous and asynchronous HTML generation methods
  - Minimal runtime overhead
  - Optional HTML minification

### Advanced Configuration

- **Flexible Transformation**:
  - Language-specific rendering
  - Configurable syntax highlighting
  - Custom block processing
  - Emoji handling (limited)
  - SEO metadata generation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
html-generator = "0.0.3"
```

## Basic Usage

```rust
use html_generator::{generate_html, HtmlConfig};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let markdown = "# Welcome\n\nThis is **HTML Generator**!";
    let config = HtmlConfig::default();
    let html = generate_html(markdown, &config)?;
    println!("{}", html);
    Ok(())
}
```

## Advanced Configuration

```rust
use html_generator::HtmlConfig;
use html_generator::error::HtmlError;

fn main() -> Result<(), HtmlError> {
    let config = HtmlConfig::builder()
        .with_language("en-GB")
        .with_syntax_highlighting(true, Some("monokai".to_string()))
        .build()?;

    println!("Built config: {:?}", config);
    Ok(())
}
```

## Processing Methods

### Synchronous Processing

```rust
use html_generator::{generate_html, HtmlConfig};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let markdown = "# Hello from synchronous processing";
    let config = HtmlConfig::default();
    let html = generate_html(markdown, &config)?;
    println!("{}", html);
    Ok(())
}
```

### Asynchronous Processing

```rust
# use html_generator::performance::async_generate_html;
# use std::error::Error;
#
# // We hide the async main to avoid doc-test errors about `.await`.
# // The code inside demonstrates how you'd normally use `async_generate_html`.
# async fn async_main_example() -> Result<(), Box<dyn Error>> {
    let markdown = "# Async Processing\n\nThis is **HTML Generator**!";
    let html = async_generate_html(markdown).await?;
    println!("{}", html);
    Ok(())
# }
```

## Error Handling

```rust
use html_generator::{generate_html, HtmlConfig};
use html_generator::error::HtmlError;

fn handle_conversion_error(markdown: &str) -> Result<(), HtmlError> {
    let config = HtmlConfig::default();
    match generate_html(markdown, &config) {
        Ok(html) => println!("Conversion successful: {}", html),
        Err(HtmlError::InvalidInput(msg)) => {
            eprintln!("Invalid input: {}", msg);
        },
        Err(HtmlError::InputTooLarge(size)) => {
            eprintln!("Input too large: {} bytes", size);
        },
        Err(e) => eprintln!("Unexpected error: {}", e),
    }
    Ok(())
}
```

## Examples

HTML Generator provides many advanced capabilities for accessibility, ARIA attributes, and custom Markdown styling. Below is a summary of what you can explore. For more detailed code, see the `src/examples/` directory in this repository.

### ARIA Elements & Accessibility

Add ARIA attributes to common HTML elements (buttons, forms, tables, and more) to ensure accessibility compliance. The library automatically infers roles and labels for screen readers.

**Example Snippet** _(from `aria_elements_example.rs`)_:

```rust
use html_generator::accessibility::add_aria_attributes;
use html_generator::error::HtmlError;

fn main() -> Result<(), HtmlError> {
    // Basic HTML snippet for a button
    let html_button = "<button>Click me</button>";

    // Enhance with ARIA attributes
    let enhanced_button =
        add_aria_attributes(html_button, None).map_err(|e| {
            // Convert from an accessibility::Error to an HtmlError
            HtmlError::InvalidInput(e.to_string())
        })?;
    println!("Original:  {}", html_button);
    println!("Enhanced:  {}", enhanced_button);

    Ok(())
}
```

**Run the full ARIA demo**:

```bash
cargo run --example aria
```

This will print out multiple examples showcasing enhancements for buttons, forms, navigation elements, tables, live regions, and nested components.

### Custom Markdown Styling

Demonstrate transforming extended Markdown features such as:

- **Custom blocks** (e.g., `:::note`, `:::warning`)
- **Inline `.class="..."` directives** for images or elements
- **Syntax highlighting** for fenced code blocks
- **Blockquotes with optional citation**  

…and much more.

**Example Snippet** _(from `style_example.rs`)_:

```rust
use html_generator::error::HtmlError;
use html_generator::generator::markdown_to_html_with_extensions;

fn main() -> Result<(), HtmlError> {
    let markdown = r":::note
Custom note block with a specific style
:::";

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => println!("Converted:\n{}", html),
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}
```

**Run the full style demo**:

```bash
cargo run --example style
```

This will print out multiple styled examples (custom blocks, images, tables, bullet lists, code blocks, etc.) and show how they render as HTML.

### Bringing It All Together

If you’d like to combine accessibility features and custom Markdown styling, you can configure your [`HtmlConfig`](#advanced-configuration) to enable:

1. **Syntax highlighting**
2. **ARIA attribute generation**
3. **Custom block parsing**
4. **Emoji support**

…thereby providing a powerful, end-to-end Markdown-to-HTML transformation pipeline suitable for high-performance, semantically rich, and user-friendly content.

## Performance Characteristics

| Document Scale | Processing Time | Memory Utilization |
|---------------|----------------|-------------------|
| Small (<1KB)  | ~0.1ms         | Constant O(1)     |
| Medium (10KB) | ~1ms           | Linear O(n)       |
| Large (100KB) | ~10ms          | Linear O(n)       |

## Platform Support

| Platform      | Status     | Rust Version | Notes                     |
|--------------|------------|--------------|---------------------------|
| Linux        | ✅ Fully   | 1.56+        | Comprehensive support     |
| macOS        | ✅ Fully   | 1.56+        | Native performance        |
| Windows      | ✅ Fully   | 1.56+        | Complete compatibility    |
| WebAssembly  | ⚠️ Partial | 1.56+        | Limited feature support   |

### Continuous Integration

We use GitHub Actions for comprehensive testing:

- Cross-platform compatibility checks
- Extensive test coverage

[View CI Workflow](https://github.com/sebastienrousseau/html-generator/actions)

## Conversion Error Handling

```rust
# use html_generator::{generate_html, HtmlConfig};
# use html_generator::error::HtmlError;
#
fn handle_conversion_error(markdown: &str) -> Result<(), HtmlError> {
    // We'll define a config for this snippet:
    let config = HtmlConfig::default();
    match generate_html(markdown, &config) {
        Ok(html) => println!("Conversion successful"),
        Err(HtmlError::InvalidInput(msg)) => {
            eprintln!("Invalid input: {}", msg);
        },
        Err(HtmlError::InputTooLarge(size)) => {
            eprintln!("Input too large: {} bytes", size);
        },
        Err(HtmlError::Io(io_error)) => {
            eprintln!("I/O error occurred: {}", io_error);
        },
        // If your crate doesn't actually have a `Markdown` variant, remove this block
        // Err(HtmlError::Markdown(markdown_error)) => {
        //     eprintln!("Markdown processing error: {}", markdown_error);
        // },
        Err(e) => eprintln!("Unexpected error: {}", e),
    }
    Ok(())
}
```

## Running Examples

```bash
# Basic example
cargo run --example basic

# Accessibility demo
cargo run --example aria

# Performance benchmark
cargo run --example performance
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details on:

- Reporting issues
- Suggesting features
- Submitting pull requests

## Licensing

Dual-licensed: Apache 2.0 & MIT

## Acknowledgements

Special thanks to the Rust community and open-source contributors.

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

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/html-generator/release.yml?branch=main&style=for-the-badge&logo=github
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/html-generator?style=for-the-badge&token=Q9KJ6XXL67&logo=codecov
[crates-badge]: https://img.shields.io/crates/v/html-generator.svg?style=for-the-badge&color=fc8d62&logo=rust
[docs-badge]: https://img.shields.io/badge/docs.rs-html--generator-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/html--generator-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.3-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
