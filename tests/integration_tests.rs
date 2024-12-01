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

    /// Helper function to create unique directories for each test.
    fn create_test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(format!("test_env_{}", name));
        if dir.exists() {
            cleanup_test_dir(&dir);
        }
        fs::create_dir_all(&dir)
            .expect("Failed to create test directory");
        dir
    }

    #[test]
    fn test_markdown_to_html_with_code_block(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let markdown = "# Title\n\n```rust\nfn main() {}\n```";
        let config = MarkdownConfig {
            html_config: html_generator::HtmlConfig {
                enable_syntax_highlighting: true,
                ..Default::default()
            },
            ..MarkdownConfig::default()
        };

        let result = markdown_to_html(markdown, Some(config))?;
        assert!(
            result.contains("<pre><code class=\"language-rust\">"),
            "Missing syntax-highlighted code block in output HTML"
        );
        assert!(
            result.contains("<span"),
            "Expected syntax-highlighted span elements in output HTML"
        );
        Ok(())
    }

    #[test]
    fn test_end_to_end_markdown_to_html(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let markdown = "# Test Heading\n\nTest paragraph.";
        let config = MarkdownConfig::default();
        let result = markdown_to_html(markdown, Some(config))?;

        assert!(
            result.contains("<h1>Test Heading</h1>"),
            "Generated HTML missing <h1> tag"
        );
        assert!(
            result.contains("<p>Test paragraph.</p>"),
            "Generated HTML missing <p> tag"
        );
        Ok(())
    }

    #[test]
    fn test_file_conversion_with_custom_config(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let input_dir = create_test_dir("file_conversion_input");
        let output_dir = create_test_dir("file_conversion_output");
        let input_path = input_dir.join("test.md");
        let output_path = output_dir.join("output.html");

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

        markdown_file_to_html(
            Some(&input_path),
            Some(OutputDestination::File(
                output_path.to_string_lossy().into(),
            )),
            Some(config),
        )?;

        let html = fs::read_to_string(&output_path)?;
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
        Ok(())
    }

    #[test]
    fn test_error_conditions() -> Result<(), Box<dyn std::error::Error>>
    {
        let nonexistent_file = Path::new("nonexistent.md");
        assert!(
            nonexistent_file.canonicalize().is_err(),
            "Expected an error for nonexistent file"
        );

        let input_dir = create_test_dir("error_conditions_input");
        let input_path = input_dir.join("test.md");
        let _ = setup_test_file(Some("# Test"), &input_path);

        let invalid_output_path =
            PathBuf::from("invalid/path/output.html");
        assert!(
            markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::File(
                    invalid_output_path.to_string_lossy().into()
                )),
                None
            )
            .is_err(),
            "Expected an error for invalid output path"
        );

        cleanup_test_dir(&input_dir);
        Ok(())
    }

    #[test]
    fn test_custom_configurations(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let markdown = "# Test\n\n## Section\n\nContent with [link](http://example.com)";
        let config = MarkdownConfig {
            html_config: html_generator::HtmlConfig {
                enable_syntax_highlighting: true,
                ..Default::default()
            },
            ..MarkdownConfig::default()
        };

        let result = markdown_to_html(markdown, Some(config))?;
        assert!(
            result.contains("<h1>"),
            "Generated HTML missing <h1> tag"
        );
        assert!(
            result.contains("<h2>"),
            "Generated HTML missing <h2> tag"
        );
        assert!(
            result.contains("<a href=\"http://example.com\""),
            "Generated HTML missing anchor tag with href"
        );
        Ok(())
    }
}
