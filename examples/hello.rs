// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Basic markdown to HTML conversion.
//!
//! Run: `cargo run --example hello`

#[path = "support.rs"]
mod support;

use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- hello");

    // ── Simple heading + paragraph ───────────────────────────────────
    support::task_with_output("Convert heading and paragraph", || {
        let md = "# Hello, world!\n\nThis is a paragraph.";
        let config = HtmlConfig::default();
        match generate_html(md, &config) {
            Ok(html) => {
                vec![format!("Input:  {md}"), format!("Output: {html}")]
            }
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── Emphasis and strong ──────────────────────────────────────────
    support::task_with_output("Convert inline formatting", || {
        let md = "Text with *emphasis* and **strong** words.";
        let config = HtmlConfig::default();
        match generate_html(md, &config) {
            Ok(html) => vec![format!("Output: {html}")],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── Unordered list ───────────────────────────────────────────────
    support::task_with_output("Convert unordered list", || {
        let md = "- Alpha\n- Bravo\n- Charlie";
        let config = HtmlConfig::default();
        match generate_html(md, &config) {
            Ok(html) => vec![format!("Output: {html}")],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── Code block with syntax highlighting ──────────────────────────
    support::task_with_output("Convert fenced code block", || {
        let md =
            "```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
        let config = HtmlConfig {
            enable_syntax_highlighting: true,
            ..HtmlConfig::default()
        };
        match generate_html(md, &config) {
            Ok(html) => {
                let has_lang = html.contains("language-rust");
                vec![
                    format!("Syntax highlighting detected: {has_lang}"),
                    format!("Output length: {} bytes", html.len()),
                ]
            }
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    support::summary(4);
}
