// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Front matter extraction: YAML, TOML, and JSON metadata parsing.
//!
//! Supports nested structures and arrays via industrial-strength
//! parsers (the vendored `yaml_safe` crate for YAML, the `toml`
//! crate for TOML).
//!
//! Run: `cargo run --example frontmatter`

#[path = "support.rs"]
mod support;

use html_generator::utils::{
    extract_front_matter, extract_front_matter_data,
};

fn main() {
    support::header("html-generator -- frontmatter");

    // ── YAML front matter (---) ─────────────────────────────────────
    support::task_with_output("Extract YAML front matter", || {
        let content =
            "---\ntitle: My Page\nauthor: Jane Doe\n---\n# Hello";
        let (data, body) = extract_front_matter_data(content).unwrap();
        vec![
            format!("title  = {}", data["title"]),
            format!("author = {}", data["author"]),
            format!("body   = {body}"),
        ]
    });

    // ── TOML front matter (+++) ─────────────────────────────────────
    support::task_with_output("Extract TOML front matter", || {
        let content =
            "+++\ntitle = \"TOML Page\"\ndraft = true\n+++\n# Content";
        let (data, body) = extract_front_matter_data(content).unwrap();
        vec![
            format!("title = {}", data["title"]),
            format!("draft = {}", data["draft"]),
            format!("body  = {body}"),
        ]
    });

    // ── JSON front matter ({...}) ───────────────────────────────────
    support::task_with_output("Extract JSON front matter", || {
        let content =
            "{\"title\": \"JSON Page\", \"lang\": \"en\"}\n# Content";
        let (data, body) = extract_front_matter_data(content).unwrap();
        vec![
            format!("title = {}", data["title"]),
            format!("lang  = {}", data["lang"]),
            format!("body  = {body}"),
        ]
    });

    // ── Raw extraction (strip only) ─────────────────────────────────
    support::task_with_output(
        "Strip front matter without parsing",
        || {
            let content = "---\ntitle: Stripped\n---\n# Body only";
            let body = extract_front_matter(content).unwrap();
            vec![format!("body = {body}")]
        },
    );

    // ── Content without front matter ────────────────────────────────
    support::task_with_output("Content without front matter", || {
        let content = "# Just a heading\n\nNo metadata here.";
        let (data, body) = extract_front_matter_data(content).unwrap();
        vec![format!("data = {data}"), format!("body = {body}")]
    });

    // ── Nested YAML structures ──────────────────────────────────────
    support::task_with_output("YAML with many metadata fields", || {
        let content = "---\ntitle: Blog Post\nauthor: Jane Doe\ndate: 2025-01-15\nlang: en-GB\ndraft: false\n---\n# Content";
        let (data, body) = extract_front_matter_data(content).unwrap();
        vec![
            format!("title  = {}", data["title"]),
            format!("author = {}", data["author"]),
            format!("date   = {}", data["date"]),
            format!("lang   = {}", data["lang"]),
            format!("draft  = {}", data["draft"]),
            format!("body   = {body}"),
        ]
    });

    support::summary(6);
}
