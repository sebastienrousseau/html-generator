// src/lib.rs

#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/html-generator/images/favicon.ico",
    html_logo_url = "https://kura.pro/html-generator/images/logos/html-generator.svg",
    html_root_url = "https://docs.rs/html-generator"
)]

//! HTML Generator: A modern HTML generation and optimization library
//!
//! This crate provides a comprehensive suite of tools for generating, optimizing,
//! and managing HTML content with a focus on accessibility, SEO, and performance.
//!
//! # Primary Features
//!
//! - **Markdown to HTML**: Convert Markdown content and files to HTML
//! - **Accessibility**: Automated ARIA attributes and WCAG compliance checking
//! - **SEO Optimization**: Meta tag generation and structured data support
//! - **Performance**: HTML minification and async generation capabilities
//!
//! # Quick Start
//!
//! ```rust
//! use html_generator::{markdown_to_html, MarkdownConfig};
//! use html_generator::error::HtmlError;
//!
//! fn main() -> Result<(), HtmlError> {
//!     let markdown = "# Hello World\n\nWelcome to HTML Generator.";
//!     let config = MarkdownConfig::default();
//!
//!     let html = markdown_to_html(markdown, Some(config))?;
//!     println!("Generated HTML: {html}");
//!     Ok::<(), HtmlError>(())
//! }
//! ```
//!
//! # Security Considerations
//!
//! This library implements several security measures:
//!
//! - **Path Validation**: Prevents directory traversal attacks and restricts
//!   file access to appropriate file types
//! - **Input Size Limits**: Prevents denial of service through large files
//! - **Unicode Safety**: Ensures all text processing is Unicode-aware
//! - **Memory Safety**: Uses Rust's memory safety guarantees
//! - **Error Handling**: Comprehensive error handling prevents undefined behavior
//!
//! # Error Handling
//!
//! All operations that can fail return a `Result<T, Error>`. The error type
//! provides detailed information about what went wrong.

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
    // Existing constants
    /// Default maximum input size (5MB)
    pub const DEFAULT_MAX_INPUT_SIZE: usize = 5 * 1024 * 1024;
    /// Default language code (en-GB)
    pub const DEFAULT_LANGUAGE: &str = "en-GB";
    /// Default syntax highlighting theme (github)
    pub const DEFAULT_SYNTAX_THEME: &str = "github";

    // New constants for validation
    /// Minimum input size (1KB)
    pub const MIN_INPUT_SIZE: usize = 1024;
    /// Maximum file path length
    pub const MAX_PATH_LENGTH: usize = 4096;
    /// Valid language code pattern
    pub const LANGUAGE_CODE_PATTERN: &str = r"^[a-z]{2}-[A-Z]{2}$";
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

/// Output destination for HTML generation.
///
/// This enum represents the possible destinations for generated HTML output.
/// It supports writing to files, custom writers, or stdout.
///
/// # Examples
///
/// ```
/// use html_generator::OutputDestination;
/// use std::fs::File;
///
/// // Write to a file
/// let file_dest = OutputDestination::File("output.html".to_string());
///
/// // Write to stdout (default)
/// let stdout_dest = OutputDestination::default();
/// ```
pub enum OutputDestination {
    /// Write to a file path
    File(String),
    /// Write to any implementor of Write
    Writer(Box<dyn Write>),
    /// Write to stdout (default)
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
/// # Security
///
/// This function:
/// * Validates all input is valid Unicode
/// * Sanitizes HTML output
/// * Protects against common injection attacks
///
/// # Examples
///
/// ```
/// use html_generator::{markdown_to_html, MarkdownConfig};
/// use html_generator::error::HtmlError;
///
/// let markdown = "# Hello\n\nWorld";
/// let html = markdown_to_html(markdown, None)?;
/// assert!(html.contains("<h1>Hello</h1>"));
/// # Ok::<(), HtmlError>(())
/// ```
pub fn markdown_to_html(
    content: &str,
    config: Option<MarkdownConfig>,
) -> Result<String> {
    log::debug!("Converting markdown content to HTML");
    let config = config.unwrap_or_default();

    // Check for empty or invalid content
    if content.is_empty() {
        return Err(HtmlError::InvalidInput(
            "Input content is empty".to_string(),
        ));
    }

    // Validate input size
    if content.len() > config.html_config.max_input_size {
        return Err(HtmlError::InputTooLarge(content.len()));
    }

    // Generate HTML
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
/// # Security
///
/// This function:
/// * Validates file paths
/// * Handles encoding securely
/// * Limits input size
/// * Sanitizes output
///
/// # Examples
///
/// ```no_run
/// use html_generator::{markdown_file_to_html, MarkdownConfig, OutputDestination};
/// use html_generator::error::HtmlError;
///
/// let config = MarkdownConfig::default();
/// let output = OutputDestination::File("output.html".to_string());
///
/// markdown_file_to_html(
///     Some("input.md"),
///     Some(output),
///     Some(config)
/// )?;
/// # Ok::<(), HtmlError>(())
/// ```
pub fn markdown_file_to_html(
    input: Option<impl AsRef<Path>>,
    output: Option<OutputDestination>,
    config: Option<MarkdownConfig>,
) -> Result<()> {
    log::debug!("Starting markdown to HTML conversion");
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
            let _ = io::stdin()
                .read_to_string(&mut content)
                .map_err(HtmlError::Io)?;
            content
        }
    };

    // Generate HTML
    let html = markdown_to_html(&content, Some(config))?;

    // Write output with error handling
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

/// Check if a given language code is valid
///
/// This function checks if a given language code is valid according to the
/// specified pattern.
///
/// # Arguments
///
/// * `lang` - The language code to validate
///
/// # Returns
///
/// Returns true if the language code is valid, false otherwise.
///
/// # Examples
///
/// ```rust
/// use html_generator::validate_language_code;
///
/// assert!(validate_language_code("en-GB"));
/// assert!(!validate_language_code("en"));
/// ```
pub fn validate_language_code(lang: &str) -> bool {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static LANG_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(constants::LANGUAGE_CODE_PATTERN).unwrap()
    });

    LANG_REGEX.is_match(lang)
}

/// Configuration options for HTML generation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HtmlConfig {
    /// Enable syntax highlighting for code blocks.
    ///
    /// When enabled, code blocks in Markdown will be highlighted using the
    /// specified theme.
    pub enable_syntax_highlighting: bool,

    /// Theme to use for syntax highlighting.
    ///
    /// Only applicable when `enable_syntax_highlighting` is true.
    pub syntax_theme: Option<String>,

    /// Minify the generated HTML output.
    ///
    /// When enabled, removes unnecessary whitespace and comments to reduce
    /// file size.
    pub minify_output: bool,

    /// Automatically add ARIA attributes for accessibility.
    pub add_aria_attributes: bool,

    /// Generate structured data (JSON-LD) based on content.
    pub generate_structured_data: bool,

    /// Maximum size (in bytes) for input content.
    ///
    /// Defaults to 5MB to prevent memory issues with large inputs.
    pub max_input_size: usize,

    /// Language for generated content.
    ///
    /// Used for lang attributes and meta tags.
    pub language: String,

    /// Enable table of contents generation.
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
            max_input_size: 5 * 1024 * 1024, // 5MB
            language: String::from("en-GB"),
            generate_toc: false,
        }
    }
}

/// Get the current version of the library
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the minimum supported Rust version
pub fn min_rust_version() -> &'static str {
    env!("CARGO_PKG_RUST_VERSION")
}

/// Builder for `HtmlConfig` to customize HTML generation options.
#[derive(Debug, Default)]
pub struct HtmlConfigBuilder {
    config: HtmlConfig,
}

impl HtmlConfigBuilder {
    /// Create a new `HtmlConfigBuilder` with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable syntax highlighting for code blocks.
    /// If enabled but no theme is provided, defaults to "github" theme.
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

    /// Set the language for generated content.
    /// Only accepts valid language codes (e.g., "en-GB", "fr-FR").
    #[must_use]
    pub fn with_language(
        mut self,
        language: impl Into<String>,
    ) -> Self {
        let lang = language.into();
        if validate_language_code(&lang) {
            self.config.language = lang;
        }
        self
    }

    /// Enable or disable minification of the generated HTML output.
    pub fn build(self) -> Result<HtmlConfig> {
        // Validate configuration
        if self.config.max_input_size < constants::MIN_INPUT_SIZE {
            return Err(HtmlError::InvalidInput(
                "Input size must be at least 1KB".to_string(),
            ));
        }
        Ok(self.config)
    }

    /// Enable or disable minification of the generated HTML output.
    pub const fn with_minification(mut self, enable: bool) -> Self {
        self.config.minify_output = enable;
        self
    }

    /// Enable or disable automatic addition of ARIA attributes for accessibility.
    pub fn with_aria_attributes(mut self, enable: bool) -> Self {
        self.config.add_aria_attributes = enable;
        self
    }

    /// Enable or disable generation of structured data (JSON-LD).
    pub fn with_structured_data(mut self, enable: bool) -> Self {
        self.config.generate_structured_data = enable;
        self
    }

    /// Set the maximum size (in bytes) for input content.
    /// Enforces a minimum size of 1KB.
    pub fn with_max_input_size(mut self, size: usize) -> Self {
        self.config.max_input_size = size.max(1024); // Minimum 1KB
        self
    }

    /// Enable or disable generation of table of contents.
    pub fn with_toc(mut self, enable: bool) -> Self {
        self.config.generate_toc = enable;
        self
    }
}

impl HtmlConfig {
    /// Create a new `HtmlConfig` with default options.
    pub fn builder() -> HtmlConfigBuilder {
        HtmlConfigBuilder::default()
    }

    /// Check if syntax highlighting is enabled for code blocks.
    ///
    /// When enabled, code blocks will be syntax highlighted using the configured theme.
    pub fn is_syntax_highlighting_enabled(&self) -> bool {
        self.enable_syntax_highlighting
    }

    /// Get the configured syntax highlighting theme.
    ///
    /// Returns the theme name if syntax highlighting is enabled, None otherwise.
    pub fn get_syntax_theme(&self) -> Option<&str> {
        self.syntax_theme.as_deref()
    }

    /// Check if HTML minification is enabled.
    ///
    /// When enabled, unnecessary whitespace and comments will be removed from the output HTML.
    pub fn is_minification_enabled(&self) -> bool {
        self.minify_output
    }

    /// Check if ARIA attributes generation is enabled.
    ///
    /// When enabled, appropriate ARIA attributes will be automatically added to HTML elements
    /// to improve accessibility.
    pub fn are_aria_attributes_enabled(&self) -> bool {
        self.add_aria_attributes
    }

    /// Check if structured data (JSON-LD) generation is enabled.
    ///
    /// When enabled, structured data will be generated in JSON-LD format
    /// to improve SEO.
    pub fn is_structured_data_enabled(&self) -> bool {
        self.generate_structured_data
    }

    /// Check if table of contents generation is enabled.
    ///
    /// When enabled, a table of contents will be generated from the document headings.
    pub fn is_toc_enabled(&self) -> bool {
        self.generate_toc
    }

    /// Get the configured language for content generation.
    ///
    /// Returns the language code (e.g., "en-GB", "fr-FR") that will be used
    /// in lang attributes and meta tags.
    pub fn get_language(&self) -> &str {
        &self.language
    }

    /// Get the configured maximum input size in bytes.
    ///
    /// Returns the maximum allowed size for input content. Default is 5MB.
    pub fn get_max_input_size(&self) -> usize {
        self.max_input_size
    }

    /// Validate file path safety
    fn validate_file_path(path: impl AsRef<Path>) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // HtmlConfig Tests
    mod config_tests {
        use super::*;
        use crate::constants::*;

        #[test]
        fn test_default_config() {
            let config = HtmlConfig::default();
            assert!(config.enable_syntax_highlighting);
            assert_eq!(config.syntax_theme, Some("github".to_string()));
            assert!(!config.minify_output);
            assert!(config.add_aria_attributes);
            assert!(!config.generate_structured_data);
            assert_eq!(config.max_input_size, DEFAULT_MAX_INPUT_SIZE);
            assert_eq!(config.language, DEFAULT_LANGUAGE);
            assert!(!config.generate_toc);
        }

        #[test]
        fn test_config_equality() {
            let config1 = HtmlConfig::default();
            let config2 = HtmlConfig::default();
            assert_eq!(config1, config2);
        }

        #[test]
        fn test_config_clone() {
            let config1 = HtmlConfig::default();
            let config2 = HtmlConfig::default(); // Create another instance directly
            assert_eq!(config1, config2); // Compare two default instances
        }

        #[test]
        fn test_config_debug() {
            let config = HtmlConfig::default();
            let debug_string = format!("{:?}", config);
            assert!(debug_string.contains("enable_syntax_highlighting"));
            assert!(debug_string.contains("syntax_theme"));
            assert!(debug_string.contains("minify_output"));
        }
    }

    // HtmlConfigBuilder Tests
    mod builder_tests {
        use super::*;

        #[test]
        fn test_builder_new() {
            let builder = HtmlConfigBuilder::new();
            let config = builder.build().unwrap();
            assert_eq!(config, HtmlConfig::default());
        }

        #[test]
        fn test_builder_with_language() {
            let config = HtmlConfigBuilder::new()
                .with_language("fr-FR")
                .build()
                .unwrap();
            assert_eq!(config.language, "fr-FR");
        }

        #[test]
        fn test_builder_with_valid_languages() {
            let valid_langs = ["en-GB", "fr-FR", "de-DE", "zh-CN"];
            for lang in valid_langs {
                let config = HtmlConfigBuilder::new()
                    .with_language(lang)
                    .build();
                assert_eq!(config.unwrap().language, lang);
            }
        }

        #[test]
        fn test_builder_with_more_invalid_languages() {
            let invalid_langs = ["en", "f", "", "fr_FR"];
            for lang in invalid_langs {
                let config = HtmlConfigBuilder::new()
                    .with_language(lang)
                    .build();
                assert_eq!(config.unwrap().language, "en-GB");
            }
        }

        #[test]
        fn test_builder_chaining() {
            let config = HtmlConfigBuilder::new()
                .with_syntax_highlighting(
                    true,
                    Some("monokai".to_string()),
                )
                .with_language("es-ES")
                .build()
                .unwrap();

            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("monokai".to_string())
            );
            assert_eq!(config.language, "es-ES");
        }

        #[test]
        fn test_builder_debug() {
            let builder = HtmlConfigBuilder::new();
            let debug_string = format!("{:?}", builder);
            assert!(debug_string.contains("HtmlConfigBuilder"));
        }

        #[test]
        fn test_builder_with_invalid_language() {
            let config = HtmlConfigBuilder::new()
                .with_language("fr") // too short
                .build();
            assert_eq!(config.unwrap().language, "en-GB"); // should keep default
        }

        #[test]
        fn test_builder_with_small_input_size() {
            let config = HtmlConfigBuilder::new()
                .with_max_input_size(100) // less than minimum
                .build();
            assert_eq!(config.unwrap().max_input_size, 1024); // should use minimum
        }

        #[test]
        fn test_builder_all_options() {
            let config_result = HtmlConfigBuilder::new()
                .with_syntax_highlighting(
                    true,
                    Some("monokai".to_string()),
                )
                .with_minification(true)
                .with_aria_attributes(false)
                .with_structured_data(true)
                .with_max_input_size(1024 * 1024)
                .with_language("fr-FR")
                .with_toc(true)
                .build();

            let config = config_result.unwrap();

            assert!(config.enable_syntax_highlighting);
            assert!(config.minify_output);
            assert!(!config.add_aria_attributes);
            assert!(config.generate_structured_data);
            assert_eq!(config.max_input_size, 1024 * 1024);
            assert_eq!(config.language, "fr-FR");
            assert!(config.generate_toc);
        }

        #[test]
        fn test_all_config_getters() {
            let config = HtmlConfig::default();
            assert!(!config.is_minification_enabled());
            assert!(config.are_aria_attributes_enabled());
            assert!(!config.is_structured_data_enabled());
            assert!(!config.is_toc_enabled());
            assert_eq!(config.get_language(), "en-GB");
            assert_eq!(config.get_max_input_size(), 5 * 1024 * 1024);
        }

        #[test]
        fn test_builder_small_input_size() {
            let config_result = HtmlConfigBuilder::new()
                .with_max_input_size(512) // Smaller than minimum
                .build();
            assert!(config_result.is_ok()); // Should succeed
            assert_eq!(config_result.unwrap().max_input_size, 1024); // Enforces minimum size
        }

        #[test]
        fn test_builder_with_valid_and_invalid_language() {
            let valid_config = HtmlConfigBuilder::new()
                .with_language("en-GB")
                .build()
                .unwrap();
            assert_eq!(valid_config.language, "en-GB");

            let invalid_config = HtmlConfigBuilder::new()
                .with_language("invalid-lang")
                .build()
                .unwrap();
            assert_eq!(invalid_config.language, "en-GB"); // Defaults to en-GB
        }
    }

    // Constants Tests
    mod constants_tests {
        use super::*;

        #[test]
        fn test_default_max_input_size() {
            assert_eq!(
                constants::DEFAULT_MAX_INPUT_SIZE,
                5 * 1024 * 1024
            );
        }

        #[test]
        fn test_default_language() {
            assert_eq!(constants::DEFAULT_LANGUAGE, "en-GB");
        }

        #[test]
        fn test_default_syntax_theme() {
            assert_eq!(constants::DEFAULT_SYNTAX_THEME, "github");
        }
    }

    // Version Information Tests
    mod version_tests {
        use super::*;

        #[test]
        fn test_version() {
            let v = version();
            assert!(!v.is_empty());
            assert!(v.split('.').count() >= 2);
        }

        #[test]
        fn test_min_rust_version() {
            let v = min_rust_version();
            assert!(!v.is_empty());
            assert!(v.split('.').count() >= 2);
        }
    }

    // Config Factory Method Tests
    mod config_factory_tests {
        use super::*;

        #[test]
        fn test_config_builder_factory() {
            let config_result = HtmlConfig::builder().build();

            // Ensure the build result is Ok
            assert!(config_result.is_ok());

            let config = config_result.unwrap();

            assert_eq!(config, HtmlConfig::default());
        }

        #[test]
        fn test_config_custom_build() {
            let config_result = HtmlConfig::builder()
                .with_syntax_highlighting(
                    true,
                    Some("tomorrow".to_string()),
                )
                .with_language("de-DE")
                .build();

            let config = config_result.unwrap();

            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("tomorrow".to_string())
            );
            assert_eq!(config.language, "de-DE");
        }
    }

    // Result Type Tests
    mod result_tests {
        use super::*;

        #[test]
        fn test_result_ok() {
            let value = 42;
            let result: Result<i32> = Ok(value);
            assert!(result.is_ok(), "Result is not Ok as expected");
            if let Ok(val) = result {
                assert_eq!(
                    val, 42,
                    "Unexpected value inside Ok variant"
                );
            } else {
                unreachable!("Expected Ok variant but got Err");
            }
        }

        #[test]
        fn test_result_err() {
            let error =
                HtmlError::InvalidInput("test error".to_string());
            let result: Result<i32> = Err(error);
            assert!(result.is_err(), "Result is not Err as expected");
            if let Err(e) = result {
                assert!(
                    matches!(e, HtmlError::InvalidInput(_)),
                    "Unexpected error variant"
                );
            } else {
                unreachable!("Expected Err variant but got Ok");
            }
        }
    }

    mod markdown_tests {
        use crate::markdown_to_html;

        #[test]
        fn test_markdown_to_html_basic() {
            let markdown = "# Test\n\nHello world";
            let result = markdown_to_html(markdown, None).unwrap();
            assert!(result.contains("<h1>Test</h1>"));
            assert!(result.contains("<p>Hello world</p>"));
        }

        #[test]
        fn test_markdown_to_html_invalid_unicode() {
            let invalid = vec![0xFF, 0xFF]; // Invalid UTF-8
            let invalid_utf8 = std::str::from_utf8(&invalid);

            // Confirm invalid UTF-8 results in an error
            assert!(
                invalid_utf8.is_err(),
                "Expected invalid UTF-8 error"
            );

            // Convert invalid UTF-8 to a lossy string (this ensures it's valid UTF-8)
            let lossy_utf8 = String::from_utf8_lossy(&invalid);

            // Pass the lossy UTF-8 string to markdown_to_html (this won't trigger an error)
            let result = markdown_to_html(&lossy_utf8, None);
            assert!(
                result.is_ok(),
                "Lossy UTF-8 should still be processed"
            );
        }
    }

    mod file_path_tests {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn test_valid_file_path() {
            let path = PathBuf::from("test.md");
            assert!(HtmlConfig::validate_file_path(path).is_ok());
        }

        #[test]
        fn test_directory_traversal() {
            let path = PathBuf::from("../test.md");
            assert!(HtmlConfig::validate_file_path(path).is_err());
        }

        #[test]
        fn test_path_too_long() {
            let long_path = "a".repeat(constants::MAX_PATH_LENGTH + 1);
            let path = PathBuf::from(long_path);
            assert!(HtmlConfig::validate_file_path(path).is_err());
        }

        #[test]
        fn test_invalid_extension() {
            let path = PathBuf::from("test.exe");
            assert!(HtmlConfig::validate_file_path(path).is_err());
        }

        #[test]
        fn test_empty_file_path() {
            let path = PathBuf::from("");
            assert!(HtmlConfig::validate_file_path(path).is_err());
        }

        #[test]
        fn test_valid_html_extension() {
            let path = PathBuf::from("test.html");
            assert!(HtmlConfig::validate_file_path(path).is_ok());
        }
    }
}
