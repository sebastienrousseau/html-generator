// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Full pipeline: every processing stage enabled in a single pass.
//!
//! Demonstrates ARIA injection, TOC generation, JSON-LD structured
//! data, and HTML minification working together.
//!
//! Run: `cargo run --example pipeline`

#[path = "support.rs"]
mod support;

use html_generator::{
    generate_html, generate_html_with_diagnostics, HtmlConfig,
};

fn main() {
    support::header("html-generator -- pipeline");

    let markdown = "\
[[TOC]]\n\n\
# Introduction\n\n\
Welcome to the guide.\n\n\
## Getting Started\n\n\
Follow these steps.\n\n\
## Advanced Usage\n\n\
For power users.\n";

    // ── Full pipeline (all features) ────────────────────────────────
    support::task_with_output(
        "Generate with all features enabled",
        || {
            let config = HtmlConfig {
                add_aria_attributes: true,
                generate_toc: true,
                generate_structured_data: true,
                minify_output: true,
                ..HtmlConfig::default()
            };
            let html = generate_html(markdown, &config).unwrap();
            vec![
                format!("length    = {} bytes", html.len()),
                format!("has_toc   = {}", html.contains("<ul>")),
                format!("has_aria  = {}", html.contains("aria-")),
                format!("has_jsonld = {}", html.contains("ld+json")),
            ]
        },
    );

    // ── Pipeline with diagnostics ───────────────────────────────────
    support::task_with_output(
        "Pipeline with diagnostics inspection",
        || {
            let config = HtmlConfig {
                add_aria_attributes: true,
                generate_toc: true,
                generate_structured_data: true,
                minify_output: true,
                ..HtmlConfig::default()
            };
            let output =
                generate_html_with_diagnostics(markdown, &config)
                    .unwrap();
            let mut lines = vec![
                format!("html_len    = {} bytes", output.html.len()),
                format!("diagnostics = {}", output.diagnostics.len()),
            ];
            for d in &output.diagnostics {
                lines.push(format!("  {d}"));
            }
            lines
        },
    );

    // ── Full HTML5 document output ────────────────────────────────
    support::task_with_output("Generate full HTML5 document", || {
        let config = HtmlConfig {
            generate_full_document: true,
            generate_structured_data: true,
            language: "en-US".into(),
            ..HtmlConfig::default()
        };
        let html = generate_html(markdown, &config).unwrap();
        vec![
            format!(
                "has <!DOCTYPE>  = {}",
                html.contains("<!DOCTYPE html>")
            ),
            format!(
                "has lang=en-US  = {}",
                html.contains("lang=\"en-US\"")
            ),
            format!("has <head>      = {}", html.contains("<head>")),
            format!("has <body>      = {}", html.contains("<body>")),
            format!("has ld+json     = {}", html.contains("ld+json")),
        ]
    });

    // ── Sanitized HTML mode ─────────────────────────────────────────
    support::task_with_output(
        "Sanitize unsafe HTML via ammonia",
        || {
            let unsafe_md = "# Title\n\n<script>alert('xss')</script>\n\n<div class=\"safe\">Allowed</div>";
            let config = HtmlConfig {
                allow_unsafe_html: true,
                sanitize_html: true,
                ..HtmlConfig::default()
            };
            let html = generate_html(unsafe_md, &config).unwrap();
            vec![
                format!(
                    "has <script>    = {}",
                    html.contains("<script>")
                ),
                format!("has <div>       = {}", html.contains("<div")),
                format!(
                    "script stripped = {}",
                    !html.contains("<script>")
                ),
            ]
        },
    );

    // ── Minimal pipeline (defaults) ─────────────────────────────────
    support::task_with_output(
        "Default config (syntax highlighting only)",
        || {
            let html = generate_html(markdown, &HtmlConfig::default())
                .unwrap();
            vec![
                format!("length   = {} bytes", html.len()),
                format!("minified = {}", !html.contains('\n')),
            ]
        },
    );

    support::summary(5);
}
