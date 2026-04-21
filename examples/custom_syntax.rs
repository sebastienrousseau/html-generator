// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Custom Markdown extensions: triple-colon blocks and image classes.
//!
//! Demonstrates `:::class ... :::` styled blocks and
//! `![alt](url).class="cls"` image syntax.
//!
//! Run: `cargo run --example custom_syntax`

#[path = "support.rs"]
mod support;

use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- custom_syntax");

    // ── Triple-colon class blocks ───────────────────────────────────
    support::task_with_output(
        "Triple-colon styled block (:::warning)",
        || {
            let md = ":::warning\n**Caution:** This action is irreversible.\n:::";
            let html =
                generate_html(md, &HtmlConfig::default()).unwrap();
            vec![
                format!(
                    "has class=warning = {}",
                    html.contains("class=\"warning\"")
                ),
                format!(
                    "has <strong>      = {}",
                    html.contains("<strong>")
                ),
            ]
        },
    );

    // ── Multiple block classes ──────────────────────────────────────
    support::task_with_output("Multiple styled blocks", || {
        let md = ":::note\nThis is a note.\n:::\n\n:::danger\nThis is dangerous.\n:::";
        let html = generate_html(md, &HtmlConfig::default()).unwrap();
        vec![
            format!(
                "has class=note   = {}",
                html.contains("class=\"note\"")
            ),
            format!(
                "has class=danger = {}",
                html.contains("class=\"danger\"")
            ),
        ]
    });

    // ── Image with custom class ─────────────────────────────────────
    support::task_with_output("Image with .class syntax", || {
        let md = r#"![Photo](photo.jpg).class="img-fluid rounded""#;
        let config = HtmlConfig {
            allow_unsafe_html: true,
            ..HtmlConfig::default()
        };
        let html = generate_html(md, &config).unwrap();
        vec![
            format!("has img-fluid = {}", html.contains("img-fluid")),
            format!("has alt=Photo = {}", html.contains("Photo")),
        ]
    });

    // ── Mixed custom syntax ─────────────────────────────────────────
    support::task_with_output(
        "Mixed: block + image + standard markdown",
        || {
            let md = "\
:::info\nCheck the **docs** for details.\n:::\n\n\
# Heading\n\n\
Regular paragraph.\n";
            let html =
                generate_html(md, &HtmlConfig::default()).unwrap();
            vec![
                format!(
                    "has class=info = {}",
                    html.contains("class=\"info\"")
                ),
                format!("has <h1>       = {}", html.contains("<h1>")),
                format!("has <p>        = {}", html.contains("<p>")),
            ]
        },
    );

    support::summary(4);
}
