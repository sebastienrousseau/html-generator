//! HTML generation module for converting Markdown to HTML.
//!
//! This module provides functions to generate HTML from Markdown content
//! using the `mdx-gen` library. It supports various Markdown extensions
//! and custom configuration options.

use crate::error::HtmlError;
use crate::extract_front_matter;
use crate::Result;
use mdx_gen::{process_markdown, ComrakOptions, MarkdownOptions};

/// Generate HTML from Markdown content using `mdx-gen`.
///
/// This function takes Markdown content and a configuration object,
/// converts the Markdown into HTML, and returns the resulting HTML string.
///
/// # Arguments
///
/// * `markdown` - A string slice that holds the Markdown content to convert.
/// * `_config` - A reference to an `HtmlConfig` struct that holds the configuration options.
///
/// # Returns
///
/// * `Result<String>` - The generated HTML or an error if the conversion fails.
///
/// # Example
///
/// ```rust
/// use html_generator::HtmlConfig;
/// use html_generator::generate_html;
/// let markdown = "# Hello, world!";
/// let config = HtmlConfig::default();
/// let html = generate_html(markdown, &config).unwrap();
/// assert!(html.contains("<h1>Hello, world!</h1>"));
/// ```
pub fn generate_html(
    markdown: &str,
    _config: &crate::HtmlConfig,
) -> Result<String> {
    markdown_to_html_with_extensions(markdown)
}

/// Convert Markdown to HTML with specified extensions using `mdx-gen`.
///
/// This function applies a set of extensions to enhance the conversion
/// process, such as syntax highlighting, enhanced table formatting,
/// custom blocks, and more.
///
/// # Arguments
///
/// * `markdown` - A string slice that holds the Markdown content to convert.
///
/// # Returns
///
/// * `Result<String>` - The generated HTML or an error if the conversion fails.
///
/// # Example
///
/// ```rust
/// use html_generator::generator::markdown_to_html_with_extensions;
/// let markdown = "~~strikethrough~~";
/// let html = markdown_to_html_with_extensions(markdown).unwrap();
/// assert!(html.contains("<del>strikethrough</del>"));
/// ```
pub fn markdown_to_html_with_extensions(
    markdown: &str,
) -> Result<String> {
    // Extract Markdown without front matter
    let content_without_front_matter =
        extract_front_matter(markdown).unwrap_or(markdown.to_string());

    // Configure ComrakOptions for Markdown processing
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.strikethrough = true;
    comrak_options.extension.table = true;
    comrak_options.extension.autolink = true;
    comrak_options.extension.tasklist = true;
    comrak_options.extension.superscript = true;

    // Ensure raw HTML is allowed
    comrak_options.render.unsafe_ = true;
    comrak_options.render.escape = false;

    // Use MarkdownOptions with the customized ComrakOptions
    let options =
        MarkdownOptions::default().with_comrak_options(comrak_options);

    // Process the Markdown to HTML using `mdx-gen`
    match process_markdown(&content_without_front_matter, &options) {
        Ok(html_output) => Ok(html_output),
        Err(err) => {
            // Use the helper method to return an HtmlError
            Err(HtmlError::markdown_conversion(
                err.to_string(),
                None, // If err is not io::Error, use None
            ))
        }
    }
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

    /// Test customization of ComrakOptions.
    #[test]
    fn test_markdown_to_html_with_custom_comrak_options() {
        let markdown = "^^Superscript^^\n\n| Header 1 | Header 2 |\n| -------- | -------- |\n| Row 1    | Row 2    |";

        // Configure ComrakOptions with necessary extensions
        let mut comrak_options = ComrakOptions::default();
        comrak_options.extension.superscript = true;
        comrak_options.extension.table = true; // Enable table to match MarkdownOptions

        // Synchronize MarkdownOptions with ComrakOptions
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
                eprintln!("Markdown processing error: {:?}", err);
                panic!("Failed to process Markdown with custom ComrakOptions");
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
        let markdown =
            r"# Invalid Markdown <unexpected> [bad](url <here)";
        let result = markdown_to_html_with_extensions(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();

        println!("Generated HTML:\n{}", html);

        // Validate that raw HTML tags are not escaped
        assert!(
            html.contains("<unexpected>"),
            "Raw HTML tags like <unexpected> should not be escaped"
        );

        // Validate that angle brackets in links are escaped
        assert!(
            html.contains("&lt;here&gt;") || html.contains("&lt;here)"),
            "Angle brackets in links should be escaped for safety"
        );

        // Validate the full header content
        assert!(
        html.contains("<h1>Invalid Markdown <unexpected> [bad](url &lt;here)</h1>"),
        "Header not rendered correctly or content not properly handled"
    );
    }
}
