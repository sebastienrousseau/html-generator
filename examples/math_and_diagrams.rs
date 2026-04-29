// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 HTML Generator. All rights reserved.

//! Server-side LaTeX → MathML and Mermaid passthrough.
//!
//! Demonstrates the two new v0.0.5 post-processing flags:
//!
//! * `enable_math` — `$..$` and `$$..$$` LaTeX spans become
//!   `<math>…</math>` MathML inline. Browsers render MathML
//!   natively, so no client-side JS is required.
//! * `enable_diagrams` — `\u{60}\u{60}\u{60}mermaid` fenced code blocks are
//!   rewritten to `<pre class="mermaid">…</pre>` so the standard
//!   client-side mermaid.js bundle renders them.
//!
//! Run: `cargo run --example math_and_diagrams`

#[path = "support.rs"]
mod support;

use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- math and diagrams");

    // ── Inline + display math ────────────────────────────────────
    support::task_with_output(
        "Inline + display LaTeX → MathML",
        || {
            let md = r"# Pythagorean

In a right triangle, $a^2 + b^2 = c^2$.

The famous mass-energy equivalence:

$$E = mc^2$$";
            let cfg = HtmlConfig {
                enable_math: true,
                add_aria_attributes: false, // keep output tight for the demo
                ..HtmlConfig::default()
            };
            let html = generate_html(md, &cfg).unwrap();
            vec![
                format!(
                    "contains_inline_math = {}",
                    html.contains("<math")
                ),
                format!(
                    "contains_block_math  = {}",
                    html.contains(r#"display="block""#)
                ),
                format!("output_length        = {} bytes", html.len()),
            ]
        },
    );

    // ── Currency stays literal ───────────────────────────────────
    support::task_with_output("Prose with `$5` is left alone", || {
        let md = "That cost $5 yesterday and $10 today.";
        let cfg = HtmlConfig {
            enable_math: true,
            add_aria_attributes: false,
            ..HtmlConfig::default()
        };
        let html = generate_html(md, &cfg).unwrap();
        // No <math> emitted; the `$` digits look like currency.
        vec![
            format!("contains_math = {}", html.contains("<math")),
            format!("output        = {}", html.trim()),
        ]
    });

    // ── Mermaid passthrough ──────────────────────────────────────
    support::task_with_output(
        "Mermaid block rewritten for client-side rendering",
        || {
            let md = r"# Pipeline

```mermaid
graph LR
    A[Markdown] --> B[Parser]
    B --> C[ARIA]
    C --> D[HTML]
```";
            let cfg = HtmlConfig {
                enable_diagrams: true,
                add_aria_attributes: false,
                ..HtmlConfig::default()
            };
            let html = generate_html(md, &cfg).unwrap();
            vec![
                format!(
                    "contains_mermaid_class = {}",
                    html.contains(r#"<pre class="mermaid""#)
                ),
                format!(
                    "drops_language_marker  = {}",
                    !html.contains("language-mermaid")
                ),
            ]
        },
    );

    // ── Both at once ─────────────────────────────────────────────
    support::task_with_output(
        "Math and diagrams composed in one pass",
        || {
            let md = r"# Combined

Newton's second law: $$F = ma$$

```mermaid
flowchart TD
    Force --> Acceleration
```";
            let cfg = HtmlConfig {
                enable_math: true,
                enable_diagrams: true,
                add_aria_attributes: false,
                ..HtmlConfig::default()
            };
            let html = generate_html(md, &cfg).unwrap();
            vec![
                format!("has_mathml  = {}", html.contains("<math")),
                format!(
                    "has_mermaid = {}",
                    html.contains(r#"<pre class="mermaid""#)
                ),
            ]
        },
    );

    support::summary(4);
}
