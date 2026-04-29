// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2025 HTML Generator. All rights reserved.

//! Fixture-driven integration tests.
//!
//! Each `.txt` file in `tests/fixtures/` is a Markdown document that
//! exercises a specific category of input (i18n, edge cases, extensions,
//! stress).  The tests run the full `generate_html` pipeline and assert
//! that it succeeds without panicking and produces non-empty output.

use html_generator::{
    generate_html, generate_html_with_diagnostics, HtmlConfig,
};
use std::fs;
use std::path::Path;

/// Load a fixture file from `tests/fixtures/`.
fn load_fixture(name: &str) -> String {
    let path = Path::new("tests/fixtures").join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("failed to read fixture {}: {e}", path.display())
    })
}

// ── Core rendering ──────────────────────────────────────────────────

#[test]
fn fixture_edge_cases() {
    let md = load_fixture("edge_cases.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "edge_cases produced empty output");
}

#[test]
fn fixture_escaped_characters() {
    let md = load_fixture("escaped_characters.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty());
}

#[test]
fn fixture_strong_and_em() {
    let md = load_fixture("strong-and-em-together.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("<strong>") || html.contains("<em>"));
}

#[test]
fn fixture_blockquotes_with_code() {
    let md = load_fixture("blockquotes-with-code-blocks.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("<blockquote>") || html.contains("<code>"));
}

#[test]
fn fixture_nested_blockquotes() {
    let md = load_fixture("nested-blockquotes.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("<blockquote>"));
}

#[test]
fn fixture_links_inline() {
    let md = load_fixture("links-inline.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("<a "));
}

#[test]
fn fixture_links_reference() {
    let md = load_fixture("links-reference.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("<a "));
}

#[test]
fn fixture_auto_links() {
    let md = load_fixture("auto-links.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty());
}

#[test]
fn fixture_ordered_and_unordered_lists() {
    let md = load_fixture("ordered-and-unordered-list.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("<li>"));
}

#[test]
fn fixture_horizontal_rules() {
    let md = load_fixture("horizontal-rules.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty());
}

#[test]
fn fixture_code_syntax_highlighting() {
    let md = load_fixture("code_syntax_highlighting.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("<code") || html.contains("<pre"));
}

// ── Emoji ───────────────────────────────────────────────────────────

#[test]
fn fixture_emoji_content() {
    let md = load_fixture("emoji_content.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "emoji content produced empty output");
    // Emoji characters should pass through to output
    assert!(html.contains('\u{1F389}') || html.contains('\u{1F680}'));
}

// ── i18n / Unicode ──────────────────────────────────────────────────

#[test]
fn fixture_arabic() {
    let md = load_fixture("arabic.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "Arabic content produced empty output");
    assert!(html.contains("بايثون"), "Arabic text missing from output");
}

#[test]
fn fixture_japanese() {
    let md = load_fixture("japanese.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "Japanese content produced empty output");
    assert!(
        html.contains("パイソン"),
        "Japanese text missing from output"
    );
}

#[test]
fn fixture_bidi() {
    let md = load_fixture("bidi.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "Bidi content produced empty output");
    // Contains both RTL Arabic and LTR English
    assert!(html.contains("بايثون"));
    assert!(html.contains("Python"));
}

#[test]
fn fixture_russian() {
    let md = load_fixture("russian.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "Russian content produced empty output");
}

// ── Line endings ────────────────────────────────────────────────────

#[test]
fn fixture_crlf_line_ends() {
    let md = load_fixture("CRLF_line_ends.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "CRLF content produced empty output");
}

// ── Extensions ──────────────────────────────────────────────────────

#[test]
fn fixture_github_flavored() {
    let md = load_fixture("github_flavored.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty());
}

#[test]
fn fixture_toc_nested() {
    let md = load_fixture("toc_nested.txt");
    let config = HtmlConfig {
        generate_toc: true,
        ..HtmlConfig::default()
    };
    let html = generate_html(&md, &config).unwrap();
    assert!(
        html.contains("<h1>")
            || html.contains("<h2>")
            || html.contains("<h3>")
    );
}

#[test]
fn fixture_toc_out_of_order() {
    let md = load_fixture("toc_out_of_order.txt");
    let config = HtmlConfig {
        generate_toc: true,
        ..HtmlConfig::default()
    };
    let html = generate_html(&md, &config).unwrap();
    assert!(!html.is_empty());
}

#[test]
fn fixture_toc_invalid() {
    let md = load_fixture("toc_invalid.txt");
    let config = HtmlConfig {
        generate_toc: true,
        ..HtmlConfig::default()
    };
    // Should not panic even with invalid TOC input
    let _html = generate_html(&md, &config).unwrap();
}

// ── Stress / large input ────────────────────────────────────────────

#[test]
fn fixture_large_markdown() {
    let md = load_fixture("large_markdown.txt");
    let config = HtmlConfig {
        add_aria_attributes: true,
        generate_toc: true,
        minify_output: true,
        ..HtmlConfig::default()
    };
    let output = generate_html_with_diagnostics(&md, &config).unwrap();
    assert!(
        !output.html.is_empty(),
        "large markdown produced empty output"
    );
    assert!(
        output.html.len() > 100,
        "large markdown output suspiciously small"
    );
}

// ── Security: XSS vectors ───────────────────────────────────────────

#[test]
fn fixture_xss_default_mode() {
    let md = load_fixture("xss_vectors.txt");
    // NOTE: mdx-gen passes raw HTML through regardless of the comrak
    // unsafe flag.  The safe path is sanitize_html=true (ammonia).
    // This test verifies the default pipeline doesn't panic.
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty());
}

#[test]
fn fixture_xss_ammonia_strips_dangerous_tags() {
    let md = load_fixture("xss_vectors.txt");
    let config = HtmlConfig {
        allow_unsafe_html: true,
        sanitize_html: true,
        ..HtmlConfig::default()
    };
    let html = generate_html(&md, &config).unwrap();
    assert!(!html.contains("<script"), "ammonia must strip <script>");
    assert!(!html.contains("<iframe"), "ammonia must strip <iframe>");
    assert!(!html.contains("<svg"), "ammonia must strip <svg>");
    assert!(
        !html.contains("onerror"),
        "ammonia must strip event handlers"
    );
    assert!(
        !html.contains("onload"),
        "ammonia must strip onload handlers"
    );
}

#[test]
fn fixture_xss_sanitized_by_ammonia() {
    let md = load_fixture("xss_vectors.txt");
    let config = HtmlConfig {
        allow_unsafe_html: true,
        sanitize_html: true,
        ..HtmlConfig::default()
    };
    let html = generate_html(&md, &config).unwrap();
    assert!(
        !html.contains("<script>"),
        "ammonia must strip script tags"
    );
    assert!(!html.contains("<iframe"), "ammonia must strip iframes");
}

// ── Deep nesting ────────────────────────────────────────────────────

#[test]
fn fixture_deep_nesting() {
    let md = load_fixture("deep_nesting.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(
        html.contains("<blockquote>"),
        "nested blockquotes must render"
    );
    assert!(html.contains("<li>"), "nested lists must render");
}

// ── Duplicate elements (DomReplacer correctness) ────────────────────

#[test]
fn fixture_duplicate_elements_aria() {
    let md = load_fixture("duplicate_elements.txt");
    let config = HtmlConfig {
        allow_unsafe_html: true,
        add_aria_attributes: true,
        ..HtmlConfig::default()
    };
    let html = generate_html(&md, &config).unwrap();
    let nav_count = html.matches("role=\"navigation\"").count();
    assert!(
        nav_count >= 2,
        "both navs must get ARIA (got {nav_count})"
    );
    let btn_count = html.matches("aria-label").count();
    assert!(
        btn_count >= 3,
        "all buttons must get aria-label (got {btn_count})"
    );
}

// ── Custom syntax ───────────────────────────────────────────────────

#[test]
fn fixture_custom_syntax_blocks() {
    let md = load_fixture("custom_syntax_blocks.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(
        html.contains("class=\"warning\""),
        ":::warning block must render"
    );
    assert!(
        html.contains("class=\"note\""),
        ":::note block must render"
    );
    assert!(
        html.contains("class=\"danger\""),
        ":::danger block must render"
    );
}

// ── Front matter edge cases ─────────────────────────────────────────

#[test]
fn fixture_frontmatter_edge_cases() {
    use html_generator::utils::extract_front_matter_data;

    let md = load_fixture("frontmatter_edge_cases.txt");
    let (data, body) = extract_front_matter_data(&md).unwrap();
    assert_eq!(data["title"], "Front Matter Edge Cases");
    assert_eq!(data["author"], "Jane Doe");
    assert!(!body.is_empty(), "body must not be empty");
}

// ── i18n: CJK + Devanagari ─────────────────────────────────────────

#[test]
fn fixture_chinese() {
    let md = load_fixture("chinese.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("中文测试"), "Chinese heading must render");
    assert!(html.contains("内存安全"), "Chinese body text must render");
    assert!(
        html.contains("所有权"),
        "Chinese table cell text must render"
    );
}

#[test]
fn fixture_korean() {
    let md = load_fixture("korean.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("한국어"), "Korean heading must render");
}

#[test]
fn fixture_hindi() {
    let md = load_fixture("hindi.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(html.contains("हिंदी"), "Hindi heading must render");
}

#[test]
fn fixture_unicode_edge_cases() {
    let md = load_fixture("unicode_edge_cases.txt");
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty());
    assert!(
        html.contains("🇫🇷") || html.contains("🇬🇧"),
        "flag emoji must survive"
    );
}

// ── Performance: large table ────────────────────────────────────────

#[test]
fn fixture_huge_table() {
    let md = load_fixture("huge_table.txt");
    // Verify large table input doesn't panic or timeout
    let html = generate_html(&md, &HtmlConfig::default()).unwrap();
    assert!(!html.is_empty(), "large table must produce output");
    // Note: mdx-gen may render tables as <table> or as pipe-delimited text
    // depending on the markdown processor. The key assertion is no panic.
    assert!(
        html.contains("Alpha") && html.contains("Echo"),
        "table cell content must be present in output"
    );
}

// ── Full document generation ────────────────────────────────────────

#[test]
fn fixture_full_document_mode() {
    let md = load_fixture("full_document_mode.txt");
    let config = HtmlConfig {
        generate_full_document: true,
        language: "en-GB".into(),
        ..HtmlConfig::default()
    };
    let html = generate_html(&md, &config).unwrap();
    assert!(html.contains("<!DOCTYPE html>"), "must have doctype");
    assert!(html.contains("lang=\"en-GB\""), "must have lang");
    assert!(html.contains("<head>"), "must have head");
    assert!(html.contains("<body>"), "must have body");
    assert!(html.contains("<title>"), "must extract title from h1");
}

// ── Real-world blog post ────────────────────────────────────────────

#[test]
fn fixture_real_world_blog() {
    let md = load_fixture("real_world_blog.txt");
    let config = HtmlConfig {
        add_aria_attributes: true,
        generate_toc: true,
        generate_full_document: true,
        ..HtmlConfig::default()
    };
    let output = generate_html_with_diagnostics(&md, &config).unwrap();
    assert!(output.html.contains("<!DOCTYPE html>"));
    assert!(output.html.contains("<code"), "code blocks must render");
    assert!(
        output.html.contains("class=\"warning\""),
        "custom block must render"
    );
    assert!(
        output.html.len() > 1000,
        "realistic blog must produce substantial output"
    );
}

#[test]
fn fixture_real_world_blog_minified() {
    let md = load_fixture("real_world_blog.txt");
    let normal = generate_html(&md, &HtmlConfig::default()).unwrap();
    let config = HtmlConfig {
        minify_output: true,
        ..HtmlConfig::default()
    };
    let minified = generate_html(&md, &config).unwrap();
    assert!(
        minified.len() < normal.len(),
        "minified ({}) must be smaller than normal ({})",
        minified.len(),
        normal.len()
    );
}

// ── Full pipeline on all fixtures ───────────────────────────────────

#[test]
fn all_fixtures_survive_full_pipeline() {
    let fixture_dir = Path::new("tests/fixtures");
    let entries: Vec<_> = fs::read_dir(fixture_dir)
        .expect("tests/fixtures directory missing")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "txt")
        })
        .collect();

    assert!(
        !entries.is_empty(),
        "no fixture files found in tests/fixtures/"
    );

    let config = HtmlConfig {
        add_aria_attributes: true,
        generate_toc: true,
        generate_structured_data: false, // fragments lack <title>
        minify_output: true,
        ..HtmlConfig::default()
    };

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        let md = fs::read_to_string(&path).unwrap();

        if md.trim().is_empty() {
            continue; // skip empty fixtures
        }

        let result = generate_html(&md, &config);
        assert!(
            result.is_ok(),
            "fixture {name} failed: {}",
            result.unwrap_err()
        );
    }
}
