// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Emoji accessibility: map emojis to descriptive ARIA labels.
//!
//! Uses compile-time bundled emoji data (`include_str!`) so the
//! mapping is always available regardless of working directory.
//!
//! Run: `cargo run --example emojis`

#[path = "support.rs"]
mod support;

use html_generator::accessibility::add_aria_attributes;
use html_generator::emojis::bundled_emoji_sequences;

fn main() {
    support::header("html-generator -- emojis");

    // ── Bundled emoji data ──────────────────────────────────────────
    support::task_with_output("Inspect bundled emoji map", || {
        let map = bundled_emoji_sequences();
        let fr =
            map.get("\u{1F1EB}\u{1F1F7}").cloned().unwrap_or_default();
        let gb =
            map.get("\u{1F1EC}\u{1F1E7}").cloned().unwrap_or_default();
        let tie = map.get("\u{1F454}").cloned().unwrap_or_default();
        vec![
            format!("total entries = {}", map.len()),
            format!("  \u{1F1EB}\u{1F1F7} -> {fr}"),
            format!("  \u{1F1EC}\u{1F1E7} -> {gb}"),
            format!("  \u{1F454}  -> {tie}"),
        ]
    });

    // ── Button with emoji ───────────────────────────────────────────
    support::task_with_output("ARIA label for emoji button", || {
        let html = "<button>\u{1F44D}</button>";
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

    // ── Button with text + emoji ────────────────────────────────────
    support::task_with_output(
        "ARIA label for text+emoji button",
        || {
            let html = "<button>Like \u{1F44D}</button>";
            let enhanced = add_aria_attributes(html, None).unwrap();
            vec![
                format!("input  = {html}"),
                format!(
                    "has aria-label = {}",
                    enhanced.contains("aria-label")
                ),
            ]
        },
    );

    // ── Multiple emoji buttons ──────────────────────────────────────
    support::task_with_output(
        "Multiple emoji buttons in one pass",
        || {
            let html = "<button>\u{2B06}\u{FE0F}</button><button>\u{26A1}</button>";
            let enhanced = add_aria_attributes(html, None).unwrap();
            let aria_count = enhanced.matches("aria-label").count();
            vec![
                format!("input buttons  = 2"),
                format!("aria-labels    = {aria_count}"),
            ]
        },
    );

    support::summary(4);
}
