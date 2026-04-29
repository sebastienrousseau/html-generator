// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Basic usage: convert Markdown to HTML with default settings.
//!
//! Run: `cargo run --example hello`

#[path = "support.rs"]
mod support;

use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- hello");

    // ── Heading + paragraph ─────────────────────────────────────────
    support::task_with_output("Convert heading and paragraph", || {
        let md = "# Hello, World!\n\nThis is a paragraph.";
        let html = generate_html(md, &HtmlConfig::default()).unwrap();
        html.lines().map(|l| l.to_string()).collect()
    });

    // ── Inline formatting ───────────────────────────────────────────
    support::task_with_output(
        "Inline formatting (bold, italic, code)",
        || {
            let md = "Use **bold**, *italic*, and `code` in prose.";
            let html =
                generate_html(md, &HtmlConfig::default()).unwrap();
            html.lines().map(|l| l.to_string()).collect()
        },
    );

    // ── Lists ───────────────────────────────────────────────────────
    support::task_with_output("Unordered and ordered lists", || {
        let md = "- Alpha\n- Bravo\n- Charlie\n\n1. First\n2. Second";
        let html = generate_html(md, &HtmlConfig::default()).unwrap();
        html.lines().map(|l| l.to_string()).collect()
    });

    // ── Fenced code block ───────────────────────────────────────────
    support::task_with_output("Fenced code block", || {
        let md = "```rust\nfn main() {\n    println!(\"hi\");\n}\n```";
        let html = generate_html(md, &HtmlConfig::default()).unwrap();
        vec![format!("output_length = {} bytes", html.len())]
    });

    // ── Links and images ────────────────────────────────────────────
    support::task_with_output("Links and images", || {
        let md = "[Rust](https://rust-lang.org) and ![logo](logo.png)";
        let html = generate_html(md, &HtmlConfig::default()).unwrap();
        html.lines().map(|l| l.to_string()).collect()
    });

    support::summary(5);
}
