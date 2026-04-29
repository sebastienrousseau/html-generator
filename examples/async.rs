// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Asynchronous HTML generation using the `async` feature.
//!
//! Run: `cargo run --example async --features async`

#[path = "support.rs"]
mod support;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    support::header("html-generator -- async");

    // ── Async generation ────────────────────────────────────────────
    let html = html_generator::performance::async_generate_html(
        "# Async Hello\n\nGenerated in a thread pool.",
    )
    .await
    .unwrap();

    support::task_with_output("Generate HTML asynchronously", || {
        vec![
            format!("length   = {} bytes", html.len()),
            format!("has <h1> = {}", html.contains("<h1>")),
            format!("has <p>  = {}", html.contains("<p>")),
        ]
    });

    support::summary(1);
}

#[cfg(not(feature = "async"))]
fn main() {
    support::header("html-generator -- async");
    println!("  (skipped: rerun with --features async)");
    support::summary(0);
}
