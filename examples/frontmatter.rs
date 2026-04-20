// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Front matter extraction: YAML, TOML, and JSON formats.
//!
//! Run: `cargo run --example frontmatter`

#[path = "support.rs"]
mod support;

use html_generator::utils::{
    extract_front_matter, extract_front_matter_data,
};

fn main() {
    support::header("html-generator -- frontmatter");

    // ── YAML front matter (raw string) ───────────────────────────────
    support::task_with_output("Extract raw YAML front matter", || {
        let content =
            "---\ntitle: My Page\nauthor: Jane Doe\n---\n# Hello";
        match extract_front_matter(content) {
            Ok(raw) => vec![format!("Raw front matter: {raw}")],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── YAML front matter (parsed data) ──────────────────────────────
    support::task_with_output(
        "Parse YAML front matter to JSON",
        || {
            let content =
                "---\ntitle: My Page\nauthor: Jane Doe\n---\n# Hello";
            match extract_front_matter_data(content) {
                Ok((data, remaining)) => vec![
                    format!("Data: {data}"),
                    format!("Remaining content: {remaining}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── TOML front matter ────────────────────────────────────────────
    support::task_with_output("Parse TOML front matter", || {
        let content =
            "+++\ntitle = \"TOML Page\"\ndraft = true\n+++\n# Content";
        match extract_front_matter_data(content) {
            Ok((data, remaining)) => vec![
                format!("Data: {data}"),
                format!("Remaining: {remaining}"),
            ],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── JSON front matter ────────────────────────────────────────────
    support::task_with_output("Parse JSON front matter", || {
        let content =
            "{\"title\": \"JSON Page\", \"version\": 2}\n# Content";
        match extract_front_matter_data(content) {
            Ok((data, remaining)) => vec![
                format!("Data: {data}"),
                format!("Remaining: {remaining}"),
            ],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── No front matter ──────────────────────────────────────────────
    support::task_with_output(
        "Handle content without front matter",
        || {
            let content = "# Just a heading\n\nNo front matter here.";
            match extract_front_matter_data(content) {
                Ok((data, remaining)) => vec![
                    format!("Data: {data}"),
                    format!("Remaining: {remaining}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    support::summary(5);
}
