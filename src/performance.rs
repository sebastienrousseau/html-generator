//! Performance-related functionality for HTML processing.
//!
//! This module provides functions for minifying HTML and generating HTML from Markdown, with a focus on performance and efficiency.

use crate::{HtmlError, Result};
use comrak::{markdown_to_html, ComrakOptions};
use minify_html::{minify, Cfg};
use std::{fs, path::Path};
use tokio::task;

/// Returns a default `Cfg` for HTML minification.
///
/// This helper function creates a default configuration for minifying HTML
/// with pre-set options for CSS, JS, and attributes.
///
/// # Returns
/// A `Cfg` object containing the default minification settings.
fn default_minify_cfg() -> Cfg {
    let mut cfg = Cfg::new();
    cfg.do_not_minify_doctype = true;
    cfg.ensure_spec_compliant_unquoted_attribute_values = true;
    cfg.keep_closing_tags = true;
    cfg.keep_html_and_head_opening_tags = true;
    cfg.keep_spaces_between_attributes = true;
    cfg.keep_comments = false;
    cfg.minify_css = true;
    cfg.minify_js = true;
    cfg.remove_bangs = true;
    cfg.remove_processing_instructions = true;
    cfg
}

/// Minifies a single HTML file.
///
/// This function takes a reference to a `Path` object for an HTML file and
/// returns a string containing the minified HTML.
///
/// # Arguments
///
/// * `file_path` - A reference to a `Path` object for the HTML file.
///
/// # Returns
///
/// * `Result<String, HtmlError>` - A result containing a string
///    containing the minified HTML.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use html_generator::performance::minify_html;
///
/// let path = Path::new("index.html");
/// match minify_html(path) {
///     Ok(minified) => println!("Minified HTML: {}", minified),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn minify_html(file_path: &Path) -> Result<String> {
    // Read the file content
    let content = fs::read_to_string(file_path).map_err(|e| {
        HtmlError::MinificationError(format!(
            "Failed to read file: {}",
            e
        ))
    })?;

    // Minify the content
    let minified_content =
        minify(content.as_bytes(), &default_minify_cfg());

    // Convert the minified content back to a UTF-8 string
    String::from_utf8(minified_content).map_err(|e| {
        HtmlError::MinificationError(format!(
            "Invalid UTF-8 in minified content: {}",
            e
        ))
    })
}

/// Asynchronously generate HTML from Markdown.
///
/// This function converts a Markdown string into an HTML string using
/// Comrak, a CommonMark-compliant Markdown parser and renderer.
/// The conversion is performed in a separate thread to avoid blocking.
///
/// # Arguments
///
/// * `markdown` - A reference to a Markdown string.
///
/// # Returns
///
/// * `Result<String, HtmlError>` - A result containing a string with the
///   generated HTML.
///
/// # Examples
///
/// ```
/// use html_generator::performance::async_generate_html;
///
/// #[tokio::main]
/// async fn main() {
///     let markdown = "# Hello\n\nThis is a test.";
///     match async_generate_html(markdown).await {
///         Ok(html) => println!("Generated HTML: {}", html),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub async fn async_generate_html(markdown: &str) -> Result<String> {
    let markdown = markdown.to_string();
    task::spawn_blocking(move || {
        let options = ComrakOptions::default();
        Ok(markdown_to_html(&markdown, &options))
    })
    .await
    .map_err(|e| HtmlError::MarkdownConversionError(e.to_string()))?
}

/// Synchronously generate HTML from Markdown.
///
/// This function converts a Markdown string into an HTML string using
/// Comrak, a CommonMark-compliant Markdown parser and renderer.
///
/// # Arguments
///
/// * `markdown` - A reference to a Markdown string.
///
/// # Returns
///
/// * `Result<String, HtmlError>` - A result containing a string with the
///   generated HTML.
///
/// # Examples
///
/// ```
/// use html_generator::performance::generate_html;
///
/// let markdown = "# Hello\n\nThis is a test.";
/// match generate_html(markdown) {
///     Ok(html) => println!("Generated HTML: {}", html),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn generate_html(markdown: &str) -> Result<String> {
    let options = ComrakOptions::default();
    Ok(markdown_to_html(markdown, &options))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    /// Helper function to create an HTML file for testing.
    fn create_html_file(file_path: &Path, content: &str) {
        let mut file = File::create(file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_minify_html_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.html");
        let html = "<html>  <body>    <p>Test</p>  </body>  </html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "<html><body><p>Test</p></body></html>"
        );
    }

    #[test]
    fn test_minify_html_with_comments() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_comments.html");
        let html = "<html>  <body>    <!-- This is a comment -->    <p>Test</p>  </body>  </html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "<html><body><p>Test</p></body></html>"
        );
    }

    #[test]
    fn test_minify_html_with_css() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_css.html");
        let html = "<html><head><style>  body  {  color:  red;  }  </style></head><body><p>Test</p></body></html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<html><head><style>body{color:red}</style></head><body><p>Test</p></body></html>");
    }

    #[test]
    fn test_minify_html_with_js() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_js.html");
        let html = "<html><head><script>  function  test()  {  console.log('Hello');  }  </script></head><body><p>Test</p></body></html>";

        create_html_file(&file_path, html);

        let result = minify_html(&file_path);
        assert!(result.is_ok());
        let minified = result.unwrap();
        assert!(minified.contains("<script>"));
        assert!(minified.contains("console.log"));
        assert!(minified.contains("Hello"));
        assert!(minified.contains("<p>Test</p>"));
    }

    #[test]
    fn test_minify_html_non_existent_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent.html");

        let result = minify_html(&file_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MinificationError(_)
        ));
    }

    #[test]
    fn test_minify_html_invalid_utf8() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid_utf8.html");
        let invalid_utf8 = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence

        let mut file = File::create(&file_path).unwrap();
        file.write_all(&invalid_utf8).unwrap();

        let result = minify_html(&file_path);

        assert!(
            result.is_err(),
            "Expected an error due to invalid UTF-8 sequence"
        );
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MinificationError(_)
        ));
    }

    #[test]
    fn test_generate_html_basic() {
        let markdown = "# Test\n\nThis is a test.";
        let result = generate_html(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Test</h1>"));
        assert!(html.contains("<p>This is a test.</p>"));
    }

    #[test]
    fn test_generate_html_complex() {
        let markdown = "# Header\n\n## Subheader\n\n- List item 1\n- List item 2\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let result = generate_html(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<h1>Header</h1>"));
        assert!(html.contains("<h2>Subheader</h2>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>List item 1</li>"));
        assert!(html.contains("<li>List item 2</li>"));
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main()"));
        assert!(html.contains("println!"));
        assert!(html.contains("Hello, world!"));
    }

    #[test]
    fn test_generate_html_empty_input() {
        let markdown = "";
        let result = generate_html(markdown);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.trim().is_empty());
    }
}
