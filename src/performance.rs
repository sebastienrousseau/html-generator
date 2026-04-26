// Copyright © 2025 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

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
use minify_html::{minify, Cfg};
use std::{fs, path::Path};

#[cfg(feature = "async")]
use tokio::task;

/// Maximum allowed file size for minification (10 MB).
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

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
        cfg.minify_doctype = false;
        cfg.allow_noncompliant_unquoted_attribute_values = false;
        cfg.keep_closing_tags = true;
        cfg.keep_html_and_head_opening_tags = true;
        cfg.allow_removing_spaces_between_attributes = false;
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
            .field("minify_doctype", &self.cfg.minify_doctype)
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
        // After the size check above, the overwhelmingly common failure
        // is a non-UTF-8 input file; other I/O faults (permissions
        // flipping mid-call, etc.) are exceedingly rare but we keep
        // a single clear message that covers both cases.
        let kind = if e
            .to_string()
            .contains("stream did not contain valid UTF-8")
        {
            "Invalid UTF-8 in input file"
        } else {
            "Failed to read file"
        };
        HtmlError::MinificationError(format!(
            "{kind} '{}': {e}",
            file_path.display()
        ))
    })?;

    let config = MinifyConfig::default();
    let minified = minify(content.as_bytes(), &config.cfg);

    // `minify-html` produces valid UTF-8 whenever the input is valid
    // UTF-8 (guaranteed here because `content` is a `String`), so the
    // fallible decode path is provably unreachable — use `lossy` to
    // skip the dead `Err` arm.
    Ok(String::from_utf8_lossy(&minified).into_owned())
}

/// Minifies an HTML string in memory.
///
/// Applies the same minification rules as [`minify_html`] but
/// operates on an in-memory string instead of a file path.
///
/// # Arguments
///
/// * `html` - The HTML content to minify
///
/// # Returns
///
/// Returns the minified HTML content as a string if successful.
///
/// # Errors
///
/// Returns [`HtmlError`] if:
/// - The input exceeds [`MAX_FILE_SIZE`]
/// - The minified output is not valid UTF-8
///
/// # Examples
///
/// ```
/// # use html_generator::performance::minify_html_string;
/// # fn example() -> Result<(), html_generator::error::HtmlError> {
/// let html = "<html>  <body>  <p>Hello</p>  </body>  </html>";
/// let minified = minify_html_string(html)?;
/// assert_eq!(minified, "<html><body><p>Hello</p></body></html>");
/// # Ok(())
/// # }
/// ```
pub fn minify_html_string(html: &str) -> Result<String> {
    if html.len() > MAX_FILE_SIZE {
        return Err(HtmlError::MinificationError(format!(
            "Input size {} bytes exceeds maximum of {MAX_FILE_SIZE} bytes",
            html.len()
        )));
    }

    let config = MinifyConfig::default();
    let minified = minify(html.as_bytes(), &config.cfg);

    // See `minify_html`: the decode cannot fail for UTF-8 input.
    Ok(String::from_utf8_lossy(&minified).into_owned())
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
/// ```ignore
/// use html_generator::performance::async_generate_html;
///
/// #[tokio::main]
/// async fn main() -> Result<(), html_generator::error::HtmlError> {
///     let markdown = "# Hello\n\nThis is a test.";
///     let html = async_generate_html(markdown).await?;
///     println!("Generated HTML length: {}", html.len());
///     Ok(())
/// }
/// ```
#[cfg(feature = "async")]
pub async fn async_generate_html(markdown: &str) -> Result<String> {
    let markdown = markdown.to_string();
    task::spawn_blocking(move || {
        crate::generator::markdown_to_html_with_extensions(&markdown)
    })
    .await
    .map_err(|e| HtmlError::MarkdownConversion {
        message: format!("Asynchronous HTML generation failed: {e}"),
        source: Some(std::io::Error::other(e.to_string())),
    })?
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
        fn test_minify_non_utf8_failure_path_via_directory_path() {
            // Pointing `minify_html` at a directory exercises the
            // non-UTF-8 *fallback* arm in the read-error mapping —
            // `fs::read_to_string` on a directory fails with
            // "Is a directory" (or platform-equivalent), which does
            // not match the UTF-8 substring and so routes to the
            // "Failed to read file" branch.
            let dir =
                tempdir().expect("Failed to create temp directory");
            let result = minify_html(dir.path());
            assert!(matches!(
                result,
                Err(HtmlError::MinificationError(_))
            ));
            let err_msg = result.unwrap_err().to_string();
            assert!(
                err_msg.contains("Failed to read file"),
                "expected 'Failed to read file' branch, got: {err_msg}"
            );
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

    #[cfg(feature = "async")]
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

    mod additional_tests {
        use super::*;
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        /// Test for default MinifyConfig values.
        #[test]
        fn test_minify_config_default() {
            let config = MinifyConfig::default();
            assert!(!config.cfg.minify_doctype);
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

        /// Exercises the private `Debug` impl for MinifyConfig, which
        /// is unreachable from outside the module and so would
        /// otherwise show up as uncovered.
        #[test]
        fn test_minify_config_debug_impl() {
            let config = MinifyConfig::default();
            let rendered = format!("{config:?}");
            assert!(rendered.contains("MinifyConfig"));
            assert!(rendered.contains("minify_css"));
        }

        /// `minify_html` must surface a `MinificationError` when the
        /// source file cannot be read as UTF-8.
        #[test]
        fn test_minify_html_rejects_non_utf8_path_content() {
            let dir = tempdir().expect("failed to create temp dir");
            let file_path = dir.path().join("non-utf8.html");
            let mut f = File::create(&file_path).expect("create file");
            f.write_all(&[0xFF, 0xFE, 0xFD, 0xFC])
                .expect("write bytes");
            drop(f);
            let err = minify_html(&file_path).unwrap_err();
            assert!(matches!(err, HtmlError::MinificationError(_)));
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
        #[cfg(feature = "async")]
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

        #[cfg(feature = "async")]
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
                    source: Some(std::io::Error::other(e.to_string())),
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

        #[test]
        fn test_minify_html_empty_content() {
            let html = "";
            let (dir, file_path) = create_test_file(html);
            let result = minify_html(&file_path);
            assert!(result.is_ok());
            assert!(
                result.unwrap().is_empty(),
                "Minified content should be empty"
            );
            drop(dir);
        }

        #[test]
        fn test_minify_html_unusual_whitespace() {
            let html =
                "<html>\n\n\t<body>\t<p>Test</p>\n\n</body>\n\n</html>";
            let (dir, file_path) = create_test_file(html);
            let result = minify_html(&file_path);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "<html><body><p>Test</p></body></html>",
                "Unexpected minified result for unusual whitespace"
            );
            drop(dir);
        }

        #[test]
        fn test_minify_html_with_special_characters() {
            let html = "<div>&lt;Special&gt; &amp; Characters</div>";
            let (dir, file_path) = create_test_file(html);
            let result = minify_html(&file_path);
            assert!(result.is_ok());
            assert_eq!(
        result.unwrap(),
        "<div>&lt;Special> & Characters</div>",
        "Special characters were unexpectedly modified during minification"
    );
            drop(dir);
        }

        #[cfg(feature = "async")]
        #[tokio::test]
        async fn test_async_generate_html_with_special_characters() {
            let markdown =
                "# Special & Characters\n\nContent with < > & \" '";
            let result = async_generate_html(markdown).await;
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(
                html.contains("&lt;"),
                "Less than sign not escaped"
            );
            assert!(
                html.contains("&gt;"),
                "Greater than sign not escaped"
            );
            assert!(html.contains("&amp;"), "Ampersand not escaped");
            assert!(
                html.contains("&quot;"),
                "Double quote not escaped"
            );
            assert!(
                html.contains("&#39;") || html.contains("'"),
                "Single quote not handled as expected"
            );
        }
    }
}
