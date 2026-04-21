// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Accessibility: ARIA attribute injection and WCAG validation.
//!
//! Demonstrates automatic ARIA enhancement for buttons, navs,
//! forms, and inputs, plus WCAG AA/AAA compliance checking.
//!
//! Run: `cargo run --example accessibility`

#[path = "support.rs"]
mod support;

use html_generator::accessibility::{
    add_aria_attributes, validate_wcag, AccessibilityConfig, WcagLevel,
};

fn main() {
    support::header("html-generator -- accessibility");

    // ── Button ARIA enhancement ─────────────────────────────────────
    support::task_with_output("Add ARIA to buttons", || {
        let html = r#"<button>Submit</button>"#;
        let enhanced = add_aria_attributes(html, None).unwrap();
        vec![
            format!("input  = {html}"),
            format!("output = {enhanced}"),
            format!(
                "has aria-label = {}",
                enhanced.contains("aria-label")
            ),
        ]
    });

    // ── Navigation ARIA enhancement ─────────────────────────────────
    support::task_with_output("Add ARIA to navigation", || {
        let html = r#"<nav><ul><li>Home</li><li>About</li></ul></nav>"#;
        let enhanced = add_aria_attributes(html, None).unwrap();
        vec![
            format!(
                "has role=navigation = {}",
                enhanced.contains("role=\"navigation\"")
            ),
            format!(
                "has aria-label     = {}",
                enhanced.contains("aria-label")
            ),
        ]
    });

    // ── Form ARIA enhancement ───────────────────────────────────────
    support::task_with_output("Add ARIA to forms", || {
        let html = r#"<form><input type="checkbox"></form>"#;
        let enhanced = add_aria_attributes(html, None).unwrap();
        vec![
            format!(
                "has aria-labelledby = {}",
                enhanced.contains("aria-labelledby")
            ),
            format!("output_length      = {} bytes", enhanced.len()),
        ]
    });

    // ── WCAG AA validation ──────────────────────────────────────────
    support::task_with_output("Validate WCAG AA compliance", || {
        let html = r#"<html lang="en"><body><h1>Title</h1><h2>Sub</h2></body></html>"#;
        let config = AccessibilityConfig::default();
        let report = validate_wcag(html, &config, None).unwrap();
        vec![
            format!("issues = {}", report.issue_count),
            format!("level  = {:?}", config.wcag_level),
        ]
    });

    // ── WCAG AAA validation ─────────────────────────────────────────
    support::task_with_output("Validate WCAG AAA (stricter)", || {
        let html = r#"<html lang="en"><body><h1>Title</h1><h3>Skipped</h3></body></html>"#;
        let config = AccessibilityConfig {
            wcag_level: WcagLevel::AAA,
            ..AccessibilityConfig::default()
        };
        let report = validate_wcag(html, &config, None).unwrap();
        vec![
            format!("issues = {}", report.issue_count),
            format!(
                "heading skip detected = {}",
                report.issue_count > 0
            ),
        ]
    });

    // ── Multiple elements in one pass ───────────────────────────────
    support::task_with_output("Enhance multiple element types", || {
        let html = r#"<nav><ul><li>Home</li></ul></nav><button>Click</button><form><input type="text"></form>"#;
        let enhanced = add_aria_attributes(html, None).unwrap();
        vec![
            format!(
                "nav aria   = {}",
                enhanced.contains("role=\"navigation\"")
            ),
            format!("btn label  = {}", enhanced.contains("aria-label")),
            format!(
                "form label = {}",
                enhanced.contains("aria-labelledby")
            ),
        ]
    });

    support::summary(6);
}
