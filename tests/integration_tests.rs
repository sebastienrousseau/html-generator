//! # Markdown to HTML Conversion Tests
//!
//! This module contains integration tests for converting Markdown content and files into HTML
//! using the `html_generator` library. These tests ensure correctness, validate configurations,
//! and check edge cases for error handling, contributing to the library's overall stability.
//!
//! ## Overview
//!
//! The tests cover the following scenarios:
//!
//! - **End-to-End Conversion**: Ensures basic Markdown content is converted to valid HTML.
//! - **File-Based Conversion**: Validates conversion from Markdown files to HTML files with
//!   configurable options.
//! - **Error Conditions**: Tests the behaviour when invalid inputs or configurations are provided.
//! - **Custom Configurations**: Checks the application of custom settings like syntax highlighting
//!   and table of contents generation.
//!
//! ## Organization
//!
//! - Utility functions for test setup and cleanup are defined in the `test_utils` module.
//! - Tests are grouped into individual functions, each covering a specific scenario.
//! - Each test is isolated, with proper directory creation and cleanup to prevent interference.
//!
//! ## Usage
//!
//! To run the tests, use the following command:
//!
//! ```bash
//! cargo test --test integration_tests
//! ```
//!
//! Ensure that the `html_generator` library is correctly configured and that all dependencies are installed before running the tests.

use html_generator::{
    markdown_file_to_html, markdown_to_html, MarkdownConfig,
    OutputDestination,
};
use std::{
    fs::{self},
    path::PathBuf,
};

/// Utility functions for setting up and cleaning test environments.
mod test_utils {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    /// Creates a test file with the given content at the specified path.
    pub(crate) fn setup_test_file(
        content: Option<&str>,
        file_path: &Path,
    ) -> Option<PathBuf> {
        if let Some(content) = content {
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .expect("Failed to create test directory");
            }
            let mut file = File::create(file_path)
                .expect("Failed to create test file");
            file.write_all(content.as_bytes())
                .expect("Failed to write test file");
            file.sync_all().expect("Failed to sync test file");

            let abs_path = file_path
                .canonicalize()
                .expect("Failed to canonicalize test file path");
            assert!(
                abs_path.exists(),
                "Test file does not exist after creation"
            );
            Some(abs_path)
        } else {
            None
        }
    }

    /// Cleans up the specified directory by removing it and all its contents.
    pub(crate) fn cleanup_test_dir(dir_path: &Path) {
        if dir_path.exists() {
            fs::remove_dir_all(dir_path)
                .expect("Failed to clean up test directory");
        } else {
            eprintln!(
                "Directory {:?} does not exist, skipping cleanup.",
                dir_path
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use test_utils::{cleanup_test_dir, setup_test_file};

    #[test]
    fn test_markdown_to_html_with_code_block() {
        let markdown = "# Title\n\n```rust\nfn main() {}\n```";
        let config = MarkdownConfig {
            html_config: html_generator::HtmlConfig {
                enable_syntax_highlighting: true,
                ..Default::default()
            },
            ..MarkdownConfig::default()
        };

        let result = markdown_to_html(markdown, Some(config));
        assert!(result.is_ok(), "Markdown conversion failed");

        let html = result.unwrap();
        assert!(
            html.contains("<pre><code class=\"language-rust\">"),
            "Missing syntax-highlighted code block in output HTML"
        );
        assert!(
            html.contains("<span"),
            "Expected syntax-highlighted span elements in output HTML"
        );
    }

    #[test]
    fn test_end_to_end_markdown_to_html() {
        let markdown = "# Test Heading\n\nTest paragraph.";
        let config = MarkdownConfig::default();
        let result = markdown_to_html(markdown, Some(config));

        assert!(result.is_ok(), "Markdown conversion failed");
        let html = result.unwrap();
        assert!(
            html.contains("<h1>Test Heading</h1>"),
            "Generated HTML missing <h1> tag"
        );
        assert!(
            html.contains("<p>Test paragraph.</p>"),
            "Generated HTML missing <p> tag"
        );
    }

    #[test]
    fn test_file_conversion_with_custom_config() {
        let input_dir = PathBuf::from("test_input");
        let input_path = input_dir.join("test.md");
        let output_dir = PathBuf::from("test_output");
        let output_path = output_dir.join("output.html");

        if input_dir.exists() {
            fs::remove_dir_all(&input_dir)
                .expect("Failed to remove existing input directory");
        }
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir)
                .expect("Failed to remove existing output directory");
        }
        fs::create_dir_all(&input_dir)
            .expect("Failed to create input directory");
        fs::create_dir_all(&output_dir)
            .expect("Failed to create output directory");

        let _ = setup_test_file(
            Some("# Test\n\n```rust\nfn main() {}\n```"),
            &input_path,
        );

        let config = MarkdownConfig {
            html_config: html_generator::HtmlConfig {
                enable_syntax_highlighting: true,
                ..Default::default()
            },
            ..MarkdownConfig::default()
        };

        let result = markdown_file_to_html(
            Some(&input_path),
            Some(OutputDestination::File(
                output_path.to_string_lossy().into(),
            )),
            Some(config),
        );

        assert!(result.is_ok(), "Markdown conversion failed");

        let html = fs::read_to_string(&output_path)
            .expect("Failed to read output file");
        assert!(
            html.contains("<h1>"),
            "Missing <h1> tag in output HTML"
        );
        assert!(
            html.contains("<pre><code class=\"language-rust\">"),
            "Missing syntax-highlighted code block in output HTML"
        );

        cleanup_test_dir(&input_dir);
        cleanup_test_dir(&output_dir);
    }

    #[test]
    fn test_error_conditions() {
        let nonexistent_file = Path::new("nonexistent.md");

        let result = nonexistent_file.canonicalize();
        assert!(
            result.is_err(),
            "Expected an error for nonexistent file, but got: {:?}",
            result
        );

        let input_dir = PathBuf::from("test_input");
        let input_path = input_dir.join("test.md");
        let _ = setup_test_file(Some("# Test"), &input_path);

        let invalid_output_path =
            PathBuf::from("invalid/path/output.html");
        let result = markdown_file_to_html(
            Some(&input_path),
            Some(OutputDestination::File(
                invalid_output_path.to_string_lossy().into(),
            )),
            None,
        );
        assert!(
            result.is_err(),
            "Expected an error for invalid output path"
        );

        cleanup_test_dir(&input_dir);
    }

    #[test]
    fn test_custom_configurations() {
        let markdown = "# Test\n\n## Section\n\nContent with [link](http://example.com)";
        let config = MarkdownConfig {
            html_config: html_generator::HtmlConfig {
                enable_syntax_highlighting: true,
                ..Default::default()
            },
            ..MarkdownConfig::default()
        };

        let result = markdown_to_html(markdown, Some(config));
        assert!(result.is_ok(), "Markdown conversion failed");

        let html = result.unwrap();
        assert!(
            html.contains("<h1>"),
            "Generated HTML missing <h1> tag"
        );
        assert!(
            html.contains("<h2>"),
            "Generated HTML missing <h2> tag"
        );
        assert!(
            html.contains("<a href=\"http://example.com\""),
            "Generated HTML missing anchor tag with href"
        );
    }
}
