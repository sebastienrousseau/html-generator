// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Header formatting with custom ID and class generators.
//!
//! Run: `cargo run --example headers`

#[path = "support.rs"]
mod support;

use html_generator::utils::format_header_with_id_class;

fn main() {
    support::header("html-generator -- headers");

    // ── Default formatting (no custom generators) ────────────────────
    support::task_with_output("Format header with defaults", || {
        let header = "<h2>Getting Started</h2>";
        match format_header_with_id_class(header, None, None) {
            Ok(formatted) => vec![
                format!("Input:  {header}"),
                format!("Output: {formatted}"),
            ],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── Custom ID generator ──────────────────────────────────────────
    support::task_with_output(
        "Format header with custom ID generator",
        || {
            fn custom_id(text: &str) -> String {
                format!(
                    "section-{}",
                    text.to_lowercase().replace(' ', "-")
                )
            }
            let header = "<h2>API Reference</h2>";
            match format_header_with_id_class(
                header,
                Some(custom_id),
                None,
            ) {
                Ok(formatted) => vec![
                    format!("Input:  {header}"),
                    format!("Output: {formatted}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── Custom class generator ───────────────────────────────────────
    support::task_with_output(
        "Format header with custom class generator",
        || {
            fn custom_class(_text: &str) -> String {
                "doc-heading".to_string()
            }
            let header = "<h3>Installation</h3>";
            match format_header_with_id_class(
                header,
                None,
                Some(custom_class),
            ) {
                Ok(formatted) => vec![
                    format!("Input:  {header}"),
                    format!("Output: {formatted}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── Both custom ID and class ─────────────────────────────────────
    support::task_with_output(
        "Format header with both generators",
        || {
            fn slug_id(text: &str) -> String {
                text.to_lowercase().replace(' ', "-")
            }
            fn level_class(_text: &str) -> String {
                "heading-styled".to_string()
            }
            let header = "<h1>Welcome Guide</h1>";
            match format_header_with_id_class(
                header,
                Some(slug_id),
                Some(level_class),
            ) {
                Ok(formatted) => vec![
                    format!("Input:  {header}"),
                    format!("Output: {formatted}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── Invalid header format ────────────────────────────────────────
    support::task_with_output(
        "Handle invalid header (expected failure)",
        || {
            let header = "<div>Not a header</div>";
            match format_header_with_id_class(header, None, None) {
                Err(e) => vec![format!("Caught: {e}")],
                Ok(v) => vec![format!("Unexpected success: {v}")],
            }
        },
    );

    support::summary(5);
}
