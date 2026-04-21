// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! HTML minification: compress output by stripping whitespace.
//!
//! Run: `cargo run --example minify`

#[path = "support.rs"]
mod support;

use html_generator::performance::minify_html_string;
use html_generator::{generate_html, HtmlConfig};

fn main() {
    support::header("html-generator -- minify");

    // ── Minify a fragment ───────────────────────────────────────────
    support::task_with_output("Minify HTML fragment", || {
        let html = "<html>  <body>  <p>Hello</p>  </body>  </html>";
        let minified = minify_html_string(html).unwrap();
        vec![
            format!("before = {} bytes", html.len()),
            format!("after  = {} bytes", minified.len()),
            format!("saved  = {} bytes", html.len() - minified.len()),
            format!("output = {minified}"),
        ]
    });

    // ── Minify a full document ──────────────────────────────────────
    support::task_with_output("Minify full HTML document", || {
        let html = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Test</title>
  </head>
  <body>
    <h1>Hello</h1>
    <p>World</p>
  </body>
</html>"#;
        let minified = minify_html_string(html).unwrap();
        let pct =
            100.0 - (minified.len() as f64 / html.len() as f64 * 100.0);
        vec![
            format!("before  = {} bytes", html.len()),
            format!("after   = {} bytes", minified.len()),
            format!("savings = {pct:.1}%"),
        ]
    });

    // ── Minify via pipeline config ──────────────────────────────────
    support::task_with_output(
        "Minify via generate_html pipeline",
        || {
            let md = "# Title\n\nParagraph with **bold** text.";
            let normal =
                generate_html(md, &HtmlConfig::default()).unwrap();
            let config = HtmlConfig {
                minify_output: true,
                ..HtmlConfig::default()
            };
            let minified = generate_html(md, &config).unwrap();
            vec![
                format!("normal   = {} bytes", normal.len()),
                format!("minified = {} bytes", minified.len()),
            ]
        },
    );

    support::summary(3);
}
