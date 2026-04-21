// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Table of contents: generate navigation from HTML headings.
//!
//! Run: `cargo run --example toc`

#[path = "support.rs"]
mod support;

use html_generator::utils::generate_table_of_contents;
use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- toc");

    // ── Direct TOC from HTML ────────────────────────────────────────
    support::task_with_output(
        "Generate TOC from HTML headings",
        || {
            let html = "<h1>Introduction</h1><p>Text</p><h2>Setup</h2><p>More</p><h2>Usage</h2><h3>Advanced</h3>";
            let toc = generate_table_of_contents(html).unwrap();
            toc.lines().map(|l| l.to_string()).collect()
        },
    );

    // ── TOC via [[TOC]] placeholder ─────────────────────────────────
    support::task_with_output(
        "Replace [[TOC]] placeholder in pipeline",
        || {
            let md = "[[TOC]]\n\n# Chapter 1\n\n## Section 1.1\n\n# Chapter 2\n\n## Section 2.1";
            let config = HtmlConfig {
                generate_toc: true,
                ..HtmlConfig::default()
            };
            let html = generate_html(md, &config).unwrap();
            vec![
                format!("has <ul>       = {}", html.contains("<ul>")),
                format!(
                    "has chapter-1  = {}",
                    html.contains("chapter-1")
                ),
                format!(
                    "has chapter-2  = {}",
                    html.contains("chapter-2")
                ),
                format!(
                    "has section    = {}",
                    html.contains("section")
                ),
            ]
        },
    );

    // ── Single heading ──────────────────────────────────────────────
    support::task_with_output("TOC with single heading", || {
        let html = "<h1>Only Title</h1><p>Body content.</p>";
        let toc = generate_table_of_contents(html).unwrap();
        vec![format!("toc = {toc}")]
    });

    // ── No headings ─────────────────────────────────────────────────
    support::task_with_output("TOC with no headings", || {
        let html = "<p>No headings at all.</p>";
        let toc = generate_table_of_contents(html).unwrap();
        vec![format!("toc = {toc}")]
    });

    support::summary(4);
}
