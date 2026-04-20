// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Table of contents generation from HTML headings.
//!
//! Run: `cargo run --example toc`

#[path = "support.rs"]
mod support;

use html_generator::generate_html;
use html_generator::utils::generate_table_of_contents;
use html_generator::HtmlConfig;

fn main() {
    support::header("html-generator -- toc");

    // ── Generate TOC from HTML with headings ─────────────────────────
    support::task_with_output("Generate table of contents", || {
        let html = "<h1>Introduction</h1><p>Text</p>\
                     <h2>Background</h2><p>More text</p>\
                     <h2>Methods</h2><p>Details</p>\
                     <h3>Experiment A</h3><p>Data</p>\
                     <h2>Results</h2><p>Findings</p>";
        match generate_table_of_contents(html) {
            Ok(toc) => vec![format!("TOC HTML:\n{toc}")],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── TOC via HtmlConfig with generate_toc enabled ─────────────────
    support::task_with_output("Pipeline with TOC enabled", || {
        let md = "# Chapter 1\n\n## Section 1.1\n\n## Section 1.2\n\n# Chapter 2\n\nContent.";
        let config = HtmlConfig {
            generate_toc: true,
            ..HtmlConfig::default()
        };
        match generate_html(md, &config) {
            Ok(html) => {
                let has_headings = html.contains("<h1");
                vec![
                    format!("TOC generation enabled: true"),
                    format!("Contains headings: {has_headings}"),
                    format!("Output length: {} bytes", html.len()),
                ]
            }
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    support::summary(2);
}
