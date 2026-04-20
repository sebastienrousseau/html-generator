// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Error handling patterns across the library.
//!
//! Run: `cargo run --example errors`

#[path = "support.rs"]
mod support;

use html_generator::error::HtmlError;
use html_generator::utils::{
    extract_front_matter, extract_front_matter_data,
};
use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- errors");

    // ── Empty input ──────────────────────────────────────────────────
    support::task_with_output(
        "Handle empty input (expected failure)",
        || match generate_html("", &HtmlConfig::default()) {
            Err(e) => vec![format!("Caught: {e}")],
            Ok(_) => vec!["BUG: should have failed".to_string()],
        },
    );

    // ── Oversized input ──────────────────────────────────────────────
    support::task_with_output(
        "Handle oversized input (expected failure)",
        || {
            let config = HtmlConfig {
                max_input_size: 2048,
                ..HtmlConfig::default()
            };
            let big = "x".repeat(3000);
            match generate_html(&big, &config) {
                Err(e) => vec![format!("Caught: {e}")],
                Ok(_) => vec!["BUG: should have failed".to_string()],
            }
        },
    );

    // ── Invalid front matter ─────────────────────────────────────────
    support::task_with_output(
        "Handle invalid front matter (expected failure)",
        || {
            let content = "---\nunclosed front matter\n# Heading";
            match extract_front_matter(content) {
                Err(e) => vec![format!("Caught: {e}")],
                Ok(v) => vec![format!("Returned: {v}")],
            }
        },
    );

    // ── Invalid language code in config builder ──────────────────────
    support::task_with_output(
        "Handle invalid language code (expected failure)",
        || match HtmlConfig::builder().with_language("invalid").build()
        {
            Err(e) => vec![format!("Caught: {e}")],
            Ok(_) => vec!["BUG: should have failed".to_string()],
        },
    );

    // ── Graceful recovery pattern ────────────────────────────────────
    support::task_with_output(
        "Graceful error recovery pattern",
        || {
            let inputs: Vec<(&str, &str)> = vec![
                ("valid markdown", "# Title\n\nBody text."),
                ("empty string", ""),
                ("yaml front matter", "---\ntitle: Ok\n---\n# Content"),
                ("broken yaml fm", "---\nno closing\n# Content"),
            ];
            inputs
                .into_iter()
                .map(|(label, input)| {
                    let result =
                        generate_html(input, &HtmlConfig::default());
                    match result {
                        Ok(html) => format!(
                            "{label:<20} -> ok ({} bytes)",
                            html.len()
                        ),
                        Err(e) => format!("{label:<20} -> err ({e})"),
                    }
                })
                .collect()
        },
    );

    // ── Error type matching ──────────────────────────────────────────
    support::task_with_output(
        "Programmatic error type matching",
        || {
            let test_cases: Vec<(&str, Result<String, HtmlError>)> = vec![
                (
                    "empty input",
                    generate_html("", &HtmlConfig::default()),
                ),
                (
                    "front matter",
                    extract_front_matter_data("---\nbad\n# x")
                        .map(|(v, _)| v.to_string()),
                ),
            ];
            test_cases
                .into_iter()
                .map(|(label, result)| {
                    let status = match &result {
                        Ok(_) => "ok".to_string(),
                        Err(e) => format!("err: {e}"),
                    };
                    format!("{label:<16} -> {status}")
                })
                .collect()
        },
    );

    support::summary(6);
}
