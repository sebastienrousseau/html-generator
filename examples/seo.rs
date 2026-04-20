// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! SEO: meta tag generation and structured data.
//!
//! Run: `cargo run --example seo`

#[path = "support.rs"]
mod support;

use html_generator::seo::{
    escape_html, generate_structured_data, MetaTagsBuilder,
    StructuredDataConfig,
};

fn main() {
    support::header("html-generator -- seo");

    // ── Build meta tags ──────────────────────────────────────────────
    support::task_with_output("Generate meta tags", || {
        match MetaTagsBuilder::new()
            .with_title("My Website")
            .with_description("A description of my website")
            .add_meta_tag("author", "HTML Generator")
            .build()
        {
            Ok(tags) => vec![format!("Meta tags: {tags}")],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── Generate structured data (default config) ────────────────────
    support::task_with_output(
        "Generate structured data (WebPage)",
        || {
            let html = "<html><head><title>My Page</title></head>\
                     <body><p>Page content here.</p></body></html>";
            match generate_structured_data(html, None) {
                Ok(json_ld) => vec![format!("JSON-LD: {json_ld}")],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── Generate structured data with custom config ──────────────────
    support::task_with_output(
        "Generate structured data (Article)",
        || {
            let html = "<html><head><title>Blog Post</title></head>\
                     <body><p>Article content.</p></body></html>";
            let config = StructuredDataConfig {
                page_type: "Article".to_string(),
                ..StructuredDataConfig::default()
            };
            match generate_structured_data(html, Some(config)) {
                Ok(json_ld) => vec![format!("JSON-LD: {json_ld}")],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── Escape HTML entities ─────────────────────────────────────────
    support::task_with_output("Escape HTML special characters", || {
        let raw = "<script>alert('xss')</script>";
        let escaped = escape_html(raw);
        vec![format!("Input:   {raw}"), format!("Escaped: {escaped}")]
    });

    support::summary(4);
}
