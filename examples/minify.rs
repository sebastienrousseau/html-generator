// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! In-memory HTML minification.
//!
//! Run: `cargo run --example minify`

#[path = "support.rs"]
mod support;

use html_generator::performance::minify_html_string;

fn main() {
    support::header("html-generator -- minify");

    // ── Minify a small HTML fragment ─────────────────────────────────
    support::task_with_output("Minify simple HTML", || {
        let html = "<div>  <p>  Hello,   world!  </p>  </div>";
        match minify_html_string(html) {
            Ok(minified) => vec![
                format!("Input:    {} bytes", html.len()),
                format!("Minified: {} bytes", minified.len()),
                format!("Output:   {minified}"),
            ],
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── Minify a full HTML document ──────────────────────────────────
    support::task_with_output("Minify full document", || {
        let html = r#"<!DOCTYPE html>
<html lang="en-GB">
  <head>
    <meta charset="utf-8">
    <title>Test Page</title>
  </head>
  <body>
    <h1>Heading</h1>
    <p>
      Paragraph with   extra   whitespace.
    </p>
  </body>
</html>"#;
        match minify_html_string(html) {
            Ok(minified) => {
                let savings = html.len() - minified.len();
                let pct = (savings as f64 / html.len() as f64) * 100.0;
                vec![
                    format!("Input:   {} bytes", html.len()),
                    format!("Output:  {} bytes", minified.len()),
                    format!("Savings: {savings} bytes ({pct:.1}%)"),
                ]
            }
            Err(e) => vec![format!("Error: {e}")],
        }
    });

    // ── Minify via pipeline config ───────────────────────────────────
    support::task_with_output(
        "Minify via generate_html pipeline",
        || {
            let md = "# Heading\n\nA paragraph with content.";
            let config = html_generator::HtmlConfig {
                minify_output: true,
                ..html_generator::HtmlConfig::default()
            };
            match html_generator::generate_html(md, &config) {
                Ok(html) => vec![
                    format!(
                        "Minified output length: {} bytes",
                        html.len()
                    ),
                    format!("Output: {html}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    support::summary(3);
}
