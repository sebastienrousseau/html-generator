// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Header formatting: custom ID and class generators for headings.
//!
//! Run: `cargo run --example headers`

#[path = "support.rs"]
mod support;

use html_generator::utils::format_header_with_id_class;

fn main() {
    support::header("html-generator -- headers");

    // ── Default ID + class generation ───────────────────────────────
    support::task_with_output(
        "Default generators (slug-based)",
        || {
            let header = "<h2>Hello, World!</h2>";
            let result =
                format_header_with_id_class(header, None, None)
                    .unwrap();
            vec![
                format!("input  = {header}"),
                format!("output = {result}"),
            ]
        },
    );

    // ── Custom ID generator ─────────────────────────────────────────
    support::task_with_output("Custom ID generator", || {
        fn id_gen(content: &str) -> String {
            format!(
                "section-{}",
                content.to_lowercase().replace(' ', "-")
            )
        }
        let header = "<h3>Getting Started</h3>";
        let result =
            format_header_with_id_class(header, Some(id_gen), None)
                .unwrap();
        vec![format!("output = {result}")]
    });

    // ── Custom class generator ──────────────────────────────────────
    support::task_with_output("Custom class generator", || {
        fn class_gen(_: &str) -> String {
            "heading-primary".to_string()
        }
        let header = "<h1>Main Title</h1>";
        let result =
            format_header_with_id_class(header, None, Some(class_gen))
                .unwrap();
        vec![format!("output = {result}")]
    });

    // ── Both custom generators ──────────────────────────────────────
    support::task_with_output("Both ID and class generators", || {
        fn id_gen(content: &str) -> String {
            format!("nav-{}", content.to_lowercase().replace(' ', "-"))
        }
        fn class_gen(content: &str) -> String {
            format!("toc-{}", content.len())
        }
        let header = "<h2>API Reference</h2>";
        let result = format_header_with_id_class(
            header,
            Some(id_gen),
            Some(class_gen),
        )
        .unwrap();
        vec![format!("output = {result}")]
    });

    // ── Invalid header (expected failure) ───────────────────────────
    support::task_with_output(
        "Invalid header tag (expected failure)",
        || match format_header_with_id_class(
            "<p>Not a header</p>",
            None,
            None,
        ) {
            Err(e) => vec![format!("caught: {e}")],
            Ok(_) => vec!["BUG: should have failed".to_string()],
        },
    );

    // ── Special characters in content ───────────────────────────────
    support::task_with_output("Special characters in heading", || {
        let header = "<h3>C++ & Rust: A Comparison!</h3>";
        let result =
            format_header_with_id_class(header, None, None).unwrap();
        vec![format!("output = {result}")]
    });

    support::summary(6);
}
