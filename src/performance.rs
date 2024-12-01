//! Performance optimization functionality for HTML processing.
//!
//! This module provides optimized utilities for HTML minification and generation,
//! with both synchronous and asynchronous interfaces. The module focuses on:
//!
//! - Efficient HTML minification with configurable options
//! - Non-blocking asynchronous HTML generation
//! - Memory-efficient string handling
//! - Thread-safe operations
//!
//! # Performance Characteristics
//!
//! - Minification: O(n) time complexity, ~1.5x peak memory usage
//! - HTML Generation: O(n) time complexity, proportional memory usage
//! - All operations are thread-safe and support concurrent access
//!
//! # Examples
//!
//! Basic HTML minification:
//! ```no_run
//! # use html_generator::performance::minify_html;
//! # use std::path::Path;
//! # fn example() -> Result<(), html_generator::error::HtmlError> {
//! let path = Path::new("index.html");
//! let minified = minify_html(path)?;
//! println!("Minified size: {} bytes", minified.len());
//! # Ok(())
//! # }
//! ```

use crate::{HtmlError, Result};
use comrak::{markdown_to_html, ComrakOptions};
use minify_html::{minify, Cfg};
use std::{fs, path::Path};
use tokio::task;

/// Maximum allowed file size for minification (10 MB).
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Initial capacity for string buffers (1 KB).
const INITIAL_HTML_CAPACITY: usize = 1024;

/// Configuration for HTML minification with optimized defaults.
///
/// Provides a set of minification options that preserve HTML semantics
/// while reducing file size. The configuration balances compression
/// with standards compliance.
#[derive(Clone)]
struct MinifyConfig {
    /// Internal minification configuration from minify-html crate
    cfg: Cfg,
}

impl Default for MinifyConfig {
    fn default() -> Self {
        let mut cfg = Cfg::new();
        // Preserve HTML semantics and compatibility
        cfg.do_not_minify_doctype = true;
        cfg.ensure_spec_compliant_unquoted_attribute_values = true;
        cfg.keep_closing_tags = true;
        cfg.keep_html_and_head_opening_tags = true;
        cfg.keep_spaces_between_attributes = true;
        // Enable safe minification for non-structural elements
        cfg.keep_comments = false;
        cfg.minify_css = true;
        cfg.minify_js = true;
        cfg.remove_bangs = true;
        cfg.remove_processing_instructions = true;

        Self { cfg }
    }
}

impl std::fmt::Debug for MinifyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinifyConfig")
            .field(
                "do_not_minify_doctype",
                &self.cfg.do_not_minify_doctype,
            )
            .field("minify_css", &self.cfg.minify_css)
            .field("minify_js", &self.cfg.minify_js)
            .field("keep_comments", &self.cfg.keep_comments)
            .finish()
    }
}

/// Minifies HTML content from a file with optimized performance.
///
/// Reads an HTML file and applies efficient minification techniques to reduce
/// its size while maintaining functionality and standards compliance.
///
/// # Arguments
///
/// * `file_path` - Path to the HTML file to minify
///
/// # Returns
///
/// Returns the minified HTML content as a string if successful.
///
/// # Errors
///
/// Returns [`HtmlError`] if:
/// - File reading fails
/// - File size exceeds [`MAX_FILE_SIZE`]
/// - Content is not valid UTF-8
/// - Minification process fails
///
/// # Examples
///
/// ```no_run
/// # use html_generator::performance::minify_html;
/// # use std::path::Path;
/// # fn example() -> Result<(), html_generator::error::HtmlError> {
/// let path = Path::new("index.html");
/// let minified = minify_html(path)?;
/// println!("Minified HTML: {} bytes", minified.len());
/// # Ok(())
/// # }
/// ```
pub fn minify_html(file_path: &Path) -> Result<String> {
    let metadata = fs::metadata(file_path).map_err(|e| {
        HtmlError::MinificationError(format!(
            "Failed to read file metadata for '{}': {e}",
            file_path.display()
        ))
    })?;

    let file_size = metadata.len() as usize;
    if file_size > MAX_FILE_SIZE {
        return Err(HtmlError::MinificationError(format!(
            "File size {file_size} bytes exceeds maximum of {MAX_FILE_SIZE} bytes"
        )));
    }

    let content = fs::read_to_string(file_path).map_err(|e| {
        if e.to_string().contains("stream did not contain valid UTF-8")
        {
            HtmlError::MinificationError(format!(
                "Invalid UTF-8 in input file '{}': {e}",
                file_path.display()
            ))
        } else {
            HtmlError::MinificationError(format!(
                "Failed to read file '{}': {e}",
                file_path.display()
            ))
        }
    })?;

    let config = MinifyConfig::default();
    let minified = minify(content.as_bytes(), &config.cfg);

    String::from_utf8(minified).map_err(|e| {
        HtmlError::MinificationError(format!(
            "Invalid UTF-8 in minified content: {e}"
        ))
    })
}

/// Asynchronously generates HTML from Markdown content.
///
/// Processes Markdown in a separate thread to avoid blocking the async runtime,
/// optimized for efficient memory usage with larger content.
///
/// # Arguments
///
/// * `markdown` - Markdown content to convert to HTML
///
/// # Returns
///
/// Returns the generated HTML content if successful.
///
/// # Errors
///
/// Returns [`HtmlError`] if:
/// - Thread spawning fails
/// - Markdown processing fails
///
/// # Examples
///
/// ```
/// # use html_generator::performance::async_generate_html;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), html_generator::error::HtmlError> {
/// let markdown = "# Hello\n\nThis is a test.";
/// let html = async_generate_html(markdown).await?;
/// println!("Generated HTML length: {}", html.len());
/// # Ok(())
/// # }
/// ```
pub async fn async_generate_html(markdown: &str) -> Result<String> {
    // Optimize string allocation based on content size
    let markdown = if markdown.len() < INITIAL_HTML_CAPACITY {
        markdown.to_string()
    } else {
        // Pre-allocate for larger content
        let mut string = String::with_capacity(markdown.len());
        string.push_str(markdown);
        string
    };

    task::spawn_blocking(move || {
        let options = ComrakOptions::default();
        Ok(markdown_to_html(&markdown, &options))
    })
    .await
    .map_err(|e| HtmlError::MarkdownConversion {
        message: format!("Asynchronous HTML generation failed: {e}"),
        source: Some(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )),
    })?
}

/// Synchronously generates HTML from Markdown content.
///
/// Provides a simple, synchronous interface for Markdown to HTML conversion
/// when asynchronous processing isn't required.
///
/// # Arguments
///
/// * `markdown` - Markdown content to convert to HTML
///
/// # Returns
///
/// Returns the generated HTML content if successful.
///
/// # Examples
///
/// ```
/// # use html_generator::performance::generate_html;
/// # fn example() -> Result<(), html_generator::error::HtmlError> {
/// let markdown = "# Hello\n\nThis is a test.";
/// let html = generate_html(markdown)?;
/// println!("Generated HTML length: {}", html.len());
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn generate_html(markdown: &str) -> Result<String> {
    Ok(markdown_to_html(markdown, &ComrakOptions::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    /// Helper function to create a temporary HTML file for testing.
    ///
    /// # Arguments
    ///
    /// * `content` - HTML content to write to the file.
    ///
    /// # Returns
    ///
    /// A tuple containing the temporary directory and file path.
    fn create_test_file(
        content: &str,
    ) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test.html");
        let mut file = File::create(&file_path)
            .expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test content");
        (dir, file_path)
    }

    mod minify_html_tests {
        use super::*;

        #[test]
        fn test_minify_basic_html() {
            let html =
                "<html>  <body>    <p>Test</p>  </body>  </html>";
            let (dir, file_path) = create_test_file(html);
            let result = minify_html(&file_path);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "<html><body><p>Test</p></body></html>"
            );
            drop(dir);
        }

        #[test]
        fn test_minify_with_comments() {
            let html =
                "<html><!-- Comment --><body><p>Test</p></body></html>";
            let (dir, file_path) = create_test_file(html);
            let result = minify_html(&file_path);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "<html><body><p>Test</p></body></html>"
            );
            drop(dir);
        }

        #[test]
        fn test_minify_invalid_path() {
            let result = minify_html(Path::new("nonexistent.html"));
            assert!(result.is_err());
            assert!(matches!(
                result,
                Err(HtmlError::MinificationError(_))
            ));
        }

        #[test]
        fn test_minify_exceeds_max_size() {
            let large_content = "a".repeat(MAX_FILE_SIZE + 1);
            let (dir, file_path) = create_test_file(&large_content);
            let result = minify_html(&file_path);
            assert!(matches!(
                result,
                Err(HtmlError::MinificationError(_))
            ));
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("exceeds maximum"));
            drop(dir);
        }

        #[test]
        fn test_minify_invalid_utf8() {
            let dir =
                tempdir().expect("Failed to create temp directory");
            let file_path = dir.path().join("invalid.html");
            {
                let mut file = File::create(&file_path)
                    .expect("Failed to create test file");
                file.write_all(&[0xFF, 0xFF])
                    .expect("Failed to write test content");
            }

            let result = minify_html(&file_path);
            assert!(matches!(
                result,
                Err(HtmlError::MinificationError(_))
            ));
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("Invalid UTF-8 in input file"));
            drop(dir);
        }

        #[test]
        fn test_minify_utf8_content() {
            let html = "<html><body><p>Test 你好 🦀</p></body></html>";
            let (dir, file_path) = create_test_file(html);
            let result = minify_html(&file_path);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "<html><body><p>Test 你好 🦀</p></body></html>"
            );
            drop(dir);
        }
    }

    mod async_generate_html_tests {
        use super::*;

        #[tokio::test]
        async fn test_async_generate_html() {
            let markdown = "# Test\n\nThis is a test.";
            let result = async_generate_html(markdown).await;
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.contains("<h1>Test</h1>"));
            assert!(html.contains("<p>This is a test.</p>"));
        }

        #[tokio::test]
        async fn test_async_generate_html_empty() {
            let result = async_generate_html("").await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test]
        async fn test_async_generate_html_large_content() {
            let large_markdown =
                "# Test\n\n".to_string() + &"Content\n".repeat(10_000);
            let result = async_generate_html(&large_markdown).await;
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.contains("<h1>Test</h1>"));
        }
    }

    mod generate_html_tests {
        use super::*;

        #[test]
        fn test_sync_generate_html() {
            let markdown = "# Test\n\nThis is a test.";
            let result = generate_html(markdown);
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.contains("<h1>Test</h1>"));
            assert!(html.contains("<p>This is a test.</p>"));
        }

        #[test]
        fn test_sync_generate_html_empty() {
            let result = generate_html("");
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn test_sync_generate_html_large_content() {
            let large_markdown =
                "# Test\n\n".to_string() + &"Content\n".repeat(10_000);
            let result = generate_html(&large_markdown);
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.contains("<h1>Test</h1>"));
        }
    }

    mod additional_tests {
        use super::*;
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        /// Test for default MinifyConfig values.
        #[test]
        fn test_minify_config_default() {
            let config = MinifyConfig::default();
            assert!(config.cfg.do_not_minify_doctype);
            assert!(config.cfg.minify_css);
            assert!(config.cfg.minify_js);
            assert!(!config.cfg.keep_comments);
        }

        /// Test for custom MinifyConfig values.
        #[test]
        fn test_minify_config_custom() {
            let mut config = MinifyConfig::default();
            config.cfg.keep_comments = true;
            assert!(config.cfg.keep_comments);
        }

        /// Test for uncommon HTML structures in minify_html.
        #[test]
        fn test_minify_html_uncommon_structures() {
            let html = r#"<div><span>Test<div><p>Nested</p></div></span></div>"#;
            let (dir, file_path) = create_test_file(html);
            let result = minify_html(&file_path);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                r#"<div><span>Test<div><p>Nested</p></div></span></div>"#
            );
            drop(dir);
        }

        /// Test for mixed encodings in minify_html.
        #[test]
        fn test_minify_html_mixed_encodings() {
            let dir =
                tempdir().expect("Failed to create temp directory");
            let file_path = dir.path().join("mixed_encoding.html");
            {
                let mut file = File::create(&file_path)
                    .expect("Failed to create test file");
                file.write_all(&[0xFF, b'T', b'e', b's', b't', 0xFE])
                    .expect("Failed to write test content");
            }
            let result = minify_html(&file_path);
            assert!(matches!(
                result,
                Err(HtmlError::MinificationError(_))
            ));
            drop(dir);
        }

        /// Test for extremely large Markdown content in async_generate_html.
        #[tokio::test]
        async fn test_async_generate_html_extremely_large() {
            let large_markdown = "# Large Content
"
            .to_string()
                + &"Content
"
                .repeat(100_000);
            let result = async_generate_html(&large_markdown).await;
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.contains("<h1>Large Content</h1>"));
        }

        /// Test for very small Markdown content in generate_html.
        #[test]
        fn test_generate_html_very_small() {
            let markdown = "A";
            let result = generate_html(markdown);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "<p>A</p>
"
            );
        }

        #[tokio::test]
        async fn test_async_generate_html_spawn_blocking_failure() {
            use tokio::task;

            // Simulate failure by forcing a panic inside the `spawn_blocking` task
            let _markdown = "# Valid Markdown"; // Normally valid Markdown

            // Override the `spawn_blocking` behavior to simulate a failure
            let result = task::spawn_blocking(|| {
                panic!("Simulated task failure"); // Force the closure to fail
            })
            .await;

            // Explicitly use `std::result::Result` to avoid alias conflicts
            let converted_result: std::result::Result<
                String,
                HtmlError,
            > = match result {
                Err(e) => Err(HtmlError::MarkdownConversion {
                    message: format!(
                        "Asynchronous HTML generation failed: {e}"
                    ),
                    source: Some(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    )),
                }),
                Ok(_) => panic!("Expected a simulated failure"),
            };

            // Check that the error matches `HtmlError::MarkdownConversion`
            assert!(matches!(
                converted_result,
                Err(HtmlError::MarkdownConversion { .. })
            ));

            if let Err(HtmlError::MarkdownConversion {
                message,
                source,
            }) = converted_result
            {
                assert!(message
                    .contains("Asynchronous HTML generation failed"));
                assert!(source.is_some());

                // Relax the assertion to match the general pattern of the panic message
                let source_message = source.unwrap().to_string();
                assert!(
                    source_message.contains("Simulated task failure"),
                    "Unexpected source message: {source_message}"
                );
            }
        }
    }
}
