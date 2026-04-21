// Copyright © 2025 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! HTML generation module for converting Markdown to HTML.
//!
//! This module provides functions to generate HTML from Markdown content
//! using the `mdx-gen` library. It supports various Markdown extensions
//! and custom configuration options.

use crate::{
    accessibility::add_aria_attributes,
    error::HtmlError,
    extract_front_matter,
    performance::minify_html_string,
    seo::{escape_html, generate_structured_data_from_doc},
    utils::generate_table_of_contents,
    Result,
};
use log::warn;
use mdx_gen::{process_markdown, MarkdownOptions, Options};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use std::fmt;

/// Severity level for a processing diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Informational — a step succeeded with notable metrics.
    Info,
    /// A non-fatal issue — the pipeline continued with a fallback.
    Warning,
    /// A step failed entirely and was skipped.
    Error,
}

/// A diagnostic emitted when a post-processing step fails non-fatally.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Which pipeline step produced this diagnostic.
    pub step: &'static str,
    /// Severity.
    pub level: DiagnosticLevel,
    /// Human-readable description of what went wrong.
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}: {}", self.level, self.step, self.message)
    }
}

/// The result of [`generate_html_with_diagnostics`]: final HTML plus any
/// warnings from post-processing steps that failed non-fatally.
#[derive(Debug, Clone)]
pub struct HtmlOutput {
    /// The generated HTML content.
    pub html: String,
    /// Diagnostics from pipeline steps that were skipped or degraded.
    /// Empty when every step succeeded.
    pub diagnostics: Vec<Diagnostic>,
}

/// Regex matching triple-colon custom class blocks.
static CUSTOM_CLASS_REGEX: Lazy<Option<Regex>> =
    Lazy::new(|| Regex::new(r":::(\w+)\n([\s\S]*?)\n:::").ok());

/// Regex matching image-with-class syntax: `![alt](url).class="cls"`.
static IMAGE_CLASS_REGEX: Lazy<Option<Regex>> = Lazy::new(|| {
    Regex::new(r#"!\[(.*?)\]\((.*?)\)\.class="(.*?)""#).ok()
});

/// Generate HTML from Markdown content using `mdx-gen`.
///
/// This function takes Markdown content and a configuration object,
/// converts the Markdown into HTML, and applies the full processing
/// pipeline based on configuration:
///
/// 1. Markdown → HTML conversion (with extensions)
/// 2. Accessibility: adds ARIA attributes if enabled
/// 3. Table of contents: injects TOC at `[[TOC]]` placeholder
/// 4. Structured data: appends JSON-LD script tag
/// 5. Minification: compresses output if enabled
///
/// Non-fatal failures in steps 2–5 are silently skipped. Use
/// [`generate_html_with_diagnostics`] to inspect which steps failed.
pub fn generate_html(
    markdown: &str,
    config: &crate::HtmlConfig,
) -> Result<String> {
    generate_html_with_diagnostics(markdown, config).map(|o| o.html)
}

/// Like [`generate_html`], but returns an [`HtmlOutput`] that includes
/// diagnostics for any post-processing steps that failed non-fatally.
pub fn generate_html_with_diagnostics(
    markdown: &str,
    config: &crate::HtmlConfig,
) -> Result<HtmlOutput> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Step 1: Core Markdown → HTML (fatal on failure)
    let mut html = markdown_to_html_impl(markdown, config)?;

    // Step 2: HTML sanitization via ammonia (when raw HTML is allowed)
    if config.allow_unsafe_html && config.sanitize_html {
        html = ammonia::clean(&html);
        diagnostics.push(Diagnostic {
            step: "sanitization",
            level: DiagnosticLevel::Info,
            message: "HTML sanitized via ammonia".to_string(),
        });
    }

    // Step 3: Accessibility — add ARIA attributes
    if config.add_aria_attributes {
        match add_aria_attributes(&html, None) {
            Ok(enhanced) => {
                html = enhanced;
                diagnostics.push(Diagnostic {
                    step: "accessibility",
                    level: DiagnosticLevel::Info,
                    message: "ARIA attributes added".to_string(),
                });
            }
            Err(e) => {
                let d = Diagnostic {
                    step: "accessibility",
                    level: DiagnosticLevel::Error,
                    message: format!("ARIA enhancement skipped: {e}"),
                };
                warn!("{d}");
                diagnostics.push(d);
            }
        }
    }

    // Step 4: Table of contents — replace [[TOC]] placeholder
    if config.generate_toc {
        match generate_table_of_contents(&html) {
            Ok(toc) => {
                html = html.replace("[[TOC]]", &toc);
                diagnostics.push(Diagnostic {
                    step: "toc",
                    level: DiagnosticLevel::Info,
                    message: "Table of contents injected".to_string(),
                });
            }
            Err(e) => {
                let d = Diagnostic {
                    step: "toc",
                    level: DiagnosticLevel::Error,
                    message: format!(
                        "Table of contents generation failed: {e}"
                    ),
                };
                warn!("{d}");
                diagnostics.push(d);
            }
        }
    }

    // Step 5: Parse DOM once for read-only steps (SEO, heading extraction)
    let document = scraper::Html::parse_document(&html);

    // Step 5a: Structured data — generate JSON-LD
    let mut json_ld_fragment = String::new();
    if config.generate_structured_data {
        match generate_structured_data_from_doc(&document, None) {
            Ok(json_ld) => {
                json_ld_fragment = json_ld;
                diagnostics.push(Diagnostic {
                    step: "structured_data",
                    level: DiagnosticLevel::Info,
                    message: "JSON-LD structured data generated"
                        .to_string(),
                });
            }
            Err(e) => {
                let d = Diagnostic {
                    step: "structured_data",
                    level: DiagnosticLevel::Error,
                    message: format!(
                        "Structured data generation failed: {e}"
                    ),
                };
                warn!("{d}");
                diagnostics.push(d);
            }
        }
    }

    // Step 6: Full document wrapping or fragment language injection
    if config.generate_full_document {
        // Extract title from already-parsed DOM (no extra parse)
        let title = extract_first_heading_from_doc(&document);
        html = wrap_full_document(
            &html,
            &json_ld_fragment,
            title.as_deref(),
            config,
        );
    } else {
        // Fragment mode: append JSON-LD at the end (legacy behaviour)
        if !json_ld_fragment.is_empty() {
            html.push_str(&json_ld_fragment);
        }
        // Wrap in a lang div when the user set a non-default language
        if config.language != crate::constants::DEFAULT_LANGUAGE {
            html = format!(
                "<div lang=\"{}\">{}</div>",
                escape_html(&config.language),
                html
            );
        }
    }

    // Step 7: Minification
    if config.minify_output {
        let before_len = html.len();
        match minify_html_string(&html) {
            Ok(minified) => {
                let saved = before_len.saturating_sub(minified.len());
                html = minified;
                diagnostics.push(Diagnostic {
                    step: "minification",
                    level: DiagnosticLevel::Info,
                    message: format!(
                        "Minified: saved {} bytes ({:.0}%)",
                        saved,
                        if before_len > 0 {
                            saved as f64 / before_len as f64 * 100.0
                        } else {
                            0.0
                        }
                    ),
                });
            }
            Err(e) => {
                let d = Diagnostic {
                    step: "minification",
                    level: DiagnosticLevel::Error,
                    message: format!("Minification failed: {e}"),
                };
                warn!("{d}");
                diagnostics.push(d);
            }
        }
    }

    Ok(HtmlOutput { html, diagnostics })
}

/// Wraps HTML body content in a valid HTML5 document skeleton.
fn wrap_full_document(
    body: &str,
    json_ld: &str,
    title: Option<&str>,
    config: &crate::HtmlConfig,
) -> String {
    let lang = escape_html(&config.language);
    let mut head = String::from("<meta charset=\"utf-8\">");

    if let Some(t) = title {
        head.push_str(&format!("<title>{}</title>", escape_html(t)));
    }

    if !json_ld.is_empty() {
        head.push_str(json_ld);
    }

    format!(
        "<!DOCTYPE html>\n<html lang=\"{lang}\">\n<head>{head}</head>\n<body>\n{body}\n</body>\n</html>"
    )
}

/// Extracts text content from the first `<h1>` in a pre-parsed DOM.
fn extract_first_heading_from_doc(
    document: &scraper::Html,
) -> Option<String> {
    let selector = scraper::Selector::parse("h1").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>())
}

/// Convert Markdown to HTML with specified extensions using `mdx-gen`.
pub fn markdown_to_html_with_extensions(
    markdown: &str,
) -> Result<String> {
    markdown_to_html_impl(markdown, &crate::HtmlConfig::default())
}

fn markdown_to_html_impl(
    markdown: &str,
    config: &crate::HtmlConfig,
) -> Result<String> {
    // 1) Extract front matter
    let content_without_front_matter = extract_front_matter(markdown)
        .unwrap_or_else(|_| markdown.to_string());

    // 2) Convert triple-colon blocks, re-parsing inline Markdown inside them
    let markdown_with_classes = add_custom_classes(
        &content_without_front_matter,
        config.allow_unsafe_html,
    );

    // 3) Convert images with `.class="..."`
    let markdown_with_images =
        process_images_with_classes(&markdown_with_classes);

    // 4) Configure Comrak/Markdown Options
    let mut comrak_options = Options::default();
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.table = true;
    comrak_options.extension.autolink = true;
    comrak_options.extension.tasklist = true;
    comrak_options.extension.superscript = true;
    comrak_options.render.r#unsafe = config.allow_unsafe_html;

    // 5) Wire syntax highlighting settings into mdx-gen
    let mut md_options = MarkdownOptions::default()
        .with_comrak_options(comrak_options)
        .with_syntax_highlighting(config.enable_syntax_highlighting);

    if let Some(ref theme) = config.syntax_theme {
        md_options = md_options.with_custom_theme(theme.clone());
    }

    // 6) Convert final Markdown to HTML
    match process_markdown(&markdown_with_images, &md_options) {
        Ok(html_output) => Ok(html_output),
        Err(err) => {
            Err(HtmlError::markdown_conversion(err.to_string(), None))
        }
    }
}

/// Re-parse inline Markdown for triple-colon blocks, e.g.:
///
/// ```markdown
/// :::warning
/// **Caution:** This is risky.
/// :::
/// ```
///
/// Produces something like:
/// ```html
/// <div class="warning"><strong>Caution:</strong> This is risky.</div>
/// ```
///
/// # Example
/// ...
fn add_custom_classes(
    markdown: &str,
    allow_unsafe_html: bool,
) -> String {
    let Some(regex) = CUSTOM_CLASS_REGEX.as_ref() else {
        return markdown.to_string();
    };
    regex
        .replace_all(markdown, |caps: &regex::Captures| {
            let class_name = &caps[1];
            let block_content = &caps[2];

            let inline_html = match process_markdown_inline_impl(
                block_content,
                allow_unsafe_html,
            ) {
                Ok(html) => html,
                Err(_) => block_content.to_string(),
            };

            // class_name is validated by the \w+ regex — safe to interpolate
            format!(
                "<div class=\"{}\">{}</div>",
                class_name, inline_html
            )
        })
        .to_string()
}

/// Processes inline Markdown (bold, italics, links, etc.) without block-level syntax.
pub fn process_markdown_inline(
    content: &str,
) -> std::result::Result<String, Box<dyn Error>> {
    process_markdown_inline_impl(content, false)
}

fn process_markdown_inline_impl(
    content: &str,
    allow_unsafe_html: bool,
) -> std::result::Result<String, Box<dyn Error>> {
    let mut comrak_opts = Options::default();

    comrak_opts.extension.strikethrough = true;
    comrak_opts.extension.table = true;
    comrak_opts.extension.autolink = true;
    comrak_opts.extension.tasklist = true;
    comrak_opts.extension.superscript = true;

    comrak_opts.render.r#unsafe = allow_unsafe_html;

    let options =
        MarkdownOptions::default().with_comrak_options(comrak_opts);
    let inline_html = process_markdown(content, &options)?;
    Ok(inline_html)
}

/// Replaces image patterns like
/// `![Alt text](URL).class="some-class"` with `<img src="URL" alt="Alt text" class="some-class" />`.
fn process_images_with_classes(markdown: &str) -> String {
    let Some(regex) = IMAGE_CLASS_REGEX.as_ref() else {
        return markdown.to_string();
    };
    regex
        .replace_all(markdown, |caps: &regex::Captures| {
            format!(
                r#"<img src="{}" alt="{}" class="{}" />"#,
                escape_html(&caps[2]), // URL
                escape_html(&caps[1]), // alt text
                escape_html(&caps[3]), // class attribute
            )
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlConfig;

    /// Test basic Markdown to HTML conversion.
    ///
    /// This test verifies that a simple Markdown input is correctly converted to HTML.
    #[test]
    fn test_generate_html_basic() {
        let markdown = "# Hello, world!\n\nThis is a test.";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Hello, world!</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }

    /// Test conversion with Markdown extensions.
    ///
    /// This test ensures that the Markdown extensions (e.g., custom blocks, enhanced tables, etc.)
    /// are correctly applied when converting Markdown to HTML.
    #[test]
    fn test_markdown_to_html_with_extensions() {
        let markdown = r"
| Header 1 | Header 2 |
| -------- | -------- |
| Row 1    | Row 2    |
";
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        println!("{}", html);

        // Update the test to look for the div wrapper and table classes
        assert!(html.contains("<div class=\"table-responsive\"><table class=\"table\">"), "Table element not found");
        assert!(
            html.contains("<th>Header 1</th>"),
            "Table header not found"
        );
        assert!(
            html.contains("<td class=\"text-left\">Row 1</td>"),
            "Table row not found"
        );
    }

    /// Test conversion of empty Markdown.
    ///
    /// This test checks that an empty Markdown input results in an empty HTML string.
    #[test]
    fn test_generate_html_empty() {
        let markdown = "";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.is_empty());
    }

    /// Test handling of invalid Markdown.
    ///
    /// This test verifies that even with poorly formatted Markdown, the function
    /// will not panic and will return valid HTML.
    #[test]
    fn test_generate_html_invalid_markdown() {
        let markdown = "# Unclosed header\nSome **unclosed bold";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();

        println!("{}", html);

        assert!(
            html.contains("<h1>Unclosed header</h1>"),
            "Header not found"
        );
        assert!(
            html.contains("<p>Some **unclosed bold</p>"),
            "Unclosed bold tag not properly handled"
        );
    }

    /// Test conversion with complex Markdown content.
    ///
    /// This test checks how the function handles more complex Markdown input with various
    /// elements like lists, headers, code blocks, and links.
    /// Test conversion with complex Markdown content.
    #[test]
    fn test_generate_html_complex() {
        let markdown = r#"
# Header

## Subheader

Some `inline code` and a [link](https://example.com).

```rust
fn main() {
    println!("Hello, world!");
}
```

1. First item
2. Second item
"#;
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        println!("{}", html);

        // Verify the header and subheader
        assert!(
            html.contains("<h1>Header</h1>"),
            "H1 Header not found"
        );
        assert!(
            html.contains("<h2>Subheader</h2>"),
            "H2 Header not found"
        );

        // Verify the inline code and link
        assert!(
            html.contains("<code>inline code</code>"),
            "Inline code not found"
        );
        assert!(
            html.contains(r#"<a href="https://example.com">link</a>"#),
            "Link not found"
        );

        // Verify the code block structure
        assert!(
            html.contains(r#"<code class="language-rust">"#),
            "Code block with language-rust class not found"
        );
        assert!(
            html.contains(r#"<span style="color:#b48ead;">fn </span>"#),
            "`fn` keyword with syntax highlighting not found"
        );
        assert!(
            html.contains(
                r#"<span style="color:#8fa1b3;">main</span>"#
            ),
            "`main` function name with syntax highlighting not found"
        );

        // Check for the ordered list items
        assert!(
            html.contains("<li>First item</li>"),
            "First item not found"
        );
        assert!(
            html.contains("<li>Second item</li>"),
            "Second item not found"
        );
    }

    /// Test handling of valid front matter.
    #[test]
    fn test_generate_html_with_valid_front_matter() {
        let markdown = r#"---
title: Test
author: Jane Doe
---
# Hello, world!"#;
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Hello, world!</h1>"));
    }

    /// Test handling of invalid front matter.
    #[test]
    fn test_generate_html_with_invalid_front_matter() {
        let markdown = r#"---
title Test
author: Jane Doe
---
# Hello, world!"#;
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(
            result.is_ok(),
            "Invalid front matter should be ignored"
        );
        let html = result.unwrap();
        assert!(html.contains("<h1>Hello, world!</h1>"));
    }

    /// Test with a large Markdown input.
    #[test]
    fn test_generate_html_large_input() {
        let markdown = "# Large Markdown\n\n".repeat(10_000);
        let config = HtmlConfig::default();
        let result = generate_html(&markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Large Markdown</h1>"));
    }

    /// Test with different MarkdownOptions configurations.
    #[test]
    fn test_generate_html_with_custom_markdown_options() {
        let markdown = "**Bold text**";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<strong>Bold text</strong>"));
    }

    /// Test unsupported Markdown elements.
    #[test]
    fn test_generate_html_with_unsupported_elements() {
        let markdown = "::: custom_block\nContent\n:::";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("::: custom_block"));
    }

    /// Test error handling for invalid Markdown conversion.
    #[test]
    fn test_markdown_to_html_with_conversion_error() {
        let markdown = "# Unclosed header\nSome **unclosed bold";
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<p>Some **unclosed bold</p>"));
    }

    /// Test handling of whitespace-only Markdown.
    #[test]
    fn test_generate_html_whitespace_only() {
        let markdown = "   \n   ";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(
            html.is_empty(),
            "Whitespace-only Markdown should produce empty HTML"
        );
    }

    /// Test customization of Options.
    #[test]
    fn test_markdown_to_html_with_custom_comrak_options() {
        let markdown = "^^Superscript^^\n\n| Header 1 | Header 2 |\n| -------- | -------- |\n| Row 1    | Row 2    |";

        // Configure Options with necessary extensions
        let mut comrak_options = Options::default();
        comrak_options.extension.superscript = true;
        comrak_options.extension.table = true; // Enable table to match MarkdownOptions

        // Synchronize MarkdownOptions with Options
        let options = MarkdownOptions::default()
            .with_comrak_options(comrak_options.clone());
        let content_without_front_matter =
            extract_front_matter(markdown)
                .unwrap_or(markdown.to_string());

        println!("Comrak options: {:?}", comrak_options);

        let result =
            process_markdown(&content_without_front_matter, &options);

        match result {
            Ok(ref html) => {
                // Assert superscript rendering
                assert!(
                    html.contains("<sup>Superscript</sup>"),
                    "Superscript not found in HTML output"
                );

                // Assert table rendering
                assert!(
                    html.contains("<table"),
                    "Table element not found in HTML output"
                );
            }
            Err(err) => {
                panic!(
                    "Failed to process Markdown with custom Options: {:?}",
                    err
                );
            }
        }
    }
    #[test]
    fn test_generate_html_with_default_config() {
        let markdown = "# Default Configuration Test";
        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Default Configuration Test</h1>"));
    }

    #[test]
    fn test_generate_html_with_custom_front_matter_delimiter() {
        let markdown = r#";;;;
title: Custom
author: John Doe
;;;;
# Custom Front Matter Delimiter"#;

        let config = HtmlConfig::default();
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Custom Front Matter Delimiter</h1>"));
    }
    #[test]
    fn test_generate_html_with_task_list() {
        let markdown = r"
- [x] Task 1
- [ ] Task 2
";

        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        println!("Generated HTML:\n{}", html);

        // Adjust assertions to match the rendered HTML structure
        assert!(
        html.contains(r#"<li><input type="checkbox" checked="" disabled="" /> Task 1</li>"#),
        "Task 1 checkbox not rendered as expected"
    );
        assert!(
        html.contains(r#"<li><input type="checkbox" disabled="" /> Task 2</li>"#),
        "Task 2 checkbox not rendered as expected"
    );
    }
    #[test]
    fn test_generate_html_with_large_table() {
        let header =
            "| Header 1 | Header 2 |\n| -------- | -------- |\n";
        let rows = "| Row 1    | Row 2    |\n".repeat(1000);
        let markdown = format!("{}{}", header, rows);

        let result = markdown_to_html_with_extensions(&markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        let row_count = html.matches("<tr>").count();
        assert_eq!(
            row_count, 1001,
            "Incorrect number of rows: {}",
            row_count
        ); // 1 header + 1000 rows
    }
    #[test]
    fn test_generate_html_with_special_characters() {
        let markdown = r#"Markdown with special characters: <, >, &, "quote", 'single-quote'."#;
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        assert!(html.contains("&lt;"), "Less than sign not escaped");
        assert!(html.contains("&gt;"), "Greater than sign not escaped");
        assert!(html.contains("&amp;"), "Ampersand not escaped");
        assert!(html.contains("&quot;"), "Double quote not escaped");

        // Adjust if single quotes are intended to remain unescaped
        assert!(
            html.contains("&#39;") || html.contains("'"),
            "Single quote not handled as expected"
        );
    }

    #[test]
    fn test_generate_html_with_invalid_markdown_syntax() {
        // With unsafe_html disabled (default), raw HTML tags are stripped
        let markdown =
            r"# Invalid Markdown <unexpected> [bad](url <here)";
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        println!("Generated HTML:\n{}", html);

        // Raw HTML tags are stripped when unsafe=false
        assert!(html.contains("<h1>"), "Header tag should be present");
    }

    /// Test handling of Markdown with a mix of valid and invalid syntax.
    #[test]
    fn test_generate_html_mixed_markdown() {
        let markdown = r"# Valid Header
Some **bold text** followed by invalid Markdown:
~~strikethrough~~ without a closing tag.";
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        assert!(
            html.contains("<h1>Valid Header</h1>"),
            "Header not found"
        );
        assert!(
            html.contains("<strong>bold text</strong>"),
            "Bold text not rendered correctly"
        );
        assert!(
            html.contains("<del>strikethrough</del>"),
            "Strikethrough not rendered correctly"
        );
    }

    /// Test handling of deeply nested Markdown content.
    #[test]
    fn test_generate_html_deeply_nested_content() {
        let markdown = r"
1. Level 1
    1.1. Level 2
        1.1.1. Level 3
            1.1.1.1. Level 4
";
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        assert!(html.contains("<ol>"), "Ordered list not rendered");
        assert!(html.contains("<li>Level 1"), "Level 1 not rendered");
        assert!(
            html.contains("1.1.1.1. Level 4"),
            "Deeply nested levels not rendered correctly"
        );
    }

    /// Test Markdown with embedded raw HTML content (opt-in unsafe).
    #[test]
    fn test_generate_html_with_raw_html() {
        let markdown = r"
# Header with HTML
<p>This is a paragraph with <strong>HTML</strong>.</p>
";
        // Opt in to unsafe HTML for this test
        let config = HtmlConfig {
            allow_unsafe_html: true,
            ..HtmlConfig::default()
        };
        let result = generate_html(markdown, &config);
        assert!(result.is_ok());
        let html = result.unwrap();

        assert!(
            html.contains("<p>This is a paragraph with <strong>HTML</strong>.</p>"),
            "Raw HTML content not preserved in output"
        );
    }

    /// Test Markdown with invalid front matter format.
    #[test]
    fn test_generate_html_invalid_front_matter_handling() {
        let markdown = "---
key_without_value
another_key: valid
---
# Markdown Content
";
        let result = generate_html(markdown, &HtmlConfig::default());
        assert!(
            result.is_ok(),
            "Invalid front matter should not cause an error"
        );
        let html = result.unwrap();
        assert!(
            html.contains("<h1>Markdown Content</h1>"),
            "Content not processed correctly"
        );
    }

    /// Test handling of very large front matter in Markdown.
    #[test]
    fn test_generate_html_large_front_matter() {
        let front_matter = "---\n".to_owned()
            + &"key: value\n".repeat(10_000)
            + "---\n# Content";
        let result =
            generate_html(&front_matter, &HtmlConfig::default());
        assert!(
            result.is_ok(),
            "Large front matter should be handled gracefully"
        );
        let html = result.unwrap();
        assert!(
            html.contains("<h1>Content</h1>"),
            "Content not rendered correctly"
        );
    }

    /// Test handling of Markdown with long consecutive lines.
    #[test]
    fn test_generate_html_with_long_lines() {
        let markdown = "A ".repeat(10_000);
        let result = markdown_to_html_with_extensions(&markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        assert!(
            html.contains("A A A A"),
            "Long consecutive lines should be rendered properly"
        );
    }

    #[test]
    fn test_markdown_with_custom_classes() {
        let markdown = r":::note
This is a note with a custom class.
:::";

        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok(), "Markdown conversion should not fail.");

        let html = result.unwrap();
        println!("HTML:\n{}", html);

        // Ensure we see <div class="note"> in the final output:
        assert!(
            html.contains(r#"<div class="note">"#),
            "Custom block should wrap in <div class=\"note\">"
        );

        // Ensure the block content is present:
        assert!(
            html.contains("This is a note with a custom class."),
            "Block text is missing or incorrectly rendered"
        );
    }

    #[test]
    fn test_markdown_with_custom_blocks_and_images() {
        let markdown = "![A very tall building](https://example.com/image.webp).class=\"img-fluid\"";
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();
        println!("{}", html);
        assert!(
        html.contains(r#"<img src="https://example.com/image.webp" alt="A very tall building" class="img-fluid" />"#),
        "First image not rendered correctly"
    );
    }

    /// Test empty front matter handling.
    #[test]
    fn test_empty_front_matter_handling() {
        let markdown = "---\n---\n# Content";
        let result = generate_html(markdown, &HtmlConfig::default());
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(
            html.contains("<h1>Content</h1>"),
            "Content should be processed correctly"
        );
    }

    /// Test invalid image syntax.
    #[test]
    fn test_invalid_image_syntax() {
        let markdown = "![Image with missing URL]()";
        let result = process_images_with_classes(markdown);
        assert_eq!(
            result, markdown,
            "Invalid image syntax should remain unchanged"
        );
    }

    /// Test incorrect front matter delimiters.
    #[test]
    fn test_incorrect_front_matter_delimiters() {
        let markdown = ";;;\ntitle: Test\n---\n# Header";
        let result = generate_html(markdown, &HtmlConfig::default());
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(
            html.contains("<h1>Header</h1>"),
            "Header should be processed correctly"
        );
    }
    #[cfg(test)]
    mod missing_scenarios_tests {
        use super::*;

        /// 1) Triple-colon block with inline bold text
        ///
        /// Verifies that **Caution:** inside `:::warning` is parsed as `<strong>Caution:</strong>`.
        #[test]
        fn test_triple_colon_warning_with_bold() {
            let markdown = r":::warning
**Caution:** This operation is sensitive.
:::";

            let result = markdown_to_html_with_extensions(markdown);
            assert!(
                result.is_ok(),
                "Markdown conversion should succeed."
            );

            let html = result.unwrap();
            println!("HTML:\n{}", html);

            // Expect the block to contain <strong>Caution:</strong>
            // plus a <div class="warning">
            assert!(
                html.contains(r#"<div class="warning">"#),
                "Expected <div class=\"warning\"> wrapping the block"
            );
            assert!(html.contains("<strong>Caution:</strong>"),
            "Expected inline bold text to become <strong>Caution:</strong>");
        }

        /// 2) Multiple triple-colon blocks in the same snippet.
        ///
        /// Ensures that the parser correctly handles more than one custom block.
        #[test]
        fn test_multiple_triple_colon_blocks() {
            let markdown = r":::note
**Note:** First block
:::

:::warning
**Warning:** Second block
:::";

            let result = markdown_to_html_with_extensions(markdown);
            assert!(
                result.is_ok(),
                "Markdown conversion should succeed."
            );

            let html = result.unwrap();
            println!("HTML:\n{}", html);

            // Expect <div class="note"> ...</div> and <div class="warning"> ...</div>
            assert!(
                html.contains(r#"<div class="note">"#),
                "Missing <div class=\"note\"> for the first block"
            );
            assert!(
                html.contains(r#"<div class="warning">"#),
                "Missing <div class=\"warning\"> for the second block"
            );

            // Check inline markdown
            assert!(
                html.contains("<strong>Note:</strong>"),
                "Bold text in the note block not parsed"
            );
            assert!(
                html.contains("<strong>Warning:</strong>"),
                "Bold text in the warning block not parsed"
            );
        }

        /// 3) Triple-colon block with multi-paragraph content
        ///
        /// Checks how inline parsing deals with extra blank lines and multiple paragraphs.
        #[test]
        fn test_triple_colon_block_multi_paragraph() {
            let markdown = r":::note
**Paragraph 1:** This is the first paragraph.

This is the second paragraph, also with **bold** text.
:::";

            let result = markdown_to_html_with_extensions(markdown);
            assert!(
                result.is_ok(),
                "Markdown conversion should succeed."
            );

            let html = result.unwrap();
            println!("HTML:\n{}", html);

            // The block is inline-processed. Paragraphs might be combined or
            // each appear in separate <p> tags, depending on the parser.
            // Typically, inline parsing doesn't break paragraphs. If you want block-level
            // formatting, you'd need a full block parse. But let's at least confirm bold text.
            assert!(
                html.contains("<strong>Paragraph 1:</strong>"),
                "Inline bold text not parsed in the first paragraph"
            );
            assert!(html.contains("second paragraph, also with <strong>bold</strong> text"),
            "Inline bold text not parsed in the second paragraph");
        }

        /// 4) Fallback logic: forcing an error in `process_markdown_inline`
        ///
        /// We'll create a scenario that intentionally breaks the inline parser.
        /// If an error occurs, we expect the raw text (with triple-colon block content).
        #[test]
        fn test_triple_colon_block_forcing_inline_error() {
            // Suppose the inline parser fails when we pass some nonsense markup or unhandled structure.
            // It's not always guaranteed to fail, but let's try an improbable snippet:
            let markdown = r":::error
This block tries < to break > inline parsing & [some link (unclosed).
:::";

            // We'll artificially modify the parser to fail if it sees "[some link (unclosed)."
            // But since your code doesn't do that by default, we can't *guarantee* a real error.
            // We'll at least check that, if an error *did* occur, we fallback to raw text.
            //
            // For demonstration, let's proceed with the test and see if it just parses or not.
            let result = markdown_to_html_with_extensions(markdown);
            assert!(
                result.is_ok(),
                "We won't forcibly error, but let's see the output."
            );

            let html = result.unwrap();
            println!("HTML:\n{}", html);

            // If your parser did handle it, we'll just check the block.
            // If your parser chokes, you'd see a fallback with raw text.
            // Let's verify there's a <div class="error"> either way:
            assert!(
                html.contains(r#"<div class="error">"#),
                "Block div not found for 'error' class"
            );

            // If the inline parser didn't fail, we might see <p> with weird text.
            // If it fails, we should see the original snippet inside the block.
            // We'll just check that it's not empty.
            assert!(
                html.contains("This block tries "),
                "Expected parsed content in the block"
            );
        }
    }
}
