// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! HtmlConfig builder and validation.
//!
//! Run: `cargo run --example config`

#[path = "support.rs"]
mod support;

use html_generator::{HtmlConfig, HtmlConfigBuilder};

fn main() {
    support::header("html-generator -- config");

    // ── Default configuration ────────────────────────────────────────
    support::task_with_output("Inspect default config", || {
        let config = HtmlConfig::default();
        vec![
            format!(
                "syntax_highlighting: {}",
                config.enable_syntax_highlighting
            ),
            format!("syntax_theme:        {:?}", config.syntax_theme),
            format!("minify_output:       {}", config.minify_output),
            format!(
                "add_aria_attributes: {}",
                config.add_aria_attributes
            ),
            format!(
                "structured_data:     {}",
                config.generate_structured_data
            ),
            format!("generate_toc:        {}", config.generate_toc),
            format!("language:            {}", config.language),
            format!(
                "max_input_size:      {} bytes",
                config.max_input_size
            ),
        ]
    });

    // ── Builder pattern ──────────────────────────────────────────────
    support::task_with_output("Build config with builder", || {
        match HtmlConfigBuilder::new()
            .with_syntax_highlighting(true, Some("monokai".to_string()))
            .with_language("en-US")
            .build()
        {
            Ok(config) => vec![
                format!("syntax_theme: {:?}", config.syntax_theme),
                format!("language:     {}", config.language),
            ],
            Err(e) => vec![format!("Build error: {e}")],
        }
    });

    // ── Builder with invalid language ────────────────────────────────
    support::task_with_output("Reject invalid language code", || {
        match HtmlConfig::builder().with_language("nope").build() {
            Ok(_) => vec!["BUG: should have been rejected".to_string()],
            Err(e) => vec![format!("Correctly rejected: {e}")],
        }
    });

    // ── Validate existing config ─────────────────────────────────────
    support::task_with_output(
        "Validate a manually constructed config",
        || {
            let valid = HtmlConfig::default();
            let invalid = HtmlConfig {
                max_input_size: 100, // below minimum
                ..HtmlConfig::default()
            };
            vec![
                format!(
                    "Default config valid: {}",
                    valid.validate().is_ok()
                ),
                format!(
                    "Undersized config valid: {}",
                    invalid.validate().is_ok()
                ),
            ]
        },
    );

    // ── Syntax highlighting toggle ───────────────────────────────────
    support::task_with_output("Disable syntax highlighting", || {
        match HtmlConfig::builder()
            .with_syntax_highlighting(false, None)
            .with_language("en-GB")
            .build()
        {
            Ok(config) => vec![
                format!(
                    "highlighting: {}",
                    config.enable_syntax_highlighting
                ),
                format!("theme:        {:?}", config.syntax_theme),
            ],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    support::summary(5);
}
