// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Accessibility: ARIA attribute injection and WCAG validation.
//!
//! Run: `cargo run --example accessibility`

#[path = "support.rs"]
mod support;

use html_generator::accessibility::{
    add_aria_attributes, validate_wcag, AccessibilityConfig,
};

fn main() {
    support::header("html-generator -- accessibility");

    // ── Add ARIA attributes to a navigation element ──────────────────
    support::task_with_output(
        "Add ARIA attributes to nav element",
        || {
            let html = "<nav><a href=\"/\">Home</a><a href=\"/about\">About</a></nav>";
            match add_aria_attributes(html, None) {
                Ok(enhanced) => vec![
                    format!("Input:  {html}"),
                    format!("Output: {enhanced}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── Add ARIA attributes to a form ────────────────────────────────
    support::task_with_output(
        "Add ARIA attributes to form element",
        || {
            let html = "<form><input type=\"text\"><button>Submit</button></form>";
            match add_aria_attributes(html, None) {
                Ok(enhanced) => vec![
                    format!("Input:  {html}"),
                    format!("Output: {enhanced}"),
                ],
                Err(e) => vec![format!("Error: {e}")],
            }
        },
    );

    // ── WCAG validation with default config ──────────────────────────
    support::task_with_output(
        "Validate WCAG compliance (AA level)",
        || {
            let html = "<html lang=\"en-GB\"><head><title>Test</title></head>\
                     <body><h1>Main heading</h1><p>Content</p></body></html>";
            let config = AccessibilityConfig::default();
            match validate_wcag(html, &config, None) {
                Ok(report) => vec![
                    format!(
                        "Elements checked: {}",
                        report.elements_checked
                    ),
                    format!("Issues found: {}", report.issue_count),
                    format!(
                        "Check duration: {}ms",
                        report.check_duration_ms
                    ),
                ],
                Err(e) => vec![format!("Validation error: {e}")],
            }
        },
    );

    // ── WCAG validation on problematic HTML ──────────────────────────
    support::task_with_output("Detect accessibility issues", || {
        let html =
            "<html><body><h3>Skipped heading levels</h3></body></html>";
        let config = AccessibilityConfig::default();
        match validate_wcag(html, &config, None) {
            Ok(report) => {
                let mut lines = vec![format!(
                    "Issues found: {}",
                    report.issue_count
                )];
                for issue in report.issues.iter().take(3) {
                    lines.push(format!("  - {}", issue.message));
                }
                lines
            }
            Err(e) => vec![format!("Validation error: {e}")],
        }
    });

    support::summary(4);
}
