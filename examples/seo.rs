// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! SEO: meta tags, structured data (JSON-LD), and HTML escaping.
//!
//! Run: `cargo run --example seo`

#[path = "support.rs"]
mod support;

use html_generator::seo::{
    escape_html, generate_structured_data, MetaTagsBuilder,
    StructuredDataConfig,
};
use std::collections::HashMap;

fn main() {
    support::header("html-generator -- seo");

    // ── Meta tag generation ─────────────────────────────────────────
    support::task_with_output("Generate meta tags", || {
        let meta = MetaTagsBuilder::new()
            .with_title("My Page")
            .with_description("A great page about Rust")
            .add_meta_tag("author", "Jane Doe")
            .add_meta_tag("robots", "index,follow")
            .build()
            .unwrap();
        meta.lines().map(|l| l.to_string()).collect()
    });

    // ── JSON-LD structured data (WebPage) ───────────────────────────
    support::task_with_output(
        "Generate JSON-LD (WebPage, default)",
        || {
            let html = r#"<html><head><title>My Site</title></head><body><p>Content here.</p></body></html>"#;
            let json_ld = generate_structured_data(html, None).unwrap();
            vec![
                format!(
                    "has ld+json   = {}",
                    json_ld.contains("ld+json")
                ),
                format!(
                    "has @context  = {}",
                    json_ld.contains("schema.org")
                ),
                format!("length        = {} bytes", json_ld.len()),
            ]
        },
    );

    // ── JSON-LD structured data (Article) ───────────────────────────
    support::task_with_output(
        "Generate JSON-LD (Article, custom config)",
        || {
            let html = r#"<html><head><title>Blog Post</title></head><body><p>Article body.</p></body></html>"#;
            let config = StructuredDataConfig {
                page_type: "Article".to_string(),
                additional_data: Some(HashMap::from([(
                    "author".to_string(),
                    "Jane Doe".to_string(),
                )])),
                ..Default::default()
            };
            let json_ld =
                generate_structured_data(html, Some(config)).unwrap();
            vec![
                format!(
                    "has Article = {}",
                    json_ld.contains("Article")
                ),
                format!(
                    "has author  = {}",
                    json_ld.contains("Jane Doe")
                ),
            ]
        },
    );

    // ── HTML entity escaping ────────────────────────────────────────
    support::task_with_output("Escape HTML entities", || {
        let raw = r#"<script>alert("XSS & more")</script>"#;
        let escaped = escape_html(raw);
        vec![format!("input  = {raw}"), format!("output = {escaped}")]
    });

    support::summary(4);
}
