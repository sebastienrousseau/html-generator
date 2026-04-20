// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Full pipeline: markdown to optimised HTML with all features enabled.
//!
//! Run: `cargo run --example pipeline`

#[path = "support.rs"]
mod support;

use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- pipeline");

    let markdown = r#"---
title: Pipeline Demo
author: HTML Generator
---

# Pipeline Demo

## Introduction

This document exercises the **full conversion pipeline**.

- Table of contents generation
- Accessibility (ARIA attributes)
- SEO structured data
- HTML minification

## Code Sample

```rust
fn greet(name: &str) {
    println!("Hello, {name}!");
}
```

## Conclusion

All features enabled in a single pass.
"#;

    // ── Generate with all features ───────────────────────────────────
    let config = HtmlConfig {
        enable_syntax_highlighting: true,
        syntax_theme: Some("github".to_string()),
        minify_output: true,
        add_aria_attributes: true,
        generate_structured_data: true,
        generate_toc: true,
        ..HtmlConfig::default()
    };

    let html =
        support::task("Generate HTML with full pipeline", || {
            generate_html(markdown, &config)
        });

    // ── Inspect results ──────────────────────────────────────────────
    support::task_with_output(
        "Inspect generated HTML",
        || match &html {
            Ok(h) => {
                let has_heading = h.contains("<h1");
                let has_code = h.contains("language-rust");
                let has_list = h.contains("<li>");
                vec![
                    format!("Output length: {} bytes", h.len()),
                    format!("Contains heading: {has_heading}"),
                    format!("Contains code block: {has_code}"),
                    format!("Contains list items: {has_list}"),
                ]
            }
            Err(e) => vec![format!("Error: {e}")],
        },
    );

    support::summary(2);
}
