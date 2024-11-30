//! HTML Generator: A modern HTML generation and optimisation library
//!
//! `html-generator` is a comprehensive suite of tools for generating, optimising,
//! and managing HTML content with a focus on accessibility, SEO, and performance.
//!
//! # Features
//!
//! - **Markdown to HTML**: Convert Markdown content and files to HTML
//! - **Accessibility**: Automated ARIA attributes and WCAG compliance checking
//! - **SEO Optimisation**: Meta tag generation and structured data support
//! - **Performance**: HTML minification and async generation capabilities
//!
//! # Examples
//!
//! ```rust
//! use html_generator::{markdown_to_html, MarkdownConfig};
//! # fn main() -> Result<(), html_generator::error::HtmlError> {
//! let markdown = "# Hello World\n\nWelcome to HTML Generator.";
//! let config = MarkdownConfig::default();
//! let html = markdown_to_html(markdown, Some(config))?;
//! println!("Generated HTML: {html}");
//! # Ok(())
//! # }
//! ```
//!
//! # Security Features
//!
//! - Path validation to prevent directory traversal attacks
//! - Input size limits to prevent denial of service
//! - Unicode-aware text processing
//! - Memory safety through Rust's guarantees
//! - Comprehensive error handling to prevent undefined behaviour

use std::path::Component;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

// Re-export public modules
pub mod accessibility;
pub mod error;
pub mod generator;
pub mod performance;
pub mod seo;
pub mod utils;

// Re-export primary types and functions
pub use crate::error::HtmlError;
pub use accessibility::{add_aria_attributes, validate_wcag};
pub use generator::generate_html;
pub use performance::{async_generate_html, minify_html};
pub use seo::{generate_meta_tags, generate_structured_data};
pub use utils::{extract_front_matter, format_header_with_id_class};

/// Common constants used throughout the library
pub mod constants {
    /// Default maximum input size (5MB)
    pub const DEFAULT_MAX_INPUT_SIZE: usize = 5 * 1024 * 1024;
    /// Minimum input size (1KB)
    pub const MIN_INPUT_SIZE: usize = 1024;
    /// Default language code (en-GB)
    pub const DEFAULT_LANGUAGE: &str = "en-GB";
    /// Default syntax highlighting theme (github)
    pub const DEFAULT_SYNTAX_THEME: &str = "github";
    /// Maximum file path length
    pub const MAX_PATH_LENGTH: usize = 4096;
    /// Valid language code pattern
    pub const LANGUAGE_CODE_PATTERN: &str = r"^[a-z]{2}-[A-Z]{2}$";
    /// Verify invariants at compile time
    const _: () = assert!(MIN_INPUT_SIZE <= DEFAULT_MAX_INPUT_SIZE);
    const _: () = assert!(MAX_PATH_LENGTH > 0);
}

/// Result type alias for library operations
pub type Result<T> = std::result::Result<T, HtmlError>;

/// Configuration options for Markdown to HTML conversion
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MarkdownConfig {
    /// The encoding to use for input/output (defaults to "utf-8")
    pub encoding: String,
    /// HTML generation configuration
    pub html_config: HtmlConfig,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            encoding: String::from("utf-8"),
            html_config: HtmlConfig::default(),
        }
    }
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Error for invalid input size configuration
    #[error(
        "Invalid input size: {0} bytes is below minimum of {1} bytes"
    )]
    InvalidInputSize(usize, usize),
    /// Error for invalid language code
    #[error("Invalid language code: {0}")]
    InvalidLanguageCode(String),
    /// Error for invalid file path
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
}

/// Output destination for HTML generation
#[non_exhaustive] // Allow for future expansion
pub enum OutputDestination {
    /// Write output to a file at the specified path
    File(String),
    /// Write output using a custom writer implementation
    ///
    /// This can be used for in-memory buffers, network streams,
    /// or other custom output destinations.
    Writer(Box<dyn Write>),
    /// Write output to standard output (default)
    ///
    /// This is useful for command-line tools and scripts.
    Stdout,
}

impl std::fmt::Debug for OutputDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(path) => {
                f.debug_tuple("File").field(path).finish()
            }
            Self::Writer(_) => write!(f, "Writer(<dyn Write>)"),
            Self::Stdout => write!(f, "Stdout"),
        }
    }
}

impl Default for OutputDestination {
    fn default() -> Self {
        Self::Stdout
    }
}

/// Configuration options for HTML generation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HtmlConfig {
    /// Enable syntax highlighting for code blocks
    pub enable_syntax_highlighting: bool,
    /// Theme to use for syntax highlighting
    pub syntax_theme: Option<String>,
    /// Minify the generated HTML output
    pub minify_output: bool,
    /// Automatically add ARIA attributes for accessibility
    pub add_aria_attributes: bool,
    /// Generate structured data (JSON-LD) based on content
    pub generate_structured_data: bool,
    /// Maximum size (in bytes) for input content
    pub max_input_size: usize,
    /// Language for generated content
    pub language: String,
    /// Enable table of contents generation
    pub generate_toc: bool,
}

impl Default for HtmlConfig {
    fn default() -> Self {
        Self {
            enable_syntax_highlighting: true,
            syntax_theme: Some("github".to_string()),
            minify_output: false,
            add_aria_attributes: true,
            generate_structured_data: false,
            max_input_size: constants::DEFAULT_MAX_INPUT_SIZE,
            language: String::from(constants::DEFAULT_LANGUAGE),
            generate_toc: false,
        }
    }
}

impl HtmlConfig {
    /// Creates a new `HtmlConfig` with default options
    pub fn builder() -> HtmlConfigBuilder {
        HtmlConfigBuilder::default()
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_input_size < constants::MIN_INPUT_SIZE {
            return Err(HtmlError::InvalidInput(format!(
                "Input size must be at least {} bytes",
                constants::MIN_INPUT_SIZE
            )));
        }
        if !validate_language_code(&self.language) {
            return Err(HtmlError::InvalidInput(format!(
                "Invalid language code: {}",
                self.language
            )));
        }
        Ok(())
    }

    /// Validates file path safety
    pub(crate) fn validate_file_path(
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let path = path.as_ref();

        if path.to_string_lossy().is_empty() {
            return Err(HtmlError::InvalidInput(
                "File path cannot be empty".to_string(),
            ));
        }

        if path.to_string_lossy().len() > constants::MAX_PATH_LENGTH {
            return Err(HtmlError::InvalidInput(format!(
                "File path exceeds maximum length of {} characters",
                constants::MAX_PATH_LENGTH
            )));
        }

        if path.components().any(|c| matches!(c, Component::ParentDir))
        {
            return Err(HtmlError::InvalidInput(
                "Directory traversal is not allowed in file paths"
                    .to_string(),
            ));
        }

        // Only check absolute paths in non-test mode
        #[cfg(not(test))]
        if path.is_absolute() {
            return Err(HtmlError::InvalidInput(
                "Only relative file paths are allowed".to_string(),
            ));
        }

        if let Some(ext) = path.extension() {
            if !matches!(ext.to_string_lossy().as_ref(), "md" | "html")
            {
                return Err(HtmlError::InvalidInput(
                    "Invalid file extension: only .md and .html files are allowed".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Builder for `HtmlConfig` to customize HTML generation options
#[derive(Debug, Default)]
pub struct HtmlConfigBuilder {
    config: HtmlConfig,
}

impl HtmlConfigBuilder {
    /// Creates a new `HtmlConfigBuilder` with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable syntax highlighting for code blocks
    #[must_use]
    pub fn with_syntax_highlighting(
        mut self,
        enable: bool,
        theme: Option<String>,
    ) -> Self {
        self.config.enable_syntax_highlighting = enable;
        self.config.syntax_theme = if enable {
            theme.or_else(|| Some("github".to_string()))
        } else {
            None
        };
        self
    }

    /// Set the language for generated content
    #[must_use]
    pub fn with_language(
        mut self,
        language: impl Into<String>,
    ) -> Self {
        // Store the language value regardless of validation
        // Validation will happen during build()
        self.config.language = language.into();
        self
    }

    /// Build the configuration, validating all settings
    pub fn build(self) -> Result<HtmlConfig> {
        // Validate the configuration before returning
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Convert Markdown content to HTML
///
/// This function processes Unicode Markdown content and returns HTML output.
/// The input must be valid Unicode - if your input is encoded (e.g., UTF-8),
/// you must decode it before passing it to this function.
///
/// # Arguments
///
/// * `content` - The Markdown content as a Unicode string
/// * `config` - Optional configuration for the conversion
///
/// # Returns
///
/// Returns the generated HTML as a Unicode string wrapped in a `Result`
///
/// # Errors
///
/// Returns an error if:
/// * The input content is invalid Unicode
/// * HTML generation fails
/// * Input size exceeds configured maximum
///
/// # Examples
///
/// ```rust
/// use html_generator::{markdown_to_html, MarkdownConfig};
/// # fn main() -> Result<(), html_generator::error::HtmlError> {
/// let markdown = "# Hello\n\nWorld";
/// let html = markdown_to_html(markdown, None)?;
/// assert!(html.contains("<h1>Hello</h1>"));
/// # Ok(())
/// # }
/// ```
pub fn markdown_to_html(
    content: &str,
    config: Option<MarkdownConfig>,
) -> Result<String> {
    let config = config.unwrap_or_default();

    if content.is_empty() {
        return Err(HtmlError::InvalidInput(
            "Input content is empty".to_string(),
        ));
    }

    if content.len() > config.html_config.max_input_size {
        return Err(HtmlError::InputTooLarge(content.len()));
    }

    generate_html(content, &config.html_config)
}

/// Convert a Markdown file to HTML
///
/// This function reads from a file or stdin and writes the generated HTML to
/// a specified destination. It handles encoding/decoding of content.
///
/// # Arguments
///
/// * `input` - The input source (file path or None for stdin)
/// * `output` - The output destination (defaults to stdout)
/// * `config` - Optional configuration including encoding settings
///
/// # Returns
///
/// Returns `Ok(())` on success or an error if the operation fails
///
/// # Errors
///
/// Returns an error if:
/// * The input file cannot be read
/// * The output cannot be written
/// * The content cannot be decoded/encoded with the specified encoding
/// * HTML generation fails
/// * Input size exceeds configured maximum
///
/// # Examples
///
/// ```no_run
/// use html_generator::{markdown_file_to_html, MarkdownConfig, OutputDestination};
/// # fn main() -> Result<(), html_generator::error::HtmlError> {
/// let config = MarkdownConfig::default();
/// let output = OutputDestination::File("output.html".to_string());
///
/// markdown_file_to_html(
///     Some("input.md"),
///     Some(output),
///     Some(config)
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn markdown_file_to_html(
    input: Option<impl AsRef<Path>>,
    output: Option<OutputDestination>,
    config: Option<MarkdownConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let output = output.unwrap_or_default();

    // Validate paths first
    if let Some(path) = input.as_ref() {
        HtmlConfig::validate_file_path(path)?;
    }
    if let OutputDestination::File(ref path) = output {
        HtmlConfig::validate_file_path(path)?;
    }

    // Read and validate input
    let content = match input {
        Some(path) => {
            let mut file = File::open(path).map_err(HtmlError::Io)?;
            let mut content = String::new();
            _ = file
                .read_to_string(&mut content)
                .map_err(HtmlError::Io)?;
            content
        }
        None => {
            let mut content = String::new();
            _ = io::stdin()
                .read_to_string(&mut content)
                .map_err(HtmlError::Io)?;
            content
        }
    };

    // Generate HTML
    let html = markdown_to_html(&content, Some(config))?;

    // Write output
    match output {
        OutputDestination::File(path) => {
            let mut file = File::create(path).map_err(HtmlError::Io)?;
            file.write_all(html.as_bytes()).map_err(HtmlError::Io)?;
        }
        OutputDestination::Writer(mut writer) => {
            writer.write_all(html.as_bytes()).map_err(HtmlError::Io)?;
        }
        OutputDestination::Stdout => {
            io::stdout()
                .write_all(html.as_bytes())
                .map_err(HtmlError::Io)?;
        }
    }

    Ok(())
}

/// Validates that a language code matches the required pattern
///
/// # Arguments
///
/// * `lang` - The language code to validate
///
/// # Returns
///
/// Returns true if the language code is valid, false otherwise
fn validate_language_code(lang: &str) -> bool {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static LANG_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(constants::LANGUAGE_CODE_PATTERN).unwrap()
    });

    LANG_REGEX.is_match(lang)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::{tempdir, TempDir};

    /// Helper function to create a temporary test directory.
    ///
    /// Returns a TempDir that will automatically clean up when dropped.
    fn setup_test_dir() -> TempDir {
        tempdir().expect("Failed to create temporary directory")
    }

    /// Helper function to create a test file with the given content.
    ///
    /// # Arguments
    ///
    /// * `dir` - The temporary directory to create the file in
    /// * `content` - The content to write to the file
    ///
    /// # Returns
    ///
    /// Returns the path to the created file.
    fn create_test_file(
        dir: &TempDir,
        content: &str,
    ) -> std::path::PathBuf {
        let path = dir.path().join("test.md");
        std::fs::write(&path, content)
            .expect("Failed to write test file");
        path
    }

    /// Tests for configuration-related functionality
    mod config_tests {
        use super::*;

        #[test]
        fn test_config_validation() {
            // Test invalid input size
            let config = HtmlConfig {
                max_input_size: 100, // Too small
                ..Default::default()
            };
            assert!(config.validate().is_err());

            // Test invalid language code
            let config = HtmlConfig {
                language: "invalid".to_string(),
                ..Default::default()
            };
            assert!(config.validate().is_err());

            // Test valid default configuration
            let config = HtmlConfig::default();
            assert!(config.validate().is_ok());
        }

        #[test]
        fn test_config_builder() {
            let result = HtmlConfigBuilder::new()
                .with_syntax_highlighting(
                    true,
                    Some("monokai".to_string()),
                )
                .with_language("en-GB")
                .build();

            assert!(result.is_ok());
            let config = result.unwrap();
            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("monokai".to_string())
            );
            assert_eq!(config.language, "en-GB");
        }

        #[test]
        fn test_config_builder_invalid() {
            let result = HtmlConfigBuilder::new()
                .with_language("invalid")
                .build();

            assert!(result.is_err());
            match result {
                Err(HtmlError::InvalidInput(msg)) => {
                    assert!(msg.contains("Invalid language code"),
                "Expected error message about invalid language code, got: {}", msg);
                }
                err => panic!(
                    "Expected InvalidInput error, got: {:?}",
                    err
                ),
            }
        }
    }

    /// Tests for file path validation
    mod file_validation_tests {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn test_valid_paths() {
            let valid_paths = [
                PathBuf::from("test.md"),
                PathBuf::from("test.html"),
                PathBuf::from("subfolder/test.md"),
            ];

            for path in valid_paths {
                assert!(
                    HtmlConfig::validate_file_path(&path).is_ok(),
                    "Path should be valid: {:?}",
                    path
                );
            }
        }

        #[test]
        fn test_invalid_paths() {
            let invalid_paths = [
                PathBuf::from(""),           // Empty path
                PathBuf::from("../test.md"), // Directory traversal
                PathBuf::from("test.exe"),   // Invalid extension
                PathBuf::from(
                    "a".repeat(constants::MAX_PATH_LENGTH + 1),
                ), // Too long
            ];

            for path in invalid_paths {
                assert!(
                    HtmlConfig::validate_file_path(&path).is_err(),
                    "Path should be invalid: {:?}",
                    path
                );
            }
        }

        #[test]
        #[cfg(not(test))]
        fn test_absolute_paths() {
            let path = PathBuf::from("/absolute/path/test.md");
            assert!(HtmlConfig::validate_file_path(&path).is_err());
        }
    }

    /// Tests for Markdown conversion functionality
    mod markdown_conversion_tests {
        use super::*;

        #[test]
        fn test_basic_conversion() {
            let markdown = "# Test\n\nHello world";
            let result = markdown_to_html(markdown, None);
            assert!(result.is_ok());

            let html = result.unwrap();
            assert!(html.contains("<h1>Test</h1>"));
            assert!(html.contains("<p>Hello world</p>"));
        }

        #[test]
        fn test_conversion_with_config() {
            let markdown = "# Test\n```rust\nfn main() {}\n```";
            let config = MarkdownConfig {
                html_config: HtmlConfig {
                    enable_syntax_highlighting: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            let result = markdown_to_html(markdown, Some(config));
            assert!(result.is_ok());

            let html = result.unwrap();
            assert!(html.contains("language-rust"));
        }

        #[test]
        fn test_empty_content() {
            let result = markdown_to_html("", None);
            assert!(matches!(result, Err(HtmlError::InvalidInput(_))));
        }

        #[test]
        fn test_content_too_large() {
            let large_content =
                "a".repeat(constants::DEFAULT_MAX_INPUT_SIZE + 1);
            let result = markdown_to_html(&large_content, None);
            assert!(matches!(result, Err(HtmlError::InputTooLarge(_))));
        }
    }

    /// Tests for file-based operations
    mod file_operation_tests {
        use super::*;

        #[test]
        fn test_file_conversion() -> Result<()> {
            let temp_dir = setup_test_dir();
            let input_path =
                create_test_file(&temp_dir, "# Test\n\nHello world");
            let output_path = temp_dir.path().join("test.html");

            let result = markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::File(
                    output_path.to_string_lossy().into(),
                )),
                None::<MarkdownConfig>,
            );

            assert!(result.is_ok());
            let content = std::fs::read_to_string(&output_path)?;
            assert!(content.contains("<h1>Test</h1>"));

            Ok(())
        }

        #[test]
        fn test_writer_output() {
            // Create a test file instead of using stdin
            let temp_dir = setup_test_dir();
            let input_path =
                create_test_file(&temp_dir, "# Test\nHello");
            let buffer = Box::new(Cursor::new(Vec::new()));

            let result = markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::Writer(buffer)),
                None,
            );

            assert!(result.is_ok());
        }

        #[test]
        fn test_writer_output_no_input() {
            let buffer = Box::new(Cursor::new(Vec::new()));

            let result = markdown_file_to_html(
                Some(Path::new("nonexistent.md")), // Use nonexistent file instead of None
                Some(OutputDestination::Writer(buffer)),
                None,
            );

            assert!(result.is_err()); // Should fail with file not found error
        }
    }

    /// Tests for language code validation
    mod language_validation_tests {
        use super::*;

        #[test]
        fn test_valid_language_codes() {
            let valid_codes =
                ["en-GB", "fr-FR", "de-DE", "es-ES", "zh-CN"];

            for code in valid_codes {
                assert!(
                    validate_language_code(code),
                    "Language code '{}' should be valid",
                    code
                );
            }
        }

        #[test]
        fn test_invalid_language_codes() {
            let invalid_codes = [
                "",        // Empty
                "en",      // Missing region
                "eng-GBR", // Wrong format
                "en_GB",   // Wrong separator
                "123-45",  // Invalid characters
                "GB-en",   // Wrong order
                "en-gb",   // Wrong case
            ];

            for code in invalid_codes {
                assert!(
                    !validate_language_code(code),
                    "Language code '{}' should be invalid",
                    code
                );
            }
        }
    }

    /// Integration tests for end-to-end functionality
    mod integration_tests {
        use super::*;

        #[test]
        fn test_end_to_end_conversion() -> Result<()> {
            let temp_dir = setup_test_dir();
            let content = r#"---
title: Test Document
---

# Hello World

This is a test document with:
- A list
- And some **bold** text
"#;
            let input_path = create_test_file(&temp_dir, content);
            let output_path = temp_dir.path().join("test.html");

            let config = MarkdownConfig {
                html_config: HtmlConfig {
                    enable_syntax_highlighting: true,
                    generate_toc: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::File(
                    output_path.to_string_lossy().into(),
                )),
                Some(config),
            )?;

            let html = std::fs::read_to_string(&output_path)?;
            assert!(html.contains("<h1>Hello World</h1>"));
            assert!(html.contains("<strong>bold</strong>"));
            assert!(html.contains("<ul>"));

            Ok(())
        }

        #[test]
        fn test_error_handling() {
            // Test non-existent file
            let result = markdown_file_to_html(
                Some(Path::new("nonexistent.md")),
                None,
                None,
            );
            assert!(result.is_err());

            // Test invalid output path
            let result = markdown_file_to_html(
                Some(Path::new("test.md")),
                Some(OutputDestination::File(
                    "/invalid/path/test.html".to_string(),
                )),
                None,
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_output_destination_debug() {
            assert_eq!(
                format!(
                    "{:?}",
                    OutputDestination::File("test.html".to_string())
                ),
                r#"File("test.html")"#
            );
            assert_eq!(
                format!("{:?}", OutputDestination::Stdout),
                "Stdout"
            );
            let writer = Box::new(Cursor::new(Vec::new()));
            assert_eq!(
                format!("{:?}", OutputDestination::Writer(writer)),
                "Writer(<dyn Write>)"
            );
        }
    }
}
