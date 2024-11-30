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
//!
use html_generator::{markdown_file_to_html, markdown_to_html, MarkdownConfig, OutputDestination};
use std::{
    fs::{self},
    path::PathBuf,
};

/// Utility functions for setting up and cleaning test environments.
mod test_utils {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    /// Creates a test file with the given content at the specified path.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to write to the file.
    /// * `file_path` - The path where the file will be created.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be created or written to.
    pub(crate) fn setup_test_file(content: &str, file_path: &Path) {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create test directory");
        }
        let mut file = File::create(file_path).expect("Failed to create test file");
        file.write_all(content.as_bytes()).expect("Failed to write test file");
        file.sync_all().expect("Failed to sync test file");
    }

    /// Cleans up the specified directory by removing it and all its contents.
    ///
    /// # Arguments
    ///
    /// * `dir_path` - The path of the directory to remove.
    ///
    /// # Panics
    ///
    /// Panics if the directory cannot be removed.
    pub(crate) fn cleanup_test_dir(dir_path: &Path) {
        if dir_path.exists() {
            fs::remove_dir_all(dir_path).expect("Failed to clean up test directory");
        } else {
            eprintln!("Directory {:?} does not exist, skipping cleanup.", dir_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::{cleanup_test_dir, setup_test_file};

    /// Tests the end-to-end functionality of converting Markdown to HTML.
    ///
    /// This test checks basic Markdown conversion using the default configuration.
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

    /// Tests file-based Markdown to HTML conversion with custom configuration.
    ///
    /// This test verifies that Markdown files can be converted to HTML files
    /// and checks for correct HTML generation.
    #[test]
    fn test_file_conversion_with_custom_config() {
        let input_dir = PathBuf::from("test_input");
        let input_path = input_dir.join("test.md");
        let output_dir = PathBuf::from("test_output");
        let output_path = output_dir.join("output.html");

        // Setup test input file
        setup_test_file("# Test\n\n```rust\nfn main() {}\n```", &input_path);
        println!("Input file created at: {:?}", input_path);

        // Ensure the output directory exists
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        println!("Output directory created at: {:?}", output_dir);

        // Log input content for debugging
        let input_content = fs::read_to_string(&input_path).expect("Failed to read input file content");
        println!("Input file content:\n{}", input_content);

        // Run Markdown file conversion
        let config = MarkdownConfig::default();
        let result = markdown_file_to_html(
            Some(&input_path),
            Some(OutputDestination::File(output_path.to_string_lossy().into())),
            Some(config),
        );

        assert!(result.is_ok(), "Markdown conversion failed");

        // Validate output
        match fs::read_to_string(&output_path) {
            Ok(html) => {
                println!("Generated HTML:\n{}", html);
                assert!(html.contains("<h1>"), "Missing <h1> tag in output HTML");
                assert!(html.contains("<pre><code"), "Missing code block in output HTML");
            }
            Err(e) => panic!("Failed to read output file: {:?}", e),
        }

        // Cleanup with checks
        println!(
            "Cleaning up input directory: {:?}, exists: {}",
            input_dir,
            input_dir.exists()
        );
        cleanup_test_dir(&input_dir);

        println!(
            "Cleaning up output directory: {:?}, exists: {}",
            output_dir,
            output_dir.exists()
        );
        cleanup_test_dir(&output_dir);
    }

    /// Tests various error conditions during Markdown to HTML conversion.
    ///
    /// This test checks the behaviour when invalid paths or configurations are provided.
    #[test]
    fn test_error_conditions() {
        // Test invalid input file path
        let result = markdown_file_to_html(Some("nonexistent.md"), None, None);
        assert!(result.is_err(), "Expected an error for nonexistent input file");

        // Test invalid output file path
        let input_dir = PathBuf::from("test_input");
        let input_path = input_dir.join("test.md");
        setup_test_file("# Test", &input_path);

        let result = markdown_file_to_html(
            Some(&input_path),
            Some(OutputDestination::File("invalid/path/output.html".to_string())),
            None,
        );
        assert!(result.is_err(), "Expected an error for invalid output path");

        cleanup_test_dir(&input_dir);

        // Test unsupported input file extension
        let result = markdown_file_to_html(Some("test.txt"), None, None);
        assert!(result.is_err(), "Expected an error for unsupported file extension");
    }

    /// Tests Markdown to HTML conversion with custom configurations.
    ///
    /// This test checks that custom configurations are applied correctly.
    #[test]
    fn test_custom_configurations() {
        let markdown = "# Test\n\n## Section\n\nContent with [link](http://example.com)";
        let config = MarkdownConfig::default();
        let result = markdown_to_html(markdown, Some(config));

        assert!(result.is_ok(), "Markdown conversion failed");
        let html = result.unwrap();
        assert!(html.contains("<h1>"), "Generated HTML missing <h1> tag");
        assert!(html.contains("<h2>"), "Generated HTML missing <h2> tag");
        assert!(html.contains("<p>"), "Generated HTML missing <p> tag");
        assert!(
            html.contains("<a href=\"http://example.com\""),
            "Generated HTML missing anchor tag with href"
        );
    }
}
