// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Configuration: HtmlConfig builder, validation, and field inspection.
//!
//! Run: `cargo run --example config`

#[path = "support.rs"]
mod support;

use html_generator::{
    generate_html, validate_language_code, HtmlConfig,
};

fn main() {
    support::header("html-generator -- config");

    // ── Default configuration ───────────────────────────────────────
    support::task_with_output("Inspect default HtmlConfig", || {
        let c = HtmlConfig::default();
        vec![
            format!(
                "syntax_highlighting = {}",
                c.enable_syntax_highlighting
            ),
            format!("syntax_theme        = {:?}", c.syntax_theme),
            format!("minify_output       = {}", c.minify_output),
            format!("add_aria            = {}", c.add_aria_attributes),
            format!(
                "structured_data     = {}",
                c.generate_structured_data
            ),
            format!("generate_toc        = {}", c.generate_toc),
            format!("language            = {}", c.language),
            format!("max_input_size      = {} bytes", c.max_input_size),
            format!("allow_unsafe_html   = {}", c.allow_unsafe_html),
        ]
    });

    // ── Builder pattern ─────────────────────────────────────────────
    support::task_with_output(
        "Build config with builder pattern",
        || {
            let config = HtmlConfig::builder()
                .with_syntax_highlighting(true, Some("monokai".into()))
                .with_language("en-US")
                .build()
                .unwrap();
            vec![
                format!("theme    = {:?}", config.syntax_theme),
                format!("language = {}", config.language),
            ]
        },
    );

    // ── Language code validation ─────────────────────────────────────
    support::task_with_output("Language code validation", || {
        let codes = ["en-GB", "fr-FR", "en", "123", "en_US"];
        codes
            .iter()
            .map(|c| {
                format!(
                    "{c:<6} -> valid = {}",
                    validate_language_code(c)
                )
            })
            .collect()
    });

    // ── Config affects output ───────────────────────────────────────
    support::task_with_output("Config toggles change output", || {
        let md = "# Test\n\nParagraph.";
        let plain = generate_html(md, &HtmlConfig::default()).unwrap();
        let minified = generate_html(
            md,
            &HtmlConfig {
                minify_output: true,
                ..HtmlConfig::default()
            },
        )
        .unwrap();
        vec![
            format!("default_len  = {} bytes", plain.len()),
            format!("minified_len = {} bytes", minified.len()),
            format!("smaller      = {}", minified.len() < plain.len()),
        ]
    });

    // ── Direct struct construction ──────────────────────────────────
    support::task_with_output("Direct struct construction", || {
        let config = HtmlConfig {
            enable_syntax_highlighting: false,
            syntax_theme: None,
            minify_output: true,
            add_aria_attributes: true,
            generate_structured_data: false,
            generate_toc: false,
            allow_unsafe_html: false,
            sanitize_html: false,
            generate_full_document: false,
            max_input_size: 1024,
            language: "de-DE".into(),
            max_buffer_size: 8 * 1024 * 1024,
            encoding: "utf-8".into(),
            enable_math: false,
            enable_diagrams: false,
        };
        vec![
            format!("language = {}", config.language),
            format!("max_in   = {} bytes", config.max_input_size),
        ]
    });

    support::summary(5);
}
