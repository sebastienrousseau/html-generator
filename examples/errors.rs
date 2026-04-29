// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Error handling: every error variant and graceful recovery.
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

    // ── Empty input ─────────────────────────────────────────────────
    support::task_with_output("Empty input (expected failure)", || {
        match generate_html("", &HtmlConfig::default()) {
            Err(e) => vec![format!("caught: {e}")],
            Ok(_) => vec!["BUG: should have failed".to_string()],
        }
    });

    // ── Oversized input ─────────────────────────────────────────────
    support::task_with_output(
        "Oversized input (expected failure)",
        || {
            let config = HtmlConfig {
                max_input_size: 64,
                ..HtmlConfig::default()
            };
            let big = "x".repeat(128);
            match generate_html(&big, &config) {
                Err(e) => vec![format!("caught: {e}")],
                Ok(_) => vec!["BUG: should have failed".to_string()],
            }
        },
    );

    // ── Invalid front matter ────────────────────────────────────────
    support::task_with_output(
        "Invalid front matter (expected failure)",
        || {
            let content =
                "---\ntitle: value\ninvalid_line\n---\nContent";
            match extract_front_matter(content) {
                Err(e) => vec![format!("caught: {e}")],
                Ok(_) => vec!["BUG: should have failed".to_string()],
            }
        },
    );

    // ── Invalid TOML front matter ───────────────────────────────────
    support::task_with_output(
        "Malformed TOML (expected failure)",
        || {
            let content = "+++\ntitle = unquoted value\n+++\n# Body";
            match extract_front_matter_data(content) {
                Err(e) => vec![format!("caught: {e}")],
                Ok(_) => vec!["BUG: should have failed".to_string()],
            }
        },
    );

    // ── Error type matching ─────────────────────────────────────────
    support::task_with_output(
        "Programmatic error type matching",
        || {
            let cases: Vec<(&str, Result<String, HtmlError>)> = vec![
                ("empty", generate_html("", &HtmlConfig::default())),
                (
                    "valid",
                    generate_html("# Hi", &HtmlConfig::default()),
                ),
                ("bad_fm", extract_front_matter("---\nbad\n---\nBody")),
            ];
            cases
                .into_iter()
                .map(|(label, result)| match result {
                    Ok(_) => format!("{label:<7} -> ok"),
                    Err(e) => {
                        format!("{label:<7} -> {}", error_kind(&e))
                    }
                })
                .collect()
        },
    );

    // ── Graceful recovery ───────────────────────────────────────────
    support::task_with_output(
        "Graceful recovery over multiple inputs",
        || {
            let inputs = vec![
                ("valid md", "# Title\n\nBody text."),
                ("empty", ""),
                ("no front", "Just plain text."),
            ];
            inputs
                .into_iter()
                .map(|(label, input)| {
                    match generate_html(input, &HtmlConfig::default()) {
                        Ok(html) => format!(
                            "{label:<10} -> ok ({} bytes)",
                            html.len()
                        ),
                        Err(e) => format!("{label:<10} -> err ({e})"),
                    }
                })
                .collect()
        },
    );

    support::summary(6);
}

fn error_kind(e: &HtmlError) -> &'static str {
    match e {
        HtmlError::InvalidInput(_) => "InvalidInput",
        HtmlError::InputTooLarge(_) => "InputTooLarge",
        HtmlError::InvalidFrontMatterFormat(_) => {
            "InvalidFrontMatterFormat"
        }
        HtmlError::InvalidHeaderFormat(_) => "InvalidHeaderFormat",
        _ => "Other",
    }
}
